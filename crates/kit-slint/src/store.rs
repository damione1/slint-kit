// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
//! `Store<T>`: one typed value shared by Rust code and any number of views.
//!
//! The pattern is "one Slint `global` = one store". The host owns a `Store<T>`
//! for a domain (session, settings, cart); the store projects its value into a
//! Slint global by subscribing a projection closure per window. Writes go through
//! the store, so every bound window shows the same value.
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::HashMap,
    marker::PhantomData,
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering},
};

/// Process-unique identity of a store, used to find it again on the UI thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct StoreId(u64);

static NEXT_STORE: AtomicU64 = AtomicU64::new(1);

impl StoreId {
    fn next() -> Self {
        Self(NEXT_STORE.fetch_add(1, Ordering::Relaxed))
    }
}

thread_local! {
    /// Stores alive on this thread. A store is bound to the thread that made it.
    static LIVE: RefCell<HashMap<StoreId, Weak<dyn Any>>> = RefCell::new(HashMap::new());
}

type Observer<T> = Rc<dyn Fn(&T)>;

struct Inner<T> {
    id: StoreId,
    value: RefCell<T>,
    observers: RefCell<Vec<(u64, Observer<T>)>>,
    next_observer: Cell<u64>,
    notifying: Cell<bool>,
    changed_while_notifying: Cell<bool>,
}

impl<T> Drop for Inner<T> {
    fn drop(&mut self) {
        let id = self.id;
        // The registry can already be gone while the thread shuts down.
        let _ = LIVE.try_with(|live| live.borrow_mut().remove(&id));
    }
}

/// Clears the notification flag even when an observer panics.
struct NotifyGuard<'a>(&'a Cell<bool>);
impl Drop for NotifyGuard<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

/// A typed value shared by Rust and the views that observe it.
///
/// A store lives on the UI thread and is cheap to clone: clones share one
/// value. `T` need not be `Send`. Other threads change it through
/// [`StoreRemote`], obtained from [`Store::remote`].
///
/// Observers run synchronously after each change and receive a snapshot. If an
/// observer changes the store, the change is applied at once and observers run
/// again with the latest value once the current pass ends, so no observer ends
/// on a stale value.
pub struct Store<T: Clone + 'static> {
    inner: Rc<Inner<T>>,
}

impl<T: Clone + 'static> Clone for Store<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Clone + 'static> Store<T> {
    /// Create a store holding `initial`.
    pub fn new(initial: T) -> Self {
        let inner = Rc::new(Inner {
            id: StoreId::next(),
            value: RefCell::new(initial),
            observers: RefCell::new(Vec::new()),
            next_observer: Cell::new(0),
            notifying: Cell::new(false),
            changed_while_notifying: Cell::new(false),
        });
        let registered: Weak<dyn Any> = Rc::downgrade(&inner) as Weak<dyn Any>;
        LIVE.with(|live| live.borrow_mut().insert(inner.id, registered));
        Self { inner }
    }

    /// A copy of the current value.
    pub fn get(&self) -> T {
        self.inner.value.borrow().clone()
    }

    /// Read the current value without copying it. `read` must not change this
    /// store.
    pub fn with<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        read(&self.inner.value.borrow())
    }

    /// Replace the value and notify observers.
    pub fn set(&self, value: T) {
        self.update(|current| *current = value);
    }

    /// Change the value in place and notify observers. `change` must not
    /// change this store.
    pub fn update(&self, change: impl FnOnce(&mut T)) {
        change(&mut self.inner.value.borrow_mut());
        self.inner.publish();
    }

    /// Call `observer` after every change. It is not called for the value the
    /// store already holds; use [`Store::observe`] for that. Dropping the
    /// returned [`Subscription`] stops the calls.
    #[must_use = "dropping the subscription stops the observer"]
    pub fn subscribe(&self, observer: impl Fn(&T) + 'static) -> Subscription<T> {
        let id = self.inner.next_observer.get();
        self.inner.next_observer.set(id + 1);
        self.inner
            .observers
            .borrow_mut()
            .push((id, Rc::new(observer)));
        Subscription {
            store: Rc::downgrade(&self.inner),
            id,
        }
    }

    /// Call `observer` now with the current value, then after every change.
    /// This is how a store projects into a window: the closure sets the
    /// properties of a Slint global.
    #[must_use = "dropping the subscription stops the observer"]
    pub fn observe(&self, observer: impl Fn(&T) + 'static) -> Subscription<T> {
        observer(&self.get());
        self.subscribe(observer)
    }

    /// A handle that other threads use to change this store.
    pub fn remote(&self) -> StoreRemote<T> {
        StoreRemote {
            id: self.inner.id,
            marker: PhantomData,
        }
    }
}

impl<T: Clone + 'static> Inner<T> {
    fn publish(&self) {
        if self.notifying.replace(true) {
            self.changed_while_notifying.set(true);
            return;
        }
        let _guard = NotifyGuard(&self.notifying);
        loop {
            self.changed_while_notifying.set(false);
            let snapshot = self.value.borrow().clone();
            let observers: Vec<Observer<T>> = self
                .observers
                .borrow()
                .iter()
                .map(|(_, observer)| observer.clone())
                .collect();
            for observer in observers {
                observer(&snapshot);
            }
            if !self.changed_while_notifying.get() {
                break;
            }
        }
    }
}

/// Keeps an observer registered. Dropping it unregisters the observer.
pub struct Subscription<T: Clone + 'static> {
    store: Weak<Inner<T>>,
    id: u64,
}

impl<T: Clone + 'static> Drop for Subscription<T> {
    fn drop(&mut self) {
        if let Some(store) = self.store.upgrade() {
            store
                .observers
                .borrow_mut()
                .retain(|(id, _)| *id != self.id);
        }
    }
}

/// The event loop could not accept a change from another thread.
#[derive(Debug)]
pub enum StoreError {
    /// Slint has no running event loop, or it has ended.
    EventLoopUnavailable(slint::EventLoopError),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventLoopUnavailable(error) => {
                write!(f, "the store change could not be queued: {error}")
            }
        }
    }
}

impl std::error::Error for StoreError {}

/// A `Send` handle that changes a [`Store`] from any thread.
///
/// Each change is queued with `slint::invoke_from_event_loop` and applied on the
/// UI thread, where observers run. A change queued for a store that has since
/// been dropped is discarded.
pub struct StoreRemote<T: Clone + 'static> {
    id: StoreId,
    marker: PhantomData<fn() -> T>,
}

impl<T: Clone + 'static> Clone for StoreRemote<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            marker: PhantomData,
        }
    }
}

impl<T: Clone + 'static> StoreRemote<T> {
    /// Queue a replacement of the value.
    pub fn set(&self, value: T) -> Result<(), StoreError>
    where
        T: Send,
    {
        self.update(move |current| *current = value)
    }

    /// Queue an in-place change of the value. `change` runs on the UI thread.
    pub fn update(&self, change: impl FnOnce(&mut T) + Send + 'static) -> Result<(), StoreError> {
        let id = self.id;
        slint::invoke_from_event_loop(move || {
            let store = LIVE.with(|live| live.borrow().get(&id).and_then(Weak::upgrade));
            if let Some(inner) = store.and_then(|any| any.downcast::<Inner<T>>().ok()) {
                Store { inner }.update(change);
            }
        })
        .map_err(StoreError::EventLoopUnavailable)
    }
}

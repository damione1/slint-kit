// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::{KeyedModel, ui};
use kit_core::{
    ToastCloseReason, ToastClosed, ToastId, ToastLifetime, ToastPause, ToastQueue, ToastQueueError,
};
use slint::Model;
use std::{cell::RefCell, collections::HashSet, num::NonZeroUsize, rc::Rc, time::Duration};
use web_time::Instant;
type CloseHandler<T> = Rc<dyn Fn(ToastId, T, ToastCloseReason)>;
struct Inner<T> {
    on_closed: RefCell<Option<CloseHandler<T>>>,
    queue: RefCell<ToastQueue<(T, ui::ToastEntry)>>,
    rows: KeyedModel<ToastId, ui::ToastEntry>,
    epoch: Instant,
    timer: slint::Timer,
}
/// Typed toast actions and lifetimes, with independent pointer/focus pauses.
/// One single-shot timer targets the next running lifetime; idle, persistent
/// and fully paused queues have no active timer. Its weak state reference
/// never retains a window.
pub struct ToastBinding<T> {
    inner: Rc<Inner<T>>,
}
impl<T> Clone for ToastBinding<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<T: Clone + 'static> ToastBinding<T> {
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            inner: Rc::new(Inner {
                queue: RefCell::new(ToastQueue::new(capacity)),
                on_closed: RefCell::new(None),
                rows: KeyedModel::new([]).expect("empty toast model"),
                epoch: Instant::now(),
                timer: slint::Timer::default(),
            }),
        }
    }
    fn schedule(&self) {
        let Some(delay) = self
            .inner
            .queue
            .borrow()
            .next_timeout(self.inner.epoch.elapsed())
        else {
            self.inner.timer.stop();
            return;
        };
        // Slint timers have millisecond resolution. A positive remainder
        // must not truncate to a zero-delay spin before the clock catches up.
        let delay = if delay.is_zero() {
            delay
        } else {
            delay.max(Duration::from_millis(1))
        };
        let weak = Rc::downgrade(&self.inner);
        self.inner
            .timer
            .start(slint::TimerMode::SingleShot, delay, move || {
                if let Some(inner) = weak.upgrade() {
                    let expired = inner.queue.borrow_mut().advance(inner.epoch.elapsed());
                    let binding = Self { inner };
                    binding.finish(expired);
                }
            });
    }
    pub fn model(&self) -> slint::ModelRc<ui::ToastEntry> {
        self.inner.rows.model()
    }
    /// Replace this binding's closure observer. Capture UI windows weakly.
    /// The model and timer are current and no queue borrow is held when called.
    pub fn on_closed(&self, handler: impl Fn(ToastId, T, ToastCloseReason) + 'static) {
        *self.inner.on_closed.borrow_mut() = Some(Rc::new(handler));
    }
    pub fn len(&self) -> usize {
        self.inner.queue.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.queue.borrow().is_empty()
    }
    fn finish(&self, closed: Vec<ToastClosed<(T, ui::ToastEntry)>>) {
        self.refresh();
        self.schedule();
        let handler = self.inner.on_closed.borrow().clone();
        if let Some(handler) = handler {
            for record in closed {
                handler(record.id, record.value.0, record.reason);
            }
        }
    }
    fn refresh(&self) {
        let ids = self
            .inner
            .queue
            .borrow()
            .entries()
            .map(|entry| entry.id)
            .collect::<Vec<_>>();
        let retained: HashSet<_> = ids.iter().copied().collect();
        let removed = self
            .inner
            .rows
            .model()
            .iter()
            .filter_map(|row| self.inner.rows.key(row.token))
            .filter(|id| !retained.contains(id))
            .collect::<Vec<_>>();
        for id in removed {
            self.inner.rows.remove_by_key(&id);
        }
        for id in ids {
            // Publications can re-enter this binding. Read each current value
            // after the preceding publication, without keeping the queue borrowed.
            let row = self
                .inner
                .queue
                .borrow()
                .entries()
                .find(|entry| entry.id == id)
                .map(|entry| entry.value.1.clone());
            let Some(mut row) = row else {
                continue;
            };
            if let Some(current) = self.inner.rows.get(&id) {
                row.token = current.token;
                if current != row {
                    self.inner.rows.update_by_key(&id, |current| *current = row);
                }
            } else {
                self.inner
                    .rows
                    .insert(id, row)
                    .expect("toast IDs are unique");
            }
        }
    }
    pub fn push(
        &self,
        action: T,
        view: ui::ToastEntry,
        lifetime: ToastLifetime,
    ) -> Result<ToastId, ToastQueueError> {
        let change = self.inner.queue.borrow_mut().push(
            (action, view),
            lifetime,
            self.inner.epoch.elapsed(),
        )?;
        let id = change.result;
        self.finish(change.closed);
        Ok(id)
    }
    fn close_id(&self, id: ToastId, reason: ToastCloseReason) -> Option<T> {
        let (mut closed, value) = {
            let mut queue = self.inner.queue.borrow_mut();
            let closed = queue.advance(self.inner.epoch.elapsed());
            (closed, queue.remove(id))
        };
        if let Some(value) = &value {
            closed.push(ToastClosed {
                id,
                value: value.clone(),
                reason,
            });
        }
        self.finish(closed);
        value.map(|(action, _)| action)
    }
    pub fn dismiss(&self, token: i32) -> Option<T> {
        let id = self.inner.rows.key(token)?;
        self.dismiss_id(id)
    }
    pub fn dismiss_id(&self, id: ToastId) -> Option<T> {
        self.close_id(id, ToastCloseReason::Dismissed)
    }
    pub fn clear(&self) {
        let closed = self
            .inner
            .queue
            .borrow_mut()
            .clear(self.inner.epoch.elapsed());
        self.finish(closed);
    }
    /// Keep the action, identity and pause flags, and restart the live lifetime.
    /// An expired or missing ID cannot be revived.
    pub fn update(&self, id: ToastId, view: ui::ToastEntry, lifetime: ToastLifetime) -> bool {
        let change = self.inner.queue.borrow_mut().update(
            id,
            lifetime,
            self.inner.epoch.elapsed(),
            |value| value.1 = view,
        );
        let updated = change.result;
        self.finish(change.closed);
        updated
    }
    pub fn action_handler(&self, handler: impl Fn(T) + 'static) -> impl Fn(i32) + 'static {
        let binding = self.clone();
        move |token| {
            let Some(id) = binding.inner.rows.key(token) else {
                return;
            };
            if let Some(action) = binding.close_id(id, ToastCloseReason::Action) {
                handler(action);
            }
        }
    }
    pub fn dismiss_handler(&self) -> impl Fn(i32) + 'static {
        let binding = self.clone();
        move |token| {
            binding.dismiss(token);
        }
    }
    pub fn interaction_handler(&self) -> impl Fn(i32, ui::ToastInteraction, bool) + 'static {
        let binding = self.clone();
        move |token, source, engaged| {
            let Some(id) = binding.inner.rows.key(token) else {
                return;
            };
            let source = match source {
                ui::ToastInteraction::Pointer => ToastPause::Pointer,
                ui::ToastInteraction::Focus => ToastPause::Focus,
            };
            let change = binding.inner.queue.borrow_mut().set_paused(
                id,
                source,
                engaged,
                binding.inner.epoch.elapsed(),
            );
            binding.finish(change.closed);
        }
    }
}

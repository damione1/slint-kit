// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
//! Asynchronous work that ends on the UI thread.
//!
//! Slint updates a window only from its event loop. [`spawn`] runs a future on
//! that loop and hands its result to a closure together with the window, already
//! upgraded, so application code holds no `as_weak` and no `upgrade`. [`Latest`]
//! adds the guard against stale responses: when a newer request starts, the
//! result of the older one is dropped.
//!
//! The kit imposes no asynchronous runtime. The future is polled by Slint's
//! event loop through `slint::spawn_local`, so it is not `Send`. A future that
//! needs a runtime reactor (tokio timers or sockets, for example) must bring it
//! along, as in the Slint documentation: wrap the future with `async_compat`, or
//! await the `JoinHandle` of a task that your own runtime runs.
use kit_core::{Fence, Loading};
use slint::{ComponentHandle, JoinHandle};
use std::{cell::RefCell, future::Future, rc::Rc};

/// The event loop could not accept the task.
#[derive(Debug)]
pub enum SpawnError {
    /// Slint has no event loop on this thread, or it has ended. Call `spawn`
    /// from the UI thread, for example from a callback.
    EventLoopUnavailable(slint::EventLoopError),
}

impl std::fmt::Display for SpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventLoopUnavailable(error) => {
                write!(f, "the task could not be started: {error}")
            }
        }
    }
}

impl std::error::Error for SpawnError {}

fn start<T: 'static>(
    future: impl Future<Output = T> + 'static,
) -> Result<JoinHandle<T>, SpawnError> {
    slint::spawn_local(future).map_err(SpawnError::EventLoopUnavailable)
}

/// Run `future` on the event loop, then call `apply` with the window and the
/// result.
///
/// `apply` runs on the UI thread. If the window was destroyed in the meantime,
/// the result is dropped and `apply` does not run. Every call delivers its own
/// result; use [`Latest`] when only the newest request may deliver.
pub fn spawn<C, T>(
    app: &C,
    future: impl Future<Output = T> + 'static,
    apply: impl FnOnce(&C, T) + 'static,
) -> Result<(), SpawnError>
where
    C: ComponentHandle + 'static,
    T: 'static,
{
    let weak = app.as_weak();
    start(async move {
        let result = future.await;
        if let Some(app) = weak.upgrade() {
            apply(&app, result);
        }
    })
    .map(drop)
}

struct LatestInner {
    fence: Rc<RefCell<Fence>>,
    running: RefCell<Option<JoinHandle<()>>>,
}

impl LatestInner {
    fn abort_running(&self) {
        if let Some(handle) = self.running.borrow_mut().take() {
            handle.abort();
        }
    }
}

impl Drop for LatestInner {
    fn drop(&mut self) {
        self.abort_running();
    }
}

/// Starts requests so that only the newest one delivers its result.
///
/// Each [`Latest::spawn`] or [`Latest::load`] makes every earlier request stale:
/// its future is dropped at its next wake-up and its result is never applied,
/// whatever the order in which responses arrive. Create one `Latest` per source
/// of requests (a search box, a list that reloads) and keep it as long as the
/// window. Clones share the same guard. Dropping the last clone cancels the
/// request in flight.
#[derive(Clone)]
pub struct Latest {
    inner: Rc<LatestInner>,
}

impl Default for Latest {
    fn default() -> Self {
        Self::new()
    }
}

impl Latest {
    /// A guard with no request in flight.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(LatestInner {
                fence: Rc::new(RefCell::new(Fence::default())),
                running: RefCell::new(None),
            }),
        }
    }

    /// Run `future` and call `apply` with the window and the result, unless a
    /// newer request started or [`Latest::cancel`] ran first. If the event loop
    /// refuses the task, the earlier requests are still made stale.
    pub fn spawn<C, T>(
        &self,
        app: &C,
        future: impl Future<Output = T> + 'static,
        apply: impl FnOnce(&C, T) + 'static,
    ) -> Result<(), SpawnError>
    where
        C: ComponentHandle + 'static,
        T: 'static,
    {
        let ticket = self.inner.fence.borrow_mut().begin();
        self.inner.abort_running();
        let weak = app.as_weak();
        let fence = self.inner.fence.clone();
        let handle = start(async move {
            let result = future.await;
            if !fence.borrow().is_current(ticket) {
                return;
            }
            if let Some(app) = weak.upgrade() {
                apply(&app, result);
            }
        })?;
        *self.inner.running.borrow_mut() = Some(handle);
        Ok(())
    }

    /// Load a value and report its progress as [`Loading`].
    ///
    /// `apply` first receives [`Loading::Loading`], at once, then the outcome of
    /// `future` as [`Loading::Ready`] or [`Loading::Failed`]. A request that is
    /// stale by then reports nothing. `apply` typically projects the state into a
    /// component's properties or a [`Store`](crate::Store).
    pub fn load<C, T, E>(
        &self,
        app: &C,
        future: impl Future<Output = Result<T, E>> + 'static,
        apply: impl Fn(&C, Loading<T, E>) + 'static,
    ) -> Result<(), SpawnError>
    where
        C: ComponentHandle + 'static,
        T: 'static,
        E: 'static,
    {
        let apply = Rc::new(apply);
        let on_done = apply.clone();
        self.spawn(app, future, move |app, result| on_done(app, result.into()))?;
        apply(app, Loading::Loading);
        Ok(())
    }

    /// Discard the request in flight, if any. Its result is never applied.
    pub fn cancel(&self) {
        self.inner.fence.borrow_mut().invalidate();
        self.inner.abort_running();
    }
}

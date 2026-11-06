// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::{KeyedModel, ui};
use kit_core::{ToastId, ToastLifetime, ToastPause, ToastQueue, ToastQueueError};
use std::{cell::RefCell, num::NonZeroUsize, rc::Rc, time::Duration};
use web_time::Instant;
struct Inner<T> {
    queue: RefCell<ToastQueue<(T, ui::ToastEntry)>>,
    rows: KeyedModel<ToastId, ui::ToastEntry>,
    epoch: Instant,
    timer: slint::Timer,
}
/// Typed toast actions and lifetimes, with independent pointer/focus pauses.
/// The timer owns a weak state reference and never retains a window.
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
        let result = Self {
            inner: Rc::new(Inner {
                queue: RefCell::new(ToastQueue::new(capacity)),
                rows: KeyedModel::new([]).expect("empty toast model"),
                epoch: Instant::now(),
                timer: slint::Timer::default(),
            }),
        };
        let weak = Rc::downgrade(&result.inner);
        result.inner.timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(50),
            move || {
                if let Some(inner) = weak.upgrade() {
                    let expired = inner.queue.borrow_mut().advance(inner.epoch.elapsed());
                    if !expired.is_empty() {
                        Self { inner }.refresh();
                    }
                }
            },
        );
        result
    }
    pub fn model(&self) -> slint::ModelRc<ui::ToastEntry> {
        self.inner.rows.model()
    }
    fn refresh(&self) {
        let rows = self
            .inner
            .queue
            .borrow()
            .entries()
            .map(|entry| (entry.id, entry.value.1.clone()))
            .collect::<Vec<_>>();
        self.inner.rows.replace(rows).expect("toast IDs are unique");
    }
    pub fn push(
        &self,
        action: T,
        view: ui::ToastEntry,
        lifetime: ToastLifetime,
    ) -> Result<ToastId, ToastQueueError> {
        let id = self.inner.queue.borrow_mut().push(
            (action, view),
            lifetime,
            self.inner.epoch.elapsed(),
        )?;
        self.refresh();
        Ok(id)
    }
    pub fn dismiss(&self, token: i32) -> Option<T> {
        let id = self.inner.rows.key(token)?;
        let value = self
            .inner
            .queue
            .borrow_mut()
            .remove(id)
            .map(|(action, _)| action);
        self.refresh();
        value
    }
    pub fn action_handler(&self, handler: impl Fn(T) + 'static) -> impl Fn(i32) + 'static {
        let binding = self.clone();
        move |token| {
            if let Some(action) = binding.dismiss(token) {
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
            let changed = {
                let mut queue = binding.inner.queue.borrow_mut();
                let before = queue.entries().count();
                queue.set_paused(id, source, engaged, binding.inner.epoch.elapsed());
                before != queue.entries().count()
            };
            if changed {
                binding.refresh();
            }
        }
    }
}

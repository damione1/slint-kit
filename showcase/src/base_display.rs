// SPDX-License-Identifier: GPL-3.0-or-later
//! Application-owned lifetimes for the display gallery's interactive examples.
use super::ui;
use kit::{ToastId, ToastLifetime, ToastPause, ToastQueue};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{
    cell::RefCell,
    num::NonZeroUsize,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
enum Notification {
    Published,
    Failed,
}
struct Examples {
    queue: ToastQueue<Notification>,
    clock: Instant,
    job_started: Option<Instant>,
    progress: u16,
    job_state: ui::ProgressDemoState,
    model: Rc<VecModel<ui::ToastEntry>>,
    timer: slint::Timer,
}
impl Examples {
    fn new() -> Self {
        Self {
            queue: ToastQueue::new(NonZeroUsize::new(3).expect("nonzero notification capacity")),
            clock: Instant::now(),
            job_started: None,
            progress: 0,
            job_state: ui::ProgressDemoState::Idle,
            model: Rc::new(VecModel::default()),
            timer: slint::Timer::default(),
        }
    }
    fn id(&self, token: i32) -> Option<ToastId> {
        self.queue
            .entries()
            .find(|entry| i32::try_from(entry.id.get()).ok() == Some(token))
            .map(|entry| entry.id)
    }
    fn rows(&self) -> Vec<ui::ToastEntry> {
        self.queue
            .entries()
            .filter_map(|entry| {
                let token = i32::try_from(entry.id.get()).ok()?;
                let (title, message, action_label, severity) = match entry.value {
                    Notification::Published => (
                        "Update published",
                        "Your collection is ready to share.",
                        "Undo",
                        ui::AlertSeverity::Success,
                    ),
                    Notification::Failed => (
                        "Could not publish",
                        "The draft is safe. Try again when you are ready.",
                        "Retry",
                        ui::AlertSeverity::Error,
                    ),
                };
                Some(ui::ToastEntry {
                    token,
                    title: title.into(),
                    message: message.into(),
                    action_label: action_label.into(),
                    severity,
                    dismissible: true,
                })
            })
            .collect()
    }
}

// Project before mutating UI properties: focus/hover callbacks may re-enter the
// application during model changes. Retain unchanged delegates and their focus.
fn publish(state: &Rc<RefCell<Examples>>, window: &ui::Showcase) {
    let (progress, job_state, model, rows) = {
        let state = state.borrow();
        (
            state.progress,
            state.job_state,
            Rc::clone(&state.model),
            state.rows(),
        )
    };
    let global = window.global::<ui::DisplayExamples>();
    global.set_progress(f32::from(progress) / 100.0);
    global.set_progress_state(job_state);
    for index in (0..model.row_count()).rev() {
        if let Some(row) = model.row_data(index)
            && !rows.iter().any(|wanted| wanted.token == row.token)
        {
            model.remove(index);
        }
    }
    for row in rows {
        if !(0..model.row_count()).any(|index| {
            model
                .row_data(index)
                .is_some_and(|present| present.token == row.token)
        }) {
            model.push(row);
        }
    }
}

fn arm(state: &Rc<RefCell<Examples>>, window: slint::Weak<ui::Showcase>) {
    if state.borrow().timer.running() {
        return;
    }
    let weak_state = Rc::downgrade(state);
    state.borrow().timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
            let (Some(state), Some(window)) = (weak_state.upgrade(), window.upgrade()) else {
                return;
            };
            let changed = {
                let mut examples = state.borrow_mut();
                let now = examples.clock.elapsed();
                let expired = examples.queue.advance(now);
                let mut changed = !expired.is_empty();
                if let Some(started) = examples.job_started {
                    let progress = u16::try_from((started.elapsed().as_millis() / 50).min(100))
                        .expect("bounded progress");
                    changed |= progress != examples.progress;
                    examples.progress = progress;
                    if progress == 100 {
                        examples.job_started = None;
                        examples.job_state = ui::ProgressDemoState::Completed;
                    }
                }
                if examples.job_started.is_none()
                    && examples
                        .queue
                        .entries()
                        .all(|entry| matches!(entry.value, Notification::Failed))
                {
                    examples.timer.stop();
                }
                changed
            };
            if changed {
                publish(&state, &window);
            }
        },
    );
}

pub fn install(window: &ui::Showcase) {
    let state = Rc::new(RefCell::new(Examples::new()));
    let global = window.global::<ui::DisplayExamples>();
    global.set_toast_entries(ModelRc::from(Rc::clone(&state.borrow().model)));
    publish(&state, window);
    let weak = window.as_weak();
    let examples = Rc::clone(&state);
    global.on_add_toast(move |kind| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let value = match kind {
            ui::ToastExample::Published => Notification::Published,
            ui::ToastExample::Failed => Notification::Failed,
        };
        let lifetime = match value {
            Notification::Published => ToastLifetime::Timed(Duration::from_secs(8)),
            Notification::Failed => ToastLifetime::Persistent,
        };
        let failed = {
            let mut state = examples.borrow_mut();
            let now = state.clock.elapsed();
            state.queue.push(value, lifetime, now).is_err()
        };
        if failed {
            window
                .global::<ui::DisplayExamples>()
                .set_toast_status("Could not create another notification.".into());
            return;
        }
        publish(&examples, &window);
        arm(&examples, window.as_weak());
    });
    let weak = window.as_weak();
    let examples = Rc::clone(&state);
    global.on_toast_dismiss(move |token| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        {
            let mut state = examples.borrow_mut();
            let Some(id) = state.id(token) else {
                return;
            };
            state.queue.remove(id);
        }
        publish(&examples, &window);
    });
    let weak = window.as_weak();
    let examples = Rc::clone(&state);
    global.on_toast_action(move |token| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let status = {
            let mut state = examples.borrow_mut();
            let Some(id) = state.id(token) else {
                return;
            };
            match state.queue.remove(id) {
                Some(Notification::Published) => "Publication undone. Your draft is ready to edit.",
                Some(Notification::Failed) => {
                    let now = state.clock.elapsed();
                    if state
                        .queue
                        .push(
                            Notification::Published,
                            ToastLifetime::Timed(Duration::from_secs(8)),
                            now,
                        )
                        .is_err()
                    {
                        "Could not create another notification."
                    } else {
                        "Published successfully."
                    }
                }
                None => return,
            }
        };
        window
            .global::<ui::DisplayExamples>()
            .set_toast_status(status.into());
        publish(&examples, &window);
        arm(&examples, window.as_weak());
    });
    let examples = Rc::clone(&state);
    global.on_toast_interaction(move |token, source, engaged| {
        let mut state = examples.borrow_mut();
        let Some(id) = state.id(token) else {
            return;
        };
        let now = state.clock.elapsed();
        state.queue.set_paused(
            id,
            match source {
                ui::ToastInteraction::Pointer => ToastPause::Pointer,
                ui::ToastInteraction::Focus => ToastPause::Focus,
            },
            engaged,
            now,
        );
    });
    let weak = window.as_weak();
    let examples = Rc::clone(&state);
    global.on_progress_start(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        {
            let mut state = examples.borrow_mut();
            state.job_started = Some(Instant::now());
            state.progress = 0;
            state.job_state = ui::ProgressDemoState::Running;
        }
        publish(&examples, &window);
        arm(&examples, window.as_weak());
    });
    let weak = window.as_weak();
    global.on_progress_cancel(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        {
            let mut examples = state.borrow_mut();
            if examples.job_started.take().is_some() {
                examples.job_state = ui::ProgressDemoState::Cancelled;
            }
        }
        publish(&state, &window);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_actions_reject_evicted_ids_and_retry_with_a_new_identity() {
        i_slint_backend_testing::init_no_event_loop();
        let window = ui::Showcase::new().unwrap();
        install(&window);
        let global = window.global::<ui::DisplayExamples>();
        global.invoke_add_toast(ui::ToastExample::Published);
        let first = global.get_toast_entries().row_data(0).unwrap().token;
        global.invoke_add_toast(ui::ToastExample::Published);
        global.invoke_add_toast(ui::ToastExample::Published);
        global.invoke_add_toast(ui::ToastExample::Failed);
        assert_eq!(global.get_toast_entries().row_count(), 3);
        global.invoke_toast_action(first);
        global.invoke_toast_dismiss(first);
        assert_eq!(global.get_toast_entries().row_count(), 3);
        let failed = global.get_toast_entries().row_data(2).unwrap();
        assert_eq!(failed.severity, ui::AlertSeverity::Error);
        global.invoke_toast_action(failed.token);
        let retried = global.get_toast_entries().row_data(2).unwrap();
        assert_ne!(retried.token, failed.token);
        assert_eq!(retried.severity, ui::AlertSeverity::Success);
        global.invoke_toast_action(failed.token);
        assert_eq!(global.get_toast_entries().row_count(), 3);
        global.invoke_toast_action(retried.token);
        assert_eq!(global.get_toast_entries().row_count(), 2);
        assert!(global.get_toast_status().contains("undone"));
    }
}

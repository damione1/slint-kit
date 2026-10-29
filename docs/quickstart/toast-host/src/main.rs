// SPDX-License-Identifier: GPL-3.0-or-later
use web_time::Instant;
use slint::{ComponentHandle, ModelRc, VecModel};
use slint_kit::{ToastQueue, ToastLifetime, ToastPause};
use std::{cell::RefCell, rc::Rc, num::NonZeroUsize};
mod ui { slint::include_modules!(); }
fn publish(app: &ui::App, queue: &ToastQueue<()>) {
    let entries = queue.entries().map(|entry| ui::ToastEntry { token: i32::try_from(entry.id.get()).expect("view token capacity"), title: "Project archived".into(), message: "Your project can be restored.".into(), action_label: "Undo".into(), severity: ui::AlertSeverity::Success, dismissible: true }).collect::<Vec<_>>();
    app.set_entries(ModelRc::new(VecModel::from(entries)));
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let epoch = Instant::now();
    let queue = Rc::new(RefCell::new(ToastQueue::new(NonZeroUsize::new(3).unwrap())));
    queue.borrow_mut().push((), ToastLifetime::Persistent, epoch.elapsed()).expect("toast identity");
    publish(&app, &queue.borrow());
    let state = queue.clone(); let weak = app.as_weak();
    app.on_add(move || { state.borrow_mut().push((), ToastLifetime::Persistent, epoch.elapsed()).expect("toast identity"); if let Some(app) = weak.upgrade() { publish(&app, &state.borrow()); app.set_result("Project archived".into()); } });
    let state = queue.clone(); let weak = app.as_weak();
    app.on_undo(move |token| {
        let id = state.borrow().entries().find(|entry| i32::try_from(entry.id.get()).ok() == Some(token)).map(|entry| entry.id);
        if let Some(id) = id { state.borrow_mut().remove(id); if let Some(app) = weak.upgrade() { publish(&app, &state.borrow()); app.set_result("Archive undone".into()); } }
    });
    let state = queue.clone(); let weak = app.as_weak();
    app.on_dismiss_toast(move |token| {
        let id = state.borrow().entries().find(|entry| i32::try_from(entry.id.get()).ok() == Some(token)).map(|entry| entry.id);
        if let Some(id) = id { state.borrow_mut().remove(id); if let Some(app) = weak.upgrade() { publish(&app, &state.borrow()); app.set_result("Toast dismissed".into()); } }
    });
    app.on_interaction(move |token, source, engaged| {
        let id = queue.borrow().entries().find(|entry| i32::try_from(entry.id.get()).ok() == Some(token)).map(|entry| entry.id);
        if let Some(id) = id { queue.borrow_mut().set_paused(id, match source { ui::ToastInteraction::Pointer => ToastPause::Pointer, ui::ToastInteraction::Focus => ToastPause::Focus }, engaged, epoch.elapsed()); }
    });
    app.run()
}

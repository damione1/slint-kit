// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{ToastBinding, ToastLifetime, event, ui};
slint::include_modules!();
#[derive(Clone, Debug)] enum Undo { Archive }
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?; slint_kit::install(&app);
    let toasts = ToastBinding::new(std::num::NonZeroUsize::new(3).unwrap()); app.set_entries(toasts.model());
    app.on_undo(toasts.action_handler(event(&app, |app, action| { println!("Undo: {action:?}"); app.set_result("Project restored".into()); })));
    app.on_dismiss_toast(toasts.dismiss_handler());
    app.on_interaction(toasts.interaction_handler());
    app.on_add(move || { toasts.push(Undo::Archive, ui::ToastEntry {title: "Project archived".into(), message: "Your project can be restored.".into(), action_label: "Undo".into(), dismissible: true, severity: ui::AlertSeverity::Success, ..Default::default()}, ToastLifetime::Persistent).unwrap(); });
    app.invoke_add();
    app.run()
}

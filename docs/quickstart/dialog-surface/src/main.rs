// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_open(slint_kit::action(&app, |app| { app.set_showing(true); app.invoke_focus_dialog(); }));
    app.on_dismiss_requested(slint_kit::event(&app, |app, _reason| { app.set_showing(false); app.invoke_focus_opener(); }));
    app.on_archive(slint_kit::action(&app, |app| {
        app.set_result("Project archived".into());
        app.invoke_dismiss_requested(slint_kit::ui::CloseReason::Programmatic);
    }));
    app.run()
}

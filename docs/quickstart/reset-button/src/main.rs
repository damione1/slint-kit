// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_reset_draft(slint_kit::action(&app, |app| {
        app.set_draft("Field notes".into());
        app.set_result("Draft reset to Field notes".into());
    }));
    app.run()
}

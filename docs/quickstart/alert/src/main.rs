// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_dismiss(slint_kit::action(&app, |app| app.set_showing(false)));
    app.on_show(slint_kit::action(&app, |app| app.set_showing(true)));
    app.run()
}

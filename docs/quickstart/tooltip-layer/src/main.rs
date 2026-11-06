// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_create(slint_kit::action(&app, |app| {
        app.set_created(app.get_created().checked_add(1).expect("project counter"));
    }));
    app.run()
}

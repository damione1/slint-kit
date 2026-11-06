// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_increment(slint_kit::action(&app, |app| {
        app.set_count(app.get_count().checked_add(1).expect("counter capacity"));
    }));
    app.run()
}

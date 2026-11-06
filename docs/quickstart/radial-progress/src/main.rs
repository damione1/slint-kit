// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_advance(slint_kit::action(&app, |app| {
        app.set_percent(if app.get_percent() >= 100 { 10 } else { (app.get_percent() + 10).min(100) });
    }));
    app.run()
}

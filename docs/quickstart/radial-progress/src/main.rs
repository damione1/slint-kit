// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;

mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let weak = app.as_weak();
    app.on_advance(move || { if let Some(app) = weak.upgrade() { app.set_percent(if app.get_percent() >= 100 { 10 } else { (app.get_percent() + 10).min(100) }); } });
    app.run()
}

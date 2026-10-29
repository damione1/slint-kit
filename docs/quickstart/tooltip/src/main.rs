// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;

mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let weak = app.as_weak();
    app.on_create(move || { if let Some(app) = weak.upgrade() { app.set_created(app.get_created().checked_add(1).expect("project counter")); } });
    app.run()
}

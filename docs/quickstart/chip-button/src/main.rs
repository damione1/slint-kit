// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    // The host owns the active filter; a chip only requests it.
    app.set_active(2);
    let weak = app.as_weak();
    app.on_choose(move |filter| { if let Some(app) = weak.upgrade() { app.set_active(filter); } });
    app.run()
}

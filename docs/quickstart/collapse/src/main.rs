// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;

mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let weak = app.as_weak();
    app.on_expand(move |expanded| { if let Some(app) = weak.upgrade() { app.set_expanded(expanded); } });
    app.run()
}

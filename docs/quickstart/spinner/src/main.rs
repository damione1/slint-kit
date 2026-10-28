// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;

mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    let weak = app.as_weak();
    app.on_toggle(move || { if let Some(app) = weak.upgrade() { app.set_running(!app.get_running()); } });
    app.run()
}

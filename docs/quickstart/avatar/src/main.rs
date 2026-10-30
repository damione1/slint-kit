// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::init_form_validation!(self, app);
    let weak = app.as_weak();
    app.on_profile_requested(move || {
        if let Some(app) = weak.upgrade() { app.set_status("Alex Morgan’s profile opened by Rust.".into()); }
    });
    app.run()
}

// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::init_form_validation!(self, app);
    let weak = app.as_weak();
    app.on_notes_scrolled(move |offset, range| {
        if let Some(app) = weak.upgrade()
            && range > 0.0
        {
            let percent = (offset / range * 100.0).round().clamp(0.0, 100.0);
            app.set_status(format!("Rust: {percent}% of the notes read").into());
        }
    });
    app.run()
}

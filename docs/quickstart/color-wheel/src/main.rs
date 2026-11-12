// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{AlphaPolicy, SrgbColor, action, event};
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let initial = SrgbColor { red: 124, green: 108, blue: 240, alpha: 255 };
    app.set_draft_color(initial.into());
    app.set_status(format!("Draft: {}", initial.hex(AlphaPolicy::Opaque)).into());
    app.on_color_chosen(event(&app, |app, view| { if let Ok(color) = SrgbColor::try_from(view) { app.set_draft_color(color.into()); app.set_status(format!("Draft: {}", color.hex(AlphaPolicy::Opaque)).into()); } }));
    app.on_submit_color(action(&app, |app| { if let Ok(color) = SrgbColor::try_from(app.get_draft_color()) { app.set_status(format!("Applied: {}", color.hex(AlphaPolicy::Opaque)).into()); } }));
    app.run()
}

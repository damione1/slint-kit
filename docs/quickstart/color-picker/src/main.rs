// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?; slint_kit::install(&app);
    app.on_accent_changed(|view| { if let Ok(color) = slint_kit::SrgbColor::try_from(view) { println!("Accent: {}", color.hex(slint_kit::AlphaPolicy::Opaque)); } });
    app.run()
}

// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?; slint_kit::install(&app);
    app.on_volume_changed(|text| { if let Ok(value) = text.parse::<slint_kit::DecimalValue>() { println!("Volume: {value}"); } });
    app.run()
}

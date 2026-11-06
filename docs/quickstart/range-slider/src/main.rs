// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?; slint_kit::install(&app);
    app.on_limits_changed(|view| { if let Ok(range) = slint_kit::OrderedRange::<slint_kit::DecimalValue>::try_from(view) { println!("{} to {}", range.start(), range.end()); } });
    app.run()
}

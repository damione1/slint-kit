// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?; slint_kit::install(&app);
    app.on_feedback_changed(|view| { if let Ok(value) = Option::<slint_kit::Rating>::try_from(view) { println!("Rating: {value:?}"); } });
    app.run()
}

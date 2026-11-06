// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_chosen(|view| { if let Ok(value) = slint_kit::LocalDateTime::try_from(view) { println!("Appointment: {}", value.date.iso_string()); } });
    app.run()
}

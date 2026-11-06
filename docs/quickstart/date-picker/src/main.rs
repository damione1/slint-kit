// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_chosen(|view| {
        if let Ok(date) = slint_kit::CivilDate::try_from(view) {
            println!("Appointment: {}", date.iso_string());
        }
    });
    app.run()
}

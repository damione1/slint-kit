// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_chosen(|view| {
        if let Ok(range) = slint_kit::DateRange::try_from(view) {
            println!("Booking: {} nights", range.nights());
        }
    });
    app.run()
}

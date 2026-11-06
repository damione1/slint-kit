// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::CivilDate;
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_chosen(slint_kit::event(&app, |app, candidate| {
        if let Ok(date) = CivilDate::try_from(candidate) { println!("Booked: {}", date.iso_string()); app.set_selected(date.into()); }
    }));
    app.run()
}

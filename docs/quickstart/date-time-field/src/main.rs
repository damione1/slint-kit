// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let accepted = std::rc::Rc::new(std::cell::Cell::new(Some(slint_kit::LocalDateTime::try_from(app.get_accepted_value()).expect("valid initial value"))));
    let chosen = accepted.clone();
    app.on_chosen(move |view| {
        if let Ok(value) = slint_kit::LocalDateTime::try_from(view) { chosen.set(Some(value)); println!("Appointment accepted."); }
    });
    app.on_cleared(move || { accepted.set(None); println!("No appointment."); });
    app.run()
}

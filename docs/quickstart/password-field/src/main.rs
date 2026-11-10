// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::*;
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let spec = FieldSpec::text();
    install_field_specs(&app, [spec.named(FieldSpecKey::new("value").unwrap())]).unwrap();
    // Keep the checked value in application state; never render or log it.
    let password = std::cell::RefCell::new(None::<TextValue>);
    app.on_committed(event(&app, move |app, commit| { if let Ok(Some(value)) = spec.accept(commit) { *password.borrow_mut() = Some(value); app.set_accepted(password.borrow().is_some()); } }));
    app.run()
}

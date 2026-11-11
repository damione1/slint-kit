// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
slint::include_modules!();
slint_kit::form_schema!(Registration, App, RegistrationField, {
    name: required(slint_kit::TextValue, slint_kit::FieldSpec::text(), "Check name"),
    email: required(slint_kit::EmailAddress, slint_kit::FieldSpec::email(), "Check email"),
    birth: required(slint_kit::CivilDate, slint_kit::FieldSpec::date(), "Check birth"),
    consent: checked,
});
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    Registration::on_submit(&app, |app, data| app.set_result(data.email.as_str().into()));
    app.run()
}

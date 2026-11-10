// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle; use slint_kit::{event, FieldSpec, OtpValue, OtpRules, OtpLength, HtmlPattern, FieldSpecKey, install_field_specs}; use std::cell::RefCell;
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?; slint_kit::install(&app);
    let numeric_code = RefCell::new(None::<OtpValue>); let alpha_code = RefCell::new(None::<OtpValue>); let grouped_code = RefCell::new(None::<OtpValue>);
    let numeric = FieldSpec::otp(OtpRules::new(OtpLength::new(6).unwrap(), HtmlPattern::compile("[0-9]*").unwrap()));
    let alpha = FieldSpec::otp(OtpRules::new(OtpLength::new(6).unwrap(), HtmlPattern::compile("[A-Za-z0-9]*").unwrap()));
    let grouped = FieldSpec::otp(OtpRules::new(OtpLength::new(7).unwrap(), HtmlPattern::compile("[A-Z]{2}[0-9]{5}").unwrap())).with_grouping(2, "‑");
    install_field_specs(&app, [numeric.named(FieldSpecKey::new("numeric").unwrap()), alpha.named(FieldSpecKey::new("alpha").unwrap()), grouped.named(FieldSpecKey::new("grouped").unwrap())]).unwrap();
    app.on_numeric(event(&app, move |window, commit| { if let Ok(Some(value)) = numeric.accept(commit) { *numeric_code.borrow_mut() = Some(value); window.set_numeric_accepted(numeric_code.borrow().is_some()); } }));
    app.on_alpha(event(&app, move |window, commit| { if let Ok(Some(value)) = alpha.accept(commit) { *alpha_code.borrow_mut() = Some(value); window.set_alpha_accepted(alpha_code.borrow().is_some()); } }));
    app.on_grouped(event(&app, move |window, commit| { if let Ok(Some(value)) = grouped.accept(commit) { *grouped_code.borrow_mut() = Some(value); window.set_grouped_accepted(grouped_code.borrow().is_some()); } }));
    app.run()
}

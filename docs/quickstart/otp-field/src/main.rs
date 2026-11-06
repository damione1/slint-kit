// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{FieldSpec, OtpRules, OtpLength, HtmlPattern, FieldSpecKey, install_field_specs};
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?; slint_kit::install(&app);
    let numeric = FieldSpec::otp(OtpRules::new(OtpLength::new(6).unwrap(), HtmlPattern::compile("[0-9]*").unwrap()));
    let alpha = FieldSpec::otp(OtpRules::new(OtpLength::new(6).unwrap(), HtmlPattern::compile("[A-Za-z0-9]*").unwrap()));
    let grouped = FieldSpec::otp(OtpRules::new(OtpLength::new(7).unwrap(), HtmlPattern::compile("[A-Z]{2}[0-9]{5}").unwrap())).with_grouping(2, "‑");
    install_field_specs(&app, [numeric.named(FieldSpecKey::new("numeric").unwrap()), alpha.named(FieldSpecKey::new("alpha").unwrap()), grouped.named(FieldSpecKey::new("grouped").unwrap())]).unwrap();
    app.on_numeric(move |commit| { if let Ok(Some(value)) = numeric.accept(commit) { println!("Numeric: {}", value.as_str()); } });
    app.on_alpha(move |commit| { if let Ok(Some(value)) = alpha.accept(commit) { println!("Alphanumeric: {}", value.as_str()); } });
    app.on_grouped(move |commit| { if let Ok(Some(value)) = grouped.accept(commit) { println!("Grouped: {}", value.as_str()); } });
    app.run()
}

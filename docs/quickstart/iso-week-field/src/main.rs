// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::*;
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let spec = FieldSpec::iso_week();
    install_field_specs(&app, [spec.named(FieldSpecKey::new("value").unwrap())]).unwrap();
    app.on_committed(move |commit| { if let Ok(Some(value)) = spec.accept(commit) { println!("Accepted: {}", spec.format(&value)); } });
    app.run()
}

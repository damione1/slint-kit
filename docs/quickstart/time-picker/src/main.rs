// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{FieldSpec, TimePrecision, ClockFormat};
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let spec = FieldSpec::time(TimePrecision::Minute, ClockFormat::TwentyFourHour);
    app.on_committed(move |commit| { if let Ok(Some(time)) = spec.accept(commit) { println!("Time: {}", spec.format(&time)); } });
    app.run()
}

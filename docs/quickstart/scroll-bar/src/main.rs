// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_notes_scrolled(slint_kit::event2(&app, |app, offset: f32, range: f32| {
        if range > 0.0 {
            let percent = (offset / range * 100.0).round().clamp(0.0, 100.0);
            app.set_status(format!("Rust: {percent}% of the notes read").into());
        }
    }));
    app.run()
}

// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.set_status("Showing orders awaiting shipment".into());
    app.on_choose(slint_kit::event(&app, |app, filter| {
        app.set_active(filter);
        app.set_status(format!("Showing {filter:?} orders").into());
    }));
    app.run()
}

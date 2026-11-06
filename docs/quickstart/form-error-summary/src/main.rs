// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_request_focus(slint_kit::event(&app, |app, mut target: slint_kit::ui::FocusRequest| {
        target.request = app.get_focus_request().request.checked_add(1).expect("focus request capacity");
        app.set_focus_request(target);
    }));
    app.run()
}

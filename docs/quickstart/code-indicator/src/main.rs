// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.set_code_length(6);
    app.on_digit(slint_kit::action(&app, |app| app.set_entered((app.get_entered() + 1).min(6))));
    app.on_delete_requested(slint_kit::action(&app, |app| app.set_entered((app.get_entered() - 1).max(0))));
    app.run()
}

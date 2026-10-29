// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::init_form_validation!(self, app);
    app.run()
}

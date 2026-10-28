// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    App::new()?.run()
}

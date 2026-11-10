// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
pub mod ui {
    pub use slint_kit::ui::*;
    slint::include_modules!();
}
mod controller;
pub use controller::create_app;
pub fn main() -> Result<(), slint::PlatformError> {
    let language = if std::env::args().any(|arg| arg == "--fr") {
        ui::DialogLanguage::French
    } else {
        ui::DialogLanguage::English
    };
    create_app(language)?.run()
}

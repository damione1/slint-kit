// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../../table-consumer/src/main.rs"]
mod example;
mod table_verify;

fn main() -> Result<(), slint::PlatformError> {
    use slint::ComponentHandle;
    if std::env::args().any(|arg| arg == "--verify") {
        let language = if std::env::args().any(|arg| arg == "--fr") {
            example::ui::TableLanguage::French
        } else {
            example::ui::TableLanguage::English
        };
        let app = example::create_app(language)?;
        table_verify::schedule(&app);
        app.run()
    } else {
        example::main()
    }
}

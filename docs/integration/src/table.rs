// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../../table-consumer/src/main.rs"]
mod example;
mod table_verify;

fn main() -> Result<(), slint::PlatformError> {
    use slint::ComponentHandle;
    if std::env::args().any(|arg| arg == "--verify") {
        let app = example::create_app(std::env::args().any(|arg| arg == "--fr"))?;
        table_verify::schedule(&app);
        app.run()
    } else {
        example::main()
    }
}

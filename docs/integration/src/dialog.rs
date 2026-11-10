// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../../dialog-consumer/src/main.rs"]
mod example;

fn main() -> Result<(), slint::PlatformError> {
    example::main()
}

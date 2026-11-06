// SPDX-License-Identifier: GPL-3.0-or-later
fn main() {
    if std::env::var_os("CARGO_FEATURE_ADAPTER").is_none() {
        return;
    }
    let config = slint_build::CompilerConfiguration::new()
        .as_library("kit")
        .rust_module("ui");
    slint_build::compile_with_config("ui/kit.slint", config).expect("compile Slint Kit library");
}

// SPDX-License-Identifier: GPL-3.0-or-later
fn main() {
    if std::env::var_os("CARGO_FEATURE_ADAPTER").is_none() {
        return;
    }
    println!("cargo:rerun-if-env-changed=SLINT_LIVE_PREVIEW");
    assert!(
        std::env::var_os("SLINT_LIVE_PREVIEW").is_none(),
        "SLINT_LIVE_PREVIEW is not supported with Slint Kit's shared library globals in Slint 1.18.1 (slint-ui/slint#14812). Unset SLINT_LIVE_PREVIEW and rebuild; slint/mcp and SLINT_EMIT_DEBUG_INFO remain supported."
    );
    let config = slint_build::CompilerConfiguration::new()
        .as_library("kit")
        .rust_module("ui");
    slint_build::compile_with_config("ui/kit.slint", config).expect("compile Slint Kit library");
}

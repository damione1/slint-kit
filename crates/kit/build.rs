// SPDX-License-Identifier: GPL-3.0-or-later
fn main() {
    if std::env::var_os("CARGO_FEATURE_NATIVE").is_some() {
        slint_build::compile_with_config(
            "src/native_test_ui.slint",
            slint_build::CompilerConfiguration::new().with_edition((1, 19)),
        )
        .expect("compile native adapter test fixture");
    }
    // Cargo metadata reaches immediate dependents only. The facade forwards the
    // library contract and exposes its generated types through the same ui module.
    for key in ["NAME", "SOURCE", "MODULE"] {
        let value = std::env::var(format!("DEP_KIT_SLINT_SLINT_LIBRARY_{key}"))
            .expect("Slint Kit library metadata");
        println!("cargo::metadata=SLINT_LIBRARY_{key}={value}");
    }
    println!("cargo::metadata=SLINT_LIBRARY_PACKAGE=slint-kit");
}

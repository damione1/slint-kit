// SPDX-License-Identifier: GPL-3.0-or-later
fn main() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/assets/examples/workspace.slint");
    println!("cargo:rerun-if-changed={}", source.display());
    slint_kit_build::compile(source).expect("workspace compiles");
}

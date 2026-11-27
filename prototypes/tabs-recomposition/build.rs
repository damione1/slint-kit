// SPDX-License-Identifier: GPL-3.0-or-later
//! Compiles the kit's own tab fixtures once per recomposition. The fixture
//! sources are read from the kit test crate and only their `KitTabs` import is
//! redirected to a shim that inherits the application component.
use std::{fs, path::PathBuf};

const FIXTURES: [(&str, &str); 3] = [
    ("tabs-panels.slint", "TabsPanelsFixture"),
    (
        "composite-accessibility.slint",
        "CompositeAccessibilityFixture",
    ),
    ("scroll-transactions.slint", "TabsMountFixture"),
];

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let kit_fixtures = manifest.join("../../tests/ui");
    let output = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    for (variant, component) in [("underline", "UnderlineTabs"), ("pill", "PillTabs")] {
        let directory = output.join(variant);
        fs::create_dir_all(&directory).unwrap();
        write_if_changed(
            directory.join("kit-tabs.slint"),
            format!(
                "import {{ {component} }} from \"tabs.slint\";\nexport component KitTabs inherits {component} {{}}\n"
            ),
        );
        let mut entry = String::new();
        for (file, fixture) in FIXTURES {
            let path = kit_fixtures.join(file);
            println!("cargo:rerun-if-changed={}", path.display());
            let source = fs::read_to_string(&path).unwrap();
            let kit_import = "from \"@kit\";";
            assert!(source.contains("KitTabs, ") && source.contains(kit_import));
            let source = source.replacen("KitTabs, ", "", 1).replacen(
                kit_import,
                "from \"@kit\";\nimport { KitTabs } from \"kit-tabs.slint\";",
                1,
            );
            write_if_changed(directory.join(file), source);
            entry.push_str(&format!("export {{ {fixture} }} from \"{file}\";\n"));
        }
        let entry_path = directory.join("fixture.slint");
        write_if_changed(entry_path.clone(), entry);
        kit_build::compile_with_config(
            entry_path,
            kit_build::config()
                .with_debug_info(true)
                .with_include_paths(vec![manifest.join("ui")]),
        )
        .unwrap_or_else(|error| panic!("{variant} fixtures compile: {error:?}"));
        fs::copy(
            output.join("kit-app.rs"),
            output.join(format!("{variant}.rs")),
        )
        .unwrap();
    }
}

// An identical rewrite would give the file a newer timestamp and recompile.
fn write_if_changed(path: PathBuf, contents: String) {
    if fs::read_to_string(&path).is_ok_and(|existing| existing == contents) {
        return;
    }
    fs::write(path, contents).unwrap();
}

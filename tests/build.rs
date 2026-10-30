// SPDX-License-Identifier: GPL-3.0-or-later
fn main() {
    let expected = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../crates/kit-slint/ui")
        .canonicalize()
        .expect("workspace UI directory");
    assert_eq!(
        kit_build::ui_directory()
            .canonicalize()
            .expect("resolved UI directory"),
        expected,
        "workspace tests must use this checkout's UI; isolate Cargo target directories between checkouts"
    );
    for (path, diagnostic) in [
        ("ui/invalid-property.slint", "unknown-domain-value"),
        ("ui/invalid-enum.slint", "unknown-domain-value"),
    ] {
        let error =
            kit_tailwind::compile(path).expect_err("invalid public contract must fail compilation");
        assert!(
            format!("{error:?}").contains(diagnostic),
            "unexpected failure for {path}: {error:?}"
        );
    }
    kit_tailwind::compile("ui/fixture.slint").expect("public component fixture compiles");
}

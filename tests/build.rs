// SPDX-License-Identifier: GPL-3.0-or-later
fn main() {
    for (path, diagnostic) in [
        ("ui/invalid-property.slint", "unknown-domain-value"),
        ("ui/invalid-enum.slint", "unknown-domain-value"),
    ] {
        let error =
            kit_build::compile(path).expect_err("invalid public contract must fail compilation");
        assert!(
            format!("{error:?}").contains(diagnostic),
            "unexpected failure for {path}: {error:?}"
        );
    }
    kit_build::compile("ui/fixture.slint").expect("public component fixture compiles");
}

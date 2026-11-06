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
    kit_build::compile("ui/build-consumer.slint")
        .expect("public build helper compiles a minimal consumer");
    let absolute = std::path::absolute("ui/build-consumer.slint").unwrap();
    kit_build::compile(&absolute).expect("absolute source paths compile");
    kit_build::compile(absolute.canonicalize().unwrap())
        .expect("canonical source paths, including Windows verbatim paths, compile");
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/build consumer été");
    for source in [
        directory.join("app.slint"),
        directory.join("app.slint").canonicalize().unwrap(),
    ] {
        let config = kit_build::config_with_libraries(std::collections::HashMap::from([(
            "application".into(),
            directory.join("application"),
        )]))
        .with_include_paths(vec![directory.join("include")]);
        kit_build::compile_with_config(source, config).expect(
            "spaces, Unicode, relative imports, application aliases and include paths compile",
        );
    }
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

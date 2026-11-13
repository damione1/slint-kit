// SPDX-License-Identifier: GPL-3.0-or-later
//! Build-time companion to the Slint Kit Rust UI component library. Resolves
//! installed `@kit` imports, compiles application UI and verifies shared
//! Rust/Slint contracts without requiring a toolkit source checkout.
//!
//! Use the same-release `slint-kit` runtime and call its per-window installer.
//! See [project setup](https://slint-kit.dev/guides/project-setup/) and
//! [compatibility and scope](https://slint-kit.dev/guides/why-slint-kit/).
//! The helper uses Slint's experimental module compilation.
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Absolute installed source directory, also valid when consumed from Cargo's registry.
pub fn ui_directory() -> PathBuf {
    PathBuf::from(kit_slint::UI_DIRECTORY)
}

/// Resolved build package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Register the kit while preserving application-specific libraries.
/// Debug metadata follows Slint defaults, including `SLINT_EMIT_DEBUG_INFO`.
/// `kit` is reserved; conflicting registrations fail at the boundary.
pub fn config_with_libraries(
    mut libraries: HashMap<String, PathBuf>,
) -> slint_build::CompilerConfiguration {
    assert!(
        libraries.insert("kit".into(), ui_directory()).is_none(),
        "kit is a reserved library alias"
    );
    slint_build::CompilerConfiguration::new().with_library_paths(libraries)
}

pub fn config() -> slint_build::CompilerConfiguration {
    config_with_libraries(HashMap::new())
}

pub fn compile(path: impl AsRef<Path>) -> Result<(), slint_build::CompileError> {
    compile_with_config(path, config())
}

/// Compile with additional libraries or presentation settings while preserving
/// the shared kit identities and automatically exported window services.
pub fn compile_with_config(
    path: impl AsRef<Path>,
    configuration: slint_build::CompilerConfiguration,
) -> Result<(), slint_build::CompileError> {
    // Slint's live-preview generator omits library global implementations.
    // Presence selects that generator, even for an empty value or "0".
    println!("cargo:rerun-if-env-changed=SLINT_LIVE_PREVIEW");
    if std::env::var_os("SLINT_LIVE_PREVIEW").is_some() {
        return Err(slint_build::CompileError::CompileError(vec![
            "SLINT_LIVE_PREVIEW is not supported with Slint Kit's shared library globals in Slint 1.18.1 (slint-ui/slint#14812). Unset SLINT_LIVE_PREVIEW, including in build.rs, and rebuild. The slint/mcp feature and SLINT_EMIT_DEBUG_INFO remain supported; editor/slint-viewer static previews do not require this variable.".into(),
        ]));
    }
    let diagnostic =
        |error: std::io::Error| slint_build::CompileError::CompileError(vec![error.to_string()]);
    verify_contracts().map_err(diagnostic)?;
    let source = std::fs::canonicalize(path.as_ref()).map_err(diagnostic)?;
    let source = import_literal(&source).map_err(diagnostic)?;
    let output = std::path::PathBuf::from(
        std::env::var_os("OUT_DIR").ok_or(slint_build::CompileError::NotRunViaCargo)?,
    );
    let wrapper = output.join("kit-app.slint");
    write_if_changed(&wrapper, &wrapper_source(&source))
        .map_err(slint_build::CompileError::SaveError)?;
    slint_build::compile_with_config(wrapper, configuration)
}

fn wrapper_source(source: &str) -> String {
    format!(
        "export * from {source};\nimport {{ KitFormValidation, KitCalendarService, KitFieldService, KitSelectionService, KitTimeService, KitDataService, KitThemes, KitThemeContext }} from \"@kit\";\nexport {{ KitFormValidation, KitCalendarService, KitFieldService, KitSelectionService, KitTimeService, KitDataService, KitThemes, KitThemeContext }}\n"
    )
}

// Slint registers the wrapper as a build-script input. Rewriting identical
// bytes gives it a newer timestamp than the script run, so Cargo would
// recompile the application crate on every build.
fn write_if_changed(path: &Path, contents: &str) -> std::io::Result<()> {
    if std::fs::read_to_string(path).is_ok_and(|existing| existing == contents) {
        return Ok(());
    }
    std::fs::write(path, contents)
}

// Slint import strings are literal paths, not Rust strings: Debug escaping
// changes Windows prefixes and separators instead of quoting them for Slint.
fn import_literal(source: &Path) -> std::io::Result<String> {
    let source = source.to_str().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Slint source paths must be UTF-8",
        )
    })?;
    if source.contains(['"', '\n', '\r']) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Slint source paths cannot contain quotes or line breaks",
        ));
    }
    let source = if let Some(unc) = source.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else if source.starts_with(r"\\") && !source.starts_with(r"\\?\") {
        source.to_owned()
    } else {
        source
            .strip_prefix(r"\\?\")
            .unwrap_or(source)
            .replace('\\', "/")
    };
    Ok(format!("\"{source}\""))
}

/// Deterministically renders enums declared once by the pure Rust core.
pub fn shared_contract_source() -> String {
    let mut result = "// SPDX-License-Identifier: GPL-3.0-or-later\n// Generated from kit_core::SLINT_ENUM_CONTRACTS.\n".to_string();
    for (name, variants) in kit_core::SLINT_ENUM_CONTRACTS {
        result.push_str(&format!(
            "export enum {name} {{ {} }}\n",
            variants.join(", ")
        ));
    }
    result
}

pub fn verify_contracts() -> std::io::Result<()> {
    let path = ui_directory().join("shared-contracts.slint");
    if std::fs::read_to_string(&path)? == shared_contract_source() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "{} differs from its Rust source",
            path.display()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_preview_reports_the_upstream_limitation_before_codegen() {
        const CHILD: &str = "KIT_TEST_LIVE_PREVIEW_CHILD";
        if std::env::var_os(CHILD).is_some() {
            let error = compile("does-not-need-to-exist.slint").unwrap_err();
            let diagnostic = format!("{error:?}");
            assert!(diagnostic.contains("slint-ui/slint#14812"));
            assert!(diagnostic.contains("Unset SLINT_LIVE_PREVIEW"));
            return;
        }
        for value in ["1", "0", ""] {
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "tests::live_preview_reports_the_upstream_limitation_before_codegen",
                ])
                .env(CHILD, "1")
                .env("SLINT_LIVE_PREVIEW", value)
                .status()
                .unwrap();
            assert!(status.success(), "live-preview value {value:?}");
        }
    }

    #[test]
    fn import_paths_preserve_windows_drives_and_network_shares() {
        for (source, expected) in [
            (r"C:\project\ui\app.slint", "\"C:/project/ui/app.slint\""),
            (
                r"\\?\G:\projet été\ui\app.slint",
                "\"G:/projet été/ui/app.slint\"",
            ),
            (
                r"\\server\share\ui\app.slint",
                r#""\\server\share\ui\app.slint""#,
            ),
            (
                r"\\?\UNC\server\share\ui\app.slint",
                r#""\\server\share\ui\app.slint""#,
            ),
            ("/project/été ui/app.slint", "\"/project/été ui/app.slint\""),
        ] {
            assert_eq!(import_literal(Path::new(source)).unwrap(), expected);
        }
    }

    #[test]
    fn import_paths_reject_unrepresentable_slint_literals() {
        for source in [
            "/project/\"app.slint",
            "/project/\napp.slint",
            "/project/\rapp.slint",
        ] {
            assert_eq!(
                import_literal(Path::new(source)).unwrap_err().kind(),
                std::io::ErrorKind::InvalidInput
            );
        }
    }

    #[test]
    fn generated_contracts_match_source() {
        super::verify_contracts().unwrap();
    }

    #[test]
    fn consumer_debug_metadata_is_opt_in() {
        const CHILD: &str = "KIT_TEST_DEBUG_METADATA_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "tests::consumer_debug_metadata_is_opt_in"])
                .env(CHILD, "1")
                .env_remove("SLINT_EMIT_DEBUG_INFO")
                .status()
                .unwrap();
            assert!(status.success());
            return;
        }
        let directory =
            std::env::temp_dir().join(format!("kit-debug-probe-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("debug-probe.slint");
        std::fs::write(
            &source,
            "export component Probe inherits Window { probe := Rectangle {} }",
        )
        .unwrap();
        for (name, configuration, expected) in [
            ("default", config(), false),
            ("inspection", config().with_debug_info(true), true),
        ] {
            let output = directory.join(format!("{name}.rs"));
            slint_build::compile_with_output_path(&source, &output, configuration).unwrap();
            let generated = std::fs::read_to_string(output).unwrap();
            assert_eq!(
                generated.contains("Rectangle,Probe::probe,"),
                expected,
                "{name} metadata"
            );
        }
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn application_libraries_compile_with_the_default_theme() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "kit-default-consumer-{}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("adornment.slint"),"export component Adornment inherits Rectangle { preferred-width: 16px; preferred-height: 16px; }").unwrap();
        let input = directory.join("app.slint");
        std::fs::write(
            &input,
            r#"import { KitButton } from "@kit/kit.slint";
            import { Adornment } from "@app/adornment.slint";
            export component App inherits Window {
                HorizontalLayout { Adornment {} KitButton { text: "Consumer"; } }
            }"#,
        )
        .unwrap();
        let config = config_with_libraries(HashMap::from([("app".into(), directory.clone())]));
        let result =
            slint_build::compile_with_output_path(&input, directory.join("app.rs"), config);
        std::fs::remove_dir_all(directory).unwrap();
        result.expect("kit and application aliases compile together");
    }

    #[test]
    fn unchanged_wrapper_keeps_its_modification_time() {
        let directory =
            std::env::temp_dir().join(format!("kit-wrapper-stamp-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let wrapper = directory.join("kit-app.slint");
        let contents = wrapper_source("\"ui/app.slint\"");
        write_if_changed(&wrapper, &contents).unwrap();
        let first = std::fs::metadata(&wrapper).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        write_if_changed(&wrapper, &contents).unwrap();
        assert_eq!(
            std::fs::metadata(&wrapper).unwrap().modified().unwrap(),
            first
        );
        write_if_changed(&wrapper, "export * from \"other.slint\";\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(&wrapper).unwrap(),
            "export * from \"other.slint\";\n"
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
}

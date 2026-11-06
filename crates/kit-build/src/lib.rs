// SPDX-License-Identifier: GPL-3.0-or-later
//! Build-time registration of the `@kit` import library.
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
/// `kit` is reserved; conflicting registrations fail at the boundary.
pub fn config_with_libraries(
    mut libraries: HashMap<String, PathBuf>,
) -> slint_build::CompilerConfiguration {
    assert!(
        libraries.insert("kit".into(), ui_directory()).is_none(),
        "kit is a reserved library alias"
    );
    slint_build::CompilerConfiguration::new()
        .with_library_paths(libraries)
        .with_debug_info(true)
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
    let diagnostic =
        |error: std::io::Error| slint_build::CompileError::CompileError(vec![error.to_string()]);
    verify_contracts().map_err(diagnostic)?;
    let source = std::fs::canonicalize(path.as_ref()).map_err(diagnostic)?;
    let source = import_literal(&source).map_err(diagnostic)?;
    let output = std::path::PathBuf::from(
        std::env::var_os("OUT_DIR").ok_or(slint_build::CompileError::NotRunViaCargo)?,
    );
    let wrapper = output.join("kit-app.slint");
    std::fs::write(&wrapper, format!(
        "export * from {source};\nimport {{ KitFormValidation, KitCalendarService, KitFieldService, KitSelectionService, KitTimeService, KitThemes, KitThemeContext }} from \"@kit\";\nexport {{ KitFormValidation, KitCalendarService, KitFieldService, KitSelectionService, KitTimeService, KitThemes, KitThemeContext }}\n"
    )).map_err(slint_build::CompileError::SaveError)?;
    slint_build::compile_with_config(wrapper, configuration)
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
}

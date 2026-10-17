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

/// Preserve this configuration when adding application-specific libraries.
pub fn config() -> slint_build::CompilerConfiguration {
    slint_build::CompilerConfiguration::new()
        .with_library_paths(HashMap::from([("kit".into(), ui_directory())]))
        .with_debug_info(true)
}

pub fn compile(path: impl AsRef<Path>) -> Result<(), slint_build::CompileError> {
    // A stale shared contract is a build error, never a silently incompatible projection.
    verify_contracts().expect("Slint Kit shared contracts are stale");
    slint_build::compile_with_config(path, config())
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
    #[test]
    fn generated_contracts_match_source() {
        super::verify_contracts().unwrap();
    }
}

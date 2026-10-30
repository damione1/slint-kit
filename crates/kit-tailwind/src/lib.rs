// SPDX-License-Identifier: GPL-3.0-or-later
//! Optional build-time presentation library. No runtime or renderer selection.
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub fn ui_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

/// Register both kit libraries while preserving all application library aliases.
/// Reserved aliases are rejected rather than silently replacing application code.
pub fn config_with_libraries(
    mut libraries: HashMap<String, PathBuf>,
) -> slint_build::CompilerConfiguration {
    assert_eq!(
        kit_build::VERSION,
        env!("CARGO_PKG_VERSION"),
        "Slint Kit and Tailwind package versions must match"
    );
    assert!(
        libraries
            .insert("kit-tailwind".into(), ui_directory())
            .is_none(),
        "kit-tailwind is a reserved library alias"
    );
    kit_build::config_with_libraries(libraries)
}

pub fn config() -> slint_build::CompilerConfiguration {
    config_with_libraries(HashMap::new())
}

pub fn compile(path: impl AsRef<Path>) -> Result<(), slint_build::CompileError> {
    kit_build::verify_contracts().expect("Slint Kit shared contracts are stale");
    slint_build::compile_with_config(path, config())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn application_libraries_survive_pack_registration_and_compile_together() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "kit-tailwind-consumer-{}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("adornment.slint"),"export component Adornment inherits Rectangle { preferred-width: 16px; preferred-height: 16px; }").unwrap();
        let input = directory.join("app.slint");
        std::fs::write(
            &input,
            r#"import { KitButton, KitThemeContext } from "@kit/kit.slint";
            import { TailwindThemes } from "@kit-tailwind/theme.slint";
            import { Adornment } from "@app/adornment.slint";
            export component App inherits Window {
                init => { KitThemeContext.current = TailwindThemes.light; }
                HorizontalLayout { Adornment {} KitButton { text: "Consumer"; } }
            }"#,
        )
        .unwrap();
        let config = config_with_libraries(HashMap::from([("app".into(), directory.clone())]));
        let result =
            slint_build::compile_with_output_path(&input, directory.join("app.rs"), config);
        std::fs::remove_dir_all(directory).unwrap();
        result.expect("kit, optional pack and application aliases compile together");
    }
}

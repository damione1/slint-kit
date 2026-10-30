// SPDX-License-Identifier: GPL-3.0-or-later
//! Fingerprint the compiler's reachable UI, rather than every imported file.
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    source: PathBuf,
    output: PathBuf,
    #[serde(default)]
    library: Option<PathBuf>,
}

pub fn compile(path: &Path) -> Result<(), String> {
    let requests: Vec<Request> =
        serde_json::from_slice(&std::fs::read(path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    kit_build::verify_contracts().map_err(|error| error.to_string())?;
    for request in requests {
        let mut config = kit_build::config()
            .with_style("fluent".into())
            .with_debug_info(false)
            .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles);
        if let Some(library) = request.library {
            config = config
                .with_library_paths(std::collections::HashMap::from([("kit".into(), library)]));
        }
        slint_build::compile_with_output_path(request.source, request.output, config)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copy_tree(source: &Path, destination: &Path) {
        std::fs::create_dir_all(destination).unwrap();
        for entry in std::fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let target = destination.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    struct Temporary(PathBuf);
    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn reachable_output_isolates_siblings_and_tracks_input_dependents() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temporary = Temporary(
            std::env::temp_dir().join(format!("slint-doc-inputs-{}-{stamp}", std::process::id())),
        );
        let library = temporary.0.join("ui");
        copy_tree(&kit_build::ui_directory(), &library);
        let names = [
            "checkbox",
            "radio-group",
            "text-field",
            "password-field",
            "integer-field",
            "combobox",
        ];
        let generate = |phase: &str| {
            let requests: Vec<_> = names
                .iter()
                .map(|name| {
                    serde_json::json!({
                        "source": root.join(format!("docs/examples/{name}.slint")),
                        "output": temporary.0.join(format!("{name}-{phase}.rs")),
                        "library": library,
                    })
                })
                .collect();
            let request = temporary.0.join("requests.json");
            std::fs::write(&request, serde_json::to_vec(&requests).unwrap()).unwrap();
            compile(&request).unwrap();
            names
                .iter()
                .map(|name| std::fs::read(temporary.0.join(format!("{name}-{phase}.rs"))).unwrap())
                .collect::<Vec<_>>()
        };
        let before = generate("before");
        let choice = library.join("choice.slint");
        let original = std::fs::read_to_string(&choice).unwrap();
        let changed = original.replace("M 2 7 L 5 10 L 12 3", "M 2 8 L 5 11 L 12 4");
        assert_ne!(changed, original);
        std::fs::write(&choice, changed).unwrap();
        let checkbox = generate("checkbox");
        assert_ne!(before[0], checkbox[0]);
        assert_eq!(
            before[1], checkbox[1],
            "radio shares choice.slint but must stay unchanged"
        );
        assert_eq!(before[2], checkbox[2]);
        std::fs::write(&choice, original).unwrap();
        let input = library.join("input.slint");
        let original = std::fs::read_to_string(&input).unwrap();
        let changed = original.replacen(
            "in property <length> text-letter-spacing: 0px;",
            "in property <length> text-letter-spacing: 1px;",
            1,
        );
        assert_ne!(changed, original, "test must modify the actual base field");
        std::fs::write(input, changed).unwrap();
        let fields = generate("input");
        assert_eq!(before[0], fields[0]);
        assert_eq!(before[1], fields[1]);
        for index in 2..names.len() {
            assert_ne!(
                before[index], fields[index],
                "{} must track the base input",
                names[index]
            );
        }
        assert_eq!(fields, generate("repeated"));
    }
}

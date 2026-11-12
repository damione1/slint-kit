// SPDX-License-Identifier: GPL-3.0-or-later
//! Fingerprint the compiler's reachable UI, rather than every imported file.
use serde::Deserialize;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    source: PathBuf,
    output: PathBuf,
    #[serde(default)]
    library: Option<PathBuf>,
}

/// Environment variable that sets the number of concurrent Slint compilations.
const JOBS_VARIABLE: &str = "KIT_COMPILE_JOBS";

/// Parse the optional worker count once; an invalid value is an error, never ignored.
fn parse_jobs(value: Option<&str>) -> Result<Option<NonZeroUsize>, String> {
    value
        .map(|text| {
            text.parse::<NonZeroUsize>()
                .map_err(|_| format!("{JOBS_VARIABLE} must be a positive integer, got {text:?}"))
        })
        .transpose()
}

fn worker_count(requests: usize) -> Result<usize, String> {
    let configured = parse_jobs(std::env::var(JOBS_VARIABLE).ok().as_deref())?;
    let jobs = match configured {
        Some(jobs) => jobs.get(),
        None => std::thread::available_parallelism().map_or(1, NonZeroUsize::get),
    };
    Ok(jobs.min(requests.max(1)))
}

pub fn compile(path: &Path) -> Result<(), String> {
    let requests: Vec<Request> =
        serde_json::from_slice(&std::fs::read(path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    kit_build::verify_contracts().map_err(|error| error.to_string())?;
    let workers = worker_count(requests.len())?;
    let next = AtomicUsize::new(0);
    // Requests are independent: each reads its own source and writes its own
    // output. Workers pull the next index, so the load balances itself.
    let mut outcomes: Vec<(usize, Result<(), String>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(request) = requests.get(index) else {
                            break;
                        };
                        done.push((index, compile_request(request)));
                    }
                    done
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| match handle.join() {
                Ok(done) => done,
                // A panicking worker must fail the run, never drop its requests.
                Err(_) => vec![(usize::MAX, Err("a compile worker panicked".to_string()))],
            })
            .collect()
    });
    // Input order, whatever order the workers finished in.
    outcomes.sort_by_key(|(index, _)| *index);
    let failures: Vec<String> = outcomes
        .into_iter()
        .filter_map(|(index, outcome)| {
            outcome.err().map(|error| match requests.get(index) {
                Some(request) => format!("{}: {error}", request.source.display()),
                None => error,
            })
        })
        .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} of {} documentation compilations failed:\n{}",
            failures.len(),
            requests.len(),
            failures.join("\n")
        ))
    }
}

fn compile_request(request: &Request) -> Result<(), String> {
    let mut config = kit_build::config()
        .with_style("fluent".into())
        .with_debug_info(false)
        .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles);
    if let Some(library) = &request.library {
        config = config.with_library_paths(std::collections::HashMap::from([(
            "kit".into(),
            library.clone(),
        )]));
    }
    // Fingerprints deliberately expand the reachable implementation in file
    // mode. This process is not an application Cargo build and receives no
    // DEP_KIT module metadata. Normalize only the public root import; keep
    // the directory alias for supplemental font/icon imports. Prepared
    // documentation sources already contain absolute local resource paths.
    let source = std::fs::read_to_string(&request.source).map_err(|error| error.to_string())?;
    let input = request.output.with_extension("fingerprint.slint");
    std::fs::write(&input, source.replace("\"@kit\"", "\"@kit/kit.slint\""))
        .map_err(|error| error.to_string())?;
    let result = slint_build::compile_with_output_path(&input, request.output.clone(), config)
        .map_err(|error| error.to_string());
    let removal = std::fs::remove_file(&input).map_err(|error| error.to_string());
    result.and(removal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_jobs_are_parsed_once_and_invalid_values_fail() {
        assert_eq!(parse_jobs(None), Ok(None));
        assert_eq!(parse_jobs(Some("4")), Ok(NonZeroUsize::new(4)));
        for invalid in ["0", "-1", "many", ""] {
            assert!(parse_jobs(Some(invalid)).is_err(), "{invalid:?}");
        }
    }

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
        let changed = original.replacen(
            "glyph: root.theme.icons.check;",
            "glyph: root.theme.icons.plus;",
            1,
        );
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

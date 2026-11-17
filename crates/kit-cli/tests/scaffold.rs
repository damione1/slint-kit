// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "slint-kit-scaffold-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn command(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_slint-kit"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn desktop_project_has_standalone_public_sources_and_connected_rust() {
    let dir = Directory::new();
    let result = dir.command(&["new", "my-app"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let app = dir.0.join("my-app");
    let manifest = fs::read_to_string(app.join("Cargo.toml")).unwrap();
    assert!(manifest.contains("name = \"my-app\""));
    assert!(manifest.contains("[workspace]"));
    assert!(!manifest.contains("path ="));
    assert!(!manifest.contains("__"));
    assert!(manifest.contains("renderer-skia"));
    assert!(!app.join("index.html").exists());
    assert!(
        fs::read_to_string(app.join("src/main.rs"))
            .unwrap()
            .contains("my_app::run()")
    );
    assert!(
        fs::read_to_string(app.join("src/lib.rs"))
            .unwrap()
            .contains("slint_kit::install(&app)")
    );
    let count: usize = ["src/main.rs", "src/lib.rs"]
        .into_iter()
        .map(|file| fs::read_to_string(app.join(file)).unwrap().lines().count())
        .sum();
    assert!(count <= 15, "generated Rust has {count} lines");
}
#[test]
fn web_project_keeps_desktop_entry_and_adds_a_browser_start() {
    let dir = Directory::new();
    let result = dir.command(&[
        "new",
        "project with spaces",
        "--name",
        "preview-app",
        "--web",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let app = dir.0.join("project with spaces");
    assert!(
        fs::read_to_string(app.join("Cargo.toml"))
            .unwrap()
            .contains("renderer-femtovg")
    );
    assert!(
        fs::read_to_string(app.join("src/lib.rs"))
            .unwrap()
            .contains("wasm_bindgen(start)")
    );
    assert!(
        fs::read_to_string(app.join("index.html"))
            .unwrap()
            .contains("pkg/preview_app.js")
    );
    assert!(app.join("src/main.rs").exists());
}
#[test]
fn rejects_invalid_names_and_options_before_creating_a_directory() {
    let dir = Directory::new();
    for name in [
        "",
        "123",
        "fn",
        "gen",
        "a/b",
        "x.y",
        "CON",
        "core",
        "two words",
        "-bad",
    ] {
        let result = dir.command(&["new", "destination", "--name", name]);
        assert!(!result.status.success(), "accepted {name:?}");
        assert!(!dir.0.join("destination").exists());
    }
    let long_name = "a".repeat(65);
    assert!(
        !dir.command(&["new", "destination", "--name", &long_name])
            .status
            .success()
    );
    assert!(!dir.0.join("destination").exists());
    for args in [
        vec!["new", "destination", "--web", "--web"],
        vec!["new", "destination", "--name"],
        vec!["new", "destination", "--unknown"],
        vec!["new", "destination", "--name", "one", "--name", "two"],
    ] {
        assert!(!dir.command(&args).status.success());
        assert!(!dir.0.join("destination").exists());
    }
}
#[test]
fn refuses_existing_files_and_nonempty_or_empty_directories_without_modification() {
    let dir = Directory::new();
    fs::write(dir.0.join("existing-file"), "keep me").unwrap();
    fs::create_dir(dir.0.join("existing-directory")).unwrap();
    fs::create_dir(dir.0.join("empty-directory")).unwrap();
    fs::write(dir.0.join("existing-directory/secret.txt"), "keep this too").unwrap();
    for destination in ["existing-file", "existing-directory", "empty-directory"] {
        assert!(!dir.command(&["new", destination]).status.success());
    }
    assert_eq!(
        fs::read_to_string(dir.0.join("existing-file")).unwrap(),
        "keep me"
    );
    assert_eq!(
        fs::read_to_string(dir.0.join("existing-directory/secret.txt")).unwrap(),
        "keep this too"
    );
    assert_eq!(
        fs::read_dir(dir.0.join("empty-directory")).unwrap().count(),
        0
    );
}
#[test]
fn help_does_not_create_files() {
    let dir = Directory::new();
    assert!(dir.command(&["--help"]).status.success());
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
}

#[cfg(unix)]
#[test]
fn an_existing_symlink_and_its_target_are_preserved() {
    let dir = Directory::new();
    let target = dir.0.join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("keep.txt"), "unchanged").unwrap();
    let link = dir.0.join("linked");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(!dir.command(&["new", "linked"]).status.success());
    assert_eq!(fs::read_link(&link).unwrap(), target);
    assert_eq!(
        fs::read_to_string(target.join("keep.txt")).unwrap(),
        "unchanged"
    );
}

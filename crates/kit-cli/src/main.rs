// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use std::{
    env,
    ffi::OsString,
    fmt, fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

const MANIFEST: &str = include_str!("../templates/project/Cargo.toml.in");
const BUILD: &str = include_str!("../templates/project/build.rs");
const LIBRARY: &str = include_str!("../templates/project/lib.rs.in");
const UI: &str = include_str!("../templates/project/app.slint");
const HTML: &str = include_str!("../templates/project/index.html.in");
const LICENSE: &str = include_str!("../LICENSE");
const HELP: &str = "Usage: slint-kit new <directory> [--name <package-name>] [--web]\n\nCreates a new desktop application. --web also includes a WASM browser preview.\nThe directory must not exist. Package names start with an ASCII letter and\ncontain only ASCII letters, digits, hyphens or underscores (64 characters maximum).\nSee https://slint-kit.dev/guides/project-setup/ for browser preview commands.\n";

#[derive(Clone, Copy)]
enum Target {
    Desktop,
    DesktopAndWeb,
}
struct ProjectName {
    package: String,
    rust_crate: String,
}
impl ProjectName {
    fn parse(value: String) -> Result<Self, Error> {
        let mut chars = value.chars();
        if value.len() > 64
            || !chars.next().is_some_and(|ch| ch.is_ascii_alphabetic())
            || !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        {
            return Err(Error::Usage("invalid package name".into()));
        }
        let rust_crate = value.replace('-', "_");
        // Rust crate identifiers cannot be language keywords, even when Cargo
        // accepts the package name as an external label.
        if [
            "std",
            "core",
            "alloc",
            "proc_macro",
            "test",
            "as",
            "async",
            "await",
            "break",
            "const",
            "continue",
            "crate",
            "dyn",
            "else",
            "enum",
            "extern",
            "false",
            "fn",
            "for",
            "if",
            "impl",
            "in",
            "let",
            "loop",
            "match",
            "mod",
            "move",
            "mut",
            "pub",
            "ref",
            "return",
            "self",
            "Self",
            "static",
            "struct",
            "super",
            "trait",
            "true",
            "type",
            "unsafe",
            "use",
            "where",
            "while",
            "abstract",
            "become",
            "box",
            "do",
            "final",
            "gen",
            "macro",
            "override",
            "priv",
            "try",
            "typeof",
            "unsized",
            "virtual",
            "yield",
        ]
        .contains(&rust_crate.as_str())
        {
            return Err(Error::Usage("package name is a Rust keyword".into()));
        }
        let portable = rust_crate.to_ascii_uppercase();
        if [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ]
        .contains(&portable.as_str())
        {
            return Err(Error::Usage("package name is reserved on Windows".into()));
        }
        Ok(Self {
            package: value,
            rust_crate,
        })
    }
}
struct NewProject {
    destination: PathBuf,
    name: ProjectName,
    target: Target,
}
enum Command {
    Help,
    New(NewProject),
}
#[derive(Debug)]
enum Error {
    Usage(String),
    Io { path: PathBuf, source: io::Error },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}\n{HELP}"),
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}
fn io_at<T>(path: &Path, value: io::Result<T>) -> Result<T, Error> {
    value.map_err(|source| Error::Io {
        path: path.into(),
        source,
    })
}
fn parse(arguments: impl Iterator<Item = OsString>) -> Result<Command, Error> {
    let mut args = arguments;
    let Some(first) = args.next() else {
        return Ok(Command::Help);
    };
    if first == "--help" || first == "-h" {
        return if args.next().is_none() {
            Ok(Command::Help)
        } else {
            Err(Error::Usage("unexpected arguments after help".into()))
        };
    }
    if first != "new" {
        return Err(Error::Usage("unknown command".into()));
    }
    let destination = PathBuf::from(
        args.next()
            .ok_or_else(|| Error::Usage("missing project directory".into()))?,
    );
    let mut name = None;
    let mut target = Target::Desktop;
    while let Some(option) = args.next() {
        if option == "--name" {
            if name.is_some() {
                return Err(Error::Usage("--name was supplied twice".into()));
            }
            name = Some(
                args.next()
                    .ok_or_else(|| Error::Usage("missing package name".into()))?
                    .into_string()
                    .map_err(|_| Error::Usage("package name must be UTF-8".into()))?,
            );
        } else if option == "--web" {
            match target {
                Target::Desktop => target = Target::DesktopAndWeb,
                Target::DesktopAndWeb => {
                    return Err(Error::Usage("--web was supplied twice".into()));
                }
            }
        } else {
            return Err(Error::Usage(format!(
                "unknown option: {}",
                option.to_string_lossy()
            )));
        }
    }
    let name = match name {
        Some(name) => name,
        None => destination
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Error::Usage("use --name with this directory".into()))?
            .to_owned(),
    };
    Ok(Command::New(NewProject {
        destination,
        name: ProjectName::parse(name)?,
        target,
    }))
}
fn manifest(project: &NewProject) -> String {
    let (renderer, extra, library) = match project.target {
        Target::Desktop => ("renderer-skia", "", ""),
        Target::DesktopAndWeb => (
            "renderer-femtovg",
            "\n[target.'cfg(target_arch = \"wasm32\")'.dependencies]\nwasm-bindgen = \"=0.2.129\"\n",
            "\n[lib]\ncrate-type = [\"rlib\", \"cdylib\"]\n",
        ),
    };
    MANIFEST.replace("__PACKAGE_NAME__", &project.name.package)
        .replace("__RUNTIME_DEPENDENCY__", &format!("slint-kit = \"={}\"", env!("CARGO_PKG_VERSION")))
        .replace("__SLINT_DEPENDENCY__", &format!("slint = {{ version = \"=1.18.1\", default-features = false, features = [\"std\", \"compat-1-18\", \"backend-winit\", \"{renderer}\", \"accessibility\"] }}"))
        .replace("__EXTRA_DEPENDENCIES__", extra)
        .replace("__KIT_VERSION__", env!("CARGO_PKG_VERSION"))
        .replace("__LIBRARY_CONFIGURATION__", library)
}
fn create(project: &NewProject) -> Result<(), Error> {
    // create_dir is the admission boundary: existing files, directories and
    // symlinks are rejected without changing them. Never overwrite a project.
    io_at(&project.destination, fs::create_dir(&project.destination))?;
    let mut created_files = Vec::new();
    let mut created_directories = Vec::new();
    let mut write = |name: &str, content: &str| {
        use io::Write;
        let path = project.destination.join(name);
        let mut file = io_at(
            &path,
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path),
        )?;
        created_files.push(path.clone());
        io_at(&path, file.write_all(content.as_bytes()))
    };
    let result = (|| {
        for name in ["src", "ui"] {
            let path = project.destination.join(name);
            io_at(&path, fs::create_dir(&path))?;
            created_directories.push(path);
        }
        let attribute = match project.target {
            Target::Desktop => "",
            Target::DesktopAndWeb => {
                "#[cfg_attr(target_arch = \"wasm32\", wasm_bindgen::prelude::wasm_bindgen(start))]"
            }
        };
        write("Cargo.toml", &manifest(project))?;
        write("build.rs", BUILD)?;
        write("LICENSE", LICENSE)?;
        write(
            "src/lib.rs",
            &LIBRARY.replace("__WEB_START_ATTRIBUTE__", attribute),
        )?;
        write(
            "src/main.rs",
            &format!(
                "fn main() {{\n    {}::run();\n}}\n",
                project.name.rust_crate
            ),
        )?;
        write("ui/app.slint", UI)?;
        write(".gitignore", "/target/\n/pkg/\n")?;
        match project.target {
            Target::Desktop => {}
            Target::DesktopAndWeb => write(
                "index.html",
                &HTML.replace("__CRATE_NAME__", &project.name.rust_crate),
            )?,
        }
        Ok(())
    })();
    if result.is_err() {
        // Only paths created by this invocation are retired on an I/O failure.
        // Preserve the original error if cleanup itself cannot finish.
        for path in created_files.into_iter().rev() {
            if let Err(error) = fs::remove_file(&path) {
                eprintln!(
                    "Could not remove incomplete file {}: {error}",
                    path.display()
                );
            }
        }
        created_directories.insert(0, project.destination.clone());
        for path in created_directories.into_iter().rev() {
            if let Err(error) = fs::remove_dir(&path) {
                eprintln!(
                    "Could not remove incomplete directory {}: {error}",
                    path.display()
                );
            }
        }
    }
    result
}
fn main() -> ExitCode {
    let result = parse(env::args_os().skip(1)).and_then(|command| match command {
        Command::Help => {
            print!("{HELP}");
            Ok(())
        }
        Command::New(project) => {
            create(&project)?;
            println!(
                "Created {}. Run cargo run inside that directory.",
                project.destination.display()
            );
            Ok(())
        }
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

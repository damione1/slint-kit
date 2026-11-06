// SPDX-License-Identifier: GPL-3.0-or-later
use std::{env, fs, path::PathBuf};
fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    let mut names = fs::read_dir(&root)
        .expect("examples")
        .map(|entry| entry.expect("example entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "slint")
        })
        .collect::<Vec<_>>();
    names.sort();
    let mut source = String::from("import { KitTheme } from \"@kit\";\n");
    let mut routes = String::from(
        "fn capture(path: &Path, options: CaptureOptions) -> Result<()> { match (path.file_stem().and_then(|name| name.to_str()), options.locale) {\n",
    );
    for path in names {
        let slug = path
            .file_stem()
            .expect("example name")
            .to_str()
            .expect("UTF-8 slug");
        for (folder, locale) in [("", "English"), ("fr", "French")] {
            let path = root
                .join(folder)
                .join(format!("{slug}.slint"))
                .canonicalize()
                .expect("localized example");
            println!("cargo:rerun-if-changed={}", path.display());
            let name = format!(
                "Capture{}{}",
                slug.split('-')
                    .map(|word| {
                        let mut chars = word.chars();
                        format!(
                            "{}{}",
                            chars.next().expect("slug segment").to_uppercase(),
                            chars.as_str()
                        )
                    })
                    .collect::<String>(),
                locale
            );
            let status_property = if slug == "button" {
                "in-out property <string> capture-status <=> root.status;"
            } else {
                ""
            };
            source.push_str(&format!("import {{ Example as Base{name} }} from {:?};\nexport component {name} inherits Base{name} {{ in-out property <KitTheme> capture-theme <=> root.theme; {status_property} }}\n", path.to_str().expect("UTF-8 path")));
            let status = if slug == "button" {
                format!("Some((ui::{name}::get_capture_status, ui::{name}::set_capture_status))")
            } else {
                "None".into()
            };
            routes.push_str(&format!("(Some({slug:?}), ExampleLocale::{locale}) => {{ let app = ui::{name}::new()?; slint_kit::install(&app); run_example(app, ui::{name}::get_capture_theme, ui::{name}::set_capture_theme, {status}, options) }},\n"));
        }
    }
    routes.push_str(
        "_ => Err(format!(\"Unknown compiled example: {}\", path.display()).into()), } }\n",
    );
    fs::write(out.join("capture.slint"), source).expect("capture wrapper");
    kit_build::compile(out.join("capture.slint")).expect("compile exact bilingual examples");
    fs::write(out.join("capture_routes.rs"), routes).expect("typed capture routes");
}

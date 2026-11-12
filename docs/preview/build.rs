// SPDX-License-Identifier: GPL-3.0-or-later
use std::{env, fs, path::PathBuf};

#[derive(Clone, Copy)]
enum SourceKind {
    Starter,
    English,
    French,
}

fn main() {
    let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let directory = docs.join("quickstart");
    println!("cargo:rerun-if-changed={}", directory.display());
    let mut names = fs::read_dir(&directory)
        .expect("starter directory")
        .map(|entry| entry.expect("starter directory entry"))
        .filter(|entry| entry.path().join("src/main.rs").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let catalogue = docs.join("examples");
    println!("cargo:rerun-if-changed={}", catalogue.display());
    let catalogue_names = fs::read_dir(&catalogue)
        .expect("catalogue directory")
        .map(|entry| entry.expect("catalogue directory entry"))
        .filter(|entry| {
            entry.path().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "slint")
        })
        .map(|entry| {
            entry
                .path()
                .file_stem()
                .expect("catalogue file stem")
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    assert!(
        names.iter().all(|name| catalogue_names.contains(name)),
        "every starter requires a catalogue fixture"
    );
    for name in &catalogue_names {
        assert!(
            catalogue.join("fr").join(format!("{name}.slint")).is_file(),
            "French catalogue fixture missing: {name}"
        );
    }
    let expected_routes = names.len() + 2 * catalogue_names.len();
    names.extend(catalogue_names);
    names.sort();
    names.dedup();
    let mut route_count = 0;
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let mut modules = String::new();
    let mut routes = String::from(
        "fn run_component(name: &str, options: PreviewOptions) -> Result<(), wasm_bindgen::JsValue> { match (options.kind, options.locale, name) {\n",
    );
    // Every source is compiled once in the same Cargo invocation. Presentation
    // (including viewport and text scale) is selected when a scene is opened.
    for name in names {
        assert!(name.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
        let starter = directory.join(&name);
        let host = starter.join("src/main.rs");
        if host.is_file() {
            println!("cargo:rerun-if-changed={}", host.display());
        }
        for kind in [SourceKind::Starter, SourceKind::English, SourceKind::French] {
            if matches!(kind, SourceKind::Starter) && !host.is_file() {
                continue;
            }
            let input = match kind {
                SourceKind::Starter => starter.join("ui/app.slint"),
                SourceKind::English => docs.join("examples").join(format!("{name}.slint")),
                SourceKind::French => docs.join("examples/fr").join(format!("{name}.slint")),
            };
            println!("cargo:rerun-if-changed={}", input.display());
            slint_kit_build::compile(&input)
                .expect("compile public component with shared services");
            let suffix = match kind {
                SourceKind::Starter => "starter",
                SourceKind::English => "en",
                SourceKind::French => "fr",
            };
            let module = format!("{suffix}_{}", name.replace('-', "_"));
            fs::copy(out.join("kit-app.rs"), out.join(format!("{module}.rs")))
                .expect("name compiled preview");
            let include = format!("include!(concat!(env!(\"OUT_DIR\"), \"/{module}.rs\"));");
            let source = if matches!(kind, SourceKind::Starter) {
                let host = fs::read_to_string(&host)
                    .expect("starter Rust source")
                    .replace("slint::include_modules!();", &include)
                    .replace("pub fn main()", "fn main()")
                    .replace("fn main()", "pub fn run(options: crate::PreviewOptions)");
                let creation = if host.contains("let app = ui::App::new()?;") {
                    "let app = ui::App::new()?;"
                } else {
                    assert!(
                        host.contains("let app = App::new()?;"),
                        "known starter entry point"
                    );
                    "let app = App::new()?;"
                };
                host.replacen(creation, &format!("{creation}\n{}\nif let Some((width, height)) = options.viewport {{ slint::ComponentHandle::window(&app).set_size(slint::LogicalSize::new(width, height)); }}", theme_setup()), 1)
            } else {
                format!(
                    "{include}\npub fn run(options: crate::PreviewOptions) -> Result<(), slint::PlatformError> {{\n\
                     use slint::ComponentHandle;\nlet app = Example::new()?;\n\
                     slint_kit::install(&app);\n{}\n\
                     if let Some((width, height)) = options.viewport {{\n\
                       slint::ComponentHandle::window(&app).set_size(slint::LogicalSize::new(width, height));\n}}\n\
                     app.run()\n}}",
                    theme_setup()
                )
            };
            fs::write(out.join(format!("host_{module}.rs")), source).expect("write preview host");
            modules.push_str(&format!(
                "mod demo_{module} {{ include!(concat!(env!(\"OUT_DIR\"), \"/host_{module}.rs\")); }}\n"
            ));
            let selector = match kind {
                SourceKind::Starter => "crate::PreviewKind::Starter, _",
                SourceKind::English => {
                    "crate::PreviewKind::Catalogue, crate::PreviewLocale::English"
                }
                SourceKind::French => "crate::PreviewKind::Catalogue, crate::PreviewLocale::French",
            };
            routes.push_str(&format!("({selector}, \"{name}\") => demo_{module}::run(options).map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))?,\n"));
            route_count += 1;
        }
    }
    assert_eq!(
        route_count, expected_routes,
        "every starter and localized catalogue must have a route"
    );
    routes.push_str(
        "_ => return Err(wasm_bindgen::JsValue::from_str(\"Unknown component\")), } Ok(()) }\n",
    );
    fs::write(out.join("component_previews.rs"), modules + &routes).expect("preview routes");
}

fn theme_setup() -> &'static str {
    r#"
    let theme_window = slint::ComponentHandle::as_weak(&app);
    crate::register_theme(move |mode| {
        if let Some(app) = theme_window.upgrade() {
            let palettes = slint::ComponentHandle::global::<slint_kit::ui::KitThemes>(&app);
            let mut theme = match mode {
                crate::PreviewTheme::Light => palettes.get_light(),
                crate::PreviewTheme::Dark => palettes.get_dark(),
            };
            theme.reduced_motion = match options.kind {
                crate::PreviewKind::Catalogue => true,
                crate::PreviewKind::Starter => crate::prefers_reduced_motion(),
            };
            theme.text_scale = options.text_scale;
            slint::ComponentHandle::global::<slint_kit::ui::KitThemeContext>(&app).set_current(theme.clone());
            app.set_theme(theme);
        }
    });
    "#
}

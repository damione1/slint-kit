// SPDX-License-Identifier: GPL-3.0-or-later
use std::{env, fs, path::PathBuf};

fn main() {
    let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let directory = docs.join("quickstart");
    println!("cargo:rerun-if-changed={}", directory.display());
    let mut names = fs::read_dir(&directory)
        .expect("starter directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().join("src/main.rs").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    names.sort();
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
        println!("cargo:rerun-if-changed={}", host.display());
        for kind in ["starter", "en", "fr"] {
            let input = match kind {
                "starter" => starter.join("ui/app.slint"),
                "en" => docs.join("examples").join(format!("{name}.slint")),
                "fr" => docs.join("examples/fr").join(format!("{name}.slint")),
                _ => unreachable!(),
            };
            println!("cargo:rerun-if-changed={}", input.display());
            slint_kit_build::compile(&input)
                .expect("compile public component with shared services");
            let module = format!("{kind}_{}", name.replace('-', "_"));
            fs::copy(out.join("kit-app.rs"), out.join(format!("{module}.rs")))
                .expect("name compiled preview");
            let include = format!("include!(concat!(env!(\"OUT_DIR\"), \"/{module}.rs\"));");
            let source = if kind == "starter" {
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
                host.replacen(creation, &format!("{creation}\n{}\nif let Some((width, height)) = options.viewport {{ app.window().set_size(slint::LogicalSize::new(width, height)); }}", theme_setup()), 1)
            } else {
                format!(
                    "{include}\npub fn run(options: crate::PreviewOptions) -> Result<(), slint::PlatformError> {{\n\
                     use slint::ComponentHandle;\nlet app = Example::new()?;\n\
                     slint_kit::install(&app);\n{}\n\
                     if let Some((width, height)) = options.viewport {{\n\
                       app.window().set_size(slint::LogicalSize::new(width, height));\n}}\n\
                     app.run()\n}}",
                    theme_setup()
                )
            };
            fs::write(out.join(format!("host_{module}.rs")), source).expect("write preview host");
            modules.push_str(&format!(
                "mod demo_{module} {{ include!(concat!(env!(\"OUT_DIR\"), \"/host_{module}.rs\")); }}\n"
            ));
            let selector = match kind {
                "starter" => "crate::PreviewKind::Starter, _",
                "en" => "crate::PreviewKind::Catalogue, crate::PreviewLocale::English",
                "fr" => "crate::PreviewKind::Catalogue, crate::PreviewLocale::French",
                _ => unreachable!(),
            };
            routes.push_str(&format!("({selector}, \"{name}\") => demo_{module}::run(options).map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))?,\n"));
        }
    }
    routes.push_str(
        "_ => return Err(wasm_bindgen::JsValue::from_str(\"Unknown component\")), } Ok(()) }\n",
    );
    fs::write(out.join("component_previews.rs"), modules + &routes).expect("preview routes");
}

fn theme_setup() -> &'static str {
    r#"
    let theme_window = app.as_weak();
    crate::register_theme(move |mode| {
        if let Some(app) = theme_window.upgrade() {
            let palettes = app.global::<slint_kit::ui::KitThemes>();
            let mut theme = match mode {
                crate::PreviewTheme::Light => palettes.get_light(),
                crate::PreviewTheme::Dark => palettes.get_dark(),
            };
            theme.reduced_motion = match options.kind {
                crate::PreviewKind::Catalogue => true,
                crate::PreviewKind::Starter => crate::prefers_reduced_motion(),
            };
            theme.text_scale = options.text_scale;
            app.global::<slint_kit::ui::KitThemeContext>().set_current(theme.clone());
            app.set_theme(theme);
        }
    });
    "#
}

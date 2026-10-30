// SPDX-License-Identifier: GPL-3.0-or-later
use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=SLINT_PREVIEW_COMPONENTS");
    println!("cargo:rerun-if-env-changed=SLINT_PREVIEW_SOURCE_DIR");
    println!("cargo:rerun-if-env-changed=SLINT_PREVIEW_FIXTURE");
    println!("cargo:rerun-if-env-changed=SLINT_PREVIEW_TEXT_SCALE");
    println!("cargo:rerun-if-env-changed=SLINT_PREVIEW_VIEWPORT");
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../quickstart");
    let mut names = if let Ok(names) = env::var("SLINT_PREVIEW_COMPONENTS") {
        names.split(',').map(str::to_owned).collect::<Vec<_>>()
    } else {
        fs::read_dir(&directory)
            .expect("starter directory")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().join("src/main.rs").is_file())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
    };
    names.sort();
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let mut modules = String::new();
    let mut routes = String::from(
        "pub fn run_component(name: &str) -> Result<(), wasm_bindgen::JsValue> { match name {\n",
    );
    for name in names {
        assert!(name.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
        let starter = directory.join(&name);
        let input = env::var_os("SLINT_PREVIEW_SOURCE_DIR").map_or_else(
            || starter.join("ui/app.slint"),
            |directory| PathBuf::from(directory).join(format!("{name}.slint")),
        );
        let host = starter.join("src/main.rs");
        println!("cargo:rerun-if-changed={}", input.display());
        println!("cargo:rerun-if-changed={}", host.display());
        let compiled_input = out.join(format!("{name}.slint"));
        fs::copy(input, &compiled_input).expect("copy exact starter UI");
        slint_build::compile_with_config(&compiled_input, slint_kit_build::config())
            .expect("compile public component");
        let source = if env::var_os("SLINT_PREVIEW_FIXTURE").is_some() {
            let scale = match env::var("SLINT_PREVIEW_TEXT_SCALE").as_deref() {
                Ok("1") => 1,
                Ok("2") => 2,
                other => panic!("invalid fixture text scale: {other:?}"),
            };
            let viewport = env::var("SLINT_PREVIEW_VIEWPORT").expect("fixture viewport");
            let (width, height) = viewport.split_once('x').expect("WIDTHxHEIGHT");
            let width: u16 = width.parse().expect("viewport width");
            let height: u16 = height.parse().expect("viewport height");
            format!(
                "include!(concat!(env!(\"OUT_DIR\"), \"/{name}.rs\"));\n\
                 pub fn run() -> Result<(), slint::PlatformError> {{\n\
                   use slint::ComponentHandle;\n\
                   let app = Example::new()?;\n\
                   let mut theme = app.get_theme();\n\
                   theme.reduced_motion = true; theme.text_scale = {scale}.0;\n\
                   app.set_theme(theme);\n\
                   app.window().set_size(slint::LogicalSize::new({width}.0, {height}.0));\n\
                   app.run()\n}}"
            )
        } else {
            fs::read_to_string(host)
                .expect("starter Rust source")
                .replace(
                    "slint::include_modules!();",
                    &format!("include!(concat!(env!(\"OUT_DIR\"), \"/{name}.rs\"));"),
                )
                .replace("pub fn main()", "fn main()")
                .replace("fn main()", "pub fn run()")
        };
        assert!(source.contains("pub fn run()"), "starter main entry point");
        let module = name.replace('-', "_");
        fs::write(out.join(format!("host_{module}.rs")), source).expect("write preview host");
        modules.push_str(&format!(
            "mod demo_{module} {{ include!(concat!(env!(\"OUT_DIR\"), \"/host_{module}.rs\")); }}\n"
        ));
        routes.push_str(&format!("\"{name}\" => demo_{module}::run().map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))?,\n"));
    }
    routes.push_str(
        "_ => return Err(wasm_bindgen::JsValue::from_str(\"Unknown component\")), } Ok(()) }\n",
    );
    fs::write(out.join("component_previews.rs"), modules + &routes).expect("preview routes");
}

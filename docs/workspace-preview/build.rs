// SPDX-License-Identifier: GPL-3.0-or-later
fn main() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/assets/examples/workspace.slint");
    println!("cargo:rerun-if-changed={}",source.display());
    let source = std::fs::read_to_string(source).expect("workspace consumer");
    let source = format!("import {{ TailwindThemes }} from \"@kit-tailwind/theme.slint\";\nimport {{ InterFont }} from \"@kit-tailwind/fonts/inter.slint\";\n{source}");
    let source = source.replace("export component App inherits Window {", "export component App inherits Window {\n default-font-family: root.use-inter ? InterFont.family : \"\";\n in property<bool> use-inter;\n out property<KitTheme> kit-light: KitThemes.light;\n out property<KitTheme> kit-dark: KitThemes.dark;\n out property<KitTheme> tailwind-light: TailwindThemes.light;\n out property<KitTheme> tailwind-dark: TailwindThemes.dark;");
    let path = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR")).join("workspace.slint");
    std::fs::write(&path,source).expect("preview presentation");
    slint_build::compile_with_config(path,slint_kit_tailwind::config()).expect("workspace compiles");
}

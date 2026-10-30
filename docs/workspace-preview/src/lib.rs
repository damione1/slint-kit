// SPDX-License-Identifier: GPL-3.0-or-later
#[path = "../../src/assets/examples/workspace.rs"]
mod consumer;
use slint::ComponentHandle;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No browser window"))?;
    let parameters = web_sys::UrlSearchParams::new_with_str(&window.location().search()?)?;
    let app = consumer::create().map_err(|error|JsValue::from_str(&error.to_string()))?;
    let tailwind = parameters.get("theme").as_deref() != Some("kit");
    let dark = parameters.get("mode").as_deref() == Some("dark");
    let mut theme = match (tailwind,dark) {
        (false,false) => app.get_kit_light(), (false,true) => app.get_kit_dark(),
        (true,false) => app.get_tailwind_light(), (true,true) => app.get_tailwind_dark(),
    };
    theme.text_scale = if parameters.get("scale").as_deref() == Some("2") {2.} else {1.};
    app.set_theme(theme);
    app.set_use_inter(tailwind);
    app.set_french(parameters.get("lang").as_deref() == Some("fr"));
    let scene = parameters.get("scene");
    if matches!(scene.as_deref(),Some("projects" | "dialog")) { app.set_page(consumer::ui::WorkspacePage::Projects); }
    if scene.as_deref() == Some("dialog") { app.set_editor_status(consumer::ui::EditorStatus::Editing); }
    app.run().map_err(|error|JsValue::from_str(&error.to_string()))
}

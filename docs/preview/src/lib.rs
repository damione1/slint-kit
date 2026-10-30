// SPDX-License-Identifier: GPL-3.0-or-later
use wasm_bindgen::prelude::*;

include!(concat!(env!("OUT_DIR"), "/component_previews.rs"));

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let window =
        web_sys::window().ok_or_else(|| JsValue::from_str("Browser window unavailable"))?;
    let parameters = web_sys::UrlSearchParams::new_with_str(&window.location().search()?)?;
    let component = parameters
        .get("component")
        .unwrap_or_else(|| "button".into());
    run_component(&component)
}

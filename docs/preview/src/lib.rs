// SPDX-License-Identifier: GPL-3.0-or-later
use std::cell::{Cell, RefCell};
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy)]
enum PreviewTheme {
    Light,
    Dark,
}

impl PreviewTheme {
    fn parse(value: &str) -> Result<Self, JsValue> {
        match value {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            _ => Err(JsValue::from_str("Unknown preview theme")),
        }
    }
}

type ThemeUpdate = Box<dyn Fn(PreviewTheme)>;
thread_local! {
    static CURRENT_THEME: Cell<PreviewTheme> = const { Cell::new(PreviewTheme::Light) };
    static THEME_UPDATE: RefCell<Option<ThemeUpdate>> = const { RefCell::new(None) };
}

fn register_theme(update: impl Fn(PreviewTheme) + 'static) {
    CURRENT_THEME.with(|theme| update(theme.get()));
    THEME_UPDATE.with(|slot| *slot.borrow_mut() = Some(Box::new(update)));
}

/// Change presentation on the existing Slint window, preserving native drafts,
/// selection and host-controller state.
#[wasm_bindgen]
pub fn set_theme(value: &str) -> Result<(), JsValue> {
    let theme = PreviewTheme::parse(value)?;
    CURRENT_THEME.with(|current| current.set(theme));
    if THEME_UPDATE.with(|slot| slot.borrow().is_some()) {
        slint::invoke_from_event_loop(move || {
            THEME_UPDATE.with(|slot| {
                if let Some(update) = slot.borrow().as_ref() {
                    update(theme);
                }
            });
        })
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    }
    Ok(())
}

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
    set_theme(&parameters.get("theme").unwrap_or_else(|| "light".into()))?;
    run_component(&component)
}

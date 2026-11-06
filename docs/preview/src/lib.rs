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

#[derive(Clone, Copy)]
enum PreviewKind {
    Starter,
    Catalogue,
}
#[derive(Clone, Copy)]
enum PreviewLocale {
    English,
    French,
}
#[derive(Clone, Copy)]
struct PreviewOptions {
    kind: PreviewKind,
    locale: PreviewLocale,
    text_scale: f32,
    viewport: Option<(f32, f32)>,
}

impl PreviewOptions {
    fn parse(parameters: &web_sys::UrlSearchParams) -> Result<Self, JsValue> {
        let kind = match parameters.get("kind").as_deref().unwrap_or("starter") {
            "starter" => PreviewKind::Starter,
            "catalogue" => PreviewKind::Catalogue,
            _ => return Err(JsValue::from_str("Unknown scene kind")),
        };
        let locale = match parameters.get("locale").as_deref().unwrap_or("en") {
            "en" => PreviewLocale::English,
            "fr" => PreviewLocale::French,
            _ => return Err(JsValue::from_str("Unknown scene locale")),
        };
        let text_scale = match parameters.get("textScale").as_deref().unwrap_or("1") {
            "1" => 1.0,
            "2" => 2.0,
            _ => return Err(JsValue::from_str("Unsupported text scale")),
        };
        let viewport = parameters
            .get("viewport")
            .map(|value| {
                let (width, height) = value
                    .split_once('x')
                    .ok_or_else(|| JsValue::from_str("Expected WIDTHxHEIGHT"))?;
                let parse = |value: &str| {
                    value
                        .parse::<u16>()
                        .ok()
                        .filter(|value| *value > 0 && *value <= 4096)
                        .map(f32::from)
                        .ok_or_else(|| JsValue::from_str("Invalid viewport dimension"))
                };
                Ok::<_, JsValue>((parse(width)?, parse(height)?))
            })
            .transpose()?;
        Ok(Self {
            kind,
            locale,
            text_scale,
            viewport,
        })
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

fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|window| {
            window
                .match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .is_some_and(|query| query.matches())
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
    run_component(&component, PreviewOptions::parse(&parameters)?)
}

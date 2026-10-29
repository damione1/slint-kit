// SPDX-License-Identifier: GPL-3.0-or-later
//! Browser paste events bypass TextInput key callbacks in Slint 1.18.1.
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

#[doc(hidden)]
pub struct WebPhonePasteGuard {
    window: web_sys::Window,
    callback: Closure<dyn FnMut(web_sys::ClipboardEvent)>,
}
impl Drop for WebPhonePasteGuard {
    fn drop(&mut self) {
        let _ = self.window.remove_event_listener_with_callback_and_bool(
            "paste",
            self.callback.as_ref().unchecked_ref(),
            true,
        );
    }
}
/// Capture rejected phone paste before the backend inserts it. The caller owns
/// this guard for the lifetime of its window; other focused controls pass through.
#[doc(hidden)]
pub fn guard_web_phone_paste(
    is_phone_focused: impl Fn() -> bool + 'static,
) -> Result<WebPhonePasteGuard, JsValue> {
    let window = web_sys::window()
        .ok_or_else(|| JsValue::from_str("Phone paste requires a browser window"))?;
    let callback = Closure::<dyn FnMut(web_sys::ClipboardEvent)>::new(
        move |event: web_sys::ClipboardEvent| {
            if is_phone_focused()
                && event
                    .clipboard_data()
                    .and_then(|data| data.get_data("text/plain").ok())
                    .is_some_and(|text| kit_core::normalize_phone(&text).is_err())
            {
                event.prevent_default();
                event.stop_immediate_propagation();
            }
        },
    );
    window.add_event_listener_with_callback_and_bool(
        "paste",
        callback.as_ref().unchecked_ref(),
        true,
    )?;
    Ok(WebPhonePasteGuard { window, callback })
}

// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use slint::ComponentHandle;

/// Connect an application action without keeping its window alive.
///
/// The handler runs on the UI thread while the window exists. It receives a
/// temporary strong handle and must not store that handle in the window itself.
pub fn action<W, F>(window: &W, handler: F) -> impl Fn() + 'static
where
    W: ComponentHandle + 'static,
    F: Fn(W) + 'static,
{
    let weak = window.as_weak();
    move || {
        if let Some(window) = weak.upgrade() {
            handler(window);
        }
    }
}

/// Connect a typed application event without keeping its window alive.
pub fn event<W, A, F>(window: &W, handler: F) -> impl Fn(A) + 'static
where
    W: ComponentHandle + 'static,
    A: 'static,
    F: Fn(W, A) + 'static,
{
    let weak = window.as_weak();
    move |argument| {
        if let Some(window) = weak.upgrade() {
            handler(window, argument);
        }
    }
}

/// Connect a two-value application event without keeping its window alive.
pub fn event2<W, A, B, F>(window: &W, handler: F) -> impl Fn(A, B) + 'static
where
    W: ComponentHandle + 'static,
    A: 'static,
    B: 'static,
    F: Fn(W, A, B) + 'static,
{
    let weak = window.as_weak();
    move |first, second| {
        if let Some(window) = weak.upgrade() {
            handler(window, first, second);
        }
    }
}

/// Connect a three-value application event without keeping its window alive.
pub fn event3<W, A, B, C, F>(window: &W, handler: F) -> impl Fn(A, B, C) + 'static
where
    W: ComponentHandle + 'static,
    A: 'static,
    B: 'static,
    C: 'static,
    F: Fn(W, A, B, C) + 'static,
{
    let weak = window.as_weak();
    move |first, second, third| {
        if let Some(window) = weak.upgrade() {
            handler(window, first, second, third);
        }
    }
}

// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{KeyboardScopeFixture, SelectState, SelectionRef};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use std::{cell::Cell, rc::Rc};

fn key(window: &KeyboardScopeFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}

#[test]
fn disabled_keyboard_scope_rejects_native_keys_but_trigger_translation_remains_available() {
    i_slint_backend_testing::init_no_event_loop();
    let window = KeyboardScopeFixture::new().unwrap();
    window.set_state(SelectState {
        open: true,
        active: SelectionRef {
            token: 41,
            present: true,
        },
        generation: 7,
        ..Default::default()
    });
    let activations = Rc::new(Cell::new(0));
    let observed = activations.clone();
    window.on_activate(move |token, generation| {
        assert_eq!((token, generation), (41, 7));
        observed.set(observed.get() + 1);
    });
    let other_intentions = Rc::new(Cell::new(0));
    let observed = other_intentions.clone();
    window.on_navigate(move |_| observed.set(observed.get() + 1));
    let observed = other_intentions.clone();
    window.on_dismiss(move |_| observed.set(observed.get() + 1));
    let observed = other_intentions.clone();
    window.on_typeahead(move |_| observed.set(observed.get() + 1));
    window.show().unwrap();
    window.invoke_focus_keyboard();
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Return.into(),
    });
    assert_eq!(activations.get(), 1);
    window.set_keyboard_enabled(false);
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Return.into(),
    });
    for value in [
        Key::Return,
        Key::Space,
        Key::DownArrow,
        Key::Home,
        Key::End,
        Key::PageDown,
        Key::PageUp,
        Key::Escape,
    ] {
        key(&window, value);
    }
    key(&window, "x");
    assert_eq!(
        activations.get(),
        1,
        "disabling a focused scope cancels held activation"
    );
    assert_eq!(
        other_intentions.get(),
        0,
        "disabled native keys cannot navigate, dismiss or type ahead"
    );
    window.set_keyboard_enabled(true);
    window.invoke_focus_keyboard();
    key(&window, Key::Return);
    assert_eq!(activations.get(), 2);

    // KitSelectTrigger intentionally delegates to a non-focusable translator.
    // Disabling the translator's native scope must not disable that public path.
    let opened = Rc::new(Cell::new(0));
    let observed = opened.clone();
    window.on_open_requested(move || observed.set(observed.get() + 1));
    let mut state = window.get_state();
    state.open = false;
    window.set_state(state.clone());
    window.invoke_focus_trigger();
    key(&window, Key::DownArrow);
    assert_eq!(opened.get(), 1);
    state.open = true;
    window.set_state(state);
    key(&window, Key::Return);
    assert_eq!(
        activations.get(),
        3,
        "normal select trigger activation still delegates"
    );
}

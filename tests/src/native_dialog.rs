// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::ElementQuery;
use slint::{
    ComponentHandle,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{cell::Cell, rc::Rc};

fn key(window: &NativeDialogFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
#[test]
fn native_dialog_traverses_real_editors_skips_disabled_controls_and_wraps_in_both_directions() {
    i_slint_backend_testing::init_no_event_loop();
    let window = NativeDialogFixture::new().unwrap();
    let calls = Rc::new(Cell::new(0));
    let output = calls.clone();
    window.on_outside_activated(move || output.set(output.get() + 1));
    let sorts = calls.clone();
    window.on_outside_sort(move || sorts.set(sorts.get() + 1));
    window.show().unwrap();
    window.invoke_focus_first();
    assert!(window.get_first_focused());
    for _ in 0..3 {
        key(&window, Key::Tab);
        assert!(window.get_editor_focused());
        key(&window, Key::Tab);
        assert!(window.get_last_focused());
        key(&window, Key::Tab);
        assert!(window.get_first_focused());
        key(&window, Key::Backtab);
        assert!(window.get_last_focused());
        key(&window, Key::Backtab);
        assert!(window.get_editor_focused());
        key(&window, Key::Backtab);
        assert!(window.get_first_focused());
    }
    window.invoke_focus_background_menu();
    window.invoke_focus_background_table();
    assert!(window.get_first_focused());
    for label in [
        "Background destination",
        "Sortable background",
        "Background row",
        "Outside",
    ] {
        ElementQuery::from_root(&window)
            .match_predicate(|element| element.accessible_label().as_deref() == Some(label))
            .find_first()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);
    }
    assert_eq!(calls.get(), 0);
    window.invoke_focus_first();
    window.set_tab_policy(DialogTabPolicy::Host);
    let tab = Rc::new(Cell::new(0));
    let output = tab.clone();
    window.on_host_tab(move |_| output.set(output.get() + 1));
    key(&window, Key::Tab);
    assert_eq!(tab.get(), 1);
    assert!(window.get_first_focused());
}

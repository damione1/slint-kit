// SPDX-License-Identifier: GPL-3.0-or-later
use crate::LogTabOrderFixture;
use i_slint_backend_testing::{ElementQuery, init_no_event_loop};
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{Key, PointerEventButton, WindowEvent},
};

fn fixture(tab_stop: bool) -> LogTabOrderFixture {
    init_no_event_loop();
    let window = LogTabOrderFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_tab_stop(tab_stop);
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}
fn key(window: &LogTabOrderFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}

#[test]
fn read_only_text_areas_are_tab_stops_by_default() {
    let window = fixture(true);
    window.invoke_focus_before();
    key(&window, Key::Tab);
    assert!(window.get_first_focused());
}

#[test]
fn a_text_area_outside_the_tab_order_is_skipped_in_both_directions() {
    let window = fixture(false);
    window.invoke_focus_before();
    key(&window, Key::Tab);
    assert!(
        window.get_after_focused(),
        "Tab passes the pinned entry and every log line"
    );
    key(&window, Key::Backtab);
    assert!(window.get_before_focused(), "Shift+Tab passes them back");
}

#[test]
fn a_text_area_outside_the_tab_order_keeps_pointer_selection_and_programmatic_focus() {
    let window = fixture(false);
    let field = ElementQuery::from_root(&window)
        .match_id("LogTabOrderFixture::first-line")
        .find_first()
        .unwrap();
    let origin = field.absolute_position();
    let size = field.size();
    let y = origin.y + size.height / 2.;
    let start = LogicalPosition::new(origin.x + 14., y);
    let end = LogicalPosition::new(origin.x + 90., y);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: start });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: start,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: end });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: end,
            button: PointerEventButton::Left,
        });
    assert!(window.get_first_focused(), "a press focuses the text area");
    assert_ne!(
        window.get_anchor_first(),
        window.get_cursor_first(),
        "dragging selects text"
    );
    window.invoke_focus_before();
    assert!(!window.get_first_focused());
    window.invoke_focus_line();
    assert!(
        window.get_first_focused(),
        "take-focus still reaches the text area"
    );
}

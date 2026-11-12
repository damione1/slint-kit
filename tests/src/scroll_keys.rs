// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{Key, PointerEventButton, WindowEvent},
};

fn fixture(axes: ScrollAxes, focusable: bool) -> ScrollKeysFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = ScrollKeysFixture::new().unwrap();
    window.set_axes(axes);
    window.set_focusable(focusable);
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}
fn tap(window: &ScrollKeysFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    slint::platform::update_timers_and_animations();
}
fn shifted(window: &ScrollKeysFixture, key: Key) {
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    tap(window, key);
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
}
fn near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.01, "{actual} != {expected}");
}

#[test]
fn a_focusable_viewport_is_a_tab_stop_with_a_visible_ring() {
    let window = fixture(ScrollAxes::Vertical, true);
    assert!(!window.get_ring());
    tap(&window, Key::Tab);
    assert!(window.get_ring());
    assert!(!window.get_button_focused());
    tap(&window, Key::Tab);
    assert!(window.get_button_focused());
    assert!(!window.get_ring());
}

#[test]
fn the_default_viewport_stays_out_of_the_tab_order_and_ignores_keys() {
    let window = fixture(ScrollAxes::Vertical, false);
    tap(&window, Key::Tab);
    assert!(!window.get_ring());
    assert!(window.get_button_focused(), "Tab goes straight to content");
    tap(&window, Key::DownArrow);
    tap(&window, Key::PageDown);
    near(window.get_offset_y(), 0.);
    assert_eq!(window.get_scrolls(), 0);
}

#[test]
fn arrows_pages_space_home_and_end_scroll_the_vertical_axis() {
    let window = fixture(ScrollAxes::Vertical, true);
    let line = window.get_line_step();
    let page = window.get_viewport_height() - line;
    tap(&window, Key::Tab);
    tap(&window, Key::DownArrow);
    near(window.get_offset_y(), -line);
    tap(&window, Key::UpArrow);
    near(window.get_offset_y(), 0.);
    tap(&window, Key::PageDown);
    near(window.get_offset_y(), -page);
    tap(&window, Key::PageUp);
    near(window.get_offset_y(), 0.);
    tap(&window, Key::Space);
    near(window.get_offset_y(), -page);
    shifted(&window, Key::Space);
    near(window.get_offset_y(), 0.);
    tap(&window, Key::End);
    assert!(window.get_at_end());
    assert!(!window.get_at_start());
    near(
        window.get_offset_y(),
        -(window.get_content_height() - window.get_viewport_height()),
    );
    tap(&window, Key::Home);
    assert!(window.get_at_start());
    near(window.get_offset_y(), 0.);
    // Left and Right do nothing on a vertical-only viewport.
    tap(&window, Key::RightArrow);
    near(window.get_offset_x(), 0.);
}

#[test]
fn keys_at_an_edge_are_rejected_so_an_outer_scroller_can_use_them() {
    let window = fixture(ScrollAxes::Vertical, true);
    tap(&window, Key::Tab);
    tap(&window, Key::UpArrow);
    tap(&window, Key::Home);
    assert_eq!(window.get_scrolls(), 0);
    tap(&window, Key::End);
    let scrolls = window.get_scrolls();
    tap(&window, Key::DownArrow);
    tap(&window, Key::End);
    assert_eq!(window.get_scrolls(), scrolls);
}

#[test]
fn keys_bubbling_from_focused_content_scroll_the_viewport() {
    let window = fixture(ScrollAxes::Vertical, true);
    tap(&window, Key::Tab);
    tap(&window, Key::Tab);
    assert!(window.get_button_focused());
    tap(&window, Key::DownArrow);
    near(window.get_offset_y(), -window.get_line_step());
    tap(&window, Key::Space);
    assert_eq!(window.get_activations(), 1, "the button keeps Space");
}

#[test]
fn shortcuts_with_a_command_modifier_are_not_consumed() {
    let window = fixture(ScrollAxes::Vertical, true);
    tap(&window, Key::Tab);
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    tap(&window, Key::End);
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    near(window.get_offset_y(), 0.);
}

#[test]
fn both_axes_scroll_by_arrows_and_report_the_vertical_edges() {
    let window = fixture(ScrollAxes::Both, true);
    assert!(window.get_scrollable());
    tap(&window, Key::Tab);
    tap(&window, Key::RightArrow);
    near(window.get_offset_x(), -window.get_line_step());
    tap(&window, Key::LeftArrow);
    near(window.get_offset_x(), 0.);
    tap(&window, Key::DownArrow);
    near(window.get_offset_y(), -window.get_line_step());
    tap(&window, Key::End);
    assert!(window.get_at_end(), "the vertical axis is primary");
    near(window.get_offset_x(), 0.);
    // Content 600px wide in a viewport narrowed by the vertical gutter.
    for _ in 0..40 {
        tap(&window, Key::RightArrow);
    }
    near(window.get_offset_x(), -(600. - window.get_viewport_width()));
}

#[test]
fn horizontal_viewports_use_the_horizontal_axis_for_every_key() {
    let window = fixture(ScrollAxes::Horizontal, true);
    tap(&window, Key::Tab);
    tap(&window, Key::DownArrow);
    near(window.get_offset_y(), 0.);
    near(window.get_offset_x(), 0.);
    tap(&window, Key::RightArrow);
    near(window.get_offset_x(), -window.get_line_step());
    tap(&window, Key::End);
    near(window.get_offset_x(), -(600. - window.get_viewport_width()));
    assert!(window.get_at_end());
    tap(&window, Key::Home);
    assert!(window.get_at_start());
    window.invoke_to_end();
    assert!(window.get_at_end());
    window.invoke_to_start();
    near(window.get_offset_x(), 0.);
}

#[test]
fn edges_are_reported_once_when_reached_and_not_for_content_that_fits() {
    let window = fixture(ScrollAxes::Vertical, true);
    assert_eq!((window.get_starts(), window.get_ends()), (0, 0));
    window.invoke_to_end();
    slint::platform::update_timers_and_animations();
    assert_eq!((window.get_starts(), window.get_ends()), (0, 1));
    window.invoke_to_end();
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_ends(), 1);
    window.invoke_to_start();
    slint::platform::update_timers_and_animations();
    assert_eq!((window.get_starts(), window.get_ends()), (1, 1));
    // Content that fits is at both edges and reports neither.
    window.invoke_resize_content(10.);
    slint::platform::update_timers_and_animations();
    assert!(!window.get_scrollable());
    assert!(window.get_at_start() && window.get_at_end());
    assert_eq!((window.get_starts(), window.get_ends()), (1, 1));
}

#[test]
fn pressing_the_viewport_background_gives_it_the_keyboard() {
    let window = fixture(ScrollAxes::Vertical, true);
    let point = LogicalPosition::new(100., 100.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: point });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: point,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: point,
            button: PointerEventButton::Left,
        });
    assert!(!window.get_ring(), "a pointer press shows no ring");
    tap(&window, Key::DownArrow);
    near(window.get_offset_y(), -window.get_line_step());
}

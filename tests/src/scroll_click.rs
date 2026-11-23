// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ScrollClickFixture;
use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{PointerEventButton, WindowEvent},
};
use slint_kit::ui::ScrollPanPolicy;

fn fixture(policy: ScrollPanPolicy) -> ScrollClickFixture {
    let window = ScrollClickFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_policy(policy);
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}

fn press(window: &ScrollClickFixture, at: (f32, f32), to: (f32, f32)) {
    let down = LogicalPosition::new(at.0, at.1);
    let up = LogicalPosition::new(to.0, to.1);
    let events = window.window();
    events.dispatch_event(WindowEvent::PointerMoved { position: down });
    events.dispatch_event(WindowEvent::PointerPressed {
        position: down,
        button: PointerEventButton::Left,
    });
    events.dispatch_event(WindowEvent::PointerMoved { position: up });
    events.dispatch_event(WindowEvent::PointerReleased {
        position: up,
        button: PointerEventButton::Left,
    });
    slint::platform::update_timers_and_animations();
}

#[test]
fn empty_background_reports_a_click_and_content_keeps_its_own() {
    i_slint_backend_testing::init_no_event_loop();
    for policy in [ScrollPanPolicy::Native, ScrollPanPolicy::ChildrenFirst] {
        let window = fixture(policy);
        press(&window, (150., 150.), (150., 150.));
        assert_eq!(window.get_background_clicks(), 1, "{policy:?}");
        let button = ElementHandle::find_by_accessible_label(&window, "Inside")
            .find(|element| element.accessible_role() == Some(AccessibleRole::Button))
            .unwrap();
        let center = button.absolute_position();
        let size = button.size();
        let at = (center.x + size.width / 2., center.y + size.height / 2.);
        press(&window, at, at);
        assert_eq!(window.get_button_clicks(), 1, "{policy:?}");
        assert_eq!(window.get_background_clicks(), 1, "{policy:?}");
    }
}

#[test]
fn a_pan_gesture_is_not_a_background_click() {
    i_slint_backend_testing::init_no_event_loop();
    let window = fixture(ScrollPanPolicy::ChildrenFirst);
    press(&window, (150., 150.), (150., 90.));
    assert_eq!(window.get_background_clicks(), 0);
}

// SPDX-License-Identifier: GPL-3.0-or-later
use crate::SliderFocusFixture;
use i_slint_backend_testing::init_no_event_loop;
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};

#[test]
fn slider_reports_keyboard_focus_gain_and_loss() {
    init_no_event_loop();
    let window = SliderFocusFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    window.invoke_focus_outside();
    assert_eq!(window.get_focus_events(), 0);
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Tab.into(),
    });
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Tab.into(),
    });
    slint::platform::update_timers_and_animations();
    assert!(window.get_slider_focused());
    assert_eq!(window.get_focus_events(), 1);
    assert!(window.get_observed_focus());
    window.invoke_focus_outside();
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_focus_events(), 2);
    assert!(!window.get_observed_focus());
}

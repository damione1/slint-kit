// SPDX-License-Identifier: GPL-3.0-or-later
use crate::SliderRecoveryFixture;
use i_slint_backend_testing::{ElementHandle, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::ui::{DecimalRangeView, FieldAccess, Ownership, SliderSpec};
use std::{cell::RefCell, rc::Rc};
fn key(window: &SliderRecoveryFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
fn range(lower: &str, upper: &str) -> DecimalRangeView {
    DecimalRangeView {
        lower: lower.into(),
        upper: upper.into(),
    }
}
fn setup() -> SliderRecoveryFixture {
    let window = SliderRecoveryFixture::new().unwrap();
    assert!(!window.get_scalar_spec_valid());
    slint_kit::install(&window);
    window.show().unwrap();
    assert!(window.get_scalar_spec_valid() && window.get_range_spec_valid());
    assert_eq!(window.get_scalar(), "7");
    assert_eq!(window.get_range(), range("7", "83"));
    assert!(!window.get_scalar_value_valid() && !window.get_range_value_valid());
    window
}
#[test]
fn off_grid_and_loaded_bounds_recover_on_native_keys_without_controlled_ack() {
    i_slint_backend_testing::init_no_event_loop();
    let window = setup();
    assert!((window.get_scalar_position() - 0.07).abs() < 0.0001);
    window.invoke_focus_scalar();
    key(&window, Key::RightArrow);
    assert_eq!(window.get_scalar(), "10");
    assert!(window.get_scalar_value_valid());
    window.set_scalar("7".into());
    key(&window, Key::LeftArrow);
    assert_eq!(window.get_scalar(), "5");
    window.set_scalar("-20".into());
    assert_eq!(window.get_scalar_position(), 0.);
    key(&window, Key::RightArrow);
    assert_eq!(window.get_scalar(), "0");
    window.set_scalar("120".into());
    assert_eq!(window.get_scalar_position(), 1.);
    key(&window, Key::LeftArrow);
    assert_eq!(window.get_scalar(), "100");
    window.invoke_focus_range();
    key(&window, Key::RightArrow);
    assert_eq!(window.get_range(), range("10", "85"));
    assert!(window.get_range_value_valid());
    window.set_range(range("-20", "120"));
    assert_eq!(
        (window.get_lower_position(), window.get_upper_position()),
        (0., 1.)
    );
    key(&window, Key::RightArrow);
    assert_eq!(window.get_range(), range("0", "100"));
    let scalar = Rc::new(RefCell::new(Vec::new()));
    let seen = scalar.clone();
    window.on_scalar_chosen(move |value| seen.borrow_mut().push(value));
    let ranges = Rc::new(RefCell::new(Vec::new()));
    let seen = ranges.clone();
    window.on_range_chosen(move |value| seen.borrow_mut().push(value));
    window.set_ownership(Ownership::Controlled);
    window.set_scalar("7".into());
    window.set_range(range("7", "83"));
    window.invoke_focus_scalar();
    key(&window, Key::RightArrow);
    assert_eq!(window.get_scalar(), "7");
    assert_eq!(scalar.borrow().last().unwrap(), "10");
    window.invoke_focus_range();
    key(&window, Key::LeftArrow);
    assert_eq!(window.get_range(), range("7", "83"));
    assert_eq!(ranges.borrow().last().unwrap(), &range("5", "85"));
    window.set_range(ranges.borrow().last().unwrap().clone());
    assert!(window.get_range_value_valid());
}
#[test]
fn pointer_recovery_escape_access_and_bad_specs_preserve_admission() {
    i_slint_backend_testing::init_no_event_loop();
    let window = setup();
    let slider = ElementHandle::find_by_accessible_label(&window, "Recovery slider")
        .next()
        .unwrap();
    slider.mock_single_click(PointerEventButton::Left);
    assert_eq!(window.get_scalar(), "50");
    let ranged = ElementHandle::find_by_accessible_label(&window, "Recovery range")
        .next()
        .unwrap();
    ranged.mock_single_click(PointerEventButton::Left);
    assert_eq!(window.get_range(), range("5", "50"));
    window.set_scalar("7".into());
    let origin = slider.absolute_position();
    let size = slider.size();
    let position = LogicalPosition::new(origin.x + size.width * 0.75, origin.y + size.height / 2.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    assert!(window.get_scalar_value_valid());
    key(&window, Key::Escape);
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    assert_eq!(window.get_scalar(), "7");
    assert!(!window.get_scalar_value_valid());
    for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
        window.set_access(access);
        window.invoke_focus_scalar();
        key(&window, Key::RightArrow);
        slider.mock_single_click(PointerEventButton::Left);
        assert_eq!(window.get_scalar(), "7");
        window.set_range(range("7", "83"));
        window.invoke_focus_range();
        key(&window, Key::RightArrow);
        ranged.mock_single_click(PointerEventButton::Left);
        assert_eq!(window.get_range(), range("7", "83"));
    }
    window.set_access(FieldAccess::Editable);
    let accepted = Rc::new(RefCell::new(Vec::new()));
    let seen = accepted.clone();
    window.on_scalar_chosen(move |value| seen.borrow_mut().push(value));
    for spec in [
        SliderSpec {
            minimum: "0".into(),
            maximum: "100".into(),
            step: "0".into(),
        },
        SliderSpec {
            minimum: "100".into(),
            maximum: "0".into(),
            step: "5".into(),
        },
        SliderSpec {
            minimum: "bad".into(),
            maximum: "100".into(),
            step: "5".into(),
        },
    ] {
        window.set_spec(spec);
        slint::platform::update_timers_and_animations();
        assert!(!window.get_scalar_spec_valid() && !window.get_range_spec_valid());
        assert!(!window.get_scalar_value_valid() && !window.get_range_value_valid());
        window.invoke_focus_scalar();
        key(&window, Key::RightArrow);
        slider.mock_single_click(PointerEventButton::Left);
        assert_eq!(window.get_scalar(), "7");
        ranged.mock_single_click(PointerEventButton::Left);
        assert_eq!(window.get_range(), range("7", "83"));
    }
    assert!(accepted.borrow().is_empty());
    window.set_spec(SliderSpec {
        minimum: "0".into(),
        maximum: "100".into(),
        step: "5".into(),
    });
    window.set_scalar("bad".into());
    window.set_range(range("80", "20"));
    assert!(window.get_scalar_spec_valid());
    assert!(!window.get_scalar_value_valid() && !window.get_range_value_valid());
}
#[test]
fn loaded_recovered_and_invalid_slider_states_render_in_both_themes() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    for (mode, dark) in [("light", false), ("dark", true)] {
        for scale in [1., 2.] {
            let window = setup();
            let mut theme = if dark {
                window.global::<slint_kit::ui::KitThemes>().get_dark()
            } else {
                window.get_theme()
            };
            theme.text_scale = scale;
            theme.reduced_motion = true;
            window.set_theme(theme);
            let frame = || {
                slint::platform::update_timers_and_animations();
                window.window().take_snapshot().unwrap()
            };
            window.set_range(range("-20", "120"));
            crate::choice_presentation::save_capture(
                &frame(),
                &format!("slider-recovery-loaded-{mode}-{scale}"),
            );
            window.set_range(range("7", "83"));
            window.invoke_focus_scalar();
            key(&window, Key::RightArrow);
            window.invoke_focus_range();
            key(&window, Key::RightArrow);
            assert!(window.get_scalar_value_valid() && window.get_range_value_valid());
            crate::choice_presentation::save_capture(
                &frame(),
                &format!("slider-recovery-repaired-{mode}-{scale}"),
            );
            window.set_spec(SliderSpec {
                minimum: "0".into(),
                maximum: "100".into(),
                step: "0".into(),
            });
            crate::choice_presentation::save_capture(
                &frame(),
                &format!("slider-recovery-invalid-{mode}-{scale}"),
            );
        }
    }
}

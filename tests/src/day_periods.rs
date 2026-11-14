// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{DayPeriodsFixture, LocalTimeValue, LocalTimeView};
use i_slint_backend_testing::{ElementHandle, TestingBackend, TestingBackendOptions};
use slint::{ComponentHandle, platform::PointerEventButton};

fn init() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
}
fn frame(app: &impl ComponentHandle) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
}
fn value(hour: i32, minute: i32) -> LocalTimeValue {
    LocalTimeValue {
        present: true,
        time: LocalTimeView {
            hour,
            minute,
            second: 0,
        },
    }
}

#[test]
fn twelve_hour_editors_print_parse_and_step_with_the_window_markers() {
    init();
    let app = DayPeriodsFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app.set_value(value(0, 5));
    frame(&app);
    assert_eq!(app.get_text(), "12:05 AM");
    assert_eq!(app.get_placeholder(), "hh:mm AM");

    app.invoke_localize("午前".into(), "午後".into());
    app.set_value(value(0, 6));
    frame(&app);
    assert_eq!(app.get_text(), "12:06 午前");
    assert_eq!(app.get_placeholder(), "hh:mm 午前");
    app.set_value(value(23, 5));
    frame(&app);
    assert_eq!(app.get_text(), "11:05 午後");

    // The spin-button path parses and prints with the same markers.
    ElementHandle::find_by_accessible_label(&app, "Increase Hours")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    frame(&app);
    assert_eq!(app.get_value(), value(0, 5));
    assert_eq!(app.get_text(), "12:05 午前");
}

#[test]
fn an_unusable_marker_pair_falls_back_to_am_and_pm() {
    init();
    let app = DayPeriodsFixture::new().unwrap();
    slint_kit::install(&app);
    app.invoke_localize("x".into(), "X".into());
    app.set_value(value(13, 30));
    frame(&app);
    assert_eq!(app.get_text(), "01:30 PM");
}

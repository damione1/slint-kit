// SPDX-License-Identifier: GPL-3.0-or-later
//! Date popup reopening, keyboard open and keyboard stepping; headless only.
use crate::{CivilDateView, DatePopupKeyboardFixture, FieldAccess, TimePrecision};
use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};

fn fixture() -> DatePopupKeyboardFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = DatePopupKeyboardFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    settle(&app);
    app
}
fn settle(app: &DatePopupKeyboardFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
}
fn key(app: &DatePopupKeyboardFixture, value: impl Into<slint::SharedString>) {
    let text = value.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle(app);
}
fn with_alt(app: &DatePopupKeyboardFixture, value: Key) {
    let alt: slint::SharedString = Key::Alt.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: alt.clone() });
    key(app, value);
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: alt });
    settle(app);
}
fn date(year: i32, month: i32, day: i32) -> CivilDateView {
    CivilDateView { year, month, day }
}

#[test]
fn popup_reopens_on_the_accepted_date_after_browsing_and_after_a_host_change() {
    let app = fixture();
    app.set_open(true);
    settle(&app);
    assert_eq!(app.get_picker_focused(), date(2026, 10, 2));
    // Browse two months ahead, then close without choosing.
    key(&app, Key::PageDown);
    key(&app, Key::PageDown);
    assert_ne!(app.get_picker_focused(), date(2026, 10, 2));
    app.set_open(false);
    settle(&app);
    app.set_open(true);
    settle(&app);
    assert_eq!(app.get_picker_focused(), date(2026, 10, 2));
    app.set_open(false);
    settle(&app);
    // A host change of the accepted date moves the reopened popup too.
    app.set_accepted(date(2026, 3, 15));
    settle(&app);
    app.set_open(true);
    settle(&app);
    assert_eq!(app.get_picker_focused(), date(2026, 3, 15));
}

#[test]
fn alt_down_and_f4_open_the_popup_from_the_editor_and_escape_closes_it() {
    let app = fixture();
    app.invoke_focus_editor();
    settle(&app);
    with_alt(&app, Key::DownArrow);
    assert!(app.get_open());
    key(&app, Key::Escape);
    assert!(!app.get_open());
    app.invoke_focus_editor();
    key(&app, Key::F4);
    assert!(app.get_open());
}

#[test]
fn alt_down_does_not_open_a_read_only_or_disabled_field() {
    let app = fixture();
    for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
        app.set_access(access);
        settle(&app);
        app.invoke_focus_editor();
        with_alt(&app, Key::DownArrow);
        key(&app, Key::F4);
        assert!(!app.get_open());
    }
}

#[test]
fn arrow_keys_step_the_date_draft_and_page_keys_step_months_within_bounds() {
    let app = fixture();
    app.invoke_focus_editor();
    settle(&app);
    key(&app, Key::UpArrow);
    assert_eq!(app.get_text(), "2026-10-03");
    key(&app, Key::DownArrow);
    key(&app, Key::DownArrow);
    assert_eq!(app.get_text(), "2026-10-01");
    key(&app, Key::PageUp);
    assert_eq!(app.get_text(), "2026-11-01");
    app.set_text("2026-12-31".into());
    settle(&app);
    key(&app, Key::UpArrow);
    assert_eq!(app.get_text(), "2026-12-31", "clamped at max");
    assert!(!app.get_open());
}

#[test]
fn a_draft_that_is_not_a_date_is_left_to_the_native_editor() {
    let app = fixture();
    app.invoke_focus_editor();
    app.set_text("2026-1".into());
    settle(&app);
    key(&app, Key::UpArrow);
    assert_eq!(app.get_text(), "2026-1");
}

#[test]
fn time_field_arrow_keys_step_minutes_or_seconds_and_page_keys_step_hours() {
    let app = fixture();
    app.invoke_focus_time();
    app.set_time_text("09:30".into());
    settle(&app);
    key(&app, Key::UpArrow);
    assert_eq!(app.get_time_text(), "09:31");
    key(&app, Key::DownArrow);
    key(&app, Key::DownArrow);
    assert_eq!(app.get_time_text(), "09:29");
    key(&app, Key::PageUp);
    assert_eq!(app.get_time_text(), "10:29");
    key(&app, Key::PageDown);
    key(&app, Key::PageDown);
    assert_eq!(app.get_time_text(), "08:29");
    app.set_precision(TimePrecision::Second);
    app.set_time_text("08:29:10".into());
    settle(&app);
    key(&app, Key::UpArrow);
    assert_eq!(app.get_time_text(), "08:29:11");
}

#[test]
fn standalone_calendar_leaves_escape_to_the_host_when_escape_cancels_is_false() {
    let app = fixture();
    app.invoke_focus_calendar();
    settle(&app);
    key(&app, Key::Escape);
    assert_eq!(app.get_cancels(), 1);
    app.set_escape_cancels(false);
    settle(&app);
    key(&app, Key::Escape);
    assert_eq!(app.get_cancels(), 1);
}

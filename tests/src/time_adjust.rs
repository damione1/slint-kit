// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    ClockFormat, CommitPolicy, FieldAccess, LocalTimeValue, LocalTimeView, Ownership,
    TimeAdjustFixture, TimePrecision,
};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{cell::RefCell, rc::Rc};
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
fn app() -> TimeAdjustFixture {
    let app = TimeAdjustFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    frame(&app);
    app
}
fn value(hour: i32, minute: i32, second: i32) -> LocalTimeValue {
    LocalTimeValue {
        present: true,
        time: LocalTimeView {
            hour,
            minute,
            second,
        },
    }
}
fn button(app: &TimeAdjustFixture, label: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(app, label)
        .next()
        .unwrap()
}
fn click(app: &TimeAdjustFixture, label: &str) {
    button(app, label).mock_single_click(PointerEventButton::Left);
    frame(app);
}
fn key(app: &impl ComponentHandle, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    frame(app);
}
fn select_previous_character(app: &impl ComponentHandle) {
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    key(app, Key::LeftArrow);
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    frame(app);
}
fn menu(app: &TimeAdjustFixture, label: &str) {
    key(app, Key::Menu);
    let label = label.to_owned();
    ElementQuery::from_root(app)
        .match_inherits("KitOption")
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label.as_str()))
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    frame(app);
}
#[test]
fn empty_buttons_seed_once_then_follow_a_900_second_step_and_real_typed_commits() {
    init();
    for first in ["Increase Minutes", "Decrease Minutes"] {
        let app = app();
        app.set_commit_policy(CommitPolicy::OnSubmit);
        app.set_min("09:00".into());
        app.set_max("17:00".into());
        app.set_step("900".into());
        frame(&app);
        let typed = Rc::new(RefCell::new(Vec::new()));
        let received = typed.clone();
        app.on_committed(move |commit| {
            received.borrow_mut().push(
                slint_kit::FieldSpec::time(
                    slint_kit::TimePrecision::Minute,
                    slint_kit::ClockFormat::TwentyFourHour,
                )
                .accept(commit)
                .unwrap(),
            )
        });
        click(&app, first);
        assert_eq!(app.get_value(), value(9, 0, 0));
        assert_eq!(app.get_choices(), 1);
        click(&app, "Increase Minutes");
        assert_eq!(app.get_value(), value(9, 15, 0));
        click(&app, "Increase Hours");
        assert_eq!(app.get_value(), value(10, 15, 0));
        assert!(app.get_validity().valid);
        assert_eq!(
            *typed.borrow(),
            vec![
                Some(slint_kit::LocalTime::new(9, 0, 0).unwrap()),
                Some(slint_kit::LocalTime::new(9, 15, 0).unwrap()),
                Some(slint_kit::LocalTime::new(10, 15, 0).unwrap())
            ]
        );
        app.invoke_focus_after();
        frame(&app);
        assert_eq!(
            app.get_choices(),
            3,
            "blur does not repeat adjustment proposals"
        );
        app.invoke_focus_editor();
        menu(&app, "Select all");
        key(&app, "11:00");
        app.invoke_focus_after();
        frame(&app);
        assert_eq!(
            app.get_value(),
            value(10, 15, 0),
            "on-submit keeps native typing provisional on blur"
        );
        assert_eq!(typed.borrow().len(), 3);
        app.invoke_focus_editor();
        key(&app, Key::Return);
        app.invoke_focus_after();
        frame(&app);
        assert_eq!(app.get_choices(), 4, "Enter accepts the native draft once");
        assert_eq!(
            typed.borrow().last(),
            Some(&Some(slint_kit::LocalTime::new(11, 0, 0).unwrap()))
        );
        app.hide().unwrap();
    }
}
#[test]
fn adjustment_obeys_bounds_midnight_carry_periodic_ranges_and_precision_without_rounding() {
    init();
    for (initial, step, min, max, precision, clock, action, expected) in [
        (
            "23:45",
            "900",
            "",
            "",
            TimePrecision::Minute,
            ClockFormat::TwentyFourHour,
            "Increase Minutes",
            "00:00",
        ),
        (
            "10:59",
            "60",
            "",
            "",
            TimePrecision::Minute,
            ClockFormat::TwentyFourHour,
            "Increase Minutes",
            "11:00",
        ),
        (
            "23:59:59",
            "1",
            "",
            "",
            TimePrecision::Second,
            ClockFormat::TwentyFourHour,
            "Increase Seconds",
            "00:00:00",
        ),
        (
            "23:45",
            "900",
            "23:00",
            "02:00",
            TimePrecision::Minute,
            ClockFormat::TwentyFourHour,
            "Increase Minutes",
            "00:00",
        ),
        (
            "09:30",
            "900",
            "09:00",
            "09:30",
            TimePrecision::Minute,
            ClockFormat::TwentyFourHour,
            "Increase Minutes",
            "09:30",
        ),
        (
            "23:59",
            "0",
            "malformed",
            "malformed",
            TimePrecision::Minute,
            ClockFormat::TwentyFourHour,
            "Increase Minutes",
            "00:00",
        ),
        (
            "09:05",
            "AnY",
            "09:05",
            "09:07",
            TimePrecision::Minute,
            ClockFormat::TwentyFourHour,
            "Increase Minutes",
            "09:06",
        ),
        (
            "",
            "1",
            "09:00:00.1",
            "09:00:00.9",
            TimePrecision::Second,
            ClockFormat::TwentyFourHour,
            "Increase Seconds",
            "",
        ),
        (
            "",
            "60",
            "09:00:30",
            "",
            TimePrecision::Minute,
            ClockFormat::TwentyFourHour,
            "Increase Minutes",
            "",
        ),
        (
            "",
            "30",
            "09:00:30",
            "",
            TimePrecision::Minute,
            ClockFormat::TwentyFourHour,
            "Increase Minutes",
            "09:01",
        ),
        (
            "11:59 PM",
            "60",
            "",
            "",
            TimePrecision::Minute,
            ClockFormat::TwelveHour,
            "Increase Minutes",
            "12:00 AM",
        ),
        (
            "09:",
            "900",
            "",
            "",
            TimePrecision::Minute,
            ClockFormat::TwentyFourHour,
            "Increase Minutes",
            "09:",
        ),
    ] {
        let app = TimeAdjustFixture::new().unwrap();
        app.set_precision(precision);
        app.set_clock(clock);
        app.set_step(step.into());
        app.set_min(min.into());
        app.set_max(max.into());
        app.set_text(initial.into());
        slint_kit::install(&app);
        app.show().unwrap();
        frame(&app);
        let typed = Rc::new(RefCell::new(Vec::new()));
        let received = typed.clone();
        app.on_committed(move |commit| {
            received.borrow_mut().push(
                slint_kit::FieldSpec::time(precision.into(), clock.into())
                    .accept(commit)
                    .unwrap(),
            )
        });
        click(&app, action);
        assert_eq!(
            app.get_text(),
            expected,
            "{initial} step={step} min={min} max={max}"
        );
        if expected != initial {
            assert!(app.get_value().present);
            assert!(app.get_validity().valid);
            assert_eq!(app.get_proposed(), app.get_value().time);
            assert_eq!(
                typed.borrow().as_slice(),
                &[Some(
                    slint_kit::LocalTime::try_from(app.get_proposed()).unwrap()
                )]
            );
        } else {
            assert_eq!(app.get_choices(), 0);
            assert!(typed.borrow().is_empty());
        }
        app.hide().unwrap();
    }
    let app = app();
    app.set_min("23:00".into());
    app.set_max("02:00".into());
    app.set_step("900".into());
    app.set_value(value(23, 0, 0));
    frame(&app);
    click(&app, "Decrease Minutes");
    assert_eq!(app.get_text(), "23:00");
    assert_eq!(app.get_choices(), 0);
    app.invoke_focus_after();
    frame(&app);
    app.set_value(value(2, 0, 0));
    frame(&app);
    click(&app, "Increase Minutes");
    assert_eq!(app.get_text(), "02:00");
    assert_eq!(app.get_choices(), 0);
}
#[test]
fn controlled_adjustment_acknowledges_without_clobbering_native_caret_undo_or_access_guards() {
    init();
    let reference = crate::TemporalAcceptedNativeFixture::new().unwrap();
    reference.set_text("09:30".into());
    reference.show().unwrap();
    frame(&reference);
    reference.invoke_focus_editor();
    reference.invoke_select_editor();
    key(&reference, "09:45");
    key(&reference, Key::LeftArrow);
    select_previous_character(&reference);
    key(&reference, "0");
    let before = reference.get_text();
    assert_eq!(
        before, "09:05",
        "native selection replacement establishes the caret/selection expectation"
    );
    reference.invoke_undo_editor();
    frame(&reference);
    let undo_one = reference.get_text();
    reference.invoke_undo_editor();
    frame(&reference);
    let undo_two = reference.get_text();
    reference.hide().unwrap();
    assert_ne!(before, undo_one);
    assert_ne!(undo_one, undo_two);
    let app = app();
    app.set_ownership(Ownership::Controlled);
    app.set_step("900".into());
    app.set_value(value(9, 30, 0));
    frame(&app);
    app.invoke_focus_editor();
    let editor = ElementQuery::from_root(&app)
        .match_type_name("TextInput")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Adjustable time"))
        .find_first()
        .unwrap();
    click(&app, "Increase Minutes");
    assert_eq!(app.get_text(), "09:45");
    assert_eq!(app.get_value(), value(9, 30, 0));
    assert_eq!(app.get_proposed(), value(9, 45, 0).time);
    assert_eq!(app.get_choices(), 1);
    key(&app, Key::LeftArrow);
    select_previous_character(&app);
    app.set_value(value(9, 45, 0));
    frame(&app);
    key(&app, "0");
    assert_eq!(app.get_text(), before);
    assert!(editor.is_valid());
    menu(&app, "Undo");
    assert_eq!(app.get_text(), undo_one);
    menu(&app, "Undo");
    assert_eq!(app.get_text(), undo_two);
    assert!(editor.is_valid());
    app.invoke_cancel_editor();
    frame(&app);
    assert_eq!(app.get_text(), "09:45");
    app.invoke_focus_after();
    frame(&app);
    assert_eq!(app.get_choices(), 1);
    for access in [FieldAccess::Disabled, FieldAccess::ReadOnly] {
        app.set_access(access);
        frame(&app);
        button(&app, "Increase Hours").invoke_accessible_default_action();
        frame(&app);
        assert_eq!(app.get_text(), "09:45");
        assert_eq!(app.get_choices(), 1);
    }
    app.hide().unwrap();
    let uninstalled = TimeAdjustFixture::new().unwrap();
    uninstalled.show().unwrap();
    frame(&uninstalled);
    click(&uninstalled, "Increase Minutes");
    assert_eq!(uninstalled.get_text(), "");
    assert_eq!(uninstalled.get_choices(), 0);
}
#[test]
fn stepped_time_renders_in_both_themes_at_100_and_200_percent() {
    init();
    for dark in [false, true] {
        for scale in [1.0, 2.0] {
            let app = app();
            let mut theme = if dark {
                app.global::<slint_kit::ui::KitThemes>().get_dark()
            } else {
                app.global::<slint_kit::ui::KitThemes>().get_light()
            };
            theme.text_scale = scale;
            app.set_theme(theme);
            app.set_min("09:00".into());
            app.set_max("17:00".into());
            app.set_step("900".into());
            app.set_value(value(9, 30, 0));
            frame(&app);
            click(&app, "Increase Minutes");
            assert_eq!(app.get_text(), "09:45");
            assert!(app.get_validity().valid);
            for label in [
                "Increase Hours",
                "Decrease Hours",
                "Increase Minutes",
                "Decrease Minutes",
            ] {
                let button = button(&app, label);
                let p = button.absolute_position();
                let s = button.size();
                assert!(
                    p.x >= 0.
                        && p.y >= 0.
                        && s.width > 0.
                        && s.height > 0.
                        && p.x + s.width <= 560.
                        && p.y + s.height <= 520.
                );
            }
            if let Some(dir) = std::env::var_os("KIT_TIME_ADJUST_RENDER_DIR") {
                let dir = std::path::PathBuf::from(dir);
                std::fs::create_dir_all(&dir).unwrap();
                let pixels = app.window().take_snapshot().unwrap();
                let mut ppm =
                    format!("P6\n{} {}\n255\n", pixels.width(), pixels.height()).into_bytes();
                for p in pixels.as_slice() {
                    ppm.extend_from_slice(&[p.r, p.g, p.b]);
                }
                std::fs::write(
                    dir.join(format!(
                        "time-step-{}-{scale}.ppm",
                        if dark { "dark" } else { "light" }
                    )),
                    ppm,
                )
                .unwrap();
            }
            app.hide().unwrap();
        }
    }
}

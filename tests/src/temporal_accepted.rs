// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    CivilDateView, LocalDateTimeView, LocalTimeValue, LocalTimeView, Ownership,
    TemporalAcceptedFixture,
};
use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
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
fn fixture() -> TemporalAcceptedFixture {
    init();
    let app = TemporalAcceptedFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    frame(&app);
    app
}
fn time(hour: i32, minute: i32) -> LocalTimeValue {
    LocalTimeValue {
        present: true,
        time: LocalTimeView {
            hour,
            minute,
            second: 0,
        },
    }
}
fn date(day: i32) -> CivilDateView {
    CivilDateView {
        year: 2026,
        month: 11,
        day,
    }
}
fn key(app: &impl ComponentHandle, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    frame(app);
}
#[test]
fn all_seven_typed_inputs_install_update_and_clear_without_text_mirrors() {
    let app = fixture();
    assert_eq!(app.get_date_text(), "2026-11-11");
    assert_eq!(app.get_month_text(), "2026-11");
    assert_eq!(app.get_year_text(), "2026");
    assert_eq!(app.get_week_text(), "2020-W53");
    assert_eq!(app.get_time_text(), "00:00");
    assert_eq!(app.get_time_state().accepted.text, "00:00");
    assert!(app.get_time_state().accepted.present);
    assert_eq!(app.get_picker_text(), "09:30");
    assert_eq!(
        (app.get_pair_date(), app.get_pair_time()),
        ("2026-11-11".into(), "12:15".into())
    );
    app.set_date_value(date(12));
    app.set_month(12);
    app.set_year(2027);
    app.set_week_year(2026);
    app.set_week(46);
    app.set_time_value(time(23, 59));
    app.set_picker_value(time(0, 0));
    app.set_pair_value(LocalDateTimeView {
        date: date(13),
        time: time(10, 15).time,
    });
    frame(&app);
    assert_eq!(app.get_date_text(), "2026-11-12");
    assert_eq!(app.get_month_text(), "2026-12");
    assert_eq!(app.get_year_text(), "2027");
    assert_eq!(app.get_week_text(), "2026-W46");
    assert_eq!(app.get_time_text(), "23:59");
    assert_eq!(app.get_picker_text(), "00:00");
    assert_eq!(
        (app.get_pair_date(), app.get_pair_time()),
        ("2026-11-13".into(), "10:15".into())
    );
    app.set_month(13);
    app.set_week_year(2021);
    app.set_week(53);
    app.set_time_value(LocalTimeValue {
        present: true,
        time: LocalTimeView {
            hour: 24,
            minute: 0,
            second: 0,
        },
    });
    frame(&app);
    assert_eq!(app.get_month_text(), "2026-12");
    assert_eq!(app.get_week_text(), "2026-W46");
    assert_eq!(app.get_time_text(), "23:59");
    app.set_date_value(CivilDateView::default());
    app.set_month_year(0);
    app.set_month(0);
    app.set_year(0);
    app.set_week_year(0);
    app.set_week(0);
    app.set_time_value(LocalTimeValue::default());
    app.set_picker_value(LocalTimeValue::default());
    app.set_pair_value(LocalDateTimeView::default());
    frame(&app);
    for text in [
        app.get_date_text(),
        app.get_month_text(),
        app.get_year_text(),
        app.get_week_text(),
        app.get_time_text(),
        app.get_picker_text(),
        app.get_pair_date(),
        app.get_pair_time(),
    ] {
        assert_eq!(text, "");
    }
    assert!(!app.get_time_state().accepted.present);
}
#[test]
fn host_ack_preserves_focused_native_selection_draft_undo_and_cancel_baseline() {
    let app = fixture();
    let typed = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let received = typed.clone();
    app.on_time_commit(move |commit| {
        let value = slint_kit::FieldSpec::time(
            slint_kit::TimePrecision::Minute,
            slint_kit::ClockFormat::TwentyFourHour,
        )
        .accept(commit)
        .unwrap();
        received.borrow_mut().push(value);
    });
    let native = crate::TemporalAcceptedNativeFixture::new().unwrap();
    native.set_text("00:00".into());
    native.show().unwrap();
    frame(&native);
    native.set_text("09:30".into());
    frame(&native);
    native.invoke_focus_editor();
    native.invoke_select_editor();
    key(&native, "09:31");
    native.invoke_undo_editor();
    frame(&native);
    let native_undo = native.get_text();
    native.hide().unwrap();
    app.set_time_value(time(9, 30));
    frame(&app);
    app.invoke_focus_time();
    app.invoke_select_time();
    key(&app, "09:31");
    assert_eq!(app.get_time_text(), "09:31");
    let caret = app.get_time_caret();
    let anchor = app.get_time_anchor();
    app.set_time_value(time(11, 45));
    frame(&app);
    assert_eq!(app.get_time_text(), "09:31");
    assert_eq!(app.get_time_caret(), caret);
    assert_eq!(app.get_time_anchor(), anchor);
    assert_eq!(app.get_time_state().accepted.text, "11:45");
    app.invoke_undo_time();
    frame(&app);
    assert_eq!(app.get_time_text(), native_undo);
    app.invoke_cancel_time();
    frame(&app);
    assert_eq!(app.get_time_text(), "11:45");
    app.invoke_select_time();
    key(&app, "12:00");
    key(&app, Key::Return);
    assert_eq!(
        typed.borrow().last(),
        Some(&Some(slint_kit::LocalTime::new(12, 0, 0).unwrap()))
    );
    app.set_time_value(LocalTimeValue::default());
    frame(&app);
    assert_eq!(app.get_time_text(), "12:00");
    assert!(!app.get_time_state().accepted.present);
    app.invoke_cancel_time();
    frame(&app);
    assert_eq!(app.get_time_text(), "");
    key(&app, Key::Return);
    assert_eq!(typed.borrow().last(), Some(&None));
}
#[test]
fn picker_local_and_controlled_typed_commit_clear_acknowledgment_deduplicate() {
    let app = fixture();
    app.invoke_picker_draft("00:00".into());
    app.invoke_commit_picker();
    frame(&app);
    assert_eq!(app.get_picker_value(), time(0, 0));
    assert_eq!(app.get_picker_choices(), 1);
    app.set_ownership(Ownership::Controlled);
    app.invoke_focus_picker();
    app.invoke_picker_draft("10:45".into());
    app.invoke_commit_picker();
    frame(&app);
    assert_eq!(app.get_picker_value(), time(0, 0));
    assert_eq!(app.get_picker_choices(), 2);
    assert_eq!(app.get_last_time(), time(10, 45).time);
    app.invoke_focus_after();
    frame(&app);
    assert_eq!(app.get_picker_choices(), 2);
    app.set_picker_value(time(10, 45));
    frame(&app);
    assert_eq!(app.get_picker_text(), "10:45");
    app.invoke_picker_draft("".into());
    app.invoke_commit_picker();
    frame(&app);
    assert_eq!(app.get_picker_clears(), 1);
    assert!(app.get_picker_value().present);
    app.set_picker_value(LocalTimeValue::default());
    frame(&app);
    assert_eq!(app.get_picker_text(), "");
    app.set_picker_value(time(0, 0));
    frame(&app);
    assert_eq!(app.get_picker_text(), "00:00");
}
#[test]
fn datetime_initial_host_projection_is_atomic_and_controlled_proposals_do_not_repeat() {
    let app = fixture();
    app.set_ownership(Ownership::Controlled);
    app.invoke_pair_draft("2026-11-12".into(), "".into());
    app.invoke_commit_pair();
    frame(&app);
    assert_eq!(app.get_pair_choices(), 0);
    app.invoke_pair_draft("2026-11-12".into(), "00:00".into());
    app.invoke_commit_pair();
    frame(&app);
    assert_eq!(app.get_pair_choices(), 1);
    assert_eq!(
        app.get_last_pair(),
        LocalDateTimeView {
            date: date(12),
            time: time(0, 0).time
        }
    );
    assert_eq!(app.get_pair_value().date, date(11));
    app.invoke_commit_pair();
    frame(&app);
    assert_eq!(app.get_pair_choices(), 1);
    app.set_pair_value(LocalDateTimeView {
        date: date(12),
        time: time(0, 0).time,
    });
    frame(&app);
    assert_eq!(app.get_pair_time(), "00:00");
    app.invoke_focus_pair();
    key(&app, "2");
    let draft = app.get_pair_date();
    app.set_pair_value(LocalDateTimeView {
        date: date(14),
        time: time(18, 30).time,
    });
    frame(&app);
    assert_eq!(app.get_pair_date(), draft);
    assert_eq!(app.get_pair_time(), "00:00");
    app.invoke_focus_after();
    frame(&app);
    assert_eq!(app.get_pair_date(), "2026-11-14");
    assert_eq!(app.get_pair_time(), "18:30");
}
#[test]
fn explicit_empty_draft_and_dynamic_post_install_fields_preserve_midnight_baseline() {
    init();
    let app = TemporalAcceptedFixture::new().unwrap();
    app.invoke_explicit_time("".into());
    slint_kit::install(&app);
    app.show().unwrap();
    frame(&app);
    assert_eq!(app.get_time_text(), "");
    assert!(app.get_time_state().accepted.present);
    assert_eq!(app.get_time_state().accepted.text, "00:00");
    let dynamic = crate::DynamicTimeFixture::new().unwrap();
    slint_kit::install(&dynamic);
    dynamic.set_mounted(true);
    dynamic.show().unwrap();
    frame(&dynamic);
    assert!(dynamic.get_state().accepted.present);
    assert_eq!(dynamic.get_state().accepted.text, "00:00");
    assert_eq!(dynamic.get_current_text(), "");
    app.invoke_cancel_time();
    frame(&app);
    assert_eq!(app.get_time_text(), "00:00");
}
#[test]
fn typed_temporal_projection_renders_in_light_dark_and_both_text_scales() {
    let app = fixture();
    for dark in [false, true] {
        for scale in [1.0, 2.0] {
            let mut theme = if dark {
                app.global::<slint_kit::ui::KitThemes>().get_dark()
            } else {
                app.global::<slint_kit::ui::KitThemes>().get_light()
            };
            theme.text_scale = scale;
            app.set_theme(theme);
            frame(&app);
            assert_eq!(app.get_time_text(), "00:00");
            assert_eq!(app.get_date_text(), "2026-11-11");
            if let Some(directory) = std::env::var_os("KIT_TEMPORAL_RENDER_DIR") {
                let path = std::path::Path::new(&directory);
                std::fs::create_dir_all(path).unwrap();
                let pixels = app.window().take_snapshot().unwrap();
                let mut output =
                    format!("P6\n{} {}\n255\n", pixels.width(), pixels.height()).into_bytes();
                for pixel in pixels.as_slice() {
                    output.extend_from_slice(&[pixel.r, pixel.g, pixel.b]);
                }
                std::fs::write(
                    path.join(format!(
                        "temporal-{}-{}.ppm",
                        if dark { "dark" } else { "light" },
                        scale
                    )),
                    output,
                )
                .unwrap();
            }
        }
    }
}

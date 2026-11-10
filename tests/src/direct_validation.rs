// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{AllDirectReportsFixture, DirectValidationFixture, ZonedReportFocusFixture};
use slint::{ComponentHandle, Model};

#[test]
fn all_constraint_reports_preserve_focus_and_skip_disabled_controls() {
    i_slint_backend_testing::init_no_event_loop();
    let app = AllDirectReportsFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app.invoke_focus_sentinel();
    let results = app.invoke_report_all();
    assert_eq!(results.row_count(), 17);
    assert!(results.iter().all(|valid| !valid));
    assert!(
        app.get_sentinel_focus(),
        "reporting must preserve the caller's focus across all families"
    );
    assert_eq!(
        app.get_invalid_count(),
        18,
        "all17 controls report, including both zoned parts"
    );
    app.set_access(slint_kit::ui::FieldAccess::Disabled);
    assert!(app.invoke_report_all().iter().all(|valid| valid));
    assert!(
        app.invoke_focus_all_invalid()
            .iter()
            .all(|focused| !focused)
    );
    assert_eq!(app.get_invalid_count(), 18);
    assert!(app.get_sentinel_focus());
}

#[test]
fn explicit_invalid_focus_is_separate_from_reporting_for_mixed_controls() {
    i_slint_backend_testing::init_no_event_loop();
    let app = DirectValidationFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app.invoke_focus_sentinel();
    assert!(!app.invoke_report_consent());
    assert!(app.get_sentinel_focus());
    assert!(!app.invoke_report_selection());
    assert!(app.get_sentinel_focus());
    assert!(!app.invoke_report_attachment());
    assert!(app.get_sentinel_focus());
    assert!(!app.invoke_report_meeting());
    assert!(app.get_sentinel_focus());
    assert!(app.invoke_focus_invalid_consent());
    assert!(app.get_consent_focus());
    app.invoke_focus_sentinel();
    assert!(app.invoke_focus_invalid_selection());
    assert!(!app.get_sentinel_focus());
    app.invoke_focus_sentinel();
    assert!(app.invoke_focus_invalid_attachment());
    assert!(!app.get_sentinel_focus());
    app.invoke_focus_sentinel();
    assert!(app.invoke_focus_invalid_meeting());
    assert!(!app.get_sentinel_focus());
}

#[test]
fn direct_constraint_errors_and_explicit_focus_render_in_both_themes_and_double_text() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = DirectValidationFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            app.invoke_focus_sentinel();
            assert!(!app.invoke_report_consent());
            assert!(!app.invoke_report_selection());
            assert!(!app.invoke_report_attachment());
            assert!(!app.invoke_report_meeting());
            assert!(app.get_sentinel_focus());
            assert!(app.invoke_focus_invalid_consent());
            assert!(app.get_consent_focus());
            crate::choice_presentation::save_capture(
                &app.window().take_snapshot().unwrap(),
                &format!("direct-report-{name}-{scale}"),
            );
        }
    }
}

#[test]
fn zoned_reporting_collects_both_invalid_parts_without_focus_then_explicitly_focuses_once() {
    i_slint_backend_testing::init_no_event_loop();
    let app = ZonedReportFocusFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app.invoke_focus_sentinel();
    assert!(!app.invoke_report_zoned());
    assert_eq!(
        app.get_invalid_count(),
        2,
        "both local time and zone report their constraint failures"
    );
    assert!(app.get_sentinel_focus());
    assert!(!app.get_zoned_focus());
    assert!(app.invoke_focus_invalid_zoned());
    assert!(app.get_zoned_focus());
    assert!(!app.get_sentinel_focus());
    assert_eq!(
        app.get_invalid_count(),
        2,
        "focusing does not report the constraints again"
    );
    app.window()
        .dispatch_event(slint::platform::WindowEvent::KeyPressed { text: "2".into() });
    app.window()
        .dispatch_event(slint::platform::WindowEvent::KeyReleased { text: "2".into() });
    slint::platform::update_timers_and_animations();
    assert_eq!(
        app.get_date_text(),
        "2",
        "native typing reaches the first invalid local editor"
    );
    assert_eq!(app.get_zone_text(), "");
    app.set_date_text("2026-11-10".into());
    app.set_time_text("09:00".into());
    app.invoke_focus_sentinel();
    slint::platform::update_timers_and_animations();
    assert!(!app.invoke_report_zoned());
    assert!(app.get_sentinel_focus());
    assert_eq!(
        app.get_invalid_count(),
        3,
        "only the remaining invalid zone reports"
    );
    assert!(app.invoke_focus_invalid_zoned());
    assert!(app.get_zoned_focus());
    app.window()
        .dispatch_event(slint::platform::WindowEvent::KeyPressed {
            text: "America/Toronto".into(),
        });
    app.window()
        .dispatch_event(slint::platform::WindowEvent::KeyReleased {
            text: "America/Toronto".into(),
        });
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_date_text(), "2026-11-10");
    assert_eq!(app.get_time_text(), "09:00");
    assert_eq!(
        app.get_zone_text(),
        "America/Toronto",
        "native typing reaches the remaining invalid zone"
    );
}

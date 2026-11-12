// SPDX-License-Identifier: GPL-3.0-or-later
use crate::QaEmailZonedFixture;
use i_slint_backend_testing::{ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{ComponentHandle, platform::WindowEvent};
use slint_kit::{
    FieldSpec,
    ui::{FieldAccess, FieldCommit, FieldValue},
};
use std::{cell::Cell, rc::Rc};

fn fixture() -> QaEmailZonedFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = QaEmailZonedFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    settle();
    app
}
fn settle() {
    slint::platform::update_timers_and_animations();
}
fn errors(app: &QaEmailZonedFixture) -> Vec<String> {
    ElementQuery::from_root(app)
        .match_inherits("KitFieldError")
        .find_all()
        .into_iter()
        .filter_map(|element| element.accessible_label())
        .map(|text| text.to_string())
        .filter(|text| !text.is_empty())
        .collect()
}
fn commit(app: &QaEmailZonedFixture, text: &str) -> FieldCommit {
    FieldCommit {
        value: FieldValue {
            present: true,
            text: text.into(),
        },
        constraints: app.get_email_constraints(),
        spec: Default::default(),
    }
}
#[test]
fn dotted_domain_policy_uses_the_same_pattern_for_ui_and_rust_submission() {
    let app = fixture();
    let accepted = Rc::new(Cell::new(0));
    let observed = accepted.clone();
    app.on_email_committed(move |value| {
        assert!(FieldSpec::email().accept(value).is_ok());
        observed.set(observed.get() + 1);
    });
    for text in ["john@acme", "john@@acme.tld", "john@acme."] {
        app.set_email_text(text.into());
        settle();
        assert!(!app.invoke_report_email(), "{text}");
        assert!(
            FieldSpec::email().accept(commit(&app, text)).is_err(),
            "{text}"
        );
        app.invoke_commit_email();
    }
    assert_eq!(accepted.get(), 0);
    app.set_email_text("john@acme".into());
    settle();
    assert!(app.get_email_validity().pattern_mismatch);
    assert!(
        !app.get_email_validity().type_mismatch,
        "single-label domains remain valid HTML email syntax"
    );
    for text in ["john@acme.tld", "john@sub.acme.tld"] {
        app.set_email_text(text.into());
        settle();
        assert!(app.invoke_report_email());
        let value = FieldSpec::email()
            .accept(commit(&app, text))
            .unwrap()
            .unwrap();
        assert_eq!(value.as_str(), text);
        app.invoke_commit_email();
    }
    assert_eq!(accepted.get(), 2);
    app.set_email_pattern("".into());
    app.set_email_text("john@acme".into());
    settle();
    assert!(
        app.invoke_report_email(),
        "generic HTML compatibility is preserved when the product pattern is absent"
    );
    assert!(FieldSpec::email().accept(commit(&app, "john@acme")).is_ok());
}
#[test]
fn incomplete_parts_show_only_their_own_error_and_focus_the_invalid_editor() {
    let app = fixture();
    app.set_date_text("2026-11-".into());
    settle();
    assert!(!app.invoke_report_zoned());
    settle();
    assert!(!app.get_zoned_validity().valid);
    let messages = errors(&app);
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(!messages[0].contains("fill out"), "{messages:?}");
    assert_eq!(app.get_zoned_message().as_str(), messages[0]);
    // The syntax error belongs below the date editor, before the valid time.
    app.window().take_snapshot().unwrap();
    let date = ElementQuery::from_root(&app)
        .match_inherits("TextInput")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Appointment — date"))
        .find_first()
        .unwrap();
    let time = ElementQuery::from_root(&app)
        .match_inherits("TextInput")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Appointment — time"))
        .find_first()
        .unwrap();
    let error = ElementQuery::from_root(&app)
        .match_inherits("KitFieldError")
        .find_first()
        .unwrap();
    assert!(error.absolute_position().y >= date.absolute_position().y + date.size().height);
    assert!(error.absolute_position().y < time.absolute_position().y);

    app.invoke_resolve();
    assert_eq!(app.get_chosen_count(), 0);
    assert!(app.invoke_focus_zoned_error());
    let old = app.get_date_text();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: "9".into() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: "9".into() });
    settle();
    assert_ne!(app.get_date_text(), old);
    assert_eq!(app.get_time_text(), "01:30");
    app.set_date_text("2026-11-02".into());
    app.set_time_text("".into());
    settle();
    assert!(!app.invoke_report_zoned());
    settle();
    assert_eq!(errors(&app).len(), 1);
    assert!(app.invoke_focus_zoned_error());
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: "3".into() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: "3".into() });
    settle();
    assert_eq!(app.get_date_text(), "2026-11-02");
    assert!(!app.get_time_text().is_empty());
}
#[test]
fn complete_pair_group_errors_and_zone_errors_keep_their_owners_and_reset() {
    let app = fixture();
    app.set_group_error("Appointment conflicts with another event.".into());
    settle();
    assert!(!app.invoke_report_zoned());
    settle();
    assert_eq!(errors(&app), ["Appointment conflicts with another event."]);
    app.invoke_reset_zoned();
    settle();
    assert!(errors(&app).is_empty());
    app.set_group_error("".into());
    app.set_zone_text("Not/AZone".into());
    settle();
    assert!(!app.invoke_report_zoned());
    settle();
    assert_eq!(errors(&app), ["Please enter a known IANA time zone."]);
    app.set_zone_text("America/Toronto".into());
    settle();
    app.invoke_resolve();
    settle();
    assert_eq!(app.get_chosen_count(), 1);
    app.set_access(FieldAccess::ReadOnly);
    app.set_date_text("2026-11-".into());
    settle();
    assert!(app.invoke_report_zoned());
    assert!(!app.invoke_focus_zoned_error());
    app.invoke_resolve();
    assert_eq!(app.get_chosen_count(), 1);
}
#[test]
fn configured_email_and_partial_date_errors_render_in_both_themes_and_text_scales() {
    use std::io::Write;
    let app = fixture();
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1, 2] {
            let mut theme = base.clone();
            theme.text_scale = if scale == 1 { 1. } else { 2. };
            app.set_theme(theme);
            app.set_email_text("john@acme".into());
            app.set_date_text("2026-11-".into());
            settle();
            assert!(!app.invoke_report_email());
            assert!(!app.invoke_report_zoned());
            settle();
            assert_eq!(errors(&app).len(), 2);
            let image = app.window().take_snapshot().unwrap();
            if let Some(directory) = std::env::var_os("KIT_QA_CAPTURE_DIR") {
                std::fs::create_dir_all(&directory).unwrap();
                let mut file = std::fs::File::create(
                    std::path::PathBuf::from(directory)
                        .join(format!("email-zoned-{name}-{scale}.ppm")),
                )
                .unwrap();
                write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
                for pixel in image.as_slice() {
                    file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
                }
            }
        }
    }
}

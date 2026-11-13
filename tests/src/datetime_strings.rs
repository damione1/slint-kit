// SPDX-License-Identifier: GPL-3.0-or-later
use crate::DateTimeStringsFixture;
use i_slint_backend_testing::{ElementHandle, TestingBackend, TestingBackendOptions};
use slint::ComponentHandle;

fn app(localized: bool) -> DateTimeStringsFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .ok();
    let app = DateTimeStringsFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_localized(localized);
    app.show().unwrap();
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
    app
}

fn has(app: &DateTimeStringsFixture, label: &str) -> bool {
    ElementHandle::find_by_accessible_label(app, label)
        .next()
        .is_some()
}

#[test]
fn time_picker_segment_names_and_validation_follow_the_consumer_language() {
    let english = app(false);
    for label in ["Hours", "Increase Hours", "Decrease Seconds"] {
        assert!(has(&english, label), "{label}");
    }
    let french = app(true);
    for label in [
        "Heures",
        "Augmenter : Heures",
        "Diminuer : Secondes",
        "Minutes",
    ] {
        assert!(has(&french, label), "{label}");
    }
    assert!(!has(&french, "Increase Hours"));
    assert!(!french.invoke_check_time());
    assert_eq!(french.get_time_message(), "Saisissez une heure.");
}

#[test]
fn date_and_date_time_validation_messages_are_overridable() {
    let french = app(true);
    assert!(!french.invoke_check_date());
    assert_eq!(french.get_date_message(), "Saisissez une date.");
    assert!(!french.invoke_check_pair());
    assert_eq!(french.get_pair_message(), "Renseignez la date et l'heure.");
    let english = app(false);
    assert!(!english.invoke_check_pair());
    assert_eq!(english.get_pair_message(), "Please fill out this field.");
}

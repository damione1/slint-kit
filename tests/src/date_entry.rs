// SPDX-License-Identifier: GPL-3.0-or-later
//! A date field can be typed in an explicit pattern while its accepted value,
//! bounds and constraints stay ISO. Software backend checks only: they do not
//! qualify a platform IME or a screen reader.
use crate::{CivilDateView, DateEntryFixture};
use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};

fn app(pattern: &str, min: &str, max: &str) -> DateEntryFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .ok();
    let app = DateEntryFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_pattern(pattern.into());
    app.set_min(min.into());
    app.set_max(max.into());
    app.show().unwrap();
    frame();
    app
}
fn frame() {
    slint::platform::update_timers_and_animations();
}
fn press(app: &DateEntryFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    frame();
}
fn replace_with(app: &DateEntryFixture, text: &str) {
    app.invoke_focus_field();
    frame();
    // Slint follows native Apple Home/End scrolling semantics; its end-of-line
    // shortcut is Control+RightArrow on Apple and End elsewhere.
    if cfg!(target_vendor = "apple") {
        app.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Control.into(),
        });
        press(app, Key::RightArrow);
        app.window().dispatch_event(WindowEvent::KeyReleased {
            text: Key::Control.into(),
        });
    } else {
        press(app, Key::End);
    }
    for _ in 0..app.get_typed().chars().count() {
        press(app, Key::Backspace);
    }
    for character in text.chars() {
        press(app, character.to_string());
    }
}
fn date(year: i32, month: i32, day: i32) -> CivilDateView {
    CivilDateView { year, month, day }
}

#[test]
fn the_edit_text_follows_the_entry_pattern_and_the_accepted_date_stays_typed() {
    for (pattern, expected) in [
        ("%d/%m/%Y", "16/10/2026"),
        ("%m/%d/%Y", "10/16/2026"),
        ("%-d %B %Y", "16 octobre 2026"),
    ] {
        let app = app(pattern, "", "");
        assert!(app.get_active(), "{pattern}");
        assert_eq!(app.get_typed(), expected, "{pattern}");
        assert_eq!(app.get_accepted(), date(2026, 10, 16), "{pattern}");
    }
}

#[test]
fn typing_a_pattern_date_and_leaving_the_field_accepts_it() {
    let app = app("%d/%m/%Y", "", "");
    replace_with(&app, "05/11/2026");
    press(&app, Key::Tab);
    assert_eq!(app.get_accepted(), date(2026, 11, 5));
    assert_eq!(app.get_typed(), "05/11/2026");
    assert!(app.get_validity().valid);
}

#[test]
fn typed_month_names_use_the_host_names() {
    let app = app("%-d %B %Y", "", "");
    replace_with(&app, "5 novembre 2026");
    press(&app, Key::Tab);
    assert_eq!(app.get_accepted(), date(2026, 11, 5));
    replace_with(&app, "5 november 2026");
    press(&app, Key::Tab);
    assert!(app.get_validity().bad_input);
    assert_eq!(app.get_accepted(), date(2026, 11, 5));
}

#[test]
fn an_iso_or_impossible_date_is_bad_input_and_never_accepted() {
    for text in ["2026-11-05", "31/02/2026", "05/11"] {
        let app = app("%d/%m/%Y", "", "");
        replace_with(&app, text);
        press(&app, Key::Tab);
        assert!(app.get_validity().bad_input, "{text}");
        assert_eq!(app.get_accepted(), date(2026, 10, 16), "{text}");
    }
}

#[test]
fn iso_bounds_constrain_the_pattern_draft_and_the_message_names_them_in_the_pattern() {
    let app = app("%d/%m/%Y", "2026-10-10", "2026-10-20");
    replace_with(&app, "09/10/2026");
    press(&app, Key::Tab);
    assert!(app.get_validity().range_underflow);
    assert!(
        app.get_message().contains("10/10/2026"),
        "{}",
        app.get_message()
    );
    replace_with(&app, "21/10/2026");
    press(&app, Key::Tab);
    assert!(app.get_validity().range_overflow);
    assert!(
        app.get_message().contains("20/10/2026"),
        "{}",
        app.get_message()
    );
    replace_with(&app, "15/10/2026");
    press(&app, Key::Tab);
    assert!(app.get_validity().valid);
    assert_eq!(app.get_accepted(), date(2026, 10, 15));
}

#[test]
fn arrow_keys_step_the_draft_in_the_pattern() {
    let app = app("%d/%m/%Y", "", "");
    app.invoke_focus_field();
    frame();
    press(&app, Key::UpArrow);
    assert_eq!(app.get_typed(), "17/10/2026");
    press(&app, Key::PageDown);
    assert_eq!(app.get_typed(), "17/09/2026");
}

#[test]
fn an_empty_or_invalid_entry_pattern_keeps_iso_entry() {
    for pattern in ["", "%B %Y", "%H:%M", "%Q"] {
        let app = app(pattern, "", "");
        assert!(!app.get_active(), "{pattern:?}");
        assert!(!app.get_picker_active(), "{pattern:?}");
        assert_eq!(app.get_typed(), "2026-10-16", "{pattern:?}");
        replace_with(&app, "2026-11-05");
        press(&app, Key::Tab);
        assert_eq!(app.get_accepted(), date(2026, 11, 5), "{pattern:?}");
    }
}

#[test]
fn the_picker_writes_and_reads_its_text_in_the_entry_pattern() {
    let app = app("%d/%m/%Y", "2026-10-10", "2026-10-20");
    assert!(app.get_picker_active());
    assert_eq!(app.get_picker_text(), "16/10/2026");
    app.invoke_choose_in_picker(date(2026, 10, 17));
    frame();
    assert_eq!(app.get_picked(), date(2026, 10, 17));
    assert_eq!(app.get_picker_text(), "17/10/2026");
    app.invoke_choose_in_picker(date(2026, 10, 25));
    frame();
    assert_eq!(app.get_picked(), date(2026, 10, 17));
}

#[test]
fn a_host_replacing_the_accepted_date_reprints_the_text_in_the_pattern() {
    let app = app("%d/%m/%Y", "", "");
    app.set_accepted(date(2027, 1, 2));
    frame();
    assert_eq!(app.get_typed(), "02/01/2027");
}

#[test]
fn the_pattern_codec_accepts_the_commit_a_pattern_field_produces() {
    use slint_kit::{CivilDate, DateNames, DatePattern, FieldSpec, ui};
    let spec = FieldSpec::date_pattern(
        DatePattern::new("%-d %B %Y").unwrap(),
        Some(DateNames::french()),
    );
    let entry = ui::DateDisplay {
        pattern: "%-d %B %Y".into(),
        months: Default::default(),
        short_months: Default::default(),
        weekdays: Default::default(),
        short_weekdays: Default::default(),
    };
    // Four empty name lists mean English names; a field that sets host names
    // sets the same names on its spec.
    let commit = |text: &str, entry: ui::DateDisplay| ui::FieldCommit {
        value: ui::FieldValue {
            present: !text.is_empty(),
            text: text.into(),
        },
        constraints: ui::KitInputConstraints {
            kind: ui::HtmlInputKind::Date,
            date_entry: entry,
            ..Default::default()
        },
        spec: Default::default(),
    };
    let model = |names: &[&str]| -> slint::ModelRc<slint::SharedString> {
        slint::ModelRc::new(slint::VecModel::from(
            names
                .iter()
                .map(|name| slint::SharedString::from(*name))
                .collect::<Vec<_>>(),
        ))
    };
    let french = ui::DateDisplay {
        months: model(&[
            "janvier",
            "février",
            "mars",
            "avril",
            "mai",
            "juin",
            "juillet",
            "août",
            "septembre",
            "octobre",
            "novembre",
            "décembre",
        ]),
        short_months: model(&[
            "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.",
            "nov.", "déc.",
        ]),
        weekdays: model(&[
            "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche",
        ]),
        short_weekdays: model(&["lun.", "mar.", "mer.", "jeu.", "ven.", "sam.", "dim."]),
        ..entry.clone()
    };
    assert_eq!(
        spec.accept(commit("5 novembre 2026", french.clone()))
            .unwrap(),
        Some(CivilDate::new(2026, 11, 5).unwrap())
    );
    assert!(spec.accept(commit("2026-11-05", french)).is_err());
    assert_eq!(spec.accept(commit("", entry)).unwrap(), None);
    assert_eq!(
        spec.format(&CivilDate::new(2026, 11, 5).unwrap()),
        "5 novembre 2026"
    );
}

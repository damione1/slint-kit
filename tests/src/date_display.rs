// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{DateDisplayFixture, TemporalDisplayFixture};
use i_slint_backend_testing::{ElementHandle, TestingBackend, TestingBackendOptions};
use slint::ComponentHandle;

fn app(pattern: &str) -> DateDisplayFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .ok();
    let app = DateDisplayFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_pattern(pattern.into());
    app.show().unwrap();
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
    app
}

#[test]
fn display_pattern_formats_the_resting_text_and_keeps_the_edit_text_iso() {
    for (pattern, expected) in [
        ("%d/%m/%Y", "16/10/2026"),
        ("%m/%d/%Y", "10/16/2026"),
        ("%-d %b %Y", "16 oct. 2026"),
        ("%A %-d %B %Y", "vendredi 16 octobre 2026"),
    ] {
        let app = app(pattern);
        assert_eq!(app.get_shown(), expected, "{pattern}");
        assert_eq!(app.get_edited(), "2026-10-16", "{pattern}");
    }
}

#[test]
fn an_empty_or_invalid_display_pattern_keeps_the_iso_text() {
    for pattern in ["", "%H:%M", "%Q"] {
        let app = app(pattern);
        assert_eq!(app.get_shown(), "2026-10-16", "{pattern:?}");
    }
}

#[test]
fn the_picker_forwards_its_display_pattern_to_the_editor() {
    let app = app("");
    let shown = ElementHandle::find_by_accessible_label(&app, "Picker")
        .filter_map(|element| element.accessible_value())
        .collect::<Vec<_>>();
    assert!(shown.iter().any(|value| value == "16/10/2026"), "{shown:?}");
}

fn temporal(date: &str, month: &str, week: &str) -> TemporalDisplayFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .ok();
    let app = TemporalDisplayFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_date_pattern(date.into());
    app.set_month_pattern(month.into());
    app.set_week_pattern(week.into());
    app.show().unwrap();
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
    app
}
fn rested(app: &TemporalDisplayFixture, label: &str) -> Vec<String> {
    ElementHandle::find_by_accessible_label(app, label)
        .filter_map(|element| element.accessible_value())
        .map(|value| value.to_string())
        .collect()
}

#[test]
fn a_month_field_formats_its_resting_text_and_keeps_the_edit_text_iso() {
    for (pattern, expected) in [
        ("%B %Y", "octobre 2026"),
        ("%m/%Y", "10/2026"),
        ("%b %Y", "oct. 2026"),
    ] {
        let app = temporal("", pattern, "");
        assert_eq!(app.get_month_shown(), expected, "{pattern}");
        assert_eq!(app.get_month_edited(), "2026-10", "{pattern}");
    }
}

#[test]
fn an_iso_week_field_formats_its_resting_text_and_keeps_the_edit_text_iso() {
    for (pattern, expected) in [
        ("%G, semaine %V", "2026, semaine 42"),
        ("%V/%G", "42/2026"),
        ("%G-W%V", "2026-W42"),
    ] {
        let app = temporal("", "", pattern);
        assert_eq!(app.get_week_shown(), expected, "{pattern}");
        assert_eq!(app.get_week_edited(), "2026-W42", "{pattern}");
    }
}

#[test]
fn an_empty_or_invalid_month_or_week_pattern_keeps_the_iso_text() {
    for pattern in ["", "%H:%M", "%Q"] {
        let app = temporal("", pattern, pattern);
        assert_eq!(app.get_month_shown(), "2026-10", "{pattern:?}");
        assert_eq!(app.get_week_shown(), "2026-W42", "{pattern:?}");
    }
}

#[test]
fn a_focused_month_field_shows_the_iso_draft_and_formats_again_on_blur() {
    let app = temporal("", "%B %Y", "");
    assert_eq!(rested(&app, "Month"), ["octobre 2026"]);
    app.invoke_focus_month();
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    assert_eq!(rested(&app, "Month"), ["2026-10"]);
    app.invoke_blur_month();
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    assert_eq!(rested(&app, "Month"), ["octobre 2026"]);
}

#[test]
fn date_time_and_zoned_fields_format_the_resting_date_and_leave_the_time_alone() {
    let app = temporal("%A %-d %B %Y", "", "");
    assert_eq!(rested(&app, "Stamp — date"), ["vendredi 16 octobre 2026"]);
    assert_eq!(rested(&app, "Stamp — time"), ["09:30"]);
    assert_eq!(rested(&app, "Zoned — date"), ["vendredi 16 octobre 2026"]);
}

#[test]
fn an_empty_or_invalid_date_time_pattern_keeps_the_iso_date() {
    for pattern in ["", "%H:%M", "%Q"] {
        let app = temporal(pattern, "", "");
        assert_eq!(rested(&app, "Stamp — date"), ["2026-10-16"], "{pattern:?}");
        assert_eq!(rested(&app, "Zoned — date"), ["2026-10-16"], "{pattern:?}");
    }
}

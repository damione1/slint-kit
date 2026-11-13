// SPDX-License-Identifier: GPL-3.0-or-later
use crate::DateDisplayFixture;
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

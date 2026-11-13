// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{DatePickerTrigger, PickerFieldContractFixture, ValidationTiming};
use i_slint_backend_testing::{AccessibleRole, ElementHandle, ElementQuery};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use std::{fs, path::Path};

fn fixture() -> PickerFieldContractFixture {
    i_slint_backend_testing::init_no_event_loop();
    let app = PickerFieldContractFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app
}
fn key(app: &PickerFieldContractFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    slint::platform::update_timers_and_animations();
}
fn calendar_buttons(app: &PickerFieldContractFixture) -> usize {
    ElementQuery::from_root(app)
        .match_predicate(|e: &ElementHandle| {
            e.accessible_role() == Some(AccessibleRole::Button)
                && e.accessible_label()
                    .is_some_and(|label| label.starts_with("Open calendar"))
        })
        .find_all()
        .len()
}

#[test]
fn date_picker_reset_restores_the_default_and_the_accepted_date() {
    let app = fixture();
    assert_eq!(app.get_picked().day, 19);
    app.invoke_reset_date();
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_date_text(), "2026-01-02");
    assert_eq!(app.get_date_accepted_text(), "2026-01-02");
    assert_eq!(
        (
            app.get_picked().year,
            app.get_picked().month,
            app.get_picked().day
        ),
        (2026, 1, 2)
    );
}

#[test]
fn date_picker_cancel_edit_discards_the_draft() {
    let app = fixture();
    app.invoke_focus_date();
    app.invoke_select_date();
    key(&app, "2");
    assert_ne!(app.get_date_text(), "2026-10-19");
    app.invoke_cancel_date();
    assert_eq!(app.get_date_text(), "2026-10-19");
}

#[test]
fn date_picker_reports_focus_and_forwards_commits() {
    let app = fixture();
    app.invoke_focus_date();
    slint::platform::update_timers_and_animations();
    assert!(app.get_date_focused());
    assert_eq!(app.get_date_focus_events(), 1);
    app.invoke_select_date();
    for character in "2026-10-05".chars() {
        key(&app, character.to_string());
    }
    key(&app, Key::Return);
    assert_eq!(app.get_date_commit_count(), 1);
    assert_eq!(app.get_picked().day, 5);
    assert_eq!(app.get_date_state().accepted.text, "2026-10-05");
    key(&app, Key::Tab);
    assert!(!app.get_date_focused());
    assert_eq!(app.get_date_focus_events(), 2);
}

#[test]
fn time_picker_reports_focus_and_resets_to_its_default() {
    let app = fixture();
    app.invoke_focus_time();
    slint::platform::update_timers_and_animations();
    assert!(app.get_time_focused());
    assert_eq!(app.get_time_focus_events(), 1);
    app.invoke_reset_time();
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_time_text(), "08:15");
    let time = app.get_time_value();
    assert!(time.present);
    assert_eq!((time.time.hour, time.time.minute), (8, 15));
}

#[test]
fn date_time_field_reset_restores_both_drafts_and_the_accepted_value() {
    let app = fixture();
    app.invoke_reset_local();
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_local_date(), "2026-03-04");
    assert_eq!(app.get_local_time(), "05:06");
    let value = app.get_local_value();
    assert_eq!(
        (
            value.date.year,
            value.date.month,
            value.date.day,
            value.time.hour,
            value.time.minute
        ),
        (2026, 3, 4, 5, 6)
    );
}

#[test]
fn zoned_field_reset_empties_every_part() {
    let app = fixture();
    app.invoke_reset_zoned();
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_zoned_date(), "");
    assert_eq!(app.get_zoned_time(), "");
    assert_eq!(app.get_zoned_zone(), "");
}

#[test]
fn the_calendar_trigger_presentation_follows_the_trigger_style() {
    let app = fixture();
    assert_eq!(calendar_buttons(&app), 1, "text button by default");
    app.set_trigger_style(DatePickerTrigger::IconButton);
    slint::platform::update_timers_and_animations();
    assert_eq!(calendar_buttons(&app), 1, "the icon button keeps its name");
    app.set_trigger_style(DatePickerTrigger::None);
    slint::platform::update_timers_and_animations();
    assert_eq!(calendar_buttons(&app), 0, "no trigger is exposed");
    app.invoke_focus_date();
    key(&app, Key::F4);
    key(&app, Key::Escape);
}

#[test]
fn validation_timing_is_accepted_by_every_picker() {
    let app = fixture();
    app.set_timing(ValidationTiming::OnInput);
    slint::platform::update_timers_and_animations();
    app.set_timing(ValidationTiming::OnSubmit);
    slint::platform::update_timers_and_animations();
}

// Contract guard: the field port KitTextField defines must stay forwarded by
// each composite. A name missing here means a picker silently lost a member.
fn component_body(source: &str, name: &str) -> String {
    let marker = format!("export component {name} ");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("{name} is declared"));
    let rest = &source[start + marker.len()..];
    let end = rest.find("export component ").unwrap_or(rest.len());
    rest[..end].to_owned()
}

#[test]
fn composite_pickers_declare_the_shared_field_port() {
    let ui = Path::new(env!("CARGO_MANIFEST_DIR")).join("../crates/kit-slint/ui");
    let calendar = fs::read_to_string(ui.join("calendar.slint")).unwrap();
    let datetime = fs::read_to_string(ui.join("datetime.slint")).unwrap();
    let text_field = component_body(
        &fs::read_to_string(ui.join("input.slint")).unwrap(),
        "KitTextField",
    );
    let shared_port = [
        "public function reset-field(",
        "property <ValidationTiming> validation-timing",
    ];
    for member in shared_port {
        assert!(
            text_field.contains(member),
            "KitTextField no longer declares {member}"
        );
    }
    let picker_port = [
        "public function commit(",
        "public function cancel-edit(",
        "public function reset-field(",
        "public function select-all(",
        "public function acknowledge(",
        "property <ValidationTiming> validation-timing",
        "property <string> spec-key",
        "property <CommitPolicy> commit-policy",
        "property <FieldState> field-state",
        "callback committed(",
        "callback focus-observed(",
        "property <bool> has-focus",
    ];
    for member in picker_port {
        assert!(
            component_body(&calendar, "KitDatePicker").contains(member),
            "KitDatePicker does not forward {member}"
        );
    }
    let time_port = [
        "public function commit(",
        "public function cancel-edit(",
        "public function reset-field(",
        "public function select-all(",
        "public function acknowledge(",
        "property <ValidationTiming> validation-timing",
        "property <string> spec-key",
        "property <CommitPolicy> commit-policy",
        "property <FieldState> field-state",
        "callback committed(",
        "callback focus-observed(",
        "property <bool> has-focus",
        "property <string> description",
        "property <string> placeholder",
    ];
    for member in time_port {
        assert!(
            component_body(&datetime, "KitTimePicker").contains(member),
            "KitTimePicker does not forward {member}"
        );
    }
    for name in ["KitDateTimeField", "KitZonedDateTimeField"] {
        let body = component_body(&datetime, name);
        for member in [
            "public function reset-field(",
            "property <ValidationTiming> validation-timing",
            "property <CommitPolicy> commit-policy",
            "callback focus-observed(",
        ] {
            assert!(body.contains(member), "{name} does not forward {member}");
        }
    }
}

// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ui;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use std::{cell::Cell, rc::Rc};
fn fixture() -> ui::HtmlFixture {
    i_slint_backend_testing::init_no_event_loop();
    let view = ui::HtmlFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, view);
    view.show().unwrap();
    slint::platform::update_timers_and_animations();
    view
}
fn element(view: &ui::HtmlFixture, label: &str) -> ElementHandle {
    let label = label.to_string();
    ElementQuery::from_root(view)
        .match_inherits("TextInput")
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label.as_str()))
        .find_first()
        .unwrap()
}
fn key(view: &ui::HtmlFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    view.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    view.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}
#[test]
fn email_blur_reports_intrinsic_error_and_enter_cannot_commit_it() {
    let view = fixture();
    let commits = Rc::new(Cell::new(0));
    let counter = commits.clone();
    view.on_email_commit(move |_| counter.set(counter.get() + 1));
    let original = element(&view, "Email");
    view.invoke_focus_email();
    slint::platform::update_timers_and_animations();
    key(&view, "alex");
    slint::platform::update_timers_and_animations();
    assert!(view.get_email_validity().type_mismatch);
    assert!(!view.get_email_touched());
    view.invoke_focus_url();
    slint::platform::update_timers_and_animations();
    assert!(view.get_email_touched());
    assert_eq!(
        element(&view, "Email").accessible_description().as_deref(),
        Some("Please enter a valid email address.")
    );
    view.invoke_focus_email();
    key(&view, Key::Return);
    assert_eq!(commits.get(), 0);
    key(&view, "@@example.com");
    assert!(view.get_email_validity().type_mismatch);
    view.set_email_text("alex@example.com".into());
    assert!(view.get_email_validity().valid);
    key(&view, Key::Return);
    assert_eq!(commits.get(), 1);
    assert_eq!(
        original.accessible_value().as_deref(),
        Some("alex@example.com"),
        "original native editor handle remains alive"
    );
}
#[test]
fn url_rejects_the_reported_trailing_at_and_relative_addresses() {
    let view = fixture();
    for text in ["https://example.comddf.dofmo@", "example.com", "https://"] {
        view.set_url_text(text.into());
        assert!(view.get_url_validity().type_mismatch, "{text}");
    }
    for text in [
        "https://example.com",
        "http://localhost",
        "mailto:a@example.com",
    ] {
        view.set_url_text(text.into());
        assert!(view.get_url_validity().valid, "{text}");
    }
}
#[test]
fn multiple_custom_error_required_readonly_disabled_and_reset_are_live() {
    let view = fixture();
    view.set_required(true);
    assert!(view.get_email_validity().value_missing);
    assert!(!view.invoke_report_email());
    view.set_email_text("a@example.com,b@example.net".into());
    assert!(view.get_email_validity().type_mismatch);
    view.set_multiple(true);
    assert!(view.get_email_validity().valid);
    view.invoke_custom_email("Address already used".into());
    assert!(view.get_email_validity().custom_error);
    assert_eq!(view.get_email_error(), "Address already used");
    view.set_access(ui::FieldAccess::ReadOnly);
    assert!(!view.get_email_will_validate());
    assert!(view.invoke_report_email());
    view.set_access(ui::FieldAccess::Disabled);
    assert!(!view.get_email_will_validate());
    assert!(view.invoke_report_email());
    view.set_access(ui::FieldAccess::Editable);
    view.invoke_reset_email();
    assert_eq!(view.get_email_text(), "");
    assert!(view.get_email_validity().custom_error);
    assert!(!view.get_email_touched());
    view.invoke_custom_email("".into());
    assert!(!view.get_email_validity().custom_error);
}
#[test]
fn pattern_is_extensible_and_invalid_configuration_is_diagnosable() {
    let view = fixture();
    view.set_code_text("AB".into());
    assert!(view.get_code_validity().pattern_mismatch);
    view.set_code_text("ABC".into());
    assert!(view.get_code_validity().valid);
    view.set_code_text("XABCX".into());
    assert!(view.get_code_validity().pattern_mismatch);
    view.set_pattern(r"[[a-z]&&[^aeiou]]+".into());
    view.set_code_text("bcdf".into());
    assert!(view.get_code_validity().valid);
    view.set_code_text("abc".into());
    assert!(view.get_code_validity().pattern_mismatch);
    view.set_pattern("[".into());
    assert!(
        view.get_code_validity().valid,
        "invalid HTML patterns impose no constraint"
    );
    assert!(!view.get_pattern_error().is_empty());
}

#[test]
fn selected_values_and_in_place_radio_model_changes_update_required_constraints() {
    use slint::{Model, ModelRc, VecModel};
    i_slint_backend_testing::init_no_event_loop();
    let view = ui::HtmlChoiceFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, view);
    view.show().unwrap();
    assert!(view.get_checkbox_validity().value_missing);
    view.set_checked(ui::CheckState::Checked);
    assert!(view.get_checkbox_validity().valid);
    assert!(view.get_radio_validity().value_missing);
    let rows = Rc::new(VecModel::from(vec![ui::ChoiceRow {
        token: 7,
        label: "Plan".into(),
        disabled: false,
        selected: false,
    }]));
    view.set_rows(ModelRc::from(rows.clone()));
    assert!(view.get_radio_validity().value_missing);
    let mut row = rows.row_data(0).unwrap();
    row.selected = true;
    rows.set_row_data(0, row);
    assert!(
        view.get_radio_validity().valid,
        "in-place row changes must revalidate, not only replacing the model"
    );
    rows.remove(0);
    assert!(view.get_radio_validity().value_missing);
    assert!(view.get_select_validity().value_missing);
    view.set_disabled(true);
    assert!(view.get_select_validity().valid);
    assert!(view.get_multi_validity().value_missing);
    view.set_chips(ModelRc::new(VecModel::from(vec![ui::ChipRow {
        token: 2,
        label: "Email".into(),
        removable: true,
    }])));
    assert!(view.get_multi_validity().valid);
    assert!(view.get_files_validity().value_missing);
}
#[test]
fn submit_validates_but_explicit_novalidate_and_reset_preserve_web_semantics() {
    use slint::platform::PointerEventButton;
    i_slint_backend_testing::init_no_event_loop();
    let view = ui::HtmlChoiceFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, view);
    view.show().unwrap();
    let submitted = Rc::new(Cell::new(0));
    let count = submitted.clone();
    view.on_submitted(move || count.set(count.get() + 1));
    let reset = Rc::new(Cell::new(0));
    let count = reset.clone();
    view.on_reset(move || count.set(count.get() + 1));
    let button = |label: &str| {
        let label = label.to_owned();
        ElementQuery::from_root(&view)
            .match_inherits("KitButton")
            .match_predicate(move |e| e.accessible_label().as_deref() == Some(label.as_str()))
            .find_first()
            .unwrap()
    };
    button("Submit").mock_single_click(PointerEventButton::Left);
    assert_eq!(submitted.get(), 0);
    view.set_allow_submit(true);
    button("Submit").mock_single_click(PointerEventButton::Left);
    assert_eq!(submitted.get(), 1);
    view.set_allow_submit(false);
    view.set_no_validate(true);
    button("Submit").mock_single_click(PointerEventButton::Left);
    assert_eq!(submitted.get(), 2);
    button("Reset").mock_single_click(PointerEventButton::Left);
    assert_eq!(reset.get(), 1);
}
#[test]
fn numeric_editor_allows_incomplete_exponents_and_localized_decimals_without_accepting_letters() {
    let view = fixture();
    view.invoke_focus_count();
    key(&view, "1");
    key(&view, "e");
    assert_eq!(view.get_count_text(), "1e");
    assert_eq!(view.get_count_value(), "");
    assert!(view.get_count_validity().bad_input);
    key(&view, "1");
    assert_eq!(view.get_count_value(), "1e1");
    assert!(view.get_count_validity().valid);
    key(&view, "Ada");
    assert_eq!(view.get_count_text(), "1e1");
    view.set_number_format(ui::NumberInputFormat::DecimalComma);
    view.set_count_step("any".into());
    view.invoke_select_count();
    key(&view, "4,5");
    assert_eq!(view.get_count_text(), "4,5");
    assert_eq!(view.get_count_value(), "4.5");
    assert!(view.get_count_validity().valid);
}
#[test]
fn a_second_at_or_unsupported_email_character_is_visible_before_blur() {
    let view = fixture();
    view.invoke_focus_email();
    key(&view, "a@@b");
    assert!(view.get_email_validity().type_mismatch);
    assert!(!view.get_email_touched());
    assert_eq!(
        element(&view, "Email").accessible_description().as_deref(),
        Some("Please enter a valid email address.")
    );
}

#[test]
fn date_picker_temporal_groups_and_hex_fields_expose_real_constraints() {
    i_slint_backend_testing::init_no_event_loop();
    let view = ui::HtmlTemporalFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, view);
    view.show().unwrap();
    view.set_date_text("2026-10-27".into());
    assert!(view.get_date_validity().range_underflow);
    view.set_date_text("2026-10-28".into());
    assert!(view.get_date_validity().valid);
    view.set_time_text("01:30 AM".into());
    assert!(view.get_time_validity().range_underflow);
    view.set_time_text("01:30 PM".into());
    assert!(view.get_time_validity().valid);
    assert!(view.get_local_validity().valid);
    view.set_local_time("12:30 PM".into());
    assert!(view.get_local_validity().range_underflow);
    view.set_local_time("01:30 PM".into());
    view.set_zone("No/Such_Zone".into());
    assert!(view.get_zoned_validity().custom_error);
    view.set_zone("America/Toronto".into());
    assert!(view.get_zoned_validity().valid);
    view.set_color_text("#zzzzzz".into());
    assert!(view.get_color_validity().pattern_mismatch);
    view.set_color_text("#123456".into());
    assert!(view.get_color_validity().valid);
}
#[test]
fn calendar_bounds_and_steps_guard_keyboard_selection_without_rewriting_the_model() {
    use slint::{ModelRc, VecModel};
    i_slint_backend_testing::init_no_event_loop();
    let view = ui::HtmlTemporalFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, view);
    view.set_days(ModelRc::new(VecModel::from(
        (27..=29)
            .map(|day| ui::CalendarDay {
                year: 2026,
                month: 10,
                day,
                label: day.to_string().into(),
                accessible_label: format!("Day {day}").into(),
                enabled: true,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
    )));
    let count = Rc::new(Cell::new(0));
    let chosen = count.clone();
    view.on_chosen(move || chosen.set(chosen.get() + 1));
    view.show().unwrap();
    view.invoke_focus_calendar();
    let key = |text: slint::SharedString| {
        view.window()
            .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
        view.window()
            .dispatch_event(WindowEvent::KeyReleased { text });
    };
    key(Key::Return.into());
    assert_eq!(count.get(), 0, "before min");
    view.set_active_index(1);
    key(Key::Return.into());
    assert_eq!(count.get(), 1, "minimum is on step grid");
    view.set_active_index(2);
    key(Key::Return.into());
    assert_eq!(count.get(), 1, "next day is off step grid");
}
#[test]
fn a_missing_validation_engine_cannot_report_a_successful_validation() {
    i_slint_backend_testing::init_no_event_loop();
    let view = ui::HtmlFixture::new().unwrap();
    view.set_email_text("a@example.com".into());
    assert!(!view.get_email_validity().valid);
    assert!(!view.invoke_report_email());
}

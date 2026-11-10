// SPDX-License-Identifier: GPL-3.0-or-later
use super::{FieldAccess, NumberInputFormat, NumberSteppingFixture, StepDirection};
use i_slint_backend_testing::{AccessibleRole, ElementHandle, ElementQuery};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};

fn fixture() -> NumberSteppingFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = NumberSteppingFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    window
}
fn key(window: &NumberSteppingFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
fn undo(window: &NumberSteppingFixture) {
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: "z".into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: "z".into() });
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
}
fn amount(window: &NumberSteppingFixture, value: &str) {
    window.set_value(value.into());
    window.set_edited_count(0);
    window.set_value_edited_count(0);
    window.set_commit_count(0);
    window.invoke_focus_amount();
}
fn spin_button(window: &NumberSteppingFixture, label: &str) -> ElementHandle {
    let requested = label.to_owned();
    ElementQuery::from_root(window)
        .match_accessible_role(AccessibleRole::Spinbox)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(requested.as_str())
        })
        .find_first()
        .unwrap_or_else(|| panic!("missing spin button {label}"))
}
fn editor(window: &NumberSteppingFixture, label: &str) -> ElementHandle {
    let requested = label.to_owned();
    ElementQuery::from_root(window)
        .match_accessible_role(AccessibleRole::TextInput)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(requested.as_str())
        })
        .find_first()
        .unwrap_or_else(|| panic!("missing editor {label}"))
}

#[test]
fn arrow_keys_step_once_through_the_native_edit_path() {
    let window = fixture();
    amount(&window, "5");
    key(&window, Key::UpArrow);
    assert_eq!(window.get_value(), "6");
    assert_eq!(window.get_edited_count(), 1, "one native edit per step");
    assert_eq!(window.get_value_edited_count(), 1);
    assert_eq!(window.get_last_value_edited(), "6");
    assert!(
        window.get_dirty(),
        "a step marks the field dirty like typing"
    );
    assert_eq!(
        window.get_commit_count(),
        0,
        "steps follow the commit policy"
    );
    key(&window, Key::DownArrow);
    key(&window, Key::DownArrow);
    assert_eq!(window.get_value(), "4");
    assert_eq!(window.get_edited_count(), 3);
    // Like typing over a selection, a step is a native delete plus an insert.
    undo(&window);
    undo(&window);
    assert_eq!(
        window.get_value(),
        "5",
        "steps stay in the native undo history"
    );
    undo(&window);
    undo(&window);
    assert_eq!(window.get_value(), "6");
    window.invoke_blur_amount();
    slint::platform::update_timers_and_animations();
    assert_eq!(
        window.get_commit_count(),
        1,
        "blur commits the stepped draft"
    );
}

#[test]
fn page_keys_step_by_the_configured_page_size() {
    let window = fixture();
    window.set_step("0.1".into());
    amount(&window, "1");
    key(&window, Key::PageUp);
    assert_eq!(window.get_value(), "2", "default page is ten steps");
    window.set_page_steps(5);
    key(&window, Key::PageDown);
    assert_eq!(window.get_value(), "1.5");
    assert_eq!(window.get_edited_count(), 2);
    window.set_page_steps(0);
    key(&window, Key::PageUp);
    assert_eq!(
        window.get_value(),
        "1.5",
        "a page of zero steps changes nothing"
    );
    assert_eq!(window.get_edited_count(), 2);
}

#[test]
fn home_and_end_jump_to_bounds_or_keep_the_native_caret() {
    let window = fixture();
    amount(&window, "12");
    // Without a bound the key stays with the native editor; the testing
    // backend has no line layout, so caret movement itself is not asserted.
    key(&window, Key::End);
    key(&window, Key::Home);
    assert_eq!(
        window.get_value(),
        "12",
        "Home/End without bounds never step"
    );
    assert_eq!(window.get_edited_count(), 0);
    key(&window, Key::RightArrow);
    assert_eq!(
        window.get_cursor_position(),
        1,
        "the native caret still moves"
    );
    window.set_min("2".into());
    window.set_max("10".into());
    key(&window, Key::End);
    assert_eq!(window.get_value(), "10");
    key(&window, Key::Home);
    assert_eq!(window.get_value(), "2");
    assert_eq!(window.get_edited_count(), 2);
    key(&window, Key::Home);
    assert_eq!(window.get_edited_count(), 2, "Home at min edits nothing");
}

#[test]
fn bounds_clamp_without_duplicate_edits() {
    let window = fixture();
    window.set_min("0".into());
    window.set_max("10".into());
    amount(&window, "9");
    key(&window, Key::UpArrow);
    assert_eq!(window.get_value(), "10");
    assert_eq!(window.get_edited_count(), 1);
    key(&window, Key::UpArrow);
    key(&window, Key::PageUp);
    assert_eq!(window.get_value(), "10");
    assert_eq!(window.get_edited_count(), 1, "clamped steps emit no edit");
    assert_eq!(window.get_value_edited_count(), 1);
    amount(&window, "0");
    key(&window, Key::DownArrow);
    assert_eq!(window.get_value(), "0");
    assert_eq!(window.get_edited_count(), 0);
}

#[test]
fn any_step_read_only_and_disabled_fields_do_not_step() {
    let window = fixture();
    window.set_step("any".into());
    amount(&window, "5");
    key(&window, Key::UpArrow);
    key(&window, Key::PageUp);
    assert_eq!(window.get_value(), "5", "step any has no lattice");
    assert_eq!(window.get_edited_count(), 0);
    assert!(!window.invoke_step_amount(StepDirection::Increment, 1));
    window.set_step("1".into());
    window.set_access(FieldAccess::ReadOnly);
    amount(&window, "5");
    key(&window, Key::UpArrow);
    assert_eq!(window.get_value(), "5", "read-only keeps the value");
    assert!(!window.invoke_step_amount(StepDirection::Increment, 1));
    window.set_access(FieldAccess::Disabled);
    assert!(!window.invoke_step_amount(StepDirection::Decrement, 1));
    assert_eq!(window.get_value(), "5");
    assert_eq!(window.get_edited_count(), 0);
}

#[test]
fn decimal_steps_stay_exact_in_both_grammars() {
    let window = fixture();
    window.set_step("0.1".into());
    amount(&window, "0.2");
    key(&window, Key::UpArrow);
    assert_eq!(window.get_value(), "0.3");
    amount(&window, "0.7");
    key(&window, Key::UpArrow);
    assert_eq!(window.get_value(), "0.8");
    key(&window, Key::DownArrow);
    key(&window, Key::DownArrow);
    assert_eq!(window.get_value(), "0.6");
    window.set_number_format(NumberInputFormat::DecimalComma);
    amount(&window, "1,5");
    key(&window, Key::UpArrow);
    assert_eq!(
        window.get_value(),
        "1,6",
        "decimal comma drafts keep their grammar"
    );
}

#[test]
fn the_keyboard_hook_still_intercepts_step_keys_first() {
    let window = fixture();
    amount(&window, "5");
    window.set_intercept_key(true);
    key(&window, Key::UpArrow);
    assert_eq!(
        window.get_value(),
        "5",
        "an accepting host hook runs before stepping"
    );
    assert_eq!(window.get_edited_count(), 0);
    window.set_intercept_key(false);
    key(&window, Key::UpArrow);
    assert_eq!(window.get_value(), "6");
}

#[test]
fn modifier_arrows_and_unfocused_fields_leave_the_value_alone() {
    let window = fixture();
    amount(&window, "5");
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    key(&window, Key::UpArrow);
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    assert_eq!(window.get_value(), "5", "modified arrows are not steps");
    window.invoke_focus_plain();
    key(&window, Key::UpArrow);
    assert_eq!(window.get_value(), "5");
    assert_eq!(window.get_edited_count(), 0);
}

#[test]
fn number_stepper_inherits_keyboard_stepping_and_its_page_size() {
    let window = fixture();
    window.set_stepper_value("4".into());
    window.invoke_focus_stepper();
    key(&window, Key::UpArrow);
    assert_eq!(window.get_stepper_value(), "5");
    key(&window, Key::PageUp);
    assert_eq!(
        window.get_stepper_value(),
        "8",
        "three steps clamp at max 8"
    );
    key(&window, Key::PageDown);
    assert_eq!(window.get_stepper_value(), "5");
    key(&window, Key::Home);
    assert_eq!(window.get_stepper_value(), "1");
    assert_eq!(window.get_stepper_edited_count(), 4);
}

#[test]
fn wheel_stepping_is_opt_in_and_needs_focus() {
    let window = fixture();
    amount(&window, "5");
    let editor = editor(&window, "Amount");
    editor.scroll(0.0, 40.0);
    assert_eq!(window.get_value(), "5", "wheel stepping is off by default");
    window.set_wheel_stepping(true);
    editor.scroll(0.0, 40.0);
    assert_eq!(
        window.get_value(),
        "6",
        "wheel up increments the focused field"
    );
    editor.scroll(0.0, -40.0);
    editor.scroll(0.0, -40.0);
    assert_eq!(window.get_value(), "4");
    assert_eq!(window.get_edited_count(), 3);
    window.invoke_focus_plain();
    editor.scroll(0.0, 40.0);
    assert_eq!(
        window.get_value(),
        "4",
        "an unfocused field leaves the wheel alone"
    );
}

#[test]
fn numeric_fields_expose_a_spin_button_with_range_and_step_actions() {
    let window = fixture();
    window.set_value("5".into());
    let spin = spin_button(&window, "Amount");
    assert_eq!(spin.accessible_value().as_deref(), Some("5"));
    assert_eq!(spin.accessible_value_step(), Some(1.0));
    assert_eq!(
        spin.accessible_value_minimum(),
        Some(f32::NEG_INFINITY),
        "no min publishes an unbounded minimum"
    );
    assert_eq!(spin.accessible_value_maximum(), Some(f32::INFINITY));
    assert_eq!(spin.accessible_read_only(), Some(false));
    window.set_min("2".into());
    window.set_max("10".into());
    window.set_step("0.5".into());
    assert_eq!(spin.accessible_value_minimum(), Some(2.0));
    assert_eq!(spin.accessible_value_maximum(), Some(10.0));
    assert_eq!(spin.accessible_value_step(), Some(0.5));
    spin.invoke_accessible_increment_action();
    assert_eq!(
        window.get_value(),
        "5.5",
        "the increment action shares the step"
    );
    assert_eq!(window.get_edited_count(), 1);
    spin.invoke_accessible_decrement_action();
    spin.invoke_accessible_decrement_action();
    assert_eq!(window.get_value(), "4.5");
    assert_eq!(window.get_edited_count(), 3);
    window.set_access(FieldAccess::ReadOnly);
    assert_eq!(spin.accessible_read_only(), Some(true));
    spin.invoke_accessible_increment_action();
    assert_eq!(window.get_value(), "4.5", "read-only ignores the action");
    assert_eq!(
        spin_button(&window, "Guests").accessible_value_maximum(),
        Some(8.0)
    );
    assert!(
        ElementQuery::from_root(&window)
            .match_accessible_role(AccessibleRole::Spinbox)
            .match_predicate(|element| element.accessible_label().as_deref() == Some("Note"))
            .find_first()
            .is_none(),
        "plain text fields are not spin buttons"
    );
    let _ = editor(&window, "Amount");
}

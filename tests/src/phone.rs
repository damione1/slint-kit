// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ui;
use i_slint_backend_testing::ElementQuery;
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use std::{cell::Cell, rc::Rc};
fn fixture() -> ui::PhoneFixture {
    i_slint_backend_testing::init_no_event_loop();
    let view = ui::PhoneFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, view);
    view.show().unwrap();
    view
}
fn key(view: &ui::PhoneFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    view.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    view.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}
fn shortcut(view: &ui::PhoneFixture, text: &str) {
    view.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    key(view, text);
    view.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
}
#[test]
fn typing_blocks_letters_spaces_and_preserves_native_selection_and_undo() {
    let view = fixture();
    view.invoke_focus_phone();
    view.invoke_select_phone();
    for text in ["Ada", " ", "-", "abc123", "１２"] {
        key(&view, text);
        assert_eq!(view.get_draft(), "0612345678");
    }
    key(&view, "0123456789");
    assert_eq!(view.get_draft(), "0123456789");
    shortcut(&view, "z");
    shortcut(&view, "z");
    assert_eq!(view.get_draft(), "0612345678");
}
#[test]
fn formatted_paste_normalizes_the_value_but_not_the_native_draft() {
    let view = fixture();
    for (input, expected) in [
        ("06 98 76 54 32", "0698765432"),
        ("06letters12345678", "0612345678"),
    ] {
        view.set_clipboard(input.into());
        view.invoke_copy_clipboard();
        view.set_draft("0612345678".into());
        view.invoke_focus_phone();
        view.invoke_select_phone();
        shortcut(&view, "v");
        assert_eq!(view.get_canonical(), expected);
        if input.contains("letters") {
            assert_eq!(view.get_draft(), "0612345678");
        } else {
            assert_eq!(view.get_draft(), input);
            assert!(view.get_validity().valid);
        }
    }
}
#[test]
fn formatting_is_presentation_and_does_not_replace_the_original_editor() {
    let view = fixture();
    let editor = ElementQuery::from_root(&view)
        .match_inherits("TextInput")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Phone"))
        .find_first()
        .unwrap();
    assert_eq!(view.get_formatted(), "06 12 34 56 78");
    assert_eq!(editor.accessible_value().as_deref(), Some("06 12 34 56 78"));
    view.invoke_focus_phone();
    slint::platform::update_timers_and_animations();
    assert_eq!(editor.accessible_value().as_deref(), Some("0612345678"));
    view.invoke_blur_phone();
    slint::platform::update_timers_and_animations();
    assert_eq!(view.get_canonical(), "0612345678");
    assert_eq!(view.get_draft(), "0612345678");
    assert_eq!(editor.accessible_value().as_deref(), Some("06 12 34 56 78"));
}
#[test]
fn digit_counts_prefix_modes_and_regex_guard_commit_paths() {
    let view = fixture();
    let commits = Rc::new(Cell::new(0));
    let count = commits.clone();
    view.on_commit(move |_| count.set(count.get() + 1));
    for draft in ["123", "06123456789", "letters", "+33612345678"] {
        view.set_draft(draft.into());
        assert!(!view.invoke_report());
        view.invoke_focus_phone();
        key(&view, Key::Return);
    }
    assert_eq!(commits.get(), 0);
    view.set_number_mode(ui::PhoneNumberMode::International);
    view.set_expected_digits(11);
    view.set_display_mask("+# (###) ###-####".into());
    view.set_draft("+15145550123".into());
    assert!(view.invoke_report());
    assert_eq!(view.get_formatted(), "+1 (514) 555-0123");
    view.set_pattern(r"\+33[0-9]{9}".into());
    assert!(!view.invoke_report());
    view.set_expected_digits(11);
    view.set_draft("+33 6 12 34 56 78".into());
    assert!(view.invoke_report());
    assert_eq!(view.get_canonical(), "+33612345678");
    view.set_pattern("".into());
    view.set_expected_digits(0);
    view.set_min_digits(8);
    view.set_max_digits(15);
    view.set_draft("+1234567".into());
    assert!(!view.invoke_report());
    view.set_draft("+12345678".into());
    assert!(view.invoke_report());
    view.set_draft("+1234567890123456".into());
    assert!(!view.invoke_report());
    view.set_draft("+33612345678".into());
    view.set_display_mask("bad mask".into());
    assert!(!view.get_format_error().is_empty());
    assert_eq!(view.get_formatted(), "+33612345678");
}

#[test]
fn phone_commit_returns_the_canonical_string_with_its_leading_zero() {
    let view = fixture();
    let accepted = Rc::new(std::cell::RefCell::new(String::new()));
    let result = accepted.clone();
    view.on_commit(move |value| *result.borrow_mut() = value.to_string());
    view.set_draft("06 12 34 56 78".into());
    view.invoke_focus_phone();
    key(&view, Key::Return);
    assert_eq!(*accepted.borrow(), "0612345678");
}

#[test]
fn structural_phone_errors_are_shown_before_host_business_errors() {
    let view = fixture();
    view.set_business_error("Phone already used".into());
    view.set_draft("123".into());
    assert!(!view.invoke_report());
    let editor = ElementQuery::from_root(&view)
        .match_inherits("TextInput")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Phone"))
        .find_first()
        .unwrap();
    assert_eq!(
        editor.accessible_description().as_deref(),
        Some("Enter exactly 10 digits.")
    );
    view.set_draft("0612345678".into());
    assert!(!view.invoke_report());
    assert_eq!(
        editor.accessible_description().as_deref(),
        Some("Phone already used")
    );
}

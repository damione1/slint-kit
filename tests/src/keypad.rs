// SPDX-License-Identifier: GPL-3.0-or-later
//! The numeric keypad and the code indicator.
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, SharedString};
use std::{cell::RefCell, rc::Rc};

/// What the keypad asked the host to do, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Intention {
    Digit(i32),
    Delete,
    Confirm,
}

fn fixture() -> (KeypadFixture, Rc<RefCell<Vec<Intention>>>) {
    i_slint_backend_testing::init_no_event_loop();
    let window = KeypadFixture::new().unwrap();
    window.show().unwrap();
    let intentions: Rc<RefCell<Vec<Intention>>> = Rc::default();
    let observed = intentions.clone();
    window.on_digit_entered(move |value| observed.borrow_mut().push(Intention::Digit(value)));
    let observed = intentions.clone();
    window.on_deleted(move || observed.borrow_mut().push(Intention::Delete));
    let observed = intentions.clone();
    window.on_confirmed(move || observed.borrow_mut().push(Intention::Confirm));
    (window, intentions)
}
fn with_text_scale(window: &KeypadFixture, scale: f32) {
    let mut theme = window.get_light();
    theme.text_scale = scale;
    window.set_theme(theme);
}
fn element(window: &KeypadFixture, label: &str) -> ElementHandle {
    let requested = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(requested.as_str())
        })
        .find_first()
        .unwrap_or_else(|| panic!("missing element {label}"))
}
fn type_text(window: &KeypadFixture, text: impl Into<SharedString>) {
    let text = text.into();
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}
fn key(window: &KeypadFixture, key: Key) {
    type_text(window, key);
}
fn click(window: &KeypadFixture, element: &ElementHandle) {
    let origin = element.absolute_position();
    let size = element.size();
    let position = LogicalPosition::new(origin.x + size.width / 2., origin.y + size.height / 2.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
}
const KEYS: [&str; 12] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "Delete", "0", "Unlock",
];

#[test]
fn keys_request_their_digit_delete_and_confirm_by_pointer() {
    let (window, intentions) = fixture();
    window.set_can_confirm(true);
    for label in KEYS {
        click(&window, &element(&window, label));
    }
    let mut expected: Vec<Intention> = (1..=9).map(Intention::Digit).collect();
    expected.extend([Intention::Delete, Intention::Digit(0), Intention::Confirm]);
    assert_eq!(*intentions.borrow(), expected);
}

#[test]
fn confirm_waits_for_the_host_to_allow_it() {
    let (window, intentions) = fixture();
    let confirm = element(&window, "Unlock");
    assert_eq!(confirm.accessible_enabled(), Some(false));
    click(&window, &confirm);
    confirm.invoke_accessible_default_action();
    window.invoke_focus_keypad();
    key(&window, Key::Return);
    assert!(intentions.borrow().is_empty());
    window.set_can_confirm(true);
    assert_eq!(confirm.accessible_enabled(), Some(true));
    // A held Return confirms once.
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Return.into(),
    });
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Return.into(),
    });
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Return.into(),
    });
    assert_eq!(*intentions.borrow(), [Intention::Confirm]);
}

#[test]
fn typed_digits_backspace_and_return_reach_the_keypad() {
    let (window, intentions) = fixture();
    window.set_can_confirm(true);
    window.invoke_focus_keypad();
    for digit in ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"] {
        type_text(&window, digit);
    }
    // Other characters and keys are not digits.
    for other in ["a", "+", "-", ".", " "] {
        type_text(&window, other);
    }
    key(&window, Key::Tab);
    window.invoke_focus_keypad();
    key(&window, Key::Backspace);
    key(&window, Key::Return);
    let mut expected: Vec<Intention> = (0..=9).map(Intention::Digit).collect();
    expected.extend([Intention::Delete, Intention::Confirm]);
    assert_eq!(*intentions.borrow(), expected);
    // A shortcut is not a digit.
    intentions.borrow_mut().clear();
    window.invoke_focus_keypad();
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    type_text(&window, "4");
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    assert!(intentions.borrow().is_empty());
}

#[test]
fn return_after_tapped_digits_confirms_instead_of_repeating_a_digit() {
    let (window, intentions) = fixture();
    // A tapped key hands focus back to the keypad, so typed keys still
    // arrive and Return confirms rather than pressing the tapped key again.
    click(&window, &element(&window, "5"));
    type_text(&window, "7");
    key(&window, Key::Backspace);
    // Confirm is not allowed yet: Return and Space do nothing.
    key(&window, Key::Return);
    key(&window, Key::Space);
    window.set_can_confirm(true);
    key(&window, Key::Return);
    assert_eq!(
        *intentions.borrow(),
        [
            Intention::Digit(5),
            Intention::Digit(7),
            Intention::Delete,
            Intention::Confirm,
        ]
    );
    // The same holds after tapping Delete and after tapping Confirm.
    intentions.borrow_mut().clear();
    click(&window, &element(&window, "Delete"));
    key(&window, Key::Return);
    click(&window, &element(&window, "Unlock"));
    key(&window, Key::Return);
    assert_eq!(
        *intentions.borrow(),
        [
            Intention::Delete,
            Intention::Confirm,
            Intention::Confirm,
            Intention::Confirm,
        ]
    );
}

#[test]
fn a_key_with_keyboard_focus_activates_itself_and_passes_typed_digits_on() {
    let (window, intentions) = fixture();
    window.set_can_confirm(true);
    window.invoke_focus_keypad();
    // Tab moves keyboard focus from the keypad onto its first key.
    key(&window, Key::Tab);
    type_text(&window, "7");
    key(&window, Key::Backspace);
    // Return and Space activate the focused key, as on any button.
    key(&window, Key::Return);
    key(&window, Key::Space);
    assert_eq!(
        *intentions.borrow(),
        [
            Intention::Digit(7),
            Intention::Delete,
            Intention::Digit(1),
            Intention::Digit(1),
        ]
    );
}

#[test]
fn read_only_and_disabled_keypads_emit_nothing() {
    let (window, intentions) = fixture();
    window.set_can_confirm(true);
    for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
        window.set_access(access);
        for label in KEYS {
            let key = element(&window, label);
            assert_eq!(key.accessible_enabled(), Some(false), "{label} {access:?}");
            click(&window, &key);
            key.invoke_accessible_default_action();
        }
        window.invoke_focus_keypad();
        type_text(&window, "3");
        key(&window, Key::Backspace);
        key(&window, Key::Return);
    }
    assert!(intentions.borrow().is_empty());
    assert_eq!(
        element(&window, "Access code").accessible_enabled(),
        Some(false)
    );
    window.set_access(FieldAccess::Editable);
    window.invoke_focus_keypad();
    type_text(&window, "3");
    assert_eq!(*intentions.borrow(), [Intention::Digit(3)]);
}

#[test]
fn keys_are_touch_sized_and_laid_out_as_a_phone_keypad_at_normal_and_doubled_text() {
    let (window, _) = fixture();
    for scale in [1., 2.] {
        with_text_scale(&window, scale);
        let keys: Vec<ElementHandle> = KEYS.iter().map(|label| element(&window, label)).collect();
        for (label, key) in KEYS.iter().zip(&keys) {
            let size = key.size();
            assert!(
                size.height >= 48.,
                "{label} height {} at {scale}",
                size.height
            );
            assert!(size.width >= 48., "{label} width {} at {scale}", size.width);
        }
        // Three columns, four rows: 1 2 3 / 4 5 6 / 7 8 9 / Delete 0 Unlock.
        for row in keys.chunks(3) {
            let top = row[0].absolute_position().y;
            for (left, right) in row.iter().zip(row.iter().skip(1)) {
                assert_eq!(right.absolute_position().y, top);
                assert!(
                    left.absolute_position().x + left.size().width <= right.absolute_position().x
                );
            }
        }
        for (above, below) in keys.iter().zip(keys.iter().skip(3)) {
            assert!(
                above.absolute_position().y + above.size().height <= below.absolute_position().y
            );
            assert_eq!(above.absolute_position().x, below.absolute_position().x);
        }
        // The keypad measures exactly its grid of keys.
        let last = &keys[11];
        let first = &keys[0];
        let right = last.absolute_position().x + last.size().width - first.absolute_position().x;
        let bottom = last.absolute_position().y + last.size().height - first.absolute_position().y;
        assert!(
            (right - window.get_keypad_width()).abs() < 0.01,
            "width at {scale}"
        );
        assert!(
            (bottom - window.get_keypad_height()).abs() < 0.01,
            "height at {scale}"
        );
    }
}

#[test]
fn code_indicator_marks_how_many_digits_are_held_without_showing_them() {
    let (window, _) = fixture();
    let theme = window.get_light();
    let indicator = element(&window, "2 of 4 digits entered");
    assert_eq!(marks(&indicator).len(), 4);
    window.set_code_filled(3);
    window.set_code_length(6);
    let indicator = element(&window, "3 of 6 digits entered");
    assert_eq!(marks(&indicator).len(), 6);
    assert!(window.get_mark_row_height() >= 3. * theme.spacing.unit);
}

/// The square marks of a code indicator, in reading order.
fn marks(indicator: &ElementHandle) -> Vec<ElementHandle> {
    let mut marks: Vec<ElementHandle> = indicator
        .query_descendants()
        .match_type_name("Rectangle")
        .match_predicate(|mark| mark.size().width > 0. && mark.size().width == mark.size().height)
        .find_all();
    marks.sort_by(|left, right| {
        left.absolute_position()
            .x
            .total_cmp(&right.absolute_position().x)
    });
    marks
}

#[test]
fn code_indicator_fills_exactly_the_first_marks_in_both_themes() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    // This test rasterizes headlessly to read each mark's paint.
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = KeypadFixture::new().unwrap();
    window.show().unwrap();
    for theme in [window.get_light(), window.get_dark()] {
        window.set_theme(theme.clone());
        let ink = [
            theme.text_primary.red(),
            theme.text_primary.green(),
            theme.text_primary.blue(),
        ];
        // Nothing entered, a partial code, a complete code and a count past
        // the length, which fills every mark.
        for (length, filled) in [(4, 0), (4, 2), (4, 4), (4, 6), (6, 3)] {
            window.set_code_length(length);
            window.set_code_filled(filled);
            let indicator = element(&window, &format!("{filled} of {length} digits entered"));
            let marks = marks(&indicator);
            assert_eq!(marks.len(), usize::try_from(length).unwrap());
            let image = window.window().take_snapshot().unwrap();
            let width = usize::try_from(image.width()).unwrap();
            for (index, mark) in marks.iter().enumerate() {
                let origin = mark.absolute_position();
                let size = mark.size();
                // Whole logical pixels: the testing window has a scale of 1.
                let x = (origin.x + size.width / 2.) as usize;
                let y = (origin.y + size.height / 2.) as usize;
                let offset = (y * width + x) * 4;
                let centre = &image.as_bytes()[offset..offset + 3];
                let expected_filled = i32::try_from(index).unwrap() < filled;
                assert_eq!(
                    centre == ink,
                    expected_filled,
                    "mark {index} of {length} with {filled} filled"
                );
            }
        }
    }
}

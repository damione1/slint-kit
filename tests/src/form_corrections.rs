// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

fn fixture() -> FormCorrectionsFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = FormCorrectionsFixture::new().unwrap();
    window.show().unwrap();
    window
}
fn element(window: &FormCorrectionsFixture, label: &str) -> ElementHandle {
    let label = label.to_owned();
    let requested_label = label.clone();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(requested_label.as_str())
        })
        .find_first()
        .unwrap_or_else(|| panic!("missing element {label}"))
}
fn key(window: &FormCorrectionsFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}

#[test]
fn swatch_opens_picker_and_escape_or_disabled_access_never_commit() {
    let window = fixture();
    let commits = Rc::new(RefCell::new(Vec::new()));
    let observed = commits.clone();
    window.on_color_requested(move |value| observed.borrow_mut().push(value));
    element(&window, "Open color picker for Accent").mock_single_click(PointerEventButton::Left);
    assert!(window.get_picker_open());
    key(&window, Key::Escape);
    assert!(!window.get_picker_open());
    assert!(commits.borrow().is_empty());
    // Focus returns to the swatch: Enter reopens it.
    key(&window, Key::Return);
    assert!(window.get_picker_open());
    window.set_access(FieldAccess::ReadOnly);
    assert!(!window.get_picker_open());
    window.invoke_open_picker();
    assert!(!window.get_picker_open());
    assert!(commits.borrow().is_empty());
}

#[test]
fn color_palette_is_a_provisional_draft_until_apply() {
    let window = fixture();
    let commits = Rc::new(RefCell::new(Vec::new()));
    let observed = commits.clone();
    window.on_color_requested(move |value| observed.borrow_mut().push(value));
    window.invoke_open_picker();
    element(&window, "Choose RGB 239, 68, 68").invoke_accessible_default_action();
    assert!(commits.borrow().is_empty());
    // The popup initially focuses Cancel. Tab moves to Apply.
    key(&window, Key::Tab);
    key(&window, Key::Return);
    assert!(!window.get_picker_open());
    assert_eq!(
        &*commits.borrow(),
        &[SrgbColorView {
            red: 239,
            green: 68,
            blue: 68,
            alpha: 255
        }]
    );
}

#[test]
fn selected_chips_share_a_line_and_wrap_when_the_width_is_exhausted() {
    let window = fixture();
    window.set_chips(ModelRc::new(VecModel::from(vec![
        ChipRow {
            token: 1,
            label: "Alpha".into(),
            removable: true,
        },
        ChipRow {
            token: 2,
            label: "Beta".into(),
            removable: true,
        },
        ChipRow {
            token: 3,
            label: "Gamma".into(),
            removable: true,
        },
    ])));
    let chips: Vec<_> = ElementQuery::from_root(&window)
        .match_inherits("KitChip")
        .find_all();
    assert_eq!(chips.len(), 6);
    for pair in [(&chips[0], &chips[1]), (&chips[3], &chips[4])] {
        assert!((pair.0.absolute_position().y - pair.1.absolute_position().y).abs() < 1.);
        assert!(
            pair.1.absolute_position().x >= pair.0.absolute_position().x + pair.0.size().width + 7.
        );
    }
    window.set_viewport_width(250.);
    assert!(
        chips[2].absolute_position().y > chips[0].absolute_position().y,
        "size={:?}, chips-width={}, window-width={}, chips={:?}",
        window.window().size(),
        window.get_chips_width(),
        window.get_window_width(),
        chips
            .iter()
            .map(|c| (c.absolute_position(), c.size()))
            .collect::<Vec<_>>()
    );
    assert!(chips[5].absolute_position().y > chips[3].absolute_position().y);
}

#[test]
fn time_adjustment_buttons_are_adjacent_square_and_keep_their_typed_intents() {
    let window = fixture();
    let events = Rc::new(RefCell::new(Vec::new()));
    let observed = events.clone();
    window.on_adjust(move |segment, direction| observed.borrow_mut().push((segment, direction)));
    for scale in [1., 2.] {
        window.set_text_scale(scale);
        let decrease = element(&window, "Decrease Hours");
        let increase = element(&window, "Increase Hours");
        assert!((decrease.size().width - decrease.size().height).abs() < 1.);
        assert!((increase.size().width - increase.size().height).abs() < 1.);
        assert!((decrease.absolute_position().y - increase.absolute_position().y).abs() < 1.);
        assert!(
            (increase.absolute_position().x
                - decrease.absolute_position().x
                - decrease.size().width
                - 8.)
                .abs()
                < 1.
        );
        increase.invoke_accessible_default_action();
        element(&window, "Decrease Minutes").invoke_accessible_default_action();
    }
    assert_eq!(
        &*events.borrow(),
        &[
            (TimeSegment::Hour, TimeAdjustment::Increment),
            (TimeSegment::Minute, TimeAdjustment::Decrement),
            (TimeSegment::Hour, TimeAdjustment::Increment),
            (TimeSegment::Minute, TimeAdjustment::Decrement),
        ]
    );
}

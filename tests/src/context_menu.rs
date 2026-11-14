// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

fn settle() {
    slint::platform::update_timers_and_animations();
}
fn press(window: &impl ComponentHandle, text: slint::SharedString) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle();
}
fn press_key(window: &impl ComponentHandle, key: Key) {
    press(window, key.into());
}
fn press_shift_f10(window: &impl ComponentHandle) {
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    press_key(window, Key::F10);
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
}
fn pointer(window: &impl ComponentHandle, position: LogicalPosition, button: PointerEventButton) {
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window
        .window()
        .dispatch_event(WindowEvent::PointerPressed { position, button });
    settle();
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased { position, button });
    settle();
}
fn rows() -> ModelRc<OptionRow> {
    ModelRc::new(VecModel::from(vec![
        OptionRow {
            token: 1,
            generation: 1,
            label: "Rename".into(),
            ..Default::default()
        },
        OptionRow {
            token: 2,
            generation: 1,
            label: "Delete".into(),
            ..Default::default()
        },
    ]))
}
struct Fixture {
    window: ContextMenuFixture,
    chosen: Rc<RefCell<Vec<String>>>,
    reasons: Rc<RefCell<Vec<CloseReason>>>,
}
fn fixture() -> Fixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = ContextMenuFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_options(rows());
    let chosen = Rc::new(RefCell::new(Vec::new()));
    let sink = chosen.clone();
    window.on_chosen(move |row| sink.borrow_mut().push(row.label.to_string()));
    let reasons = Rc::new(RefCell::new(Vec::new()));
    let log = reasons.clone();
    window.on_dismissed(move |reason| log.borrow_mut().push(reason));
    window.show().unwrap();
    Fixture {
        window,
        chosen,
        reasons,
    }
}
fn menu_row(window: &ContextMenuFixture, label: &'static str) -> ElementHandle {
    ElementQuery::from_root(window)
        .match_inherits("KitNavigationAction")
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label))
        .find_first()
        .unwrap_or_else(|| panic!("missing menu row {label}"))
}

#[test]
fn right_button_opens_the_menu_at_the_pointer_and_a_row_chooses() {
    let Fixture {
        window,
        chosen,
        reasons,
    } = fixture();
    assert!(!window.get_open());
    pointer(
        &window,
        LogicalPosition::new(60., 60.),
        PointerEventButton::Left,
    );
    assert!(!window.get_open(), "the left button opens nothing");
    pointer(
        &window,
        LogicalPosition::new(60., 60.),
        PointerEventButton::Right,
    );
    assert!(window.get_open());
    // Window coordinates: the menu opens one gap below and at the pointer, so a
    // press inside its first row, offset from the pointer, chooses it.
    let rename = menu_row(&window, "Rename");
    assert!(rename.size().height > 0.);
    pointer(
        &window,
        LogicalPosition::new(70., 60. + 8. + 12. + 10.),
        PointerEventButton::Left,
    );
    assert!(!window.get_open());
    assert_eq!(*chosen.borrow(), ["Rename"]);
    assert_eq!(*reasons.borrow(), [CloseReason::Programmatic]);
    assert!(
        window.get_area_focused(),
        "focus returns to the content area"
    );
}

#[test]
fn menu_key_and_shift_f10_open_the_menu_and_escape_returns_focus() {
    let Fixture {
        window, reasons, ..
    } = fixture();
    press_key(&window, Key::Tab);
    assert!(window.get_area_focused(), "the content takes a tab stop");
    press_key(&window, Key::Menu);
    assert!(window.get_open());
    press_key(&window, Key::Escape);
    assert!(!window.get_open());
    assert!(window.get_area_focused());
    press_shift_f10(&window);
    assert!(window.get_open());
    press_key(&window, Key::DownArrow);
    press_key(&window, Key::Return);
    assert!(!window.get_open());
    assert_eq!(
        *reasons.borrow(),
        [CloseReason::Escape, CloseReason::Programmatic]
    );
    assert!(window.get_area_focused());
}

#[test]
fn menu_key_reaches_the_menu_from_a_focused_child() {
    let Fixture { window, chosen, .. } = fixture();
    press_key(&window, Key::Tab);
    press_key(&window, Key::Tab);
    assert!(window.get_button_focused());
    assert!(!window.get_area_focused());
    press_key(&window, Key::Menu);
    assert!(window.get_open());
    press_key(&window, Key::Return);
    assert_eq!(*chosen.borrow(), ["Rename"]);
}

#[test]
fn outside_press_closes_as_host_closed_without_choosing() {
    let Fixture {
        window,
        chosen,
        reasons,
    } = fixture();
    pointer(
        &window,
        LogicalPosition::new(60., 60.),
        PointerEventButton::Right,
    );
    assert!(window.get_open());
    pointer(
        &window,
        LogicalPosition::new(390., 290.),
        PointerEventButton::Left,
    );
    assert!(!window.get_open());
    assert!(chosen.borrow().is_empty());
    assert_eq!(*reasons.borrow(), [CloseReason::HostClosed]);
}

#[test]
fn menu_flips_above_and_stays_inside_the_window_near_the_corner() {
    let Fixture { window, chosen, .. } = fixture();
    // Content coordinates: the content starts at (24, 24), so this is the window point (380, 280).
    window.invoke_show_at(356., 256.);
    assert!(window.get_open());
    // Above and left of the point, inside the window: the second row is there.
    pointer(
        &window,
        LogicalPosition::new(340., 244.),
        PointerEventButton::Left,
    );
    assert_eq!(*chosen.borrow(), ["Delete"]);
}

#[test]
fn disabled_or_empty_context_menu_does_not_open() {
    let Fixture { window, .. } = fixture();
    window.set_disabled(true);
    pointer(
        &window,
        LogicalPosition::new(60., 60.),
        PointerEventButton::Right,
    );
    window.invoke_show_at(10., 10.);
    assert!(!window.get_open());
    window.set_disabled(false);
    window.set_options(ModelRc::new(VecModel::from(Vec::<OptionRow>::new())));
    pointer(
        &window,
        LogicalPosition::new(60., 60.),
        PointerEventButton::Right,
    );
    assert!(!window.get_open(), "no rows, no menu");
}

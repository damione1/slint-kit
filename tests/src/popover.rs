// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

fn settle(_window: &impl ComponentHandle) {
    slint::platform::update_timers_and_animations();
}
fn click(window: &impl ComponentHandle, element: &ElementHandle) {
    element.mock_single_click(PointerEventButton::Left);
    settle(window);
}
// A mouse click as a person makes it: hover, press, an event-loop pass, release.
fn mouse_click(window: &impl ComponentHandle, element: &ElementHandle) {
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
    settle(window);
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    settle(window);
}
fn press(window: &impl ComponentHandle, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    settle(window);
}
fn labelled(window: &impl ComponentHandle, label: &'static str) -> ElementHandle {
    ElementQuery::from_root(window)
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label))
        .find_first()
        .unwrap_or_else(|| panic!("missing element {label}"))
}
fn click_at(window: &impl ComponentHandle, position: LogicalPosition) {
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
fn popover() -> (PopoverFixture, Rc<RefCell<Vec<CloseReason>>>) {
    i_slint_backend_testing::init_no_event_loop();
    let window = PopoverFixture::new().unwrap();
    slint_kit::install(&window);
    let reasons = Rc::new(RefCell::new(Vec::new()));
    let sink = reasons.clone();
    window.on_dismissed(move |reason| sink.borrow_mut().push(reason));
    window.show().unwrap();
    (window, reasons)
}

#[test]
fn popover_opens_from_trigger_and_closes_with_escape_returning_focus() {
    let (window, reasons) = popover();
    click(&window, &labelled(&window, "Details"));
    assert!(window.get_open());
    press(&window, Key::Escape);
    assert!(!window.get_open());
    assert!(window.get_trigger_focused(), "Escape returns focus");
    assert_eq!(*reasons.borrow(), [CloseReason::Escape]);
}

#[test]
fn popover_trigger_toggles_and_outside_press_closes() {
    let (window, reasons) = popover();
    let trigger = labelled(&window, "Details");
    mouse_click(&window, &trigger);
    assert!(window.get_open());
    mouse_click(&window, &trigger);
    assert!(
        !window.get_open(),
        "a second trigger press closes the panel and does not reopen it"
    );
    mouse_click(&window, &trigger);
    assert!(window.get_open());
    let outside = labelled(&window, "Outside action");
    let origin = outside.absolute_position();
    click_at(&window, LogicalPosition::new(origin.x + 4., origin.y + 4.));
    assert!(!window.get_open());
    assert_eq!(
        *reasons.borrow(),
        [CloseReason::HostClosed, CloseReason::HostClosed]
    );
}

#[test]
fn popover_programmatic_show_and_close_report_and_disabled_blocks() {
    let (window, reasons) = popover();
    let opened = Rc::new(RefCell::new(0));
    let sink = opened.clone();
    window.on_opened(move || *sink.borrow_mut() += 1);
    window.set_disabled(true);
    window.invoke_show();
    assert!(!window.get_open(), "disabled popover does not open");
    window.set_disabled(false);
    window.invoke_show();
    assert!(window.get_open());
    window.invoke_dismiss();
    assert!(!window.get_open());
    assert_eq!(*opened.borrow(), 1);
    assert_eq!(*reasons.borrow(), [CloseReason::Programmatic]);
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

#[test]
fn dropdown_menu_opens_with_arrow_key_and_chooses_with_keyboard() {
    i_slint_backend_testing::init_no_event_loop();
    let window = DropdownMenuFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_options(rows());
    let chosen = Rc::new(RefCell::new(Vec::new()));
    let sink = chosen.clone();
    window.on_chosen(move |row| sink.borrow_mut().push(row.label.to_string()));
    window.show().unwrap();
    let trigger = labelled(&window, "Actions");
    click(&window, &trigger);
    assert!(window.get_open());
    press(&window, Key::DownArrow);
    press(&window, Key::Return);
    assert!(!window.get_open(), "choosing closes the menu");
    assert_eq!(*chosen.borrow(), ["Delete"]);
    assert!(window.get_trigger_focused(), "focus returns to the trigger");
}

#[test]
fn dropdown_menu_choice_by_pointer_and_escape_report_their_reason() {
    i_slint_backend_testing::init_no_event_loop();
    let window = DropdownMenuFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_options(rows());
    let chosen = Rc::new(RefCell::new(Vec::new()));
    let sink = chosen.clone();
    window.on_chosen(move |row| sink.borrow_mut().push(row.label.to_string()));
    let reasons = Rc::new(RefCell::new(Vec::new()));
    let log = reasons.clone();
    window.on_dismissed(move |reason| log.borrow_mut().push(reason));
    window.show().unwrap();
    let trigger = labelled(&window, "Actions");
    click(&window, &trigger);
    press(&window, Key::Escape);
    assert!(!window.get_open());
    click(&window, &trigger);
    let row = ElementQuery::from_root(&window)
        .match_inherits("KitNavigationAction")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Rename"))
        .find_first()
        .expect("menu row");
    click(&window, &row);
    assert_eq!(*chosen.borrow(), ["Rename"]);
    assert_eq!(
        *reasons.borrow(),
        [CloseReason::Escape, CloseReason::Programmatic]
    );
}

// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

fn option(token: i32, label: &str, disabled: bool) -> OptionRow {
    OptionRow {
        token,
        generation: 1,
        label: label.into(),
        description: "".into(),
        disabled,
    }
}

fn item(kind: MenuItemKind, shortcut: &str, destructive: bool, checked: bool) -> MenuItem {
    MenuItem {
        kind,
        leading_icon: Default::default(),
        shortcut: shortcut.into(),
        destructive,
        checked,
    }
}

fn fixture() -> MenuItemsFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = MenuItemsFixture::new().unwrap();
    window.set_rows(ModelRc::new(VecModel::from(vec![
        option(1, "Copy", false),
        option(2, "Show grid", false),
        option(3, "Dense", false),
        option(4, "Danger zone", false),
        option(5, "Delete", false),
    ])));
    window.set_items(ModelRc::new(VecModel::from(vec![
        item(MenuItemKind::Action, "Ctrl+C", false, false),
        item(MenuItemKind::Checkbox, "", false, true),
        item(MenuItemKind::Radio, "", false, false),
        item(MenuItemKind::Separator, "", false, false),
        item(MenuItemKind::Action, "", true, false),
    ])));
    window.show().unwrap();
    window
}

fn row(window: &MenuItemsFixture, label: &str) -> ElementHandle {
    let label = label.to_string();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(label.as_str())
        })
        .find_first()
        .expect("menu row")
}

#[test]
fn checkbox_and_radio_rows_report_their_checked_state_to_assistive_technology() {
    let window = fixture();
    let grid = row(&window, "Show grid");
    assert_eq!(grid.accessible_checkable(), Some(true));
    assert_eq!(grid.accessible_checked(), Some(true));
    let dense = row(&window, "Dense");
    assert_eq!(dense.accessible_checkable(), Some(true));
    assert_eq!(dense.accessible_checked(), Some(false));
    let copy = row(&window, "Copy");
    assert_eq!(copy.accessible_checkable(), Some(false));
}

#[test]
fn shortcut_text_is_described_to_assistive_technology() {
    let window = fixture();
    assert_eq!(
        row(&window, "Copy").accessible_description().as_deref(),
        Some("Ctrl+C")
    );
}

#[test]
fn separator_rows_are_inert_even_when_the_model_does_not_disable_them() {
    let window = fixture();
    let activations = Rc::new(RefCell::new(Vec::new()));
    let target = activations.clone();
    window.on_activated(move |token, generation| target.borrow_mut().push((token, generation)));
    let separator = row(&window, "Danger zone");
    assert_eq!(separator.accessible_enabled(), Some(false));
    let origin = separator.absolute_position();
    let position = LogicalPosition::new(
        origin.x + separator.size().width / 2.,
        origin.y + separator.size().height / 2.,
    );
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
    assert!(activations.borrow().is_empty());
    let delete = row(&window, "Delete");
    assert_eq!(delete.accessible_enabled(), Some(true));
    delete.invoke_accessible_default_action();
    assert_eq!(*activations.borrow(), vec![(5, 1)]);
}

#[test]
fn natural_height_cap_is_a_property() {
    let window = fixture();
    let default_height = window.get_menu_height();
    window.set_max_natural_height(40.0);
    assert!(window.get_menu_height() < default_height);
    assert_eq!(window.get_menu_height(), 40.0);
}

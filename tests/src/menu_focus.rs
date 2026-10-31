// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, Model, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

#[test]
fn drawer_keeps_simple_content_grouped_below_an_intrinsic_header() {
    i_slint_backend_testing::init_no_event_loop();
    let window = DrawerContentFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, window);
    window.show().unwrap();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme.clone());
        let find = |label: &str| {
            let label = label.to_owned();
            ElementQuery::from_root(&window)
                .match_predicate(move |element| {
                    element.accessible_label().as_deref() == Some(label.as_str())
                })
                .find_first()
                .unwrap()
        };
        let close = find("Close panel");
        let caption = find("Edit this project");
        let field = ElementQuery::from_root(&window)
            .match_inherits("KitTextField")
            .find_first()
            .unwrap();
        assert!(
            caption.absolute_position().y - close.absolute_position().y - close.size().height
                <= theme.spacing.unit * 4. + 1.,
            "the header must not absorb the drawer's unused height"
        );
        assert!(
            field.absolute_position().y - caption.absolute_position().y - caption.size().height
                <= theme.spacing.unit * 4. + 1.,
            "related drawer content must retain the same section gap"
        );
        let save = find("Save changes");
        assert!(save.absolute_position().y + save.size().height <= 600. - theme.spacing.unit * 4.);
    }
}

fn row(window: &MenuFocusFixture) -> ElementHandle {
    ElementQuery::from_root(window)
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Destination"))
        .find_first()
        .expect("public menu row")
}

#[test]
fn menu_reports_keyboard_and_pointer_focus_without_cancelling_activation() {
    i_slint_backend_testing::init_no_event_loop();
    let window = MenuFocusFixture::new().unwrap();
    window.set_rows(ModelRc::new(VecModel::from(vec![OptionRow {
        token: 17,
        generation: 1,
        label: "Destination".into(),
        description: "".into(),
        disabled: false,
    }])));
    window.show().unwrap();
    let observed = Rc::new(RefCell::new(Vec::new()));
    let target = observed.clone();
    window.on_observed(move |focused| target.borrow_mut().push(focused));
    let activations = Rc::new(RefCell::new(Vec::new()));
    let target = activations.clone();
    window.on_activated(move |token, generation| target.borrow_mut().push((token, generation)));
    window.invoke_focus_outside();
    assert!(!window.get_menu_focused());
    window.invoke_focus_menu();
    assert!(window.get_menu_focused());
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Return.into(),
    });
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Return.into(),
    });
    assert_eq!(&*activations.borrow(), &[(17, 1)]);
    window.invoke_focus_outside();
    assert!(!window.get_menu_focused());

    let destination = row(&window);
    let origin = destination.absolute_position();
    let position = LogicalPosition::new(
        origin.x + destination.size().width / 2.,
        origin.y + destination.size().height / 2.,
    );
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    assert!(
        window.get_menu_focused(),
        "row focus must be visible before release"
    );
    assert_eq!(activations.borrow().len(), 1);
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    assert_eq!(
        &*activations.borrow(),
        &[(17, 1), (17, 1)],
        "observing focus must preserve the press"
    );
    assert!(window.get_menu_focused());
    window.invoke_focus_outside();
    assert!(!window.get_menu_focused());
    slint::platform::update_timers_and_animations();
    assert!(observed.borrow().contains(&true));
    assert_eq!(observed.borrow().last(), Some(&false));
}

#[test]
fn menu_does_not_report_a_removed_generation_as_focused() {
    i_slint_backend_testing::init_no_event_loop();
    let window = MenuFocusFixture::new().unwrap();
    window.set_rows(ModelRc::new(VecModel::from(vec![OptionRow {
        token: 17,
        generation: 1,
        label: "Destination".into(),
        description: "".into(),
        disabled: false,
    }])));
    window.show().unwrap();
    window.invoke_focus_outside();
    let destination = row(&window);
    let origin = destination.absolute_position();
    let position = LogicalPosition::new(origin.x + 10., origin.y + 10.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    assert!(window.get_menu_focused());
    window.set_rows(ModelRc::new(VecModel::<OptionRow>::default()));
    window.set_generation(2);
    assert!(
        !window.get_menu_focused(),
        "a destroyed row cannot keep the modal focus registry occupied"
    );
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    window.invoke_focus_menu();
    assert!(
        window.get_menu_focused(),
        "empty menu still supports explicit keyboard focus"
    );
}

#[test]
fn menu_keeps_observing_a_focused_delegate_reconciled_in_place() {
    i_slint_backend_testing::init_no_event_loop();
    let window = MenuFocusFixture::new().unwrap();
    let rows = Rc::new(VecModel::from(vec![OptionRow {
        token: 17,
        generation: 1,
        label: "Destination".into(),
        description: "".into(),
        disabled: false,
    }]));
    window.set_rows(ModelRc::from(rows.clone()));
    window.show().unwrap();
    window.invoke_focus_outside();
    let destination = row(&window);
    let origin = destination.absolute_position();
    let position = LogicalPosition::new(origin.x + 10., origin.y + 10.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    assert!(window.get_menu_focused());
    let mut replacement = rows.row_data(0).unwrap();
    replacement.generation = 2;
    rows.set_row_data(0, replacement);
    window.set_generation(2);
    slint::platform::update_timers_and_animations();
    assert!(
        window.get_menu_focused(),
        "reconciliation must not hide a delegate that still owns actual focus"
    );
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    window.invoke_focus_outside();
    assert!(!window.get_menu_focused());
}

#[test]
fn an_inactive_menu_rejects_keyboard_pointer_and_programmatic_focus_without_losing_its_rows() {
    i_slint_backend_testing::init_no_event_loop();
    let window = MenuFocusFixture::new().unwrap();
    window.set_rows(ModelRc::from([OptionRow {
        token: 17,
        generation: 1,
        label: "Destination".into(),
        ..Default::default()
    }]));
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = calls.clone();
    window.on_activated(move |token, generation| output.borrow_mut().push((token, generation)));
    window.show().unwrap();
    let item = row(&window);
    let p = item.absolute_position();
    let position = LogicalPosition::new(p.x + 10., p.y + 10.);
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window.set_interactive(false);
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    window.invoke_focus_outside();
    window.invoke_focus_menu();
    assert!(!window.get_menu_focused());
    row(&window).mock_single_click(PointerEventButton::Left);
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Return.into(),
    });
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Return.into(),
    });
    assert!(calls.borrow().is_empty());
    assert_eq!(window.get_rows().row_count(), 1);
    window.set_interactive(true);
    window.invoke_focus_menu();
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Return.into(),
    });
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Return.into(),
    });
    assert_eq!(&*calls.borrow(), &[(17, 1)]);
}

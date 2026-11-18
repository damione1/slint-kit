// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ModalSelectFixture;
use i_slint_backend_testing::{
    AccessibleRole, ElementHandle, TestingBackend, TestingBackendOptions,
};
use slint::{
    ComponentHandle, LogicalPosition, SharedString,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::ui::{CloseReason, FieldAccess, Ownership, SelectionRef};
use std::{cell::RefCell, rc::Rc};

fn fixture() -> ModalSelectFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        renderer_name: Some("software".into()),
        mock_time: true,
        ..Default::default()
    })))
    .unwrap();
    let app = ModalSelectFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    settle(&app);
    app
}

fn settle(app: &ModalSelectFixture) {
    for _ in 0..3 {
        slint::platform::update_timers_and_animations();
        app.window().take_snapshot().unwrap();
    }
}

fn click(app: &ModalSelectFixture, position: LogicalPosition) {
    app.window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    for event in [
        WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        },
        WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        },
    ] {
        app.window().dispatch_event(event);
    }
    settle(app);
}

fn key(app: &ModalSelectFixture, key: impl Into<SharedString>) {
    let text = key.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle(app);
}

fn center(app: &ModalSelectFixture, role: AccessibleRole, label: &str) -> LogicalPosition {
    let handle = ElementHandle::find_by_accessible_label(app, label)
        .find(|item| item.accessible_role() == Some(role))
        .unwrap();
    let p = handle.absolute_position();
    let s = handle.size();
    LogicalPosition::new(p.x + s.width / 2., p.y + s.height / 2.)
}

fn reasons(app: &ModalSelectFixture) -> Rc<RefCell<Vec<CloseReason>>> {
    let reasons = Rc::new(RefCell::new(Vec::new()));
    let events = reasons.clone();
    app.on_dismissed(move |reason| events.borrow_mut().push(reason));
    reasons
}

#[test]
fn first_dialog_click_reaches_its_action_with_or_without_a_popup() {
    let app = fixture();
    let reasons = reasons(&app);
    app.on_chosen(|_| panic!("background choice"));
    let mut clicks = 0;
    let mut action_center = None;
    for host in [false, true] {
        app.set_host_tabs(host);
        for with_popup in [false, true] {
            reasons.borrow_mut().clear();
            if with_popup {
                click(&app, center(&app, AccessibleRole::Combobox, "Single"));
                assert!(app.get_picker_open() && app.get_picker_visible());
            }
            app.set_dialog_open(true);
            settle(&app);
            assert!(app.get_inert());
            assert!(!app.get_picker_open() && !app.get_picker_visible());
            assert!(app.get_close_focused() && !app.get_background_focused());
            assert_eq!(reasons.borrow().len(), usize::from(with_popup));
            if with_popup {
                assert_eq!(reasons.borrow()[0], CloseReason::Programmatic);
            }
            let position = center(&app, AccessibleRole::Button, "Close");
            assert_eq!(*action_center.get_or_insert(position), position);
            click(&app, position);
            clicks += 1;
            assert_eq!(app.get_actions(), clicks);
            assert!(!app.get_dialog_open());
        }
    }
}

#[test]
fn all_choice_popups_respect_layer_access_and_modal_keyboard_focus() {
    let app = fixture();
    let reasons = reasons(&app);
    app.set_host_tabs(true);
    app.on_chosen(|_| panic!("a blocked field proposed a value"));
    for (open, is_open) in [
        (
            ModalSelectFixture::invoke_open_picker as fn(&ModalSelectFixture),
            ModalSelectFixture::get_picker_open as fn(&ModalSelectFixture) -> bool,
        ),
        (
            ModalSelectFixture::invoke_open_multi,
            ModalSelectFixture::get_multi_open,
        ),
        (
            ModalSelectFixture::invoke_open_combo,
            ModalSelectFixture::get_combo_open,
        ),
    ] {
        app.set_field_layer(1);
        app.set_dialog_open(true);
        settle(&app);
        open(&app);
        settle(&app);
        assert!(is_open(&app));
        assert!(
            ElementHandle::find_by_accessible_label(&app, "Alpha")
                .any(|e| e.accessible_role() == Some(AccessibleRole::ListItem)
                    && e.accessible_enabled() == Some(true))
        );
        app.set_dialog_open(false);
        settle(&app);
        assert!(is_open(&app));
        app.set_field_layer(0);
        app.set_dialog_open(true);
        settle(&app);
        assert!(!is_open(&app) && app.get_close_focused());
        open(&app);
        settle(&app);
        assert!(!is_open(&app));
        key(&app, Key::DownArrow);
        let tabs = app.get_tabs();
        key(&app, Key::Tab);
        assert_eq!(app.get_tabs(), tabs + 1);
        key(&app, Key::Escape);
        assert!(!app.get_dialog_open() && !is_open(&app));
        for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
            reasons.borrow_mut().clear();
            open(&app);
            settle(&app);
            assert!(is_open(&app));
            app.set_access(access);
            settle(&app);
            assert!(!is_open(&app));
            assert_eq!(&*reasons.borrow(), &[CloseReason::Programmatic]);
            open(&app);
            settle(&app);
            assert!(!is_open(&app));
            app.set_access(FieldAccess::Editable);
            settle(&app);
        }
    }
}

#[test]
fn controlled_value_and_late_session_acknowledgment_survive_modal_interruption() {
    let app = fixture();
    app.set_host_tabs(true);
    app.set_ownership(Ownership::Controlled);
    let reasons = reasons(&app);
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let seen = proposals.clone();
    app.on_chosen(move |row| seen.borrow_mut().push(row.token));
    click(&app, center(&app, AccessibleRole::Combobox, "Single"));
    key(&app, Key::End);
    key(&app, Key::Return);
    assert_eq!(&*proposals.borrow(), &[2]);
    assert!(!app.get_accepted().present);
    // Request another open and publish the modal in the same turn.
    app.invoke_open_picker();
    app.set_dialog_open(true);
    settle(&app);
    app.set_accepted(SelectionRef {
        present: true,
        token: 2,
    });
    settle(&app);
    assert!(!app.get_picker_visible() && !app.get_picker_open());
    assert!(app.get_close_focused() && app.get_accepted().present);
    assert_eq!(app.get_accepted().token, 2);
    key(&app, Key::Escape);
    assert!(!app.get_dialog_open());

    app.set_projected_open(true);
    settle(&app);
    assert!(app.get_projected_visible());
    reasons.borrow_mut().clear();
    app.set_dialog_open(true);
    settle(&app);
    assert!(app.get_projected_open() && !app.get_projected_visible());
    assert_eq!(&*reasons.borrow(), &[CloseReason::Programmatic]);
    // The owner keeps open=true. Every subsequent access restriction still
    // reports its own dismissal, even while the popup remains modal-inert.
    for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
        reasons.borrow_mut().clear();
        app.set_access(access);
        settle(&app);
        assert_eq!(
            &*reasons.borrow(),
            &[CloseReason::Programmatic],
            "{access:?}"
        );
        assert!(
            app.get_projected_open() && !app.get_projected_visible() && app.get_close_focused()
        );
    }
    app.set_access(FieldAccess::Editable);
    key(&app, Key::Escape);
    assert!(!app.get_dialog_open() && !app.get_projected_visible());
    app.set_dialog_open(true);
    settle(&app);
    app.set_projected_open(false);
    settle(&app);
    assert!(app.get_close_focused());
    assert_eq!(&*reasons.borrow(), &[CloseReason::Programmatic]);
    key(&app, Key::Escape);
    app.set_projected_open(true);
    settle(&app);
    assert!(app.get_projected_visible());
    reasons.borrow_mut().clear();
    app.set_dialog_open(true);
    app.set_access(FieldAccess::ReadOnly);
    settle(&app);
    assert_eq!(&*reasons.borrow(), &[CloseReason::Programmatic]);
    assert!(app.get_projected_open() && !app.get_projected_visible() && app.get_close_focused());
    assert_eq!(&*proposals.borrow(), &[2]);
}

#[test]
fn opening_inline_suggestions_preserves_the_native_selection_caret_and_undo() {
    let app = fixture();
    app.set_min_query_length(10);
    click(&app, center(&app, AccessibleRole::TextInput, "Suggestions"));
    key(&app, "aébc");
    key(&app, Key::LeftArrow);
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    key(&app, Key::LeftArrow);
    key(&app, Key::LeftArrow);
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    assert_eq!(app.get_combo_text(), "aébc");
    assert!(!app.get_combo_open());
    app.invoke_open_combo();
    settle(&app);
    assert!(app.get_combo_open());
    key(&app, "X");
    assert_eq!(app.get_combo_text(), "aXc");
    // Slint records replacement as deletion followed by insertion.
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    key(&app, "z");
    assert_eq!(app.get_combo_text(), "ac");
    key(&app, "z");
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    assert_eq!(app.get_combo_text(), "aébc");
}

#[test]
fn modal_interruption_discards_the_custom_draft_without_changing_the_accepted_value() {
    let app = fixture();
    app.set_ownership(Ownership::Controlled);
    app.set_combo_selected(SelectionRef {
        present: true,
        token: 1,
    });
    settle(&app);
    app.on_chosen(|_| panic!("background draft was committed"));
    click(&app, center(&app, AccessibleRole::TextInput, "Suggestions"));
    key(&app, "draft");
    assert!(app.get_combo_open() && app.get_combo_text().contains("draft"));
    app.set_dialog_open(true);
    settle(&app);
    app.invoke_clear_combo();
    settle(&app);
    assert_eq!(app.get_combo_selected().token, 1);
    assert_eq!(app.get_combo_text(), "Alpha");
    assert!(app.get_close_focused() && !app.get_combo_open());
}

#[test]
fn composed_popover_releases_first_modal_click_and_focus() {
    let app = fixture();
    let reasons = reasons(&app);
    for host in [false, true] {
        app.set_host_tabs(host);
        for (external, with_popup) in [(false, false), (false, true), (true, true)] {
            app.set_external_popover(external);
            reasons.borrow_mut().clear();
            if with_popup {
                app.invoke_open_popover();
                settle(&app);
                assert!(app.get_popover_visible());
            }
            app.set_dialog_open(true);
            settle(&app);
            assert!(app.get_inert() && app.get_close_focused());
            assert!(!app.get_popover_visible() && !app.get_popover_focused());
            let actions = app.get_actions();
            click(&app, center(&app, AccessibleRole::Button, "Close"));
            assert_eq!(app.get_actions(), actions + 1);
            assert!(!app.get_dialog_open());
            let expected = with_popup.then_some(CloseReason::Programmatic);
            assert_eq!(reasons.borrow().as_slice(), expected.as_slice());
        }
    }
}

#[test]
fn open_dropdown_and_context_menus_release_the_first_modal_click_and_focus() {
    let app = fixture();
    app.set_test_menus(true);
    let dismissals = reasons(&app);
    for host in [false, true] {
        app.set_host_tabs(host);
        for open in [
            None,
            Some(ModalSelectFixture::invoke_open_dropdown as fn(&ModalSelectFixture)),
            Some(ModalSelectFixture::invoke_open_context),
        ] {
            dismissals.borrow_mut().clear();
            if let Some(open) = open {
                open(&app);
                settle(&app);
                assert!(app.get_menu_visible());
            }
            app.set_dialog_open(true);
            settle(&app);
            assert!(app.get_inert() && app.get_close_focused());
            assert!(!app.get_menu_visible() && !app.get_menu_focused());
            let actions = app.get_actions();
            click(&app, center(&app, AccessibleRole::Button, "Close"));
            assert_eq!(app.get_actions(), actions + 1);
            assert!(!app.get_dialog_open());
            assert_eq!(app.get_menu_choices(), 0);
            assert_eq!(
                dismissals.borrow().as_slice(),
                open.map(|_| CloseReason::Programmatic).as_slice()
            );
        }
    }
}

#[test]
fn menu_reopen_from_opened_or_dismissed_preserves_the_next_dismissal_and_choice() {
    use std::cell::Cell;
    let app = fixture();
    app.set_test_menus(true);
    let log = Rc::new(RefCell::new(Vec::new()));
    for open in [
        ModalSelectFixture::invoke_open_dropdown as fn(&ModalSelectFixture),
        ModalSelectFixture::invoke_open_context,
    ] {
        // Close the first opening synchronously. Its dismissal reopens the same menu.
        let close_once = Cell::new(true);
        let weak = app.as_weak();
        app.on_menu_opened(move || {
            if close_once.replace(false) {
                weak.unwrap().invoke_close_menus();
            }
        });
        let weak = app.as_weak();
        let events = log.clone();
        let reopen_once = Cell::new(true);
        app.on_dismissed(move |reason| {
            events.borrow_mut().push(reason);
            if reopen_once.replace(false) {
                open(&weak.unwrap());
            }
        });
        log.borrow_mut().clear();
        let choices = app.get_menu_choices();
        open(&app);
        settle(&app);
        assert!(app.get_menu_visible());
        assert_eq!(*log.borrow(), [CloseReason::Programmatic]);
        key(&app, Key::Escape);
        assert!(!app.get_menu_visible() && app.get_menu_focused());
        assert_eq!(
            *log.borrow(),
            [CloseReason::Programmatic, CloseReason::Escape]
        );
        open(&app);
        settle(&app);
        click(&app, LogicalPosition::new(660., 540.));
        assert!(!app.get_menu_visible());
        assert_eq!(log.borrow().last(), Some(&CloseReason::HostClosed));
        assert_eq!(app.get_menu_choices(), choices);
        open(&app);
        settle(&app);
        key(&app, Key::Return);
        assert_eq!(app.get_menu_choices(), choices + 1);
        assert_eq!(
            *log.borrow(),
            [
                CloseReason::Programmatic,
                CloseReason::Escape,
                CloseReason::HostClosed,
                CloseReason::Programmatic
            ]
        );
    }
}

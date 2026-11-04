// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::ElementQuery;
use slint::{
    ComponentHandle,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{cell::Cell, rc::Rc};

#[test]
fn modal_backdrop_blocks_background_wheel_without_blocking_panel_scrolling() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ModalWheelFixture::new().unwrap();
    window.show().unwrap();
    let scroll = |x, y| {
        window
            .window()
            .dispatch_event(WindowEvent::PointerScrolled {
                position: slint::LogicalPosition::new(x, y),
                delta_x: 0.,
                delta_y: -120.,
            });
    };
    scroll(48., 48.);
    assert_eq!(
        window.get_background_offset(),
        0.,
        "the scrim consumes wheel events"
    );
    scroll(300., 200.);
    assert!(
        window.get_inner_offset() < 0.,
        "the actual panel viewport still scrolls"
    );
    assert_eq!(window.get_background_offset(), 0.);
    window.set_opened(false);
    scroll(48., 48.);
    assert!(
        window.get_background_offset() < 0.,
        "closing the modal restores background scrolling"
    );
}

fn key(window: &NativeDialogFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
#[test]
fn native_dialog_traverses_real_editors_skips_disabled_controls_and_wraps_in_both_directions() {
    i_slint_backend_testing::init_no_event_loop();
    let window = NativeDialogFixture::new().unwrap();
    let calls = Rc::new(Cell::new(0));
    let output = calls.clone();
    window.on_outside_activated(move || output.set(output.get() + 1));
    let sorts = calls.clone();
    window.on_outside_sort(move || sorts.set(sorts.get() + 1));
    window.show().unwrap();
    window.invoke_focus_first();
    assert!(window.get_first_focused());
    for _ in 0..3 {
        key(&window, Key::Tab);
        assert!(window.get_editor_focused());
        key(&window, Key::Tab);
        assert!(window.get_last_focused());
        key(&window, Key::Tab);
        assert!(window.get_first_focused());
        key(&window, Key::Backtab);
        assert!(window.get_last_focused());
        key(&window, Key::Backtab);
        assert!(window.get_editor_focused());
        key(&window, Key::Backtab);
        assert!(window.get_first_focused());
    }
    window.invoke_focus_background_menu();
    window.invoke_focus_background_table();
    assert!(window.get_first_focused());
    for label in [
        "Background destination",
        "Sortable background",
        "Background row",
        "Outside",
    ] {
        ElementQuery::from_root(&window)
            .match_predicate(|element| element.accessible_label().as_deref() == Some(label))
            .find_first()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);
    }
    assert_eq!(calls.get(), 0);
    window.invoke_focus_first();
    window.set_tab_policy(DialogTabPolicy::Host);
    let tab = Rc::new(Cell::new(0));
    let output = tab.clone();
    window.on_host_tab(move |_| output.set(output.get() + 1));
    key(&window, Key::Tab);
    assert_eq!(tab.get(), 1);
    assert!(window.get_first_focused());
}

#[test]
fn backdrop_pointer_dismissal_respects_panel_and_modal_policies() {
    i_slint_backend_testing::init_no_event_loop();
    let window = DialogBackdropFixture::new().unwrap();
    let reasons = Rc::new(std::cell::RefCell::new(Vec::new()));
    let output = reasons.clone();
    window.on_dismissed(move |reason| output.borrow_mut().push(reason));
    let background = Rc::new(Cell::new(0));
    let output = background.clone();
    window.on_background_clicked(move || output.set(output.get() + 1));
    window.show().unwrap();
    let click = |x, y| {
        let position = slint::LogicalPosition::new(x, y);
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
    };
    for policy in [DialogTabPolicy::Native, DialogTabPolicy::Host] {
        window.set_tab_policy(policy);
        reasons.borrow_mut().clear();
        click(200., 100.);
        assert!(
            reasons.borrow().is_empty(),
            "empty panel space is inside the dialog"
        );
        click(40., 40.);
        assert_eq!(&*reasons.borrow(), &[CloseReason::OutsidePress]);
        assert_eq!(
            background.get(),
            0,
            "a scrim click never reaches the background"
        );
        window.set_busy(true);
        click(40., 40.);
        window.set_busy(false);
        window.set_topmost(false);
        click(40., 40.);
        window.set_topmost(true);
        window.set_dismiss_enabled(false);
        click(40., 40.);
        window.set_dismiss_enabled(true);
        assert_eq!(
            reasons.borrow().len(),
            1,
            "all dismissal guards remain effective"
        );
        assert_eq!(background.get(), 0);
    }
    window.set_opened(false);
    click(40., 40.);
    assert_eq!(
        background.get(),
        1,
        "closed dialogs release background input"
    );
}

// SPDX-License-Identifier: GPL-3.0-or-later
//! KitDialog sizing, scrolling, focus and actions; KitAlertDialog answers.
use crate::{AlertDialogFixture, CloseReason, DialogFixture, DialogSize, DialogState};
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::{
    ComponentHandle,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{cell::RefCell, rc::Rc};

fn state(open: bool, busy: bool) -> DialogState {
    DialogState {
        open,
        topmost: true,
        busy,
    }
}
fn press<W: ComponentHandle>(window: &W, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    slint::platform::update_timers_and_animations();
}
fn labelled<W: ComponentHandle>(window: &W, label: &str) -> Vec<ElementHandle> {
    let label = label.to_owned();
    ElementQuery::from_root(window)
        .match_inherits("KitPressable")
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label.as_str()))
        .find_all()
}
fn one<W: ComponentHandle>(window: &W, label: &str) -> ElementHandle {
    let mut found = labelled(window, label);
    assert_eq!(found.len(), 1, "exactly one element labelled {label}");
    found.remove(0)
}
fn click(element: &ElementHandle) {
    element.mock_single_click(PointerEventButton::Left);
}
fn panel<W: ComponentHandle>(window: &W) -> (f32, f32, f32, f32) {
    let panel = ElementQuery::from_root(window)
        .match_inherits("KitDialogPanel")
        .find_first()
        .expect("dialog panel");
    let (position, size) = (panel.absolute_position(), panel.size());
    (position.x, position.y, size.width, size.height)
}
type Log<T> = Rc<RefCell<Vec<T>>>;
fn recorder<T: 'static>() -> (Log<T>, Log<T>) {
    let log = Rc::new(RefCell::new(Vec::new()));
    (log.clone(), log)
}
fn dialog(open: bool) -> DialogFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = DialogFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_state(state(open, false));
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}
fn alert(open: bool) -> AlertDialogFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = AlertDialogFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_state(state(open, false));
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}

#[test]
fn size_presets_set_the_width_and_never_exceed_the_window() {
    let window = dialog(true);
    // (size, expected width) in an 800px window with a 24px margin.
    for (size, width) in [
        (DialogSize::Small, 360.),
        (DialogSize::Medium, 520.),
        (DialogSize::Large, 752.),
    ] {
        window.set_size(size);
        slint::platform::update_timers_and_animations();
        let (x, _, actual, _) = panel(&window);
        assert_eq!(actual, width, "{size:?}");
        assert_eq!(x, (800. - width) / 2., "{size:?} stays centered");
    }
    window.set_dialog_width(300.);
    window.set_size(DialogSize::Medium);
    slint::platform::update_timers_and_animations();
    assert_eq!(panel(&window).2, 252., "a narrow window bounds the panel");
}

#[test]
fn panel_follows_a_short_body_and_a_tall_body_scrolls_inside_the_window() {
    let window = dialog(true);
    let (_, y, _, short) = panel(&window);
    assert!(
        short < 300.,
        "a short body does not stretch the panel: {short}"
    );
    assert_eq!(
        y,
        (600. - short) / 2.,
        "the panel stays vertically centered"
    );
    window.set_body_lines(80);
    slint::platform::update_timers_and_animations();
    let (_, y, _, tall) = panel(&window);
    assert_eq!(
        (y, tall),
        (24., 552.),
        "a tall body stops at the window margin"
    );
    let footer = one(&window, "Save").absolute_position().y;
    assert!(
        footer > 24. + 400. && footer < 600.,
        "the footer stays inside the window: {footer}"
    );
    let last_line = || {
        ElementQuery::from_root(&window)
            .match_predicate(|e| e.accessible_label().as_deref() == Some("Line 79"))
            .find_first()
            .map(|line| line.absolute_position().y)
    };
    assert!(
        last_line().is_none_or(|y| y > footer),
        "the last line starts below the visible body"
    );
    press(&window, Key::Tab);
    press(&window, Key::Tab);
    press(&window, Key::End);
    press(&window, Key::Tab);
    press(&window, Key::End);
    let last = last_line().expect("End scrolls the last line into view");
    assert!(
        last < footer,
        "the last line is above the footer: {last} < {footer}"
    );
    assert_eq!(
        one(&window, "Save").absolute_position().y,
        footer,
        "the footer does not scroll with the body"
    );
}

#[test]
fn opening_focuses_close_and_it_reports_the_close_button_reason() {
    let window = dialog(false);
    let (reasons, sink) = recorder();
    window.on_dismissed(move |reason| sink.borrow_mut().push(reason));
    let (restores, restore_sink) = recorder();
    window.on_restore_requested(move || restore_sink.borrow_mut().push(()));
    window.set_state(state(true, false));
    slint::platform::update_timers_and_animations();
    press(&window, Key::Return);
    assert_eq!(
        *reasons.borrow(),
        [CloseReason::CloseButton],
        "Return acts on the focused close button"
    );
    press(&window, Key::Escape);
    assert_eq!(reasons.borrow().last().copied(), Some(CloseReason::Escape));
    window.set_state(state(false, false));
    slint::platform::update_timers_and_animations();
    assert_eq!(
        restores.borrow().len(),
        1,
        "closing asks the host to restore focus once"
    );
}

#[test]
fn footer_actions_report_their_token_and_stop_while_busy() {
    let window = dialog(true);
    let (actions, sink) = recorder();
    window.on_action(move |token| sink.borrow_mut().push(token));
    click(&one(&window, "Save"));
    click(&one(&window, "Discard"));
    assert_eq!(*actions.borrow(), [8, 7]);
    window.set_state(state(true, true));
    slint::platform::update_timers_and_animations();
    click(&one(&window, "Save"));
    assert_eq!(
        actions.borrow().len(),
        2,
        "a busy dialog ignores its actions"
    );
}

#[test]
fn without_a_close_button_opening_focuses_the_first_action() {
    let window = dialog(false);
    window.set_show_close_button(false);
    let (actions, sink) = recorder();
    window.on_action(move |token| sink.borrow_mut().push(token));
    assert!(labelled(&window, "Close dialog").is_empty());
    window.set_state(state(true, false));
    slint::platform::update_timers_and_animations();
    press(&window, Key::Return);
    assert_eq!(*actions.borrow(), [7]);
}

#[test]
fn alert_dialog_answers_explicitly_and_focuses_cancel() {
    let window = alert(false);
    let (confirmed, confirm_sink) = recorder();
    window.on_confirmed(move || confirm_sink.borrow_mut().push(()));
    let (cancelled, cancel_sink) = recorder();
    window.on_cancelled(move || cancel_sink.borrow_mut().push(()));
    let (reasons, reason_sink) = recorder();
    window.on_dismissed(move |reason| reason_sink.borrow_mut().push(reason));
    window.set_state(state(true, false));
    slint::platform::update_timers_and_animations();
    assert!(
        labelled(&window, "Close dialog").is_empty(),
        "no close button"
    );
    press(&window, Key::Return);
    assert_eq!(
        (confirmed.borrow().len(), cancelled.borrow().len()),
        (0, 1),
        "Return acts on Cancel"
    );
    click(&one(&window, "Delete"));
    assert_eq!(confirmed.borrow().len(), 1);
    // A press on the scrim outside the panel does not answer the question.
    let (x, y, _, _) = panel(&window);
    assert!(x > 10. && y > 10.);
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: slint::LogicalPosition::new(5., 5.),
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: slint::LogicalPosition::new(5., 5.),
            button: PointerEventButton::Left,
        });
    assert!(reasons.borrow().is_empty(), "the backdrop does not dismiss");
    press(&window, Key::Escape);
    assert_eq!(*reasons.borrow(), [CloseReason::Escape]);
}

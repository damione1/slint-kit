// SPDX-License-Identifier: GPL-3.0-or-later
use slint::{
    ComponentHandle, Model,
    platform::{Key, WindowEvent},
};
use slint_kit::{
    ToastBinding, ToastCloseReason, ToastLifetime,
    ui::{ToastEntry, ToastInteraction},
};
use std::{cell::RefCell, num::NonZeroUsize, rc::Rc, time::Duration};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Evicted,
    Original,
    Dismissed,
    Expired,
    Cleared,
    Followup,
}
fn entry(title: &str) -> ToastEntry {
    ToastEntry {
        title: title.into(),
        action_label: "Undo".into(),
        dismissible: true,
        ..Default::default()
    }
}
#[test]
fn every_close_reason_delivers_the_original_action_once_after_publication() {
    i_slint_backend_testing::init_no_event_loop();
    let binding = ToastBinding::new(NonZeroUsize::new(2).unwrap());
    let events = Rc::new(RefCell::new(Vec::new()));
    let observed = events.clone();
    let model = binding.model();
    binding.on_closed(move |id, action, reason| {
        assert!(model.row_count() < 3);
        observed.borrow_mut().push((id, action, reason));
    });
    let evicted = binding
        .push(Action::Evicted, entry("Evicted"), ToastLifetime::Persistent)
        .unwrap();
    let original = binding
        .push(
            Action::Original,
            entry("Original"),
            ToastLifetime::Persistent,
        )
        .unwrap();
    let dismissed = binding
        .push(
            Action::Dismissed,
            entry("Dismissed"),
            ToastLifetime::Persistent,
        )
        .unwrap();
    assert_eq!(
        events.borrow().as_slice(),
        [(evicted, Action::Evicted, ToastCloseReason::Evicted)]
    );
    assert_eq!(binding.dismiss_id(dismissed), Some(Action::Dismissed));
    let token = binding.model().row_data(0).unwrap().token;
    let actions = Rc::new(RefCell::new(Vec::new()));
    let recorded = actions.clone();
    let closed = events.clone();
    let activate = binding.action_handler(move |action| {
        assert_eq!(closed.borrow().last().unwrap().2, ToastCloseReason::Action);
        recorded.borrow_mut().push(action);
    });
    activate(token);
    activate(token);
    assert_eq!(*actions.borrow(), [Action::Original]);
    let expired = binding
        .push(
            Action::Expired,
            entry("Expired"),
            ToastLifetime::Timed(Duration::ZERO),
        )
        .unwrap();
    slint::platform::update_timers_and_animations();
    assert!(binding.is_empty());
    let cleared = binding
        .push(Action::Cleared, entry("Cleared"), ToastLifetime::Persistent)
        .unwrap();
    binding.clear();
    binding.clear();
    assert_eq!(binding.len(), 0);
    assert_eq!(
        events.borrow().as_slice(),
        [
            (evicted, Action::Evicted, ToastCloseReason::Evicted),
            (dismissed, Action::Dismissed, ToastCloseReason::Dismissed),
            (original, Action::Original, ToastCloseReason::Action),
            (expired, Action::Expired, ToastCloseReason::Expired),
            (cleared, Action::Cleared, ToastCloseReason::Cleared),
        ]
    );
    assert_eq!(binding.dismiss_id(original), None);
    assert!(!binding.update(expired, entry("No revival"), ToastLifetime::Persistent));
    assert_eq!(events.borrow().len(), 5);
}
#[test]
fn closure_callbacks_can_reenter_without_borrows_or_overwriting_new_toasts() {
    i_slint_backend_testing::init_no_event_loop();
    let binding = ToastBinding::new(NonZeroUsize::new(2).unwrap());
    let slot = Rc::new(RefCell::new(Some(binding.clone())));
    let weak = Rc::downgrade(&slot);
    let events = Rc::new(RefCell::new(Vec::new()));
    let observed = events.clone();
    binding.on_closed(move |_, action, reason| {
        let slot = weak.upgrade().unwrap();
        let binding = slot.borrow().as_ref().unwrap().clone();
        observed.borrow_mut().push((action, reason));
        if action == Action::Original {
            assert!(
                binding.is_empty(),
                "all clear removals are published before callbacks"
            );
            assert_eq!(binding.model().row_count(), 0);
            assert_eq!(slint::platform::duration_until_next_timer_update(), None);
            let id = binding
                .push(
                    Action::Followup,
                    entry("Host followup"),
                    ToastLifetime::Persistent,
                )
                .unwrap();
            assert!(binding.update(id, entry("Host updated"), ToastLifetime::Persistent));
        }
    });
    binding
        .push(
            Action::Original,
            entry("Original"),
            ToastLifetime::Persistent,
        )
        .unwrap();
    binding
        .push(Action::Cleared, entry("Cleared"), ToastLifetime::Persistent)
        .unwrap();
    binding.clear();
    assert_eq!(
        *events.borrow(),
        [
            (Action::Original, ToastCloseReason::Cleared),
            (Action::Cleared, ToastCloseReason::Cleared)
        ]
    );
    assert_eq!(binding.len(), 1);
    assert_eq!(binding.model().row_data(0).unwrap().title, "Host updated");
}
#[test]
fn updating_an_overdue_id_reports_expiry_and_cannot_replace_its_action() {
    i_slint_backend_testing::init_no_event_loop();
    let binding = ToastBinding::new(NonZeroUsize::new(1).unwrap());
    let events = Rc::new(RefCell::new(Vec::new()));
    let observed = events.clone();
    binding.on_closed(move |id, action, reason| observed.borrow_mut().push((id, action, reason)));
    let id = binding
        .push(
            Action::Expired,
            entry("Overdue"),
            ToastLifetime::Timed(Duration::ZERO),
        )
        .unwrap();
    assert!(!binding.update(id, entry("Must not revive"), ToastLifetime::Persistent));
    assert_eq!(
        events.borrow().as_slice(),
        [(id, Action::Expired, ToastCloseReason::Expired)]
    );
    assert_eq!(binding.model().row_count(), 0);
    assert_eq!(slint::platform::duration_until_next_timer_update(), None);
}
#[test]
fn in_place_updates_keep_the_native_toast_delegate_token_focus_and_pauses() {
    use i_slint_backend_testing::{ElementHandle, TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = crate::ToastHostFixture::new().unwrap();
    let binding = ToastBinding::new(NonZeroUsize::new(1).unwrap());
    let id = binding
        .push(
            Action::Original,
            entry("Working"),
            ToastLifetime::Persistent,
        )
        .unwrap();
    let token = binding.model().row_data(0).unwrap().token;
    window.set_entries(binding.model());
    window.on_toast_interaction(binding.interaction_handler());
    let actions = Rc::new(RefCell::new(Vec::new()));
    let recorded = actions.clone();
    window.on_toast_action(move |token| recorded.borrow_mut().push(token));
    window.show().unwrap();
    window.window().take_snapshot().unwrap();
    let button = ElementHandle::find_by_accessible_label(&window, "Undo")
        .next()
        .unwrap();
    button.mock_single_click(slint::platform::PointerEventButton::Left);
    slint::platform::update_timers_and_animations();
    assert_eq!(*actions.borrow(), [token]);
    assert!(binding.update(
        id,
        entry("Saved"),
        ToastLifetime::Timed(Duration::from_secs(60))
    ));
    slint::platform::update_timers_and_animations();
    window.window().take_snapshot().unwrap();
    assert!(button.is_valid());
    assert_eq!(binding.model().row_data(0).unwrap().token, token);
    assert_eq!(binding.model().row_data(0).unwrap().title, "Saved");
    assert_eq!(
        slint::platform::duration_until_next_timer_update(),
        None,
        "pointer and focus pause the new lifetime"
    );
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Space.into(),
    });
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Space.into(),
    });
    assert_eq!(
        *actions.borrow(),
        [token, token],
        "the same native action retains focus"
    );
    let interaction = binding.interaction_handler();
    interaction(token, ToastInteraction::Pointer, false);
    assert_eq!(
        slint::platform::duration_until_next_timer_update(),
        None,
        "focus retains its pause"
    );
    window.invoke_blur_toasts();
    slint::platform::update_timers_and_animations();
    assert!(slint::platform::duration_until_next_timer_update().is_some());
    assert_eq!(binding.dismiss_id(id), Some(Action::Original));
}

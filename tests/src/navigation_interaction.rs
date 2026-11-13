// SPDX-License-Identifier: GPL-3.0-or-later
//! Window tests for pagination, breadcrumbs, menu bar keys, collapse, drawer
//! focus and the toast binding lifecycle behind a real toast host.
use crate::{
    BreadcrumbsFixture, CloseReason, CollapseKeysFixture, DialogState, DrawerFocusFixture,
    MenuBarKeysFixture, NavigationStep, Ownership, PaginationFixture, SelectState,
    ToastHostFixture,
};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use slint_kit::{
    ToastBinding, ToastCloseReason, ToastLifetime,
    ui::{OptionRow, ToastEntry},
};
use std::{cell::RefCell, num::NonZeroUsize, rc::Rc, time::Duration};

fn press<W: ComponentHandle>(window: &W, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    slint::platform::update_timers_and_animations();
}
fn space<W: ComponentHandle>(window: &W) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: " ".into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: " ".into() });
    slint::platform::update_timers_and_animations();
}
fn labelled<W: ComponentHandle>(window: &W, label: &str) -> Vec<ElementHandle> {
    let label = label.to_owned();
    ElementQuery::from_root(window)
        .match_inherits("KitPressable")
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label.as_str()))
        .find_all()
}
fn init() {
    thread_local!(static READY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) });
    if !READY.replace(true) {
        i_slint_backend_testing::init_no_event_loop();
    }
}
fn one<W: ComponentHandle>(window: &W, label: &str) -> ElementHandle {
    let mut found = labelled(window, label);
    assert_eq!(found.len(), 1, "exactly one element labelled {label}");
    found.remove(0)
}
fn click(element: &ElementHandle) {
    element.mock_single_click(PointerEventButton::Left);
}
fn row(token: i32, label: &str, disabled: bool) -> OptionRow {
    OptionRow {
        token,
        generation: 1,
        label: label.into(),
        description: "".into(),
        disabled,
    }
}
type Log<T> = Rc<RefCell<Vec<T>>>;
fn recorder<T: 'static>() -> (Log<T>, Log<T>) {
    let log = Rc::new(RefCell::new(Vec::new()));
    (log.clone(), log)
}

fn pagination(count: i32, index: i32, ownership: Ownership) -> PaginationFixture {
    init();
    let window = PaginationFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_page_count(count);
    window.set_page_index(index);
    window.set_ownership(ownership);
    window.show().unwrap();
    window
}

#[test]
fn pagination_stops_at_both_bounds_and_owns_its_index_locally() {
    let window = pagination(3, 0, Ownership::Local);
    let (changes, sink) = recorder();
    window.on_page_changed(move |page| sink.borrow_mut().push(page));
    let (steps, step_sink) = recorder();
    window.on_navigated(move |step| step_sink.borrow_mut().push(step));
    let previous = one(&window, "Previous page");
    let next = one(&window, "Next page");
    assert_eq!(previous.accessible_enabled(), Some(false), "first page");
    assert_eq!(next.accessible_enabled(), Some(true));
    click(&previous);
    assert!(changes.borrow().is_empty() && window.get_page_index() == 0);
    click(&next);
    click(&next);
    assert_eq!(window.get_page_index(), 2);
    assert_eq!(*changes.borrow(), [1, 2]);
    assert_eq!(next.accessible_enabled(), Some(false), "last page");
    assert_eq!(previous.accessible_enabled(), Some(true));
    click(&next);
    assert_eq!(*changes.borrow(), [1, 2], "no event past the last page");
    click(&previous);
    assert_eq!(window.get_page_index(), 1);
    assert_eq!(*changes.borrow(), [1, 2, 1]);
    assert_eq!(
        *steps.borrow(),
        [
            NavigationStep::Next,
            NavigationStep::Next,
            NavigationStep::Previous
        ]
    );
}

#[test]
fn pagination_with_a_controlled_owner_only_requests_pages() {
    let window = pagination(3, 1, Ownership::Controlled);
    let (changes, sink) = recorder();
    window.on_page_changed(move |page| sink.borrow_mut().push(page));
    click(&one(&window, "Next page"));
    assert_eq!(*changes.borrow(), [2]);
    assert_eq!(window.get_page_index(), 1, "the host owns the index");
    window.set_page_index(2);
    slint::platform::update_timers_and_animations();
    assert_eq!(one(&window, "Next page").accessible_enabled(), Some(false));
    click(&one(&window, "Previous page"));
    assert_eq!(*changes.borrow(), [2, 1]);
    assert_eq!(window.get_page_index(), 2);
}

#[test]
fn pagination_without_pages_disables_both_actions() {
    let window = pagination(0, 0, Ownership::Local);
    let (changes, sink) = recorder();
    window.on_page_changed(move |page| sink.borrow_mut().push(page));
    for label in ["Previous page", "Next page"] {
        let action = one(&window, label);
        assert_eq!(action.accessible_enabled(), Some(false), "{label}");
        click(&action);
    }
    assert!(changes.borrow().is_empty());
    // A stale index beyond the count cannot step forward and steps back inside the range.
    window.set_page_count(3);
    window.set_page_index(9);
    slint::platform::update_timers_and_animations();
    assert_eq!(one(&window, "Next page").accessible_enabled(), Some(false));
    click(&one(&window, "Previous page"));
    assert_eq!(*changes.borrow(), [2], "clamped to the last page");
}

#[test]
fn breadcrumbs_activate_ancestors_only_and_never_the_current_page() {
    init();
    let window = BreadcrumbsFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_rows(ModelRc::new(VecModel::from(vec![
        row(1, "Home", false),
        row(2, "Library", false),
        row(3, "Locked", true),
        row(4, "Current", false),
    ])));
    let (calls, sink) = recorder();
    window.on_activated(move |token, generation| sink.borrow_mut().push((token, generation)));
    window.show().unwrap();
    let pressables = |label: &str| labelled(&window, label);
    assert!(pressables("Current").is_empty(), "the current page is text");
    click(&pressables("Home")[0]);
    click(&pressables("Library")[0]);
    click(&pressables("Locked")[0]);
    assert_eq!(pressables("Locked")[0].accessible_enabled(), Some(false));
    assert_eq!(*calls.borrow(), [(1, 1), (2, 1)]);
    // Removing a trailing row turns the new last row into the current page.
    window.set_rows(ModelRc::new(VecModel::from(vec![
        row(1, "Home", false),
        row(2, "Library", false),
    ])));
    slint::platform::update_timers_and_animations();
    assert!(pressables("Library").is_empty());
    assert_eq!(pressables("Home").len(), 1);
}

fn menu_bar(active: usize) -> MenuBarKeysFixture {
    init();
    let window = MenuBarKeysFixture::new().unwrap();
    slint_kit::install(&window);
    let rows: Vec<OptionRow> = (0..6)
        .map(|i| row(10 + i, &format!("Menu item {}", i + 1), false))
        .collect();
    window.set_state(SelectState {
        open: true,
        generation: 1,
        active_index: active as i32,
        active: slint_kit::ui::SelectionRef {
            present: true,
            token: 10 + active as i32,
        },
        ..Default::default()
    });
    window.set_rows(ModelRc::new(VecModel::from(rows)));
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}

#[test]
fn menu_bar_arrows_home_end_and_enter_report_steps_and_the_active_row() {
    let window = menu_bar(0);
    let (steps, sink) = recorder();
    window.on_navigated(move |step| sink.borrow_mut().push(step));
    let (picked, picked_sink) = recorder();
    window
        .on_activated(move |token, generation| picked_sink.borrow_mut().push((token, generation)));
    window.invoke_focus_bar();
    assert!(window.get_bar_focused());
    for key in [Key::RightArrow, Key::LeftArrow, Key::Home, Key::End] {
        press(&window, key);
    }
    assert_eq!(
        *steps.borrow(),
        [
            NavigationStep::Next,
            NavigationStep::Previous,
            NavigationStep::First,
            NavigationStep::Last
        ]
    );
    press(&window, Key::Return);
    assert_eq!(
        *picked.borrow(),
        [(10, 1)],
        "Enter activates the active row"
    );
}

#[test]
fn menu_bar_reveals_an_off_screen_active_row() {
    let window = menu_bar(0);
    let window_width = 220.0;
    let right = |element: &ElementHandle| element.absolute_position().x + element.size().width;
    assert!(
        labelled(&window, "Menu item 6").is_empty(),
        "the last row starts beyond the clipped frame"
    );
    let mut state = window.get_state();
    state.active_index = 5;
    state.active = slint_kit::ui::SelectionRef {
        present: true,
        token: 15,
    };
    window.set_state(state);
    slint::platform::update_timers_and_animations();
    assert!(
        right(&one(&window, "Menu item 6")) <= window_width + 1.0,
        "the active row is scrolled into view"
    );
    let mut state = window.get_state();
    state.active_index = 0;
    state.active = slint_kit::ui::SelectionRef {
        present: true,
        token: 10,
    };
    window.set_state(state);
    slint::platform::update_timers_and_animations();
    assert!(one(&window, "Menu item 1").absolute_position().x >= -1.0);
}

fn collapse(expanded: bool, ownership: Ownership) -> CollapseKeysFixture {
    init();
    let window = CollapseKeysFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_expanded(expanded);
    window.set_ownership(ownership);
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window.invoke_focus_trigger();
    window
}

#[test]
fn collapse_toggles_from_the_keyboard_when_it_owns_its_state() {
    let window = collapse(false, Ownership::Local);
    let (requests, sink) = recorder();
    window.on_expansion_requested(move |next| sink.borrow_mut().push(next));
    space(&window);
    assert!(window.get_expanded());
    press(&window, Key::Return);
    assert!(!window.get_expanded());
    assert_eq!(*requests.borrow(), [true, false]);
}

#[test]
fn collapse_with_a_controlled_owner_only_requests_a_change() {
    let window = collapse(false, Ownership::Controlled);
    let (requests, sink) = recorder();
    window.on_expansion_requested(move |next| sink.borrow_mut().push(next));
    space(&window);
    assert!(!window.get_expanded(), "the host owns the state");
    assert_eq!(*requests.borrow(), [true]);
}

#[test]
fn collapse_keeps_hidden_content_out_of_the_tab_order() {
    let hidden = collapse(false, Ownership::Local);
    press(&hidden, Key::Tab);
    assert!(hidden.get_outside_focused(), "Tab skips collapsed content");
    assert!(!hidden.get_inner_focused());

    let shown = collapse(true, Ownership::Local);
    press(&shown, Key::Tab);
    assert!(shown.get_inner_focused(), "Tab enters expanded content");
    press(&shown, Key::Tab);
    assert!(shown.get_outside_focused());
}

fn drawer(state: DialogState) -> DrawerFocusFixture {
    init();
    let window = DrawerFocusFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_state(state);
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}
fn state(open: bool, busy: bool) -> DialogState {
    DialogState {
        open,
        topmost: true,
        busy,
    }
}

#[test]
fn drawer_moves_focus_to_close_on_open_and_asks_the_host_to_restore_it() {
    let window = drawer(state(false, false));
    let (restores, sink) = recorder();
    window.on_restore_requested(move || sink.borrow_mut().push(()));
    let (reasons, reason_sink) = recorder();
    window.on_dismissed(move |reason| reason_sink.borrow_mut().push(reason));
    assert!(!window.get_close_focused());
    window.set_state(state(true, false));
    slint::platform::update_timers_and_animations();
    assert!(
        window.get_close_focused(),
        "opening focuses the close button"
    );
    assert!(restores.borrow().is_empty());
    press(&window, Key::Escape);
    assert_eq!(*reasons.borrow(), [CloseReason::Escape]);
    window.set_state(state(false, false));
    slint::platform::update_timers_and_animations();
    assert_eq!(
        restores.borrow().len(),
        1,
        "closing restores host focus once"
    );
    window.set_state(state(true, false));
    slint::platform::update_timers_and_animations();
    click(&one(&window, "Close panel"));
    assert_eq!(
        reasons.borrow().last().copied(),
        Some(CloseReason::Programmatic)
    );
}

#[test]
fn drawer_tab_is_forwarded_to_the_host_in_both_directions() {
    let window = drawer(state(true, false));
    let (moves, sink) = recorder();
    window.on_focus_next_requested(move |backwards| sink.borrow_mut().push(backwards));
    press(&window, Key::Tab);
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    press(&window, Key::Backtab);
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    assert_eq!(*moves.borrow(), [false, true]);
}

#[test]
fn a_busy_drawer_disables_close_and_swallows_dismissal_keys() {
    let window = drawer(state(true, true));
    let (reasons, sink) = recorder();
    window.on_dismissed(move |reason| sink.borrow_mut().push(reason));
    let (moves, move_sink) = recorder();
    window.on_focus_next_requested(move |backwards| move_sink.borrow_mut().push(backwards));
    let close = one(&window, "Close panel");
    assert_eq!(close.accessible_enabled(), Some(false));
    click(&close);
    press(&window, Key::Escape);
    assert!(reasons.borrow().is_empty(), "busy blocks every dismissal");
    press(&window, Key::Tab);
    assert_eq!(*moves.borrow(), [false], "keys stay inside the modal");
    window.set_state(state(true, false));
    slint::platform::update_timers_and_animations();
    assert_eq!(one(&window, "Close panel").accessible_enabled(), Some(true));
    press(&window, Key::Escape);
    assert_eq!(*reasons.borrow(), [CloseReason::Escape]);
}

fn entry(title: &str) -> ToastEntry {
    ToastEntry {
        title: title.into(),
        action_label: "Undo".into(),
        dismissible: true,
        ..Default::default()
    }
}
fn mock_time_host() -> ToastHostFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    ToastHostFixture::new().unwrap()
}
fn connect(
    window: &ToastHostFixture,
    binding: &ToastBinding<&'static str>,
) -> Rc<RefCell<Vec<i32>>> {
    window.set_entries(binding.model());
    window.on_toast_interaction(binding.interaction_handler());
    let (actions, sink) = recorder();
    window.on_toast_action(move |token| sink.borrow_mut().push(token));
    window.show().unwrap();
    window.window().take_snapshot().unwrap();
    actions
}

#[test]
fn a_second_toast_keeps_the_first_delegate_focus_and_pause_until_eviction_and_expiry() {
    let window = mock_time_host();
    let binding: ToastBinding<&'static str> = ToastBinding::new(NonZeroUsize::new(2).unwrap());
    let (closed, sink) = recorder();
    binding.on_closed(move |id, action, reason| sink.borrow_mut().push((id, action, reason)));
    let first = binding
        .push(
            "first",
            entry("First"),
            ToastLifetime::Timed(Duration::from_secs(60)),
        )
        .unwrap();
    let first_token = binding.model().row_data(0).unwrap().token;
    let actions = connect(&window, &binding);
    let button = ElementHandle::find_by_accessible_label(&window, "Undo")
        .next()
        .unwrap();
    click(&button);
    slint::platform::update_timers_and_animations();
    assert_eq!(*actions.borrow(), [first_token]);
    assert_eq!(slint::platform::duration_until_next_timer_update(), None);

    let second = binding
        .push(
            "second",
            entry("Second"),
            ToastLifetime::Timed(Duration::from_secs(60)),
        )
        .unwrap();
    slint::platform::update_timers_and_animations();
    window.window().take_snapshot().unwrap();
    assert!(button.is_valid(), "the first delegate survives the push");
    assert_eq!(binding.model().row_data(0).unwrap().token, first_token);
    assert_ne!(binding.model().row_data(1).unwrap().token, first_token);
    space(&window);
    assert_eq!(*actions.borrow(), [first_token, first_token], "focus kept");
    assert!(
        slint::platform::duration_until_next_timer_update()
            .is_some_and(|delay| delay > Duration::from_secs(30)),
        "only the second toast runs its lifetime"
    );

    let third = binding
        .push(
            "third",
            entry("Third"),
            ToastLifetime::Timed(Duration::ZERO),
        )
        .unwrap();
    assert_eq!(
        *closed.borrow(),
        [(first, "first", ToastCloseReason::Evicted)],
        "capacity evicts the oldest toast"
    );
    slint::platform::update_timers_and_animations();
    window.window().take_snapshot().unwrap();
    space(&window);
    assert_eq!(actions.borrow().len(), 2, "an evicted toast gets no key");
    assert_eq!(binding.len(), 1, "the zero lifetime expired on the update");
    assert_eq!(
        closed.borrow().last(),
        Some(&(third, "third", ToastCloseReason::Expired))
    );
    assert_eq!(binding.dismiss_id(second), Some("second"));
    assert_eq!(slint::platform::duration_until_next_timer_update(), None);
}

#[test]
fn a_capacity_of_one_replaces_the_toast_with_a_fresh_identity() {
    let window = mock_time_host();
    let binding: ToastBinding<&'static str> = ToastBinding::new(NonZeroUsize::new(1).unwrap());
    let (closed, sink) = recorder();
    binding.on_closed(move |id, action, reason| sink.borrow_mut().push((id, action, reason)));
    let first = binding
        .push("first", entry("First"), ToastLifetime::Persistent)
        .unwrap();
    let first_token = binding.model().row_data(0).unwrap().token;
    let actions = connect(&window, &binding);
    let second = binding
        .push("second", entry("Second"), ToastLifetime::Persistent)
        .unwrap();
    slint::platform::update_timers_and_animations();
    window.window().take_snapshot().unwrap();
    assert_ne!(first, second);
    assert_eq!(binding.len(), 1);
    let token = binding.model().row_data(0).unwrap().token;
    assert_ne!(token, first_token, "a replaced toast never reuses a token");
    assert_eq!(
        *closed.borrow(),
        [(first, "first", ToastCloseReason::Evicted)]
    );
    click(&one(&window, "Undo"));
    assert_eq!(
        *actions.borrow(),
        [token],
        "the action targets the new toast"
    );
    assert_eq!(binding.dismiss_id(first), None, "the evicted id is stale");
    assert_eq!(binding.dismiss_id(second), Some("second"));
}

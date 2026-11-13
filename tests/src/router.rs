// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{RouterFixture, Screen};
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use kit_core::RouteChange;
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use slint_kit::{Navigation, Router};
use std::{cell::RefCell, rc::Rc, time::Duration};

/// The application's routes: one parameter travels with its screen.
#[derive(Clone, Debug, PartialEq, Eq)]
enum AppRoute {
    List,
    Detail { id: u32 },
    Edit { id: u32 },
}

/// The single conversion from the route type to the Slint enum. No wildcard:
/// a new `AppRoute` variant stops compiling here.
fn screen(route: &AppRoute) -> Screen {
    match route {
        AppRoute::List => Screen::List,
        AppRoute::Detail { .. } => Screen::Detail,
        AppRoute::Edit { .. } => Screen::Edit,
    }
}

fn project(window: &RouterFixture, navigation: &Navigation<AppRoute>) {
    window.set_change(navigation.change().into());
    window.set_route(screen(navigation.route()));
    window.set_can_go_back(navigation.can_go_back());
}

fn advance(milliseconds: u64) {
    i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(milliseconds));
    slint::platform::update_timers_and_animations();
}

fn settle() {
    advance(400);
}

fn key(window: &RouterFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}

fn element(window: &RouterFixture, label: &str) -> ElementHandle {
    let label = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(label.as_str())
        })
        .find_first()
        .expect("accessible element")
}

struct App {
    window: RouterFixture,
    router: Router<AppRoute>,
    _projection: slint_kit::Subscription<Navigation<AppRoute>>,
}

/// A window whose router is wired as an application would: one projection,
/// and `back` calls `pop`.
fn app() -> App {
    i_slint_backend_testing::init_no_event_loop();
    let window = RouterFixture::new().unwrap();
    window.show().unwrap();
    let router = Router::new(AppRoute::List);
    let weak = window.as_weak();
    let projection = router.observe(move |navigation| {
        if let Some(window) = weak.upgrade() {
            project(&window, navigation);
        }
    });
    let back = router.clone();
    window.on_back(move || {
        back.pop();
    });
    // A rendered window has read every screen; animations need the old value.
    let _ = (visible(&window), window.get_detail_x(), window.get_list_x());
    App {
        window,
        router,
        _projection: projection,
    }
}

fn visible(window: &RouterFixture) -> [bool; 3] {
    [
        window.get_list_visible(),
        window.get_detail_visible(),
        window.get_edit_visible(),
    ]
}

#[test]
fn three_screens_navigate_forward_and_back_without_per_screen_conditions() {
    let app = app();
    assert_eq!(visible(&app.window), [true, false, false]);
    app.router.push(AppRoute::Detail { id: 7 });
    settle();
    assert_eq!(visible(&app.window), [false, true, false]);
    app.router.push(AppRoute::Edit { id: 7 });
    settle();
    assert_eq!(visible(&app.window), [false, false, true]);
    assert!(app.router.pop());
    settle();
    assert_eq!(visible(&app.window), [false, true, false]);
    assert!(app.router.pop());
    settle();
    assert_eq!(visible(&app.window), [true, false, false]);
    assert!(!app.router.pop(), "the root stays");
    assert_eq!(app.router.get().depth(), 1);
}

#[test]
fn route_parameters_travel_with_the_route_and_replace_keeps_the_history() {
    let app = app();
    app.router.push(AppRoute::Detail { id: 7 });
    app.router.replace(AppRoute::Detail { id: 8 });
    assert_eq!(app.router.current(), AppRoute::Detail { id: 8 });
    assert_eq!(app.router.get().history(), [AppRoute::List]);
    assert_eq!(app.router.get().change(), RouteChange::Replace);
    app.router.pop();
    assert_eq!(app.router.current(), AppRoute::List);
}

#[test]
fn navigated_reports_each_change_but_not_the_current_route_or_a_pop_at_the_root() {
    let router = Router::new(AppRoute::List);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();
    let subscription = router.on_navigated(move |route, change| {
        sink.borrow_mut().push((route.clone(), change));
    });
    assert!(!router.pop());
    router.push(AppRoute::Detail { id: 1 });
    router.replace(AppRoute::Edit { id: 1 });
    router.pop();
    assert_eq!(
        *seen.borrow(),
        [
            (AppRoute::Detail { id: 1 }, RouteChange::Push),
            (AppRoute::Edit { id: 1 }, RouteChange::Replace),
            (AppRoute::List, RouteChange::Pop),
        ]
    );
    drop(subscription);
    router.push(AppRoute::Detail { id: 2 });
    assert_eq!(seen.borrow().len(), 3);
}

#[test]
fn escape_and_system_back_pop_from_inside_a_screen_and_are_left_alone_at_the_root() {
    let app = app();
    element(&app.window, "Open detail").invoke_accessible_default_action();
    key(&app.window, Key::Escape);
    assert_eq!(app.router.get().depth(), 1, "root Escape does nothing");
    app.router.push(AppRoute::Detail { id: 1 });
    settle();
    key(&app.window, Key::Escape);
    settle();
    assert_eq!(app.router.current(), AppRoute::List);
    assert_eq!(visible(&app.window), [true, false, false]);
    app.router.push(AppRoute::Detail { id: 1 });
    app.router.push(AppRoute::Edit { id: 1 });
    settle();
    key(&app.window, Key::Back);
    settle();
    assert_eq!(app.router.current(), AppRoute::Detail { id: 1 });
}

#[test]
fn an_open_modal_layer_keeps_the_router_from_handling_back() {
    let app = app();
    app.router.push(AppRoute::Detail { id: 1 });
    settle();
    app.window.invoke_open_modal_layer();
    key(&app.window, Key::Escape);
    assert_eq!(app.router.current(), AppRoute::Detail { id: 1 });
}

#[test]
fn a_push_slides_the_new_screen_in_from_the_trailing_edge_and_settles_at_rest() {
    let app = app();
    app.router.push(AppRoute::Detail { id: 1 });
    let _ = app.window.get_detail_x();
    advance(60);
    assert!(app.window.get_detail_visible() && app.window.get_list_visible());
    assert!(app.window.get_detail_x() > 0.0);
    assert!(app.window.get_list_x() < 0.0);
    settle();
    assert_eq!(app.window.get_detail_x(), 0.0);
    assert_eq!(app.window.get_detail_opacity(), 1.0);
    app.router.pop();
    let _ = app.window.get_detail_x();
    advance(60);
    assert!(
        app.window.get_list_x() < 0.0,
        "a pop enters from the leading edge"
    );
    assert!(
        app.window.get_detail_x() > 0.0,
        "and the left screen exits trailing"
    );
}

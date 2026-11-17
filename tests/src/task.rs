// SPDX-License-Identifier: GPL-3.0-or-later
use crate::TaskFixture;
use slint_kit::{Latest, Loading, SpawnError, spawn};
use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

/// A future completed by hand, so a test chooses the order of the responses.
struct Gate<T> {
    shared: Rc<RefCell<(Option<T>, Option<Waker>)>>,
}

struct Release<T> {
    shared: Rc<RefCell<(Option<T>, Option<Waker>)>>,
}

fn gate<T>() -> (Release<T>, Gate<T>) {
    let shared = Rc::new(RefCell::new((None, None)));
    (
        Release {
            shared: shared.clone(),
        },
        Gate { shared },
    )
}

impl<T> Release<T> {
    fn resolve(self, value: T) {
        let waker = {
            let mut shared = self.shared.borrow_mut();
            shared.0 = Some(value);
            shared.1.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

impl<T> Future for Gate<T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut shared = self.shared.borrow_mut();
        match shared.0.take() {
            Some(value) => Poll::Ready(value),
            None => {
                shared.1 = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

/// Run the event loop until every event queued so far has been handled.
fn settle() {
    slint::invoke_from_event_loop(|| slint::quit_event_loop().unwrap()).unwrap();
    slint::run_event_loop().unwrap();
}

fn window() -> TaskFixture {
    TaskFixture::new().unwrap()
}

pub(crate) fn the_result_reaches_the_window_without_a_weak_reference_in_application_code() {
    let app = window();
    let (release, response) = gate::<Vec<&str>>();
    spawn(&app, response, |app, rows| {
        app.set_rows(slint::ModelRc::new(slint::VecModel::from(
            rows.into_iter()
                .map(Into::into)
                .collect::<Vec<slint::SharedString>>(),
        )));
        app.set_status("loaded".into());
    })
    .unwrap();
    release.resolve(vec!["a", "b"]);
    settle();
    assert_eq!(app.get_status(), "loaded");
    assert_eq!(slint::Model::row_count(&app.get_rows()), 2);
}

pub(crate) fn a_response_arriving_after_a_newer_request_is_discarded() {
    let app = window();
    let latest = Latest::new();
    let (first_release, first) = gate::<&str>();
    let (second_release, second) = gate::<&str>();
    for (name, response) in [("first", first), ("second", second)] {
        latest
            .spawn(&app, response, move |app, body| {
                app.set_status(format!("{name}:{body}").into());
            })
            .unwrap();
    }
    second_release.resolve("fresh");
    first_release.resolve("slow");
    settle();
    assert_eq!(app.get_status(), "second:fresh");
}

pub(crate) fn the_older_response_is_discarded_even_when_it_arrives_first() {
    let app = window();
    let latest = Latest::new();
    let applied = Rc::new(RefCell::new(Vec::new()));
    let (first_release, first) = gate::<u8>();
    let (second_release, second) = gate::<u8>();
    for response in [first, second] {
        let sink = applied.clone();
        latest
            .spawn(&app, response, move |_, value| {
                sink.borrow_mut().push(value)
            })
            .unwrap();
    }
    first_release.resolve(1);
    settle();
    assert!(applied.borrow().is_empty());
    second_release.resolve(2);
    settle();
    assert_eq!(*applied.borrow(), [2]);
}

pub(crate) fn load_reports_loading_then_the_outcome_and_a_stale_load_reports_nothing() {
    let app = window();
    let latest = Latest::new();
    let seen: Rc<RefCell<Vec<Loading<u8, &str>>>> = Rc::default();
    let (first_release, first) = gate::<Result<u8, &str>>();
    let (second_release, second) = gate::<Result<u8, &str>>();
    for response in [first, second] {
        let sink = seen.clone();
        latest
            .load(&app, response, move |_, state| {
                sink.borrow_mut().push(state)
            })
            .unwrap();
    }
    assert_eq!(*seen.borrow(), [Loading::Loading, Loading::Loading]);
    first_release.resolve(Ok(1));
    second_release.resolve(Err("offline"));
    settle();
    assert_eq!(
        *seen.borrow(),
        [
            Loading::Loading,
            Loading::Loading,
            Loading::Failed("offline")
        ]
    );
}

pub(crate) fn cancel_discards_the_request_in_flight() {
    let app = window();
    let latest = Latest::new();
    let applied = Rc::new(Cell::new(false));
    let (release, response) = gate::<u8>();
    let flag = applied.clone();
    latest
        .spawn(&app, response, move |_, _| flag.set(true))
        .unwrap();
    latest.cancel();
    release.resolve(1);
    settle();
    assert!(!applied.get());
}

pub(crate) fn a_destroyed_window_is_never_applied_to() {
    let app = window();
    let applied = Rc::new(Cell::new(false));
    let (release, response) = gate::<u8>();
    let flag = applied.clone();
    spawn(&app, response, move |_, _| flag.set(true)).unwrap();
    drop(app);
    release.resolve(1);
    settle();
    assert!(!applied.get());
}

#[test]
fn spawn_reports_a_missing_event_loop() {
    let outcome = std::thread::spawn(move || {
        i_slint_backend_testing::init_no_event_loop();
        let app = TaskFixture::new().unwrap();
        spawn(&app, async {}, |_, ()| {}).map_err(|error| match error {
            SpawnError::EventLoopUnavailable(_) => (),
        })
    })
    .join()
    .unwrap();
    assert!(outcome.is_err());
}

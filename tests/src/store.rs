// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{CounterState, StoreFixture};
use slint::ComponentHandle;
use slint_kit::Store;
use std::{cell::RefCell, rc::Rc, thread};

#[derive(Clone, Debug, PartialEq)]
struct Counter {
    count: i32,
    label: String,
}

fn project(window: &StoreFixture, value: &Counter) {
    let state = window.global::<CounterState>();
    state.set_count(value.count);
    state.set_label(value.label.as_str().into());
}

fn counter() -> Counter {
    Counter {
        count: 0,
        label: "idle".into(),
    }
}

#[test]
fn a_rust_write_reaches_both_components_reading_the_global() {
    i_slint_backend_testing::init_no_event_loop();
    let window = StoreFixture::new().unwrap();
    let store = Store::new(counter());
    let weak = window.as_weak();
    let _binding = store.observe(move |value| {
        if let Some(window) = weak.upgrade() {
            project(&window, value);
        }
    });
    assert_eq!(window.get_first_label(), "idle");
    store.update(|value| {
        value.count = 7;
        value.label = "busy".into();
    });
    assert_eq!(window.get_first_count(), 7);
    assert_eq!(window.get_second_count(), 7);
    assert_eq!(window.get_first_label(), "busy");
    store.set(Counter {
        count: 9,
        label: "done".into(),
    });
    assert_eq!(window.get_second_count(), 9);
    assert_eq!(store.get().label, "done");
    assert_eq!(store.with(|value| value.count), 9);
}

#[test]
fn one_store_drives_the_globals_of_several_windows() {
    i_slint_backend_testing::init_no_event_loop();
    let windows = [StoreFixture::new().unwrap(), StoreFixture::new().unwrap()];
    let store = Store::new(counter());
    let _bindings: Vec<_> = windows
        .iter()
        .map(|window| {
            let weak = window.as_weak();
            store.observe(move |value| {
                if let Some(window) = weak.upgrade() {
                    project(&window, value);
                }
            })
        })
        .collect();
    store.update(|value| value.count = 3);
    assert!(windows.iter().all(|window| window.get_first_count() == 3));
}

#[test]
fn subscribe_skips_the_current_value_and_dropping_stops_calls() {
    let store = Store::new(1_i32);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();
    let subscription = store.subscribe(move |value| sink.borrow_mut().push(*value));
    assert!(seen.borrow().is_empty());
    store.set(2);
    drop(subscription);
    store.set(3);
    assert_eq!(*seen.borrow(), [2]);
    let clone = store.clone();
    clone.set(4);
    assert_eq!(store.get(), 4);
}

#[test]
fn a_change_made_by_an_observer_leaves_every_observer_on_the_latest_value() {
    let store = Store::new(0_i32);
    let clamp = store.clone();
    let _clamp = store.subscribe(move |value| {
        if *value > 10 {
            clamp.set(10);
        }
    });
    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();
    let _last = store.subscribe(move |value| sink.borrow_mut().push(*value));
    store.set(50);
    assert_eq!(store.get(), 10);
    assert_eq!(seen.borrow().last(), Some(&10));
}

pub(crate) fn another_thread_updates_the_store_through_the_event_loop_and_dropped_stores_are_skipped()
 {
    let window = StoreFixture::new().unwrap();
    let store = Store::new(counter());
    let weak = window.as_weak();
    let _binding = store.observe(move |value| {
        if let Some(window) = weak.upgrade() {
            project(&window, value);
        }
    });
    let remote = store.remote();
    let dropped = Store::new(1_i32);
    let stale = dropped.remote();
    drop(dropped);
    let worker = thread::spawn(move || {
        stale.set(5).unwrap();
        remote.update(|value| value.count += 1).unwrap();
        remote
            .set(Counter {
                count: 42,
                label: "loaded".into(),
            })
            .unwrap();
        slint::invoke_from_event_loop(|| slint::quit_event_loop().unwrap()).unwrap();
    });
    slint::run_event_loop().unwrap();
    worker.join().unwrap();
    assert_eq!(window.get_second_count(), 42);
    assert_eq!(window.get_first_label(), "loaded");
    assert_eq!(store.get().count, 42);
}

// SPDX-License-Identifier: GPL-3.0-or-later
use crate::Fixture;
use slint::ComponentHandle;
use std::{cell::Cell, rc::Rc};

#[test]
fn application_callbacks_forward_values_without_owning_the_window() {
    i_slint_backend_testing::init_no_event_loop();
    let window = Fixture::new().unwrap();
    let weak = window.as_weak();
    let count = Rc::new(Cell::new(0));
    let observed = count.clone();
    let action = slint_kit::action(&window, move |_| observed.set(observed.get() + 1));
    let observed = count.clone();
    let event = slint_kit::event(&window, move |_, value| observed.set(value));
    let observed = count.clone();
    let event2 = slint_kit::event2(&window, move |_, first, second| {
        observed.set(first + second)
    });
    let observed = count.clone();
    let event3 = slint_kit::event3(&window, move |_, a, b, c| observed.set(a + b + c));
    event3(1, 2, 3);
    assert_eq!(count.get(), 6);
    count.set(0);
    action();
    assert_eq!(count.get(), 1);
    event(8);
    assert_eq!(count.get(), 8);
    event2(20, 22);
    assert_eq!(count.get(), 42);
    drop(window);
    assert!(
        weak.upgrade().is_none(),
        "callbacks must not retain the window"
    );
    action();
    event(100);
    event2(100, 200);
    event3(100, 200, 300);
    assert_eq!(count.get(), 42, "destroyed windows ignore late callbacks");
}

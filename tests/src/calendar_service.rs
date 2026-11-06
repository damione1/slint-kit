// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{CalendarServiceFixture, CivilDateView, FieldAccess};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
fn key(window: &CalendarServiceFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
#[test]
fn calendars_own_independent_state_and_reuse_library_types() {
    i_slint_backend_testing::init_no_event_loop();
    let window = CalendarServiceFixture::new().unwrap();
    slint_kit::install(&window);
    slint_kit::install(&window);
    window.show().unwrap();
    window.invoke_focus_first();
    key(&window, Key::RightArrow);
    let shared: slint_kit::ui::CivilDateView = window.get_first_focused();
    assert_eq!(
        shared,
        CivilDateView {
            year: 2024,
            month: 2,
            day: 29
        }
    );
    key(&window, Key::Return);
    assert_eq!(window.get_first_selected(), shared);
    assert_eq!(
        window.get_second_selected(),
        CivilDateView {
            year: 2026,
            month: 12,
            day: 31
        }
    );
    window.invoke_focus_second();
    key(&window, Key::RightArrow);
    assert_eq!(
        window.get_second_focused(),
        CivilDateView {
            year: 2027,
            month: 1,
            day: 1
        }
    );
    assert_eq!(window.get_first_focused(), shared);
    let another = CalendarServiceFixture::new().unwrap();
    slint_kit::install(&another);
    assert_eq!(
        another.get_first_focused(),
        CivilDateView {
            year: 2024,
            month: 2,
            day: 28
        }
    );
}
#[test]
fn read_only_browses_and_disabled_does_not_change_the_date() {
    i_slint_backend_testing::init_no_event_loop();
    let window = CalendarServiceFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    window.set_access(FieldAccess::ReadOnly);
    window.invoke_focus_first();
    key(&window, Key::RightArrow);
    key(&window, Key::Return);
    assert_eq!(window.get_first_selected().day, 28);
    assert_eq!(window.get_first_focused().day, 29);
    window.set_access(FieldAccess::Disabled);
    key(&window, Key::RightArrow);
    assert_eq!(window.get_first_focused().day, 29);
}

#[test]
fn controlled_calendar_keeps_selection_until_host_acknowledges() {
    use std::{cell::RefCell, rc::Rc};
    i_slint_backend_testing::init_no_event_loop();
    let window = CalendarServiceFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_ownership(slint_kit::ui::Ownership::Controlled);
    let chosen = Rc::new(RefCell::new(None));
    let observed = chosen.clone();
    window.on_first_chosen(move |date| *observed.borrow_mut() = Some(date));
    window.show().unwrap();
    window.invoke_focus_first();
    key(&window, Key::RightArrow);
    key(&window, Key::Return);
    assert_eq!(window.get_first_selected().day, 28);
    assert_eq!(window.get_first_focused().day, 29);
    assert_eq!(chosen.borrow().as_ref().unwrap().day, 29);
    window.invoke_acknowledge_first(chosen.borrow().clone().unwrap());
    assert_eq!(window.get_first_selected().day, 29);
}

#[test]
fn picker_range_and_datetime_controlled_proposals_require_acknowledgement() {
    use std::{cell::RefCell, rc::Rc};
    i_slint_backend_testing::init_no_event_loop();
    let window = crate::ControlledTemporalFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    let date = Rc::new(RefCell::new(None));
    let output = date.clone();
    window.on_date_proposed(move |value| *output.borrow_mut() = Some(value));
    window.invoke_propose_date(CivilDateView {
        year: 2026,
        month: 10,
        day: 5,
    });
    assert_eq!(window.get_picked().day, 1);
    window.invoke_acknowledge_date(date.borrow().clone().unwrap());
    assert_eq!(window.get_picked().day, 5);
    let range = Rc::new(RefCell::new(None));
    let output = range.clone();
    window.on_range_proposed(move |value| *output.borrow_mut() = Some(value));
    window.invoke_focus_range();
    let key = |key: Key| {
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    };
    for _ in 0..4 {
        key(Key::RightArrow);
    }
    key(Key::Return);
    key(Key::RightArrow);
    key(Key::Return);
    window.invoke_apply_range();
    assert_eq!(window.get_range_value().start.day, 1);
    let proposed = range.borrow().clone().expect("complete range proposed");
    assert_eq!(proposed.start.day, 5);
    assert_eq!(proposed.end.day, 6);
    window.invoke_acknowledge_range(proposed);
    assert_eq!(window.get_range_value().start.day, 5);
    let local = Rc::new(RefCell::new(None));
    let output = local.clone();
    window.on_local_proposed(move |value| *output.borrow_mut() = Some(value));
    window.invoke_propose_local();
    assert_eq!(window.get_local_value().date.day, 1);
    window.invoke_acknowledge_local(local.borrow().clone().unwrap());
    assert_eq!(window.get_local_value().date.day, 5);
}

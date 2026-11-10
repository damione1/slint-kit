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

#[test]
fn stored_dates_outside_new_bounds_or_availability_do_not_block_replacement() {
    use crate::CalendarConstraintFixture;
    use i_slint_backend_testing::{ElementHandle, ElementQuery};
    i_slint_backend_testing::init_no_event_loop();
    for bounds in [true, false] {
        let window = CalendarConstraintFixture::new().unwrap();
        slint_kit::install(&window);
        window.set_selected(CivilDateView {
            year: 2026,
            month: 10,
            day: 1,
        });
        if bounds {
            window.set_min("2026-10-05".into());
            window.set_max("2026-10-31".into());
        } else {
            window.set_unavailable_weekdays(slint::ModelRc::new(slint::VecModel::from(vec![
                slint_kit::ui::WeekStart::Thursday,
            ])));
        }
        window.show().unwrap();
        slint::platform::update_timers_and_animations();
        assert_eq!(
            ElementQuery::from_root(&window)
                .match_inherits("CalendarOption")
                .find_all()
                .len(),
            42,
            "invalid stored selection does not blank the grid"
        );
        assert_eq!(
            window.get_selected().day,
            1,
            "the host value is preserved until a replacement is chosen"
        );
        let old = ElementHandle::find_by_accessible_label(&window, "2026-10-01")
            .next()
            .unwrap();
        old.invoke_accessible_default_action();
        assert_eq!(
            window.get_selected().day,
            1,
            "an invalid stored day is still unavailable"
        );
        ElementHandle::find_by_accessible_label(&window, "2026-10-06")
            .next()
            .unwrap()
            .mock_single_click(slint::platform::PointerEventButton::Left);
        assert_eq!(window.get_selected().day, 6);
    }
}

#[test]
fn an_invalid_stored_range_can_be_replaced_even_after_cancel_restores_it() {
    use crate::StoredRangeFixture;
    i_slint_backend_testing::init_no_event_loop();
    let window = StoredRangeFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    window.invoke_cancel();
    window.invoke_take_focus();
    slint::platform::update_timers_and_animations();
    let key = |key: Key| {
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
        slint::platform::update_timers_and_animations();
    };
    key(Key::Return);
    key(Key::RightArrow);
    key(Key::Return);
    assert_eq!(
        window.get_selected().start.day,
        1,
        "draft selection is provisional"
    );
    window.invoke_apply();
    assert_eq!(window.get_selected().start.day, 5);
    assert_eq!(window.get_selected().end.day, 6);
}

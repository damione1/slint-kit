// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::ElementQuery;
use slint::{
    ComponentHandle,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{cell::RefCell, rc::Rc};
fn day(enabled: bool) -> CalendarDay {
    CalendarDay {
        year: 2026,
        month: 10,
        day: 19,
        label: "19".into(),
        accessible_label: "Lundi 19 octobre 2026".into(),
        enabled,
        selected: false,
        in_range: false,
        today: false,
    }
}
fn key(window: &CalendarQaFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
#[test]
fn unavailable_dates_and_read_only_never_choose_but_keyboard_browsing_remains_available() {
    i_slint_backend_testing::init_no_event_loop();
    let window = CalendarQaFixture::new().unwrap();
    window.set_days(slint::ModelRc::new(slint::VecModel::from(vec![day(false)])));
    let choices = Rc::new(RefCell::new(Vec::new()));
    let chosen = choices.clone();
    window.on_choose(move |day| chosen.borrow_mut().push(day));
    let moves = Rc::new(RefCell::new(Vec::new()));
    let observed = moves.clone();
    window.on_navigate(move |step| observed.borrow_mut().push(step));
    window.show().unwrap();
    window.invoke_focus_calendar();
    key(&window, Key::Return);
    assert!(choices.borrow().is_empty());
    key(&window, Key::RightArrow);
    assert_eq!(*moves.borrow(), [CalendarNavigation::NextDay]);
    window.set_days(slint::ModelRc::new(slint::VecModel::from(vec![day(true)])));
    window.set_access(FieldAccess::ReadOnly);
    key(&window, Key::Return);
    key(&window, Key::LeftArrow);
    assert!(choices.borrow().is_empty());
    assert_eq!(
        moves.borrow().last(),
        Some(&CalendarNavigation::PreviousDay)
    );
    window.set_access(FieldAccess::Editable);
    key(&window, Key::Space);
    assert_eq!(choices.borrow().len(), 1);
}
#[test]
fn navigation_bounds_and_localized_range_actions_are_observable_without_focus_stealing() {
    i_slint_backend_testing::init_no_event_loop();
    let window = CalendarQaFixture::new().unwrap();
    window.set_previous_enabled(false);
    let moves = Rc::new(RefCell::new(Vec::new()));
    let observed = moves.clone();
    window.on_navigate(move |step| observed.borrow_mut().push(step));
    window.show().unwrap();
    for (label, count) in [("Mois précédent", 0), ("Mois suivant", 1)] {
        ElementQuery::from_root(&window)
            .match_predicate(move |element| element.accessible_label().as_deref() == Some(label))
            .find_first()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);
        assert_eq!(moves.borrow().len(), count);
    }
    for label in [
        "Annuler",
        "Confirmer la période",
        "Période précédente",
        "Période suivante",
    ] {
        assert!(ElementQuery::from_root(&window).match_predicate(move |element| element.accessible_label().as_deref() == Some(label)).find_first().is_some(), "{label}");
    }
}

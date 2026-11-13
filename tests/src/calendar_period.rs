// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{CalendarPeriodFixture, CalendarViewMode, DatePeriodFixture};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::ui::{CivilDateView, FieldAccess, Ownership};
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};
thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn backend() {
    BACKEND.with(|backend| {
        backend.get_or_init(|| {
            slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
                mock_time: true,
                renderer_name: Some("software".into()),
                ..Default::default()
            })))
            .unwrap();
        });
    });
}

fn date(year: i32, month: i32, day: i32) -> CivilDateView {
    CivilDateView { year, month, day }
}
fn fixture() -> CalendarPeriodFixture {
    backend();
    let window = CalendarPeriodFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    window
}
fn find<W: ComponentHandle>(window: &W, label: &'static str) -> Option<ElementHandle> {
    ElementQuery::from_root(window)
        .match_predicate(move |element| element.accessible_label().as_deref() == Some(label))
        .find_first()
}
fn settle<W: ComponentHandle>(window: &W) {
    for _ in 0..2 {
        slint::platform::update_timers_and_animations();
        window.window().take_snapshot().unwrap();
    }
}
fn click<W: ComponentHandle>(window: &W, label: &'static str) {
    find(window, label)
        .unwrap_or_else(|| panic!("no element labelled {label}"))
        .mock_single_click(PointerEventButton::Left);
    settle(window);
}
fn key<W: ComponentHandle>(window: &W, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    settle(window);
}

#[test]
fn heading_opens_months_and_a_chosen_month_returns_to_days_clamping_the_day() {
    let window = fixture();
    assert_eq!(window.get_view_mode(), CalendarViewMode::Days);
    click(&window, "October 2026");
    assert_eq!(window.get_view_mode(), CalendarViewMode::Months);
    assert!(find(&window, "2026").is_some());
    click(&window, "Previous year");
    assert!(find(&window, "2025").is_some());
    click(&window, "February 2025");
    assert_eq!(window.get_view_mode(), CalendarViewMode::Days);
    // The 31st clamps to the last day of February.
    assert_eq!(window.get_focused(), date(2025, 2, 28));
    assert!(find(&window, "February 2025").is_some());
}

#[test]
fn years_view_pages_and_chooses_a_year_then_a_month() {
    let window = fixture();
    click(&window, "October 2026");
    click(&window, "2026");
    assert_eq!(window.get_view_mode(), CalendarViewMode::Years);
    assert!(find(&window, "2016 - 2027").is_some());
    click(&window, "Previous years");
    assert!(find(&window, "2004 - 2015").is_some());
    click(&window, "2010");
    assert_eq!(window.get_view_mode(), CalendarViewMode::Months);
    assert!(find(&window, "2010").is_some());
    click(&window, "June 2010");
    assert_eq!(window.get_view_mode(), CalendarViewMode::Days);
    assert_eq!(window.get_focused(), date(2010, 6, 30));
}

#[test]
fn heading_in_years_view_returns_to_days_without_moving() {
    let window = fixture();
    click(&window, "October 2026");
    click(&window, "2026");
    click(&window, "2016 - 2027");
    assert_eq!(window.get_view_mode(), CalendarViewMode::Days);
    assert_eq!(window.get_focused(), date(2026, 10, 31));
}

#[test]
fn bounds_disable_unreachable_months_years_and_paging() {
    let window = fixture();
    window.set_min("2025-03-10".into());
    window.set_max("2026-05-20".into());
    window.set_focused(date(2025, 6, 15));
    click(&window, "June 2025");
    assert!(find(&window, "2025").is_some());
    let enabled = |label| find(&window, label).unwrap().accessible_enabled();
    assert_eq!(enabled("January 2025"), Some(false));
    assert_eq!(enabled("February 2025"), Some(false));
    assert_eq!(enabled("March 2025"), Some(true));
    assert_eq!(enabled("Previous year"), Some(false));
    assert_eq!(enabled("Next year"), Some(true));
    click(&window, "Next year");
    assert_eq!(enabled("May 2026"), Some(true));
    assert_eq!(enabled("June 2026"), Some(false));
    assert_eq!(enabled("Next year"), Some(false));
    // A disabled month does nothing.
    click(&window, "June 2026");
    assert_eq!(window.get_view_mode(), CalendarViewMode::Months);
    click(&window, "2026");
    assert_eq!(enabled("2024"), Some(false));
    assert_eq!(enabled("2025"), Some(true));
    assert_eq!(enabled("2026"), Some(true));
    assert_eq!(enabled("2027"), Some(false));
    assert_eq!(enabled("Previous years"), Some(false));
    assert_eq!(enabled("Next years"), Some(false));
}

#[test]
fn keyboard_moves_between_periods_pages_and_escape_leaves_without_cancelling() {
    let window = fixture();
    window.invoke_focus_calendar();
    click(&window, "October 2026");
    // Focus October (index 9); Right goes to November, Down to February of the next page.
    key(&window, Key::RightArrow);
    key(&window, Key::DownArrow);
    assert_eq!(window.get_view_mode(), CalendarViewMode::Months);
    key(&window, Key::Return);
    assert_eq!(window.get_view_mode(), CalendarViewMode::Days);
    assert_eq!(window.get_focused(), date(2027, 2, 28));
    click(&window, "February 2027");
    key(&window, Key::PageUp);
    assert!(find(&window, "2026").is_some());
    key(&window, Key::Escape);
    assert_eq!(window.get_view_mode(), CalendarViewMode::Days);
    assert_eq!(window.get_focused(), date(2027, 2, 28));
}

#[test]
fn read_only_browses_without_accepting_and_disabled_blocks_the_heading() {
    let window = fixture();
    let chosen = Rc::new(RefCell::new(Vec::new()));
    let observed = chosen.clone();
    window.on_calendar_chosen(move |chosen| observed.borrow_mut().push(chosen));
    window.set_access(FieldAccess::ReadOnly);
    click(&window, "October 2026");
    click(&window, "March 2026");
    assert_eq!(window.get_focused(), date(2026, 3, 31));
    assert_eq!(window.get_selected(), date(2026, 10, 1));
    assert!(chosen.borrow().is_empty());
    window.set_access(FieldAccess::Disabled);
    click(&window, "March 2026");
    assert_eq!(window.get_view_mode(), CalendarViewMode::Days);
    assert_eq!(window.get_focused(), date(2026, 3, 31));
}

#[test]
fn controlled_calendar_browses_locally_and_still_proposes_only_on_day_choice() {
    let window = fixture();
    window.set_ownership(Ownership::Controlled);
    click(&window, "October 2026");
    click(&window, "April 2026");
    assert_eq!(window.get_focused(), date(2026, 4, 30));
    assert_eq!(window.get_selected(), date(2026, 10, 1));
}

#[test]
fn host_can_open_on_the_months_grid() {
    let window = fixture();
    window.set_view_mode(CalendarViewMode::Months);
    assert!(find(&window, "March 2026").is_none() || find(&window, "October 2026").is_some());
    assert!(find(&window, "Previous year").is_some());
}

#[test]
fn picker_popup_reaches_a_distant_month_and_reopens_on_days() {
    backend();
    let window = DatePeriodFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    settle(&window);
    window.invoke_open_picker();
    settle(&window);
    click(&window, "October 2026");
    click(&window, "2026");
    click(&window, "2016 - 2027");
    click(&window, "October 2026");
    click(&window, "Previous year");
    click(&window, "Previous year");
    click(&window, "March 2024");
    assert!(window.get_picker_open());
    // The accepted day (the 1st) clamps into March 2024.
    assert_eq!(window.get_focused(), date(2024, 3, 1));
    assert!(find(&window, "March 2024").is_some());
}

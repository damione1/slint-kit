// SPDX-License-Identifier: GPL-3.0-or-later
use crate::CalendarPreviewFixture;
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, Model,
    platform::{PointerEventButton, WindowEvent},
};
use slint_kit::ui::{CalendarDay, CivilDateView, EarlierEndPolicy};
use std::cell::OnceCell;
thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn fixture() -> CalendarPreviewFixture {
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
    let window = CalendarPreviewFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    settle(&window);
    window
}
fn date(day: i32) -> CivilDateView {
    CivilDateView {
        year: 2026,
        month: 10,
        day,
    }
}
fn none() -> CivilDateView {
    CivilDateView {
        year: 0,
        month: 0,
        day: 0,
    }
}
fn settle(window: &CalendarPreviewFixture) {
    for _ in 0..2 {
        slint::platform::update_timers_and_animations();
        window.window().take_snapshot().unwrap();
    }
}
// The first matching cell belongs to the plain calendar, the second to the range picker.
fn cell(window: &CalendarPreviewFixture, day: i32, picker: usize) -> ElementHandle {
    let label = format!(" {day} October 2026");
    ElementQuery::from_root(window)
        .match_inherits("CalendarOption")
        .find_all()
        .into_iter()
        .filter(|cell| {
            cell.accessible_label()
                .is_some_and(|name| name.contains(&label))
        })
        .nth(picker)
        .unwrap()
}
fn hover(window: &CalendarPreviewFixture, day: i32, picker: usize) {
    let cell = cell(window, day, picker);
    let origin = cell.absolute_position();
    let size = cell.size();
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: slint::LogicalPosition::new(
            origin.x + size.width / 2.,
            origin.y + size.height / 2.,
        ),
    });
    settle(window);
}
fn leave(window: &CalendarPreviewFixture) {
    window.window().dispatch_event(WindowEvent::PointerExited);
    settle(window);
}
fn previewed(
    days: &impl Model<Data = CalendarDay>,
    check: impl Fn(&CalendarDay) -> bool,
) -> Vec<i32> {
    days.iter()
        .filter(|day| day.month == 10 && check(day))
        .map(|day| day.day)
        .collect()
}
fn calendar_preview(window: &CalendarPreviewFixture) -> Vec<i32> {
    previewed(&window.get_calendar_days(), |day| {
        window.invoke_calendar_previewed(day.clone())
    })
}
fn range_preview(window: &CalendarPreviewFixture) -> Vec<i32> {
    previewed(&window.get_range_days(), |day| {
        window.invoke_range_previewed(day.clone())
    })
}
#[test]
fn hovering_a_day_previews_the_days_after_the_anchor_up_to_it() {
    let window = fixture();
    assert!(calendar_preview(&window).is_empty());
    window.set_anchor(date(5));
    settle(&window);
    assert!(calendar_preview(&window).is_empty());
    hover(&window, 9, 0);
    assert_eq!(window.get_calendar_preview_end(), date(9));
    assert_eq!(calendar_preview(&window), vec![6, 7, 8, 9]);
    hover(&window, 7, 0);
    assert_eq!(calendar_preview(&window), vec![6, 7]);
    leave(&window);
    assert_eq!(window.get_calendar_preview_end(), none());
    assert!(calendar_preview(&window).is_empty());
}
#[test]
fn the_keyboard_focused_day_previews_the_range_while_the_calendar_has_focus() {
    let window = fixture();
    window.set_anchor(date(5));
    window.set_focused(date(9));
    settle(&window);
    assert_eq!(window.get_calendar_preview_end(), none());
    window.invoke_focus_calendar();
    settle(&window);
    assert_eq!(window.get_calendar_preview_end(), date(9));
    assert_eq!(calendar_preview(&window), vec![6, 7, 8, 9]);
}
#[test]
fn a_day_before_the_anchor_previews_only_when_the_earlier_end_swaps() {
    let window = fixture();
    window.set_anchor(date(9));
    settle(&window);
    hover(&window, 6, 0);
    assert!(calendar_preview(&window).is_empty());
    window.set_earlier_end(EarlierEndPolicy::Swap);
    settle(&window);
    assert_eq!(calendar_preview(&window), vec![6, 7, 8]);
}
#[test]
fn the_range_picker_previews_only_after_its_start_is_chosen() {
    let window = fixture();
    hover(&window, 8, 1);
    assert!(range_preview(&window).is_empty());
    cell(&window, 5, 1).mock_single_click(PointerEventButton::Left);
    settle(&window);
    hover(&window, 8, 1);
    assert_eq!(window.get_range_preview_end(), date(8));
    assert_eq!(range_preview(&window), vec![6, 7, 8]);
    cell(&window, 8, 1).mock_single_click(PointerEventButton::Left);
    settle(&window);
    assert!(range_preview(&window).is_empty());
}

// SPDX-License-Identifier: GPL-3.0-or-later
use crate::CalendarMarksFixture;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use slint_kit::ui::{CalendarDay, CalendarDayMark, CalendarMarker, CivilDateView};

fn date(year: i32, month: i32, day: i32) -> CivilDateView {
    CivilDateView { year, month, day }
}
fn day(window: &CalendarMarksFixture, number: i32) -> CalendarDay {
    window
        .get_calendar_days()
        .iter()
        .find(|day| day.year == 2026 && day.month == 10 && day.day == number && !day.outside_month)
        .unwrap()
}

#[test]
fn day_marks_decorate_only_their_dates_and_extend_the_accessible_label() {
    i_slint_backend_testing::init_no_event_loop();
    let window = CalendarMarksFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_marks(ModelRc::new(VecModel::from(vec![
        CalendarDayMark {
            date: date(2026, 10, 14),
            marker: CalendarMarker::Dot,
            caption: "89".into(),
            description: "Holiday".into(),
        },
        CalendarDayMark {
            date: date(2026, 10, 20),
            marker: CalendarMarker::Bar,
            caption: "".into(),
            description: "".into(),
        },
        CalendarDayMark::default(),
    ])));
    window.show().unwrap();
    let marked = day(&window, 14);
    assert_eq!(marked.marker, CalendarMarker::Dot);
    assert_eq!(marked.caption, "89");
    assert!(marked.accessible_label.ends_with(", Holiday"));
    let bar = day(&window, 20);
    assert_eq!(bar.marker, CalendarMarker::Bar);
    assert!(!bar.accessible_label.ends_with(", "));
    let plain = day(&window, 15);
    assert_eq!(plain.marker, CalendarMarker::None);
    assert_eq!(plain.caption, "");
}

#[test]
fn pickers_forward_day_marks_to_their_embedded_calendar() {
    i_slint_backend_testing::init_no_event_loop();
    let window = CalendarMarksFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_marks(ModelRc::new(VecModel::from(vec![CalendarDayMark {
        date: date(2026, 10, 14),
        marker: CalendarMarker::Dot,
        caption: "89".into(),
        description: "Holiday".into(),
    }])));
    window.show().unwrap();
    window.invoke_open_date();
    for days in [window.get_date_days(), window.get_range_days()] {
        let marked = days
            .iter()
            .find(|day| day.month == 10 && day.day == 14 && !day.outside_month)
            .unwrap();
        assert_eq!(marked.marker, CalendarMarker::Dot);
        assert_eq!(marked.caption, "89");
        assert!(marked.accessible_label.ends_with(", Holiday"));
        let plain = days
            .iter()
            .find(|day| day.month == 10 && day.day == 15 && !day.outside_month)
            .unwrap();
        assert_eq!(plain.marker, CalendarMarker::None);
    }
}

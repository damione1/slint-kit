// SPDX-License-Identifier: GPL-3.0-or-later
use crate::CalendarLabelsFixture;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::{
    ComponentHandle, Model, ModelRc, SharedString, VecModel,
    platform::{Key, WindowEvent},
};
use slint_kit::ui::{
    CalendarConfigurationState, CalendarDay, CivilDateView, DateRangePhase, DateRangeView,
};

fn names(values: &[&str]) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(
        values
            .iter()
            .map(|name| SharedString::from(*name))
            .collect::<Vec<_>>(),
    ))
}
fn day(days: ModelRc<CalendarDay>, number: i32) -> CalendarDay {
    days.iter()
        .find(|day| day.year == 2026 && day.month == 10 && day.day == number)
        .unwrap()
}
fn key(window: &CalendarLabelsFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    slint::platform::update_timers_and_animations();
}
fn french(window: &CalendarLabelsFixture) {
    window.set_month_names(names(&[
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ]));
    window.set_month_short_names(names(&[
        "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
        "déc.",
    ]));
    window.set_weekday_names(names(&[
        "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche",
    ]));
    window.set_weekday_short_names(names(&[
        "lun.", "mar.", "mer.", "jeu.", "ven.", "sam.", "dim.",
    ]));
    window.set_weekdays(names(&["L", "M", "M", "J", "V", "S", "D"]));
    window.set_today_label("Aujourd’hui".into());
    window.set_range_start_label("Début".into());
    window.set_range_end_label("Fin".into());
    window.set_in_range_label("Dans la plage".into());
    window.set_title_pattern("%Y — %B".into());
}

#[test]
fn localized_calendar_labels_are_instance_local_and_reach_all_three_controls() {
    i_slint_backend_testing::init_no_event_loop();
    let window = CalendarLabelsFixture::new().unwrap();
    slint_kit::install(&window);
    french(&window);
    window.show().unwrap();
    for days in [window.get_calendar_days(), window.get_date_days()] {
        assert_eq!(
            day(days.clone(), 1).accessible_label,
            "jeudi 1 octobre 2026"
        );
        assert_eq!(
            day(days, 2).accessible_label,
            "vendredi 2 octobre 2026, Aujourd’hui"
        );
    }
    let range = window.get_range_days();
    assert_eq!(
        day(range.clone(), 1).accessible_label,
        "jeudi 1 octobre 2026, Début"
    );
    assert_eq!(
        day(range.clone(), 2).accessible_label,
        "vendredi 2 octobre 2026, Aujourd’hui, Dans la plage"
    );
    assert_eq!(day(range, 3).accessible_label, "samedi 3 octobre 2026, Fin");
    for title in [
        window.get_calendar_title(),
        window.get_date_title(),
        window.get_range_title(),
    ] {
        assert_eq!(title, "2026 — octobre");
    }
    assert!(
        ElementHandle::find_by_accessible_label(&window, "calendar: jeudi 1 octobre 2026")
            .next()
            .is_some()
    );
    let mut headers = ElementQuery::from_root(&window)
        .match_id("KitCalendar::weekday-header")
        .find_all();
    headers.sort_by(|a, b| {
        a.absolute_position()
            .y
            .total_cmp(&b.absolute_position().y)
            .then(a.absolute_position().x.total_cmp(&b.absolute_position().x))
    });
    assert_eq!(headers[0].accessible_label().as_deref(), Some("dimanche"));
    window.invoke_open_date();
    slint::platform::update_timers_and_animations();
    assert!(
        ElementHandle::find_by_accessible_label(&window, "date: jeudi 1 octobre 2026")
            .next()
            .is_some()
    );
    for calendar in ElementQuery::from_root(&window)
        .match_inherits("KitCalendar")
        .find_all()
    {
        assert!(calendar.size().width > 0. && calendar.size().height > 0.);
    }
    window.invoke_close_date();
    window.set_today_label("".into());
    window.set_in_range_label("".into());
    assert_eq!(
        day(window.get_range_days(), 2).accessible_label,
        "vendredi 2 octobre 2026"
    );
    window.set_accessible_date_pattern("%a %-d %b %Y".into());
    assert_eq!(
        day(window.get_calendar_days(), 1).accessible_label,
        "jeu. 1 oct. 2026"
    );
    let other = CalendarLabelsFixture::new().unwrap();
    slint_kit::install(&other);
    assert_eq!(other.get_calendar_title(), "October 2026");
    assert_eq!(
        day(other.get_calendar_days(), 2).accessible_label,
        "Friday 2 October 2026, Today"
    );
}

#[test]
fn range_labels_follow_provisional_and_same_day_selection_without_changing_dates() {
    i_slint_backend_testing::init_no_event_loop();
    let window = CalendarLabelsFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    window.invoke_focus_range();
    key(&window, Key::Return);
    assert_eq!(window.get_range_phase(), DateRangePhase::ChoosingEnd);
    assert_eq!(
        day(window.get_range_days(), 1).accessible_label,
        "Thursday 1 October 2026, Range start"
    );
    assert_eq!(
        day(window.get_range_days(), 2).accessible_label,
        "Friday 2 October 2026, Today"
    );
    key(&window, Key::Return);
    assert_eq!(window.get_range_phase(), DateRangePhase::Complete);
    assert_eq!(
        day(window.get_range_days(), 1).accessible_label,
        "Thursday 1 October 2026, Range start, Range end"
    );
    assert_eq!(
        window.get_range_selection().end.day,
        3,
        "formatting leaves the accepted range intact before Apply"
    );
    let today = CivilDateView {
        year: 2026,
        month: 10,
        day: 2,
    };
    window.set_range_selection(DateRangeView {
        start: today.clone(),
        end: today,
    });
    slint::platform::update_timers_and_animations();
    assert_eq!(
        day(window.get_range_days(), 2).accessible_label,
        "Friday 2 October 2026, Today, Range start, Range end"
    );
    assert!(day(window.get_range_days(), 2).selected);
}

#[test]
fn invalid_calendar_text_is_rejected_and_visual_initials_remain_free() {
    i_slint_backend_testing::init_no_event_loop();
    let window = CalendarLabelsFixture::new().unwrap();
    slint_kit::install(&window);
    assert_eq!(
        window.get_calendar_configuration(),
        CalendarConfigurationState::Valid
    );
    for bad in ["%y-%m-%d", "%F %H", "%B %Y"] {
        window.set_accessible_date_pattern(bad.into());
        for state in [
            window.get_calendar_configuration(),
            window.get_date_configuration(),
            window.get_range_configuration(),
        ] {
            assert_eq!(state, CalendarConfigurationState::Invalid);
        }
        assert!(window.get_range_days().iter().all(|day| !day.enabled));
    }
    window.set_accessible_date_pattern("%F".into());
    window.set_title_pattern("%H".into());
    assert_eq!(
        window.get_calendar_configuration(),
        CalendarConfigurationState::Invalid
    );
    window.set_title_pattern("%Y %B".into());
    window.set_weekday_names(names(&["same"; 7]));
    assert_eq!(
        window.get_calendar_configuration(),
        CalendarConfigurationState::Invalid
    );
    french(&window);
    assert_eq!(
        window.get_calendar_configuration(),
        CalendarConfigurationState::Valid
    );
    window.set_month_short_names(names(&["only one"]));
    assert_eq!(
        window.get_calendar_configuration(),
        CalendarConfigurationState::Invalid
    );
}

#[test]
fn long_french_calendar_titles_fit_in_light_dark_and_double_text() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    use std::io::Write;
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    for (mode, dark) in [("light", false), ("dark", true)] {
        for scale in [1., 2.] {
            let window = crate::CalendarLabelsRenderFixture::new().unwrap();
            slint_kit::install(&window);
            let themes = window.global::<slint_kit::ui::KitThemes>();
            let mut theme = if dark {
                themes.get_dark()
            } else {
                themes.get_light()
            };
            theme.text_scale = scale;
            theme.reduced_motion = true;
            window.set_theme(theme);
            window.show().unwrap();
            slint::platform::update_timers_and_animations();
            let image = window.window().take_snapshot().unwrap();
            let calendar = ElementQuery::from_root(&window)
                .match_inherits("KitCalendar")
                .find_first()
                .unwrap();
            let title = ElementHandle::find_by_accessible_label(
                &window,
                "octobre 2026 — calendrier de réservation",
            )
            .next()
            .unwrap();
            assert!(title.size().width > 0. && title.size().height > 0.);
            assert!(title.absolute_position().x >= calendar.absolute_position().x);
            assert!(
                title.absolute_position().x + title.size().width
                    <= calendar.absolute_position().x + calendar.size().width + 1.
            );
            assert!(
                title.absolute_position().y + title.size().height
                    <= calendar.absolute_position().y + calendar.size().height
            );
            if let Some(directory) = std::env::var_os("KIT_CALENDAR_LABEL_RENDER_DIR") {
                std::fs::create_dir_all(&directory).unwrap();
                let path = std::path::PathBuf::from(directory)
                    .join(format!("calendar-labels-{mode}-{scale}.ppm"));
                let mut file = std::fs::File::create(path).unwrap();
                write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
                for pixel in image.as_slice() {
                    file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
                }
            }
        }
    }
}

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
        let old = ElementHandle::find_by_accessible_label(&window, "Thursday 1 October 2026")
            .next()
            .unwrap();
        old.invoke_accessible_default_action();
        assert_eq!(
            window.get_selected().day,
            1,
            "an invalid stored day is still unavailable"
        );
        ElementHandle::find_by_accessible_label(&window, "Tuesday 6 October 2026")
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

#[test]
fn invalid_date_configuration_keeps_a_disabled_grid_and_a_localized_diagnostic() {
    use crate::CalendarConstraintFixture;
    use i_slint_backend_testing::{
        ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
    };
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = CalendarConstraintFixture::new().unwrap();
    slint_kit::install(&window);
    let message = "Configuration du calendrier invalide";
    window
        .global::<slint_kit::ui::KitStrings>()
        .set_calendar_configuration_error(message.into());
    window.show().unwrap();
    for (mode, base) in [
        window.get_theme(),
        window.global::<slint_kit::ui::KitThemes>().get_dark(),
    ]
    .into_iter()
    .enumerate()
    {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            for (case, minimum, maximum) in [
                ("minimum", "not-a-date", "2026-10-31"),
                ("maximum", "2026-10-01", "2026-13-01"),
                ("inverted", "2026-10-31", "2026-10-01"),
            ] {
                window.set_min(minimum.into());
                window.set_max(maximum.into());
                slint::platform::update_timers_and_animations();
                assert_eq!(
                    window.get_configuration_state(),
                    slint_kit::ui::CalendarConfigurationState::Invalid
                );
                assert_eq!(
                    ElementQuery::from_root(&window)
                        .match_inherits("CalendarOption")
                        .find_all()
                        .len(),
                    42
                );
                assert!(
                    ElementHandle::find_by_accessible_label(&window, message)
                        .next()
                        .is_some()
                );
                let old = window.get_selected();
                let unavailable =
                    ElementHandle::find_by_accessible_label(&window, "Wednesday 7 October 2026")
                        .next()
                        .unwrap();
                assert_eq!(unavailable.accessible_enabled(), Some(false));
                unavailable.invoke_accessible_default_action();
                assert_eq!(
                    window.get_selected(),
                    old,
                    "a displayed fallback grid cannot bypass invalid date configuration"
                );
                if let Some(dir) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                    use std::io::Write;
                    let image = window.window().take_snapshot().unwrap();
                    std::fs::create_dir_all(&dir).unwrap();
                    let path = std::path::PathBuf::from(dir)
                        .join(format!("calendar-configuration-{case}-{mode}-{scale}.ppm"));
                    let mut file = std::fs::File::create(path).unwrap();
                    write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
                    for p in image.as_slice() {
                        file.write_all(&[p.r, p.g, p.b]).unwrap();
                    }
                }
            }
            window.set_min("2026-10-01".into());
            window.set_max("2026-10-31".into());
            slint::platform::update_timers_and_animations();
            assert_eq!(
                window.get_configuration_state(),
                slint_kit::ui::CalendarConfigurationState::Valid
            );
            assert!(
                ElementHandle::find_by_accessible_label(&window, message)
                    .next()
                    .is_none()
            );
            ElementHandle::find_by_accessible_label(&window, "Tuesday 6 October 2026")
                .next()
                .unwrap()
                .invoke_accessible_default_action();
            assert_eq!(window.get_selected().day, 6);
        }
    }
}

#[test]
fn weekday_headers_follow_every_week_start_and_the_first_grid_column() {
    use crate::{CivilDateView, WeekdayHeaderFixture};
    use i_slint_backend_testing::{
        ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
    };
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = WeekdayHeaderFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    let starts = [
        (slint_kit::ui::WeekStart::Monday, "2026-09-28"),
        (slint_kit::ui::WeekStart::Tuesday, "2026-09-29"),
        (slint_kit::ui::WeekStart::Wednesday, "2026-09-30"),
        (slint_kit::ui::WeekStart::Thursday, "2026-10-01"),
        (slint_kit::ui::WeekStart::Friday, "2026-09-25"),
        (slint_kit::ui::WeekStart::Saturday, "2026-09-26"),
        (slint_kit::ui::WeekStart::Sunday, "2026-09-27"),
    ];
    let names = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];
    for (mode, base) in [
        window.get_theme(),
        window.global::<slint_kit::ui::KitThemes>().get_dark(),
    ]
    .into_iter()
    .enumerate()
    {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            for (offset, (start, first_date)) in starts.iter().enumerate() {
                let october = CivilDateView {
                    year: 2026,
                    month: 10,
                    day: 19,
                };
                window.set_selected(october.clone());
                window.set_calendar_focused(october);
                window.set_week_start(*start);
                slint::platform::update_timers_and_animations();
                let image = window.window().take_snapshot().unwrap();
                let mut headers = ElementQuery::from_root(&window)
                    .match_id("KitCalendar::weekday-header")
                    .find_all();
                headers.sort_by(|a, b| a.absolute_position().x.total_cmp(&b.absolute_position().x));
                assert_eq!(headers.len(), 7);
                for (column, header) in headers.iter().enumerate() {
                    assert_eq!(
                        header.accessible_label().as_deref(),
                        Some(names[(column + offset) % 7])
                    );
                }
                let date = match kit_core::CivilDate::parse_iso(first_date) {
                    kit_core::DraftValue::Parsed(date) => date,
                    other => panic!("invalid fixture date: {other:?}"),
                };
                let label = kit_core::DatePattern::new("%A %-d %B %Y")
                    .unwrap()
                    .format(date);
                let first_day = ElementHandle::find_by_accessible_label(&window, &label)
                    .next()
                    .unwrap();
                assert!(headers[0].size().width > 0.);
                assert!(
                    (headers[0].absolute_position().x - first_day.absolute_position().x).abs() < 1.
                );
                assert!((headers[0].size().width - first_day.size().width).abs() < 1.);
                if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                    use std::io::Write;
                    std::fs::create_dir_all(&directory).unwrap();
                    let path = std::path::PathBuf::from(directory)
                        .join(format!("calendar-weekdays-{offset}-{mode}-{scale}.ppm"));
                    let mut file = std::fs::File::create(path).unwrap();
                    write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
                    for pixel in image.as_slice() {
                        file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
                    }
                }
                first_day.mock_single_click(slint::platform::PointerEventButton::Left);
                let chosen = window.get_chosen_date();
                assert_eq!(
                    format!("{:04}-{:02}-{:02}", chosen.year, chosen.month, chosen.day),
                    *first_date,
                    "the rotated first column keeps its actual date when clicked"
                );
            }
        }
    }
}

#[test]
fn explicit_today_is_window_local_and_both_pickers_focus_the_nearest_available_date() {
    use crate::CalendarTodayFixture;
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    use slint::Model;
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = CalendarTodayFixture::new().unwrap();
    let other = CalendarTodayFixture::new().unwrap();
    slint_kit::install(&window);
    slint_kit::install(&other);
    let zone = slint_kit::TimeZoneId::new("UTC").unwrap();
    let local = slint_kit::LocalDateTime::new(
        slint_kit::CivilDate::new(2026, 10, 19).unwrap(),
        slint_kit::LocalTime::new(12, 0, 0).unwrap(),
    );
    let instant = match zone.resolve(local) {
        slint_kit::ZonedResolution::Unique(value) => value,
        slint_kit::ZonedResolution::Ambiguous { .. } | slint_kit::ZonedResolution::Nonexistent => {
            panic!("UTC is unambiguous")
        }
    };
    let today = slint_kit::set_calendar_today_at(&window, zone, instant).unwrap();
    slint::platform::update_timers_and_animations();
    assert_eq!(today, local.date);
    let projected: CivilDateView = today.into();
    assert_eq!(window.get_date_today(), projected);
    assert_eq!(window.get_range_today(), projected);
    assert_eq!(window.get_date_focus(), projected);
    assert_eq!(window.get_range_focus(), projected);
    assert_eq!(other.get_date_today(), CivilDateView::default());
    window.set_unavailable_weekdays(slint::ModelRc::new(slint::VecModel::from(vec![
        slint_kit::ui::WeekStart::Monday,
    ])));
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_date_focus().day, 20);
    assert_eq!(window.get_range_focus().day, 20);
    window.set_min("2026-10-21".into());
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_date_focus().day, 21);
    assert_eq!(window.get_range_focus().day, 21);
    let today_rows = window
        .get_range_days()
        .iter()
        .filter(|day| day.today)
        .collect::<Vec<_>>();
    assert_eq!(today_rows.len(), 1);
    assert_eq!(today_rows[0].day, 19);
    assert!(!today_rows[0].enabled);
    window.show().unwrap();
    for (mode, base) in [
        window.get_theme(),
        window.global::<slint_kit::ui::KitThemes>().get_dark(),
    ]
    .into_iter()
    .enumerate()
    {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            slint::platform::update_timers_and_animations();
            window.invoke_close_date();
            crate::choice_presentation::save_capture(
                &window.window().take_snapshot().unwrap(),
                &format!("calendar-today-range-{mode}-{scale}"),
            );
            window.invoke_open_date();
            crate::choice_presentation::save_capture(
                &window.window().take_snapshot().unwrap(),
                &format!("calendar-today-date-{mode}-{scale}"),
            );
            assert_eq!(
                window
                    .get_date_days()
                    .iter()
                    .filter(|day| day.today)
                    .count(),
                1
            );
            window.invoke_close_date();
        }
    }
}

#[test]
fn calendar_today_override_and_user_navigation_survive_window_clock_refresh() {
    use crate::CalendarTodayFixture;
    use slint::platform::{Key, WindowEvent};
    i_slint_backend_testing::init_no_event_loop();
    let window = CalendarTodayFixture::new().unwrap();
    let service_today = CivilDateView {
        year: 2026,
        month: 10,
        day: 19,
    };
    window
        .global::<slint_kit::ui::KitCalendarService>()
        .set_today(service_today.clone());
    slint_kit::install(&window);
    assert_eq!(window.get_date_focus(), service_today);
    let beyond = slint_kit::Instant::from_unix_seconds(253_402_300_799).unwrap();
    assert!(
        slint_kit::set_calendar_today_at(
            &window,
            slint_kit::TimeZoneId::new("Pacific/Kiritimati").unwrap(),
            beyond,
        )
        .is_err()
    );
    assert_eq!(
        window
            .global::<slint_kit::ui::KitCalendarService>()
            .get_today(),
        service_today,
        "invalid clock projection cannot overwrite the window date"
    );
    let explicit = CivilDateView {
        year: 2026,
        month: 11,
        day: 4,
    };
    window.set_explicit_today(explicit.clone());
    window.set_override_today(true);
    assert_eq!(window.get_date_today(), explicit);
    assert_eq!(window.get_date_focus(), explicit);
    let accepted = CivilDateView {
        year: 2026,
        month: 12,
        day: 8,
    };
    window.set_accepted_date(accepted.clone());
    assert_eq!(
        window.get_date_focus(),
        accepted,
        "stored value has initial priority"
    );
    window.show().unwrap();
    window.invoke_focus_range();
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::RightArrow.into(),
    });
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::RightArrow.into(),
    });
    let navigated = window.get_range_focus();
    assert_eq!(navigated.day, 20);
    window
        .global::<slint_kit::ui::KitCalendarService>()
        .set_today(CivilDateView {
            year: 2026,
            month: 11,
            day: 2,
        });
    slint::platform::update_timers_and_animations();
    assert_eq!(
        window.get_range_focus(),
        navigated,
        "clock refresh preserves native navigation"
    );
    assert_eq!(
        window.get_date_today(),
        explicit,
        "instance today override stays independent"
    );
}

macro_rules! calendar_flow_regression {
    ($test:ident, $fixture:ident, $scene:literal) => {
        #[test]
        fn $test() {
            use i_slint_backend_testing::{
                ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
            };
            use slint::{Model, platform::PointerEventButton};
            use slint_kit::ui::{ChoiceFlow, KitThemes, WeekStart};
            slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
                mock_time: true,
                renderer_name: Some("software".into()),
                ..Default::default()
            })))
            .unwrap();
            for (mode, dark) in [("light", false), ("dark", true)] {
                for scale in [1., 2.] {
                    let window = crate::$fixture::new().unwrap();
                    slint_kit::install(&window);
                    let mut theme = if dark {
                        window.global::<KitThemes>().get_dark()
                    } else {
                        window.get_theme()
                    };
                    theme.text_scale = scale;
                    theme.reduced_motion = true;
                    window.set_theme(theme.clone());
                    window.show().unwrap();
                    window.invoke_focus_current();
                    let frame = || {
                        slint::platform::update_timers_and_animations();
                        window.window().take_snapshot().unwrap()
                    };
                    let key = |key: Key| {
                        window
                            .window()
                            .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
                        window
                            .window()
                            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
                        frame();
                    };
                    let october = CivilDateView {
                        year: 2026,
                        month: 10,
                        day: 19,
                    };
                    for flow in [ChoiceFlow::LeftToRight, ChoiceFlow::RightToLeft] {
                        theme.direction = flow;
                        window.set_theme(theme.clone());
                        frame();
                        assert_eq!(
                            window.get_flow(),
                            flow,
                            "flow defaults to this component's theme"
                        );
                        for (start, offset) in [(WeekStart::Monday, 0), (WeekStart::Sunday, 6)] {
                            window.set_week_start(start);
                            window.set_focused(october.clone());
                            frame();
                            let mut headers = ElementQuery::from_root(&window)
                                .match_id("KitCalendar::weekday-header")
                                .find_all();
                            headers.sort_by(|a, b| {
                                a.absolute_position().x.total_cmp(&b.absolute_position().x)
                            });
                            assert_eq!(headers.len(), 7);
                            let names = [
                                "Monday",
                                "Tuesday",
                                "Wednesday",
                                "Thursday",
                                "Friday",
                                "Saturday",
                                "Sunday",
                            ];
                            for (position, header) in headers.iter().enumerate() {
                                let logical = if flow == ChoiceFlow::RightToLeft {
                                    6 - position
                                } else {
                                    position
                                };
                                assert_eq!(
                                    header.accessible_label().as_deref(),
                                    Some(names[(logical + offset) % 7])
                                );
                            }
                            let first = window.get_days().row_data(0).unwrap();
                            let first_cell = ElementHandle::find_by_accessible_label(
                                &window,
                                &first.accessible_label,
                            )
                            .next()
                            .unwrap();
                            let edge = if flow == ChoiceFlow::RightToLeft {
                                6
                            } else {
                                0
                            };
                            assert!(
                                (first_cell.absolute_position().x
                                    - headers[edge].absolute_position().x)
                                    .abs()
                                    < 1.
                            );
                            key(Key::LeftArrow);
                            assert_eq!(
                                window.get_focused().day,
                                if flow == ChoiceFlow::RightToLeft {
                                    20
                                } else {
                                    18
                                }
                            );
                            key(Key::RightArrow);
                            assert_eq!(window.get_focused(), october);
                            key(Key::UpArrow);
                            assert_eq!(window.get_focused().day, 12);
                            key(Key::DownArrow);
                            assert_eq!(window.get_focused(), october);
                            key(Key::Home);
                            assert_eq!(
                                window.get_focused().day,
                                if start == WeekStart::Monday { 19 } else { 18 }
                            );
                            key(Key::End);
                            assert_eq!(
                                window.get_focused().day,
                                if start == WeekStart::Monday { 25 } else { 24 }
                            );
                            window.set_focused(october.clone());
                            key(Key::PageUp);
                            assert_eq!(window.get_focused().month, 9);
                            key(Key::PageDown);
                            assert_eq!(window.get_focused(), october);
                            if flow == ChoiceFlow::RightToLeft && start == WeekStart::Monday {
                                crate::choice_presentation::save_capture(
                                    &frame(),
                                    &format!("calendar-flow-{}-{mode}-{scale}", $scene),
                                );
                            }
                        }
                    }
                    // An explicit override is independent of an RTL theme and
                    // must pass through each picker's internal calendar.
                    window.invoke_override_flow(ChoiceFlow::LeftToRight);
                    window.set_focused(october.clone());
                    frame();
                    key(Key::RightArrow);
                    assert_eq!(window.get_focused().day, 20);
                    window.invoke_override_flow(ChoiceFlow::RightToLeft);
                    frame();
                    let previous =
                        ElementHandle::find_by_accessible_label(&window, "Previous month")
                            .next()
                            .unwrap();
                    let next = ElementHandle::find_by_accessible_label(&window, "Next month")
                        .next()
                        .unwrap();
                    assert!(previous.absolute_position().x > next.absolute_position().x);
                    previous.mock_single_click(PointerEventButton::Left);
                    frame();
                    assert_eq!(window.get_focused().month, 9);
                    next.mock_single_click(PointerEventButton::Left);
                    frame();
                    assert_eq!(window.get_focused().month, 10);
                }
            }
        }
    };
}
calendar_flow_regression!(
    calendar_flow_mirrors_geometry_and_keyboard,
    CalendarFlowFixture,
    "calendar"
);
calendar_flow_regression!(
    date_picker_flow_reaches_the_popup_calendar,
    DatePickerFlowFixture,
    "date-picker"
);
calendar_flow_regression!(
    range_picker_flow_reaches_the_embedded_calendar,
    RangePickerFlowFixture,
    "range-picker"
);

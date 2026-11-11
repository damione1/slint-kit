// SPDX-License-Identifier: GPL-3.0-or-later
use crate::CalendarOutsideFixture;
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, Model,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::ui::{
    CalendarOutsideDays, CalendarWeekRows, CivilDateView, FieldAccess, Ownership, WeekStart,
};
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};
thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn fixture() -> CalendarOutsideFixture {
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
    let window = CalendarOutsideFixture::new().unwrap();
    slint_kit::install(&window);
    window
}
fn date(year: i32, month: i32, day: i32) -> CivilDateView {
    CivilDateView { year, month, day }
}
fn settle(window: &CalendarOutsideFixture) {
    for _ in 0..2 {
        slint::platform::update_timers_and_animations();
        window.window().take_snapshot().unwrap();
    }
}
fn key(window: &CalendarOutsideFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    settle(window);
}
fn cells(window: &CalendarOutsideFixture) -> Vec<ElementHandle> {
    ElementQuery::from_root(window)
        .match_inherits("CalendarOption")
        .find_all()
}
fn outside(window: &CalendarOutsideFixture) -> ElementHandle {
    cells(window)
        .into_iter()
        .find(|cell| cell.accessible_label().as_deref() == Some("Monday 28 September 2026"))
        .unwrap()
}
#[test]
fn all_three_projections_preserve_month_identity_and_use_four_five_or_six_whole_weeks() {
    let window = fixture();
    window.show().unwrap();
    settle(&window);
    for days in [
        window.get_calendar_days(),
        window.get_date_days(),
        window.get_range_days(),
    ] {
        assert_eq!(days.row_count(), 42);
        assert_eq!(days.iter().filter(|day| day.outside_month).count(), 11);
        assert!(
            days.iter()
                .all(|day| day.outside_month == (day.month != 10))
        );
    }
    window.set_week_rows(CalendarWeekRows::Variable);
    for (year, month, expected) in [(2021, 2, 28), (2026, 10, 35), (2026, 8, 42)] {
        window.set_focused(date(year, month, 1));
        settle(&window);
        for days in [
            window.get_calendar_days(),
            window.get_date_days(),
            window.get_range_days(),
        ] {
            assert_eq!(days.row_count(), expected);
            assert!(days.iter().skip(expected - 7).any(|day| !day.outside_month));
        }
    }
    window.set_focused(date(2021, 2, 1));
    window.set_week_start(WeekStart::Sunday);
    settle(&window);
    assert_eq!(window.get_calendar_days().row_count(), 35);
    assert!(
        window
            .get_calendar_days()
            .row_data(0)
            .unwrap()
            .outside_month
    );
    window.set_week_rows(CalendarWeekRows::Fixed);
    settle(&window);
    assert_eq!(window.get_calendar_days().row_count(), 42);
}
#[test]
fn outside_policy_guards_stale_pointer_admission_and_preserves_controlled_keyboard_and_bounds() {
    let window = fixture();
    window.set_ownership(Ownership::Controlled);
    let chosen = Rc::new(RefCell::new(Vec::new()));
    let observed = chosen.clone();
    window.on_calendar_chosen(move |value| observed.borrow_mut().push(value));
    window.show().unwrap();
    settle(&window);
    window.set_outside_days(CalendarOutsideDays::Show);
    outside(&window).mock_single_click(PointerEventButton::Left);
    settle(&window);
    assert_eq!(chosen.borrow().as_slice(), &[date(2026, 9, 28)]);
    assert_eq!(window.get_selected(), date(2026, 10, 1));
    window.set_focused(date(2026, 10, 1));
    settle(&window);
    let cell = outside(&window);
    let origin = cell.absolute_position();
    let size = cell.size();
    let position =
        slint::LogicalPosition::new(origin.x + size.width / 2., origin.y + size.height / 2.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window.set_outside_days(CalendarOutsideDays::Hide);
    settle(&window);
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    settle(&window);
    assert_eq!(chosen.borrow().len(), 1);
    assert!(
        cells(&window)
            .iter()
            .all(|cell| cell.accessible_label().as_deref() != Some("Monday 28 September 2026"))
    );
    window.invoke_focus_calendar();
    key(&window, Key::LeftArrow);
    assert_eq!(window.get_focused(), date(2026, 9, 30));
    key(&window, Key::Return);
    assert_eq!(chosen.borrow().last(), Some(&date(2026, 9, 30)));
    assert_eq!(window.get_selected(), date(2026, 10, 1));
    window.set_min("2026-10-01".into());
    window.set_max("2026-10-31".into());
    window.set_focused(date(2026, 10, 1));
    settle(&window);
    key(&window, Key::LeftArrow);
    assert_eq!(window.get_focused(), date(2026, 10, 1));
    let count = chosen.borrow().len();
    window.set_access(FieldAccess::ReadOnly);
    key(&window, Key::RightArrow);
    key(&window, Key::Return);
    assert_eq!(window.get_focused(), date(2026, 10, 2));
    assert_eq!(chosen.borrow().len(), count);
    window.set_access(FieldAccess::Disabled);
    key(&window, Key::RightArrow);
    assert_eq!(window.get_focused(), date(2026, 10, 2));
}
fn capture(window: &CalendarOutsideFixture, mode: &str, scale: f32, policy: &str, expected: usize) {
    let mut calendars = ElementQuery::from_root(window)
        .match_inherits("KitCalendar")
        .find_all();
    assert_eq!(calendars.len(), 3);
    calendars.sort_by(|left, right| {
        left.absolute_position()
            .x
            .total_cmp(&right.absolute_position().x)
    });
    for pair in calendars.windows(2) {
        assert!(
            pair[0].absolute_position().x + pair[0].size().width <= pair[1].absolute_position().x,
            "the popup and neighboring calendars must not overlap"
        );
    }
    let entries = cells(window);
    assert_eq!(entries.len(), expected);
    let size = window
        .window()
        .size()
        .to_logical(window.window().scale_factor());
    for cell in entries {
        let origin = cell.absolute_position();
        let bounds = cell.size();
        assert!(bounds.width > 0. && bounds.height > 0.);
        assert!(
            origin.x >= 0.
                && origin.y >= 0.
                && origin.x + bounds.width <= size.width + 1.
                && origin.y + bounds.height <= size.height + 1.
        );
    }
    let image = window.window().take_snapshot().unwrap();
    if let Some(dir) = std::env::var_os("KIT_CALENDAR_OUTSIDE_RENDER_DIR") {
        use std::io::Write;
        std::fs::create_dir_all(&dir).unwrap();
        let mut file = std::fs::File::create(
            std::path::PathBuf::from(dir)
                .join(format!("calendar-outside-{mode}-{scale}-{policy}.ppm")),
        )
        .unwrap();
        write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
        for pixel in image.as_slice() {
            file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
        }
    }
}
#[test]
fn dim_and_hidden_variable_grids_render_in_all_three_controls_without_losing_slots_or_height() {
    for (mode, dark) in [("light", false), ("dark", true)] {
        for scale in [1., 2.] {
            let window = fixture();
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
            window.invoke_open_date();
            settle(&window);
            let fixed_height = window.get_calendar_height();
            assert_eq!(window.get_outside_days(), CalendarOutsideDays::Dim);
            capture(&window, mode, scale, "dim-fixed", 126);
            window.set_outside_days(CalendarOutsideDays::Hide);
            window.set_week_rows(CalendarWeekRows::Variable);
            settle(&window);
            assert_eq!(window.get_date_days().row_count(), 35);
            assert_eq!(window.get_range_days().row_count(), 35);
            assert!(window.get_calendar_height() < fixed_height);
            capture(&window, mode, scale, "hide-variable", 93);
        }
    }
}

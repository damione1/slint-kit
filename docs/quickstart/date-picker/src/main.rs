use slint::ComponentHandle;
use slint_kit::{
    CalendarController, CalendarMove, CivilDate, DateConstraints, DatePattern, DateRange,
    NumericBounds, WeekStart,
};
use std::{cell::RefCell, rc::Rc};
mod ui {
    slint::include_modules!();
}
struct State {
    date_label: DatePattern,
    calendar: CalendarController,
    accepted: CivilDate,
}
fn movement(step: ui::CalendarNavigation) -> CalendarMove {
    match step {
        ui::CalendarNavigation::PreviousDay => CalendarMove::PreviousDay,
        ui::CalendarNavigation::NextDay => CalendarMove::NextDay,
        ui::CalendarNavigation::PreviousWeek => CalendarMove::PreviousWeek,
        ui::CalendarNavigation::NextWeek => CalendarMove::NextWeek,
        ui::CalendarNavigation::WeekStart => CalendarMove::WeekStart,
        ui::CalendarNavigation::WeekEnd => CalendarMove::WeekEnd,
        ui::CalendarNavigation::PreviousMonth => CalendarMove::PreviousMonth,
        ui::CalendarNavigation::NextMonth => CalendarMove::NextMonth,
        ui::CalendarNavigation::PreviousYear => CalendarMove::PreviousYear,
        ui::CalendarNavigation::NextYear => CalendarMove::NextYear,
    }
}
fn publish(app: &ui::App, state: &State) {
    let slots = state.calendar.grid_slots();
    let days = slots
        .iter()
        .map(|slot| {
            slot.map_or_else(ui::CalendarDay::default, |cell| ui::CalendarDay {
                year: cell.date.year(),
                month: cell.date.month() as i32,
                day: cell.date.day() as i32,
                label: cell.date.day().to_string().into(),
                accessible_label: state.date_label.format(cell.date).into(),
                enabled: cell.enabled,
                selected: cell.selected,
                in_range: false,
                today: cell.date == CivilDate::new(2026, 10, 30).unwrap(),
            })
        })
        .collect::<Vec<_>>();

    app.set_days(slint::ModelRc::new(slint::VecModel::from(days)));
    app.set_active(
        slots
            .iter()
            .position(|s| s.is_some_and(|cell| cell.focused))
            .unwrap_or(0) as i32,
    );
    let month = state.calendar.displayed();
    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    app.set_calendar_title(
        format!(
            "{} {}",
            months[usize::try_from(month.month() - 1).unwrap()],
            month.year()
        )
        .into(),
    );
    app.set_previous_enabled(state.calendar.can_navigate(CalendarMove::PreviousMonth));
    app.set_next_enabled(state.calendar.can_navigate(CalendarMove::NextMonth));
    app.set_accepted(state.accepted.iso_string().into());
    app.set_accepted_date(ui::CivilDateView {
        year: state.accepted.year(),
        month: state.accepted.month() as i32,
        day: state.accepted.day() as i32,
    });
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let date = CivilDate::new(2026, 10, 19).unwrap();
    let bounds = NumericBounds::new(
        CivilDate::new(2026, 10, 1).unwrap(),
        CivilDate::new(2027, 12, 31).unwrap(),
    )
    .unwrap();
    let constraints = DateConstraints::new(Some(bounds))
        .with_unavailable_weekdays(vec![WeekStart::Saturday, WeekStart::Sunday])
        .with_unavailable_ranges(vec![
            DateRange::new(
                CivilDate::new(2026, 10, 13).unwrap(),
                CivilDate::new(2026, 10, 15).unwrap(),
            )
            .unwrap(),
        ]);
    let state = Rc::new(RefCell::new(State {
        date_label: DatePattern::new("%A, %B %-d, %Y").unwrap(),
        calendar: CalendarController::with_constraints(
            Some(date),
            date.year_month(),
            constraints.clone(),
            WeekStart::Monday,
        )
        .unwrap(),
        accepted: date,
    }));
    publish(&app, &state.borrow());
    app.set_draft(date.iso_string().into());
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_navigate(move |step| {
        if let Some(app) = weak.upgrade() {
            let mut state = shared.borrow_mut();
            if state.calendar.navigate(movement(step)).is_ok() {
                publish(&app, &state);
            }
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_choose(move |day| {
        if !day.enabled {
            return;
        }
        if let (Some(app), Ok(date)) = (
            weak.upgrade(),
            CivilDate::new(day.year, day.month as u32, day.day as u32),
        ) {
            let mut state = shared.borrow_mut();
            if state.calendar.choose(date).is_ok() {
                state.accepted = date;
                app.set_draft(date.iso_string().into());
                app.set_error("".into());
                publish(&app, &state);
            }
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_cancel(move || {
        if let Some(app) = weak.upgrade() {
            let state = shared.borrow();
            app.set_draft(state.accepted.iso_string().into());
            app.set_error("".into());
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_commit(move |text| {
        if let Some(app) = weak.upgrade() {
            match slint_kit::CivilDate::parse_iso(&text) {
                slint_kit::DraftValue::Parsed(date) => {
                    let mut state = shared.borrow_mut();
                    if state.calendar.choose(date).is_ok() {
                        state.accepted = date;
                        app.set_draft(date.iso_string().into());
                        app.set_error("".into());
                        publish(&app, &state);
                    } else {
                        app.set_error(
                            "Choose an available weekday within the booking dates".into(),
                        );
                    }
                }
                slint_kit::DraftValue::Empty
                | slint_kit::DraftValue::Incomplete
                | slint_kit::DraftValue::Invalid(_) => {
                    app.set_error("Use a valid YYYY-MM-DD date".into())
                }
            }
        }
    });
    app.run()
}

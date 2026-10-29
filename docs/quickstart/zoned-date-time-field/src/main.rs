use slint::ComponentHandle;
use slint_kit::{
    CivilDate, DraftValue, Instant, LocalDateTime, LocalTime, TimePrecision, TimeZoneId,
    ZonedResolution,
};
use std::{cell::RefCell, rc::Rc};
mod ui {
    slint::include_modules!();
}
fn local(date: &str, time: &str) -> Option<LocalDateTime> {
    match (
        CivilDate::parse_iso(date),
        LocalTime::parse(time, TimePrecision::Minute),
    ) {
        (DraftValue::Parsed(date), DraftValue::Parsed(time)) => {
            Some(LocalDateTime::new(date, time))
        }
        _ => None,
    }
}
fn view(value: LocalDateTime) -> ui::LocalDateTimeView {
    ui::LocalDateTimeView {
        date: ui::CivilDateView {
            year: value.date.year(),
            month: value.date.month() as i32,
            day: value.date.day() as i32,
        },
        time: ui::LocalTimeView {
            hour: value.time.hour() as i32,
            minute: value.time.minute() as i32,
            second: value.time.second() as i32,
        },
    }
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let initial = local("2026-10-17", "09:30").unwrap();
    app.set_accepted_local(view(initial));
    app.set_accepted("Not resolved yet".into());
    let state = Rc::new(RefCell::new((
        initial,
        None::<Instant>,
        None::<(ui::ZonedDateTimeDraft, Instant, Instant)>,
        TimeZoneId::new("America/Toronto").unwrap(),
    )));
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_commit(move |draft| {
        if let Some(app) = weak.upgrade() {
            let Some(value) = local(&draft.date, &draft.time) else {
                app.set_error("Use a valid YYYY-MM-DD date and HH:mm time".into());
                return;
            };
            shared.borrow_mut().2 = None;
            let Ok(zone) = TimeZoneId::new(&draft.zone) else {
                app.set_resolution(ui::ZonedTimePresentation::InvalidZone);
                app.set_error("Use an IANA zone such as America/Toronto".into());
                return;
            };
            match zone.resolve(value) {
                ZonedResolution::Unique(instant) => {
                    let mut state = shared.borrow_mut();
                    state.0 = value;
                    state.1 = Some(instant);
                    state.3 = zone;
                    app.set_accepted_local(view(value));
                    app.set_resolution(ui::ZonedTimePresentation::Unique);
                    app.set_error("".into());
                    app.set_accepted(
                        format!("{} seconds since Unix epoch", instant.unix_seconds()).into(),
                    );
                }
                ZonedResolution::Nonexistent => {
                    app.set_resolution(ui::ZonedTimePresentation::Nonexistent);
                    app.set_error("This local time does not exist; choose another time".into());
                }
                ZonedResolution::Ambiguous { earlier, later } => {
                    shared.borrow_mut().2 = Some((draft, earlier, later));
                    app.set_resolution(ui::ZonedTimePresentation::Ambiguous);
                    app.set_error("Choose the earlier or later occurrence explicitly".into());
                }
            }
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_edit(move || {
        if let Some(app) = weak.upgrade() {
            app.set_error("".into());
            shared.borrow_mut().2 = None;
            app.set_resolution(ui::ZonedTimePresentation::Unresolved);
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_cancel(move || {
        if let Some(app) = weak.upgrade() {
            let mut state = shared.borrow_mut();
            state.2 = None;
            app.set_zone_text(state.3.name().into());
            app.set_date_text(state.0.date.iso_string().into());
            app.set_time_text(
                state
                    .0
                    .time
                    .format(
                        TimePrecision::Minute,
                        slint_kit::ClockFormat::TwentyFourHour,
                    )
                    .into(),
            );
            app.set_resolution(ui::ZonedTimePresentation::Unresolved);
            app.set_error("".into());
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_choose(move |choice| {
        if let Some(app) = weak.upgrade() {
            let mut state = shared.borrow_mut();
            let Some((draft, earlier, later)) = state.2.take() else {
                return;
            };
            if draft.date != app.get_date_text()
                || draft.time != app.get_time_text()
                || draft.zone != app.get_zone_text()
            {
                return;
            }
            let instant = if choice == ui::AmbiguousTimeChoice::Earlier {
                earlier
            } else {
                later
            };
            state.1 = Some(instant);
            state.3 = TimeZoneId::new(&draft.zone).unwrap();
            state.0 = local(&draft.date, &draft.time).unwrap();
            app.set_accepted_local(view(state.0));
            app.set_resolution(ui::ZonedTimePresentation::Unique);
            app.set_error("".into());
            app.set_accepted(format!("{} seconds since Unix epoch", instant.unix_seconds()).into());
        }
    });
    app.run()
}

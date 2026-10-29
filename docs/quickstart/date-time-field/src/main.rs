use slint::ComponentHandle;
use slint_kit::{CivilDate, DraftValue, LocalDateTime, LocalTime, TimePrecision};
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
    app.set_accepted("2026-10-17 09:30".into());
    let state = Rc::new(RefCell::new(initial));
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_commit(move |draft| {
        if let Some(app) = weak.upgrade() {
            let Some(value) = local(&draft.date, &draft.time) else {
                app.set_error("Use a valid YYYY-MM-DD date and HH:mm time".into());
                return;
            };
            *shared.borrow_mut() = value;
            app.set_accepted_local(view(value));
            app.set_error("".into());
            app.set_accepted(
                format!(
                    "{} {}",
                    value.date.iso_string(),
                    value.time.format(
                        TimePrecision::Minute,
                        slint_kit::ClockFormat::TwentyFourHour
                    )
                )
                .into(),
            );
        }
    });
    let weak = app.as_weak();
    app.on_edit(move || {
        if let Some(app) = weak.upgrade() {
            app.set_error("".into());
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_cancel(move || {
        if let Some(app) = weak.upgrade() {
            let value = *shared.borrow();
            app.set_date_text(value.date.iso_string().into());
            app.set_time_text(
                value
                    .time
                    .format(
                        TimePrecision::Minute,
                        slint_kit::ClockFormat::TwentyFourHour,
                    )
                    .into(),
            );
            app.set_error("".into());
        }
    });

    app.run()
}

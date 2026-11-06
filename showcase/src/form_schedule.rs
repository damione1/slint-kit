// SPDX-License-Identifier: GPL-3.0-or-later
//! Business schedule values; component drafts and interaction stay in the kit.
use super::ui::*;
use slint::ComponentHandle;
use slint_kit::{
    CivilDate, DateRange, DraftValue, FieldSpec, FieldSpecKey, IsoWeek, LocalDateTime, LocalTime,
    ResolvedDateTime, TimePrecision, Year, YearMonth, event, event2,
};
use std::{cell::RefCell, rc::Rc};
struct Schedule {
    date: CivilDate,
    month: YearMonth,
    year: Year,
    week: IsoWeek,
    time: LocalTime,
    local: LocalDateTime,
    zoned: Option<ResolvedDateTime>,
}
impl Schedule {
    fn publish(&self, window: &Showcase) {
        let global = window.global::<FormExamples>();
        global.set_accepted_date(self.date.into());
        global.set_accepted_time(self.time.into());
    }
}
pub fn install(window: &Showcase) {
    let date = CivilDate::new(2026, 10, 16).expect("example date");
    let time = LocalTime::new(9, 30, 0).expect("example time");
    let state = Rc::new(RefCell::new(Schedule {
        date,
        month: date.year_month(),
        year: Year::new(2026).unwrap(),
        week: IsoWeek::new(2026, 42).unwrap(),
        time,
        local: LocalDateTime::new(date, time),
        zoned: None,
    }));
    let key = |name| FieldSpecKey::new(name).expect("nonempty codec key");
    slint_kit::install_field_specs(
        window,
        [
            FieldSpec::date().named(key("schedule-date")),
            FieldSpec::month().named(key("schedule-month")),
            FieldSpec::year().named(key("schedule-year")),
            FieldSpec::iso_week().named(key("schedule-week")),
            FieldSpec::time(
                TimePrecision::Minute,
                slint_kit::ClockFormat::TwentyFourHour,
            )
            .named(key("schedule-time24")),
            FieldSpec::time(TimePrecision::Minute, slint_kit::ClockFormat::TwelveHour)
                .named(key("schedule-time12")),
        ],
    )
    .expect("unique schedule codec keys");
    let global = window.global::<FormExamples>();
    global.set_date_text(date.iso_string().into());
    global.set_month_text("2026-10".into());
    global.set_year_text("2026".into());
    global.set_week_text("2026-W42".into());
    global.set_time_text(
        time.format(
            TimePrecision::Minute,
            slint_kit::ClockFormat::TwentyFourHour,
        )
        .into(),
    );
    state.borrow().publish(window);
    let shared = state.clone();
    global.on_time_format_changed(event(window, move |window, format: ClockFormat| {
        let global = window.global::<FormExamples>();
        // A format preference only changes after the current native draft is a valid time.
        let time = match LocalTime::parse_with_format(
            &global.get_time_text(),
            TimePrecision::Minute,
            global.get_time_format().into(),
        ) {
            DraftValue::Parsed(time) => time,
            DraftValue::Empty | DraftValue::Incomplete | DraftValue::Invalid(_) => {
                global.set_time_error(
                    "Finish entering a valid time before changing its format.".into(),
                );
                return;
            }
        };
        shared.borrow_mut().time = time;
        global.set_time_format(format);
        global.set_time_text(time.format(TimePrecision::Minute, format.into()).into());
        global.set_time_error("".into());
        shared.borrow().publish(&window);
    }));
    let shared = state.clone();
    global.on_date_commit(event2(window, move |window, field, commit| {
        let global = window.global::<FormExamples>();
        let mut state = shared.borrow_mut();
        let accepted = match field {
            DateExample::Date => FieldSpec::date()
                .accept(commit)
                .ok()
                .flatten()
                .map(|value| {
                    state.date = value;
                    global.set_date_error("".into());
                }),
            DateExample::Month => FieldSpec::month()
                .accept(commit)
                .ok()
                .flatten()
                .map(|value| {
                    state.month = value;
                    global.set_month_error("".into());
                }),
            DateExample::Year => FieldSpec::year()
                .accept(commit)
                .ok()
                .flatten()
                .map(|value| {
                    state.year = value;
                    global.set_year_error("".into());
                }),
            DateExample::Week => FieldSpec::iso_week()
                .accept(commit)
                .ok()
                .flatten()
                .map(|value| {
                    state.week = value;
                    global.set_week_error("".into());
                }),
            DateExample::Time => {
                FieldSpec::time(TimePrecision::Minute, global.get_time_format().into())
                    .accept(commit)
                    .ok()
                    .flatten()
                    .map(|value| {
                        state.time = value;
                        global.set_time_error("".into());
                    })
            }
        };
        if accepted.is_none() {
            return;
        }
        state.publish(&window);
    }));
    let shared = state.clone();
    global.on_calendar_chosen(event(window, move |window, value| {
        if let Ok(date) = CivilDate::try_from(value) {
            shared.borrow_mut().date = date;
            window.global::<FormExamples>().set_date_error("".into());
            shared.borrow().publish(&window);
        }
    }));
    global.on_date_range_chosen(event(window, |window, value| {
        if let Ok(range) = DateRange::try_from(value) {
            window.global::<FormExamples>().set_date_range_summary(
                format!(
                    "{} → {} · {} nights",
                    range.start().iso_string(),
                    range.end().iso_string(),
                    range.nights()
                )
                .into(),
            );
        }
    }));
    let shared = state.clone();
    global.on_local_chosen(event(window, move |window, value| {
        if let Ok(value) = LocalDateTime::try_from(value) {
            shared.borrow_mut().local = value;
            window.global::<FormExamples>().set_local_error("".into());
        }
    }));
    global.on_zone_chosen(event(window, move |window, value| {
        if let Ok(value) = ResolvedDateTime::try_from(value) {
            state.borrow_mut().zoned = Some(value);
            window.global::<FormExamples>().set_zone_message(
                format!(
                    "Accepted instant: {} seconds since Unix epoch.",
                    value.instant.unix_seconds()
                )
                .into(),
            );
        }
    }));
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Showcase {
        i_slint_backend_testing::init_no_event_loop();
        let window = Showcase::new().unwrap();
        slint_kit::install(&window);
        install(&window);
        window.set_page(ShowcasePage::Schedule);
        window.show().unwrap();
        window
    }
    fn commit(text: &str, kind: HtmlInputKind) -> FieldCommit {
        FieldCommit {
            value: FieldValue {
                present: true,
                text: text.into(),
            },
            constraints: KitInputConstraints {
                kind,
                min_length: -1,
                max_length: -1,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    #[test]
    fn direct_submission_guards_date_and_time_payloads() {
        let window = fixture();
        let form = window.global::<FormExamples>();
        form.invoke_date_commit(DateExample::Date, commit("2026-02-29", HtmlInputKind::Date));
        assert_eq!(form.get_accepted_date().month, 10);
        form.invoke_date_commit(DateExample::Time, commit("24:30", HtmlInputKind::Time));
        assert_eq!(form.get_accepted_time().hour, 9);
        form.invoke_date_commit(DateExample::Date, commit("2024-02-29", HtmlInputKind::Date));
        assert_eq!(form.get_accepted_date().day, 29);
    }
    #[test]
    fn time_format_preference_preserves_incomplete_native_drafts() {
        let window = fixture();
        let form = window.global::<FormExamples>();
        form.set_time_text("1:".into());
        form.invoke_time_format_changed(ClockFormat::TwelveHour);
        assert_eq!(form.get_time_text(), "1:");
        assert_eq!(form.get_time_format(), ClockFormat::TwentyFourHour);
        form.set_time_text("12:00".into());
        form.invoke_time_format_changed(ClockFormat::TwelveHour);
        assert_eq!(form.get_time_text(), "12:00 PM");
    }
    #[test]
    fn calendar_business_callbacks_accept_dates_and_ranges_independently() {
        let window = fixture();
        let form = window.global::<FormExamples>();
        form.invoke_calendar_chosen(CivilDateView {
            year: 2026,
            month: 10,
            day: 19,
        });
        form.invoke_date_range_chosen(DateRangeView {
            start: CivilDateView {
                year: 2026,
                month: 10,
                day: 20,
            },
            end: CivilDateView {
                year: 2026,
                month: 10,
                day: 22,
            },
        });
        assert_eq!(form.get_accepted_date().day, 19);
        assert!(form.get_date_range_summary().contains("2026-10-20"));
    }
}

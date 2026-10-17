// SPDX-License-Identifier: GPL-3.0-or-later
//! Civil drafts and IANA resolution stay separate from accepted instants.
use super::ui::*;
use kit::{
    CalendarController, CalendarMove, CivilDate, DateRangeController, DraftValue, IsoWeek,
    LocalDateTime, LocalTime, RangeSelection, TimePrecision, TimeZoneId, WeekStart, Year,
    YearMonth, ZonedResolution,
};
use slint::ComponentHandle;
use std::{cell::RefCell, rc::Rc};

struct Schedule {
    calendar: CalendarController,
    range_calendar: CalendarController,
    range: DateRangeController,
    date: CivilDate,
    month: YearMonth,
    year: Year,
    week: IsoWeek,
    time: LocalTime,
    clock_format: kit::ClockFormat,
    local: LocalDateTime,
    zoned: Option<kit::Instant>,
    ambiguity: Option<(ZonedDateTimeDraft, kit::Instant, kit::Instant)>,
}
fn view(date: CivilDate) -> CivilDateView {
    CivilDateView {
        year: date.year(),
        month: date.month() as i32,
        day: date.day() as i32,
    }
}
fn time_view(time: LocalTime) -> LocalTimeView {
    LocalTimeView {
        hour: time.hour() as i32,
        minute: time.minute() as i32,
        second: time.second() as i32,
    }
}
fn time_text(time: LocalTime) -> String {
    format!("{:02}:{:02}", time.hour(), time.minute())
}
fn from_day(day: &CalendarDay) -> Option<CivilDate> {
    if !day.enabled {
        return None;
    }
    CivilDate::new(
        day.year,
        u32::try_from(day.month).ok()?,
        u32::try_from(day.day).ok()?,
    )
    .ok()
}
fn navigation(step: CalendarNavigation) -> CalendarMove {
    match step {
        CalendarNavigation::PreviousDay => CalendarMove::PreviousDay,
        CalendarNavigation::NextDay => CalendarMove::NextDay,
        CalendarNavigation::PreviousWeek => CalendarMove::PreviousWeek,
        CalendarNavigation::NextWeek => CalendarMove::NextWeek,
        CalendarNavigation::WeekStart => CalendarMove::WeekStart,
        CalendarNavigation::WeekEnd => CalendarMove::WeekEnd,
        CalendarNavigation::PreviousMonth => CalendarMove::PreviousMonth,
        CalendarNavigation::NextMonth => CalendarMove::NextMonth,
        CalendarNavigation::PreviousYear => CalendarMove::PreviousYear,
        CalendarNavigation::NextYear => CalendarMove::NextYear,
    }
}
impl Schedule {
    fn publish(&self, window: &Showcase) {
        let global = window.global::<FormExamples>();
        let draft = self.range.draft();
        let slots = self.calendar.grid_slots();
        let rows: Vec<_> = slots
            .iter()
            .map(|slot| {
                slot.map_or_else(CalendarDay::default, |cell| CalendarDay {
                    year: cell.date.year(),
                    month: cell.date.month() as i32,
                    day: cell.date.day() as i32,
                    label: cell.date.day().to_string().into(),
                    accessible_label: cell.date.iso_string().into(),
                    enabled: cell.enabled,
                    selected: cell.selected,
                    in_range: false,
                    today: false,
                })
            })
            .collect();
        let range_slots = self.range_calendar.grid_slots();
        let range_rows: Vec<_> = range_slots
            .iter()
            .map(|slot| {
                let mut row = slot.map_or_else(CalendarDay::default, |cell| CalendarDay {
                    year: cell.date.year(),
                    month: cell.date.month() as i32,
                    day: cell.date.day() as i32,
                    label: cell.date.day().to_string().into(),
                    accessible_label: cell.date.iso_string().into(),
                    enabled: cell.enabled,
                    selected: false,
                    in_range: false,
                    today: false,
                });
                if let Some(cell) = slot {
                    match draft {
                        RangeSelection::ChoosingStart => {
                            row.selected = false;
                        }
                        RangeSelection::ChoosingEnd { start } => {
                            row.selected = cell.date == start;
                        }
                        RangeSelection::Complete(range) => {
                            row.selected = cell.date == range.start() || cell.date == range.end();
                            row.in_range = range.contains(cell.date);
                        }
                    }
                }
                row
            })
            .collect();
        global.set_calendar_active(
            slots
                .iter()
                .position(|slot| slot.is_some_and(|cell| cell.focused))
                .unwrap_or(0) as i32,
        );
        global.set_calendar_days(slint::ModelRc::new(slint::VecModel::from(rows)));
        global.set_range_days(slint::ModelRc::new(slint::VecModel::from(range_rows)));
        let month = self.calendar.displayed();
        const MONTHS: [&str; 12] = [
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
        global.set_calendar_title(
            format!("{} {}", MONTHS[month.month() as usize - 1], month.year()).into(),
        );
        let range_month = self.range_calendar.displayed();
        global.set_range_title(
            format!(
                "{} {}",
                MONTHS[range_month.month() as usize - 1],
                range_month.year()
            )
            .into(),
        );
        global.set_range_active(
            range_slots
                .iter()
                .position(|slot| slot.is_some_and(|cell| cell.focused))
                .unwrap_or(0) as i32,
        );
        global.set_accepted_date(view(self.date));
        global.set_accepted_time(time_view(self.time));
        global.set_range_phase(match draft {
            RangeSelection::ChoosingStart => DateRangePhase::ChoosingStart,
            RangeSelection::ChoosingEnd { .. } => DateRangePhase::ChoosingEnd,
            RangeSelection::Complete(_) => DateRangePhase::Complete,
        });
        global.set_date_range_summary(match draft {
            RangeSelection::ChoosingStart => "Choose a start date, then an end date.".into(),
            RangeSelection::ChoosingEnd { start } => {
                format!("{} → choose an end date", start.iso_string()).into()
            }
            RangeSelection::Complete(range) => format!(
                "{} → {}",
                range.start().iso_string(),
                range.end().iso_string()
            )
            .into(),
        });
    }
}
fn parsed<T>(draft: DraftValue<T>) -> Result<T, &'static str> {
    match draft {
        DraftValue::Parsed(value) => Ok(value),
        DraftValue::Empty => Err("Enter a value."),
        DraftValue::Incomplete => Err("Finish entering this value."),
        DraftValue::Invalid(_) => Err("Enter a valid value in the format shown."),
    }
}
fn local(date: &str, time: &str) -> Result<LocalDateTime, &'static str> {
    Ok(LocalDateTime::new(
        parsed(CivilDate::parse_iso(date))?,
        parsed(LocalTime::parse(time, TimePrecision::Minute))?,
    ))
}
fn pair(text: &str, separator: &str) -> Option<(i32, u32)> {
    let (year, part) = text.split_once(separator)?;
    if year.len() != 4
        || !year.bytes().all(|c| c.is_ascii_digit())
        || part.len() != 2
        || !part.bytes().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    Some((year.parse().ok()?, part.parse().ok()?))
}
pub fn install(window: &Showcase) {
    let date = CivilDate::new(2026, 10, 16).expect("valid example date");
    let time = LocalTime::new(9, 30, 0).expect("valid example time");
    let state = Rc::new(RefCell::new(Schedule {
        calendar: CalendarController::new(Some(date), date.year_month(), None, WeekStart::Monday),
        range_calendar: CalendarController::new(
            Some(date),
            date.year_month(),
            None,
            WeekStart::Monday,
        ),
        range: DateRangeController::new(None),
        date,
        month: date.year_month(),
        year: Year::new(2026).unwrap(),
        week: IsoWeek::new(2026, 42).unwrap(),
        time,
        clock_format: kit::ClockFormat::TwentyFourHour,
        local: LocalDateTime::new(date, time),
        zoned: None,
        ambiguity: None,
    }));
    let global = window.global::<FormExamples>();
    global.set_date_text(date.iso_string().into());
    global.set_month_text("2026-10".into());
    global.set_year_text("2026".into());
    global.set_week_text("2026-W42".into());
    global.set_time_text(time_text(time).into());
    global.set_zone_message(
        "Resolve the local time explicitly; daylight saving transitions may require a choice."
            .into(),
    );
    state.borrow().publish(window);
    // Draft text is two-way bound to native editors. Editing never modifies an accepted civil value.
    global.on_date_edited(|_, _| {});
    let weak = window.as_weak();
    let shared = state.clone();
    global.on_time_format_changed(move |format| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let global = window.global::<FormExamples>();
        let mut state = shared.borrow_mut();
        let Ok(time) = parsed(LocalTime::parse_with_format(
            &global.get_time_text(),
            TimePrecision::Minute,
            state.clock_format,
        )) else {
            global
                .set_time_error("Finish entering a valid time before changing its format.".into());
            return;
        };
        state.clock_format = match format {
            ClockFormat::TwentyFourHour => kit::ClockFormat::TwentyFourHour,
            ClockFormat::TwelveHour => kit::ClockFormat::TwelveHour,
        };
        state.time = time;
        global.set_time_format(format);
        global.set_time_text(
            time.format(TimePrecision::Minute, state.clock_format)
                .into(),
        );
        global.set_time_error("".into());
        state.publish(&window);
    });
    let weak = window.as_weak();
    let shared = state.clone();
    global.on_date_commit(move |field, text| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let global = window.global::<FormExamples>();
        let mut state = shared.borrow_mut();
        let result: Result<(), &str> = match field {
            DateExample::Date => parsed(CivilDate::parse_iso(&text)).map(|value| {
                state.date = value;
                let _ = state.calendar.choose(value);
                global.set_date_text(value.iso_string().into());
            }),
            DateExample::Month => pair(&text, "-")
                .and_then(|(year, month)| YearMonth::new(year, month).ok())
                .ok_or("Use YYYY-MM with a valid month.")
                .map(|value| {
                    state.month = value;
                    global
                        .set_month_text(format!("{:04}-{:02}", value.year(), value.month()).into());
                }),
            DateExample::Year => if text.len() == 4 && text.bytes().all(|c| c.is_ascii_digit()) {
                text.parse().ok().and_then(|year| Year::new(year).ok())
            } else {
                None
            }
            .ok_or("Use a year from 0001 to 9999.")
            .map(|value| {
                state.year = value;
                global.set_year_text(format!("{:04}", value.get()).into());
            }),
            DateExample::Week => pair(&text, "-W")
                .and_then(|(year, week)| IsoWeek::new(year, week).ok())
                .ok_or("Use YYYY-Www with a valid ISO week.")
                .map(|value| {
                    state.week = value;
                    global.set_week_text(
                        format!("{:04}-W{:02}", value.year().get(), value.week()).into(),
                    );
                }),
            DateExample::Time => parsed(LocalTime::parse_with_format(
                &text,
                TimePrecision::Minute,
                state.clock_format,
            ))
            .map(|value| {
                state.time = value;
                global.set_time_text(
                    value
                        .format(TimePrecision::Minute, state.clock_format)
                        .into(),
                );
            }),
        };
        let error = result.err().unwrap_or("").into();
        match field {
            DateExample::Date => global.set_date_error(error),
            DateExample::Month => global.set_month_error(error),
            DateExample::Year => global.set_year_error(error),
            DateExample::Week => global.set_week_error(error),
            DateExample::Time => global.set_time_error(error),
        }
        state.publish(&window);
    });
    let weak = window.as_weak();
    let shared = state.clone();
    global.on_date_range_navigate(move |step| {
        if let Some(window) = weak.upgrade() {
            let mut state = shared.borrow_mut();
            let _ = state.range_calendar.navigate(navigation(step));
            state.publish(&window);
        }
    });
    let weak = window.as_weak();
    let shared = state.clone();
    global.on_calendar_navigate(move |step| {
        if let Some(window) = weak.upgrade() {
            let mut state = shared.borrow_mut();
            let _ = state.calendar.navigate(navigation(step));
            state.publish(&window);
        }
    });
    let weak = window.as_weak();
    let shared = state.clone();
    global.on_calendar_choose(move |day| {
        if let (Some(window), Some(date)) = (weak.upgrade(), from_day(&day)) {
            let mut state = shared.borrow_mut();
            if state.calendar.choose(date).is_ok() {
                state.date = date;
                window
                    .global::<FormExamples>()
                    .set_date_text(date.iso_string().into());
                window.global::<FormExamples>().set_date_error("".into());
                state.publish(&window);
            }
        }
    });
    let weak = window.as_weak();
    let shared = state.clone();
    global.on_date_range_choose(move |day| {
        if let (Some(window), Some(date)) = (weak.upgrade(), from_day(&day)) {
            let mut state = shared.borrow_mut();
            let _ = state.range_calendar.choose(date);
            let error = if state.range.choose(date).is_err() {
                "The end must be on or after the start."
            } else {
                ""
            };
            window
                .global::<FormExamples>()
                .set_date_range_error(error.into());
            state.publish(&window);
        }
    });
    let weak = window.as_weak();
    let shared = state.clone();
    global.on_date_range_apply(move || {
        if let Some(window) = weak.upgrade() {
            let mut state = shared.borrow_mut();
            let error = if state.range.apply().is_some() {
                ""
            } else {
                "Choose both dates before applying."
            };
            window
                .global::<FormExamples>()
                .set_date_range_error(error.into());
            state.publish(&window);
        }
    });
    let weak = window.as_weak();
    let shared = state.clone();
    global.on_date_range_cancel(move || {
        if let Some(window) = weak.upgrade() {
            let mut state = shared.borrow_mut();
            state.range.cancel();
            window
                .global::<FormExamples>()
                .set_date_range_error("".into());
            state.publish(&window);
        }
    });
    let weak = window.as_weak();
    let shared = state.clone();
    global.on_time_adjust(move |segment, direction| {
        if let Some(window) = weak.upgrade() {
            let global = window.global::<FormExamples>();
            let mut state = shared.borrow_mut();
            let Ok(time) = parsed(LocalTime::parse_with_format(
                &global.get_time_text(),
                TimePrecision::Minute,
                state.clock_format,
            )) else {
                global.set_time_error("Finish entering a valid time before adjusting it.".into());
                return;
            };
            let delta = match segment {
                TimeSegment::Hour => 3600,
                TimeSegment::Minute => 60,
                TimeSegment::Second => 1,
            } * match direction {
                TimeAdjustment::Increment => 1,
                TimeAdjustment::Decrement => -1,
            };
            let seconds = (time.hour() as i32 * 3600
                + time.minute() as i32 * 60
                + time.second() as i32
                + delta)
                .rem_euclid(86400) as u32;
            state.time = LocalTime::new(seconds / 3600, seconds / 60 % 60, seconds % 60)
                .expect("normalized time");
            global.set_time_text(
                state
                    .time
                    .format(TimePrecision::Minute, state.clock_format)
                    .into(),
            );
            global.set_time_error("".into());
            state.publish(&window);
        }
    });
    let weak = window.as_weak();
    let shared = state.clone();
    global.on_local_commit(move |draft| {
        if let Some(window) = weak.upgrade() {
            match local(&draft.date, &draft.time) {
                Ok(value) => {
                    shared.borrow_mut().local = value;
                    window.global::<FormExamples>().set_local_error("".into());
                }
                Err(error) => window
                    .global::<FormExamples>()
                    .set_local_error(error.into()),
            }
        }
    });
    let weak = window.as_weak();
    let shared = state.clone();
    global.on_zone_edited(move || {
        if let Some(window) = weak.upgrade() {
            shared.borrow_mut().ambiguity = None;
            window
                .global::<FormExamples>()
                .set_zone_resolution(ZonedTimePresentation::Unresolved);
            window.global::<FormExamples>().set_zone_message("".into());
        }
    });
    let weak = window.as_weak();
    let shared = state.clone();
    global.on_zone_resolve(move |draft| {let Some(window)=weak.upgrade()else{return;};let global=window.global::<FormExamples>();let mut state=shared.borrow_mut();state.ambiguity=None;
        let local=match local(&draft.date,&draft.time){Ok(value)=>value,Err(error)=>{global.set_zone_resolution(ZonedTimePresentation::Unresolved);global.set_zone_message(error.into());return;}};
        let zone=match TimeZoneId::new(&draft.zone){Ok(value)=>value,Err(_)=>{global.set_zone_resolution(ZonedTimePresentation::InvalidZone);global.set_zone_message("Use a valid IANA time zone, such as America/Toronto.".into());return;}};
        match zone.resolve(local){
            ZonedResolution::Unique(instant)=>{state.zoned=Some(instant);global.set_zone_resolution(ZonedTimePresentation::Unique);global.set_zone_message(format!("Resolved instant: {} seconds since Unix epoch.",instant.unix_seconds()).into());},
            ZonedResolution::Nonexistent=>{global.set_zone_resolution(ZonedTimePresentation::Nonexistent);global.set_zone_message("This local time does not exist during the daylight saving jump. Choose another time.".into());},
            ZonedResolution::Ambiguous{earlier,later}=>{state.ambiguity=Some((draft,earlier,later));global.set_zone_resolution(ZonedTimePresentation::Ambiguous);global.set_zone_message("This local time occurs twice. Choose the earlier or later occurrence.".into());},
        }
    });
    let weak = window.as_weak();
    global.on_ambiguity_chosen(move |choice| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let global = window.global::<FormExamples>();
        let mut state = state.borrow_mut();
        let Some((draft, earlier, later)) = state.ambiguity.take() else {
            return;
        };
        if draft.date != global.get_zone_date_text()
            || draft.time != global.get_zone_time_text()
            || draft.zone != global.get_zone_text()
        {
            global.set_zone_resolution(ZonedTimePresentation::Unresolved);
            return;
        }
        let instant = match choice {
            AmbiguousTimeChoice::Earlier => earlier,
            AmbiguousTimeChoice::Later => later,
        };
        state.zoned = Some(instant);
        global.set_zone_resolution(ZonedTimePresentation::Unique);
        global.set_zone_message(
            format!(
                "Resolved instant: {} seconds since Unix epoch.",
                instant.unix_seconds()
            )
            .into(),
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Showcase {
        i_slint_backend_testing::init_no_event_loop();
        let window = Showcase::new().unwrap();
        install(&window);
        window.set_page(ShowcasePage::Schedule);
        window.show().unwrap();
        window
    }
    #[test]
    fn impossible_date_and_time_keep_the_last_accepted_civil_values() {
        let window = fixture();
        let form = window.global::<FormExamples>();
        form.set_date_text("2026-02-29".into());
        form.invoke_date_commit(DateExample::Date, "2026-02-29".into());
        assert_eq!(form.get_accepted_date().month, 10);
        assert_eq!(form.get_date_text().as_str(), "2026-02-29");
        assert!(!form.get_date_error().is_empty());
        form.set_time_text("24:30".into());
        form.invoke_date_commit(DateExample::Time, "24:30".into());
        assert_eq!(form.get_accepted_time().hour, 9);
        assert_eq!(form.get_time_text().as_str(), "24:30");
        form.invoke_date_commit(DateExample::Date, "2024-02-29".into());
        assert_eq!(form.get_accepted_date().day, 29);
        assert!(form.get_date_error().is_empty());
    }
    #[test]
    fn time_format_change_preserves_invalid_draft_and_maps_noon_explicitly() {
        let window = fixture();
        let form = window.global::<FormExamples>();
        form.set_time_text("1:".into());
        form.invoke_time_format_changed(ClockFormat::TwelveHour);
        assert_eq!(form.get_time_text().as_str(), "1:");
        assert_eq!(form.get_time_format(), ClockFormat::TwentyFourHour);
        form.set_time_text("12:00".into());
        form.invoke_time_format_changed(ClockFormat::TwelveHour);
        assert_eq!(form.get_time_text().as_str(), "12:00 PM");
        form.invoke_date_commit(DateExample::Time, "12:00 AM".into());
        assert_eq!(form.get_accepted_time().hour, 0);
    }
    #[test]
    fn dst_fold_requires_an_explicit_current_choice_and_gap_never_resolves() {
        let window = fixture();
        let form = window.global::<FormExamples>();
        form.invoke_zone_resolve(ZonedDateTimeDraft {
            date: "2026-11-01".into(),
            time: "01:30".into(),
            zone: "America/Toronto".into(),
        });
        assert_eq!(form.get_zone_resolution(), ZonedTimePresentation::Ambiguous);
        form.invoke_ambiguity_chosen(AmbiguousTimeChoice::Later);
        assert_eq!(form.get_zone_resolution(), ZonedTimePresentation::Unique);
        form.invoke_zone_resolve(ZonedDateTimeDraft {
            date: "2026-03-08".into(),
            time: "02:30".into(),
            zone: "America/Toronto".into(),
        });
        assert_eq!(
            form.get_zone_resolution(),
            ZonedTimePresentation::Nonexistent
        );
        form.invoke_ambiguity_chosen(AmbiguousTimeChoice::Earlier);
        assert_eq!(
            form.get_zone_resolution(),
            ZonedTimePresentation::Nonexistent
        );
        form.invoke_zone_resolve(ZonedDateTimeDraft {
            date: "2026-11-01".into(),
            time: "01:30".into(),
            zone: "America/Toronto".into(),
        });
        form.set_zone_time_text("03:00".into());
        form.invoke_ambiguity_chosen(AmbiguousTimeChoice::Earlier);
        assert_eq!(
            form.get_zone_resolution(),
            ZonedTimePresentation::Unresolved
        );
    }
    #[test]
    fn date_and_range_calendars_navigate_independently() {
        let window = fixture();
        let form = window.global::<FormExamples>();
        let date_month = form.get_calendar_title();
        let range_month = form.get_range_title();
        form.invoke_calendar_navigate(CalendarNavigation::NextMonth);
        assert_ne!(form.get_calendar_title(), date_month);
        assert_eq!(form.get_range_title(), range_month);
        let date_month = form.get_calendar_title();
        form.invoke_date_range_navigate(CalendarNavigation::PreviousMonth);
        assert_ne!(form.get_range_title(), range_month);
        assert_eq!(form.get_calendar_title(), date_month);
    }
}

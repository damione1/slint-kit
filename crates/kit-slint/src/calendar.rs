// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::ui;
use kit_core::{
    CalendarController, CalendarMove, CalendarState, CivilDate, DateConstraints,
    DateDisplayPattern, DateNames, DatePattern, DateRange, DateRangeController, DateRangeRules,
    DraftValue, EarlierEndPolicy, Instant, NumericBounds, ParseIssue, RangeRejection,
    RangeSelection, TimeZoneId, WeekStart, YearMonth,
};
use slint::{Model, ModelRc, VecModel};

impl From<CivilDate> for ui::CivilDateView {
    fn from(date: CivilDate) -> Self {
        Self {
            year: date.year(),
            month: i32::try_from(date.month()).expect("month fits i32"),
            day: i32::try_from(date.day()).expect("day fits i32"),
        }
    }
}
impl TryFrom<ui::CivilDateView> for CivilDate {
    type Error = ParseIssue;
    fn try_from(date: ui::CivilDateView) -> Result<Self, Self::Error> {
        Self::new(
            date.year,
            u32::try_from(date.month).map_err(|_| ParseIssue::InvalidDate)?,
            u32::try_from(date.day).map_err(|_| ParseIssue::InvalidDate)?,
        )
    }
}
impl TryFrom<ui::CalendarDay> for CivilDate {
    type Error = ParseIssue;
    fn try_from(day: ui::CalendarDay) -> Result<Self, Self::Error> {
        Self::try_from(ui::CivilDateView {
            year: day.year,
            month: day.month,
            day: day.day,
        })
    }
}
impl From<ui::CalendarNavigation> for CalendarMove {
    fn from(value: ui::CalendarNavigation) -> Self {
        match value {
            ui::CalendarNavigation::PreviousDay => Self::PreviousDay,
            ui::CalendarNavigation::NextDay => Self::NextDay,
            ui::CalendarNavigation::PreviousWeek => Self::PreviousWeek,
            ui::CalendarNavigation::NextWeek => Self::NextWeek,
            ui::CalendarNavigation::WeekStart => Self::WeekStart,
            ui::CalendarNavigation::WeekEnd => Self::WeekEnd,
            ui::CalendarNavigation::PreviousMonth => Self::PreviousMonth,
            ui::CalendarNavigation::NextMonth => Self::NextMonth,
            ui::CalendarNavigation::PreviousYear => Self::PreviousYear,
            ui::CalendarNavigation::NextYear => Self::NextYear,
        }
    }
}
impl From<ui::WeekStart> for WeekStart {
    fn from(value: ui::WeekStart) -> Self {
        match value {
            ui::WeekStart::Monday => Self::Monday,
            ui::WeekStart::Tuesday => Self::Tuesday,
            ui::WeekStart::Wednesday => Self::Wednesday,
            ui::WeekStart::Thursday => Self::Thursday,
            ui::WeekStart::Friday => Self::Friday,
            ui::WeekStart::Saturday => Self::Saturday,
            ui::WeekStart::Sunday => Self::Sunday,
        }
    }
}
impl From<ui::EarlierEndPolicy> for EarlierEndPolicy {
    fn from(value: ui::EarlierEndPolicy) -> Self {
        match value {
            ui::EarlierEndPolicy::Restart => Self::Restart,
            ui::EarlierEndPolicy::Swap => Self::Swap,
            ui::EarlierEndPolicy::Reject => Self::Reject,
        }
    }
}
impl TryFrom<ui::DateRangeView> for DateRange {
    type Error = ParseIssue;
    fn try_from(range: ui::DateRangeView) -> Result<Self, Self::Error> {
        Self::new(range.start.try_into()?, range.end.try_into()?)
    }
}
impl From<DateRange> for ui::DateRangeView {
    fn from(range: DateRange) -> Self {
        Self {
            start: range.start().into(),
            end: range.end().into(),
        }
    }
}
impl TryFrom<ui::DateRangeDraft> for RangeSelection {
    type Error = ParseIssue;
    fn try_from(draft: ui::DateRangeDraft) -> Result<Self, Self::Error> {
        match draft.phase {
            ui::DateRangePhase::ChoosingStart => Ok(Self::ChoosingStart),
            ui::DateRangePhase::ChoosingEnd => Ok(Self::ChoosingEnd {
                start: draft.start.try_into()?,
            }),
            ui::DateRangePhase::Complete => Ok(Self::Complete(DateRange::new(
                draft.start.try_into()?,
                draft.end.try_into()?,
            )?)),
        }
    }
}
impl From<ParseIssue> for ui::DateRangeRejection {
    /// A value the host supplied could not be parsed: the configuration is invalid.
    fn from(_: ParseIssue) -> Self {
        Self::InvalidConfiguration
    }
}
impl From<RangeRejection> for ui::DateRangeRejection {
    fn from(rejection: RangeRejection) -> Self {
        match rejection {
            RangeRejection::UnavailableDate => Self::UnavailableDate,
            RangeRejection::EndBeforeStart => Self::EndBeforeStart,
            RangeRejection::BlockedInterior => Self::BlockedInterior,
            RangeRejection::TooShort => Self::TooShort,
            RangeRejection::TooLong => Self::TooLong,
        }
    }
}
impl From<RangeSelection> for ui::DateRangeDraft {
    fn from(draft: RangeSelection) -> Self {
        match draft {
            RangeSelection::ChoosingStart => Self::default(),
            RangeSelection::ChoosingEnd { start } => Self {
                phase: ui::DateRangePhase::ChoosingEnd,
                start: start.into(),
                ..Default::default()
            },
            RangeSelection::Complete(range) => Self {
                phase: ui::DateRangePhase::Complete,
                start: range.start().into(),
                end: range.end().into(),
            },
        }
    }
}
fn optional_date(date: ui::CivilDateView) -> Result<Option<CivilDate>, ParseIssue> {
    if date == ui::CivilDateView::default() {
        Ok(None)
    } else {
        CivilDate::try_from(date).map(Some)
    }
}
fn bound(text: &str, default: CivilDate) -> Result<CivilDate, ParseIssue> {
    if text.is_empty() {
        return Ok(default);
    }
    match CivilDate::parse_iso(text) {
        DraftValue::Parsed(date) => Ok(date),
        DraftValue::Empty | DraftValue::Incomplete | DraftValue::Invalid(_) => {
            Err(ParseIssue::InvalidDate)
        }
    }
}
fn parse(view: &ui::CalendarView) -> Result<CalendarController, ParseIssue> {
    let minimum = bound(view.min.as_str(), CivilDate::new(1, 1, 1)?)?;
    let maximum = bound(view.max.as_str(), CivilDate::new(9999, 12, 31)?)?;
    let ranges = view
        .unavailable_ranges
        .iter()
        .map(|range| DateRange::new(range.start.try_into()?, range.end.try_into()?))
        .collect::<Result<Vec<_>, ParseIssue>>()?;
    let constraints = DateConstraints::new(Some(NumericBounds::new(minimum, maximum)?))
        .with_unavailable_weekdays(
            view.unavailable_weekdays
                .iter()
                .map(WeekStart::from)
                .collect(),
        )
        .with_unavailable_ranges(ranges);
    CalendarController::from_state(
        CalendarState {
            // The stored value remains host-owned even when availability changes.
            // Invalid old selection must not prevent projecting or choosing a replacement.
            selected: optional_date(view.selected.clone())?
                .filter(|date| constraints.validate(*date).is_ok()),
            focused: view.focused.clone().try_into()?,
        },
        constraints,
        view.week_start.into(),
    )
}
/// Project a validated controller using shared library types. Used by custom,
/// controlled calendars as well as the autonomous service.
pub fn calendar_projection(
    calendar: &CalendarController,
    today: Option<CivilDate>,
    month_names: &ModelRc<slint::SharedString>,
) -> ui::CalendarProjection {
    let slots = calendar.grid_slots();
    let active_index = slots
        .iter()
        .position(|slot| slot.is_some_and(|cell| cell.focused))
        .map_or(-1, |index| i32::try_from(index).expect("42 calendar slots"));
    let days = slots
        .into_iter()
        .map(|slot| {
            slot.map_or_else(ui::CalendarDay::default, |cell| {
                let date: ui::CivilDateView = cell.date.into();
                ui::CalendarDay {
                    year: date.year,
                    month: date.month,
                    day: date.day,
                    label: cell.date.day().to_string().into(),
                    accessible_label: cell.date.iso_string().into(),
                    enabled: cell.enabled,
                    selected: cell.selected,
                    in_range: false,
                    today: today == Some(cell.date),
                    outside_month: !cell.in_displayed_month,
                    marker: ui::CalendarMarker::None,
                    caption: slint::SharedString::default(),
                }
            })
        })
        .collect::<Vec<_>>();
    let month = calendar.displayed();
    let title = month_names
        .row_data(usize::try_from(month.month() - 1).expect("month fits usize"))
        .map_or_else(
            || format!("{}-{:02}", month.year(), month.month()),
            |name| format!("{name} {}", month.year()),
        );
    ui::CalendarProjection {
        configuration_state: ui::CalendarConfigurationState::Valid,
        days: ModelRc::new(VecModel::from(days)),
        active_index,
        title: title.into(),
        previous_enabled: calendar.can_navigate(CalendarMove::PreviousMonth),
        next_enabled: calendar.can_navigate(CalendarMove::NextMonth),
    }
}
fn with_week_rows(
    mut projection: ui::CalendarProjection,
    rows: ui::CalendarWeekRows,
) -> ui::CalendarProjection {
    match rows {
        ui::CalendarWeekRows::Fixed => projection,
        ui::CalendarWeekRows::Variable => {
            let mut days = projection.days.iter().collect::<Vec<_>>();
            let last = days
                .iter()
                .rposition(|day| day.year > 0 && !day.outside_month)
                .expect("a displayed month contains civil dates");
            days.truncate((last + 1).div_ceil(7) * 7);
            projection.days = ModelRc::new(VecModel::from(days));
            projection
        }
    }
}
/// Format an accepted ISO date for display at rest. Any text that is not a
/// complete ISO date, or any invalid pattern or name set, yields `None` so the
/// caller keeps the ISO text.
fn display_date(text: &str, display: &ui::DateDisplay) -> Option<String> {
    fn names<const N: usize>(model: &ModelRc<slint::SharedString>) -> Option<[String; N]> {
        model
            .iter()
            .map(|name| name.to_string())
            .collect::<Vec<_>>()
            .try_into()
            .ok()
    }
    let DraftValue::Parsed(date) = CivilDate::parse_iso(text) else {
        return None;
    };
    let pattern = DateDisplayPattern::new(display.pattern.as_str()).ok()?;
    let names = DateNames::new(
        names(&display.months)?,
        names(&display.short_months)?,
        names(&display.weekdays)?,
        names(&display.short_weekdays)?,
    )
    .ok()?;
    Some(pattern.format_with_names(date, &names))
}
struct CalendarLabels {
    names: DateNames,
    date: DatePattern,
    title: DateDisplayPattern,
    today: slint::SharedString,
    range_start: slint::SharedString,
    range_end: slint::SharedString,
    in_range: slint::SharedString,
}
impl TryFrom<ui::CalendarText> for CalendarLabels {
    type Error = ParseIssue;
    fn try_from(text: ui::CalendarText) -> Result<Self, Self::Error> {
        fn names<const N: usize>(
            model: &ModelRc<slint::SharedString>,
        ) -> Result<[String; N], ParseIssue> {
            model
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| ParseIssue::InvalidSyntax)
        }
        Ok(Self {
            names: DateNames::new(
                names(&text.months)?,
                names(&text.short_months)?,
                names(&text.weekdays)?,
                names(&text.short_weekdays)?,
            )?,
            date: DatePattern::new(text.date_pattern.as_str())?,
            title: DateDisplayPattern::new(text.title_pattern.as_str())?,
            today: text.today,
            range_start: text.range_start,
            range_end: text.range_end,
            in_range: text.in_range,
        })
    }
}
fn localized_projection(
    view: ui::CalendarView,
    text: ui::CalendarText,
    selection: RangeSelection,
) -> ui::CalendarProjection {
    let Ok(labels) = CalendarLabels::try_from(text.clone()) else {
        return with_week_rows(
            invalid_configuration_projection(&view, &text.months),
            view.week_rows,
        );
    };
    let calendar = parse(&view);
    let mut projection = match &calendar {
        Ok(calendar) => calendar_projection(
            calendar,
            optional_date(view.today.clone()).ok().flatten(),
            &text.months,
        ),
        Err(_) => invalid_configuration_projection(&view, &text.months),
    };
    let reference = match calendar {
        Ok(calendar) => calendar.displayed(),
        Err(_) => CivilDate::try_from(view.focused)
            .unwrap_or_else(|_| CivilDate::new(2000, 1, 1).expect("valid reference date"))
            .year_month(),
    };
    let first =
        CivilDate::new(reference.year(), reference.month(), 1).expect("validated displayed month");
    projection.title = labels.title.format_with_names(first, &labels.names).into();
    projection.days = ModelRc::new(VecModel::from(
        projection
            .days
            .iter()
            .map(|mut day| {
                if let Ok(date) = CivilDate::try_from(day.clone()) {
                    let mut label = labels.date.format_with_names(date, &labels.names);
                    let mut append = |suffix: &str| {
                        if !suffix.is_empty() {
                            label.push_str(", ");
                            label.push_str(suffix);
                        }
                    };
                    if day.today {
                        append(&labels.today);
                    }
                    match selection {
                        RangeSelection::ChoosingStart => {}
                        RangeSelection::ChoosingEnd { start } => {
                            day.selected = date == start;
                            if day.selected {
                                append(&labels.range_start);
                            }
                        }
                        RangeSelection::Complete(range) => {
                            day.selected = date == range.start() || date == range.end();
                            day.in_range = range.contains(date);
                            if date == range.start() {
                                append(&labels.range_start);
                            }
                            if date == range.end() {
                                append(&labels.range_end);
                            }
                            if day.in_range && !day.selected {
                                append(&labels.in_range);
                            }
                        }
                    }
                    if let Some(mark) = view.marks.iter().find(|mark| {
                        CivilDate::try_from(mark.date.clone()).is_ok_and(|marked| marked == date)
                    }) {
                        day.marker = mark.marker;
                        day.caption = mark.caption.clone();
                        append(&mark.description);
                    }
                    day.accessible_label = label.into();
                }
                day
            })
            .collect::<Vec<_>>(),
    ));
    with_week_rows(projection, view.week_rows)
}
fn invalid_configuration_projection(
    view: &ui::CalendarView,
    names: &ModelRc<slint::SharedString>,
) -> ui::CalendarProjection {
    let focused = CivilDate::try_from(view.focused.clone())
        .unwrap_or_else(|_| CivilDate::new(2000, 1, 1).expect("valid reference date"));
    let calendar =
        CalendarController::new(None, focused.year_month(), None, view.week_start.into());
    let mut projection = calendar_projection(&calendar, None, names);
    projection.configuration_state = ui::CalendarConfigurationState::Invalid;
    projection.active_index = -1;
    projection.previous_enabled = false;
    projection.next_enabled = false;
    projection.days = ModelRc::new(VecModel::from(
        projection
            .days
            .iter()
            .map(|mut day| {
                day.enabled = false;
                day
            })
            .collect::<Vec<_>>(),
    ));
    projection
}

/// Set this window's calendar reference date from an explicit instant and zone.
/// Component-level today values remain independent overrides.
pub fn set_calendar_today_at<W>(
    window: &W,
    zone: TimeZoneId,
    instant: Instant,
) -> Result<CivilDate, ParseIssue>
where
    W: slint::ComponentHandle,
    for<'a> ui::KitCalendarService<'a>: slint::Global<'a, W>,
{
    let date = zone.local_at(instant)?.date;
    window
        .global::<ui::KitCalendarService>()
        .set_today(date.into());
    Ok(date)
}
fn unix_seconds_at(now: web_time::SystemTime) -> Result<i64, ParseIssue> {
    match now.duration_since(web_time::UNIX_EPOCH) {
        Ok(duration) => i64::try_from(duration.as_secs()).map_err(|_| ParseIssue::InvalidTime),
        Err(error) => {
            let duration = error.duration();
            let whole = i64::try_from(duration.as_secs()).map_err(|_| ParseIssue::InvalidTime)?;
            whole
                .checked_add(i64::from(duration.subsec_nanos() != 0))
                .and_then(i64::checked_neg)
                .ok_or(ParseIssue::InvalidTime)
        }
    }
}
/// Refresh the window-local today value using the current clock in an explicit zone.
/// Call again at a date rollover; no timer or operating-system zone is inferred.
pub fn refresh_calendar_today<W>(window: &W, zone: TimeZoneId) -> Result<CivilDate, ParseIssue>
where
    W: slint::ComponentHandle,
    for<'a> ui::KitCalendarService<'a>: slint::Global<'a, W>,
{
    let instant = Instant::from_unix_seconds(unix_seconds_at(web_time::SystemTime::now())?)?;
    set_calendar_today_at(window, zone, instant)
}
fn initial_focus(mut view: ui::CalendarView) -> ui::CivilDateView {
    let candidate = optional_date(view.selected.clone())
        .ok()
        .flatten()
        .or_else(|| optional_date(view.today.clone()).ok().flatten())
        .unwrap_or_else(|| CivilDate::new(2000, 1, 1).expect("valid reference date"));
    view.focused = candidate.into();
    match parse(&view) {
        Ok(calendar) => calendar
            .constraints()
            .nearest_allowed(calendar.focused())
            .unwrap_or(calendar.focused())
            .into(),
        Err(_) => candidate.into(),
    }
}

fn range_rules(
    view: &ui::CalendarView,
    minimum: &str,
    maximum: &str,
) -> Result<DateRangeRules, ParseIssue> {
    let calendar = parse(view)?;
    let minimum = if minimum.is_empty() {
        0
    } else {
        minimum
            .parse::<u32>()
            .map_err(|_| ParseIssue::InvalidRange)?
    };
    let maximum = if maximum.is_empty() {
        u32::MAX
    } else {
        maximum
            .parse::<u32>()
            .map_err(|_| ParseIssue::InvalidRange)?
    };
    Ok(DateRangeRules::new(
        calendar.constraints().clone(),
        Some(NumericBounds::new(minimum, maximum)?),
    ))
}
pub(crate) fn install_calendar<W>(window: &W)
where
    W: slint::ComponentHandle,
    for<'a> ui::KitCalendarService<'a>: slint::Global<'a, W>,
{
    let service = window.global::<ui::KitCalendarService>();
    if service.get_ready() {
        return;
    }
    service.on_initial_focus(initial_focus);
    service.on_range_choose(|view, draft, date, minimum, maximum, earlier_end| {
        let update = || {
            let rules =
                range_rules(&view, &minimum, &maximum)?.with_earlier_end(earlier_end.into());
            let selection = RangeSelection::try_from(draft.clone())?;
            DateRangeController::reduce(selection, rules, date.try_into()?)
                .map_err(ui::DateRangeRejection::from)
        };
        match update() {
            Ok(next) => ui::DateRangeUpdate {
                draft: next.into(),
                rejection: ui::DateRangeRejection::None,
            },
            Err(rejection) => ui::DateRangeUpdate { draft, rejection },
        }
    });
    service.on_range_valid(|view, range, minimum, maximum| {
        let Ok(rules) = range_rules(&view, &minimum, &maximum) else {
            return false;
        };
        DateRange::try_from(range).is_ok_and(|range| rules.validate(range).is_ok())
    });
    service.on_range_project(|view, draft, text| {
        let selection = match RangeSelection::try_from(draft) {
            Ok(selection) => selection,
            Err(_) => RangeSelection::ChoosingStart,
        };
        localized_projection(view, text, selection)
    });
    service.on_parse_date(|text| match CivilDate::parse_iso(text.as_str()) {
        DraftValue::Parsed(date) => date.into(),
        DraftValue::Empty | DraftValue::Incomplete | DraftValue::Invalid(_) => {
            ui::CivilDateView::default()
        }
    });
    service.on_format_date(|date| {
        CivilDate::try_from(date).map_or_else(
            |_| slint::SharedString::default(),
            |date| date.iso_string().into(),
        )
    });
    service.on_display_date(|text, display| {
        display_date(text.as_str(), &display).map_or(text, slint::SharedString::from)
    });
    service
        .on_project(|view, text| localized_projection(view, text, RangeSelection::ChoosingStart));
    service.on_navigate(|view, direction| {
        let Ok(calendar) = parse(&view) else {
            return view.focused;
        };
        CalendarController::reduce(
            calendar.state(),
            calendar.constraints().clone(),
            calendar.week_start(),
            direction.into(),
        )
        .map_or(view.focused, |state| state.focused.into())
    });
    // Month and year pickers jump to any month; the day clamps to that month and
    // the result clamps into the bounds, so an unreachable month comes back
    // as a different month and the caller can disable it.
    service.on_show_month(|view, year, month| {
        let target = u32::try_from(month)
            .map_err(|_| ParseIssue::InvalidDate)
            .and_then(|month| YearMonth::new(year, month));
        match (parse(&view), target) {
            (Ok(mut calendar), Ok(target)) => {
                calendar.show_month(target);
                calendar.focused().into()
            }
            (Err(_), _) | (_, Err(_)) => view.focused,
        }
    });
    service.on_allowed(|view, date| {
        let Ok(calendar) = parse(&view) else {
            return false;
        };
        CivilDate::try_from(date).is_ok_and(|date| calendar.constraints().validate(date).is_ok())
    });
    service.set_ready(true);
}

#[cfg(test)]
mod clock_tests {
    use super::unix_seconds_at;
    #[test]
    fn wall_clock_seconds_floor_fractional_instants_before_and_after_epoch() {
        use web_time::{Duration, UNIX_EPOCH};
        assert_eq!(unix_seconds_at(UNIX_EPOCH).unwrap(), 0);
        assert_eq!(
            unix_seconds_at(UNIX_EPOCH + Duration::from_millis(500)).unwrap(),
            0
        );
        assert_eq!(
            unix_seconds_at(UNIX_EPOCH + Duration::from_millis(1500)).unwrap(),
            1
        );
        assert_eq!(
            unix_seconds_at(UNIX_EPOCH - Duration::from_millis(500)).unwrap(),
            -1
        );
        assert_eq!(
            unix_seconds_at(UNIX_EPOCH - Duration::from_millis(1500)).unwrap(),
            -2
        );
    }
}

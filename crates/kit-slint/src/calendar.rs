// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::ui;
use kit_core::{
    CalendarController, CalendarMove, CalendarState, CivilDate, DateConstraints, DateRange,
    DateRangeController, DateRangeRules, DraftValue, NumericBounds, ParseIssue, RangeSelection,
    WeekStart,
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
            selected: optional_date(view.selected.clone())?,
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
        days: ModelRc::new(VecModel::from(days)),
        active_index,
        title: title.into(),
        previous_enabled: calendar.can_navigate(CalendarMove::PreviousMonth),
        next_enabled: calendar.can_navigate(CalendarMove::NextMonth),
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
    service.on_range_choose(|view, draft, date, minimum, maximum| {
        let update = || {
            let rules = range_rules(&view, &minimum, &maximum)?;
            DateRangeController::reduce(draft.clone().try_into()?, rules, date.try_into()?)
        };
        match update() {
            Ok(next) => ui::DateRangeUpdate {
                draft: next.into(),
                rejected: false,
            },
            Err(_) => ui::DateRangeUpdate {
                draft,
                rejected: true,
            },
        }
    });
    service.on_range_valid(|view, range, minimum, maximum| {
        let Ok(rules) = range_rules(&view, &minimum, &maximum) else {
            return false;
        };
        DateRange::try_from(range).is_ok_and(|range| rules.validate(range).is_ok())
    });
    service.on_range_project(|view, draft, names| {
        let Ok(calendar) = parse(&view) else {
            return ui::CalendarProjection {
                active_index: -1,
                ..Default::default()
            };
        };
        let mut projection = calendar_projection(&calendar, None, &names);
        let Ok(selection) = RangeSelection::try_from(draft) else {
            return projection;
        };
        let rows = projection
            .days
            .iter()
            .map(|mut day| {
                if let Ok(date) = CivilDate::try_from(day.clone()) {
                    match selection {
                        RangeSelection::ChoosingStart => {}
                        RangeSelection::ChoosingEnd { start } => {
                            day.selected = date == start;
                        }
                        RangeSelection::Complete(range) => {
                            day.selected = date == range.start() || date == range.end();
                            day.in_range = range.contains(date);
                        }
                    }
                }
                day
            })
            .collect::<Vec<_>>();
        projection.days = ModelRc::new(VecModel::from(rows));
        projection
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
    service.on_project(|view, names| match parse(&view) {
        Ok(calendar) => {
            calendar_projection(&calendar, optional_date(view.today).ok().flatten(), &names)
        }
        Err(_) => ui::CalendarProjection {
            active_index: -1,
            ..Default::default()
        },
    });
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
    service.on_allowed(|view, date| {
        let Ok(calendar) = parse(&view) else {
            return false;
        };
        CivilDate::try_from(date).is_ok_and(|date| calendar.constraints().validate(date).is_ok())
    });
    service.set_ready(true);
}

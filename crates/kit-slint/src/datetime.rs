// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::{FieldSpec, ui};
use kit_core::{
    CivilDate, ClockFormat, DayPeriodMarkers, DraftValue, Instant, IsoWeek, LocalDateTime,
    LocalTime, ParseIssue, TimePrecision, TimeZoneId, UtcOffset, Year, YearMonth, ZonedResolution,
};
impl From<LocalTime> for ui::LocalTimeView {
    fn from(value: LocalTime) -> Self {
        Self {
            hour: i32::try_from(value.hour()).expect("hour fits"),
            minute: i32::try_from(value.minute()).expect("minute fits"),
            second: i32::try_from(value.second()).expect("second fits"),
        }
    }
}
impl TryFrom<ui::LocalTimeView> for LocalTime {
    type Error = ParseIssue;
    fn try_from(value: ui::LocalTimeView) -> Result<Self, Self::Error> {
        Self::new(
            u32::try_from(value.hour).map_err(|_| ParseIssue::InvalidTime)?,
            u32::try_from(value.minute).map_err(|_| ParseIssue::InvalidTime)?,
            u32::try_from(value.second).map_err(|_| ParseIssue::InvalidTime)?,
        )
    }
}
impl From<LocalDateTime> for ui::LocalDateTimeView {
    fn from(value: LocalDateTime) -> Self {
        Self {
            date: value.date.into(),
            time: value.time.into(),
        }
    }
}
impl TryFrom<ui::LocalDateTimeView> for LocalDateTime {
    type Error = ParseIssue;
    fn try_from(value: ui::LocalDateTimeView) -> Result<Self, Self::Error> {
        Ok(Self::new(value.date.try_into()?, value.time.try_into()?))
    }
}
/// A resolved civil value with its explicit IANA zone and accepted occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedDateTime {
    pub local: LocalDateTime,
    pub zone: TimeZoneId,
    pub instant: Instant,
}
impl ResolvedDateTime {
    /// Offset from UTC that applies to the accepted occurrence.
    pub fn utc_offset(self) -> UtcOffset {
        self.zone.offset_at(self.instant)
    }
}
impl TryFrom<ui::ZonedDateTimeCommit> for ResolvedDateTime {
    type Error = ParseIssue;
    fn try_from(value: ui::ZonedDateTimeCommit) -> Result<Self, Self::Error> {
        let local = value.local.try_into()?;
        let zone = TimeZoneId::new(value.zone.as_str())?;
        let instant = match (zone.resolve(local), value.acceptance) {
            (ZonedResolution::Unique(instant), ui::ZonedAcceptance::Unique) => instant,
            (ZonedResolution::Ambiguous { earlier, .. }, ui::ZonedAcceptance::Earlier) => earlier,
            (ZonedResolution::Ambiguous { later, .. }, ui::ZonedAcceptance::Later) => later,
            (
                ZonedResolution::Nonexistent,
                ui::ZonedAcceptance::Unique
                | ui::ZonedAcceptance::Earlier
                | ui::ZonedAcceptance::Later,
            )
            | (
                ZonedResolution::Unique(_),
                ui::ZonedAcceptance::Earlier | ui::ZonedAcceptance::Later,
            )
            | (ZonedResolution::Ambiguous { .. }, ui::ZonedAcceptance::Unique) => {
                return Err(ParseIssue::InvalidTime);
            }
        };
        Ok(Self {
            local,
            zone,
            instant,
        })
    }
}
impl From<ui::TimePrecision> for TimePrecision {
    fn from(value: ui::TimePrecision) -> Self {
        match value {
            ui::TimePrecision::Minute => Self::Minute,
            ui::TimePrecision::Second => Self::Second,
        }
    }
}
impl From<ui::ClockFormat> for ClockFormat {
    fn from(value: ui::ClockFormat) -> Self {
        match value {
            ui::ClockFormat::TwentyFourHour => Self::TwentyFourHour,
            ui::ClockFormat::TwelveHour => Self::TwelveHour,
        }
    }
}
/// Slint cannot surface a construction error, so a pair the grammar cannot use (empty, equal
/// or prefixing each other) falls back to the canonical AM and PM.
pub(crate) fn day_periods(value: &ui::KitDayPeriods) -> DayPeriodMarkers {
    DayPeriodMarkers::new(value.am.as_str(), value.pm.as_str()).unwrap_or_default()
}
fn parsed<T>(value: DraftValue<T>) -> Result<T, ParseIssue> {
    match value {
        DraftValue::Parsed(value) => Ok(value),
        DraftValue::Invalid(issue) => Err(issue),
        DraftValue::Empty | DraftValue::Incomplete => Err(ParseIssue::InvalidSyntax),
    }
}
fn local(
    date: &str,
    time: &str,
    precision: ui::TimePrecision,
    clock: ui::ClockFormat,
    markers: &DayPeriodMarkers,
) -> Result<LocalDateTime, ParseIssue> {
    Ok(LocalDateTime::new(
        parsed(CivilDate::parse_iso(date))?,
        parsed(LocalTime::parse_with_markers(
            time,
            precision.into(),
            clock.into(),
            markers,
        ))?,
    ))
}
fn projection(value: Result<Option<String>, ParseIssue>) -> ui::TemporalProjection {
    match value {
        Ok(value) => ui::TemporalProjection {
            valid: true,
            value: value.map_or_else(ui::FieldValue::default, |text| ui::FieldValue {
                present: true,
                text: text.into(),
            }),
        },
        Err(_) => ui::TemporalProjection::default(),
    }
}
fn unsigned(value: i32) -> Result<u32, ParseIssue> {
    u32::try_from(value).map_err(|_| ParseIssue::InvalidDate)
}
pub(crate) fn install_time<W>(window: &W)
where
    W: slint::ComponentHandle + 'static,
    for<'a> ui::KitTimeService<'a>: slint::Global<'a, W>,
{
    let service = window.global::<ui::KitTimeService>();
    if service.get_ready() {
        return;
    }
    // Read per call, so the bindings that call a service depend on the window's markers.
    let markers = {
        let window = window.as_weak();
        std::rc::Rc::new(move || {
            window
                .upgrade()
                .map_or_else(DayPeriodMarkers::default, |window| {
                    day_periods(&window.global::<ui::KitTimeService>().get_day_periods())
                })
        })
    };
    service.on_project_date(|value| {
        projection(if value == ui::CivilDateView::default() {
            Ok(None)
        } else {
            CivilDate::try_from(value).map(|date| Some(FieldSpec::date().format(&date)))
        })
    });
    service.on_project_year(|year| {
        projection(if year == 0 {
            Ok(None)
        } else {
            Year::new(year).map(|year| Some(FieldSpec::year().format(&year)))
        })
    });
    service.on_project_month(|year, month| {
        projection(if year == 0 && month == 0 {
            Ok(None)
        } else {
            unsigned(month)
                .and_then(|month| YearMonth::new(year, month))
                .map(|month| Some(FieldSpec::month().format(&month)))
        })
    });
    service.on_project_week(|year, week| {
        projection(if year == 0 && week == 0 {
            Ok(None)
        } else {
            unsigned(week)
                .and_then(|week| IsoWeek::new(year, week))
                .map(|week| Some(FieldSpec::iso_week().format(&week)))
        })
    });
    let current = markers.clone();
    service.on_project_time(move |value, precision, clock| {
        projection(if !value.present {
            Ok(None)
        } else {
            LocalTime::try_from(value.time).and_then(|time| {
                if precision == ui::TimePrecision::Minute && time.second() != 0 {
                    return Err(ParseIssue::InvalidTime);
                }
                Ok(Some(
                    FieldSpec::time_with_markers(precision.into(), clock.into(), current())
                        .format(&time),
                ))
            })
        })
    });
    service.on_parse_year(|text| match FieldSpec::year().parse_text(text.as_str()) {
        DraftValue::Empty => ui::YearResult {
            valid: true,
            year: 0,
        },
        DraftValue::Parsed(year) => ui::YearResult {
            valid: true,
            year: year.get(),
        },
        DraftValue::Incomplete | DraftValue::Invalid(_) => ui::YearResult::default(),
    });
    service.on_parse_month(|text| match FieldSpec::month().parse_text(text.as_str()) {
        DraftValue::Empty => ui::YearMonthResult {
            valid: true,
            ..Default::default()
        },
        DraftValue::Parsed(month) => ui::YearMonthResult {
            valid: true,
            year: month.year(),
            month: i32::try_from(month.month()).unwrap_or_default(),
        },
        DraftValue::Incomplete | DraftValue::Invalid(_) => ui::YearMonthResult::default(),
    });
    service.on_parse_week(
        |text| match FieldSpec::iso_week().parse_text(text.as_str()) {
            DraftValue::Empty => ui::IsoWeekResult {
                valid: true,
                ..Default::default()
            },
            DraftValue::Parsed(week) => ui::IsoWeekResult {
                valid: true,
                year: week.year().get(),
                week: i32::try_from(week.week()).unwrap_or_default(),
            },
            DraftValue::Incomplete | DraftValue::Invalid(_) => ui::IsoWeekResult::default(),
        },
    );
    let current = markers.clone();
    service.on_parse_time(move |text, precision, clock| {
        match LocalTime::parse_with_markers(
            text.as_str(),
            precision.into(),
            clock.into(),
            &current(),
        ) {
            DraftValue::Empty => ui::LocalTimeResult {
                valid: true,
                value: ui::LocalTimeValue::default(),
            },
            DraftValue::Parsed(time) => ui::LocalTimeResult {
                valid: true,
                value: ui::LocalTimeValue {
                    present: true,
                    time: time.into(),
                },
            },
            DraftValue::Incomplete | DraftValue::Invalid(_) => ui::LocalTimeResult::default(),
        }
    });
    let current = markers.clone();
    service.on_adjust(
        move |text, constraints, precision, clock, segment, direction| {
            let markers = current();
            let current = match LocalTime::parse_with_markers(
                text.as_str(),
                precision.into(),
                clock.into(),
                &markers,
            ) {
                DraftValue::Empty => None,
                DraftValue::Parsed(time) => Some(time),
                DraftValue::Incomplete | DraftValue::Invalid(_) => return text,
            };
            let amount = match segment {
                ui::TimeSegment::Hour => 3_600,
                ui::TimeSegment::Minute => 60,
                ui::TimeSegment::Second => 1,
            };
            let direction = match direction {
                ui::TimeAdjustment::Increment => kit_core::StepDirection::Increment,
                ui::TimeAdjustment::Decrement => kit_core::StepDirection::Decrement,
            };
            let (rules, _) = crate::HtmlValidationEngine::default().constraints(&constraints);
            rules
                .step_time(
                    current,
                    direction,
                    std::num::NonZeroU32::new(amount).expect("positive segment"),
                    precision.into(),
                )
                .map_or(text, |time| {
                    time.format_with_markers(precision.into(), clock.into(), &markers)
                        .into()
                })
        },
    );
    let current = markers.clone();
    service.on_parse_local(move |draft, precision, clock| {
        local(
            draft.date.as_str(),
            draft.time.as_str(),
            precision,
            clock,
            &current(),
        )
        .map_or_else(|_| ui::LocalDateTimeView::default(), Into::into)
    });
    service.on_split_local_text(|text| {
        let (date, time) = text.split_once('T').unwrap_or((text.as_str(), ""));
        ui::LocalDateTimeDraft {
            date: date.into(),
            time: time.into(),
        }
    });
    service.on_resolve(move |draft, precision, clock| {
        let Ok(zone) = TimeZoneId::new(draft.zone.as_str()) else {
            return ui::ZonedDateTimeResult {
                state: ui::ZonedTimePresentation::InvalidZone,
                ..Default::default()
            };
        };
        let Ok(local) = local(
            draft.date.as_str(),
            draft.time.as_str(),
            precision,
            clock,
            &markers(),
        ) else {
            return ui::ZonedDateTimeResult::default();
        };
        let (state, utc_offset_seconds, later_utc_offset_seconds) = match zone.resolve(local) {
            ZonedResolution::Unique(instant) => {
                let offset = zone.offset_at(instant).seconds();
                (ui::ZonedTimePresentation::Unique, offset, offset)
            }
            ZonedResolution::Ambiguous { earlier, later } => (
                ui::ZonedTimePresentation::Ambiguous,
                zone.offset_at(earlier).seconds(),
                zone.offset_at(later).seconds(),
            ),
            ZonedResolution::Nonexistent => (ui::ZonedTimePresentation::Nonexistent, 0, 0),
        };
        ui::ZonedDateTimeResult {
            state,
            local: local.into(),
            utc_offset_seconds,
            later_utc_offset_seconds,
        }
    });
    service.set_ready(true);
}

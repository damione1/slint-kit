// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::ui;
use kit_core::{
    CivilDate, ClockFormat, DraftValue, Instant, LocalDateTime, LocalTime, ParseIssue,
    TimePrecision, TimeZoneId, ZonedResolution,
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
) -> Result<LocalDateTime, ParseIssue> {
    Ok(LocalDateTime::new(
        parsed(CivilDate::parse_iso(date))?,
        parsed(LocalTime::parse_with_format(
            time,
            precision.into(),
            clock.into(),
        ))?,
    ))
}
pub(crate) fn install_time<W>(window: &W)
where
    W: slint::ComponentHandle,
    for<'a> ui::KitTimeService<'a>: slint::Global<'a, W>,
{
    let service = window.global::<ui::KitTimeService>();
    if service.get_ready() {
        return;
    }
    service.on_adjust(|text, precision, clock, segment, direction| {
        let Ok(time) = parsed(LocalTime::parse_with_format(
            text.as_str(),
            precision.into(),
            clock.into(),
        )) else {
            return text;
        };
        let amount = match segment {
            ui::TimeSegment::Hour => 3600,
            ui::TimeSegment::Minute => 60,
            ui::TimeSegment::Second => 1,
        };
        let seconds = i64::from(time.hour()) * 3600
            + i64::from(time.minute()) * 60
            + i64::from(time.second());
        let next = match direction {
            ui::TimeAdjustment::Increment => seconds + amount,
            ui::TimeAdjustment::Decrement => seconds - amount,
        }
        .rem_euclid(86400);
        let next = LocalTime::new(
            u32::try_from(next / 3600).expect("hour fits"),
            u32::try_from(next % 3600 / 60).expect("minute fits"),
            u32::try_from(next % 60).expect("second fits"),
        )
        .expect("wrapped local time");
        next.format(precision.into(), clock.into()).into()
    });
    service.on_parse_local(|draft, precision, clock| {
        local(draft.date.as_str(), draft.time.as_str(), precision, clock)
            .map_or_else(|_| ui::LocalDateTimeView::default(), Into::into)
    });
    service.on_resolve(|draft, precision, clock| {
        let Ok(zone) = TimeZoneId::new(draft.zone.as_str()) else {
            return ui::ZonedDateTimeResult {
                state: ui::ZonedTimePresentation::InvalidZone,
                ..Default::default()
            };
        };
        let Ok(local) = local(draft.date.as_str(), draft.time.as_str(), precision, clock) else {
            return ui::ZonedDateTimeResult::default();
        };
        let state = match zone.resolve(local) {
            ZonedResolution::Unique(_) => ui::ZonedTimePresentation::Unique,
            ZonedResolution::Ambiguous { .. } => ui::ZonedTimePresentation::Ambiguous,
            ZonedResolution::Nonexistent => ui::ZonedTimePresentation::Nonexistent,
        };
        ui::ZonedDateTimeResult {
            state,
            local: local.into(),
        }
    });
    service.set_ready(true);
}

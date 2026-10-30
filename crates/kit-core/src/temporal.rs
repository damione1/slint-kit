//! Civil values never acquire a timezone implicitly. IANA resolution is explicit.
mod availability;
mod date_pattern;
use crate::CalendarMove;
use crate::{ClockFormat, TimePrecision};
use crate::{DraftValue, NumericBounds, ParseIssue};
pub use availability::*;
use chrono::{
    DateTime, Datelike, Days, LocalResult, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeZone,
    Timelike, Utc, Weekday,
};
use chrono_tz::Tz;
pub use date_pattern::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Year(i32);
impl Year {
    pub fn new(value: i32) -> Result<Self, ParseIssue> {
        if (1..=9999).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ParseIssue::InvalidDate)
        }
    }
    pub fn get(self) -> i32 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CivilDate(NaiveDate);
impl CivilDate {
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self, ParseIssue> {
        Year::new(year)?;
        NaiveDate::from_ymd_opt(year, month, day)
            .map(Self)
            .ok_or(ParseIssue::InvalidDate)
    }
    pub fn year(self) -> i32 {
        self.0.year()
    }
    pub fn month(self) -> u32 {
        self.0.month()
    }
    pub fn day(self) -> u32 {
        self.0.day()
    }
    pub fn parse_iso(text: &str) -> DraftValue<Self> {
        if text.is_empty() {
            return DraftValue::Empty;
        }
        if text.len() < 10 && text.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return DraftValue::Incomplete;
        }
        if text.len() != 10
            || text.as_bytes()[4] != b'-'
            || text.as_bytes()[7] != b'-'
            || !text
                .bytes()
                .enumerate()
                .all(|(index, value)| index == 4 || index == 7 || value.is_ascii_digit())
        {
            return DraftValue::Invalid(ParseIssue::InvalidDate);
        }
        let year = text[..4].parse();
        let month = text[5..7].parse();
        let day = text[8..].parse();
        match (year, month, day) {
            (Ok(year), Ok(month), Ok(day)) => match Self::new(year, month, day) {
                Ok(date) => DraftValue::Parsed(date),
                Err(error) => DraftValue::Invalid(error),
            },
            (Err(_), _, _) | (_, Err(_), _) | (_, _, Err(_)) => {
                DraftValue::Invalid(ParseIssue::InvalidDate)
            }
        }
    }
    pub fn iso_string(self) -> String {
        self.0.format("%Y-%m-%d").to_string()
    }
    pub fn checked_add_days(self, days: i64) -> Result<Self, ParseIssue> {
        let value = if days >= 0 {
            self.0.checked_add_days(Days::new(days as u64))
        } else {
            self.0.checked_sub_days(Days::new(days.unsigned_abs()))
        };
        let value = value.ok_or(ParseIssue::InvalidDate)?;
        Self::new(value.year(), value.month(), value.day())
    }
    pub fn year_month(self) -> YearMonth {
        YearMonth {
            year: Year(self.year()),
            month: self.month(),
        }
    }
    fn weekday_index(self) -> u32 {
        self.0.weekday().num_days_from_monday()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct YearMonth {
    year: Year,
    month: u32,
}
impl YearMonth {
    pub fn new(year: i32, month: u32) -> Result<Self, ParseIssue> {
        let year = Year::new(year)?;
        if !(1..=12).contains(&month) {
            return Err(ParseIssue::InvalidDate);
        }
        Ok(Self { year, month })
    }
    pub fn year(self) -> i32 {
        self.year.0
    }
    pub fn month(self) -> u32 {
        self.month
    }
    pub fn checked_add_months(self, months: i32) -> Result<Self, ParseIssue> {
        let date =
            NaiveDate::from_ymd_opt(self.year.0, self.month, 1).ok_or(ParseIssue::InvalidDate)?;
        let date = if months >= 0 {
            date.checked_add_months(Months::new(months as u32))
        } else {
            date.checked_sub_months(Months::new(months.unsigned_abs()))
        }
        .ok_or(ParseIssue::InvalidDate)?;
        Self::new(date.year(), date.month())
    }
    pub fn days_in_month(self) -> u32 {
        let first =
            NaiveDate::from_ymd_opt(self.year.0, self.month, 1).expect("validated year month");
        let next = first
            .checked_add_months(Months::new(1))
            .expect("chrono supports the year after 9999");
        (next - first).num_days() as u32
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IsoWeek {
    year: Year,
    week: u32,
}
impl IsoWeek {
    pub fn new(year: i32, week: u32) -> Result<Self, ParseIssue> {
        let year = Year::new(year)?;
        NaiveDate::from_isoywd_opt(year.0, week, Weekday::Mon).ok_or(ParseIssue::InvalidDate)?;
        Ok(Self { year, week })
    }
    pub fn year(self) -> Year {
        self.year
    }
    pub fn week(self) -> u32 {
        self.week
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Meridiem {
    Am,
    Pm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalTime(NaiveTime);
impl LocalTime {
    pub fn new(hour: u32, minute: u32, second: u32) -> Result<Self, ParseIssue> {
        if second > 59 {
            return Err(ParseIssue::InvalidTime);
        }
        NaiveTime::from_hms_opt(hour, minute, second)
            .map(Self)
            .ok_or(ParseIssue::InvalidTime)
    }
    pub fn hour(self) -> u32 {
        self.0.hour()
    }
    pub fn minute(self) -> u32 {
        self.0.minute()
    }
    pub fn second(self) -> u32 {
        self.0.second()
    }
    /// Parse the existing, strictly 24-hour representation.
    pub fn parse(text: &str, precision: TimePrecision) -> DraftValue<Self> {
        Self::parse_with_format(text, precision, ClockFormat::TwentyFourHour)
    }
    /// Parse an explicit clock convention. Twelve-hour input requires AM/PM;
    /// absence is an incomplete draft, never an inferred meridiem.
    ///
    /// Canonical shapes are HH:mm[:ss] and hh:mm[:ss] AM/PM. Hours and each
    /// segment have two ASCII digits. AM/PM letters are case-insensitive.
    pub fn parse_with_format(
        text: &str,
        precision: TimePrecision,
        clock_format: ClockFormat,
    ) -> DraftValue<Self> {
        if text.is_empty() {
            return DraftValue::Empty;
        }
        let clock_length = match precision {
            TimePrecision::Minute => 5,
            TimePrecision::Second => 8,
        };
        let length = clock_length
            + match clock_format {
                ClockFormat::TwentyFourHour => 0,
                ClockFormat::TwelveHour => 3,
            };
        let bytes = text.as_bytes();
        if bytes.len() > length {
            return DraftValue::Invalid(ParseIssue::InvalidTime);
        }
        for (index, byte) in bytes.iter().copied().enumerate() {
            let valid = if index < clock_length {
                if index == 2 || index == 5 {
                    byte == b':'
                } else {
                    byte.is_ascii_digit()
                }
            } else if index == clock_length {
                byte == b' '
            } else if index == clock_length + 1 {
                matches!(byte.to_ascii_uppercase(), b'A' | b'P')
            } else {
                byte.eq_ignore_ascii_case(&b'M')
            };
            if !valid {
                return DraftValue::Invalid(ParseIssue::InvalidTime);
            }
        }
        // Every available complete segment is checked even while the draft is
        // incomplete, so 13:00 in twelve-hour mode cannot become an AM guess.
        let pair = |offset: usize| {
            u32::from(bytes[offset] - b'0') * 10 + u32::from(bytes[offset + 1] - b'0')
        };
        if bytes.len() >= 2 {
            let hour = pair(0);
            let valid = match clock_format {
                ClockFormat::TwentyFourHour => hour < 24,
                ClockFormat::TwelveHour => (1..=12).contains(&hour),
            };
            if !valid {
                return DraftValue::Invalid(ParseIssue::InvalidTime);
            }
        }
        if bytes.len() >= 5 && pair(3) > 59 {
            return DraftValue::Invalid(ParseIssue::InvalidTime);
        }
        if precision == TimePrecision::Second && bytes.len() >= 8 && pair(6) > 59 {
            return DraftValue::Invalid(ParseIssue::InvalidTime);
        }
        if bytes.len() < length {
            return DraftValue::Incomplete;
        }
        let displayed_hour = pair(0);
        let hour = match clock_format {
            ClockFormat::TwentyFourHour => displayed_hour,
            ClockFormat::TwelveHour => {
                let meridiem = match bytes[clock_length + 1].to_ascii_uppercase() {
                    b'A' => Meridiem::Am,
                    b'P' => Meridiem::Pm,
                    _ => unreachable!("meridiem syntax was checked above"),
                };
                match meridiem {
                    Meridiem::Am => displayed_hour % 12,
                    Meridiem::Pm => displayed_hour % 12 + 12,
                }
            }
        };
        let second = match precision {
            TimePrecision::Minute => 0,
            TimePrecision::Second => pair(6),
        };
        match Self::new(hour, pair(3), second) {
            Ok(value) => DraftValue::Parsed(value),
            Err(issue) => DraftValue::Invalid(issue),
        }
    }
    /// Format without changing the canonical civil time. Precision controls the
    /// displayed segments only; no date, timezone or instant is introduced.
    pub fn format(self, precision: TimePrecision, clock_format: ClockFormat) -> String {
        let hour = match clock_format {
            ClockFormat::TwentyFourHour => self.hour(),
            ClockFormat::TwelveHour => match self.hour() % 12 {
                0 => 12,
                hour => hour,
            },
        };
        let clock = match precision {
            TimePrecision::Minute => format!("{hour:02}:{:02}", self.minute()),
            TimePrecision::Second => format!("{hour:02}:{:02}:{:02}", self.minute(), self.second()),
        };
        match clock_format {
            ClockFormat::TwentyFourHour => clock,
            ClockFormat::TwelveHour => {
                let meridiem = if self.hour() < 12 {
                    Meridiem::Am
                } else {
                    Meridiem::Pm
                };
                let suffix = match meridiem {
                    Meridiem::Am => "AM",
                    Meridiem::Pm => "PM",
                };
                format!("{clock} {suffix}")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalDateTime {
    pub date: CivilDate,
    pub time: LocalTime,
}
impl LocalDateTime {
    pub fn new(date: CivilDate, time: LocalTime) -> Self {
        Self { date, time }
    }
    fn naive(self) -> NaiveDateTime {
        self.date.0.and_time(self.time.0)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateRange {
    start: CivilDate,
    end: CivilDate,
}
impl DateRange {
    pub fn new(start: CivilDate, end: CivilDate) -> Result<Self, ParseIssue> {
        if start > end {
            Err(ParseIssue::InvalidRange)
        } else {
            Ok(Self { start, end })
        }
    }
    pub fn start(self) -> CivilDate {
        self.start
    }
    pub fn end(self) -> CivilDate {
        self.end
    }
    /// Number of civil nights (end minus start), independent of DST.
    pub fn nights(self) -> u32 {
        u32::try_from((self.end.0 - self.start.0).num_days()).expect("ordered civil dates")
    }
    pub fn contains(self, date: CivilDate) -> bool {
        self.start <= date && date <= self.end
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimeZoneId(Tz);
impl TimeZoneId {
    pub fn new(value: &str) -> Result<Self, ParseIssue> {
        value.parse().map(Self).map_err(|_| ParseIssue::InvalidTime)
    }
    pub fn name(self) -> &'static str {
        self.0.name()
    }
    pub fn resolve(self, local: LocalDateTime) -> ZonedResolution {
        match self.0.from_local_datetime(&local.naive()) {
            LocalResult::Single(value) => {
                ZonedResolution::Unique(Instant(value.with_timezone(&Utc)))
            }
            LocalResult::Ambiguous(earlier, later) => ZonedResolution::Ambiguous {
                earlier: Instant(earlier.with_timezone(&Utc)),
                later: Instant(later.with_timezone(&Utc)),
            },
            LocalResult::None => ZonedResolution::Nonexistent,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(DateTime<Utc>);
impl Instant {
    pub fn unix_seconds(self) -> i64 {
        self.0.timestamp()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZonedResolution {
    Unique(Instant),
    Ambiguous { earlier: Instant, later: Instant },
    Nonexistent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeekStart {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
impl WeekStart {
    fn index(self) -> u32 {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarCell {
    pub date: CivilDate,
    pub in_displayed_month: bool,
    pub enabled: bool,
    pub selected: bool,
    pub focused: bool,
}
#[derive(Clone, Debug)]
pub struct CalendarController {
    selected: Option<CivilDate>,
    displayed: YearMonth,
    focused: CivilDate,
    constraints: DateConstraints,
    week_start: WeekStart,
}
impl CalendarController {
    pub fn new(
        selected: Option<CivilDate>,
        displayed: YearMonth,
        bounds: Option<NumericBounds<CivilDate>>,
        week_start: WeekStart,
    ) -> Self {
        let first =
            CivilDate::new(displayed.year(), displayed.month(), 1).expect("validated month");
        let focus = selected.unwrap_or(first);
        let focus = if let Some(bounds) = bounds {
            focus.max(bounds.minimum()).min(bounds.maximum())
        } else {
            focus
        };
        Self {
            selected,
            displayed: focus.year_month(),
            focused: focus,
            constraints: DateConstraints::new(bounds),
            week_start,
        }
    }
    pub fn with_constraints(
        selected: Option<CivilDate>,
        displayed: YearMonth,
        constraints: DateConstraints,
        week_start: WeekStart,
    ) -> Result<Self, ParseIssue> {
        if let Some(date) = selected {
            constraints.validate(date)?;
        }
        let mut calendar = Self::new(selected, displayed, constraints.bounds(), week_start);
        calendar.constraints = constraints;
        Ok(calendar)
    }
    pub fn week_start(&self) -> WeekStart {
        self.week_start
    }
    pub fn set_week_start(&mut self, value: WeekStart) {
        self.week_start = value;
    }
    /// Render another month with the same stable selection and availability.
    pub fn grid_for_month(&self, month: YearMonth) -> Vec<Option<CalendarCell>> {
        let mut other = self.clone();
        other.displayed = month;
        other.grid_slots()
    }
    pub fn constraints(&self) -> &DateConstraints {
        &self.constraints
    }
    /// Whether navigation can move the roving target within the date bounds.
    pub fn can_navigate(&self, direction: CalendarMove) -> bool {
        let mut next = self.clone();
        next.navigate(direction).is_ok() && next.focused != self.focused
    }
    pub fn selected(&self) -> Option<CivilDate> {
        self.selected
    }
    pub fn focused(&self) -> CivilDate {
        self.focused
    }
    pub fn displayed(&self) -> YearMonth {
        self.displayed
    }
    pub fn show_month(&mut self, month: YearMonth) {
        let date = CivilDate::new(
            month.year(),
            month.month(),
            self.focused.day().min(month.days_in_month()),
        )
        .expect("validated month and clamped day");
        self.focus_date(date);
    }
    /// Move focus without accepting a value; unavailable days remain browsable.
    pub fn focus_date(&mut self, date: CivilDate) {
        let date = self.constraints.bounds().map_or(date, |bounds| {
            date.max(bounds.minimum()).min(bounds.maximum())
        });
        self.focused = date;
        self.displayed = date.year_month();
    }
    pub fn grid(&self) -> Vec<CalendarCell> {
        self.grid_slots().into_iter().flatten().collect()
    }
    /// Six complete logical weeks, including unavailable slots at domain edges.
    /// Projection may paint an empty disabled cell, never an invalid civil date.
    pub fn grid_slots(&self) -> Vec<Option<CalendarCell>> {
        let first = CivilDate::new(self.displayed.year(), self.displayed.month(), 1)
            .expect("validated month");
        let offset = (first.weekday_index() + 7 - self.week_start.index()) % 7;
        (0..42)
            .map(|index| {
                first
                    .checked_add_days(index - i64::from(offset))
                    .ok()
                    .map(|date| CalendarCell {
                        date,
                        in_displayed_month: date.year_month() == self.displayed,
                        enabled: self.constraints.validate(date).is_ok(),
                        selected: self.selected == Some(date),
                        focused: self.focused == date,
                    })
            })
            .collect()
    }
    pub fn choose(&mut self, date: CivilDate) -> Result<(), ParseIssue> {
        self.constraints.validate(date)?;
        self.selected = Some(date);
        self.focused = date;
        self.displayed = date.year_month();
        Ok(())
    }
    pub fn navigate(&mut self, direction: CalendarMove) -> Result<(), ParseIssue> {
        let current = self.focused;
        let weekday = (current.weekday_index() + 7 - self.week_start.index()) % 7;
        let target = match direction {
            CalendarMove::PreviousDay => current.checked_add_days(-1)?,
            CalendarMove::NextDay => current.checked_add_days(1)?,
            CalendarMove::PreviousWeek => current.checked_add_days(-7)?,
            CalendarMove::NextWeek => current.checked_add_days(7)?,
            CalendarMove::WeekStart => current.checked_add_days(-i64::from(weekday))?,
            CalendarMove::WeekEnd => current.checked_add_days(i64::from(6 - weekday))?,
            CalendarMove::PreviousMonth
            | CalendarMove::NextMonth
            | CalendarMove::PreviousYear
            | CalendarMove::NextYear => {
                let months = match direction {
                    CalendarMove::PreviousMonth => -1,
                    CalendarMove::NextMonth => 1,
                    CalendarMove::PreviousYear => -12,
                    CalendarMove::NextYear => 12,
                    CalendarMove::PreviousDay
                    | CalendarMove::NextDay
                    | CalendarMove::PreviousWeek
                    | CalendarMove::NextWeek
                    | CalendarMove::WeekStart
                    | CalendarMove::WeekEnd => unreachable!(),
                };
                let month = current.year_month().checked_add_months(months)?;
                CivilDate::new(
                    month.year(),
                    month.month(),
                    current.day().min(month.days_in_month()),
                )?
            }
        };
        let target = if let Some(bounds) = self.constraints.bounds() {
            target.max(bounds.minimum()).min(bounds.maximum())
        } else {
            target
        };
        self.focused = target;
        self.displayed = target.year_month();
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeSelection {
    ChoosingStart,
    ChoosingEnd { start: CivilDate },
    Complete(DateRange),
}
#[derive(Debug)]
pub struct DateRangeController {
    accepted: Option<DateRange>,
    draft: RangeSelection,
    rules: DateRangeRules,
}
impl DateRangeController {
    pub fn new(accepted: Option<DateRange>) -> Self {
        Self {
            accepted,
            draft: RangeSelection::ChoosingStart,
            rules: DateRangeRules::default(),
        }
    }
    pub fn with_rules(
        accepted: Option<DateRange>,
        rules: DateRangeRules,
    ) -> Result<Self, ParseIssue> {
        if let Some(range) = accepted {
            rules.validate(range)?;
        }
        Ok(Self {
            accepted,
            draft: RangeSelection::ChoosingStart,
            rules,
        })
    }
    pub fn draft(&self) -> RangeSelection {
        self.draft
    }
    pub fn choose(&mut self, date: CivilDate) -> Result<(), ParseIssue> {
        self.rules.dates().validate(date)?;
        self.draft = match self.draft {
            RangeSelection::ChoosingStart | RangeSelection::Complete(_) => {
                RangeSelection::ChoosingEnd { start: date }
            }
            RangeSelection::ChoosingEnd { start } => {
                let range = DateRange::new(start, date)?;
                self.rules.validate(range)?;
                RangeSelection::Complete(range)
            }
        };
        Ok(())
    }
    pub fn apply(&mut self) -> Option<DateRange> {
        match self.draft {
            RangeSelection::Complete(value) => {
                self.accepted = Some(value);
                Some(value)
            }
            RangeSelection::ChoosingStart | RangeSelection::ChoosingEnd { .. } => None,
        }
    }
    pub fn cancel(&mut self) {
        self.draft = self
            .accepted
            .map_or(RangeSelection::ChoosingStart, RangeSelection::Complete);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn date(year: i32, month: u32, day: u32) -> CivilDate {
        CivilDate::new(year, month, day).unwrap()
    }
    #[test]
    fn civil_invariants_and_incomplete() {
        assert!(CivilDate::new(2025, 2, 29).is_err());
        assert!(CivilDate::new(2024, 2, 29).is_ok());
        assert!(DateRange::new(date(2026, 3, 2), date(2026, 3, 1)).is_err());
        assert!(IsoWeek::new(2021, 53).is_err());
        assert!(IsoWeek::new(2020, 53).is_ok());
        assert_eq!(CivilDate::parse_iso("2026-"), DraftValue::Incomplete);
        assert!(LocalTime::new(24, 0, 0).is_err());
    }
    #[test]
    fn twelve_hour_noon_midnight_and_meridiem_are_explicit() {
        let parse = |text| {
            LocalTime::parse_with_format(text, TimePrecision::Minute, ClockFormat::TwelveHour)
        };
        assert_eq!(
            parse("12:00 AM"),
            DraftValue::Parsed(LocalTime::new(0, 0, 0).unwrap())
        );
        assert_eq!(
            parse("12:00 PM"),
            DraftValue::Parsed(LocalTime::new(12, 0, 0).unwrap())
        );
        assert_eq!(
            parse("01:45 pm"),
            DraftValue::Parsed(LocalTime::new(13, 45, 0).unwrap())
        );
        for text in ["1", "12:", "12:0", "12:00", "12:00 ", "12:00 A", "12:00 p"] {
            assert_eq!(parse(text), DraftValue::Incomplete, "draft {text:?}");
        }
        assert_eq!(parse(""), DraftValue::Empty);
        for text in [
            "00:00 AM",
            "13:00 PM",
            "12:60 AM",
            "12:00 XM",
            "12:00 AA",
            "12:00AM",
            "12:00 AM ",
            "１２:00 AM",
        ] {
            assert_eq!(
                parse(text),
                DraftValue::Invalid(ParseIssue::InvalidTime),
                "invalid {text:?}"
            );
        }
    }
    #[test]
    fn clock_formats_round_trip_every_hour_with_both_precisions() {
        for hour in 0..24 {
            let value = LocalTime::new(hour, 45, 37).unwrap();
            for clock in [ClockFormat::TwentyFourHour, ClockFormat::TwelveHour] {
                assert_eq!(
                    LocalTime::parse_with_format(
                        &value.format(TimePrecision::Second, clock),
                        TimePrecision::Second,
                        clock
                    ),
                    DraftValue::Parsed(value)
                );
                assert_eq!(
                    LocalTime::parse_with_format(
                        &value.format(TimePrecision::Minute, clock),
                        TimePrecision::Minute,
                        clock
                    ),
                    DraftValue::Parsed(LocalTime::new(hour, 45, 0).unwrap())
                );
            }
        }
        assert_eq!(
            LocalTime::new(0, 0, 0)
                .unwrap()
                .format(TimePrecision::Minute, ClockFormat::TwelveHour),
            "12:00 AM"
        );
        assert_eq!(
            LocalTime::new(12, 0, 0)
                .unwrap()
                .format(TimePrecision::Minute, ClockFormat::TwelveHour),
            "12:00 PM"
        );
        assert_eq!(
            LocalTime::new(23, 59, 59)
                .unwrap()
                .format(TimePrecision::Second, ClockFormat::TwelveHour),
            "11:59:59 PM"
        );
    }
    #[test]
    fn clock_and_precision_never_infer_or_coerce_input() {
        assert_eq!(
            LocalTime::parse("23:59", TimePrecision::Minute),
            DraftValue::Parsed(LocalTime::new(23, 59, 0).unwrap())
        );
        assert_eq!(
            LocalTime::parse("12:00 PM", TimePrecision::Minute),
            DraftValue::Invalid(ParseIssue::InvalidTime)
        );
        assert_eq!(
            LocalTime::parse_with_format("12:00", TimePrecision::Minute, ClockFormat::TwelveHour),
            DraftValue::Incomplete
        );
        assert_eq!(
            LocalTime::parse_with_format(
                "12:00:59 PM",
                TimePrecision::Second,
                ClockFormat::TwelveHour
            ),
            DraftValue::Parsed(LocalTime::new(12, 0, 59).unwrap())
        );
        assert_eq!(
            LocalTime::parse_with_format(
                "12:00:60 PM",
                TimePrecision::Second,
                ClockFormat::TwelveHour
            ),
            DraftValue::Invalid(ParseIssue::InvalidTime)
        );
        assert_eq!(
            LocalTime::parse_with_format(
                "12:00:59 PM",
                TimePrecision::Minute,
                ClockFormat::TwelveHour
            ),
            DraftValue::Invalid(ParseIssue::InvalidTime)
        );
        assert_eq!(
            LocalTime::parse_with_format(
                "12:00 PM",
                TimePrecision::Second,
                ClockFormat::TwelveHour
            ),
            DraftValue::Invalid(ParseIssue::InvalidTime)
        );
        assert_eq!(
            LocalTime::parse("24:00", TimePrecision::Minute),
            DraftValue::Invalid(ParseIssue::InvalidTime)
        );
    }
    #[test]
    fn dst_gap_fold_and_normal_time_are_explicit() {
        let tz = TimeZoneId::new("America/Toronto").unwrap();
        assert_eq!(
            tz.resolve(LocalDateTime::new(
                date(2026, 3, 8),
                LocalTime::new(2, 30, 0).unwrap()
            )),
            ZonedResolution::Nonexistent
        );
        let ZonedResolution::Ambiguous { earlier, later } = tz.resolve(LocalDateTime::new(
            date(2026, 11, 1),
            LocalTime::new(1, 30, 0).unwrap(),
        )) else {
            panic!("expected fold")
        };
        assert_eq!(later.unix_seconds() - earlier.unix_seconds(), 3600);
        assert!(matches!(
            tz.resolve(LocalDateTime::new(
                date(2026, 6, 1),
                LocalTime::new(12, 0, 0).unwrap()
            )),
            ZonedResolution::Unique(_)
        ));
    }
    #[test]
    fn calendar_grid_and_month_clamping() {
        let mut calendar = CalendarController::new(
            Some(date(2026, 1, 31)),
            YearMonth::new(2026, 1).unwrap(),
            None,
            WeekStart::Monday,
        );
        assert_eq!(calendar.grid().len(), 42);
        calendar.navigate(CalendarMove::NextMonth).unwrap();
        assert_eq!(calendar.focused(), date(2026, 2, 28));
        calendar.navigate(CalendarMove::WeekStart).unwrap();
        assert_eq!(calendar.focused(), date(2026, 2, 23));
    }
    #[test]
    fn civil_boundaries_keep_weekday_slots() {
        let first =
            CalendarController::new(None, YearMonth::new(1, 1).unwrap(), None, WeekStart::Sunday);
        let slots = first.grid_slots();
        assert_eq!(slots.len(), 42);
        assert!(slots[0].is_none());
        assert_eq!(slots[1].unwrap().date, date(1, 1, 1));
        let last = CalendarController::new(
            None,
            YearMonth::new(9999, 12).unwrap(),
            None,
            WeekStart::Monday,
        );
        let slots = last.grid_slots();
        assert_eq!(slots.len(), 42);
        assert!(slots[41].is_none());
        assert_eq!(
            slots.iter().flatten().last().unwrap().date,
            date(9999, 12, 31)
        );
    }
    #[test]
    fn range_draft_does_not_commit_and_cancel_restores() {
        let original = DateRange::new(date(2026, 1, 1), date(2026, 1, 3)).unwrap();
        let mut range = DateRangeController::new(Some(original));
        range.choose(date(2026, 2, 1)).unwrap();
        assert!(range.apply().is_none());
        assert!(range.choose(date(2026, 1, 31)).is_err());
        range.cancel();
        assert_eq!(range.draft(), RangeSelection::Complete(original));
    }
}

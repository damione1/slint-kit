//! Host-owned booking and availability rules, shared by pointer and text paths.
use super::{CivilDate, DateRange, WeekStart};
use crate::{NumericBounds, ParseIssue};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DateConstraints {
    bounds: Option<NumericBounds<CivilDate>>,
    unavailable: Vec<DateRange>,
    unavailable_weekdays: Vec<WeekStart>,
}
impl DateConstraints {
    pub fn new(bounds: Option<NumericBounds<CivilDate>>) -> Self {
        Self {
            bounds,
            ..Self::default()
        }
    }
    pub fn with_unavailable_ranges(mut self, ranges: Vec<DateRange>) -> Self {
        self.unavailable = ranges;
        self
    }
    pub fn with_unavailable_weekdays(mut self, weekdays: Vec<WeekStart>) -> Self {
        self.unavailable_weekdays = weekdays;
        self
    }
    pub fn bounds(&self) -> Option<NumericBounds<CivilDate>> {
        self.bounds
    }
    pub fn validate(&self, date: CivilDate) -> Result<CivilDate, ParseIssue> {
        if let Some(bounds) = self.bounds {
            bounds.validate(date)?;
        }
        if self.unavailable.iter().any(|range| range.contains(date))
            || self
                .unavailable_weekdays
                .iter()
                .any(|weekday| weekday.index() == date.weekday_index())
        {
            return Err(ParseIssue::InvalidDate);
        }
        Ok(date)
    }
    /// Inclusive endpoints and interior: a range cannot bridge blocked dates.
    pub fn validate_range(&self, range: DateRange) -> Result<DateRange, ParseIssue> {
        self.validate(range.start())?;
        self.validate(range.end())?;
        if self
            .unavailable
            .iter()
            .any(|blocked| blocked.start() <= range.end() && range.start() <= blocked.end())
        {
            return Err(ParseIssue::InvalidRange);
        }
        // At most seven checks, even for a range spanning thousands of years.
        for offset in 0..=range.nights().min(6) {
            let date = range.start().checked_add_days(i64::from(offset))?;
            if self
                .unavailable_weekdays
                .iter()
                .any(|weekday| weekday.index() == date.weekday_index())
            {
                return Err(ParseIssue::InvalidRange);
            }
        }
        Ok(range)
    }
}

/// A stay length counts nights, so a same-day range has length zero.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DateRangeRules {
    dates: DateConstraints,
    nights: Option<NumericBounds<u32>>,
}
impl DateRangeRules {
    pub fn new(dates: DateConstraints, nights: Option<NumericBounds<u32>>) -> Self {
        Self { dates, nights }
    }
    pub fn dates(&self) -> &DateConstraints {
        &self.dates
    }
    pub fn validate(&self, range: DateRange) -> Result<DateRange, ParseIssue> {
        self.dates.validate_range(range)?;
        if let Some(nights) = self.nights {
            nights.validate(range.nights())?;
        }
        Ok(range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CalendarController, CalendarMove, DateRangeController, RangeSelection, YearMonth};
    fn day(day: u32) -> CivilDate {
        CivilDate::new(2026, 10, day).unwrap()
    }
    #[test]
    fn keyboard_can_inspect_unavailable_days_but_no_acceptance_path_bypasses_rules() {
        let blocked = DateRange::new(day(13), day(15)).unwrap();
        let rules = DateConstraints::new(Some(NumericBounds::new(day(1), day(30)).unwrap()))
            .with_unavailable_ranges(vec![blocked])
            .with_unavailable_weekdays(vec![WeekStart::Saturday, WeekStart::Sunday]);
        let mut calendar = CalendarController::with_constraints(
            Some(day(12)),
            day(12).year_month(),
            rules,
            WeekStart::Monday,
        )
        .unwrap();
        calendar.navigate(CalendarMove::NextDay).unwrap();
        assert_eq!(calendar.focused(), day(13));
        assert!(
            calendar
                .grid()
                .iter()
                .any(|cell| cell.date == day(13) && cell.focused && !cell.enabled)
        );
        assert!(calendar.choose(day(13)).is_err());
        assert!(calendar.choose(day(17)).is_err());
        assert!(calendar.choose(day(31)).is_err());
        assert_eq!(calendar.selected(), Some(day(12)));
        calendar.choose(day(16)).unwrap();
        assert_eq!(calendar.selected(), Some(day(16)));
    }
    #[test]
    fn all_week_starts_align_grid_and_home_end_and_focus_follows_visible_month() {
        for first in [
            WeekStart::Monday,
            WeekStart::Tuesday,
            WeekStart::Wednesday,
            WeekStart::Thursday,
            WeekStart::Friday,
            WeekStart::Saturday,
            WeekStart::Sunday,
        ] {
            let mut calendar =
                CalendarController::new(Some(day(17)), day(17).year_month(), None, first);
            assert_eq!(calendar.grid()[0].date.weekday_index(), first.index());
            calendar.navigate(CalendarMove::WeekStart).unwrap();
            assert_eq!(calendar.focused().weekday_index(), first.index());
            calendar.navigate(CalendarMove::WeekEnd).unwrap();
            assert_eq!(calendar.focused().weekday_index(), (first.index() + 6) % 7);
            calendar.show_month(YearMonth::new(2027, 2).unwrap());
            assert!(calendar.grid().iter().any(|cell| cell.focused));
            assert_eq!(calendar.focused().month(), 2);
        }
        let bounds = NumericBounds::new(day(10), day(20)).unwrap();
        let calendar = CalendarController::new(
            None,
            YearMonth::new(2027, 1).unwrap(),
            Some(bounds),
            WeekStart::Monday,
        );
        assert_eq!(calendar.displayed(), day(20).year_month());
        assert!(!calendar.can_navigate(CalendarMove::NextMonth));
        assert!(calendar.can_navigate(CalendarMove::PreviousDay));
    }
    #[test]
    fn bounded_month_navigation_and_initial_selection_have_consistent_availability() {
        let bounds = NumericBounds::new(day(5), day(26)).unwrap();
        let calendar = CalendarController::new(
            Some(day(19)),
            day(19).year_month(),
            Some(bounds),
            WeekStart::Monday,
        );
        assert!(!calendar.can_navigate(CalendarMove::PreviousMonth));
        assert!(!calendar.can_navigate(CalendarMove::NextMonth));
        assert!(!calendar.can_navigate(CalendarMove::PreviousYear));
        assert!(!calendar.can_navigate(CalendarMove::NextYear));
        assert!(calendar.can_navigate(CalendarMove::PreviousDay));
        assert!(calendar.can_navigate(CalendarMove::NextWeek));
        for invalid in [day(4), day(27)] {
            let calendar = CalendarController::new(
                Some(invalid),
                invalid.year_month(),
                Some(bounds),
                WeekStart::Monday,
            );
            assert_eq!(calendar.selected(), None);
            assert!(bounds.validate(calendar.focused()).is_ok());
            assert!(!calendar.grid().iter().any(|cell| cell.selected));
            assert!(
                CalendarController::with_constraints(
                    Some(invalid),
                    invalid.year_month(),
                    DateConstraints::new(Some(bounds)),
                    WeekStart::Monday
                )
                .is_err()
            );
        }
        let edge =
            CalendarController::new(None, YearMonth::new(1, 1).unwrap(), None, WeekStart::Monday);
        assert!(!edge.can_navigate(CalendarMove::PreviousDay));
        assert!(!edge.can_navigate(CalendarMove::PreviousMonth));
        assert!(edge.can_navigate(CalendarMove::NextMonth));
    }
    #[test]
    fn multiple_dates_preserve_identity_and_exclude_blocked_values() {
        let rules = DateConstraints::default().with_unavailable_weekdays(vec![WeekStart::Sunday]);
        let mut selected = DateSetController::new(rules);
        assert_eq!(selected.toggle(day(19)), Ok(true));
        assert_eq!(selected.toggle(day(12)), Ok(true));
        assert!(selected.toggle(day(18)).is_err());
        assert_eq!(
            selected.selected().iter().copied().collect::<Vec<_>>(),
            [day(12), day(19)]
        );
        assert_eq!(selected.toggle(day(12)), Ok(false));
        selected.clear();
        assert!(selected.selected().is_empty());
    }
    #[test]
    fn range_nights_and_interior_availability_are_atomic_and_cancel_restores() {
        let rules = DateRangeRules::new(
            DateConstraints::default()
                .with_unavailable_ranges(vec![DateRange::new(day(14), day(15)).unwrap()]),
            Some(NumericBounds::new(2, 7).unwrap()),
        );
        let accepted = DateRange::new(day(5), day(8)).unwrap();
        let mut range = DateRangeController::with_rules(Some(accepted), rules).unwrap();
        range.choose(day(12)).unwrap();
        assert!(
            range.choose(day(12)).is_err(),
            "same-day range is zero nights"
        );
        assert!(
            range.choose(day(16)).is_err(),
            "available endpoints cannot bridge a blackout"
        );
        assert_eq!(
            range.draft(),
            RangeSelection::ChoosingEnd { start: day(12) }
        );
        assert!(range.apply().is_none());
        range.cancel();
        assert_eq!(range.draft(), RangeSelection::Complete(accepted));
        range.choose(day(20)).unwrap();
        range.choose(day(23)).unwrap();
        assert_eq!(range.apply().unwrap().nights(), 3);
        let weekend = DateConstraints::default().with_unavailable_weekdays(vec![WeekStart::Sunday]);
        assert!(
            weekend
                .validate_range(DateRange::new(day(1), day(10)).unwrap())
                .is_err()
        );
    }
}

/// Multiple selected civil dates, keyed by value, with the same acceptance
/// rules as a single-date calendar. Project membership to `CalendarDay.selected`.
#[derive(Clone, Debug)]
pub struct DateSetController {
    selected: std::collections::BTreeSet<CivilDate>,
    dates: DateConstraints,
}
impl DateSetController {
    pub fn new(dates: DateConstraints) -> Self {
        Self {
            selected: std::collections::BTreeSet::new(),
            dates,
        }
    }
    pub fn selected(&self) -> &std::collections::BTreeSet<CivilDate> {
        &self.selected
    }
    pub fn toggle(&mut self, date: CivilDate) -> Result<bool, ParseIssue> {
        self.dates.validate(date)?;
        if self.selected.remove(&date) {
            Ok(false)
        } else {
            self.selected.insert(date);
            Ok(true)
        }
    }
    pub fn clear(&mut self) {
        self.selected.clear();
    }
}

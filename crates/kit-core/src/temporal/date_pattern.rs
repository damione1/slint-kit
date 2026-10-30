//! Explicit Gregorian date patterns. No locale or century is guessed.
use super::CivilDate;
use crate::{DraftValue, ParseIssue};
use chrono::{
    Datelike, NaiveDate, Weekday,
    format::{Fixed, Item, ParseErrorKind, Parsed, StrftimeItems, parse_and_remainder},
};

/// A validated, date-only strftime pattern. Supported directives are `%Y`,
/// `%m`, `%d`, `%e`, `%b`, `%B`, `%a`, `%A`, `%j`, `%G`, `%V`, `%u`, `%F`
/// and `%%`; numeric directives may use `-`, `_` or `0` padding. Month and
/// weekday names use English by default. The `*_with_names` methods accept
/// host-owned translations, including the supplied French names.
///
/// A pattern must contain a complete calendar, ordinal, or ISO week date.
/// Two-digit years, time zones and time-of-day directives are rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatePattern(String);
impl DatePattern {
    pub fn new(pattern: impl Into<String>) -> Result<Self, ParseIssue> {
        let pattern = pattern.into();
        let mut chars = pattern.chars();
        let (mut year, mut month, mut day, mut ordinal, mut iso_year, mut week, mut weekday) =
            (false, false, false, false, false, false, false);
        while let Some(ch) = chars.next() {
            if ch != '%' {
                continue;
            }
            let mut directive = chars.next().ok_or(ParseIssue::InvalidSyntax)?;
            if matches!(directive, '-' | '_' | '0') {
                directive = chars.next().ok_or(ParseIssue::InvalidSyntax)?;
                if !matches!(directive, 'Y' | 'm' | 'd' | 'e' | 'j' | 'G' | 'V' | 'u') {
                    return Err(ParseIssue::InvalidSyntax);
                }
            }
            match directive {
                'Y' => year = true,
                'm' | 'b' | 'B' => month = true,
                'd' | 'e' => day = true,
                'j' => ordinal = true,
                'G' => iso_year = true,
                'V' => week = true,
                'u' | 'a' | 'A' => weekday = true,
                'F' => {
                    year = true;
                    month = true;
                    day = true;
                }
                '%' => (),
                _ => return Err(ParseIssue::InvalidSyntax),
            }
        }
        if !(year && (month && day || ordinal) || iso_year && week && weekday) {
            return Err(ParseIssue::InvalidSyntax);
        }
        Ok(Self(pattern))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn format(&self, date: CivilDate) -> String {
        date.0.format(&self.0).to_string()
    }
    /// Parse with this single explicit pattern, without trying competing date
    /// orders. Partial drafts remain editable; impossible dates never roll over.
    pub fn parse(&self, text: &str) -> DraftValue<CivilDate> {
        if text.is_empty() {
            return DraftValue::Empty;
        }
        match NaiveDate::parse_from_str(text, &self.0) {
            Ok(date) => match CivilDate::new(date.year(), date.month(), date.day()) {
                Ok(date) => DraftValue::Parsed(date),
                Err(issue) => DraftValue::Invalid(issue),
            },
            Err(error) if error.kind() == ParseErrorKind::TooShort => DraftValue::Incomplete,
            Err(_) => DraftValue::Invalid(ParseIssue::InvalidDate),
        }
    }
}

/// Host-supplied names keep formatting open to any language without silently
/// choosing a regional date order. Weekdays are always Monday through Sunday.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateNames {
    months: [String; 12],
    short_months: [String; 12],
    weekdays: [String; 7],
    short_weekdays: [String; 7],
}
impl DateNames {
    pub fn new(
        months: [String; 12],
        short_months: [String; 12],
        weekdays: [String; 7],
        short_weekdays: [String; 7],
    ) -> Result<Self, ParseIssue> {
        fn unique(names: &[String]) -> bool {
            names.iter().enumerate().all(|(index, name)| {
                !name.is_empty()
                    && names[..index]
                        .iter()
                        .all(|other| other.to_lowercase() != name.to_lowercase())
            })
        }
        if !unique(&months)
            || !unique(&short_months)
            || !unique(&weekdays)
            || !unique(&short_weekdays)
        {
            return Err(ParseIssue::InvalidSyntax);
        }
        Ok(Self {
            months,
            short_months,
            weekdays,
            short_weekdays,
        })
    }
    pub fn french() -> Self {
        Self::new(
            [
                "janvier",
                "février",
                "mars",
                "avril",
                "mai",
                "juin",
                "juillet",
                "août",
                "septembre",
                "octobre",
                "novembre",
                "décembre",
            ]
            .map(String::from),
            [
                "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.",
                "nov.", "déc.",
            ]
            .map(String::from),
            [
                "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche",
            ]
            .map(String::from),
            ["lun.", "mar.", "mer.", "jeu.", "ven.", "sam.", "dim."].map(String::from),
        )
        .expect("distinct French date names")
    }
}
impl DatePattern {
    pub fn format_with_names(&self, date: CivilDate, names: &DateNames) -> String {
        StrftimeItems::new(&self.0)
            .map(|item| {
                let month = usize::try_from(date.month() - 1).expect("validated month");
                let weekday = usize::try_from(date.weekday_index()).expect("weekday index");
                if item == Item::Fixed(Fixed::LongMonthName) {
                    names.months[month].clone()
                } else if item == Item::Fixed(Fixed::ShortMonthName) {
                    names.short_months[month].clone()
                } else if item == Item::Fixed(Fixed::LongWeekdayName) {
                    names.weekdays[weekday].clone()
                } else if item == Item::Fixed(Fixed::ShortWeekdayName) {
                    names.short_weekdays[weekday].clone()
                } else {
                    date.0.format_with_items(std::iter::once(&item)).to_string()
                }
            })
            .collect()
    }
    /// Name matching uses Unicode lowercase, without accent stripping or
    /// locale-specific case folding. Abbreviations must include their punctuation.
    pub fn parse_with_names(&self, text: &str, names: &DateNames) -> DraftValue<CivilDate> {
        if text.is_empty() {
            return DraftValue::Empty;
        }
        let mut parsed = Parsed::new();
        let mut remaining = text;
        let mut segment = Vec::new();
        for item in StrftimeItems::new(&self.0) {
            let candidates: Option<&[String]> = if item == Item::Fixed(Fixed::LongMonthName) {
                Some(&names.months)
            } else if item == Item::Fixed(Fixed::ShortMonthName) {
                Some(&names.short_months)
            } else if item == Item::Fixed(Fixed::LongWeekdayName) {
                Some(&names.weekdays)
            } else if item == Item::Fixed(Fixed::ShortWeekdayName) {
                Some(&names.short_weekdays)
            } else {
                None
            };
            let Some(candidates) = candidates else {
                segment.push(item);
                continue;
            };
            remaining = match parse_and_remainder(&mut parsed, remaining, segment.iter()) {
                Ok(rest) => rest,
                Err(error) if error.kind() == ParseErrorKind::TooShort => {
                    return DraftValue::Incomplete;
                }
                Err(_) => return DraftValue::Invalid(ParseIssue::InvalidDate),
            };
            segment.clear();
            // Lowercase by character, preserving the actual number of input
            // bytes consumed: Unicode case mappings can change UTF-8 length.
            fn name_prefix(text: &str, name: &str) -> Option<usize> {
                let target = name.to_lowercase();
                let mut folded = String::new();
                for (offset, ch) in text.char_indices() {
                    folded.extend(ch.to_lowercase());
                    if folded == target {
                        return Some(offset + ch.len_utf8());
                    }
                    if !target.starts_with(&folded) {
                        return None;
                    }
                }
                None
            }
            let matched = candidates
                .iter()
                .enumerate()
                .filter_map(|(index, name)| {
                    name_prefix(remaining, name).map(|consumed| (index, consumed))
                })
                .max_by_key(|(_, consumed)| *consumed);
            let Some((index, consumed)) = matched else {
                return if candidates
                    .iter()
                    .any(|name| name.to_lowercase().starts_with(&remaining.to_lowercase()))
                {
                    DraftValue::Incomplete
                } else {
                    DraftValue::Invalid(ParseIssue::InvalidDate)
                };
            };
            let result = if item == Item::Fixed(Fixed::LongMonthName)
                || item == Item::Fixed(Fixed::ShortMonthName)
            {
                parsed.set_month(i64::try_from(index + 1).expect("month index"))
            } else {
                parsed.set_weekday(
                    [
                        Weekday::Mon,
                        Weekday::Tue,
                        Weekday::Wed,
                        Weekday::Thu,
                        Weekday::Fri,
                        Weekday::Sat,
                        Weekday::Sun,
                    ][index],
                )
            };
            if result.is_err() {
                return DraftValue::Invalid(ParseIssue::InvalidDate);
            }
            remaining = &remaining[consumed..];
        }
        match parse_and_remainder(&mut parsed, remaining, segment.iter()) {
            Ok("") => (),
            Err(error) if error.kind() == ParseErrorKind::TooShort => {
                return DraftValue::Incomplete;
            }
            Ok(_) | Err(_) => return DraftValue::Invalid(ParseIssue::InvalidDate),
        }
        match parsed.to_naive_date() {
            Ok(date) => match CivilDate::new(date.year(), date.month(), date.day()) {
                Ok(date) => DraftValue::Parsed(date),
                Err(issue) => DraftValue::Invalid(issue),
            },
            Err(_) => DraftValue::Invalid(ParseIssue::InvalidDate),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_patterns_cover_numeric_textual_ordinal_and_iso_week_dates() {
        let date = CivilDate::new(2024, 2, 29).unwrap();
        for (pattern, expected) in [
            ("%Y-%m-%d", "2024-02-29"),
            ("%d/%m/%Y", "29/02/2024"),
            ("%m/%d/%Y", "02/29/2024"),
            ("%d.%m.%Y", "29.02.2024"),
            ("%Y%m%d", "20240229"),
            ("%-m/%-d/%Y", "2/29/2024"),
            ("%B %d, %Y", "February 29, 2024"),
            ("%a, %e %b %Y", "Thu, 29 Feb 2024"),
            ("%Y-%j", "2024-060"),
            ("%G-W%V-%u", "2024-W09-4"),
            ("%Y年%-m月%-d日", "2024年2月29日"),
            ("%F", "2024-02-29"),
        ] {
            let pattern = DatePattern::new(pattern).unwrap();
            assert_eq!(pattern.format(date), expected);
            assert_eq!(pattern.parse(expected), DraftValue::Parsed(date));
        }
    }
    #[test]
    fn host_localized_names_round_trip_and_check_weekday_consistency() {
        let names = DateNames::french();
        let date = CivilDate::new(2024, 2, 29).unwrap();
        let full = DatePattern::new("%A %-d %B %Y").unwrap();
        assert_eq!(
            full.format_with_names(date, &names),
            "jeudi 29 février 2024"
        );
        assert_eq!(
            full.parse_with_names("JEUDI 29 FÉVRIER 2024", &names),
            DraftValue::Parsed(date)
        );
        assert_eq!(
            full.parse_with_names("vendredi 29 février 2024", &names),
            DraftValue::Invalid(ParseIssue::InvalidDate)
        );
        assert_eq!(
            full.parse_with_names("jeudi 29 fév", &names),
            DraftValue::Incomplete
        );
        for pattern in ["%d %b %Y", "%Y%m%d", "%G-W%V-%u"] {
            let pattern = DatePattern::new(pattern).unwrap();
            assert_eq!(
                pattern.parse_with_names(&pattern.format_with_names(date, &names), &names),
                DraftValue::Parsed(date)
            );
        }
    }
    #[test]
    fn localized_case_mapping_consumes_input_bytes_instead_of_name_bytes() {
        let mut names = DateNames::french();
        names.months[1] = "ßévrier".into();
        let date = CivilDate::new(2024, 2, 29).unwrap();
        let pattern = DatePattern::new("%d %B %Y").unwrap();
        assert_eq!(
            pattern.parse_with_names("29 ẞÉVRIER 2024", &names),
            DraftValue::Parsed(date)
        );
    }
    #[test]
    fn patterns_do_not_guess_order_century_or_roll_over_invalid_dates() {
        let dmy = DatePattern::new("%d/%m/%Y").unwrap();
        let mdy = DatePattern::new("%m/%d/%Y").unwrap();
        assert_eq!(
            dmy.parse("03/04/2026"),
            DraftValue::Parsed(CivilDate::new(2026, 4, 3).unwrap())
        );
        assert_eq!(
            mdy.parse("03/04/2026"),
            DraftValue::Parsed(CivilDate::new(2026, 3, 4).unwrap())
        );
        assert_eq!(
            dmy.parse("29/02/2025"),
            DraftValue::Invalid(ParseIssue::InvalidDate)
        );
        assert_eq!(dmy.parse("29/02/"), DraftValue::Incomplete);
        assert_eq!(dmy.parse(""), DraftValue::Empty);
        for pattern in ["%d/%m/%y", "%m-%d", "%F %H", "%+", "%Q", "%", "%_B"] {
            assert!(DatePattern::new(pattern).is_err(), "{pattern}");
        }
        let week = DatePattern::new("%G-W%V-%u").unwrap();
        assert_eq!(
            week.parse("2020-W53-5"),
            DraftValue::Parsed(CivilDate::new(2021, 1, 1).unwrap())
        );
        assert_eq!(
            week.parse("2021-W53-1"),
            DraftValue::Invalid(ParseIssue::InvalidDate)
        );
    }
}

// SPDX-License-Identifier: GPL-3.0-or-later
//! Strict phone values and display masks, separate from HTML's free-form tel state.
use crate::{DraftValue, ParseIssue, PhoneNumberMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhoneDigitCount(u8);
impl PhoneDigitCount {
    pub fn new(value: u8) -> Result<Self, PhoneError> {
        if (1..=15).contains(&value) {
            Ok(Self(value))
        } else {
            Err(PhoneError::InvalidConfiguration)
        }
    }
    pub fn get(self) -> u8 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhoneError {
    UnsupportedCharacter,
    InvalidPrefix,
    TooShort,
    TooLong,
    InvalidConfiguration,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PhoneNumber(String);
impl PhoneNumber {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn digits(&self) -> &str {
        self.0.strip_prefix('+').unwrap_or(&self.0)
    }
    pub fn is_international(&self) -> bool {
        self.0.starts_with('+')
    }
}
impl std::fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhoneRules {
    mode: PhoneNumberMode,
    minimum: PhoneDigitCount,
    maximum: PhoneDigitCount,
}
impl Default for PhoneRules {
    fn default() -> Self {
        Self::new(PhoneNumberMode::Either, 1, 15).expect("valid defaults")
    }
}
impl PhoneRules {
    /// Digit counts include the country calling code, but exclude the leading plus.
    pub fn new(mode: PhoneNumberMode, minimum: u8, maximum: u8) -> Result<Self, PhoneError> {
        let minimum = PhoneDigitCount::new(minimum)?;
        let maximum = PhoneDigitCount::new(maximum)?;
        if minimum.0 > maximum.0 {
            return Err(PhoneError::InvalidConfiguration);
        }
        Ok(Self {
            mode,
            minimum,
            maximum,
        })
    }
    pub fn exact(mode: PhoneNumberMode, digits: u8) -> Result<Self, PhoneError> {
        Self::new(mode, digits, digits)
    }
    pub fn validate(&self, text: &str) -> Result<Option<PhoneNumber>, PhoneError> {
        let value = normalize_phone(text)?;
        if value.is_empty() {
            return Ok(None);
        }
        let international = value.starts_with('+');
        if (self.mode == PhoneNumberMode::National && international)
            || (self.mode == PhoneNumberMode::International && !international)
            || (international && value.as_bytes().get(1) == Some(&b'0'))
        {
            return Err(PhoneError::InvalidPrefix);
        }
        let digits = value.len() - usize::from(international);
        if digits < usize::from(self.minimum.0) {
            return Err(PhoneError::TooShort);
        }
        if digits > usize::from(self.maximum.0) {
            return Err(PhoneError::TooLong);
        }
        Ok(Some(PhoneNumber(value)))
    }
    pub fn parse(&self, text: &str) -> DraftValue<PhoneNumber> {
        match self.validate(text) {
            Ok(Some(value)) => DraftValue::Parsed(value),
            Ok(None) => DraftValue::Empty,
            Err(PhoneError::TooShort) => DraftValue::Incomplete,
            Err(PhoneError::TooLong) => DraftValue::Invalid(ParseIssue::AboveMaximum),
            Err(PhoneError::InvalidConfiguration) => DraftValue::Invalid(ParseIssue::InvalidRange),
            Err(PhoneError::UnsupportedCharacter | PhoneError::InvalidPrefix) => {
                DraftValue::Invalid(ParseIssue::InvalidSyntax)
            }
        }
    }
}
/// Accept common visual separators on paste; never turn letters into digits or discard them.
/// National leading zeros are significant. An international plus is retained.
pub fn normalize_phone(text: &str) -> Result<String, PhoneError> {
    let mut value = String::with_capacity(text.len().min(32));
    for c in text
        .trim_matches(|c| matches!(c, ' ' | '\t' | '\r' | '\n'))
        .chars()
    {
        match c {
            '0'..='9' => value.push(c),
            '+' if value.is_empty() => value.push(c),
            '+' => return Err(PhoneError::InvalidPrefix),
            ' ' | '\u{a0}' | '\u{202f}' | '-' | '(' | ')' | '.' => (),
            _ => return Err(PhoneError::UnsupportedCharacter),
        }
    }
    Ok(value)
}
/// Native editing controls and shortcuts remain the editor's responsibility.
pub fn phone_insertion(text: &str) -> bool {
    let mut chars = text.chars();
    if let (Some(c), None) = (chars.next(), chars.next())
        && (c.is_control() || ('\u{e000}'..='\u{f8ff}').contains(&c))
    {
        return true;
    }
    text.bytes().all(|c| c.is_ascii_digit() || c == b'+')
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhoneDisplayMask {
    source: String,
    slots: usize,
    international: bool,
}
impl PhoneDisplayMask {
    /// `#` is a digit slot. Spaces, parentheses, dots and hyphens are presentation only.
    pub fn new(source: &str) -> Result<Self, PhoneError> {
        let slots = source.bytes().filter(|c| *c == b'#').count();
        if source.len() > 64
            || !(1..=15).contains(&slots)
            || !source
                .bytes()
                .all(|c| matches!(c, b'#' | b' ' | b'-' | b'(' | b')' | b'.' | b'+'))
            || source.bytes().filter(|c| *c == b'+').count() > usize::from(source.starts_with('+'))
        {
            return Err(PhoneError::InvalidConfiguration);
        }
        Ok(Self {
            source: source.into(),
            slots,
            international: source.starts_with('+'),
        })
    }
    pub fn format(&self, canonical: &str) -> String {
        let Ok(value) = normalize_phone(canonical) else {
            return canonical.into();
        };
        let digits = value.strip_prefix('+').unwrap_or(&value);
        // Never hide an overflowing digit or fabricate a missing digit/prefix.
        if digits.len() != self.slots || value.starts_with('+') != self.international {
            return canonical.into();
        }
        let mut digits = digits.chars();
        self.source
            .chars()
            .map(|c| {
                if c == '#' {
                    digits.next().expect("slot count checked")
                } else {
                    c
                }
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn value(rules: PhoneRules, text: &str) -> PhoneNumber {
        rules.validate(text).unwrap().unwrap()
    }
    #[test]
    fn canonical_values_preserve_national_zeros_and_international_prefixes() {
        let fr = PhoneRules::exact(PhoneNumberMode::National, 10).unwrap();
        assert_eq!(value(fr, "06 12 34 56 78").as_str(), "0612345678");
        let ca = PhoneRules::exact(PhoneNumberMode::International, 11).unwrap();
        let phone = value(ca, "+1 (514) 555-0123");
        assert_eq!(phone.as_str(), "+15145550123");
        assert_eq!(phone.digits(), "15145550123");
        assert!(phone.is_international());
    }
    #[test]
    fn letters_unknown_characters_and_extra_prefixes_are_never_silently_accepted() {
        let rules = PhoneRules::default();
        for text in [
            "abc",
            "123abc456",
            "+1 FLOWERS",
            "12+34",
            "++123",
            "+0123",
            "123/456",
            "123\n456",
        ] {
            assert!(rules.validate(text).is_err(), "{text}");
        }
        assert_eq!(
            normalize_phone("+33\u{202f}6\u{a0}1234-5678").unwrap(),
            "+33612345678"
        );
    }
    #[test]
    fn explicit_digit_bounds_and_modes_apply_to_typed_acceptance() {
        let rules = PhoneRules::exact(PhoneNumberMode::National, 10).unwrap();
        assert_eq!(rules.parse("123"), DraftValue::Incomplete);
        assert!(matches!(
            rules.parse("01234567890"),
            DraftValue::Invalid(ParseIssue::AboveMaximum)
        ));
        assert_eq!(
            rules.validate("+1234567890"),
            Err(PhoneError::InvalidPrefix)
        );
        let intl = PhoneRules::new(PhoneNumberMode::International, 8, 15).unwrap();
        assert!(intl.validate("0612345678").is_err());
        assert!(intl.validate("+33612345678").is_ok());
        assert!(intl.validate("+1234567890123456").is_err());
        assert!(PhoneRules::new(PhoneNumberMode::Either, 0, 15).is_err());
        assert!(PhoneRules::new(PhoneNumberMode::Either, 11, 10).is_err());
    }
    #[test]
    fn masks_are_visual_and_never_truncate_or_add_digits() {
        assert_eq!(
            PhoneDisplayMask::new("###-###-####")
                .unwrap()
                .format("5145550123"),
            "514-555-0123"
        );
        assert_eq!(
            PhoneDisplayMask::new("## ## ## ## ##")
                .unwrap()
                .format("0612345678"),
            "06 12 34 56 78"
        );
        assert_eq!(
            PhoneDisplayMask::new("+# (###) ###-####")
                .unwrap()
                .format("+15145550123"),
            "+1 (514) 555-0123"
        );
        let mask = PhoneDisplayMask::new("###-###-####").unwrap();
        assert_eq!(mask.format("012345678901"), "012345678901");
        assert_eq!(mask.format("123"), "123");
        assert_eq!(mask.format("+15145550123"), "+15145550123");
        for source in ["", "abc", "##+##", "################"] {
            assert!(PhoneDisplayMask::new(source).is_err());
        }
    }
    #[test]
    fn typing_is_strict_while_formatted_paste_can_be_normalized() {
        for text in ["123", "+", "\u{8}", "\u{f700}"] {
            assert!(phone_insertion(text));
        }
        for text in ["Ada", " ", "-", "()", "123 456", "e", "１２"] {
            assert!(!phone_insertion(text));
        }
        assert_eq!(normalize_phone("(514) 555-0123").unwrap(), "5145550123");
        assert!(normalize_phone("514 Ada 5550123").is_err());
    }
}

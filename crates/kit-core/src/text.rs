//! Bounded structured text parsers. Native text editing is never synthesized here.
use crate::{DraftValue, HtmlPattern, ParseIssue, grapheme_count};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OtpAlphabet {
    AsciiDigits,
    AsciiAlphanumeric,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OtpLength(u8);
impl OtpLength {
    pub fn new(value: u8) -> Result<Self, ParseIssue> {
        if (1..=64).contains(&value) {
            Ok(Self(value))
        } else {
            Err(ParseIssue::InvalidRange)
        }
    }
    pub fn get(self) -> u8 {
        self.0
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OtpValue(String);
impl OtpValue {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn parse(text: &str, length: OtpLength, alphabet: OtpAlphabet) -> DraftValue<Self> {
        if text.is_empty() {
            return DraftValue::Empty;
        }
        if !text.bytes().all(|value| match alphabet {
            OtpAlphabet::AsciiDigits => value.is_ascii_digit(),
            OtpAlphabet::AsciiAlphanumeric => value.is_ascii_alphanumeric(),
        }) || text.len() > usize::from(length.0)
        {
            return DraftValue::Invalid(ParseIssue::InvalidSyntax);
        }
        if text.len() < usize::from(length.0) {
            DraftValue::Incomplete
        } else {
            DraftValue::Parsed(Self(text.into()))
        }
    }
}
/// OTP input and acceptance rules sharing HTML's ECMAScript UnicodeSets pattern engine.
/// The pattern matches the complete draft, as the Slint field's `pattern` does.
/// Length counts UTF-16 units, matching HTML constraints. ASCII codes keep one glyph per slot.
#[derive(Clone, Debug)]
pub struct OtpRules {
    length: OtpLength,
    pattern: HtmlPattern,
}
impl OtpRules {
    pub fn new(length: OtpLength, pattern: HtmlPattern) -> Self {
        Self { length, pattern }
    }
    pub fn length(&self) -> OtpLength {
        self.length
    }
    /// A draft passes syntax and bounds; only `parse` can produce an accepted code.
    /// Use a pattern allowing partial drafts (for example `[A-Za-z0-9]*`) if desired.
    pub fn accepts_draft(&self, text: &str) -> bool {
        text.encode_utf16().count() <= usize::from(self.length.get())
            && (text.is_empty() || self.pattern.matches(text))
    }
    pub fn parse(&self, text: &str) -> DraftValue<OtpValue> {
        if text.is_empty() {
            DraftValue::Empty
        } else if !self.accepts_draft(text) {
            DraftValue::Invalid(ParseIssue::InvalidSyntax)
        } else if text.encode_utf16().count() < usize::from(self.length.get()) {
            DraftValue::Incomplete
        } else {
            DraftValue::Parsed(OtpValue(text.into()))
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaskToken {
    Literal(char),
    Digit,
    Letter,
    Alphanumeric,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaskPattern {
    tokens: Vec<MaskToken>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaskedValue(String);
impl MaskedValue {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl MaskPattern {
    pub fn new(tokens: Vec<MaskToken>) -> Result<Self, ParseIssue> {
        if tokens.is_empty()
            || tokens.len() > 256
            || tokens
                .iter()
                .all(|token| matches!(token, MaskToken::Literal(_)))
        {
            return Err(ParseIssue::InvalidSyntax);
        }
        Ok(Self { tokens })
    }
    pub fn tokens(&self) -> &[MaskToken] {
        &self.tokens
    }
    /// ASCII slots are deliberately bounded. Unicode user text belongs in TextField.
    /// Literals are supplied by the user/native editor; no caret-changing insertion occurs.
    pub fn parse(&self, text: &str) -> DraftValue<MaskedValue> {
        if text.is_empty() {
            return DraftValue::Empty;
        }
        let chars: Vec<char> = text.chars().collect();
        if chars.len() > self.tokens.len() {
            return DraftValue::Invalid(ParseIssue::InvalidSyntax);
        }
        if chars
            .iter()
            .zip(&self.tokens)
            .any(|(value, token)| !match token {
                MaskToken::Literal(expected) => value == expected,
                MaskToken::Digit => value.is_ascii_digit(),
                MaskToken::Letter => value.is_ascii_alphabetic(),
                MaskToken::Alphanumeric => value.is_ascii_alphanumeric(),
            })
        {
            return DraftValue::Invalid(ParseIssue::InvalidSyntax);
        }
        if chars.len() < self.tokens.len() {
            DraftValue::Incomplete
        } else {
            DraftValue::Parsed(MaskedValue(text.into()))
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphemeLimit(usize);
impl GraphemeLimit {
    pub fn new(maximum: usize) -> Self {
        Self(maximum)
    }
    pub fn maximum(self) -> usize {
        self.0
    }
    pub fn validate(self, text: &str) -> Result<(), ParseIssue> {
        if grapheme_count(text) > self.0 {
            Err(ParseIssue::AboveMaximum)
        } else {
            Ok(())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn otp_paste_incomplete_and_invalid() {
        let length = OtpLength::new(6).unwrap();
        assert_eq!(
            OtpValue::parse("12", length, OtpAlphabet::AsciiDigits),
            DraftValue::Incomplete
        );
        assert!(matches!(
            OtpValue::parse("123456", length, OtpAlphabet::AsciiDigits),
            DraftValue::Parsed(_)
        ));
        assert_eq!(
            OtpValue::parse("１２３４５６", length, OtpAlphabet::AsciiDigits),
            DraftValue::Invalid(ParseIssue::InvalidSyntax)
        );
    }
    #[test]
    fn otp_rules_share_html_patterns_and_keep_partial_drafts_editable() {
        let rules = OtpRules::new(
            OtpLength::new(6).unwrap(),
            HtmlPattern::compile("[A-Za-z0-9]*").unwrap(),
        );
        assert!(rules.accepts_draft(""));
        assert_eq!(rules.parse("AB1"), DraftValue::Incomplete);
        assert!(matches!(rules.parse("AB12cd"), DraftValue::Parsed(_)));
        assert!(!rules.accepts_draft("AB12cd7"));
        assert!(!rules.accepts_draft("AB 12c"));
        let grouped = OtpRules::new(
            OtpLength::new(7).unwrap(),
            HtmlPattern::compile("[A-Z]{2}[0-9]{5}").unwrap(),
        );
        assert!(matches!(grouped.parse("FG76657"), DraftValue::Parsed(_)));
        assert_eq!(
            grouped.parse("FG-76657"),
            DraftValue::Invalid(ParseIssue::InvalidSyntax)
        );
        let letters = OtpRules::new(
            OtpLength::new(2).unwrap(),
            HtmlPattern::compile("\\p{Letter}*").unwrap(),
        );
        assert!(matches!(letters.parse("éA"), DraftValue::Parsed(_)));
        assert_eq!(
            letters.parse("é1"),
            DraftValue::Invalid(ParseIssue::InvalidSyntax)
        );
    }
    #[test]
    fn bounded_mask_and_graphemes() {
        let mask = MaskPattern::new(vec![
            MaskToken::Digit,
            MaskToken::Digit,
            MaskToken::Literal('-'),
            MaskToken::Letter,
        ])
        .unwrap();
        assert_eq!(mask.parse("12-"), DraftValue::Incomplete);
        assert!(matches!(mask.parse("12-A"), DraftValue::Parsed(_)));
        assert_eq!(
            mask.parse("12A"),
            DraftValue::Invalid(ParseIssue::InvalidSyntax)
        );
        assert!(GraphemeLimit::new(1).validate("👩‍💻").is_ok());
    }
}

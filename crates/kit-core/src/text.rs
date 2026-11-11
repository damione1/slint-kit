//! Bounded structured text parsers. Native text editing is never synthesized here.
use crate::{DraftValue, HtmlPattern, OtpAlphabet, ParseIssue, grapheme_count};
impl OtpAlphabet {
    pub fn accepts(self, value: char) -> bool {
        match self {
            Self::AsciiDigits => value.is_ascii_digit(),
            Self::AsciiAlphanumeric => value.is_ascii_alphanumeric(),
        }
    }
    pub fn html_pattern_source(self) -> &'static str {
        match self {
            Self::AsciiDigits => "[0-9]*",
            Self::AsciiAlphanumeric => "[A-Za-z0-9]*",
        }
    }
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
        if !text.chars().all(|value| alphabet.accepts(value)) || text.len() > usize::from(length.0)
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
    /// Preserve custom Unicode patterns while normalizing copied separators before
    /// the same UTF-16 length and pattern boundary used for an ordinary draft.
    /// An empty result is a no-op for the editor, never a request to erase selection.
    pub fn normalize_paste(&self, text: &str) -> Result<OtpDraft, ParseIssue> {
        let mut normalized = String::new();
        let mut used = 0;
        for value in text.chars() {
            if otp_separator(value) {
                continue;
            }
            used += value.len_utf16();
            if used > usize::from(self.length.get()) {
                return Err(ParseIssue::AboveMaximum);
            }
            normalized.push(value);
        }
        if !normalized.is_empty() && !self.pattern.matches(&normalized) {
            return Err(ParseIssue::InvalidSyntax);
        }
        Ok(OtpDraft(normalized))
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
impl MaskToken {
    fn matches(self, value: char) -> bool {
        match self {
            Self::Literal(expected) => value == expected,
            Self::Digit => value.is_ascii_digit(),
            Self::Letter => value.is_ascii_alphabetic(),
            Self::Alphanumeric => value.is_ascii_alphanumeric(),
        }
    }
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
    const MAX_TOKENS: usize = 256;
    /// Maximum UTF-8 source size: each token can be an escaped four-byte scalar.
    pub const MAX_SOURCE_BYTES: usize = Self::MAX_TOKENS * 5;

    pub fn new(tokens: Vec<MaskToken>) -> Result<Self, ParseIssue> {
        if tokens.is_empty()
            || tokens.len() > Self::MAX_TOKENS
            || tokens
                .iter()
                .all(|token| matches!(token, MaskToken::Literal(_)))
        {
            return Err(ParseIssue::InvalidSyntax);
        }
        Ok(Self { tokens })
    }
    /// Compile a bounded mask string: `#` is an ASCII digit, `A` is an ASCII
    /// letter, `*` is an ASCII alphanumeric slot, `\` escapes the next
    /// character as a literal, and every other character is a literal.
    pub fn compile(source: &str) -> Result<Self, ParseIssue> {
        if source.len() > Self::MAX_SOURCE_BYTES {
            return Err(ParseIssue::InvalidSyntax);
        }
        let mut tokens = Vec::new();
        let mut chars = source.chars();
        while let Some(ch) = chars.next() {
            let token = match ch {
                '#' => MaskToken::Digit,
                'A' => MaskToken::Letter,
                '*' => MaskToken::Alphanumeric,
                '\\' => MaskToken::Literal(chars.next().ok_or(ParseIssue::InvalidSyntax)?),
                literal => MaskToken::Literal(literal),
            };
            tokens.push(token);
            if tokens.len() > Self::MAX_TOKENS {
                return Err(ParseIssue::InvalidSyntax);
            }
        }
        Self::new(tokens)
    }
    pub fn tokens(&self) -> &[MaskToken] {
        &self.tokens
    }
    /// UTF-16 code unit length of a complete value matching this mask.
    pub fn utf16_length(&self) -> u16 {
        let units: usize = self
            .tokens
            .iter()
            .map(|token| match token {
                MaskToken::Literal(value) => value.len_utf16(),
                MaskToken::Digit | MaskToken::Letter | MaskToken::Alphanumeric => 1,
            })
            .sum();
        u16::try_from(units).expect("bounded mask length fits in u16")
    }
    /// Visual placeholder using `_` for editable slots and preserving literals.
    pub fn placeholder(&self) -> String {
        self.tokens
            .iter()
            .map(|token| match token {
                MaskToken::Literal(value) => *value,
                MaskToken::Digit | MaskToken::Letter | MaskToken::Alphanumeric => '_',
            })
            .collect()
    }
    /// Full-value ECMAScript UnicodeSets pattern source for the shared HTML engine.
    pub fn html_pattern_source(&self) -> String {
        use std::fmt::Write as _;
        let mut pattern = String::new();
        for token in &self.tokens {
            match token {
                MaskToken::Digit => pattern.push_str("[0-9]"),
                MaskToken::Letter => pattern.push_str("[A-Za-z]"),
                MaskToken::Alphanumeric => pattern.push_str("[A-Za-z0-9]"),
                MaskToken::Literal(value) => {
                    let _ = write!(&mut pattern, "\\u{{{:X}}}", u32::from(*value));
                }
            }
        }
        pattern
    }
    /// Whether `text` is an empty, partial, or complete prefix matching this mask.
    pub fn accepts_draft(&self, text: &str) -> bool {
        let mut tokens = self.tokens.iter();
        for value in text.chars() {
            let Some(token) = tokens.next() else {
                return false;
            };
            if !token.matches(value) {
                return false;
            }
        }
        true
    }
    /// Longest prefix of `value` that satisfies this mask from the first token.
    pub fn valid_prefix<'a>(&self, value: &'a str) -> &'a str {
        let mut tokens = self.tokens.iter();
        for (index, ch) in value.char_indices() {
            let Some(token) = tokens.next() else {
                return &value[..index];
            };
            if !token.matches(ch) {
                return &value[..index];
            }
        }
        value
    }
    /// Whether replacing `current[anchor..cursor]` with `inserted` keeps a valid mask prefix.
    pub fn accepts_edit(
        &self,
        current: &str,
        anchor_byte_offset: usize,
        cursor_byte_offset: usize,
        inserted: &str,
    ) -> bool {
        let start = anchor_byte_offset.min(cursor_byte_offset);
        let end = anchor_byte_offset.max(cursor_byte_offset);
        if end > current.len() || !current.is_char_boundary(start) || !current.is_char_boundary(end)
        {
            return false;
        }
        let mut candidate = String::with_capacity(current.len() - (end - start) + inserted.len());
        candidate.push_str(&current[..start]);
        candidate.push_str(inserted);
        candidate.push_str(&current[end..]);
        if self.accepts_draft(&candidate) {
            return true;
        }
        // A native deletion or a host assignment may leave an invalid draft.
        // Permit incremental repair of its first mismatch without replacing
        // the suffix, caret or native history on the application's behalf.
        !self.accepts_draft(current)
            && self.valid_prefix(&candidate).chars().count()
                > self.valid_prefix(current).chars().count()
    }
    /// Whether a `key-pressed` text payload is either a non-text navigation/editing
    /// key or an insertion that keeps the draft a valid mask prefix.
    pub fn accepts_key_insertion(
        &self,
        current: &str,
        anchor_byte_offset: usize,
        cursor_byte_offset: usize,
        inserted: &str,
    ) -> bool {
        // Public key events can contain navigation codes or text delivered by
        // an input method. Filter ordinary ASCII insertions only; non-ASCII
        // input remains native and is checked by complete-draft validation.
        if inserted.is_empty() || !inserted.is_ascii() || inserted.chars().any(char::is_control) {
            return true;
        }
        self.accepts_edit(current, anchor_byte_offset, cursor_byte_offset, inserted)
    }
    /// ASCII slots are deliberately bounded. Unicode user text belongs in TextField.
    /// Literals are supplied by the user/native editor; no caret-changing insertion occurs.
    pub fn parse(&self, text: &str) -> DraftValue<MaskedValue> {
        if text.is_empty() {
            return DraftValue::Empty;
        }
        let mut count = 0usize;
        let mut tokens = self.tokens.iter();
        for value in text.chars() {
            let Some(token) = tokens.next() else {
                return DraftValue::Invalid(ParseIssue::InvalidSyntax);
            };
            if !token.matches(value) {
                return DraftValue::Invalid(ParseIssue::InvalidSyntax);
            }
            count += 1;
        }
        if count < self.tokens.len() {
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
    #[test]
    fn mask_compile_handles_slots_literals_escapes_and_bounds() {
        let mask = MaskPattern::compile("AA-####\\*\\#\\A\\\\").unwrap();
        assert_eq!(
            mask.tokens(),
            &[
                MaskToken::Letter,
                MaskToken::Letter,
                MaskToken::Literal('-'),
                MaskToken::Digit,
                MaskToken::Digit,
                MaskToken::Digit,
                MaskToken::Digit,
                MaskToken::Literal('*'),
                MaskToken::Literal('#'),
                MaskToken::Literal('A'),
                MaskToken::Literal('\\'),
            ]
        );
        assert_eq!(mask.utf16_length(), 11);
        assert_eq!(mask.placeholder(), "__-____*#A\\");
        let pattern = HtmlPattern::compile(&mask.html_pattern_source()).unwrap();
        assert!(pattern.matches("Ab-1234*#A\\"));
        assert!(!pattern.matches("Ab-1234"));
        assert!(!pattern.matches("Ab_1234*#A\\"));

        let max_mask = "#".repeat(256);
        assert!(MaskPattern::compile(&max_mask).is_ok());
        assert_eq!(
            MaskPattern::compile(&"#".repeat(257)),
            Err(ParseIssue::InvalidSyntax)
        );
        assert_eq!(MaskPattern::compile(""), Err(ParseIssue::InvalidSyntax));
        assert_eq!(MaskPattern::compile("---"), Err(ParseIssue::InvalidSyntax));
        assert_eq!(
            MaskPattern::compile("\\#\\A\\*"),
            Err(ParseIssue::InvalidSyntax)
        );
        assert_eq!(MaskPattern::compile("#\\"), Err(ParseIssue::InvalidSyntax));

        let astral = MaskPattern::compile("🔑*").unwrap();
        assert_eq!(astral.utf16_length(), 3);
        assert_eq!(astral.placeholder(), "🔑_");
        assert!(astral.accepts_draft("🔑"));
        assert!(astral.accepts_draft("🔑9"));
        assert!(!astral.accepts_draft("🔑!"));
        assert_eq!(
            astral.parse("🔑Z"),
            DraftValue::Parsed(MaskedValue("🔑Z".into()))
        );
    }
    #[test]
    fn mask_prefix_edits_preserve_native_drafts_and_allow_incremental_repairs() {
        let mask = MaskPattern::compile("AA-####").unwrap();
        assert_eq!(mask.valid_prefix("AB-12x9"), "AB-12");
        assert_eq!(mask.valid_prefix("AB-123456"), "AB-1234");
        assert!(mask.accepts_edit("AB-", 3, 3, "12"));
        assert!(!mask.accepts_edit("AB-", 3, 3, "XY"));
        assert!(mask.accepts_edit("AB-1234", 3, 7, "9876"));
        assert!(!mask.accepts_edit("AB-1234", 0, 2, "12"));
        assert!(mask.accepts_key_insertion("AB-12", 5, 5, "\u{8}"));
        assert!(mask.accepts_key_insertion("AB-12", 5, 5, "\u{f700}"));
        assert!(!mask.accepts_key_insertion("AB-12", 5, 5, "Z"));

        assert!(mask.accepts_edit("-12", 0, 0, "A"));
        assert!(mask.accepts_edit("A-12", 1, 1, "B"));
        assert!(!mask.accepts_edit("-12", 0, 0, "9"));
        assert!(mask.accepts_key_insertion("AB-12", 5, 5, "界"));
        assert!(!mask.accepts_draft("AB-12界"));
    }
}

/// Plain application text, separate from identifiers and validated addresses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextValue(String);
impl TextValue {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
/// A single address accepted by the shared HTML email constraint engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmailAddress(String);
impl EmailAddress {
    pub fn parse(text: &str) -> DraftValue<Self> {
        validated_text(text, crate::HtmlInputKind::Email, Self)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
/// An absolute URL accepted by the shared HTML URL constraint engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebUrl(String);
impl WebUrl {
    pub fn parse(text: &str) -> DraftValue<Self> {
        validated_text(text, crate::HtmlInputKind::Url, Self)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
fn validated_text<T>(
    text: &str,
    kind: crate::HtmlInputKind,
    construct: impl FnOnce(String) -> T,
) -> DraftValue<T> {
    if text.is_empty() {
        return DraftValue::Empty;
    }
    let constraints = crate::HtmlConstraints {
        kind,
        ..Default::default()
    };
    let canonical = constraints.sanitize(text);
    if constraints
        .validate(&canonical, crate::HtmlInputState::default())
        .valid()
    {
        DraftValue::Parsed(construct(canonical))
    } else {
        DraftValue::Invalid(ParseIssue::InvalidSyntax)
    }
}

/// An OTP draft validated at its editing boundary. Alphabet-based constructors
/// enforce ASCII; OtpRules retains its authored Unicode pattern. Empty and partial
/// drafts stay editable; only OtpValue or OtpRules parsing accepts a complete code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OtpDraft(String);
impl OtpDraft {
    pub fn parse(text: &str, length: OtpLength, alphabet: OtpAlphabet) -> Result<Self, ParseIssue> {
        if !text.chars().all(|value| alphabet.accepts(value)) {
            return Err(ParseIssue::InvalidSyntax);
        }
        if text.len() > usize::from(length.get()) {
            return Err(ParseIssue::AboveMaximum);
        }
        Ok(Self(text.into()))
    }
    /// Strip only whitespace and visual hyphens from copied codes, before the
    /// same strict alphabet/length boundary used for ordinary input.
    pub fn normalize_paste(
        text: &str,
        length: OtpLength,
        alphabet: OtpAlphabet,
    ) -> Result<Self, ParseIssue> {
        let mut normalized = String::new();
        for value in text.chars() {
            if otp_separator(value) {
                continue;
            }
            if !alphabet.accepts(value) {
                return Err(ParseIssue::InvalidSyntax);
            }
            normalized.push(value);
            if normalized.len() > usize::from(length.get()) {
                return Err(ParseIssue::AboveMaximum);
            }
        }
        Ok(Self(normalized))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn otp_separator(value: char) -> bool {
    value.is_whitespace()
        || matches!(
            value,
            '-' | '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2212}'
        )
}

#[cfg(test)]
mod otp_editing_tests {
    use super::*;
    #[test]
    fn copied_separators_normalize_before_the_strict_length_boundary() {
        let length = OtpLength::new(6).unwrap();
        for source in [
            "123 456",
            "123-456",
            "123\u{2011}456",
            " 123\n456\t",
            "123\u{00a0}456",
            "123\u{202f}456",
        ] {
            let draft =
                OtpDraft::normalize_paste(source, length, OtpAlphabet::AsciiDigits).unwrap();
            assert_eq!(draft.as_str(), "123456");
            assert!(matches!(
                OtpValue::parse(draft.as_str(), length, OtpAlphabet::AsciiDigits),
                DraftValue::Parsed(_)
            ));
        }
        assert_eq!(
            OtpDraft::normalize_paste("12 3", length, OtpAlphabet::AsciiDigits)
                .unwrap()
                .as_str(),
            "123"
        );
        assert_eq!(
            OtpDraft::normalize_paste(" --- ", length, OtpAlphabet::AsciiDigits)
                .unwrap()
                .as_str(),
            ""
        );
    }
    #[test]
    fn copied_codes_keep_custom_unicode_rules_and_utf16_bounds() {
        let letters = OtpRules::new(
            OtpLength::new(2).unwrap(),
            HtmlPattern::compile("\\p{Letter}*").unwrap(),
        );
        let draft = letters.normalize_paste("é - A").unwrap();
        assert_eq!(draft.as_str(), "éA");
        assert!(matches!(
            letters.parse(draft.as_str()),
            DraftValue::Parsed(_)
        ));
        assert_eq!(
            letters.normalize_paste("é - 1"),
            Err(ParseIssue::InvalidSyntax)
        );
        assert_eq!(
            letters.normalize_paste("é - AB"),
            Err(ParseIssue::AboveMaximum)
        );
        let emoji = OtpRules::new(
            OtpLength::new(2).unwrap(),
            HtmlPattern::compile(".*").unwrap(),
        );
        assert_eq!(emoji.normalize_paste("😀 - ").unwrap().as_str(), "😀");
        assert_eq!(
            emoji.normalize_paste("😀 - A"),
            Err(ParseIssue::AboveMaximum)
        );
    }
    #[test]
    fn normalization_never_drops_letters_or_extra_digits_or_punctuation() {
        let length = OtpLength::new(6).unwrap();
        for source in ["12A456", "１２３４５６", "123.456", "123/456"] {
            assert_eq!(
                OtpDraft::normalize_paste(source, length, OtpAlphabet::AsciiDigits),
                Err(ParseIssue::InvalidSyntax)
            );
        }
        assert_eq!(
            OtpDraft::normalize_paste("123 4567", length, OtpAlphabet::AsciiDigits),
            Err(ParseIssue::AboveMaximum)
        );
        assert_eq!(
            OtpDraft::normalize_paste("AB-12 cD", length, OtpAlphabet::AsciiAlphanumeric)
                .unwrap()
                .as_str(),
            "AB12cD"
        );
    }
    #[test]
    fn shared_alphabet_produces_the_same_complete_html_rule() {
        for alphabet in [OtpAlphabet::AsciiDigits, OtpAlphabet::AsciiAlphanumeric] {
            let pattern = HtmlPattern::compile(alphabet.html_pattern_source()).unwrap();
            for input in ["", "123", "AB12", "éA", "12-3", "1.2"] {
                assert_eq!(
                    pattern.matches(input),
                    OtpDraft::parse(input, OtpLength::new(64).unwrap(), alphabet).is_ok()
                );
            }
        }
    }
}

// SPDX-License-Identifier: GPL-3.0-or-later
//! HTML Living Standard constraint validation, independent of the renderer.
//! https://html.spec.whatwg.org/multipage/input.html
use crate::{HtmlInputKind, NumberInputFormat};
use chrono::NaiveDate;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::Zero;

/// Each flag is independent, as in HTML's ValidityState.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HtmlValidity {
    pub value_missing: bool,
    pub type_mismatch: bool,
    pub pattern_mismatch: bool,
    pub too_long: bool,
    pub too_short: bool,
    pub range_underflow: bool,
    pub range_overflow: bool,
    pub step_mismatch: bool,
    pub bad_input: bool,
    pub custom_error: bool,
}
impl HtmlValidity {
    pub fn valid(self) -> bool {
        self == Self::default()
    }
}

/// HTML patterns use ECMAScript UnicodeSets (`v`), not Rust regex syntax.
/// Compile application-authored patterns once; invalid syntax is an author error.
#[derive(Clone, Debug)]
pub struct HtmlPattern(regress::Regex);
impl HtmlPattern {
    pub fn compile(source: &str) -> Result<Self, regress::Error> {
        let flags = regress::Flags {
            unicode_sets: true,
            ..Default::default()
        };
        // Validate before adding the full-value anchors (HTML's specified order).
        regress::Regex::with_flags(source, flags)?;
        regress::Regex::with_flags(&format!("^(?:{source})$"), flags).map(Self)
    }
    pub fn matches(&self, value: &str) -> bool {
        self.0
            .find(value)
            .is_some_and(|m| m.range() == (0..value.len()))
    }
}

#[derive(Clone, Debug)]
pub struct HtmlConstraints {
    pub kind: HtmlInputKind,
    pub number_format: NumberInputFormat,
    /// Native twelve-hour presentation; values and constraints remain canonical.
    pub twelve_hour: bool,
    pub required: bool,
    pub multiple: bool,
    /// HTML lengths count UTF-16 code units, independently of a grapheme counter.
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<HtmlPattern>,
    pub minimum: Option<String>,
    pub maximum: Option<String>,
    /// None means the type's default; Some("any") disables step validation.
    pub step: Option<String>,
    /// The initial value attribute is the step base when min is absent.
    pub default_value: String,
    pub custom_error: String,
    pub disabled: bool,
    pub read_only: bool,
}
impl Default for HtmlConstraints {
    fn default() -> Self {
        Self {
            kind: HtmlInputKind::Text,
            number_format: NumberInputFormat::Canonical,
            twelve_hour: false,
            required: false,
            multiple: false,
            min_length: None,
            max_length: None,
            pattern: None,
            minimum: None,
            maximum: None,
            step: None,
            default_value: String::new(),
            custom_error: String::new(),
            disabled: false,
            read_only: false,
        }
    }
}
/// Flags distinguish an editor draft from a value assigned by the application.
#[derive(Clone, Copy, Debug, Default)]
pub struct HtmlInputState {
    pub user_edited: bool,
    pub checked: bool,
    /// Selected options/files; for a single select pass zero for its empty placeholder.
    pub selected_count: usize,
}
fn ascii_space(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\u{c}' | '\r' | ' ')
}
impl HtmlConstraints {
    pub fn will_validate(&self) -> bool {
        !self.disabled && !(self.read_only && self.readonly_applies())
    }
    fn readonly_applies(&self) -> bool {
        !matches!(
            self.kind,
            HtmlInputKind::Checkbox
                | HtmlInputKind::Radio
                | HtmlInputKind::Select
                | HtmlInputKind::File
                | HtmlInputKind::Range
                | HtmlInputKind::Color
        )
    }
    fn text_constraints_apply(&self) -> bool {
        matches!(
            self.kind,
            HtmlInputKind::Text
                | HtmlInputKind::Search
                | HtmlInputKind::Telephone
                | HtmlInputKind::Email
                | HtmlInputKind::Url
                | HtmlInputKind::Password
                | HtmlInputKind::TextArea
        )
    }
    /// This returns the submission value without assigning to the native editor.
    pub fn sanitize(&self, draft: &str) -> String {
        if self.kind == HtmlInputKind::TextArea {
            return draft.replace("\r\n", "\n").replace('\r', "\n");
        }
        if numeric_policy(self.kind).is_some() {
            let candidate = if self.kind == HtmlInputKind::Number {
                canonical_number_draft(draft, self.number_format)
            } else if self.twelve_hour && self.kind == HtmlInputKind::Time {
                canonical_clock_draft(draft)
            } else if self.twelve_hour && self.kind == HtmlInputKind::DateTimeLocal {
                draft
                    .split_once(['T', ' '])
                    .and_then(|(d, t)| canonical_clock_draft(t).map(|t| format!("{d}T{t}")))
            } else {
                Some(draft.into())
            };
            let value = candidate
                .filter(|v| numeric_value(self.kind, v).is_some())
                .unwrap_or_default();
            if self.kind == HtmlInputKind::DateTimeLocal
                && let Some((date, time)) = value.split_once(['T', ' '])
            {
                let mut time = time.to_string();
                if time.contains('.') {
                    time = time.trim_end_matches('0').trim_end_matches('.').into();
                }
                if time.len() == 8 && time.ends_with(":00") {
                    time.truncate(5);
                }
                return format!("{date}T{time}");
            }
            return value;
        }
        let value = draft.replace(['\r', '\n'], "");
        match self.kind {
            HtmlInputKind::Email if self.multiple => value
                .split(',')
                .map(|s| s.trim_matches(ascii_space))
                .collect::<Vec<_>>()
                .join(","),
            HtmlInputKind::Email | HtmlInputKind::Url => value.trim_matches(ascii_space).into(),
            HtmlInputKind::Text
            | HtmlInputKind::Search
            | HtmlInputKind::Telephone
            | HtmlInputKind::Password
            | HtmlInputKind::TextArea
            | HtmlInputKind::Number
            | HtmlInputKind::Date
            | HtmlInputKind::Month
            | HtmlInputKind::Week
            | HtmlInputKind::Time
            | HtmlInputKind::DateTimeLocal
            | HtmlInputKind::Checkbox
            | HtmlInputKind::Radio
            | HtmlInputKind::Select
            | HtmlInputKind::File
            | HtmlInputKind::Range
            | HtmlInputKind::Color => value,
        }
    }
    pub fn validate(&self, draft: &str, state: HtmlInputState) -> HtmlValidity {
        // Disabled/read-only controls can have invalid values but are barred from
        // validation. Consumers use will_validate(), including form aggregation.
        let value = self.sanitize(draft);
        let mut result = HtmlValidity {
            custom_error: !self.custom_error.is_empty(),
            bad_input: !draft.is_empty() && numeric_policy(self.kind).is_some() && value.is_empty(),
            ..Default::default()
        };
        let empty = value.is_empty();
        result.value_missing = self.required
            && !self.disabled
            && !(self.read_only && self.readonly_applies())
            && match self.kind {
                HtmlInputKind::Checkbox | HtmlInputKind::Radio => !state.checked,
                HtmlInputKind::Select | HtmlInputKind::File => state.selected_count == 0,
                HtmlInputKind::Range | HtmlInputKind::Color => false,
                HtmlInputKind::Text
                | HtmlInputKind::Search
                | HtmlInputKind::Telephone
                | HtmlInputKind::Email
                | HtmlInputKind::Url
                | HtmlInputKind::Password
                | HtmlInputKind::TextArea
                | HtmlInputKind::Number
                | HtmlInputKind::Date
                | HtmlInputKind::Month
                | HtmlInputKind::Week
                | HtmlInputKind::Time
                | HtmlInputKind::DateTimeLocal => empty,
            };
        if empty {
            return result;
        }
        result.type_mismatch = match self.kind {
            HtmlInputKind::Email if self.multiple => {
                value.split(',').any(|v| !html_email_address(v))
            }
            HtmlInputKind::Email => !html_email_address(&value),
            HtmlInputKind::Url => url::Url::parse(&value).is_err(),
            HtmlInputKind::Text
            | HtmlInputKind::Search
            | HtmlInputKind::Telephone
            | HtmlInputKind::Password
            | HtmlInputKind::TextArea
            | HtmlInputKind::Number
            | HtmlInputKind::Date
            | HtmlInputKind::Month
            | HtmlInputKind::Week
            | HtmlInputKind::Time
            | HtmlInputKind::DateTimeLocal
            | HtmlInputKind::Checkbox
            | HtmlInputKind::Radio
            | HtmlInputKind::Select
            | HtmlInputKind::File
            | HtmlInputKind::Range
            | HtmlInputKind::Color => false,
        };
        if self.text_constraints_apply() {
            let length = value.encode_utf16().count();
            if state.user_edited {
                result.too_short = self.min_length.is_some_and(|min| length < min);
                result.too_long = self.max_length.is_some_and(|max| length > max);
            }
            if self.kind != HtmlInputKind::TextArea
                && let Some(pattern) = &self.pattern
            {
                result.pattern_mismatch = if self.kind == HtmlInputKind::Email && self.multiple {
                    value.split(',').any(|v| !pattern.matches(v))
                } else {
                    !pattern.matches(&value)
                };
            }
        }
        if let Some((default_step, factor)) = numeric_policy(self.kind) {
            let parsed = numeric_value(self.kind, &value);
            result.bad_input = parsed.is_none();
            if let Some(number) = parsed {
                let min = self
                    .minimum
                    .as_deref()
                    .and_then(|v| numeric_value(self.kind, v));
                let max = self
                    .maximum
                    .as_deref()
                    .and_then(|v| numeric_value(self.kind, v));
                if self.kind == HtmlInputKind::Time
                    && matches!((&min,&max),(Some(a),Some(b)) if a>b)
                {
                    // HTML time is periodic: e.g. 23:00..02:00 crosses midnight.
                    let outside = min.as_ref().is_some_and(|m| number < *m)
                        && max.as_ref().is_some_and(|m| number > *m);
                    result.range_underflow = outside;
                    result.range_overflow = outside;
                } else {
                    result.range_underflow = min.as_ref().is_some_and(|m| number < *m);
                    result.range_overflow = max.as_ref().is_some_and(|m| number > *m);
                }
                if !self
                    .step
                    .as_deref()
                    .is_some_and(|s| s.eq_ignore_ascii_case("any"))
                {
                    let step = self
                        .step
                        .as_deref()
                        .and_then(html_number)
                        .filter(|s| s > &BigRational::zero())
                        .unwrap_or_else(|| BigRational::from_integer(default_step.into()))
                        * BigRational::from_integer(factor.into());
                    let base = min
                        .or_else(|| numeric_value(self.kind, &self.default_value))
                        .unwrap_or_else(|| {
                            BigRational::from_integer(if self.kind == HtmlInputKind::Week {
                                (-259_200_000_i64).into()
                            } else {
                                0.into()
                            })
                        });
                    result.step_mismatch = !((number - base) / step).is_integer();
                }
            }
        }
        result
    }
}
/// Definitively invalid email characters can be shown while an otherwise
/// incomplete address remains a normal draft until blur.
pub fn html_email_hard_error(value: &str, multiple: bool) -> bool {
    let tokens: Vec<_> = if multiple {
        value.split(',').collect()
    } else {
        vec![value]
    };
    tokens.into_iter().any(|token| {
        let token = token.trim_matches(ascii_space);
        let mut parts = token.split('@');
        let local = parts.next().unwrap_or("");
        let domain = parts.next();
        parts.next().is_some()
            || local
                .bytes()
                .any(|c| !c.is_ascii_alphanumeric() && !b".!#$%&'*+/=?^_`{|}~-".contains(&c))
            || domain.is_some_and(|d| {
                local.is_empty()
                    || d.starts_with(['.', '-'])
                    || d.contains("..")
                    || d.bytes()
                        .any(|c| !c.is_ascii_alphanumeric() && !b".-".contains(&c))
                    || d.split('.').any(|p| p.len() > 63)
            })
    })
}
/// The HTML email grammar deliberately differs from RFC 5322 mailbox parsing.
/// In particular a domain without a dot is allowed; quoted local parts are not.
pub fn html_email_address(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && local
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".!#$%&'*+/=?^_`{|}~-".contains(&c))
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label.as_bytes()[0].is_ascii_alphanumeric()
                && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
                && label
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        })
}
/// Localized editing is a projection; min/max/step always use HTML syntax.
fn canonical_clock_draft(draft: &str) -> Option<String> {
    for precision in [crate::TimePrecision::Minute, crate::TimePrecision::Second] {
        if let crate::DraftValue::Parsed(time) =
            crate::LocalTime::parse_with_format(draft, precision, crate::ClockFormat::TwelveHour)
        {
            return Some(time.format(precision, crate::ClockFormat::TwentyFourHour));
        }
    }
    None
}
fn canonical_number_draft(draft: &str, format: NumberInputFormat) -> Option<String> {
    if format == NumberInputFormat::Canonical {
        return Some(draft.into());
    }
    let (mantissa, exponent) = draft
        .find(['e', 'E'])
        .map_or((draft, ""), |i| (&draft[..i], &draft[i..]));
    let mut decimal = mantissa.split(',');
    let integer = decimal.next()?;
    let fraction = decimal.next();
    if decimal.next().is_some() || integer.contains('.') {
        return None;
    }
    let digits = integer.strip_prefix('-').unwrap_or(integer);
    if digits.contains([' ', '\u{a0}', '\u{202f}']) {
        let groups: Vec<_> = digits.split([' ', '\u{a0}', '\u{202f}']).collect();
        if groups[0].is_empty()
            || groups[0].len() > 3
            || !groups[0].bytes().all(|c| c.is_ascii_digit())
            || groups[1..]
                .iter()
                .any(|g| g.len() != 3 || !g.bytes().all(|c| c.is_ascii_digit()))
        {
            return None;
        }
    }
    let integer = integer.replace([' ', '\u{a0}', '\u{202f}'], "");
    Some(format!(
        "{integer}{}{exponent}",
        fraction.map_or(String::new(), |f| format!(".{f}"))
    ))
}
/// Filter insertions without owning native selection, caret, composition or undo.
/// Incomplete exponent/sign drafts remain editable and validate as badInput.
pub fn html_numeric_insertion(text: &str) -> bool {
    text.chars().all(|c| {
        c.is_ascii_digit()
            || matches!(
                c,
                '.' | ',' | '+' | '-' | 'e' | 'E' | ' ' | '\u{a0}' | '\u{202f}'
            )
            || (text.chars().count() == 1
                && (c.is_control() || ('\u{e000}'..='\u{f8ff}').contains(&c)))
    })
}
fn numeric_policy(kind: HtmlInputKind) -> Option<(i64, i64)> {
    match kind {
        HtmlInputKind::Number | HtmlInputKind::Range | HtmlInputKind::Month => Some((1, 1)),
        HtmlInputKind::Date => Some((1, 86_400_000)),
        HtmlInputKind::Week => Some((1, 604_800_000)),
        HtmlInputKind::Time | HtmlInputKind::DateTimeLocal => Some((60, 1000)),
        HtmlInputKind::Text
        | HtmlInputKind::Search
        | HtmlInputKind::Telephone
        | HtmlInputKind::Email
        | HtmlInputKind::Url
        | HtmlInputKind::Password
        | HtmlInputKind::TextArea
        | HtmlInputKind::Checkbox
        | HtmlInputKind::Radio
        | HtmlInputKind::Select
        | HtmlInputKind::File
        | HtmlInputKind::Color => None,
    }
}
/// Parse HTML's number grammar, with exact decimal arithmetic for step checks.
/// The finite f64 check follows HTML's number conversion; it is never used for money.
fn html_number(s: &str) -> Option<BigRational> {
    let bytes = s.as_bytes();
    let mut i = usize::from(bytes.first() == Some(&b'-'));
    let start = i;
    while bytes.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    let integer_digits = i - start;
    let mut fraction = 0;
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        fraction = i - start;
        if fraction == 0 {
            return None;
        }
    }
    if integer_digits + fraction == 0 {
        return None;
    }
    let coefficient_end = i;
    let mut exponent_text = "0";
    if matches!(bytes.get(i), Some(b'e' | b'E')) {
        i += 1;
        let start = i;
        if matches!(bytes.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let digits = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == digits {
            return None;
        }
        exponent_text = &s[start..i];
    }
    if i != bytes.len() {
        return None;
    }
    let converted = s.parse::<f64>().ok()?;
    if !converted.is_finite() {
        return None;
    }
    if converted == 0.0 {
        return Some(BigRational::zero());
    }
    let exponent = exponent_text.parse::<i32>().ok()?;
    let coefficient = s[..coefficient_end]
        .replace('.', "")
        .parse::<BigInt>()
        .ok()?;
    let power = exponent.checked_sub(i32::try_from(fraction).ok()?)?;
    let scale = BigInt::from(10).pow(power.unsigned_abs());
    Some(if power >= 0 {
        BigRational::from_integer(coefficient * scale)
    } else {
        BigRational::new(coefficient, scale)
    })
}
fn numeric_value(kind: HtmlInputKind, s: &str) -> Option<BigRational> {
    if s.is_empty() {
        return None;
    }
    let integer = |v: i64| Some(BigRational::from_integer(v.into()));
    match kind {
        HtmlInputKind::Number | HtmlInputKind::Range => html_number(s),
        HtmlInputKind::Date => integer(
            html_date(s)?
                .signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1)?)
                .num_days()
                * 86_400_000,
        ),
        HtmlInputKind::Month => {
            let (y, m) = s.rsplit_once('-')?;
            let y = html_year(y)?;
            let m = two_digits(m)?;
            if !(1..=12).contains(&m) {
                return None;
            }
            integer((i64::from(y) - 1970) * 12 + i64::from(m) - 1)
        }
        HtmlInputKind::Week => {
            let (y, w) = s.split_once("-W")?;
            let date =
                NaiveDate::from_isoywd_opt(html_year(y)?, two_digits(w)?, chrono::Weekday::Mon)?;
            integer(
                date.signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1)?)
                    .num_days()
                    * 86_400_000,
            )
        }
        HtmlInputKind::Time => html_time(s),
        HtmlInputKind::DateTimeLocal => {
            let (d, t) = s.split_once(['T', ' '])?;
            Some(
                BigRational::from_integer(
                    (html_date(d)?
                        .signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1)?)
                        .num_days()
                        * 86_400_000)
                        .into(),
                ) + html_time(t)?,
            )
        }
        HtmlInputKind::Text
        | HtmlInputKind::Search
        | HtmlInputKind::Telephone
        | HtmlInputKind::Email
        | HtmlInputKind::Url
        | HtmlInputKind::Password
        | HtmlInputKind::TextArea
        | HtmlInputKind::Checkbox
        | HtmlInputKind::Radio
        | HtmlInputKind::Select
        | HtmlInputKind::File
        | HtmlInputKind::Color => None,
    }
}
fn html_year(s: &str) -> Option<i32> {
    if s.len() < 4 || !s.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    s.parse().ok().filter(|y| *y > 0)
}
fn two_digits(s: &str) -> Option<u32> {
    if s.len() != 2 || !s.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}
fn html_date(s: &str) -> Option<NaiveDate> {
    let (y, md) = s.split_once('-')?;
    let (m, d) = md.split_once('-')?;
    NaiveDate::from_ymd_opt(html_year(y)?, two_digits(m)?, two_digits(d)?)
}
fn html_time(s: &str) -> Option<BigRational> {
    let mut parts = s.split(':');
    let h = two_digits(parts.next()?)?;
    let m = two_digits(parts.next()?)?;
    if h > 23 || m > 59 {
        return None;
    }
    let seconds = match parts.next() {
        Some(s) => {
            let (whole, fraction) = s.split_once('.').map_or((s, None), |(a, b)| (a, Some(b)));
            if two_digits(whole)? > 59
                || fraction.is_some_and(|f| {
                    f.is_empty() || f.len() > 3 || !f.bytes().all(|c| c.is_ascii_digit())
                })
            {
                return None;
            }
            html_number(s)?
        }
        None => BigRational::zero(),
    };
    if parts.next().is_some() {
        return None;
    }
    Some(
        (BigRational::from_integer((h * 3600 + m * 60).into()) + seconds)
            * BigRational::from_integer(1000.into()),
    )
}

/// Form aggregation keeps stable application keys and respects barred controls.
#[derive(Clone, Debug)]
pub struct HtmlFormValidation<K> {
    fields: Vec<(K, bool, HtmlValidity)>,
}
impl<K> Default for HtmlFormValidation<K> {
    fn default() -> Self {
        Self { fields: Vec::new() }
    }
}
impl<K: Eq> HtmlFormValidation<K> {
    pub fn update(&mut self, key: K, will_validate: bool, validity: HtmlValidity) {
        if let Some(row) = self.fields.iter_mut().find(|(k, _, _)| k == &key) {
            *row = (key, will_validate, validity);
        } else {
            self.fields.push((key, will_validate, validity));
        }
    }
    pub fn remove(&mut self, key: &K) {
        self.fields.retain(|(k, _, _)| k != key);
    }
    pub fn invalid_fields(&self) -> impl Iterator<Item = &K> {
        self.fields
            .iter()
            .filter(|(_, will, v)| *will && !v.valid())
            .map(|(k, _, _)| k)
    }
    pub fn first_invalid(&self) -> Option<&K> {
        self.invalid_fields().next()
    }
    pub fn can_submit(&self, no_validate: bool) -> bool {
        no_validate || self.first_invalid().is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rules(kind: HtmlInputKind) -> HtmlConstraints {
        HtmlConstraints {
            kind,
            ..Default::default()
        }
    }
    fn validate(c: &HtmlConstraints, s: &str) -> HtmlValidity {
        c.validate(
            s,
            HtmlInputState {
                user_edited: true,
                ..Default::default()
            },
        )
    }
    #[test]
    fn email_uses_html_grammar_including_domain_labels_and_local_punctuation() {
        let c = rules(HtmlInputKind::Email);
        for s in [
            "a@example.com",
            "a@machine",
            "a..b@example.com",
            "a+b!#$%&'*-/=?^_`{|}~@example.com",
            "a@xn--9ca.fr",
        ] {
            assert!(validate(&c, s).valid(), "{s}");
        }
        for s in [
            "alex",
            "alex@",
            "@example.com",
            "a@@example.com",
            "a b@example.com",
            "a@-bad.fr",
            "a@bad-.fr",
            "a@x..fr",
            "a@é.fr",
            "\"a\"@example.com",
            "a@example.com,b@example.net",
        ] {
            assert!(validate(&c, s).type_mismatch, "{s}");
        }
        assert!(validate(&c, &format!("a@{}.com", "x".repeat(64))).type_mismatch);
    }
    #[test]
    fn optional_required_multiple_and_email_sanitization_are_independent() {
        let mut c = rules(HtmlInputKind::Email);
        assert!(validate(&c, "").valid());
        c.required = true;
        assert!(validate(&c, "").value_missing);
        c.multiple = true;
        assert!(validate(&c, "").value_missing);
        assert!(validate(&c, " a@example.com,\t b@example.net ").valid());
        assert!(validate(&c, "a@example.com,").type_mismatch);
        assert_eq!(
            c.sanitize(" a@example.com,\t b@example.net \r\n"),
            "a@example.com,b@example.net"
        );
        assert!(validate(&c, " a@example.com ").valid());
    }
    #[test]
    fn urls_require_absolute_parseable_values_without_dns_or_http_only_policy() {
        let c = rules(HtmlInputKind::Url);
        for s in [
            "https://example.com",
            "http://localhost",
            "mailto:a@example.com",
            "foo:bar",
            "https://example.com@evil.example",
        ] {
            assert!(validate(&c, s).valid(), "{s}");
        }
        for s in [
            "example.com",
            "/relative",
            "https://",
            "https://example.comddf.dofmo@",
            "https://[:::]/",
        ] {
            assert!(validate(&c, s).type_mismatch, "{s}");
        }
    }
    #[test]
    fn patterns_use_v_sets_lookarounds_backreferences_and_full_value_anchors() {
        for (pattern, yes, no) in [
            (r"[[a-z]&&[^aeiou]]+", "bcdf", "abc"),
            (r"(a)\1", "aa", "ab"),
            (r"(?=.{3}$)[A-Z]+", "ABC", "AB"),
            (r"\p{Letter}+", "Été", "123"),
        ] {
            let p = HtmlPattern::compile(pattern).unwrap();
            assert!(p.matches(yes), "{pattern}");
            assert!(!p.matches(no), "{pattern}");
        }
        assert!(HtmlPattern::compile("[").is_err());
        assert!(!HtmlPattern::compile("abc").unwrap().matches("xabcx"));
        assert!(!HtmlPattern::compile("abc").unwrap().matches("abc\n"));
        let mut c = rules(HtmlInputKind::Email);
        c.multiple = true;
        c.pattern = Some(HtmlPattern::compile(r".+@example\.com").unwrap());
        assert!(validate(&c, "a@example.com,b@example.com").valid());
        assert!(validate(&c, "a@example.com,b@other.com").pattern_mismatch);
        c.kind = HtmlInputKind::Text;
        c.pattern = Some(HtmlPattern::compile("").unwrap());
        assert!(validate(&c, "").valid());
        assert!(validate(&c, "a").pattern_mismatch);
        c.kind = HtmlInputKind::TextArea;
        assert!(
            validate(&c, "a").valid(),
            "pattern does not apply to textarea"
        );
    }
    #[test]
    fn lengths_are_utf16_and_only_constrain_user_edits_of_nonempty_values() {
        let mut c = rules(HtmlInputKind::Text);
        c.min_length = Some(3);
        c.max_length = Some(4);
        assert!(validate(&c, "a").too_short);
        assert!(validate(&c, "😀").too_short);
        assert!(validate(&c, "😀😀").valid());
        assert!(validate(&c, "😀😀a").too_long);
        assert!(c.validate("a", HtmlInputState::default()).valid());
        assert!(validate(&c, "").valid());
        c.kind = HtmlInputKind::TextArea;
        assert_eq!(c.sanitize("a\r\nb\rc"), "a\nb\nc");
        c.kind = HtmlInputKind::Number;
        assert!(!validate(&c, "123456789").too_long);
    }
    #[test]
    fn exact_steps_use_min_then_initial_value_and_allow_any() {
        let mut c = rules(HtmlInputKind::Number);
        c.minimum = Some("0.1".into());
        c.step = Some("0.1".into());
        assert!(validate(&c, "0.3").valid());
        assert!(validate(&c, "0.31").step_mismatch);
        assert!(validate(&c, "0").range_underflow);
        c.maximum = Some("10".into());
        assert!(validate(&c, "11").range_overflow);
        c.minimum = None;
        c.maximum = None;
        c.step = None;
        assert!(validate(&c, "1.5").step_mismatch);
        c.default_value = "0.5".into();
        assert!(validate(&c, "1.5").valid());
        c.step = Some("any".into());
        assert!(validate(&c, "1e100").valid());
        assert!(validate(&c, "1e-300").valid());
        assert!(validate(&c, "NaN").bad_input);
        assert!(validate(&c, "Infinity").bad_input);
        assert!(validate(&c, "1e400").bad_input);
        assert!(validate(&c, "+1").bad_input);
        assert!(validate(&c, "1.").bad_input);
        assert!(validate(&c, "-").bad_input);
        assert!(validate(&c, ".5").valid());
    }
    #[test]
    fn temporal_constraints_keep_calendar_and_periodic_time_semantics() {
        let mut c = rules(HtmlInputKind::Date);
        assert!(validate(&c, "2024-02-29").valid());
        assert!(validate(&c, "2026-02-29").bad_input);
        c.minimum = Some("2024-01-01".into());
        c.step = Some("2".into());
        assert!(validate(&c, "2024-01-02").step_mismatch);
        c = rules(HtmlInputKind::Month);
        assert!(validate(&c, "2026-10").valid());
        assert!(validate(&c, "2026-13").bad_input);
        c = rules(HtmlInputKind::Week);
        assert!(validate(&c, "2026-W53").valid());
        assert!(validate(&c, "2025-W53").bad_input);
        c = rules(HtmlInputKind::Time);
        assert!(validate(&c, "12:30").valid());
        assert!(validate(&c, "12:30:01").step_mismatch);
        c.minimum = Some("23:00".into());
        c.maximum = Some("02:00".into());
        assert!(validate(&c, "01:00").valid());
        let outside = validate(&c, "12:00");
        assert!(outside.range_underflow && outside.range_overflow);
        c = rules(HtmlInputKind::DateTimeLocal);
        assert!(validate(&c, "2026-10-28T12:30").valid());
        assert!(validate(&c, "2026-10-28T12:30Z").bad_input);
    }
    #[test]
    fn selection_required_and_form_exclusions_are_not_text_rules() {
        let mut c = rules(HtmlInputKind::Checkbox);
        c.required = true;
        c.read_only = true;
        assert!(
            c.will_validate(),
            "HTML readonly does not apply to checkbox"
        );
        assert!(validate(&c, "on").value_missing);
        assert!(
            c.validate(
                "on",
                HtmlInputState {
                    checked: true,
                    ..Default::default()
                }
            )
            .valid()
        );
        c.kind = HtmlInputKind::Select;
        assert!(validate(&c, "placeholder").value_missing);
        assert!(
            c.validate(
                "x",
                HtmlInputState {
                    selected_count: 1,
                    ..Default::default()
                }
            )
            .valid()
        );
        c.kind = HtmlInputKind::Telephone;
        assert!(!c.will_validate());
        c.read_only = false;
        assert!(
            validate(&c, "arbitrary telephone text").valid(),
            "tel has no built-in international format"
        );
        c.custom_error = "Rejected by application".into();
        assert!(validate(&c, "x").custom_error);
        let mut form = HtmlFormValidation::<u32>::default();
        form.update(7, true, validate(&c, "x"));
        assert_eq!(form.first_invalid(), Some(&7));
        assert!(!form.can_submit(false));
        assert!(form.can_submit(true));
        c.disabled = true;
        form.update(7, c.will_validate(), validate(&c, "x"));
        assert!(form.can_submit(false));
    }
}

impl HtmlValidity {
    /// Default messages; applications may localize these without replacing validation.
    pub fn message(self, rules: &HtmlConstraints) -> String {
        if !rules.will_validate() || self.valid() {
            return String::new();
        }
        if self.custom_error {
            return rules.custom_error.clone();
        }
        if self.value_missing {
            return "Please fill out this field.".into();
        }
        if self.type_mismatch {
            return if rules.kind == HtmlInputKind::Email {
                "Please enter a valid email address."
            } else {
                "Please enter a valid absolute URL."
            }
            .into();
        }
        if self.pattern_mismatch {
            return "Please match the requested format.".into();
        }
        if self.too_short {
            return format!(
                "Please enter at least {} characters.",
                rules.min_length.unwrap_or(0)
            );
        }
        if self.too_long {
            return format!(
                "Please enter no more than {} characters.",
                rules.max_length.unwrap_or(0)
            );
        }
        if self.range_underflow {
            return format!(
                "The minimum value is {}.",
                rules.minimum.as_deref().unwrap_or("")
            );
        }
        if self.range_overflow {
            return format!(
                "The maximum value is {}.",
                rules.maximum.as_deref().unwrap_or("")
            );
        }
        if self.step_mismatch {
            return "Please enter a value matching the requested step.".into();
        }
        "Please enter a valid value.".into()
    }
}

#[cfg(test)]
mod browser_oracle {
    use super::*;
    #[test]
    fn matches_real_chrome_constraint_and_sanitization_results() {
        let cases: serde_json::Value =
            serde_json::from_str(include_str!("data/html-browser-oracle.json")).unwrap();
        for case in cases.as_array().unwrap() {
            let c = &case["config"];
            let kind = match c["type"].as_str().unwrap() {
                "email" => HtmlInputKind::Email,
                "url" => HtmlInputKind::Url,
                "text" => HtmlInputKind::Text,
                "number" => HtmlInputKind::Number,
                "date" => HtmlInputKind::Date,
                "week" => HtmlInputKind::Week,
                "time" => HtmlInputKind::Time,
                "checkbox" => HtmlInputKind::Checkbox,
                "tel" => HtmlInputKind::Telephone,
                "datetime-local" => HtmlInputKind::DateTimeLocal,
                other => panic!("unknown fixture kind {other}"),
            };
            let attr = |key: &str| c[key].as_str().map(str::to_string);
            let rules = HtmlConstraints {
                kind,
                required: c["required"].as_bool().unwrap_or(false),
                multiple: c["multiple"].as_bool().unwrap_or(false),
                disabled: c["disabled"].as_bool().unwrap_or(false),
                read_only: c["readonly"].as_bool().unwrap_or(false),
                pattern: c["pattern"]
                    .as_str()
                    .and_then(|p| HtmlPattern::compile(p).ok()),
                minimum: attr("min"),
                maximum: attr("max"),
                step: attr("step"),
                min_length: c["minlength"].as_str().and_then(|s| s.parse().ok()),
                max_length: c["maxlength"].as_str().and_then(|s| s.parse().ok()),
                ..Default::default()
            };
            let raw = c["value"].as_str().unwrap_or("on");
            let actual = rules.validate(
                case["value"].as_str().unwrap(),
                HtmlInputState {
                    checked: c["checked"].as_bool().unwrap_or(false),
                    ..Default::default()
                },
            );
            assert_eq!(
                rules.will_validate(),
                case["willValidate"].as_bool().unwrap(),
                "{c}"
            );
            assert_eq!(rules.sanitize(raw), case["value"].as_str().unwrap(), "{c}");
            for (key, value) in [
                ("valid", actual.valid()),
                ("valueMissing", actual.value_missing),
                ("typeMismatch", actual.type_mismatch),
                ("patternMismatch", actual.pattern_mismatch),
                ("tooLong", actual.too_long),
                ("tooShort", actual.too_short),
                ("rangeUnderflow", actual.range_underflow),
                ("rangeOverflow", actual.range_overflow),
                ("stepMismatch", actual.step_mismatch),
                ("badInput", actual.bad_input),
                ("customError", actual.custom_error),
            ] {
                assert_eq!(
                    value,
                    case["validity"][key].as_bool().unwrap(),
                    "{c}: {key}"
                );
            }
        }
    }
}

#[cfg(test)]
mod localized_time_tests {
    use super::*;
    #[test]
    fn twelve_hour_drafts_use_the_same_canonical_time_bounds_and_steps() {
        let mut rules = HtmlConstraints {
            kind: HtmlInputKind::Time,
            twelve_hour: true,
            minimum: Some("13:00".into()),
            maximum: Some("14:00".into()),
            ..Default::default()
        };
        assert_eq!(rules.sanitize("01:30 PM"), "13:30");
        assert!(
            rules
                .validate("01:30 PM", HtmlInputState::default())
                .valid()
        );
        assert!(
            rules
                .validate("01:30 AM", HtmlInputState::default())
                .range_underflow
        );
        assert!(
            rules
                .validate("13:30 PM", HtmlInputState::default())
                .bad_input
        );
        rules.kind = HtmlInputKind::DateTimeLocal;
        rules.minimum = Some("2026-10-28T13:00".into());
        rules.maximum = None;
        assert_eq!(rules.sanitize("2026-10-28T01:30 PM"), "2026-10-28T13:30");
    }
}

//! Exact decimal editing, with locale-specific syntax and explicit rounding.
use crate::{DraftValue, NumberInputFormat, ParseIssue, StepDirection};
use rust_decimal::{Decimal, RoundingStrategy};
use std::{fmt, marker::PhantomData, num::NonZeroU32, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecimalValue(Decimal);
impl DecimalValue {
    pub const ZERO: Self = Self(Decimal::ZERO);
    pub fn from_integer(value: i64) -> Self {
        Self(Decimal::from(value))
    }
    pub fn parse_canonical(value: &str) -> Result<Self, ParseIssue> {
        let unsigned = value.strip_prefix(['-', '+']).unwrap_or(value);
        let parts: Vec<&str> = unsigned.split('.').collect();
        if parts.len() > 2
            || parts
                .iter()
                .any(|part| part.is_empty() || !part.bytes().all(|value| value.is_ascii_digit()))
        {
            return Err(ParseIssue::InvalidSyntax);
        }
        if value
            .split_once('.')
            .is_some_and(|(_, fraction)| fraction.len() > 28)
        {
            return Err(ParseIssue::ScaleExceeded);
        }
        Decimal::from_str_exact(value)
            .map(Self)
            .map_err(|_| ParseIssue::Overflow)
    }
    /// Approximate renderer/accessibility projection only. Accepted values and
    /// lattice arithmetic retain DecimalValue; this is never a domain conversion.
    pub fn renderer_float(self) -> Result<f32, ParseIssue> {
        use rust_decimal::prelude::ToPrimitive;
        self.0
            .to_f32()
            .filter(|value| value.is_finite())
            .ok_or(ParseIssue::Overflow)
    }
    pub fn scale(self) -> u32 {
        self.0.scale()
    }
    pub fn checked_add(self, other: Self) -> Result<Self, ParseIssue> {
        exact_sum(self.0, other.0, false)
    }
    pub fn checked_sub(self, other: Self) -> Result<Self, ParseIssue> {
        exact_sum(self.0, other.0, true)
    }
    pub fn checked_mul(self, other: Self) -> Result<Self, ParseIssue> {
        let a = self.0.normalize();
        let b = other.0.normalize();
        let mut left = a.mantissa();
        let mut right = b.mantissa();
        let mut scale = (a.scale() + b.scale()) as i32;
        if left == 0 || right == 0 {
            return Ok(Self::ZERO);
        }
        // Cancel powers of ten before multiplying the 96-bit coefficients.
        while scale > 0 {
            if left % 2 == 0 && right % 5 == 0 {
                left /= 2;
                right /= 5;
                scale -= 1;
            } else if left % 5 == 0 && right % 2 == 0 {
                left /= 5;
                right /= 2;
                scale -= 1;
            } else {
                break;
            }
        }
        decimal_from_coefficient(left.checked_mul(right).ok_or(ParseIssue::Overflow)?, scale)
    }
    /// Only an exactly representable quotient succeeds. A recurring quotient
    /// requires a separate application rounding decision.
    pub fn checked_div(self, other: Self) -> Result<Self, ParseIssue> {
        let a = self.0.normalize();
        let b = other.0.normalize();
        let mut numerator = a.mantissa();
        let mut denominator = b.mantissa();
        if denominator == 0 {
            return Err(ParseIssue::DivisionByZero);
        }
        if numerator == 0 {
            return Ok(Self::ZERO);
        }
        if denominator < 0 {
            numerator = -numerator;
            denominator = -denominator;
        }
        let divisor = gcd(numerator.abs(), denominator);
        numerator /= divisor;
        denominator /= divisor;
        let mut twos = 0;
        let mut fives = 0;
        while denominator % 2 == 0 {
            denominator /= 2;
            twos += 1;
        }
        while denominator % 5 == 0 {
            denominator /= 5;
            fives += 1;
        }
        if denominator != 1 {
            return Err(ParseIssue::ScaleExceeded);
        }
        let decimal_places = twos.max(fives);
        let scale = decimal_places + a.scale() as i32 - b.scale() as i32;
        if scale > 28 {
            return Err(ParseIssue::ScaleExceeded);
        }
        numerator = numerator
            .checked_mul(
                2_i128
                    .checked_pow((decimal_places - twos) as u32)
                    .ok_or(ParseIssue::Overflow)?,
            )
            .ok_or(ParseIssue::Overflow)?;
        numerator = numerator
            .checked_mul(
                5_i128
                    .checked_pow((decimal_places - fives) as u32)
                    .ok_or(ParseIssue::Overflow)?,
            )
            .ok_or(ParseIssue::Overflow)?;
        decimal_from_coefficient(numerator, scale)
    }
    pub fn quantize(
        self,
        scale: DecimalScale,
        rounding: RoundingPolicy,
    ) -> Result<Self, ParseIssue> {
        let strategy = match rounding {
            RoundingPolicy::Reject => {
                if self.0.scale() > scale.0 && self.0.round_dp(scale.0) != self.0 {
                    return Err(ParseIssue::ScaleExceeded);
                }
                RoundingStrategy::MidpointNearestEven
            }
            RoundingPolicy::NearestEven => RoundingStrategy::MidpointNearestEven,
            RoundingPolicy::HalfAwayFromZero => RoundingStrategy::MidpointAwayFromZero,
            RoundingPolicy::TowardZero => RoundingStrategy::ToZero,
            RoundingPolicy::Floor => RoundingStrategy::ToNegativeInfinity,
            RoundingPolicy::Ceiling => RoundingStrategy::ToPositiveInfinity,
        };
        let mut value = self.0.round_dp_with_strategy(scale.0, strategy);
        value.rescale(scale.0);
        if value.scale() != scale.0 {
            return Err(ParseIssue::Overflow);
        }
        Ok(Self(value))
    }
}
fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}
fn decimal_from_coefficient(
    mut coefficient: i128,
    mut scale: i32,
) -> Result<DecimalValue, ParseIssue> {
    const MAX: i128 = (1_i128 << 96) - 1;
    if coefficient == 0 {
        return Ok(DecimalValue::ZERO);
    }
    while scale > 0
        && (scale > 28 || coefficient.unsigned_abs() > MAX as u128)
        && coefficient % 10 == 0
    {
        coefficient /= 10;
        scale -= 1;
    }
    if scale > 28 {
        return Err(ParseIssue::ScaleExceeded);
    }
    if scale < 0 {
        coefficient = coefficient
            .checked_mul(
                10_i128
                    .checked_pow(scale.unsigned_abs())
                    .ok_or(ParseIssue::Overflow)?,
            )
            .ok_or(ParseIssue::Overflow)?;
        scale = 0;
    }
    Decimal::try_from_i128_with_scale(coefficient, scale as u32)
        .map(DecimalValue)
        .map_err(|_| ParseIssue::Overflow)
}
fn exact_sum(a: Decimal, b: Decimal, subtract: bool) -> Result<DecimalValue, ParseIssue> {
    let a = a.normalize();
    let b = b.normalize();
    let scale = a.scale().max(b.scale());
    let left = a
        .mantissa()
        .checked_mul(10_i128.pow(scale - a.scale()))
        .ok_or(ParseIssue::Overflow)?;
    let right = b
        .mantissa()
        .checked_mul(10_i128.pow(scale - b.scale()))
        .ok_or(ParseIssue::Overflow)?;
    let coefficient = if subtract {
        left.checked_sub(right)
    } else {
        left.checked_add(right)
    }
    .ok_or(ParseIssue::Overflow)?;
    decimal_from_coefficient(coefficient, scale as i32)
}
impl fmt::Display for DecimalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl FromStr for DecimalValue {
    type Err = ParseIssue;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse_canonical(value)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecimalScale(u32);
impl DecimalScale {
    pub fn new(value: u32) -> Result<Self, ParseIssue> {
        if value > 28 {
            Err(ParseIssue::ScaleExceeded)
        } else {
            Ok(Self(value))
        }
    }
    pub fn get(self) -> u32 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundingPolicy {
    Reject,
    NearestEven,
    HalfAwayFromZero,
    TowardZero,
    Floor,
    Ceiling,
}
/// Digit group sizes for the integer part. A closed set the code decides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupSizes {
    /// 1,234,567: every group has three digits.
    Thousands,
    /// 12,34,567: the last group has three digits, the others two.
    Indian,
}
impl GroupSizes {
    fn secondary(self) -> usize {
        match self {
            Self::Thousands => 3,
            Self::Indian => 2,
        }
    }
}
/// Where the currency symbol sits relative to the amount.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolPlacement {
    /// `$1,234.50`, `-$1.00`.
    Prefix,
    /// `R$ 1.234,50` with a no-break space.
    PrefixSpaced,
    /// `1 234,50 $` with a no-break space.
    Suffix,
}
/// Grouping of the integer part: what is written, and what is also accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grouping {
    separator: char,
    alternate: Option<char>,
    whitespace_equivalent: bool,
    sizes: GroupSizes,
}
impl Grouping {
    /// `separator` is written by the formatter. Any other accepted character
    /// is declared explicitly; there is no implicit fallback.
    pub fn new(separator: char, sizes: GroupSizes) -> Self {
        Self {
            separator,
            alternate: None,
            whitespace_equivalent: false,
            sizes,
        }
    }
    /// Also accept one more character, for example ASCII `'` for `\u{2019}`.
    pub fn accepting(self, alternate: char) -> Self {
        Self {
            alternate: Some(alternate),
            ..self
        }
    }
    /// Also accept space, no-break space and narrow no-break space.
    pub fn accepting_whitespace(self) -> Self {
        Self {
            whitespace_equivalent: true,
            ..self
        }
    }
    pub fn separator(self) -> char {
        self.separator
    }
    pub fn sizes(self) -> GroupSizes {
        self.sizes
    }
    fn accepts(self, c: char) -> bool {
        c == self.separator
            || self.alternate == Some(c)
            || (self.whitespace_equivalent && matches!(c, ' ' | '\u{a0}' | '\u{202f}'))
    }
}
/// The number grammar of a writing convention: one value shared by the
/// parser, the formatter and `Money::format`. The set of conventions is open,
/// so this is a value type with constructors, not an enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberSyntax {
    decimal: char,
    grouping: Option<Grouping>,
    symbol: SymbolPlacement,
}
impl NumberSyntax {
    /// Fails with `InvalidSyntax` if a separator is a digit, a sign, an
    /// exponent marker or whitespace, or if decimal and grouping collide.
    pub fn new(
        decimal: char,
        grouping: Option<Grouping>,
        symbol: SymbolPlacement,
    ) -> Result<Self, ParseIssue> {
        let reserved = |c: char| c.is_alphanumeric() || c.is_whitespace() || matches!(c, '+' | '-');
        let separators_valid = !reserved(decimal)
            && grouping.is_none_or(|grouping| {
                !reserved(grouping.separator)
                    && grouping.alternate.is_none_or(|c| !reserved(c))
                    && !grouping.accepts(decimal)
            });
        if separators_valid {
            Ok(Self {
                decimal,
                grouping,
                symbol,
            })
        } else {
            Err(ParseIssue::InvalidSyntax)
        }
    }
    fn known(decimal: char, grouping: Grouping, symbol: SymbolPlacement) -> Self {
        Self {
            decimal,
            grouping: Some(grouping),
            symbol,
        }
    }
    /// 1 234,50 (no-break space), `1 234,50 $`.
    pub fn fr_ca() -> Self {
        Self::known(
            ',',
            Grouping::new('\u{a0}', GroupSizes::Thousands).accepting_whitespace(),
            SymbolPlacement::Suffix,
        )
    }
    /// 1 234,50 (narrow no-break space), `1 234,50 €`.
    pub fn fr_fr() -> Self {
        Self::known(
            ',',
            Grouping::new('\u{202f}', GroupSizes::Thousands).accepting_whitespace(),
            SymbolPlacement::Suffix,
        )
    }
    /// 1,234.50, `$1,234.50`.
    pub fn en_ca() -> Self {
        Self::known(
            '.',
            Grouping::new(',', GroupSizes::Thousands),
            SymbolPlacement::Prefix,
        )
    }
    /// 1,234.50, `$1,234.50`.
    pub fn en_us() -> Self {
        Self::en_ca()
    }
    /// 12,34,567.50, `\u{20b9}12,34,567.50`.
    pub fn en_in() -> Self {
        Self::known(
            '.',
            Grouping::new(',', GroupSizes::Indian),
            SymbolPlacement::Prefix,
        )
    }
    /// 1.234,50, `1.234,50 €`.
    pub fn de_de() -> Self {
        Self::known(
            ',',
            Grouping::new('.', GroupSizes::Thousands),
            SymbolPlacement::Suffix,
        )
    }
    /// 1\u{2019}234.50, `CHF 1\u{2019}234.50`; ASCII `'` is also accepted.
    pub fn de_ch() -> Self {
        Self::known(
            '.',
            Grouping::new('\u{2019}', GroupSizes::Thousands).accepting('\''),
            SymbolPlacement::PrefixSpaced,
        )
    }
    /// 1.234,50, `R$ 1.234,50`.
    pub fn pt_br() -> Self {
        Self::known(
            ',',
            Grouping::new('.', GroupSizes::Thousands),
            SymbolPlacement::PrefixSpaced,
        )
    }
    pub fn decimal_separator(self) -> char {
        self.decimal
    }
    pub fn grouping(self) -> Option<Grouping> {
        self.grouping
    }
    pub fn symbol_placement(self) -> SymbolPlacement {
        self.symbol
    }
}
impl NumberInputFormat {
    /// The grammar the HTML number engine applies to a draft in this format.
    /// The control's format and the typed codec share this one definition.
    pub fn syntax(self) -> NumberSyntax {
        match self {
            Self::Canonical => NumberSyntax::en_ca(),
            Self::DecimalComma => NumberSyntax::fr_ca(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignPolicy {
    Signed,
    NonNegative,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumericBounds<T> {
    minimum: T,
    maximum: T,
}
impl<T: Ord + Copy> NumericBounds<T> {
    pub fn new(minimum: T, maximum: T) -> Result<Self, ParseIssue> {
        if minimum > maximum {
            Err(ParseIssue::InvalidRange)
        } else {
            Ok(Self { minimum, maximum })
        }
    }
    pub fn minimum(self) -> T {
        self.minimum
    }
    pub fn maximum(self) -> T {
        self.maximum
    }
    pub fn validate(self, value: T) -> Result<T, ParseIssue> {
        if value < self.minimum {
            Err(ParseIssue::BelowMinimum)
        } else if value > self.maximum {
            Err(ParseIssue::AboveMaximum)
        } else {
            Ok(value)
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecimalRules {
    pub syntax: NumberSyntax,
    pub scale: DecimalScale,
    pub sign: SignPolicy,
    pub bounds: Option<NumericBounds<DecimalValue>>,
}
impl DecimalRules {
    pub fn parse(self, text: &str) -> DraftValue<DecimalValue> {
        let canonical = match canonicalize(text, self.syntax, true, self.sign) {
            Ok(value) => value,
            Err(value) => return value,
        };
        if canonical
            .split_once('.')
            .is_some_and(|(_, fraction)| fraction.len() > self.scale.0 as usize)
        {
            return DraftValue::Invalid(ParseIssue::ScaleExceeded);
        }
        match DecimalValue::parse_canonical(&canonical).and_then(|value| {
            self.bounds
                .map_or(Ok(value), |bounds| bounds.validate(value))
        }) {
            Ok(value) => DraftValue::Parsed(value),
            Err(error) => DraftValue::Invalid(error),
        }
    }
}
// Decimal and integer domains expand exponents only after structural normalization.
fn canonicalize<T>(
    text: &str,
    syntax: NumberSyntax,
    decimal: bool,
    sign: SignPolicy,
) -> Result<String, DraftValue<T>> {
    let normalized = canonical_number_syntax(text, syntax, decimal, sign)?;
    if let Some((mantissa, exponent)) = normalized.split_once(['e', 'E']) {
        let exponent = exponent
            .parse::<i32>()
            .map_err(|_| DraftValue::Invalid(ParseIssue::Overflow))?;
        expand_exponent(mantissa, exponent).map_err(DraftValue::Invalid)
    } else {
        Ok(normalized)
    }
}
// Shared syntax normalization preserves the exponent representation of native values.
// It produces text, not a parsed domain value.
pub(crate) fn canonical_number_syntax<T>(
    text: &str,
    syntax: NumberSyntax,
    decimal: bool,
    sign: SignPolicy,
) -> Result<String, DraftValue<T>> {
    let text = text.trim();
    if text.is_empty() {
        return Err(DraftValue::Empty);
    }
    if let Some((mantissa, exponent)) = text.split_once(['e', 'E']) {
        let marker = if text.contains('e') { 'e' } else { 'E' };
        let mantissa = canonical_number_syntax::<T>(mantissa, syntax, true, sign)?;
        let digits = exponent.strip_prefix(['-', '+']).unwrap_or(exponent);
        if digits.is_empty() {
            return Err(DraftValue::Incomplete);
        }
        if !digits.bytes().all(|c| c.is_ascii_digit()) {
            return Err(DraftValue::Invalid(ParseIssue::InvalidSyntax));
        }
        return Ok(format!("{mantissa}{marker}{exponent}"));
    }
    let negative = text.starts_with('-');
    if negative && sign == SignPolicy::NonNegative {
        return Err(DraftValue::Invalid(ParseIssue::BelowMinimum));
    }
    let unsigned = text.strip_prefix(['-', '+']).unwrap_or(text);
    if unsigned.is_empty() {
        return Err(DraftValue::Incomplete);
    }
    let mut parts = unsigned.split(syntax.decimal_separator());
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next();
    if parts.next().is_some() || (!decimal && fraction.is_some()) {
        return Err(DraftValue::Invalid(ParseIssue::InvalidSyntax));
    }
    if let Some(value) = fraction
        && !value.chars().all(|c| c.is_ascii_digit())
    {
        return Err(DraftValue::Invalid(ParseIssue::InvalidSyntax));
    }
    let groups: Vec<&str> = match syntax.grouping() {
        Some(grouping) => integer.split(|c| grouping.accepts(c)).collect(),
        None => vec![integer],
    };
    if let Some(grouping) = syntax.grouping()
        && groups.len() > 1
    {
        let secondary = grouping.sizes().secondary();
        let (first, rest) = (groups[0], &groups[1..]);
        let (middle, last) = rest.split_at(rest.len() - 1);
        if first.is_empty()
            || first.len() > secondary
            || last[0].len() != 3
            || middle.iter().any(|group| group.len() != secondary)
        {
            return Err(DraftValue::Invalid(ParseIssue::InvalidGrouping));
        }
    }
    if groups
        .iter()
        .any(|group| !group.chars().all(|c| c.is_ascii_digit()))
    {
        return Err(DraftValue::Invalid(ParseIssue::InvalidSyntax));
    }
    if fraction == Some("") {
        return Err(DraftValue::Incomplete);
    }
    let digits = groups.concat();
    if digits.is_empty() && fraction.is_none() {
        return Err(DraftValue::Incomplete);
    }
    let mut result = if negative {
        "-".to_string()
    } else {
        String::new()
    };
    result.push_str(if digits.is_empty() { "0" } else { &digits });
    if let Some(fraction) = fraction {
        result.push('.');
        result.push_str(fraction);
    }
    Ok(result)
}
fn expand_exponent(mantissa: &str, exponent: i32) -> Result<String, ParseIssue> {
    let negative = mantissa.starts_with('-');
    let unsigned = mantissa.trim_start_matches('-');
    let (integer, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let digits = format!("{integer}{fraction}");
    let pivot =
        i64::try_from(integer.len()).map_err(|_| ParseIssue::Overflow)? + i64::from(exponent);
    let length = i64::try_from(digits.len()).map_err(|_| ParseIssue::Overflow)?;
    if pivot.max(length) - pivot.min(0) > 4096 {
        if digits.bytes().all(|c| c == b'0') {
            return Ok("0".into());
        }
        return Err(ParseIssue::Overflow);
    }
    let value = if pivot <= 0 {
        format!("0.{}{digits}", "0".repeat((-pivot) as usize))
    } else if pivot >= length {
        format!("{digits}{}", "0".repeat((pivot - length) as usize))
    } else {
        format!(
            "{}.{}",
            &digits[..pivot as usize],
            &digits[pivot as usize..]
        )
    };
    Ok(if negative { format!("-{value}") } else { value })
}
pub fn parse_integer(
    text: &str,
    syntax: NumberSyntax,
    bounds: Option<NumericBounds<i64>>,
) -> DraftValue<i64> {
    let canonical = match canonicalize(text, syntax, true, SignPolicy::Signed) {
        Ok(value) => value,
        Err(value) => return value,
    };
    let (integer, fraction) = canonical.split_once('.').unwrap_or((&canonical, ""));
    if fraction.bytes().any(|c| c != b'0') {
        return DraftValue::Invalid(ParseIssue::InvalidSyntax);
    }
    match integer
        .parse::<i64>()
        .map_err(|_| ParseIssue::Overflow)
        .and_then(|value| bounds.map_or(Ok(value), |bounds| bounds.validate(value)))
    {
        Ok(value) => DraftValue::Parsed(value),
        Err(error) => DraftValue::Invalid(error),
    }
}
pub fn format_decimal(
    value: DecimalValue,
    syntax: NumberSyntax,
    scale: DecimalScale,
    rounding: RoundingPolicy,
) -> Result<String, ParseIssue> {
    let canonical = value.quantize(scale, rounding)?.to_string();
    let (integer, fraction) = canonical
        .split_once('.')
        .map_or((canonical.as_str(), None), |(integer, fraction)| {
            (integer, Some(fraction))
        });
    let digits = integer.strip_prefix('-').unwrap_or(integer);
    let mut formatted = String::new();
    if integer.starts_with('-') {
        formatted.push('-');
    }
    for (index, c) in digits.chars().enumerate() {
        if let Some(grouping) = syntax.grouping() {
            let remaining = digits.len() - index;
            let boundary = match grouping.sizes() {
                GroupSizes::Thousands => remaining.is_multiple_of(3),
                GroupSizes::Indian => remaining >= 3 && (remaining - 3).is_multiple_of(2),
            };
            if index > 0 && boundary {
                formatted.push(grouping.separator());
            }
        }
        formatted.push(c);
    }
    if let Some(fraction) = fraction {
        formatted.push(syntax.decimal_separator());
        formatted.push_str(fraction);
    }
    Ok(formatted)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecimalStep(DecimalValue);
impl DecimalStep {
    pub fn new(value: DecimalValue) -> Result<Self, ParseIssue> {
        if value <= DecimalValue::ZERO {
            Err(ParseIssue::InvalidRange)
        } else {
            Ok(Self(value))
        }
    }
    pub fn apply(
        self,
        value: DecimalValue,
        direction: StepDirection,
        bounds: NumericBounds<DecimalValue>,
    ) -> Result<DecimalValue, ParseIssue> {
        bounds.validate(match direction {
            StepDirection::Increment => value.checked_add(self.0)?,
            StepDirection::Decrement => value.checked_sub(self.0)?,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Percent(pub DecimalValue);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio(pub DecimalValue);
impl Percent {
    pub fn to_ratio(self) -> Result<Ratio, ParseIssue> {
        self.0
            .checked_div(DecimalValue::from_integer(100))
            .map(Ratio)
    }
}
impl Ratio {
    pub fn to_percent(self) -> Result<Percent, ParseIssue> {
        self.0
            .checked_mul(DecimalValue::from_integer(100))
            .map(Percent)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CurrencyCode(String);
impl CurrencyCode {
    pub fn new(value: impl Into<String>) -> Result<Self, ParseIssue> {
        let value = value.into();
        if value.len() == 3 && value.bytes().all(|c| c.is_ascii_uppercase()) {
            Ok(Self(value))
        } else {
            Err(ParseIssue::InvalidCurrency)
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrencyMetadata {
    pub code: CurrencyCode,
    pub symbol: String,
    pub scale: DecimalScale,
}
/// ISO 4217 minor-unit scale and a customary symbol for common currencies.
/// The scale is the standard's; a symbol is a display choice and may differ
/// per locale, so build a `CurrencyMetadata` by hand when it must.
const ISO_CURRENCIES: &[(&str, &str, u32)] = &[
    ("AUD", "$", 2),
    ("BHD", "BD", 3),
    ("BRL", "R$", 2),
    ("CAD", "$", 2),
    ("CHF", "CHF", 2),
    ("CLF", "UF", 4),
    ("CLP", "$", 0),
    ("CNY", "¥", 2),
    ("CZK", "Kč", 2),
    ("DKK", "kr", 2),
    ("EUR", "€", 2),
    ("GBP", "£", 2),
    ("HKD", "$", 2),
    ("INR", "₹", 2),
    ("ISK", "kr", 0),
    ("JOD", "JD", 3),
    ("JPY", "¥", 0),
    ("KRW", "₩", 0),
    ("KWD", "KD", 3),
    ("MXN", "$", 2),
    ("NOK", "kr", 2),
    ("NZD", "$", 2),
    ("OMR", "OMR", 3),
    ("PLN", "zł", 2),
    ("SEK", "kr", 2),
    ("SGD", "$", 2),
    ("TND", "DT", 3),
    ("USD", "$", 2),
    ("VND", "₫", 0),
    ("ZAR", "R", 2),
];
impl CurrencyMetadata {
    /// Metadata for any currency, including one outside the built-in table.
    pub fn new(code: CurrencyCode, symbol: impl Into<String>, scale: DecimalScale) -> Self {
        Self {
            code,
            symbol: symbol.into(),
            scale,
        }
    }
    /// The ISO 4217 scale and customary symbol of a common currency, or `None`
    /// when the table does not list the code; callers then use [`Self::new`].
    pub fn iso(code: &CurrencyCode) -> Option<Self> {
        ISO_CURRENCIES
            .iter()
            .find(|(iso, _, _)| *iso == code.as_str())
            .map(|&(_, symbol, scale)| Self {
                code: code.clone(),
                symbol: symbol.into(),
                scale: DecimalScale(scale),
            })
    }
    pub fn cad() -> Self {
        Self {
            code: CurrencyCode("CAD".into()),
            symbol: "$".into(),
            scale: DecimalScale(2),
        }
    }
    pub fn eur() -> Self {
        Self {
            code: CurrencyCode("EUR".into()),
            symbol: "€".into(),
            scale: DecimalScale(2),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Money {
    pub amount: DecimalValue,
    pub currency: CurrencyCode,
}
impl Money {
    pub fn format(
        &self,
        metadata: &CurrencyMetadata,
        syntax: NumberSyntax,
        rounding: RoundingPolicy,
    ) -> Result<String, ParseIssue> {
        if self.currency != metadata.code {
            return Err(ParseIssue::InvalidCurrency);
        }
        let amount = format_decimal(self.amount, syntax, metadata.scale, rounding)?;
        let symbol = &metadata.symbol;
        let (sign, unsigned) = match amount.strip_prefix('-') {
            Some(unsigned) => ("-", unsigned),
            None => ("", amount.as_str()),
        };
        Ok(match syntax.symbol_placement() {
            SymbolPlacement::Suffix => format!("{amount}\u{a0}{symbol}"),
            SymbolPlacement::Prefix => format!("{sign}{symbol}{unsigned}"),
            SymbolPlacement::PrefixSpaced => format!("{sign}{symbol}\u{a0}{unsigned}"),
        })
    }
    pub fn checked_add(&self, other: &Self) -> Result<Self, ParseIssue> {
        if self.currency != other.currency {
            return Err(ParseIssue::InvalidCurrency);
        }
        Ok(Self {
            amount: self.amount.checked_add(other.amount)?,
            currency: self.currency.clone(),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quantity<U> {
    pub value: DecimalValue,
    pub unit: U,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurationUnit {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
}
impl DurationUnit {
    fn milliseconds(self) -> i64 {
        match self {
            Self::Milliseconds => 1,
            Self::Seconds => 1_000,
            Self::Minutes => 60_000,
            Self::Hours => 3_600_000,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DurationValue {
    milliseconds: u64,
}
impl DurationValue {
    pub fn from_milliseconds(milliseconds: u64) -> Self {
        Self { milliseconds }
    }
    pub fn milliseconds(self) -> u64 {
        self.milliseconds
    }
    pub fn from_quantity(value: DecimalValue, unit: DurationUnit) -> Result<Self, ParseIssue> {
        use rust_decimal::prelude::ToPrimitive;
        let result = value.checked_mul(DecimalValue::from_integer(unit.milliseconds()))?;
        if result.0.fract() != Decimal::ZERO {
            return Err(ParseIssue::ScaleExceeded);
        }
        result
            .0
            .to_u64()
            .map(Self::from_milliseconds)
            .ok_or(ParseIssue::Overflow)
    }
    /// Convert to the requested quantity exactly. Recurring decimal results,
    /// such as one millisecond expressed in minutes, return ScaleExceeded.
    pub fn quantity(self, unit: DurationUnit) -> Result<DecimalValue, ParseIssue> {
        DecimalValue(Decimal::from(self.milliseconds))
            .checked_div(DecimalValue::from_integer(unit.milliseconds()))
    }
}
/// A typed unit marker is useful when all values in a domain have one fixed unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedQuantity<U> {
    value: DecimalValue,
    unit: PhantomData<U>,
}
impl<U> FixedQuantity<U> {
    pub fn new(value: DecimalValue) -> Self {
        Self {
            value,
            unit: PhantomData,
        }
    }
    pub fn value(&self) -> DecimalValue {
        self.value
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn d(value: &str) -> DecimalValue {
        value.parse().unwrap()
    }
    fn rules(syntax: NumberSyntax) -> DecimalRules {
        DecimalRules {
            syntax,
            scale: DecimalScale::new(2).unwrap(),
            sign: SignPolicy::Signed,
            bounds: None,
        }
    }
    #[test]
    fn exact_and_locale_round_trip() {
        assert_eq!(d("0.10").checked_add(d("0.20")).unwrap(), d("0.30"));
        for locale in [
            NumberSyntax::fr_ca(),
            NumberSyntax::fr_fr(),
            NumberSyntax::en_ca(),
            NumberSyntax::en_us(),
        ] {
            let text = format_decimal(
                d("1234.50"),
                locale,
                DecimalScale(2),
                RoundingPolicy::Reject,
            )
            .unwrap();
            assert_eq!(rules(locale).parse(&text), DraftValue::Parsed(d("1234.50")));
        }
    }
    #[test]
    fn incomplete_invalid_and_overflow() {
        assert_eq!(
            rules(NumberSyntax::fr_ca()).parse("1,"),
            DraftValue::Incomplete
        );
        assert_eq!(
            rules(NumberSyntax::fr_ca()).parse("-"),
            DraftValue::Incomplete
        );
        assert_eq!(rules(NumberSyntax::fr_ca()).parse(""), DraftValue::Empty);
        assert_eq!(
            rules(NumberSyntax::en_ca()).parse("1,23.40"),
            DraftValue::Invalid(ParseIssue::InvalidGrouping)
        );
        assert_eq!(
            parse_integer("9223372036854775808", NumberSyntax::en_us(), None),
            DraftValue::Invalid(ParseIssue::Overflow)
        );
        assert_eq!(
            rules(NumberSyntax::fr_fr()).parse("1\u{202f}234,50"),
            DraftValue::Parsed(d("1234.5"))
        );
        assert_eq!(
            d("79228162514264337593543950335").checked_add(d("1")),
            Err(ParseIssue::Overflow)
        );
    }
    #[test]
    fn arithmetic_never_silently_drops_precision() {
        assert_eq!(
            d("-1844674407370955161.6").checked_mul(d("9223372036854775808")),
            Err(ParseIssue::Overflow)
        );
        assert_eq!(
            d("79228162514264337593543950335").checked_add(d("0.1")),
            Err(ParseIssue::Overflow)
        );
        assert_eq!(d("1").checked_div(d("3")), Err(ParseIssue::ScaleExceeded));
        assert_eq!(d("1").checked_div(d("0")), Err(ParseIssue::DivisionByZero));
        assert_eq!(d("1").checked_div(d("8")).unwrap(), d("0.125"));
        assert_eq!(d("1.2").checked_mul(d("0.25")).unwrap(), d("0.3"));
        assert_eq!(
            d("1.0000000000000000000000000000")
                .checked_add(d("79228162514264337593543950334"))
                .unwrap(),
            d("79228162514264337593543950335")
        );
        assert_eq!(
            d("0.0000000000000000000000000001").checked_mul(d("0.1")),
            Err(ParseIssue::ScaleExceeded)
        );
    }
    #[test]
    fn precision_and_currency_rules() {
        let negative = Money {
            amount: DecimalValue::parse_canonical("-1.20").unwrap(),
            currency: CurrencyMetadata::cad().code,
        };
        assert_eq!(
            negative
                .format(
                    &CurrencyMetadata::cad(),
                    NumberSyntax::en_ca(),
                    RoundingPolicy::Reject
                )
                .unwrap(),
            "-$1.20"
        );
        assert_eq!(
            d("1.225")
                .quantize(DecimalScale(2), RoundingPolicy::NearestEven)
                .unwrap()
                .to_string(),
            "1.22"
        );
        assert_eq!(
            d("1.225").quantize(DecimalScale(2), RoundingPolicy::Reject),
            Err(ParseIssue::ScaleExceeded)
        );
        assert_eq!(Percent(d("12.5")).to_ratio().unwrap(), Ratio(d("0.125")));
        let money = Money {
            amount: d("1234.50"),
            currency: CurrencyMetadata::cad().code,
        };
        assert_eq!(
            money
                .format(
                    &CurrencyMetadata::cad(),
                    NumberSyntax::fr_ca(),
                    RoundingPolicy::Reject
                )
                .unwrap(),
            "1\u{a0}234,50\u{a0}$"
        );
        for scale in [0, 2, 3] {
            let metadata = CurrencyMetadata {
                code: CurrencyCode::new("XYZ").unwrap(),
                symbol: "¤".into(),
                scale: DecimalScale(scale),
            };
            let amount = Money {
                amount: d("12"),
                currency: metadata.code.clone(),
            };
            assert!(
                amount
                    .format(&metadata, NumberSyntax::en_us(), RoundingPolicy::Reject)
                    .is_ok()
            );
        }
    }
}

#[cfg(test)]
mod html_numeric_drafts {
    use super::*;
    #[test]
    fn scientific_and_integral_decimal_drafts_keep_exact_domain_values() {
        assert_eq!(
            parse_integer("1e2", NumberSyntax::en_ca(), None),
            DraftValue::Parsed(100)
        );
        assert_eq!(
            parse_integer("12.0", NumberSyntax::en_ca(), None),
            DraftValue::Parsed(12)
        );
        assert_eq!(
            parse_integer("1,5e2", NumberSyntax::fr_ca(), None),
            DraftValue::Parsed(150)
        );
        assert_eq!(
            parse_integer("1e", NumberSyntax::en_ca(), None),
            DraftValue::Incomplete
        );
        assert_eq!(
            parse_integer("1e-1", NumberSyntax::en_ca(), None),
            DraftValue::Invalid(ParseIssue::InvalidSyntax)
        );
        let rules = DecimalRules {
            syntax: NumberSyntax::en_ca(),
            scale: DecimalScale::new(2).unwrap(),
            sign: SignPolicy::Signed,
            bounds: None,
        };
        assert_eq!(
            rules.parse("0.1e-1"),
            DraftValue::Parsed(DecimalValue::parse_canonical("0.01").unwrap())
        );
        assert_eq!(
            rules.parse("1.001e0"),
            DraftValue::Invalid(ParseIssue::ScaleExceeded)
        );
        assert_eq!(
            rules.parse("1e1000000"),
            DraftValue::Invalid(ParseIssue::Overflow)
        );
        assert_eq!(
            rules.parse("NaN"),
            DraftValue::Invalid(ParseIssue::InvalidSyntax)
        );
    }
}

/// An exact decimal lattice used by both slider thumbs. Floating point appears
/// only at the renderer boundary; all accepted values remain decimal multiples.
#[derive(Clone, Copy, Debug)]
pub struct DecimalSlider {
    minimum: DecimalValue,
    maximum: DecimalValue,
    step: DecimalValue,
    page_steps: NonZeroU32,
    distance_steps: u32,
}
/// How many lattice steps Page Up and Page Down move when a host sets none.
const DEFAULT_PAGE_STEPS: NonZeroU32 = NonZeroU32::new(10).unwrap();
impl DecimalSlider {
    pub fn new(
        minimum: DecimalValue,
        maximum: DecimalValue,
        step: DecimalValue,
    ) -> Result<Self, ParseIssue> {
        if minimum >= maximum || step <= DecimalValue::ZERO {
            return Err(ParseIssue::InvalidRange);
        }
        maximum
            .checked_sub(minimum)?
            .0
            .checked_div(step.0)
            .ok_or(ParseIssue::Overflow)?;
        Ok(Self {
            minimum,
            maximum,
            step,
            page_steps: DEFAULT_PAGE_STEPS,
            distance_steps: 0,
        })
    }
    /// Sets how many lattice steps Page Up and Page Down move.
    pub fn with_page_steps(self, page_steps: NonZeroU32) -> Self {
        Self { page_steps, ..self }
    }
    /// Sets the minimum distance, in lattice steps, between the two thumbs of a
    /// range. The distance must fit in the span of the slider.
    pub fn with_distance_steps(self, distance_steps: u32) -> Result<Self, ParseIssue> {
        let slider = Self {
            distance_steps,
            ..self
        };
        if slider.distance()? > self.maximum.checked_sub(self.minimum)? {
            return Err(ParseIssue::InvalidRange);
        }
        Ok(slider)
    }
    pub fn page_steps(self) -> NonZeroU32 {
        self.page_steps
    }
    /// The smallest allowed gap between the lower and the upper thumb.
    pub fn distance(self) -> Result<DecimalValue, ParseIssue> {
        DecimalValue(Decimal::from(self.distance_steps)).checked_mul(self.step)
    }
    pub fn minimum(self) -> DecimalValue {
        self.minimum
    }
    pub fn maximum(self) -> DecimalValue {
        self.maximum
    }
    pub fn step(self) -> DecimalValue {
        self.step
    }
    pub fn validate(self, value: DecimalValue) -> Result<DecimalValue, ParseIssue> {
        if value < self.minimum {
            return Err(ParseIssue::BelowMinimum);
        }
        if value > self.maximum {
            return Err(ParseIssue::AboveMaximum);
        }
        let offset = value.checked_sub(self.minimum)?;
        if offset
            .0
            .checked_rem(self.step.0)
            .ok_or(ParseIssue::Overflow)?
            != Decimal::ZERO
        {
            return Err(ParseIssue::InvalidRange);
        }
        Ok(value)
    }
    pub fn position(self, value: DecimalValue) -> Result<f32, ParseIssue> {
        use rust_decimal::prelude::ToPrimitive;
        let value = value.clamp(self.minimum, self.maximum);
        value
            .checked_sub(self.minimum)?
            .0
            .checked_div(self.maximum.checked_sub(self.minimum)?.0)
            .and_then(|value| value.to_f32())
            .ok_or(ParseIssue::Overflow)
    }
    pub fn at_position(self, position: f32) -> Result<DecimalValue, ParseIssue> {
        if !position.is_finite() {
            return Err(ParseIssue::InvalidSyntax);
        }
        let fraction = Decimal::from_str(&position.clamp(0.0, 1.0).to_string())
            .map_err(|_| ParseIssue::InvalidSyntax)?;
        let span = self.maximum.checked_sub(self.minimum)?.0;
        let offset = span.checked_mul(fraction).ok_or(ParseIssue::Overflow)?;
        self.at_offset(offset)
    }
    /// Clamp and round an exact domain request on the same lattice as pointer
    /// input, without a domain -> float position -> domain round trip.
    pub fn at_value(self, value: DecimalValue) -> Result<DecimalValue, ParseIssue> {
        self.at_offset(
            value
                .clamp(self.minimum, self.maximum)
                .checked_sub(self.minimum)?
                .0,
        )
    }
    fn at_offset(self, offset: Decimal) -> Result<DecimalValue, ParseIssue> {
        let span = self.maximum.checked_sub(self.minimum)?.0;
        let projected = offset
            .checked_div(self.step.0)
            .ok_or(ParseIssue::Overflow)?;
        let maximum_steps = span
            .checked_div(self.step.0)
            .ok_or(ParseIssue::Overflow)?
            .floor();
        let steps = projected.round().min(maximum_steps);
        self.minimum
            .checked_add(DecimalValue(steps).checked_mul(self.step)?)
    }
    pub fn navigate(
        self,
        value: DecimalValue,
        direction: crate::NavigationStep,
    ) -> Result<DecimalValue, ParseIssue> {
        let page = i32::try_from(self.page_steps.get()).unwrap_or(i32::MAX);
        let amount: i32 = match direction {
            crate::NavigationStep::First => return Ok(self.minimum),
            crate::NavigationStep::Last => return self.at_position(1.0),
            crate::NavigationStep::Next => 1,
            crate::NavigationStep::Previous => -1,
            crate::NavigationStep::PageNext => page,
            crate::NavigationStep::PagePrevious => -page,
        };
        if value < self.minimum {
            return Ok(self.minimum);
        }
        if value > self.maximum {
            return self.at_position(1.0);
        }
        if self.validate(value).is_ok() {
            // Keep the established arithmetic scale of already-valid values.
            // Reconstructing them from a step ordinal would normalize 12.50.
            let boundary = || {
                if amount > 0 {
                    self.at_position(1.0)
                } else {
                    Ok(self.minimum)
                }
            };
            let Some(delta) = self.step.0.checked_mul(Decimal::from(amount)) else {
                return boundary();
            };
            let Some(next) = value.0.checked_add(delta) else {
                return boundary();
            };
            let next = DecimalValue(next);
            return if next < self.minimum {
                Ok(self.minimum)
            } else if next > self.maximum {
                self.at_position(1.0)
            } else {
                Ok(next)
            };
        }
        let steps = value
            .checked_sub(self.minimum)?
            .0
            .checked_div(self.step.0)
            .ok_or(ParseIssue::Overflow)?;
        let maximum_steps = self
            .maximum
            .checked_sub(self.minimum)?
            .0
            .checked_div(self.step.0)
            .ok_or(ParseIssue::Overflow)?
            .floor();
        // Floor for forward motion and ceil for backward motion retain the
        // adjacent valid step instead of skipping it after an off-grid input.
        let anchor = if amount > 0 {
            steps.floor()
        } else {
            steps.ceil()
        };
        let distance = Decimal::from(amount.unsigned_abs());
        let next = if amount > 0 {
            if maximum_steps
                .checked_sub(anchor)
                .ok_or(ParseIssue::Overflow)?
                < distance
            {
                maximum_steps
            } else {
                anchor.checked_add(distance).ok_or(ParseIssue::Overflow)?
            }
        } else if anchor < distance {
            Decimal::ZERO
        } else {
            anchor
                .checked_sub(distance)
                .ok_or(ParseIssue::Overflow)?
                .min(maximum_steps)
        };
        self.minimum
            .checked_add(DecimalValue(next).checked_mul(self.step)?)
    }
}

#[cfg(test)]
mod slider_tests {
    #[test]
    fn off_grid_navigation_uses_adjacent_steps_and_clamps_loaded_values() {
        let slider = DecimalSlider::new(
            "10".parse().unwrap(),
            "37".parse().unwrap(),
            "5".parse().unwrap(),
        )
        .unwrap();
        for (step, expected) in [
            (crate::NavigationStep::Next, "20"),
            (crate::NavigationStep::Previous, "15"),
            (crate::NavigationStep::PageNext, "35"),
            (crate::NavigationStep::PagePrevious, "10"),
            (crate::NavigationStep::First, "10"),
            (crate::NavigationStep::Last, "35"),
        ] {
            assert_eq!(
                slider.navigate("17".parse().unwrap(), step).unwrap(),
                expected.parse().unwrap()
            );
        }
        assert_eq!(
            slider
                .navigate("-99".parse().unwrap(), crate::NavigationStep::Next)
                .unwrap(),
            "10".parse().unwrap()
        );
        assert_eq!(
            slider
                .navigate("99".parse().unwrap(), crate::NavigationStep::Previous)
                .unwrap(),
            "35".parse().unwrap()
        );
        assert_eq!(slider.position("-99".parse().unwrap()).unwrap(), 0.);
        assert_eq!(slider.position("99".parse().unwrap()).unwrap(), 1.);
        assert!((slider.position("17".parse().unwrap()).unwrap() - 7. / 27.).abs() < 0.00001);
        assert!(slider.validate("17".parse().unwrap()).is_err());
    }

    use super::*;
    #[test]
    fn page_steps_default_to_ten_and_follow_the_configured_count() {
        let base = DecimalSlider::new(
            "0".parse().unwrap(),
            "100".parse().unwrap(),
            "1".parse().unwrap(),
        )
        .unwrap();
        let from = |slider: DecimalSlider, step| {
            slider
                .navigate("50".parse().unwrap(), step)
                .unwrap()
                .to_string()
        };
        assert_eq!(base.page_steps().get(), 10);
        assert_eq!(from(base, crate::NavigationStep::PageNext), "60");
        let wide = base.with_page_steps(NonZeroU32::new(25).unwrap());
        assert_eq!(from(wide, crate::NavigationStep::PageNext), "75");
        assert_eq!(from(wide, crate::NavigationStep::PagePrevious), "25");
        assert_eq!(from(wide, crate::NavigationStep::Next), "51");
    }
    #[test]
    fn distance_steps_scale_by_the_step_and_must_fit_the_span() {
        let base = DecimalSlider::new(
            "0".parse().unwrap(),
            "1".parse().unwrap(),
            "0.25".parse().unwrap(),
        )
        .unwrap();
        assert_eq!(base.distance().unwrap(), "0".parse().unwrap());
        let spaced = base.with_distance_steps(2).unwrap();
        assert_eq!(spaced.distance().unwrap(), "0.5".parse().unwrap());
        assert!(base.with_distance_steps(4).is_ok());
        assert!(base.with_distance_steps(5).is_err());
    }
    #[test]
    fn domain_requests_clamp_and_round_on_the_pointer_lattice() {
        let slider = DecimalSlider::new(
            "-12.5".parse().unwrap(),
            "37.6".parse().unwrap(),
            "0.25".parse().unwrap(),
        )
        .unwrap();
        for (input, expected) in [
            ("-99", "-12.5"),
            ("99", "37.5"),
            ("12.63", "12.75"),
            ("12.62", "12.5"),
        ] {
            assert_eq!(
                slider.at_value(input.parse().unwrap()).unwrap(),
                expected.parse().unwrap()
            );
        }
        assert_eq!(slider.step(), "0.25".parse().unwrap());
    }
    #[test]
    fn exact_domain_request_does_not_round_trip_through_a_renderer_float() {
        let slider = DecimalSlider::new(
            "1000000000000".parse().unwrap(),
            "1000000000001".parse().unwrap(),
            "0.01".parse().unwrap(),
        )
        .unwrap();
        let next = slider
            .at_value("1000000000000.266".parse().unwrap())
            .unwrap();
        assert_eq!(next, "1000000000000.27".parse().unwrap());
        // The platform's Float32 metadata cannot distinguish these endpoints.
        assert_eq!(
            slider.minimum().renderer_float().unwrap(),
            slider.maximum().renderer_float().unwrap()
        );
        assert_eq!(slider.position(next).unwrap(), 0.27);
    }
    #[test]
    fn pointer_rounding_and_navigation_keep_decimal_lattice_exact() {
        let slider = DecimalSlider::new(
            "0.1".parse().unwrap(),
            "0.95".parse().unwrap(),
            "0.2".parse().unwrap(),
        )
        .unwrap();
        assert_eq!(slider.at_position(1.0).unwrap(), "0.9".parse().unwrap());
        assert_eq!(slider.at_position(0.26).unwrap(), "0.3".parse().unwrap());
        assert_eq!(
            slider
                .navigate("0.3".parse().unwrap(), crate::NavigationStep::Next)
                .unwrap(),
            "0.5".parse().unwrap()
        );
        assert!(slider.at_position(f32::NAN).is_err());
        assert!(slider.validate("0.2".parse().unwrap()).is_err());
        let huge = DecimalSlider::new(
            DecimalValue::ZERO,
            "79228162514264337593543950335".parse().unwrap(),
            "0.0000000000000000000000000001".parse().unwrap(),
        );
        assert_eq!(huge.unwrap_err(), ParseIssue::Overflow);
    }
}

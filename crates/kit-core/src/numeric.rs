//! Exact decimal editing, with locale-specific syntax and explicit rounding.
use crate::{DraftValue, ParseIssue, StepDirection};
use rust_decimal::{Decimal, RoundingStrategy};
use std::{fmt, marker::PhantomData, str::FromStr};

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberLocale {
    FrCa,
    FrFr,
    EnCa,
    EnUs,
}
impl NumberLocale {
    pub fn decimal_separator(self) -> char {
        match self {
            Self::FrCa | Self::FrFr => ',',
            Self::EnCa | Self::EnUs => '.',
        }
    }
    pub fn grouping_separator(self) -> char {
        match self {
            Self::FrCa => '\u{a0}',
            Self::FrFr => '\u{202f}',
            Self::EnCa | Self::EnUs => ',',
        }
    }
    fn is_group(self, c: char) -> bool {
        match self {
            Self::FrCa | Self::FrFr => matches!(c, ' ' | '\u{a0}' | '\u{202f}'),
            Self::EnCa | Self::EnUs => c == ',',
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
    pub locale: NumberLocale,
    pub scale: DecimalScale,
    pub sign: SignPolicy,
    pub bounds: Option<NumericBounds<DecimalValue>>,
}
impl DecimalRules {
    pub fn parse(self, text: &str) -> DraftValue<DecimalValue> {
        let canonical = match canonicalize(text, self.locale, true, self.sign) {
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
fn canonicalize<T>(
    text: &str,
    locale: NumberLocale,
    decimal: bool,
    sign: SignPolicy,
) -> Result<String, DraftValue<T>> {
    let text = text.trim();
    if text.is_empty() {
        return Err(DraftValue::Empty);
    }
    if let Some((mantissa, exponent)) = text.split_once(['e', 'E']) {
        let mantissa = canonicalize::<T>(mantissa, locale, true, sign)?;
        let digits = exponent.strip_prefix(['-', '+']).unwrap_or(exponent);
        if digits.is_empty() {
            return Err(DraftValue::Incomplete);
        }
        if !digits.bytes().all(|c| c.is_ascii_digit()) {
            return Err(DraftValue::Invalid(ParseIssue::InvalidSyntax));
        }
        let exponent = exponent
            .parse::<i32>()
            .map_err(|_| DraftValue::Invalid(ParseIssue::Overflow))?;
        return expand_exponent(&mantissa, exponent).map_err(DraftValue::Invalid);
    }
    let negative = text.starts_with('-');
    if negative && sign == SignPolicy::NonNegative {
        return Err(DraftValue::Invalid(ParseIssue::BelowMinimum));
    }
    let unsigned = text.strip_prefix(['-', '+']).unwrap_or(text);
    if unsigned.is_empty() {
        return Err(DraftValue::Incomplete);
    }
    let mut parts = unsigned.split(locale.decimal_separator());
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
    let groups: Vec<&str> = integer.split(|c| locale.is_group(c)).collect();
    if groups.len() > 1
        && (groups[0].is_empty()
            || groups[0].len() > 3
            || groups.iter().skip(1).any(|group| group.len() != 3))
    {
        return Err(DraftValue::Invalid(ParseIssue::InvalidGrouping));
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
    locale: NumberLocale,
    bounds: Option<NumericBounds<i64>>,
) -> DraftValue<i64> {
    let canonical = match canonicalize(text, locale, true, SignPolicy::Signed) {
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
    locale: NumberLocale,
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
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            formatted.push(locale.grouping_separator());
        }
        formatted.push(c);
    }
    if let Some(fraction) = fraction {
        formatted.push(locale.decimal_separator());
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
impl CurrencyMetadata {
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
        locale: NumberLocale,
        rounding: RoundingPolicy,
    ) -> Result<String, ParseIssue> {
        if self.currency != metadata.code {
            return Err(ParseIssue::InvalidCurrency);
        }
        let amount = format_decimal(self.amount, locale, metadata.scale, rounding)?;
        match locale {
            NumberLocale::FrCa | NumberLocale::FrFr => {
                Ok(format!("{amount}\u{a0}{}", metadata.symbol))
            }
            NumberLocale::EnCa | NumberLocale::EnUs => {
                Ok(if let Some(unsigned) = amount.strip_prefix('-') {
                    format!("-{}{unsigned}", metadata.symbol)
                } else {
                    format!("{}{amount}", metadata.symbol)
                })
            }
        }
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
        let factor = match unit {
            DurationUnit::Milliseconds => 1,
            DurationUnit::Seconds => 1_000,
            DurationUnit::Minutes => 60_000,
            DurationUnit::Hours => 3_600_000,
        };
        let result = value.checked_mul(DecimalValue::from_integer(factor))?;
        if result.0.fract() != Decimal::ZERO {
            return Err(ParseIssue::ScaleExceeded);
        }
        result
            .0
            .to_u64()
            .map(Self::from_milliseconds)
            .ok_or(ParseIssue::Overflow)
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
    fn rules(locale: NumberLocale) -> DecimalRules {
        DecimalRules {
            locale,
            scale: DecimalScale::new(2).unwrap(),
            sign: SignPolicy::Signed,
            bounds: None,
        }
    }
    #[test]
    fn exact_and_locale_round_trip() {
        assert_eq!(d("0.10").checked_add(d("0.20")).unwrap(), d("0.30"));
        for locale in [
            NumberLocale::FrCa,
            NumberLocale::FrFr,
            NumberLocale::EnCa,
            NumberLocale::EnUs,
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
            rules(NumberLocale::FrCa).parse("1,"),
            DraftValue::Incomplete
        );
        assert_eq!(rules(NumberLocale::FrCa).parse("-"), DraftValue::Incomplete);
        assert_eq!(rules(NumberLocale::FrCa).parse(""), DraftValue::Empty);
        assert_eq!(
            rules(NumberLocale::EnCa).parse("1,23.40"),
            DraftValue::Invalid(ParseIssue::InvalidGrouping)
        );
        assert_eq!(
            parse_integer("9223372036854775808", NumberLocale::EnUs, None),
            DraftValue::Invalid(ParseIssue::Overflow)
        );
        assert_eq!(
            rules(NumberLocale::FrFr).parse("1\u{202f}234,50"),
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
                    NumberLocale::EnCa,
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
                    NumberLocale::FrCa,
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
                    .format(&metadata, NumberLocale::EnUs, RoundingPolicy::Reject)
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
            parse_integer("1e2", NumberLocale::EnCa, None),
            DraftValue::Parsed(100)
        );
        assert_eq!(
            parse_integer("12.0", NumberLocale::EnCa, None),
            DraftValue::Parsed(12)
        );
        assert_eq!(
            parse_integer("1,5e2", NumberLocale::FrCa, None),
            DraftValue::Parsed(150)
        );
        assert_eq!(
            parse_integer("1e", NumberLocale::EnCa, None),
            DraftValue::Incomplete
        );
        assert_eq!(
            parse_integer("1e-1", NumberLocale::EnCa, None),
            DraftValue::Invalid(ParseIssue::InvalidSyntax)
        );
        let rules = DecimalRules {
            locale: NumberLocale::EnCa,
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

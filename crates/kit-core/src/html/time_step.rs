// SPDX-License-Identifier: GPL-3.0-or-later
use super::{HtmlConstraints, numeric_policy, numeric_step, numeric_value};
use crate::{HtmlInputKind, LocalTime, StepDirection, TimePrecision};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};
use std::num::NonZeroU32;

// Bounds live on one lifted day. A periodic interval starts at min and ends
// at max on the next day; it is not an inverted numeric interval.
enum TimeBounds {
    Unbounded,
    Linear {
        first: BigInt,
        last: BigInt,
    },
    Periodic {
        first: BigInt,
        last: BigInt,
        minimum: BigRational,
        maximum: BigRational,
    },
}
struct TimeLattice {
    base: BigInt,
    stride: BigInt,
    count: BigInt,
    bounds: TimeBounds,
    default_seed: Option<BigRational>,
}
fn modulo(value: BigInt, modulus: &BigInt) -> BigInt {
    ((value % modulus) + modulus) % modulus
}
// Extended Euclid gives the exact intersection q*k = base + step*n. No
// enumerating seconds, rounding a fractional configuration or renderer float.
fn intersection(base: &BigRational, step: &BigRational, quantum: i64) -> Option<(BigInt, BigInt)> {
    let a = BigInt::from(quantum) * base.denom() * step.denom();
    let c = base.numer() * step.denom();
    let d = step.numer() * base.denom();
    let (mut old_r, mut r) = (a, d.clone());
    let (mut old_s, mut s) = (BigInt::from(1), BigInt::from(0));
    while !r.is_zero() {
        let quotient = &old_r / &r;
        (old_r, r) = (r.clone(), old_r - &quotient * r);
        (old_s, s) = (s.clone(), old_s - quotient * s);
    }
    if !(&c % &old_r).is_zero() {
        return None;
    }
    let period = d / &old_r;
    let first = modulo((c / old_r) * old_s, &period);
    Some((first * quantum, period * quantum))
}
impl TimeLattice {
    fn new(rules: &HtmlConstraints, precision: TimePrecision) -> Option<Self> {
        let quantum = match precision {
            TimePrecision::Minute => 60_000,
            TimePrecision::Second => 1_000,
        };
        let minimum = rules
            .minimum
            .as_deref()
            .and_then(|v| numeric_value(HtmlInputKind::Time, v));
        let maximum = rules
            .maximum
            .as_deref()
            .and_then(|v| numeric_value(HtmlInputKind::Time, v));
        let default_value = numeric_value(HtmlInputKind::Time, &rules.default_value);
        let default_seed = if minimum.is_none() {
            default_value.clone()
        } else {
            None
        };
        let base = minimum
            .clone()
            .or(default_value)
            .unwrap_or_else(BigRational::zero);
        let (default, factor) = numeric_policy(HtmlInputKind::Time)?;
        let (base, stride) = match numeric_step(rules, default, factor) {
            Some(step) => intersection(&base, &step, quantum)?,
            None => (BigInt::from(0), BigInt::from(quantum)),
        };
        let day_last = BigInt::from(86_400_000 - quantum);
        if base > day_last {
            return None;
        }
        let last = (&day_last - &base) / &stride;
        let count = &last + 1;
        let b = BigRational::from_integer(base.clone());
        let s = BigRational::from_integer(stride.clone());
        let interval = |lower: BigRational, upper: BigRational| {
            let first = ((lower - &b) / &s).ceil().to_integer().max(BigInt::from(0));
            let end = ((upper - &b) / &s).floor().to_integer().min(last.clone());
            (first <= end).then_some((first, end))
        };
        let bounds = match (minimum, maximum) {
            (None, None) => TimeBounds::Unbounded,
            (Some(min), Some(max)) if min > max => {
                let high = interval(min.clone(), BigRational::from_integer(day_last));
                let low = interval(BigRational::zero(), max.clone());
                let first = high
                    .as_ref()
                    .map(|v| v.0.clone())
                    .or_else(|| low.as_ref().map(|v| &v.0 + &count))?;
                let end = low
                    .as_ref()
                    .map(|v| &v.1 + &count)
                    .or_else(|| high.as_ref().map(|v| v.1.clone()))?;
                TimeBounds::Periodic {
                    first,
                    last: end,
                    minimum: min,
                    maximum: max,
                }
            }
            (min, max) => {
                let (first, last) = interval(
                    min.unwrap_or_else(BigRational::zero),
                    max.unwrap_or_else(|| BigRational::from_integer(day_last)),
                )?;
                TimeBounds::Linear { first, last }
            }
        };
        Some(Self {
            base,
            stride,
            count,
            bounds,
            default_seed,
        })
    }
    fn position(&self, value: &BigRational, direction: StepDirection) -> BigRational {
        let position = (value - BigRational::from_integer(self.base.clone()))
            / BigRational::from_integer(self.stride.clone());
        match &self.bounds {
            TimeBounds::Unbounded | TimeBounds::Linear { .. } => position,
            TimeBounds::Periodic {
                minimum, maximum, ..
            } => {
                if value < minimum && (value <= maximum || direction == StepDirection::Decrement) {
                    position + BigRational::from_integer(self.count.clone())
                } else {
                    position
                }
            }
        }
    }
    fn value(&self, index: BigInt) -> Option<LocalTime> {
        let millis = (&self.base + modulo(index, &self.count) * &self.stride).to_u32()?;
        let seconds = millis / 1_000;
        LocalTime::new(seconds / 3_600, seconds % 3_600 / 60, seconds % 60).ok()
    }
    fn first(&self) -> BigInt {
        match &self.bounds {
            TimeBounds::Unbounded => BigInt::from(0),
            TimeBounds::Linear { first, .. } | TimeBounds::Periodic { first, .. } => first.clone(),
        }
    }
    fn contains(&self, value: &BigRational) -> bool {
        let position = self.position(value, StepDirection::Increment);
        if !position.is_integer() {
            return false;
        }
        match &self.bounds {
            TimeBounds::Unbounded => {
                position >= BigRational::zero()
                    && position < BigRational::from_integer(self.count.clone())
            }
            TimeBounds::Linear { first, last } | TimeBounds::Periodic { first, last, .. } => {
                position >= BigRational::from_integer(first.clone())
                    && position <= BigRational::from_integer(last.clone())
            }
        }
    }
}
impl HtmlConstraints {
    /// Adjust a parsed local time on the exact HTML step lattice intersected
    /// with the chosen typed precision. Empty input seeds min/default/first
    /// permitted time without an extra increment. Invalid drafts are rejected
    /// by the caller at its parsing boundary. None means no admissible move.
    /// Unbounded times wrap by one day; bounded intervals clamp, including
    /// periodic min > max intervals which cross midnight internally.
    pub fn step_time(
        &self,
        current: Option<LocalTime>,
        direction: StepDirection,
        segment_seconds: NonZeroU32,
        precision: TimePrecision,
    ) -> Option<LocalTime> {
        if self.kind != HtmlInputKind::Time
            || self.disabled
            || self.read_only
            || !self.custom_error.is_empty()
        {
            return None;
        }
        let grid = TimeLattice::new(self, precision)?;
        let Some(current) = current else {
            if let Some(value) = &grid.default_seed
                && grid.contains(value)
            {
                return grid.value(grid.position(value, direction).to_integer());
            }
            return grid.value(grid.first());
        };
        let millis = (i64::from(current.hour()) * 3_600
            + i64::from(current.minute()) * 60
            + i64::from(current.second()))
            * 1_000;
        let position = grid.position(&BigRational::from_integer(millis.into()), direction);
        let count = BigRational::new(
            BigInt::from(segment_seconds.get()) * 1_000,
            grid.stride.clone(),
        )
        .ceil()
        .to_integer()
        .max(BigInt::from(1));
        let next = match direction {
            StepDirection::Increment => {
                if position.is_integer() {
                    position.to_integer() + count
                } else {
                    position.ceil().to_integer() + count - 1
                }
            }
            StepDirection::Decrement => {
                if position.is_integer() {
                    position.to_integer() - count
                } else {
                    position.floor().to_integer() - count + 1
                }
            }
        };
        let index = match &grid.bounds {
            TimeBounds::Unbounded => next,
            TimeBounds::Linear { first, last } | TimeBounds::Periodic { first, last, .. } => {
                let next = next.max(first.clone()).min(last.clone());
                let next_position = BigRational::from_integer(next.clone());
                let overshoot = match direction {
                    StepDirection::Increment => next_position < position,
                    StepDirection::Decrement => next_position > position,
                };
                if overshoot {
                    return None;
                }
                next
            }
        };
        grid.value(index).filter(|next| *next != current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClockFormat, HtmlInputState, NumberStep, NumberStepOutcome};

    fn time(hour: u32, minute: u32, second: u32) -> LocalTime {
        LocalTime::new(hour, minute, second).unwrap()
    }
    fn rules(step: &str) -> HtmlConstraints {
        HtmlConstraints {
            kind: HtmlInputKind::Time,
            step: Some(step.into()),
            ..Default::default()
        }
    }
    fn adjust(
        rules: &HtmlConstraints,
        current: Option<LocalTime>,
        direction: StepDirection,
        segment_seconds: u32,
        precision: TimePrecision,
    ) -> Option<LocalTime> {
        let next = rules.step_time(
            current,
            direction,
            NonZeroU32::new(segment_seconds).unwrap(),
            precision,
        );
        if let Some(value) = next {
            let clock = if rules.twelve_hour {
                ClockFormat::TwelveHour
            } else {
                ClockFormat::TwentyFourHour
            };
            let text = value.format(precision, clock);
            assert!(
                rules.validate(&text, HtmlInputState::default()).valid(),
                "candidate {text} violates {rules:?}"
            );
            match precision {
                TimePrecision::Minute => assert_eq!(value.second(), 0),
                TimePrecision::Second => {}
            }
            assert_ne!(Some(value), current, "an unchanged bound is a no-op");
        }
        next
    }

    #[test]
    fn empty_seed_uses_minimum_then_admissible_default_without_an_extra_step() {
        for direction in [StepDirection::Increment, StepDirection::Decrement] {
            for precision in [TimePrecision::Minute, TimePrecision::Second] {
                let mut r = rules("900");
                r.required = true;
                assert_eq!(
                    adjust(&r, None, direction, 60, precision),
                    Some(time(0, 0, 0))
                );
                r.default_value = "09:37".into();
                assert_eq!(
                    adjust(&r, None, direction, 3600, precision),
                    Some(time(9, 37, 0))
                );
                r.minimum = Some("09:10".into());
                r.maximum = Some("10:00".into());
                assert_eq!(
                    adjust(&r, None, direction, 60, precision),
                    Some(time(9, 10, 0))
                );
                r.minimum = None;
                r.maximum = Some("01:00".into());
                r.default_value = "22:00".into();
                assert_eq!(
                    adjust(&r, None, direction, 60, precision),
                    Some(time(0, 0, 0))
                );
            }
        }
        let mut r = rules("0.5");
        r.minimum = Some("00:00:00.500".into());
        r.maximum = Some("00:00:01.500".into());
        assert_eq!(
            adjust(&r, None, StepDirection::Increment, 1, TimePrecision::Second),
            Some(time(0, 0, 1))
        );
    }

    #[test]
    fn quarter_hour_steps_use_segment_magnitude_and_directional_readmission() {
        let r = rules("900");
        for (segment, expected) in [
            (1, time(12, 15, 0)),
            (60, time(12, 15, 0)),
            (3600, time(13, 0, 0)),
        ] {
            assert_eq!(
                adjust(
                    &r,
                    Some(time(12, 0, 0)),
                    StepDirection::Increment,
                    segment,
                    TimePrecision::Second
                ),
                Some(expected)
            );
        }
        for (direction, segment, expected) in [
            (StepDirection::Increment, 60, time(12, 15, 0)),
            (StepDirection::Decrement, 60, time(12, 0, 0)),
            (StepDirection::Increment, 3600, time(13, 0, 0)),
        ] {
            assert_eq!(
                adjust(
                    &r,
                    Some(time(12, 1, 0)),
                    direction,
                    segment,
                    TimePrecision::Minute
                ),
                Some(expected)
            );
        }
        let mut r = r;
        r.minimum = Some("09:10".into());
        assert_eq!(
            adjust(
                &r,
                Some(time(9, 11, 0)),
                StepDirection::Increment,
                60,
                TimePrecision::Minute
            ),
            Some(time(9, 25, 0))
        );
        let mut twelve_hour = rules("900");
        twelve_hour.twelve_hour = true;
        assert_eq!(
            adjust(
                &twelve_hour,
                Some(time(11, 45, 0)),
                StepDirection::Increment,
                60,
                TimePrecision::Minute
            ),
            Some(time(12, 0, 0))
        );
        assert_eq!(
            adjust(
                &twelve_hour,
                Some(time(23, 45, 0)),
                StepDirection::Increment,
                60,
                TimePrecision::Minute
            ),
            Some(time(0, 0, 0))
        );
    }

    #[test]
    fn whole_time_carry_borrow_and_unbounded_midnight_wrap_stay_on_the_grid() {
        let minute = rules("60");
        for (current, direction, segment, expected) in [
            (
                time(12, 59, 0),
                StepDirection::Increment,
                60,
                time(13, 0, 0),
            ),
            (time(23, 59, 0), StepDirection::Increment, 60, time(0, 0, 0)),
            (time(0, 0, 0), StepDirection::Decrement, 60, time(23, 59, 0)),
            (
                time(0, 30, 0),
                StepDirection::Decrement,
                3600,
                time(23, 30, 0),
            ),
        ] {
            assert_eq!(
                adjust(
                    &minute,
                    Some(current),
                    direction,
                    segment,
                    TimePrecision::Minute
                ),
                Some(expected)
            );
        }
        let second = rules("1");
        assert_eq!(
            adjust(
                &second,
                Some(time(23, 59, 59)),
                StepDirection::Increment,
                1,
                TimePrecision::Second
            ),
            Some(time(0, 0, 0))
        );
        assert_eq!(
            adjust(
                &second,
                Some(time(0, 0, 0)),
                StepDirection::Decrement,
                1,
                TimePrecision::Second
            ),
            Some(time(23, 59, 59))
        );
        let quarter = rules("900");
        assert_eq!(
            adjust(
                &quarter,
                Some(time(0, 0, 0)),
                StepDirection::Decrement,
                60,
                TimePrecision::Minute
            ),
            Some(time(23, 45, 0))
        );
    }

    #[test]
    fn linear_and_periodic_bounds_clamp_without_reversing_direction() {
        let mut r = rules("900");
        r.minimum = Some("09:00".into());
        r.maximum = Some("10:00".into());
        for (current, direction, segment, expected) in [
            (time(9, 0, 0), StepDirection::Decrement, 60, None),
            (time(10, 0, 0), StepDirection::Increment, 60, None),
            (
                time(9, 45, 0),
                StepDirection::Increment,
                3600,
                Some(time(10, 0, 0)),
            ),
            (
                time(8, 59, 0),
                StepDirection::Increment,
                60,
                Some(time(9, 0, 0)),
            ),
            (
                time(10, 1, 0),
                StepDirection::Decrement,
                60,
                Some(time(10, 0, 0)),
            ),
            (time(8, 59, 0), StepDirection::Decrement, 60, None),
        ] {
            assert_eq!(
                adjust(&r, Some(current), direction, segment, TimePrecision::Minute),
                expected
            );
        }
        r.minimum = Some("23:00".into());
        r.maximum = Some("02:00".into());
        assert_eq!(
            adjust(
                &r,
                None,
                StepDirection::Decrement,
                60,
                TimePrecision::Minute
            ),
            Some(time(23, 0, 0))
        );
        for (current, direction, expected) in [
            (
                time(23, 45, 0),
                StepDirection::Increment,
                Some(time(0, 0, 0)),
            ),
            (
                time(0, 0, 0),
                StepDirection::Decrement,
                Some(time(23, 45, 0)),
            ),
            (time(2, 0, 0), StepDirection::Increment, None),
            (time(23, 0, 0), StepDirection::Decrement, None),
            (
                time(12, 0, 0),
                StepDirection::Increment,
                Some(time(23, 0, 0)),
            ),
            (
                time(12, 0, 0),
                StepDirection::Decrement,
                Some(time(2, 0, 0)),
            ),
        ] {
            assert_eq!(
                adjust(&r, Some(current), direction, 60, TimePrecision::Minute),
                expected
            );
        }
    }

    #[test]
    fn fractional_steps_intersect_precision_exactly_or_offer_no_candidate() {
        for (step, precision, expected) in [
            ("0.5", TimePrecision::Second, time(0, 0, 1)),
            ("0.0015", TimePrecision::Second, time(0, 0, 3)),
            ("90", TimePrecision::Minute, time(0, 3, 0)),
        ] {
            assert_eq!(
                adjust(
                    &rules(step),
                    Some(time(0, 0, 0)),
                    StepDirection::Increment,
                    1,
                    precision
                ),
                Some(expected)
            );
        }
        let mut r = rules("1");
        r.minimum = Some("00:00:00.500".into());
        assert_eq!(
            adjust(&r, None, StepDirection::Increment, 1, TimePrecision::Second),
            None
        );
        r.minimum = Some("00:00:00.001".into());
        r.step = Some("0.002".into());
        assert_eq!(
            adjust(
                &r,
                None,
                StepDirection::Increment,
                60,
                TimePrecision::Minute
            ),
            None
        );
        r.step = Some("any".into());
        r.minimum = Some("09:00:00.001".into());
        r.maximum = Some("09:00:00.999".into());
        assert_eq!(
            adjust(&r, None, StepDirection::Increment, 1, TimePrecision::Second),
            None
        );
        let huge = rules("1e100");
        assert_eq!(
            adjust(
                &huge,
                Some(time(0, 0, 0)),
                StepDirection::Increment,
                3600,
                TimePrecision::Second
            ),
            None
        );
    }

    #[test]
    fn html_fallback_any_and_access_rules_remain_explicit() {
        for step in ["0", "-1", "invalid"] {
            assert_eq!(
                adjust(
                    &rules(step),
                    Some(time(12, 0, 0)),
                    StepDirection::Increment,
                    1,
                    TimePrecision::Second
                ),
                Some(time(12, 1, 0))
            );
        }
        for step in ["any", "ANY", "AnY"] {
            assert_eq!(
                adjust(
                    &rules(step),
                    Some(time(12, 0, 1)),
                    StepDirection::Increment,
                    1,
                    TimePrecision::Second
                ),
                Some(time(12, 0, 2))
            );
            let mut number = rules(step);
            number.kind = HtmlInputKind::Number;
            assert_eq!(
                number.step_number(
                    "1",
                    NumberStep::Steps {
                        direction: StepDirection::Increment,
                        count: NonZeroU32::new(1).unwrap()
                    }
                ),
                NumberStepOutcome::Unavailable
            );
        }
        let mut r = rules("60");
        r.minimum = Some("25:00".into());
        r.maximum = Some("not a time".into());
        assert_eq!(
            adjust(
                &r,
                Some(time(23, 59, 0)),
                StepDirection::Increment,
                60,
                TimePrecision::Minute
            ),
            Some(time(0, 0, 0))
        );
        r.disabled = true;
        assert_eq!(
            adjust(
                &r,
                None,
                StepDirection::Increment,
                60,
                TimePrecision::Minute
            ),
            None
        );
        r.disabled = false;
        r.read_only = true;
        assert_eq!(
            adjust(
                &r,
                None,
                StepDirection::Increment,
                60,
                TimePrecision::Minute
            ),
            None
        );
        r.read_only = false;
        r.custom_error = "Host rejection".into();
        assert_eq!(
            adjust(
                &r,
                None,
                StepDirection::Increment,
                60,
                TimePrecision::Minute
            ),
            None
        );
        r.custom_error.clear();
        r.kind = HtmlInputKind::Date;
        assert_eq!(
            adjust(
                &r,
                None,
                StepDirection::Increment,
                60,
                TimePrecision::Minute
            ),
            None
        );
    }

    #[test]
    fn returned_candidates_always_satisfy_constraints_across_representative_lattices() {
        for (step, minimum, maximum, default) in [
            ("900", "", "", ""),
            ("420", "00:01", "", ""),
            ("90", "", "", "00:00:30"),
            ("0.0015", "", "", ""),
            ("0.002", "00:00:00.001", "", ""),
            ("any", "09:00", "10:03", ""),
            ("900", "23:00", "02:00", ""),
            ("any", "23:59:59.500", "00:00:00.500", ""),
        ] {
            let mut r = rules(step);
            r.minimum = (!minimum.is_empty()).then(|| minimum.into());
            r.maximum = (!maximum.is_empty()).then(|| maximum.into());
            r.default_value = default.into();
            for precision in [TimePrecision::Minute, TimePrecision::Second] {
                for direction in [StepDirection::Increment, StepDirection::Decrement] {
                    adjust(&r, None, direction, 60, precision);
                    for current in [
                        time(0, 0, 0),
                        time(0, 1, 0),
                        time(1, 59, 59),
                        time(2, 0, 0),
                        time(9, 10, 0),
                        time(12, 59, 59),
                        time(23, 59, 59),
                    ] {
                        for segment in [1, 60, 3600] {
                            adjust(&r, Some(current), direction, segment, precision);
                        }
                    }
                }
            }
        }
    }
}

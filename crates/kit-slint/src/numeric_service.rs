// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::ui;
use kit_core::{DecimalSlider, DecimalValue, OrderedRange, ParseIssue};
use std::num::NonZeroU32;
impl TryFrom<ui::DecimalRangeView> for OrderedRange<DecimalValue> {
    type Error = ParseIssue;
    fn try_from(value: ui::DecimalRangeView) -> Result<Self, Self::Error> {
        Self::new(value.lower.parse()?, value.upper.parse()?)
    }
}
fn spec(value: ui::SliderSpec) -> Result<DecimalSlider, ParseIssue> {
    let slider = DecimalSlider::new(
        value.minimum.parse()?,
        value.maximum.parse()?,
        value.step.parse()?,
    )?;
    // Zero keeps the kit default of ten steps per page.
    let slider = match u32::try_from(value.page_steps).map_err(|_| ParseIssue::InvalidRange)? {
        0 => slider,
        count => slider.with_page_steps(NonZeroU32::new(count).ok_or(ParseIssue::InvalidRange)?),
    };
    slider
        .with_distance_steps(u32::try_from(value.min_steps).map_err(|_| ParseIssue::InvalidRange)?)
}
/// The lower and upper values after one thumb moves to `next`. The moving thumb
/// stops at the other thumb, plus the minimum distance between them.
fn fence(
    thumb: ui::RangeThumb,
    next: DecimalValue,
    range: OrderedRange<DecimalValue>,
    rules: DecimalSlider,
) -> Result<(DecimalValue, DecimalValue), ParseIssue> {
    let gap = rules.distance()?;
    Ok(match thumb {
        ui::RangeThumb::Lower => (next.min(range.end().checked_sub(gap)?), range.end()),
        ui::RangeThumb::Upper => (range.start(), next.max(range.start().checked_add(gap)?)),
    })
}
fn accessible_value(
    value: DecimalValue,
    minimum: DecimalValue,
    maximum: DecimalValue,
    step: DecimalValue,
) -> Result<ui::SliderAccessibleValue, ParseIssue> {
    Ok(ui::SliderAccessibleValue {
        valid: true,
        minimum: minimum.renderer_float()?,
        maximum: maximum.renderer_float()?,
        step: step.renderer_float()?,
        value: value.to_string().into(),
    })
}
fn slider_accessibility(
    value: slint::SharedString,
    rules: ui::SliderSpec,
) -> Result<ui::SliderAccessibleValue, ParseIssue> {
    let rules = spec(rules)?;
    let value = value.parse()?;
    rules.validate(value)?;
    accessible_value(value, rules.minimum(), rules.maximum(), rules.step())
}
fn range_values(
    current: ui::DecimalRangeView,
    rules: ui::SliderSpec,
) -> Result<(OrderedRange<DecimalValue>, DecimalSlider), ParseIssue> {
    let range = OrderedRange::<DecimalValue>::try_from(current)?;
    let rules = spec(rules)?;
    rules.validate(range.start())?;
    rules.validate(range.end())?;
    if range.end().checked_sub(range.start())? < rules.distance()? {
        return Err(ParseIssue::InvalidRange);
    }
    Ok((range, rules))
}
fn normalize_range(
    range: OrderedRange<DecimalValue>,
    rules: DecimalSlider,
) -> Result<OrderedRange<DecimalValue>, ParseIssue> {
    let admitted = |value| {
        if rules.validate(value).is_ok() {
            Ok(value)
        } else {
            rules.at_value(value)
        }
    };
    let (start, end) = (admitted(range.start())?, admitted(range.end())?);
    let gap = rules.distance()?;
    if end.checked_sub(start)? >= gap {
        return OrderedRange::new(start, end);
    }
    // The thumbs sit closer than the minimum distance. Push the upper thumb
    // away, or pull the lower thumb back when the upper one is at the maximum.
    let pushed = start.checked_add(gap)?;
    if pushed <= rules.maximum() {
        OrderedRange::new(start, pushed)
    } else {
        OrderedRange::new(rules.maximum().checked_sub(gap)?, rules.maximum())
    }
}
#[cfg(debug_assertions)]
fn install_spec_diagnostics(service: &ui::KitFieldService<'_>) {
    let seen = std::cell::RefCell::new(std::collections::HashSet::new());
    service.on_slider_invalid_spec(move |value| {
        if let Err(issue) = spec(value.clone()) {
            let key = (value.minimum.to_string(), value.maximum.to_string(), value.step.to_string());
            if seen.borrow_mut().insert(key) {
                eprintln!("Slint Kit: invalid slider specification ({issue:?}): minimum={}, maximum={}, step={}", value.minimum, value.maximum, value.step);
            }
        }
    });
}
fn range_view(lower: DecimalValue, upper: DecimalValue) -> ui::DecimalRangeView {
    ui::DecimalRangeView {
        lower: lower.to_string().into(),
        upper: upper.to_string().into(),
    }
}
fn range_accessibility(
    current: ui::DecimalRangeView,
    rules: ui::SliderSpec,
) -> Result<ui::RangeAccessibleValue, ParseIssue> {
    let (range, rules) = range_values(current, rules)?;
    Ok(ui::RangeAccessibleValue {
        lower: accessible_value(
            range.start(),
            rules.minimum(),
            range.end().checked_sub(rules.distance()?)?,
            rules.step(),
        )?,
        upper: accessible_value(
            range.end(),
            range.start().checked_add(rules.distance()?)?,
            rules.maximum(),
            rules.step(),
        )?,
    })
}
fn slider_set_value(
    current: slint::SharedString,
    proposed: slint::SharedString,
    rules: ui::SliderSpec,
) -> Result<ui::NumberStepResult, ParseIssue> {
    let rules = spec(rules)?;
    let current = current.parse()?;
    rules.validate(current)?;
    let next = rules.at_value(proposed.parse()?)?;
    Ok(ui::NumberStepResult {
        status: if next == current {
            ui::NumberStepStatus::Unchanged
        } else {
            ui::NumberStepStatus::Stepped
        },
        text: next.to_string().into(),
    })
}
fn range_set_value(
    current: ui::DecimalRangeView,
    thumb: ui::RangeThumb,
    proposed: slint::SharedString,
    rules: ui::SliderSpec,
) -> Result<ui::RangeSetValueResult, ParseIssue> {
    let (range, rules) = range_values(current, rules)?;
    let next = rules.at_value(proposed.parse()?)?;
    let (lower, upper) = fence(thumb, next, range, rules)?;
    Ok(ui::RangeSetValueResult {
        status: if lower == range.start() && upper == range.end() {
            ui::NumberStepStatus::Unchanged
        } else {
            ui::NumberStepStatus::Stepped
        },
        value: range_view(lower, upper),
    })
}
pub(crate) fn install_numeric(service: &ui::KitFieldService<'_>) {
    service.on_slider_spec_valid(|value| spec(value).is_ok());
    #[cfg(debug_assertions)]
    install_spec_diagnostics(service);
    service.on_slider_accessibility(|value, rules| {
        slider_accessibility(value, rules).unwrap_or_default()
    });
    service.on_range_accessibility(|value, rules| {
        range_accessibility(value, rules).unwrap_or_default()
    });
    service.on_slider_set_value(|current, proposed, rules| {
        slider_set_value(current, proposed, rules).unwrap_or(ui::NumberStepResult {
            status: ui::NumberStepStatus::Unavailable,
            text: Default::default(),
        })
    });
    service.on_range_set_value(|current, thumb, proposed, rules| {
        range_set_value(current, thumb, proposed, rules).unwrap_or(ui::RangeSetValueResult {
            status: ui::NumberStepStatus::Unavailable,
            value: Default::default(),
        })
    });

    service.on_slider_valid(|value, rules| {
        spec(rules)
            .and_then(|rules| rules.validate(value.parse()?))
            .is_ok()
    });
    service.on_range_valid(|value, rules| range_values(value, rules).is_ok());
    service.on_range_gap_position(|rules| {
        spec(rules)
            .and_then(|rules| {
                let span = rules.maximum().checked_sub(rules.minimum())?;
                rules.position(rules.minimum().checked_add(rules.distance()?.min(span))?)
            })
            .unwrap_or(0.0)
    });
    service.on_slider_position(|value, rules| {
        spec(rules)
            .and_then(|rules| rules.position(value.parse()?))
            .unwrap_or(0.0)
    });
    service.on_slider_pointer(|value, position, rules| {
        spec(rules)
            .and_then(|rules| rules.at_position(position))
            .map(|next| next.to_string().into())
            .unwrap_or(value)
    });
    service.on_slider_navigate(|value, step, rules| {
        spec(rules)
            .and_then(|rules| rules.navigate(value.parse()?, step.into()))
            .map(|next| next.to_string().into())
            .unwrap_or(value)
    });
    service.on_range_pointer(|current, thumb, position, rules| {
        let update = || -> Result<ui::DecimalRangeView, ParseIssue> {
            let range = OrderedRange::<DecimalValue>::try_from(current.clone())?;
            let rules = spec(rules)?;
            let range = normalize_range(range, rules)?;
            let next = rules.at_position(position)?;
            let (lower, upper) = fence(thumb, next, range, rules)?;
            Ok(ui::DecimalRangeView {
                lower: lower.to_string().into(),
                upper: upper.to_string().into(),
            })
        };
        update().unwrap_or(current)
    });
    service.on_range_navigate(|current, thumb, step, rules| {
        let update = || -> Result<ui::DecimalRangeView, ParseIssue> {
            let range = OrderedRange::<DecimalValue>::try_from(current.clone())?;
            let rules = spec(rules)?;
            let raw = range;
            let range = normalize_range(raw, rules)?;
            let moved = match thumb {
                ui::RangeThumb::Lower => raw.start(),
                ui::RangeThumb::Upper => raw.end(),
            };
            let (lower, upper) = fence(thumb, rules.navigate(moved, step.into())?, range, rules)?;
            Ok(ui::DecimalRangeView {
                lower: lower.to_string().into(),
                upper: upper.to_string().into(),
            })
        };
        update().unwrap_or(current)
    });
}

impl From<ui::StepDirection> for kit_core::StepDirection {
    fn from(value: ui::StepDirection) -> Self {
        match value {
            ui::StepDirection::Increment => Self::Increment,
            ui::StepDirection::Decrement => Self::Decrement,
        }
    }
}
impl From<kit_core::StepDirection> for ui::StepDirection {
    fn from(value: kit_core::StepDirection) -> Self {
        match value {
            kit_core::StepDirection::Increment => Self::Increment,
            kit_core::StepDirection::Decrement => Self::Decrement,
        }
    }
}
impl From<ui::RangeThumb> for kit_core::RangeThumb {
    fn from(value: ui::RangeThumb) -> Self {
        match value {
            ui::RangeThumb::Lower => Self::Start,
            ui::RangeThumb::Upper => Self::End,
        }
    }
}
impl From<kit_core::RangeThumb> for ui::RangeThumb {
    fn from(value: kit_core::RangeThumb) -> Self {
        match value {
            kit_core::RangeThumb::Start => Self::Lower,
            kit_core::RangeThumb::End => Self::Upper,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rules() -> ui::SliderSpec {
        ui::SliderSpec {
            minimum: "-10".into(),
            maximum: "50".into(),
            step: "2".into(),
            page_steps: 0,
            min_steps: 0,
        }
    }
    fn spaced() -> ui::SliderSpec {
        ui::SliderSpec {
            min_steps: 3,
            ..rules()
        }
    }
    fn current() -> ui::DecimalRangeView {
        range_view("10".parse().unwrap(), "30".parse().unwrap())
    }
    #[test]
    fn accessible_profiles_share_domain_values_bounds_and_step() {
        let profile = slider_accessibility("12.00".into(), rules()).unwrap();
        assert!(profile.valid);
        assert_eq!(
            (profile.minimum, profile.maximum, profile.step),
            (-10., 50., 2.)
        );
        assert_eq!(profile.value, "12.00");
        let range = range_accessibility(current(), rules()).unwrap();
        assert_eq!(
            (range.lower.minimum, range.lower.maximum, range.lower.step),
            (-10., 30., 2.)
        );
        assert_eq!(
            (range.upper.minimum, range.upper.maximum, range.upper.step),
            (10., 50., 2.)
        );
        assert_eq!(range.lower.value, "10");
        assert_eq!(range.upper.value, "30");
    }
    #[test]
    fn invalid_current_domains_never_publish_a_valid_accessible_profile() {
        for value in ["garbage", "11", "99"] {
            assert!(slider_accessibility(value.into(), rules()).is_err());
        }
        let reversed = ui::DecimalRangeView {
            lower: "30".into(),
            upper: "10".into(),
        };
        assert!(range_accessibility(reversed, rules()).is_err());
        let mut invalid = rules();
        invalid.step = "0".into();
        assert!(slider_accessibility("12".into(), invalid).is_err());
    }
    #[test]
    fn set_value_uses_exact_rounding_and_rejects_invalid_external_text() {
        let next = slider_set_value("10".into(), "13".into(), rules()).unwrap();
        assert_eq!(next.status, ui::NumberStepStatus::Stepped);
        assert_eq!(next.text, "14");
        assert_eq!(
            slider_set_value("10".into(), "10.1".into(), rules())
                .unwrap()
                .status,
            ui::NumberStepStatus::Unchanged
        );
        for invalid in ["NaN", "inf", "", "1e2"] {
            assert!(slider_set_value("10".into(), invalid.into(), rules()).is_err());
        }
    }
    #[test]
    fn range_set_value_fences_the_other_thumb_after_rounding() {
        let lower =
            range_set_value(current(), ui::RangeThumb::Lower, "99".into(), rules()).unwrap();
        assert_eq!(
            lower.value,
            range_view("30".parse().unwrap(), "30".parse().unwrap())
        );
        assert_eq!(lower.status, ui::NumberStepStatus::Stepped);
        let upper =
            range_set_value(current(), ui::RangeThumb::Upper, "-99".into(), rules()).unwrap();
        assert_eq!(
            upper.value,
            range_view("10".parse().unwrap(), "10".parse().unwrap())
        );
        let unchanged =
            range_set_value(current(), ui::RangeThumb::Lower, "10.1".into(), rules()).unwrap();
        assert_eq!(unchanged.status, ui::NumberStepStatus::Unchanged);
    }
    #[test]
    fn minimum_distance_fences_the_moving_thumb_and_the_accessible_bounds() {
        let lower =
            range_set_value(current(), ui::RangeThumb::Lower, "99".into(), spaced()).unwrap();
        assert_eq!(
            lower.value,
            range_view("24".parse().unwrap(), "30".parse().unwrap())
        );
        let upper =
            range_set_value(current(), ui::RangeThumb::Upper, "-99".into(), spaced()).unwrap();
        assert_eq!(
            upper.value,
            range_view("10".parse().unwrap(), "16".parse().unwrap())
        );
        let profile = range_accessibility(current(), spaced()).unwrap();
        assert_eq!(profile.lower.maximum, 24.);
        assert_eq!(profile.upper.minimum, 16.);
        let closer = range_view("10".parse().unwrap(), "14".parse().unwrap());
        assert!(range_values(closer, spaced()).is_err());
    }
    #[test]
    fn minimum_distance_repairs_a_close_pair_on_the_pointer_path() {
        let close = range_view("10".parse().unwrap(), "12".parse().unwrap());
        let (range, rules) = (
            OrderedRange::<DecimalValue>::try_from(close).unwrap(),
            spec(spaced()).unwrap(),
        );
        let fixed = normalize_range(range, rules).unwrap();
        assert_eq!(fixed.end().to_string(), "16");
        let edge = range_view("48".parse().unwrap(), "50".parse().unwrap());
        let edge = OrderedRange::<DecimalValue>::try_from(edge).unwrap();
        let fixed = normalize_range(edge, rules).unwrap();
        assert_eq!(
            (fixed.start().to_string(), fixed.end().to_string()),
            ("44".to_string(), "50".to_string())
        );
    }
    #[test]
    fn page_steps_and_distance_reject_negative_or_oversized_values() {
        let negative = ui::SliderSpec {
            page_steps: -1,
            ..rules()
        };
        assert!(spec(negative).is_err());
        let wide = ui::SliderSpec {
            min_steps: 31,
            ..rules()
        };
        assert!(spec(wide).is_err());
        let paged = ui::SliderSpec {
            page_steps: 5,
            ..rules()
        };
        assert_eq!(spec(paged).unwrap().page_steps().get(), 5);
        assert_eq!(spec(rules()).unwrap().page_steps().get(), 10);
    }
}

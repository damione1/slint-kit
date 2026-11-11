// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::ui;
use kit_core::{DecimalSlider, DecimalValue, OrderedRange, ParseIssue};
impl TryFrom<ui::DecimalRangeView> for OrderedRange<DecimalValue> {
    type Error = ParseIssue;
    fn try_from(value: ui::DecimalRangeView) -> Result<Self, Self::Error> {
        Self::new(value.lower.parse()?, value.upper.parse()?)
    }
}
fn spec(value: ui::SliderSpec) -> Result<DecimalSlider, ParseIssue> {
    DecimalSlider::new(
        value.minimum.parse()?,
        value.maximum.parse()?,
        value.step.parse()?,
    )
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
    Ok((range, rules))
}
fn normalize_range(
    range: OrderedRange<DecimalValue>,
    rules: DecimalSlider,
) -> Result<OrderedRange<DecimalValue>, ParseIssue> {
    OrderedRange::new(rules.at_value(range.start())?, rules.at_value(range.end())?)
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
        lower: accessible_value(range.start(), rules.minimum(), range.end(), rules.step())?,
        upper: accessible_value(range.end(), range.start(), rules.maximum(), rules.step())?,
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
    let (lower, upper) = match thumb {
        ui::RangeThumb::Lower => (next.min(range.end()), range.end()),
        ui::RangeThumb::Upper => (range.start(), next.max(range.start())),
    };
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
    service.on_range_valid(|value, rules| {
        let valid = || -> Result<(), ParseIssue> {
            let rules = spec(rules)?;
            let range = OrderedRange::<DecimalValue>::try_from(value)?;
            rules.validate(range.start())?;
            rules.validate(range.end())?;
            Ok(())
        };
        valid().is_ok()
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
            let (lower, upper) = match thumb {
                ui::RangeThumb::Lower => (next.min(range.end()), range.end()),
                ui::RangeThumb::Upper => (range.start(), next.max(range.start())),
            };
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
            let (lower, upper) = match thumb {
                ui::RangeThumb::Lower => (
                    rules.navigate(raw.start(), step.into())?.min(range.end()),
                    range.end(),
                ),
                ui::RangeThumb::Upper => (
                    range.start(),
                    rules.navigate(raw.end(), step.into())?.max(range.start()),
                ),
            };
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
}

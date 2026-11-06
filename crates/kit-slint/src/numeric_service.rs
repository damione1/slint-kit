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
pub(crate) fn install_numeric(service: &ui::KitFieldService<'_>) {
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
            rules.validate(range.start())?;
            rules.validate(range.end())?;
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
            rules.validate(range.start())?;
            rules.validate(range.end())?;
            let (lower, upper) = match thumb {
                ui::RangeThumb::Lower => (
                    rules.navigate(range.start(), step.into())?.min(range.end()),
                    range.end(),
                ),
                ui::RangeThumb::Upper => (
                    range.start(),
                    rules.navigate(range.end(), step.into())?.max(range.start()),
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

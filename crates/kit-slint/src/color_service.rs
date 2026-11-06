// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::ui;
use kit_core::{AlphaPolicy, DraftValue, ParseIssue, SrgbColor};
impl From<SrgbColor> for ui::SrgbColorView {
    fn from(value: SrgbColor) -> Self {
        Self {
            red: i32::from(value.red),
            green: i32::from(value.green),
            blue: i32::from(value.blue),
            alpha: i32::from(value.alpha),
        }
    }
}
impl TryFrom<ui::SrgbColorView> for SrgbColor {
    type Error = ParseIssue;
    fn try_from(value: ui::SrgbColorView) -> Result<Self, Self::Error> {
        let byte = |value| u8::try_from(value).map_err(|_| ParseIssue::InvalidColor);
        Ok(Self {
            red: byte(value.red)?,
            green: byte(value.green)?,
            blue: byte(value.blue)?,
            alpha: byte(value.alpha)?,
        })
    }
}
fn policy(value: ui::ColorAlphaMode) -> AlphaPolicy {
    match value {
        ui::ColorAlphaMode::Opaque => AlphaPolicy::Opaque,
        ui::ColorAlphaMode::Editable => AlphaPolicy::Editable,
    }
}
fn project(value: Result<SrgbColor, ParseIssue>, mode: AlphaPolicy) -> ui::ColorResult {
    match value {
        Ok(mut value) => {
            if mode == AlphaPolicy::Opaque {
                value.alpha = 255;
            }
            ui::ColorResult {
                valid: true,
                text: value.hex(mode).into(),
                value: value.into(),
            }
        }
        Err(_) => ui::ColorResult::default(),
    }
}
pub(crate) fn install_color(service: &ui::KitFieldService<'_>) {
    service.on_color_format(|value, mode| project(value.try_into(), policy(mode)).text);
    service.on_color_validate(|value, mode| project(value.try_into(), policy(mode)));
    service.on_color_parse(|text, mode| {
        let mode = policy(mode);
        let text = if mode == AlphaPolicy::Editable && text.len() == 7 {
            format!("{text}FF")
        } else {
            text.to_string()
        };
        let value = match SrgbColor::parse_hex(&text, mode) {
            DraftValue::Parsed(value) => Ok(value),
            DraftValue::Empty | DraftValue::Incomplete | DraftValue::Invalid(_) => {
                Err(ParseIssue::InvalidColor)
            }
        };
        project(value, mode)
    });
    service.on_color_component(|value, channel| {
        match channel {
            ui::ColorChannel::Red => value.red,
            ui::ColorChannel::Green => value.green,
            ui::ColorChannel::Blue => value.blue,
            ui::ColorChannel::Alpha => value.alpha,
        }
        .to_string()
        .into()
    });
    service.on_color_channel(|value, channel, text, mode| {
        let update = || -> Result<SrgbColor, ParseIssue> {
            let mut value = SrgbColor::try_from(value)?;
            let channel_value = text.parse::<u8>().map_err(|_| ParseIssue::InvalidColor)?;
            match channel {
                ui::ColorChannel::Red => value.red = channel_value,
                ui::ColorChannel::Green => value.green = channel_value,
                ui::ColorChannel::Blue => value.blue = channel_value,
                ui::ColorChannel::Alpha => value.alpha = channel_value,
            };
            Ok(value)
        };
        project(update(), policy(mode))
    });
}

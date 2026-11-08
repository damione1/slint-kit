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
    service.set_color_wheel_image(wheel_image());
    service.on_color_wheel_position(|value| {
        let Ok(value) = SrgbColor::try_from(value) else {
            return ui::ColorWheelPosition::default();
        };
        let hsv = slint::Color::from_rgb_u8(value.red, value.green, value.blue).to_hsva();
        let (hue, saturation) = (hsv.hue, hsv.saturation);
        let angle = hue.to_radians();
        ui::ColorWheelPosition {
            x: angle.sin() * saturation,
            y: -angle.cos() * saturation,
            hue,
            saturation,
        }
    });
    service.on_color_wheel(|x, y| {
        if !x.is_finite() || !y.is_finite() {
            return ui::ColorResult::default();
        }
        let saturation = x.hypot(y).min(1.0);
        let hue = x.atan2(-y).to_degrees().rem_euclid(360.0);
        wheel_color(hue, saturation)
    });
    service.on_color_hue(wheel_color);
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

// The wheel is the full-value hue/saturation plane; light intensity remains a
// separate host capability. This computes a color choice, never an observation.
fn wheel_color(hue: f32, saturation: f32) -> ui::ColorResult {
    if !hue.is_finite()
        || !saturation.is_finite()
        || !(0.0..=360.0).contains(&hue)
        || !(0.0..=1.0).contains(&saturation)
    {
        return ui::ColorResult::default();
    }
    let color = slint::Color::from_hsva(hue.rem_euclid(360.0), saturation, 1.0, 1.0);
    project(
        Ok(SrgbColor {
            red: color.red(),
            green: color.green(),
            blue: color.blue(),
            alpha: 255,
        }),
        AlphaPolicy::Opaque,
    )
}

#[cfg(test)]
mod wheel_tests {
    use super::*;

    #[test]
    fn spectrum_choices_are_opaque_and_invalid_coordinates_fail_closed() {
        for (hue, channels) in [
            (0.0, [255, 0, 0]),
            (120.0, [0, 255, 0]),
            (240.0, [0, 0, 255]),
            (360.0, [255, 0, 0]),
        ] {
            let result = wheel_color(hue, 1.0);
            assert!(result.valid);
            assert_eq!(
                [result.value.red, result.value.green, result.value.blue],
                channels
            );
            assert_eq!(result.value.alpha, 255);
        }
        let white = wheel_color(37.0, 0.0);
        assert_eq!(
            [white.value.red, white.value.green, white.value.blue],
            [255, 255, 255]
        );
        for (hue, saturation) in [
            (f32::NAN, 0.5),
            (f32::INFINITY, 0.5),
            (-1.0, 0.5),
            (361.0, 0.5),
            (1.0, f32::NAN),
            (1.0, -0.1),
            (1.0, 1.1),
        ] {
            assert!(!wheel_color(hue, saturation).valid);
        }
    }
}

// Render the spectrum once per window. Software gradients ignore rounded
// clipping in Slint 1.18.1; explicit RGBA coverage gives all backends the same
// circular plane, without a theme-dependent mask or installation asset.
fn wheel_image() -> slint::Image {
    const SIZE: u32 = 512;
    let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(SIZE, SIZE);
    for (index, pixel) in buffer.make_mut_slice().iter_mut().enumerate() {
        let x = ((index % SIZE as usize) as f32 + 0.5) / SIZE as f32 * 2.0 - 1.0;
        let y = ((index / SIZE as usize) as f32 + 0.5) / SIZE as f32 * 2.0 - 1.0;
        let radius = x.hypot(y);
        let hue = x.atan2(-y).to_degrees().rem_euclid(360.0);
        let alpha = ((1.0 - radius) * SIZE as f32 / 2.0 + 0.5).clamp(0.0, 1.0);
        let color = slint::Color::from_hsva(hue, radius.min(1.0), 1.0, alpha);
        *pixel = slint::Rgba8Pixel::new(color.red(), color.green(), color.blue(), color.alpha());
    }
    slint::Image::from_rgba8(buffer)
}

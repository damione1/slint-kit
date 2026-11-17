//! Explicit sRGB channels; partial hex remains an editing draft.
use crate::AlphaPolicy;
use crate::{DraftValue, ParseIssue};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SrgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
impl SrgbColor {
    pub fn parse_hex(text: &str, alpha: AlphaPolicy) -> DraftValue<Self> {
        if text.is_empty() {
            return DraftValue::Empty;
        }
        let text = text.strip_prefix('#').unwrap_or(text);
        let length = match alpha {
            AlphaPolicy::Opaque => 6,
            AlphaPolicy::Editable => 8,
        };
        if !text.bytes().all(|c| c.is_ascii_hexdigit()) || text.len() > length {
            return DraftValue::Invalid(ParseIssue::InvalidColor);
        }
        let shorthand = match alpha {
            AlphaPolicy::Opaque => text.len() == 3,
            AlphaPolicy::Editable => text.len() == 4,
        };
        if shorthand {
            let digits: Vec<u8> = text
                .bytes()
                .map(|value| (value as char).to_digit(16).expect("validated hex") as u8 * 17)
                .collect();
            return DraftValue::Parsed(Self {
                red: digits[0],
                green: digits[1],
                blue: digits[2],
                alpha: match alpha {
                    AlphaPolicy::Opaque => 255,
                    AlphaPolicy::Editable => digits[3],
                },
            });
        }
        if text.len() < length {
            return DraftValue::Incomplete;
        }
        let channel =
            |start| u8::from_str_radix(&text[start..start + 2], 16).expect("validated hex");
        DraftValue::Parsed(Self {
            red: channel(0),
            green: channel(2),
            blue: channel(4),
            alpha: match alpha {
                AlphaPolicy::Opaque => 255,
                AlphaPolicy::Editable => channel(6),
            },
        })
    }
    pub fn hex(self, alpha: AlphaPolicy) -> String {
        match alpha {
            AlphaPolicy::Opaque => format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue),
            AlphaPolicy::Editable => format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                self.red, self.green, self.blue, self.alpha
            ),
        }
    }
}
/// Hue in whole degrees, always normalised to `0..360`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hue(u16);
impl Hue {
    /// Wraps any whole-degree angle onto the circle; `360` is `0` and `-30` is `330`.
    pub fn new(degrees: i32) -> Self {
        Self(u16::try_from(degrees.rem_euclid(360)).expect("rem_euclid(360) fits u16"))
    }
    pub fn degrees(self) -> u16 {
        self.0
    }
}
/// A percentage `0..=100` for saturation, value or lightness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorPercent(u8);
impl ColorPercent {
    pub fn new(value: u8) -> Result<Self, ParseIssue> {
        if value <= 100 {
            Ok(Self(value))
        } else {
            Err(ParseIssue::InvalidColor)
        }
    }
    pub fn get(self) -> u8 {
        self.0
    }
}
/// Hue, saturation and value of an sRGB colour, rounded to whole degrees and percents.
/// The grid is coarser than 8-bit RGB, so a round trip can change a channel by a few steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hsv {
    pub hue: Hue,
    pub saturation: ColorPercent,
    pub value: ColorPercent,
}
/// Hue, saturation and lightness of an sRGB colour, rounded like [`Hsv`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hsl {
    pub hue: Hue,
    pub saturation: ColorPercent,
    pub lightness: ColorPercent,
}
/// Integer division rounded half away from zero; `denominator` is positive.
fn divide_round(numerator: i32, denominator: i32) -> i32 {
    let half = denominator / 2;
    if numerator >= 0 {
        (numerator + half) / denominator
    } else {
        (numerator - half) / denominator
    }
}
fn color_percent(numerator: i32, denominator: i32) -> ColorPercent {
    ColorPercent(
        u8::try_from(divide_round(numerator * 100, denominator).clamp(0, 100)).expect("clamped"),
    )
}
fn byte(numerator: i32, denominator: i32) -> u8 {
    u8::try_from(divide_round(numerator, denominator).clamp(0, 255)).expect("clamped")
}
impl SrgbColor {
    /// Shared by HSV and HSL: the hue circle position of the channels.
    fn hue(self) -> (Hue, i32, i32) {
        let (red, green, blue) = (
            i32::from(self.red),
            i32::from(self.green),
            i32::from(self.blue),
        );
        let (maximum, minimum) = (red.max(green).max(blue), red.min(green).min(blue));
        let delta = maximum - minimum;
        if delta == 0 {
            return (Hue::new(0), maximum, minimum);
        }
        let degrees = if maximum == red {
            divide_round((green - blue) * 60, delta)
        } else if maximum == green {
            120 + divide_round((blue - red) * 60, delta)
        } else {
            240 + divide_round((red - green) * 60, delta)
        };
        (Hue::new(degrees), maximum, minimum)
    }
    pub fn to_hsv(self) -> Hsv {
        let (hue, maximum, minimum) = self.hue();
        Hsv {
            hue,
            saturation: if maximum == 0 {
                ColorPercent(0)
            } else {
                color_percent(maximum - minimum, maximum)
            },
            value: color_percent(maximum, 255),
        }
    }
    pub fn to_hsl(self) -> Hsl {
        let (hue, maximum, minimum) = self.hue();
        let sum = maximum + minimum;
        let denominator = 255 - (sum - 255).abs();
        Hsl {
            hue,
            saturation: if maximum == minimum {
                ColorPercent(0)
            } else {
                color_percent(maximum - minimum, denominator)
            },
            lightness: color_percent(sum, 510),
        }
    }
    /// `chroma` and `offset` share the fixed-point scale `denominator`.
    fn from_sector(hue: Hue, chroma: i32, offset: i32, denominator: i32, alpha: u8) -> Self {
        let sector = hue.0 / 60;
        let remainder = i32::from(hue.0 % 60);
        // Chroma scaled by 60, and the middle channel scaled likewise.
        let first = chroma * 60;
        let middle = if sector.is_multiple_of(2) {
            chroma * remainder
        } else {
            chroma * (60 - remainder)
        };
        let (red, green, blue) = match sector {
            0 => (first, middle, 0),
            1 => (middle, first, 0),
            2 => (0, first, middle),
            3 => (0, middle, first),
            4 => (middle, 0, first),
            _ => (first, 0, middle),
        };
        let offset = offset * 60;
        Self {
            red: byte(red + offset, denominator * 60),
            green: byte(green + offset, denominator * 60),
            blue: byte(blue + offset, denominator * 60),
            alpha,
        }
    }
    pub fn from_hsv(hsv: Hsv, alpha: u8) -> Self {
        let (saturation, value) = (i32::from(hsv.saturation.0), i32::from(hsv.value.0));
        // Scale 100 * 100: chroma and offset in 0..=255 * 10000.
        let chroma = value * saturation * 255;
        let offset = value * (100 - saturation) * 255;
        Self::from_sector(hsv.hue, chroma, offset, 10_000, alpha)
    }
    pub fn from_hsl(hsl: Hsl, alpha: u8) -> Self {
        let (saturation, lightness) = (i32::from(hsl.saturation.0), i32::from(hsl.lightness.0));
        // Scale 100 * 100 * 2 so that the half chroma offset stays an integer.
        let chroma = (100 - (2 * lightness - 100).abs()) * saturation * 255 * 2;
        let offset = lightness * 255 * 100 * 2 - chroma / 2;
        Self::from_sector(hsl.hue, chroma, offset, 10_000 * 2, alpha)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn rgb(red: u8, green: u8, blue: u8) -> SrgbColor {
        SrgbColor {
            red,
            green,
            blue,
            alpha: 255,
        }
    }
    fn hsv(hue: i32, saturation: u8, value: u8) -> Hsv {
        Hsv {
            hue: Hue::new(hue),
            saturation: ColorPercent::new(saturation).unwrap(),
            value: ColorPercent::new(value).unwrap(),
        }
    }
    fn hsl(hue: i32, saturation: u8, lightness: u8) -> Hsl {
        Hsl {
            hue: Hue::new(hue),
            saturation: ColorPercent::new(saturation).unwrap(),
            lightness: ColorPercent::new(lightness).unwrap(),
        }
    }
    #[test]
    fn hue_wraps_and_percent_rejects_out_of_range() {
        assert_eq!(Hue::new(360).degrees(), 0);
        assert_eq!(Hue::new(-30).degrees(), 330);
        assert_eq!(ColorPercent::new(101), Err(ParseIssue::InvalidColor));
    }
    #[test]
    fn primaries_and_grays_convert_exactly() {
        for (color, expected_hsv, expected_hsl) in [
            (rgb(255, 0, 0), hsv(0, 100, 100), hsl(0, 100, 50)),
            (rgb(255, 255, 0), hsv(60, 100, 100), hsl(60, 100, 50)),
            (rgb(0, 255, 0), hsv(120, 100, 100), hsl(120, 100, 50)),
            (rgb(0, 255, 255), hsv(180, 100, 100), hsl(180, 100, 50)),
            (rgb(0, 0, 255), hsv(240, 100, 100), hsl(240, 100, 50)),
            (rgb(255, 0, 255), hsv(300, 100, 100), hsl(300, 100, 50)),
            (rgb(255, 255, 255), hsv(0, 0, 100), hsl(0, 0, 100)),
            (rgb(0, 0, 0), hsv(0, 0, 0), hsl(0, 0, 0)),
        ] {
            assert_eq!(color.to_hsv(), expected_hsv, "{color:?}");
            assert_eq!(color.to_hsl(), expected_hsl, "{color:?}");
            assert_eq!(SrgbColor::from_hsv(expected_hsv, 255), color);
            assert_eq!(SrgbColor::from_hsl(expected_hsl, 255), color);
        }
    }
    #[test]
    fn documented_rounding_and_alpha_are_kept() {
        let color = SrgbColor {
            red: 18,
            green: 34,
            blue: 56,
            alpha: 77,
        };
        assert_eq!(color.to_hsv(), hsv(215, 68, 22));
        assert_eq!(color.to_hsl(), hsl(215, 51, 15));
        assert_eq!(SrgbColor::from_hsv(color.to_hsv(), 77).alpha, 77);
        assert_eq!(SrgbColor::from_hsl(color.to_hsl(), 77).alpha, 77);
    }
    #[test]
    fn conversion_round_trip_stays_within_the_grid_error() {
        // A step of one percent or degree moves a channel by at most three bytes.
        for red in (0..=255u8).step_by(15) {
            for green in (0..=255u8).step_by(15) {
                for blue in (0..=255u8).step_by(15) {
                    let color = rgb(red, green, blue);
                    for back in [
                        SrgbColor::from_hsv(color.to_hsv(), 255),
                        SrgbColor::from_hsl(color.to_hsl(), 255),
                    ] {
                        for (a, b) in [
                            (color.red, back.red),
                            (color.green, back.green),
                            (color.blue, back.blue),
                        ] {
                            assert!(a.abs_diff(b) <= 4, "{color:?} -> {back:?}");
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn partial_and_explicit_alpha() {
        assert_eq!(
            SrgbColor::parse_hex("#FF", AlphaPolicy::Opaque),
            DraftValue::Incomplete
        );
        assert_eq!(
            SrgbColor::parse_hex("#ABC", AlphaPolicy::Opaque),
            DraftValue::Parsed(SrgbColor {
                red: 170,
                green: 187,
                blue: 204,
                alpha: 255
            })
        );
        assert_eq!(
            SrgbColor::parse_hex("#GG", AlphaPolicy::Opaque),
            DraftValue::Invalid(ParseIssue::InvalidColor)
        );
        let color = SrgbColor {
            red: 18,
            green: 34,
            blue: 56,
            alpha: 128,
        };
        assert_eq!(
            SrgbColor::parse_hex(&color.hex(AlphaPolicy::Editable), AlphaPolicy::Editable),
            DraftValue::Parsed(color)
        );
    }
}

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
#[cfg(test)]
mod tests {
    use super::*;
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

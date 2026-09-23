//! Colour primitives. Every colour in the palette is authored as a 24-bit hex
//! literal; every other representation is derived from it.

use serde::Serialize;
use std::fmt;

/// A 24-bit sRGB colour, as authored in the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hex(u32);

/// 8-bit sRGB channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Hue in degrees, saturation and lightness as fractions of 1.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Hsl {
    pub h: f64,
    pub s: f64,
    pub l: f64,
}

impl Hex {
    /// Wrap a `0xRRGGBB` literal.
    pub const fn new(value: u32) -> Self {
        Self(value & 0x00ff_ffff)
    }

    /// Parse `#rrggbb`, `rrggbb`, `#rgb` or `rgb`.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().trim_start_matches('#');
        let expanded = match s.len() {
            3 => s.chars().flat_map(|c| [c, c]).collect(),
            6 => s.to_owned(),
            _ => return None,
        };
        u32::from_str_radix(&expanded, 16).ok().map(Self::new)
    }

    pub const fn rgb(self) -> Rgb {
        Rgb {
            r: ((self.0 >> 16) & 0xff) as u8,
            g: ((self.0 >> 8) & 0xff) as u8,
            b: (self.0 & 0xff) as u8,
        }
    }

    pub fn hsl(self) -> Hsl {
        let Rgb { r, g, b } = self.rgb();
        let (r, g, b) = (
            f64::from(r) / 255.0,
            f64::from(g) / 255.0,
            f64::from(b) / 255.0,
        );
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        let delta = max - min;

        if delta == 0.0 {
            return Hsl { h: 0.0, s: 0.0, l };
        }

        let s = if l > 0.5 {
            delta / (2.0 - max - min)
        } else {
            delta / (max + min)
        };
        let h = if max == r {
            ((g - b) / delta).rem_euclid(6.0)
        } else if max == g {
            (b - r) / delta + 2.0
        } else {
            (r - g) / delta + 4.0
        };

        Hsl { h: h * 60.0, s, l }
    }

    /// Relative luminance per WCAG 2.1.
    pub fn luminance(self) -> f64 {
        fn channel(c: u8) -> f64 {
            let c = f64::from(c) / 255.0;
            if c <= 0.040_45 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        let Rgb { r, g, b } = self.rgb();
        0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
    }

    /// WCAG 2.1 contrast ratio against `other`, from 1.0 to 21.0.
    pub fn contrast(self, other: Self) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        let (hi, lo) = if a > b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// `#rrggbb`, always lowercase.
    pub fn to_hex_string(self) -> String {
        format!("#{:06x}", self.0)
    }

    /// `rrggbb`, for ports that supply their own prefix.
    pub fn to_bare_string(self) -> String {
        format!("{:06x}", self.0)
    }
}

impl From<Hsl> for Hex {
    fn from(Hsl { h, s, l }: Hsl) -> Self {
        if s <= 0.0 {
            let v = (l.clamp(0.0, 1.0) * 255.0).round() as u32;
            return Self::new((v << 16) | (v << 8) | v);
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        let h = h.rem_euclid(360.0) / 360.0;

        let component = |mut t: f64| {
            t = t.rem_euclid(1.0);
            let v = if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 0.5 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            };
            (v.clamp(0.0, 1.0) * 255.0).round() as u32
        };

        Self::new((component(h + 1.0 / 3.0) << 16) | (component(h) << 8) | component(h - 1.0 / 3.0))
    }
}

impl fmt::Display for Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex_string())
    }
}

impl Serialize for Hex {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_widths() {
        assert_eq!(Hex::parse("#fff"), Some(Hex::new(0xffffff)));
        assert_eq!(Hex::parse("de5a48"), Some(Hex::new(0xde5a48)));
        assert_eq!(Hex::parse("nope"), None);
    }

    #[test]
    fn hsl_round_trips() {
        for hex in [0x000000, 0xffffff, 0xde5a48, 0xc9e02b, 0x45b6ff, 0x1c1b19] {
            let hex = Hex::new(hex);
            assert_eq!(Hex::from(hex.hsl()), hex, "{hex} did not round-trip");
        }
    }

    #[test]
    fn contrast_is_symmetric_and_bounded() {
        let (black, white) = (Hex::new(0x000000), Hex::new(0xffffff));
        assert!((black.contrast(white) - 21.0).abs() < 0.01);
        assert!((white.contrast(black) - 21.0).abs() < 0.01);
        assert!((black.contrast(black) - 1.0).abs() < f64::EPSILON);
    }
}

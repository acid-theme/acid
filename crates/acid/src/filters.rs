//! Colour filters available to every template.
//!
//! A filter takes the colour piped into it and returns either another colour,
//! which can be piped onwards, or the string a theme file wants. `amount`
//! arguments are fractions of 1.

// Every filter is part of the templates' vocabulary whether a port uses it or not.
#![allow(dead_code)]

use acid_palette::{Hex, Hsl};

/// `#rrggbb`, which is also what a colour renders as on its own.
#[askama::filter_fn]
pub fn hex(value: &Hex, _: &dyn askama::Values) -> askama::Result<String> {
    Ok(value.to_hex_string())
}

/// The logic behind the filters, kept separate so it can be tested: the macro
/// turns each filter into a struct rather than a callable function.
pub fn with_alpha(value: &Hex, amount: f64, prefixed: bool) -> String {
    let body = if prefixed {
        value.to_hex_string()
    } else {
        value.to_bare_string()
    };
    format!("{body}{:02x}", byte(amount))
}

pub fn blend(value: &Hex, other: &Hex, amount: f64) -> Hex {
    let amount = amount.clamp(0.0, 1.0);
    let (from, to) = (value.rgb(), other.rgb());
    let channel = |a: u8, b: u8| {
        (f64::from(a) * (1.0 - amount) + f64::from(b) * amount)
            .round()
            .clamp(0.0, 255.0) as u32
    };
    Hex::new((channel(from.r, to.r) << 16) | (channel(from.g, to.g) << 8) | channel(from.b, to.b))
}

pub fn rgb_string(value: &Hex) -> String {
    let rgb = value.rgb();
    format!("rgb({}, {}, {})", rgb.r, rgb.g, rgb.b)
}

pub fn hsl_string(value: &Hex) -> String {
    let hsl = value.hsl();
    format!(
        "hsl({:.0}, {:.0}%, {:.0}%)",
        hsl.h,
        hsl.s * 100.0,
        hsl.l * 100.0
    )
}

pub fn ratio(value: &Hex, other: &Hex) -> String {
    format!("{:.2}", value.contrast(*other))
}

/// `rrggbb`, for the formats that supply their own prefix.
#[askama::filter_fn]
pub fn bare(value: &Hex, _: &dyn askama::Values) -> askama::Result<String> {
    Ok(value.to_bare_string())
}

/// `#rrggbbaa`.
#[askama::filter_fn]
pub fn alpha(value: &Hex, _: &dyn askama::Values, amount: f64) -> askama::Result<String> {
    Ok(with_alpha(value, amount, true))
}

/// `rrggbbaa`, which is what swaylock reads.
#[askama::filter_fn]
pub fn alpha_bare(value: &Hex, _: &dyn askama::Values, amount: f64) -> askama::Result<String> {
    Ok(with_alpha(value, amount, false))
}

#[askama::filter_fn]
pub fn css_rgb(value: &Hex, _: &dyn askama::Values) -> askama::Result<String> {
    Ok(rgb_string(value))
}

#[askama::filter_fn]
pub fn css_hsl(value: &Hex, _: &dyn askama::Values) -> askama::Result<String> {
    Ok(hsl_string(value))
}

#[askama::filter_fn]
pub fn lighten(value: &Hex, _: &dyn askama::Values, amount: f64) -> askama::Result<Hex> {
    Ok(shift_lightness(value, amount))
}

#[askama::filter_fn]
pub fn darken(value: &Hex, _: &dyn askama::Values, amount: f64) -> askama::Result<Hex> {
    Ok(shift_lightness(value, -amount))
}

#[askama::filter_fn]
pub fn saturate(value: &Hex, _: &dyn askama::Values, amount: f64) -> askama::Result<Hex> {
    Ok(shift_saturation(value, amount))
}

#[askama::filter_fn]
pub fn desaturate(value: &Hex, _: &dyn askama::Values, amount: f64) -> askama::Result<Hex> {
    Ok(shift_saturation(value, -amount))
}

/// Blend towards another colour: `{{ red | mix(flavor.colors.base, 0.3) }}`.
#[askama::filter_fn]
pub fn mix(value: &Hex, _: &dyn askama::Values, other: &Hex, amount: f64) -> askama::Result<Hex> {
    Ok(blend(value, other, amount))
}

/// The WCAG 2.1 contrast ratio against another colour, to two places.
#[askama::filter_fn]
pub fn contrast(value: &Hex, _: &dyn askama::Values, other: &Hex) -> askama::Result<String> {
    Ok(ratio(value, other))
}

fn byte(amount: f64) -> u8 {
    (amount.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub fn shift_lightness(value: &Hex, by: f64) -> Hex {
    let Hsl { h, s, l } = value.hsl();
    Hex::from(Hsl {
        h,
        s,
        l: (l + by).clamp(0.0, 1.0),
    })
}

pub fn shift_saturation(value: &Hex, by: f64) -> Hex {
    let Hsl { h, s, l } = value.hsl();
    Hex::from(Hsl {
        h,
        s: (s + by).clamp(0.0, 1.0),
        l,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK: Hex = Hex::new(0x000000);
    const WHITE: Hex = Hex::new(0xffffff);
    const RED: Hex = Hex::new(0xff4536);

    #[test]
    fn alpha_appends_a_byte() {
        assert_eq!(with_alpha(&RED, 1.0, true), "#ff4536ff");
        assert_eq!(with_alpha(&RED, 0.0, true), "#ff453600");
        assert_eq!(with_alpha(&RED, 0.5, false), "ff453680");
    }

    #[test]
    fn mixing_meets_in_the_middle() {
        assert_eq!(blend(&BLACK, &WHITE, 0.5), Hex::new(0x808080));
        assert_eq!(blend(&RED, &WHITE, 0.0), RED);
        assert_eq!(blend(&RED, &WHITE, 1.0), WHITE);
    }

    #[test]
    fn lightness_moves_in_both_directions_and_clamps() {
        assert_eq!(shift_lightness(&BLACK, 1.0), WHITE);
        assert_eq!(shift_lightness(&WHITE, -1.0), BLACK);
        // Already at the limit, so it stays there rather than wrapping.
        assert_eq!(shift_lightness(&WHITE, 0.5), WHITE);
    }

    #[test]
    fn contrast_matches_wcag() {
        assert_eq!(ratio(&BLACK, &WHITE), "21.00");
        assert_eq!(ratio(&WHITE, &WHITE), "1.00");
    }

    #[test]
    fn css_forms() {
        assert_eq!(rgb_string(&RED), "rgb(255, 69, 54)");
        assert_eq!(hsl_string(&RED), "hsl(4, 100%, 61%)");
    }
}

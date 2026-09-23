//! Tera filters for manipulating colours inside templates.
//!
//! Every filter accepts either a hex string (`"#ff4536"`) or a palette colour
//! object (whose `hex` field is used), so `{{ flavor.colors.red | darken }}` and
//! `{{ "#ff4536" | darken }}` both work. Colour-returning filters hand back a
//! `#rrggbb` string, which can be piped onwards.

use std::collections::HashMap;

use acid_palette::{Hex, Hsl};
use tera::{Error, Result, Tera, Value};

/// Pull a colour out of whatever the template piped in.
fn as_hex(value: &Value, filter: &str) -> Result<Hex> {
    let raw = match value {
        Value::String(s) => s.clone(),
        Value::Object(map) => match map.get("hex") {
            Some(Value::String(s)) => s.clone(),
            _ => {
                return Err(Error::msg(format!(
                    "{filter}: object has no string `hex` field"
                )));
            }
        },
        other => {
            return Err(Error::msg(format!(
                "{filter}: expected a colour, got {other}"
            )));
        }
    };

    Hex::parse(&raw).ok_or_else(|| Error::msg(format!("{filter}: `{raw}` is not a hex colour")))
}

/// Read a fraction argument, accepting `0.2` or `20` (per cent) alike.
fn amount(args: &HashMap<String, Value>, filter: &str, default: f64) -> Result<f64> {
    let Some(value) = args.get("amount") else {
        return Ok(default);
    };
    let number = value
        .as_f64()
        .ok_or_else(|| Error::msg(format!("{filter}: `amount` must be a number")))?;
    Ok(if number > 1.0 { number / 100.0 } else { number })
}

fn shift_lightness(
    value: &Value,
    args: &HashMap<String, Value>,
    filter: &str,
    sign: f64,
) -> Result<Value> {
    let hex = as_hex(value, filter)?;
    let by = amount(args, filter, 0.1)?;
    let Hsl { h, s, l } = hex.hsl();
    let shifted = Hsl {
        h,
        s,
        l: (l + sign * by).clamp(0.0, 1.0),
    };
    Ok(Value::String(Hex::from(shifted).to_hex_string()))
}

fn shift_saturation(
    value: &Value,
    args: &HashMap<String, Value>,
    filter: &str,
    sign: f64,
) -> Result<Value> {
    let hex = as_hex(value, filter)?;
    let by = amount(args, filter, 0.1)?;
    let Hsl { h, s, l } = hex.hsl();
    let shifted = Hsl {
        h,
        s: (s + sign * by).clamp(0.0, 1.0),
        l,
    };
    Ok(Value::String(Hex::from(shifted).to_hex_string()))
}

/// Register every colour filter on a Tera instance.
pub fn register(tera: &mut Tera) {
    tera.register_filter("hex", |value: &Value, _: &HashMap<String, Value>| {
        Ok(Value::String(as_hex(value, "hex")?.to_hex_string()))
    });

    // `rrggbb` with no leading `#`, for ports that add their own prefix.
    tera.register_filter("bare", |value: &Value, _: &HashMap<String, Value>| {
        Ok(Value::String(as_hex(value, "bare")?.to_bare_string()))
    });

    // WCAG 2.1 contrast ratio, e.g. `{{ flavor.colors.red | contrast(with=flavor.colors.base) }}`.
    tera.register_filter(
        "contrast",
        |value: &Value, args: &HashMap<String, Value>| {
            let hex = as_hex(value, "contrast")?;
            let with = args
                .get("with")
                .ok_or_else(|| Error::msg("contrast: `with` is required"))?;
            let ratio = hex.contrast(as_hex(with, "contrast")?);
            Ok(Value::from((ratio * 100.0).round() / 100.0))
        },
    );

    tera.register_filter("css_rgb", |value: &Value, _: &HashMap<String, Value>| {
        let rgb = as_hex(value, "css_rgb")?.rgb();
        Ok(Value::String(format!(
            "rgb({}, {}, {})",
            rgb.r, rgb.g, rgb.b
        )))
    });

    tera.register_filter("css_hsl", |value: &Value, _: &HashMap<String, Value>| {
        let hsl = as_hex(value, "css_hsl")?.hsl();
        Ok(Value::String(format!(
            "hsl({:.0}, {:.0}%, {:.0}%)",
            hsl.h,
            hsl.s * 100.0,
            hsl.l * 100.0
        )))
    });

    // `#rrggbbaa`, for ports that take eight-digit hex.
    tera.register_filter("alpha", |value: &Value, args: &HashMap<String, Value>| {
        let hex = as_hex(value, "alpha")?;
        let opacity = amount(args, "alpha", 1.0)?.clamp(0.0, 1.0);
        Ok(Value::String(format!(
            "{}{:02x}",
            hex.to_hex_string(),
            (opacity * 255.0).round() as u8
        )))
    });

    tera.register_filter("lighten", |value: &Value, args: &HashMap<String, Value>| {
        shift_lightness(value, args, "lighten", 1.0)
    });
    tera.register_filter("darken", |value: &Value, args: &HashMap<String, Value>| {
        shift_lightness(value, args, "darken", -1.0)
    });
    tera.register_filter(
        "saturate",
        |value: &Value, args: &HashMap<String, Value>| {
            shift_saturation(value, args, "saturate", 1.0)
        },
    );
    tera.register_filter(
        "desaturate",
        |value: &Value, args: &HashMap<String, Value>| {
            shift_saturation(value, args, "desaturate", -1.0)
        },
    );

    // Blend towards another colour: `{{ red | mix(with=base, amount=0.3) }}`.
    tera.register_filter("mix", |value: &Value, args: &HashMap<String, Value>| {
        let from = as_hex(value, "mix")?.rgb();
        let with = args
            .get("with")
            .ok_or_else(|| Error::msg("mix: `with` is required"))?;
        let to = as_hex(with, "mix")?.rgb();
        let by = amount(args, "mix", 0.5)?.clamp(0.0, 1.0);

        let blend = |a: u8, b: u8| {
            (f64::from(a) * (1.0 - by) + f64::from(b) * by)
                .round()
                .clamp(0.0, 255.0) as u32
        };
        let mixed = (blend(from.r, to.r) << 16) | (blend(from.g, to.g) << 8) | blend(from.b, to.b);
        Ok(Value::String(Hex::new(mixed).to_hex_string()))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera::Context;

    fn render(template: &str) -> String {
        let mut tera = Tera::default();
        register(&mut tera);
        tera.render_str(template, &Context::new()).unwrap()
    }

    #[test]
    fn normalises_and_strips() {
        assert_eq!(render(r##"{{ "#FFF" | hex }}"##), "#ffffff");
        assert_eq!(render(r##"{{ "#de5a48" | bare }}"##), "de5a48");
    }

    #[test]
    fn lightness_moves_in_both_directions() {
        assert_eq!(
            render(r##"{{ "#808080" | darken(amount=0.5) }}"##),
            "#000000"
        );
        assert_eq!(
            render(r##"{{ "#808080" | lighten(amount=0.5) }}"##),
            "#ffffff"
        );
        // Per cent and fraction are equivalent.
        assert_eq!(
            render(r##"{{ "#808080" | darken(amount=20) }}"##),
            render(r##"{{ "#808080" | darken(amount=0.2) }}"##),
        );
    }

    #[test]
    fn mix_meets_in_the_middle() {
        assert_eq!(
            render(r##"{{ "#000000" | mix(with="#ffffff", amount=0.5) }}"##),
            "#808080"
        );
        assert_eq!(
            render(r##"{{ "#ff4536" | mix(with="#ffffff", amount=0) }}"##),
            "#ff4536"
        );
    }

    #[test]
    fn alpha_appends_a_byte() {
        assert_eq!(
            render(r##"{{ "#1c1b19" | alpha(amount=0.5) }}"##),
            "#1c1b1980"
        );
        assert_eq!(
            render(r##"{{ "#1c1b19" | alpha(amount=1) }}"##),
            "#1c1b19ff"
        );
    }

    #[test]
    fn css_helpers() {
        assert_eq!(render(r##"{{ "#ff4536" | css_rgb }}"##), "rgb(255, 69, 54)");
        assert_eq!(
            render(r##"{{ "#ff4536" | css_hsl }}"##),
            "hsl(4, 100%, 61%)"
        );
    }

    #[test]
    fn contrast_matches_wcag() {
        assert_eq!(
            render(r##"{{ "#000000" | contrast(with="#ffffff") }}"##),
            "21"
        );
        assert_eq!(
            render(r##"{{ "#de5a48" | contrast(with="#1c1b19") }}"##),
            "4.64"
        );
    }

    #[test]
    fn rejects_nonsense() {
        let mut tera = Tera::default();
        register(&mut tera);
        assert!(
            tera.render_str(r##"{{ "hello" | darken }}"##, &Context::new())
                .is_err()
        );
    }
}

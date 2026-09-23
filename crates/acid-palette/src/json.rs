//! The canonical JSON shape of the palette.
//!
//! Shared by the `codegen` binary, which writes `palette.json`, and by
//! `acidify`, which builds template contexts — so the two can never drift.

use serde_json::{Map, Value, json};

use crate::{Color, FLAVORS, Flavor};

/// Keeps the JSON stable across runs; raw f64 hue and saturation would churn
/// the diff on unrelated changes.
fn round(value: f64, places: i32) -> f64 {
    let factor = 10f64.powi(places);
    (value * factor).round() / factor
}

pub fn color(color: &Color) -> Value {
    let rgb = color.rgb();
    let hsl = color.hsl();
    json!({
        "name": color.name,
        "identifier": color.identifier,
        "order": color.order,
        "accent": color.accent,
        "hex": color.hex,
        "bare": color.hex.to_bare_string(),
        "rgb": { "r": rgb.r, "g": rgb.g, "b": rgb.b },
        "hsl": {
            "h": round(hsl.h, 2),
            "s": round(hsl.s, 4),
            "l": round(hsl.l, 4),
        },
    })
}

pub fn flavor(flavor: &Flavor) -> Value {
    let mut colors = Map::new();
    let mut accents = Vec::new();
    for entry in flavor.colors.iter() {
        let value = color(&entry);
        if entry.accent {
            accents.push(value.clone());
        }
        colors.insert(entry.identifier.to_owned(), value);
    }

    json!({
        "name": flavor.name,
        "identifier": flavor.identifier,
        "order": flavor.order,
        "dark": flavor.dark,
        "invertedDepth": flavor.inverted_depth,
        "colors": Value::Object(colors),
        "accents": Value::Array(accents),
    })
}

/// Every flavour, keyed by identifier and in canonical order.
pub fn palette() -> Value {
    let mut flavors = Map::new();
    for entry in &FLAVORS {
        flavors.insert(entry.identifier.to_owned(), flavor(entry));
    }
    Value::Object(flavors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_flavours_and_colours_in_order() {
        let palette = palette();
        let flavors: Vec<_> = palette.as_object().unwrap().keys().collect();
        assert_eq!(flavors, ["acetic", "citric"]);

        let colors = palette["acetic"]["colors"].as_object().unwrap();
        let first: Vec<_> = colors.keys().take(3).collect();
        assert_eq!(first, ["red", "orange", "yellow"]);
        assert_eq!(colors.keys().next_back().unwrap(), "crust");
    }

    #[test]
    fn exposes_derived_representations() {
        let red = &palette()["acetic"]["colors"]["red"];
        assert_eq!(red["hex"], "#ff4536");
        assert_eq!(red["bare"], "ff4536");
        assert_eq!(red["rgb"]["r"], 255);
        assert_eq!(red["accent"], true);
        assert_eq!(palette()["acetic"]["accents"].as_array().unwrap().len(), 7);
    }
}

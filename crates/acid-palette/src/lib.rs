//! Acid: a very dark colourscheme in two flavours.
//!
//! This crate is the single source of truth for both flavours. Everything
//! downstream — `palette.json`, every port, every preview — is derived from the
//! constants in this file, so a colour is only ever edited here.

mod color;
pub mod json;

use serde::Serialize;

pub use color::{Hex, Hsl, Rgb};

/// The palette's version, and so the colourscheme's. Set in the workspace
/// manifest; every generated file is stamped with it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// One colour, with its palette metadata attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Color {
    /// Machine name, e.g. `surface0`.
    pub identifier: &'static str,
    /// Display name, e.g. `Surface 0`.
    pub name: &'static str,
    /// Position in the canonical palette order.
    pub order: u32,
    /// Whether the colour is a syntax accent rather than a surface or text tone.
    pub accent: bool,
    pub hex: Hex,
}

impl Color {
    pub fn rgb(&self) -> Rgb {
        self.hex.rgb()
    }

    pub fn hsl(&self) -> Hsl {
        self.hex.hsl()
    }
}

/// Declares the colour roles, their display names and accent flags once, and
/// derives the ordered iteration that every consumer walks.
macro_rules! declare_colors {
    ($($identifier:ident => $name:literal, $accent:literal;)+) => {
        /// The colour roles a flavour must define, in canonical order.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct FlavorColors {
            $(pub $identifier: Hex,)+
        }

        impl FlavorColors {
            /// Every colour in canonical order: accents, then text, then surfaces.
            pub fn iter(&self) -> impl Iterator<Item = Color> + '_ {
                [$((stringify!($identifier), $name, $accent, self.$identifier),)+]
                    .into_iter()
                    .enumerate()
                    .map(|(order, (identifier, name, accent, hex))| Color {
                        identifier,
                        name,
                        order: order as u32,
                        accent,
                        hex,
                    })
            }

            /// Look a colour up by its identifier, e.g. `"yellow"`.
            pub fn get(&self, identifier: &str) -> Option<Color> {
                self.iter().find(|color| color.identifier == identifier)
            }
        }
    };
}

declare_colors! {
    // Accents. Seven roles, named after gruvbox's so the mapping is obvious.
    red    => "Red",    true;
    orange => "Orange", true;
    yellow => "Yellow", true;
    green  => "Green",  true;
    aqua   => "Aqua",   true;
    blue   => "Blue",   true;
    purple => "Purple", true;

    // Text, brightest first.
    text     => "Text",      false;
    subtext1 => "Subtext 1", false;
    subtext0 => "Subtext 0", false;

    // Dimmed text: comments, line numbers, inactive UI.
    overlay2 => "Overlay 2", false;
    overlay1 => "Overlay 1", false;
    overlay0 => "Overlay 0", false;

    // Raised surfaces: floats, selections, cursor line.
    surface2 => "Surface 2", false;
    surface1 => "Surface 1", false;
    surface0 => "Surface 0", false;

    // The editor field and the chrome around it.
    base   => "Base",   false;
    mantle => "Mantle", false;
    crust  => "Crust",  false;
}

/// One variant of the colourscheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flavor {
    /// Machine name, e.g. `acetic`.
    pub identifier: &'static str,
    /// Display name, e.g. `Acetic`.
    pub name: &'static str,
    /// Position in the canonical flavour order.
    pub order: u32,
    /// Always true today; both flavours are dark.
    pub dark: bool,
    /// Whether `mantle` and `crust` are *lighter* than `base` rather than
    /// darker. Acetic sets this: its `base` is pure black, so there is nothing
    /// below it and the depth axis runs upwards instead. Ports that draw chrome
    /// behind the editor should branch on this rather than assume an ordering.
    pub inverted_depth: bool,
    pub colors: FlavorColors,
}

impl Flavor {
    pub fn color(&self, identifier: &str) -> Option<Color> {
        self.colors.get(identifier)
    }

    /// The accents only, in canonical order.
    pub fn accents(&self) -> impl Iterator<Item = Color> + '_ {
        self.colors.iter().filter(|color| color.accent)
    }
}

/// Pure black, vibrant accents, neutral greys. Built for OLED panels and for
/// anyone who wants the syntax to be the only thing on screen.
pub const ACETIC: Flavor = Flavor {
    identifier: "acetic",
    name: "Acetic",
    order: 0,
    dark: true,
    inverted_depth: true,
    colors: FlavorColors {
        red: Hex::new(0xff4536),
        orange: Hex::new(0xff8b1f),
        yellow: Hex::new(0xffc832),
        green: Hex::new(0xc9e02b),
        aqua: Hex::new(0x2fe0b8),
        blue: Hex::new(0x45b6ff),
        purple: Hex::new(0xea7fc4),

        text: Hex::new(0xf2e9d0),
        subtext1: Hex::new(0xc3bba9),
        subtext0: Hex::new(0xa49f95),

        overlay2: Hex::new(0x898989),
        overlay1: Hex::new(0x707070),
        overlay0: Hex::new(0x5a5a5a),

        surface2: Hex::new(0x4a4a4a),
        surface1: Hex::new(0x383838),
        surface0: Hex::new(0x242424),

        base: Hex::new(0x000000),
        mantle: Hex::new(0x0f0f0f),
        crust: Hex::new(0x1c1c1c),
    },
};

/// Warm dark grey, muted accents. The everyday flavour, and the one that sits
/// closest to gruvbox.
pub const CITRIC: Flavor = Flavor {
    identifier: "citric",
    name: "Citric",
    order: 1,
    dark: true,
    inverted_depth: false,
    colors: FlavorColors {
        red: Hex::new(0xde5a48),
        orange: Hex::new(0xdd7b34),
        yellow: Hex::new(0xd9a441),
        green: Hex::new(0xa3ab2f),
        aqua: Hex::new(0x7cad76),
        blue: Hex::new(0x7d9cba),
        purple: Hex::new(0xc5839c),

        text: Hex::new(0xe2d6b6),
        subtext1: Hex::new(0xcabea7),
        subtext0: Hex::new(0xb4ab9b),

        overlay2: Hex::new(0x9f988a),
        overlay1: Hex::new(0x857d70),
        overlay0: Hex::new(0x6b665c),

        surface2: Hex::new(0x5b554a),
        surface1: Hex::new(0x49443b),
        surface0: Hex::new(0x35332c),

        base: Hex::new(0x1c1b19),
        mantle: Hex::new(0x121211),
        crust: Hex::new(0x060605),
    },
};

/// Every flavour, in canonical order.
pub const FLAVORS: [Flavor; 2] = [ACETIC, CITRIC];

/// Look a flavour up by its identifier, e.g. `"citric"`.
pub fn flavor(identifier: &str) -> Option<Flavor> {
    FLAVORS
        .into_iter()
        .find(|flavor| flavor.identifier == identifier)
}

/// The contrast ratio every accent and text tone must clear against its own
/// `base`. WCAG 2.1 AA for body text.
pub const MIN_CONTRAST: f64 = 4.5;

/// The contrast ratio between neighbouring fills. Below this a cursor line or a
/// selection is the same colour as the buffer behind it.
pub const MIN_FILL_STEP: f64 = 1.3;

/// The contrast ratio dimmed text must clear against the fill it sits on — a
/// comment on the cursor line being the case that matters.
pub const MIN_DIM_TEXT: f64 = 3.0;

/// The contrast ratio between `base`, `mantle` and `crust`. Chrome is meant to
/// be subtle — Catppuccin and gruvbox both sit near 1.1 — so this is a floor
/// that stops the layers collapsing into one, not a target.
pub const MIN_CHROME_STEP: f64 = 1.07;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_are_unique() {
        for flavor in FLAVORS {
            let mut seen = Vec::new();
            for color in flavor.colors.iter() {
                assert!(
                    !seen.contains(&color.identifier),
                    "{} repeats {}",
                    flavor.identifier,
                    color.identifier
                );
                seen.push(color.identifier);
            }
            assert_eq!(seen.len(), 19);
        }
    }

    #[test]
    fn every_flavor_has_seven_accents() {
        for flavor in FLAVORS {
            assert_eq!(flavor.accents().count(), 7, "{}", flavor.identifier);
        }
    }

    /// Accents and text have to be readable on the flavour's own background.
    #[test]
    fn readable_on_base() {
        for flavor in FLAVORS {
            let base = flavor.colors.base;
            for color in flavor.colors.iter() {
                if !color.accent
                    && !color.identifier.starts_with("text")
                    && !color.identifier.starts_with("subtext")
                {
                    continue;
                }
                let ratio = color.hex.contrast(base);
                assert!(
                    ratio >= MIN_CONTRAST,
                    "{}/{} is {ratio:.2}:1 on base, below {MIN_CONTRAST}:1",
                    flavor.identifier,
                    color.identifier,
                );
            }
        }
    }

    /// Fills that stack on each other have to be told apart. Only the four
    /// fills are covered: `mantle` and `crust` are chrome, and acetic keeps
    /// those nearly flush with its pure black base on purpose.
    #[test]
    fn fills_are_separable() {
        for flavor in FLAVORS {
            let c = &flavor.colors;
            let fills = [
                ("base", c.base),
                ("surface0", c.surface0),
                ("surface1", c.surface1),
                ("surface2", c.surface2),
            ];
            for pair in fills.windows(2) {
                let (lower, upper) = (pair[0], pair[1]);
                let ratio = lower.1.contrast(upper.1);
                assert!(
                    ratio >= MIN_FILL_STEP,
                    "{}: {} on {} is {ratio:.2}:1, below {MIN_FILL_STEP}:1",
                    flavor.identifier,
                    upper.0,
                    lower.0,
                );
            }
        }
    }

    /// Dimmed text keeps working on a raised fill. Text tones are not required
    /// to separate from one another — they are a hierarchy, not fills — so only
    /// their contrast against a fill is asserted.
    #[test]
    fn dim_text_is_readable_on_fills() {
        for flavor in FLAVORS {
            let c = &flavor.colors;
            for (text_name, text, fill_name, fill) in [
                ("overlay1", c.overlay1, "surface0", c.surface0),
                ("overlay2", c.overlay2, "surface1", c.surface1),
            ] {
                let ratio = text.contrast(fill);
                assert!(
                    ratio >= MIN_DIM_TEXT,
                    "{}: {text_name} on {fill_name} is {ratio:.2}:1, below {MIN_DIM_TEXT}:1",
                    flavor.identifier,
                );
            }
        }
    }

    /// The surface ramp must climb monotonically away from `base`, and the text
    /// ramp must brighten monotonically towards `text`.
    #[test]
    fn ramps_are_monotonic() {
        for flavor in FLAVORS {
            let c = &flavor.colors;
            let ramp = [
                c.base, c.surface0, c.surface1, c.surface2, c.overlay0, c.overlay1, c.overlay2,
                c.subtext0, c.subtext1, c.text,
            ];
            for pair in ramp.windows(2) {
                assert!(
                    pair[1].luminance() > pair[0].luminance(),
                    "{}: {} is not lighter than {}",
                    flavor.identifier,
                    pair[1],
                    pair[0],
                );
            }
        }
    }

    /// Chrome stays subtle, but the three layers must still be distinguishable
    /// from one another.
    #[test]
    fn chrome_layers_are_separable() {
        for flavor in FLAVORS {
            let c = &flavor.colors;
            for (lower, upper) in [("base", c.base), ("mantle", c.mantle)]
                .into_iter()
                .zip([("mantle", c.mantle), ("crust", c.crust)])
            {
                let ratio = lower.1.contrast(upper.1);
                assert!(
                    ratio >= MIN_CHROME_STEP,
                    "{}: {} against {} is {ratio:.2}:1, below {MIN_CHROME_STEP}:1",
                    flavor.identifier,
                    upper.0,
                    lower.0,
                );
            }
        }
    }

    /// `crust` is the darkest chrome and `mantle` sits between it and `base` —
    /// unless the flavour inverts the depth axis, where the order flips.
    #[test]
    fn chrome_follows_depth_direction() {
        for flavor in FLAVORS {
            let c = &flavor.colors;
            let (crust, mantle, base) = (
                c.crust.luminance(),
                c.mantle.luminance(),
                c.base.luminance(),
            );
            if flavor.inverted_depth {
                assert!(crust > mantle && mantle > base, "{}", flavor.identifier);
                assert!(
                    c.crust.luminance() < c.surface0.luminance(),
                    "{}: crust must stay below surface0",
                    flavor.identifier
                );
            } else {
                assert!(crust < mantle && mantle < base, "{}", flavor.identifier);
            }
        }
    }
}

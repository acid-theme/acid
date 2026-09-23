//! Acid: a very dark colourscheme in two flavours.
//!
//! This crate is the single source of truth for both flavours. Everything
//! downstream — `palette.json`, every port, every preview — is derived from the
//! constants in this file, so a colour is only ever edited here.

mod color;
pub mod json;

use serde::Serialize;

pub use color::{Hex, Hsl, Rgb};

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
        subtext1: Hex::new(0xcdc7b8),
        subtext0: Hex::new(0xa8a49a),

        overlay2: Hex::new(0x8c8c8c),
        overlay1: Hex::new(0x6f6f6f),
        overlay0: Hex::new(0x555555),

        surface2: Hex::new(0x3a3a3a),
        surface1: Hex::new(0x2b2b2b),
        surface0: Hex::new(0x1e1e1e),

        base: Hex::new(0x000000),
        mantle: Hex::new(0x0a0a0a),
        crust: Hex::new(0x141414),
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
        subtext1: Hex::new(0xc5b99f),
        subtext0: Hex::new(0xa89e8b),

        overlay2: Hex::new(0x918879),
        overlay1: Hex::new(0x776f64),
        overlay0: Hex::new(0x5d5850),

        surface2: Hex::new(0x433f36),
        surface1: Hex::new(0x35322b),
        surface0: Hex::new(0x282621),

        base: Hex::new(0x1c1b19),
        mantle: Hex::new(0x151413),
        crust: Hex::new(0x0e0e0d),
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

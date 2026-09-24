# Design

Both flavours share one set of role names, so a port written against them works
for either.

- **Fills**: `crust`, `mantle`, `base`, `surface0`–`surface2`
- **Text**: `overlay0`–`overlay2`, `subtext0`, `subtext1`, `text`
- **Accents**: `red`, `orange`, `yellow`, `green`, `aqua`, `blue`, `purple`

[PALETTE.md](PALETTE.md) lists the values and what each role is for.

## Rules

Three rules hold the palette together. Each is enforced by a test in
`crates/acid-palette/src/lib.rs`, so a colour that breaks one cannot be
committed.

**Accents and text clear 4.5:1 against their own `base`** — WCAG 2.1 AA for body
text. `overlay0` and `overlay1` are exempt: they are for comments and inactive
interface elements, which are meant to recede.

**Neighbouring fills clear 1.3:1**, so a cursor line or a selection is visible
against the buffer behind it. `mantle` and `crust` are excluded, being chrome
rather than fills.

**Dimmed text clears 3:1 against the fill it sits on**, which keeps comments
readable on a cursor line. Text tones are not required to separate from one
another; they are a hierarchy, never stacked.

## Acetic inverts the depth axis

Acetic's `base` is pure black, so nothing can sit beneath it. `mantle` and
`crust` are lighter rather than darker, and chrome is raised off the editor field
rather than sunk below it.

Flavours expose this as `invertedDepth` in `palette.json` and
`flavor.invertedDepth` in templates, so a port can branch on it rather than
assume an order.

## Accent mapping

The accents are named after gruvbox's but are not mapped the way gruvbox maps
them. Gruvbox paints keywords red and functions green; Acid uses purple for
keywords and aqua for functions, which leaves red for errors, deletions and
exceptions.

Gruvbox's paired bright and neutral accents are not carried over. Each flavour
has one accent set, and ports derive brighter or dimmer variants with the
`lighten` and `mix` filters — the Alacritty port builds its ANSI ramps that way.

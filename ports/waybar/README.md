# Acid for Waybar

Two flavours: **Acetic** (`#000000`), vibrant, and **Citric** (`#1c1b19`), muted.

Part of [Acid](https://github.com/acid-theme/acid), a very dark colourscheme in two
flavours. The main README lists the other ports.

## Install

```sh
curl -fsSLo ~/.config/waybar/acid-acetic.css \
  https://raw.githubusercontent.com/acid-theme/waybar/main/acid-acetic.css
```

Import it above the rules that use it, so a later definition can override it:

```css
@import "acid-acetic.css";

window#waybar {
    background-color: @base;
    color: @text;
    border: 1px solid @surface1;
}
```

Coming from a Catppuccin stylesheet, three names have no Acid equivalent:
`@sky` is `@aqua`, `@mauve` is `@purple`, and `@maroon` is `@orange` or `@red`
depending on whether it marked a warning or an error. GTK reports no error for a
colour name it does not know, so a leftover name fails silently — the widget is
simply left unstyled.

## Files

- `acid-acetic.css`
- `acid-citric.css`

## Generated

Acid 0.1.0, rendered by acidify from
[`ports/waybar/acid.css.tera`](https://github.com/acid-theme/acid/blob/main/ports/waybar/acid.css.tera).
Edits to these files are overwritten on the next release. Report issues on
[acid-theme/acid](https://github.com/acid-theme/acid/issues).

## Licence

MIT.

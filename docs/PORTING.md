# Porting

A port is a folder under `ports/`. Everything about it lives there: what it is,
how it renders, its README, its checks and its preview. There is no registry to
register it in — the folder is the registration.

```sh
acid new kitty --title Kitty
```

That writes the folder and tells you what to replace.

## The folder

```
ports/kitty/
  port.toml     what the port is and how it renders
  theme.tera    the theme, rendered once per variant
  README.tera   the port's own README, published with it
  test.rs       what the port's own program says about the theme
  preview.rs    optional, when the program renders headlessly
  dist/         generated, laid out exactly as the published repository
```

## The manifest

```toml
title = "Kitty"
description = "Acid for kitty. A very dark colourscheme."

[test]
packages = ["kitty"]

[preview]
packages = ["kitty", "xorg-server-xvfb"]

[render]
matrix = ["flavor"]
filename = "dist/acid-{flavor}.conf"
```

Unknown fields are refused, so a typo is a failed check rather than a setting
that quietly does nothing.

`matrix` always includes `flavor`, and may add one axis of its own:

```toml
matrix = ["flavor", { format = ["theme", "conf"] }]
filename = { theme = "dist/themes/Acid {flavor_name}.theme", conf = "dist/conf.d/acid-{flavor}.fish" }
prefix = { theme = "", conf = "set -g " }
```

`filename` takes `{flavor}` and `{flavor_name}`, and may differ per variant.
`prefix` is a string the template can use when a variant needs one — fish uses
it to prefix every line of its `conf` form.

## The template

Templates are [askama](https://askama.readthedocs.io), compiled into the binary.
A colour role that does not exist is a build error rather than an empty string in
a published theme.

```tera
# Acid {{ version }} — {{ flavor.name }}, for kitty

background {{ flavor.colors.base }}
foreground {{ flavor.colors.text }}
color1     {{ flavor.colors.red }}
```

| Available | |
| --- | --- |
| `flavor` | the flavour being rendered: `identifier`, `name`, `description`, `dark`, `inverted_depth`, `colors` |
| `flavor.colors.<role>` | a colour, which renders as `#rrggbb` |
| `flavor.colors.iter()` | every colour in canonical order, each with `identifier`, `name`, `order`, `accent`, `hex` |
| `flavor.accents()` | the seven accents |
| `version` | the release that produced the file |
| `format` | the value of the port's own axis, empty when it has none |
| `prefix` | the manifest's prefix for this variant, empty by default |

Every generated file names the version that produced it, so a theme installed by
hand can be identified. `acid check` fails without it.

## Filters

| Filter | Result |
| --- | --- |
| `hex` | `#rrggbb` |
| `bare` | `rrggbb`, no `#` |
| `alpha(0.5)`, `alpha_bare(0.5)` | `#rrggbbaa`, `rrggbbaa` |
| `css_rgb`, `css_hsl` | `rgb(r, g, b)`, `hsl(h, s%, l%)` |
| `lighten(0.1)`, `darken(0.1)` | shift lightness |
| `saturate(0.1)`, `desaturate(0.1)` | shift saturation |
| `mix(other, 0.5)` | blend towards another colour |
| `contrast(other)` | WCAG 2.1 contrast ratio |

Filters returning a colour can be piped onwards:
`{{ flavor.colors.red | mix(flavor.colors.base, 0.3) | lighten(0.05) }}`.

## Checks

`test.rs` runs inside a container where the port's program is installed, and
reports what that program says.

```rust
pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["kitty"]) {
        return Ok(());
    }
    for flavour in h.flavours() {
        let theme = h.dist(&format!("acid-{}.conf", flavour.identifier));
        let output = exec("kitty", &["--config", &theme.to_string_lossy(), "--version"]);
        h.check(!output.contains("Bad"), format!("{}: kitty parsed the theme", flavour.identifier));
    }
    Ok(())
}
```

Ask the program wherever the program can be asked. Where it cannot — a client
that needs a login, a format with no validator — check the theme against the
program's own data: its documented option list, its bundled defaults. Say in the
test what it cannot tell.

**Give every test a control**: a deliberately broken theme that must be
rejected. A check that cannot fail proves nothing, and this has caught a passing
test more than once.

## Previews

`preview.rs` runs the real program and captures what it draws: terminal ports
through Alacritty under a headless X server, Wayland ports under a headless
compositor.

```rust
pub fn run(p: &mut Preview) -> anyhow::Result<()> {
    p.in_terminal("760x520", 84, 26, 11, &["kitty"])?;
    p.capture_x11()
}
```

A preview is optional. A program that will not run headlessly simply has no
`preview.rs`, and its repository gets no preview section.

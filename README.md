# Acid

A very dark colourscheme in the spirit of gruvbox, in two flavours.

| Flavour | Background | Accents | For |
| --- | --- | --- | --- |
| **Acetic** | `#000000` | Vibrant | OLED panels, dark rooms, maximum separation between code and background |
| **Citric** | `#1c1b19` | Muted | Everyday use; the flavour that sits closest to gruvbox |

See [docs/PALETTE.md](docs/PALETTE.md) for every colour, with contrast ratios.

## Layout

```
crates/acid-palette/src/lib.rs   the source of truth: every colour, once
crates/acidify/                  the renderer: templates in, ports out
palette.json                     generated; what non-Rust ports read
ports/<app>/<name>.tera          one template per port
ports/<app>/...                  generated; what users install
docs/palette.md.tera             the palette table, generated the same way
```

A colour is only ever edited in `crates/acid-palette/src/lib.rs`. Everything
else is derived, so run `make` afterwards and commit what changes.

```sh
make            # regenerate palette.json, every port, and the docs
make check      # fail if anything is stale, and run the tests. For CI
make preview    # print every colour slot the current terminal theme has loaded
```

## Writing a port

Acid follows [Catppuccin's](https://github.com/catppuccin/catppuccin) approach:
one Tera template per port, rendered once per flavour. Catppuccin's own
`whiskers` cannot be reused, because it renders from the Catppuccin palette
compiled into it, so `acidify` reproduces its shape against the Acid palette.

A template is a YAML frontmatter block followed by the file to render:

```tera
---
acidify:
  version: "0.1.0"
  matrix:
    - flavor
  filename: "themes/acid-{{ flavor.identifier }}.toml"
---
background = "{{ flavor.colors.base }}"
foreground = "{{ flavor.colors.text }}"
comment    = "{{ flavor.colors.overlay1 }}"
```

Render it with `acidify path/to/template.tera`. Output paths resolve against the
template's own directory unless `--output-dir` says otherwise.

### Frontmatter

| Key | Meaning |
| --- | --- |
| `acidify.matrix` | The axes to render across. Omit it to render once. |
| `acidify.filename` | A Tera template for the output path. Omit it to write to stdout. |
| `acidify.version` | The `acidify` version the template was written against. |
| anything else | Passed through as a template variable. |

`matrix` takes `flavor`, `accent`, or `{ key: [values] }` for an axis of your
own. Axes multiply, and the first one declared varies slowest:

```yaml
matrix:
  - flavor                      # binds `flavor`
  - accent                      # binds `accent`, resolved against that flavour
  - transparent: [true, false]  # binds `transparent`
```

### Variables

| Variable | What it holds |
| --- | --- |
| `flavor` | The current flavour, when `flavor` is in the matrix. |
| `flavors` | Every flavour, keyed by identifier, always available. |
| `accent` | The current accent, when `accent` is in the matrix. |
| `flavor.colors.<role>` | A hex string, e.g. `#000000`. |
| `flavor.color_list` | Every colour in canonical order, with `identifier`, `name`, `order`, `accent`, `hex`, `bare`, `rgb`, `hsl`. |
| `flavor.accents` | The seven accents, same shape. |
| `flavor.invertedDepth` | Whether `mantle` and `crust` sit above `base` rather than below. |

Colours render as `#rrggbb`. Every other form comes from a filter, so
`{{ flavor.colors.base }}` and `{{ flavor.colors.base | css_rgb }}` both read
naturally.

### Filters

| Filter | Result |
| --- | --- |
| `hex` | `#rrggbb`, normalised |
| `bare` | `rrggbb`, with no `#` |
| `css_rgb`, `css_hsl` | `rgb(r, g, b)`, `hsl(h, s%, l%)` |
| `alpha(amount=0.5)` | `#rrggbbaa` |
| `lighten(amount=0.1)`, `darken(amount=0.1)` | Shift lightness |
| `saturate(amount=0.1)`, `desaturate(amount=0.1)` | Shift saturation |
| `mix(with=…, amount=0.5)` | Blend towards another colour |
| `contrast(with=…)` | WCAG 2.1 contrast ratio |

`amount` accepts a fraction or a percentage: `0.2` and `20` mean the same thing.
Every filter takes a hex string or a colour object, so they chain.

Use `--context` while writing a template to print the variables it will see, and
`--stdout` to render without writing files.

## Design

Both flavours share one structure, so a port written against the role names
works for both:

- **Surfaces** run `crust`, `mantle`, `base`, `surface0`–`surface2`.
- **Text** runs `overlay0`–`overlay2`, `subtext0`, `subtext1`, `text`.
- **Accents** are the seven gruvbox names: `red`, `orange`, `yellow`, `green`,
  `aqua`, `blue`, `purple`.

Two things are worth knowing:

**Acetic inverts the depth axis.** Its `base` is pure black, so nothing can sit
beneath it: `mantle` and `crust` are lighter than `base` instead of darker, and
chrome is raised off the editor field rather than sunk below it. Flavours
advertise this as `invertedDepth`, so a port can branch on it rather than assume
an ordering.

**Every accent and text tone clears 4.5:1 against its own `base`** — WCAG 2.1 AA
for body text. A test enforces it, so a colour that fails cannot be committed.
`overlay0` and `overlay1` are deliberately below that line; they are for
comments and inactive UI, which are meant to recede.

Gruvbox's paired bright and neutral accents are not carried over. Each flavour
has one accent set, and ports derive brighter or dimmer variants with
`lighten` and `mix`, the way the Alacritty port builds its ANSI ramps.

## Ports

| Port | Template |
| --- | --- |
| Alacritty | [`ports/alacritty/acid.toml.tera`](ports/alacritty/acid.toml.tera) |

### Installing the Alacritty port

Symlink the themes so a `make ports` reaches the terminal without a second step,
then import one from `alacritty.toml`. Alacritty reloads on save.

```sh
mkdir -p ~/.config/alacritty/themes
ln -sfn "$PWD"/ports/alacritty/themes/acid-*.toml ~/.config/alacritty/themes/
```

```toml
general.import = ["~/.config/alacritty/themes/acid-acetic.toml"]
```

Anything under `[colors]` in `alacritty.toml` overrides the import, so drop any
leftover `[colors.primary] background` line — it would pin citric to acetic's
black. `make preview` then shows what actually loaded.

## Licence

MIT.

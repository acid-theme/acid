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

### Installing the Neovim port

`ports/nvim` is a runtime directory, so point a plugin manager at the
repository — `dir = "~/sources/acid/ports/nvim"` for lazy.nvim — or add it to
`runtimepath` yourself:

```lua
vim.opt.runtimepath:prepend("~/sources/acid/ports/nvim")
vim.cmd.colorscheme("acid-acetic")   -- or acid-citric
```

To try it without touching your config:

```sh
nvim --clean --cmd 'set rtp^=ports/nvim' -c 'colorscheme acid-citric' README.md
```

The port covers the editor and chrome groups, legacy syntax, treesitter,
markup, diagnostics, LSP (semantic tokens link to the treesitter groups, so a
server cannot repaint a buffer differently from the parser), diffs, spelling,
`:terminal`, and git signs.

### Installing the fish port

The port renders in two formats from one mapping, because fish has two ways to
carry a theme.

`fish_config theme choose` writes **universal** variables, which live in
`fish_variables` and are not usually committed:

```fish
cp ports/fish/themes/*.theme ~/.config/fish/themes/
fish_config theme choose "Acid Acetic"
```

`conf.d/acid-<flavour>.fish` sets the same variables **globally** instead, so
the theme travels with a tracked config rather than with the machine:

```fish
ln -sf "$PWD"/ports/fish/conf.d/acid-acetic.fish ~/.config/fish/conf.d/
```

Use one or the other. A global assignment in `conf.d` wins over a universal
variable, so a leftover `fish_config` choice will not fight it — but two
`conf.d` files both setting colours will.

### Installing the Waybar port

The port is the role names as GTK colours; the stylesheet stays yours. Import it
first, because GTK resolves `@define-color` at parse time and a name used before
it is defined is an error, not a fallback.

```css
@import "/path/to/acid/ports/waybar/themes/acid-acetic.css";

window#waybar {
    background-color: @base;
    color: @text;
    border: 1px solid @surface1;
}

#battery.discharging.warning  { color: @orange; }
#battery.discharging.critical { color: @red; }
```

Coming from a Catppuccin stylesheet, three names have no Acid equivalent:
`@sky` is `@aqua`, `@mauve` is `@purple`, and `@maroon` is `@orange` or `@red`
depending on whether it marked a warning or an error.

### Installing the mako port

Colour-only, pulled in with `include=`, which needs an absolute or `~/`-prefixed
path:

```ini
include=~/.config/mako/acid-acetic.conf

# Your own settings. mako takes the last matching value, so anything below the
# include wins, and anything above it is overridden by the theme.
font=GeistMonoNerdFont 10
padding=6
border-size=1
```

The theme sets the defaults plus `[urgency=low]`, `[urgency=normal]`,
`[urgency=high]` and `[hidden]`. Those sections do not leak into the including
file — global options after the `include=` are still read as globals.

### Installing the niri port

Two `layout` blocks in one file are a duplicate-node error, but niri merges a
`layout` block that arrives through `include`, so the theme can carry the
colours while your config keeps the geometry:

```kdl
include "acid-acetic.kdl"

layout {
    gaps 4
    border {
        width 0.5
    }
}
```

Drop the `active-color`, `inactive-color` and `urgent-color` lines from your own
blocks when you add the include. Setting the same key on both sides still
validates, but which one wins is unspecified — so do not rely on it either way.

The theme sets `focus-ring`, `border`, `shadow`, `insert-hint`, `tab-indicator`
and the overview backdrop. `background-color` is left commented out, since
`transparent` is a deliberate choice when a backdrop shows through.

Check the result before reloading: `niri validate -c ~/.config/niri/config.kdl`.

### Installing the swaylock port

swaylock has no include directive, so the theme is a colours-only config to
compose with. Keep your own options in a separate file and concatenate the two:

```sh
cat ~/.config/swaylock/base.conf \
    ~/sources/acid/ports/swaylock/themes/acid-acetic.conf \
    > ~/.config/swaylock/config
```

Or add your options to a copy of the theme and point swaylock at it with
`swaylock -C`. Either way, keep the two files disjoint: an unrecognised key
makes swaylock exit rather than lock, and a duplicate key is silently the last
one to win.

All 29 of swaylock's colour options are set, so nothing falls back to its
light-grey default. The ring carries state — a surface when idle, then green
cleared, yellow for Caps Lock, blue verifying, red wrong.

## Design

Both flavours share one structure, so a port written against the role names
works for both:

- **Surfaces** run `crust`, `mantle`, `base`, `surface0`–`surface2`.
- **Text** runs `overlay0`–`overlay2`, `subtext0`, `subtext1`, `text`.
- **Accents** are the seven gruvbox names: `red`, `orange`, `yellow`, `green`,
  `aqua`, `blue`, `purple`.

The accents are named after gruvbox's but are not mapped the way gruvbox maps
them. Gruvbox paints keywords red and functions green; Acid uses purple for
keywords and aqua for functions, so red is left for things that are actually
wrong — errors, deletions, exceptions. `docs/PALETTE.md` has the full table, and
ports are expected to follow it.

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
| Neovim | [`ports/nvim/acid.lua.tera`](ports/nvim/acid.lua.tera) |
| fish | [`ports/fish/acid.fish.tera`](ports/fish/acid.fish.tera) |
| Waybar | [`ports/waybar/acid.css.tera`](ports/waybar/acid.css.tera) |
| mako | [`ports/mako/acid.conf.tera`](ports/mako/acid.conf.tera) |
| niri | [`ports/niri/acid.kdl.tera`](ports/niri/acid.kdl.tera) |
| swaylock | [`ports/swaylock/acid.conf.tera`](ports/swaylock/acid.conf.tera) |

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

# Porting

Acid follows [Catppuccin's](https://github.com/catppuccin/catppuccin) approach:
one Tera template per port, rendered once per flavour. Catppuccin's `whiskers`
cannot be reused, because it renders from the Catppuccin palette compiled into
it, so `acidify` reproduces its shape against the Acid palette.

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
template's own directory unless `--output-dir` says otherwise. `--context` prints
the variables a template will see; `--stdout` renders without writing.

Every generated file names the version that produced it, so a theme installed by
hand can be identified. Put `{{ version }}` in the header; `make check` fails
without it.

A new port also needs an entry in `resources/ports.toml` and a test in
`tests/ports/`. See [DEVELOPMENT.md](DEVELOPMENT.md).

## Frontmatter

| Key | Meaning |
| --- | --- |
| `acidify.matrix` | The axes to render across. Omit to render once. |
| `acidify.filename` | A Tera template for the output path. Omit to write to stdout. |
| `acidify.version` | The `acidify` version the template was written against. |
| anything else | Passed through as a template variable. |

`matrix` takes `flavor`, `accent`, or `{ key: [values] }` for a custom axis. Axes
multiply, and the first declared varies slowest:

```yaml
matrix:
  - flavor                      # binds `flavor`
  - accent                      # binds `accent`, resolved against that flavour
  - transparent: [true, false]  # binds `transparent`
```

The fish port uses a custom axis to render two file formats from one mapping.

## Variables

| Variable | Holds |
| --- | --- |
| `flavor` | The current flavour, when `flavor` is in the matrix. |
| `flavors` | Every flavour, keyed by identifier. Always available. |
| `accent` | The current accent, when `accent` is in the matrix. |
| `flavor.colors.<role>` | A hex string, e.g. `#000000`. |
| `flavor.color_list` | Every colour in canonical order, with `identifier`, `name`, `order`, `accent`, `hex`, `bare`, `rgb`, `hsl`. |
| `flavor.accents` | The seven accents, same shape. |
| `flavor.invertedDepth` | Whether `mantle` and `crust` sit above `base`. |
| `version` | The release that produced the file. Always available. |

Colours render as `#rrggbb`. Every other form comes from a filter.

## Filters

| Filter | Result |
| --- | --- |
| `hex` | `#rrggbb`, normalised |
| `bare` | `rrggbb`, no `#` |
| `css_rgb`, `css_hsl` | `rgb(r, g, b)`, `hsl(h, s%, l%)` |
| `alpha(amount=0.5)` | `#rrggbbaa` |
| `lighten(amount=0.1)`, `darken(amount=0.1)` | Shift lightness |
| `saturate(amount=0.1)`, `desaturate(amount=0.1)` | Shift saturation |
| `mix(with=…, amount=0.5)` | Blend towards another colour |
| `contrast(with=…)` | WCAG 2.1 contrast ratio |

`amount` accepts a fraction or a percentage: `0.2` and `20` are equivalent. Every
filter takes a hex string or a colour object, so they chain. Tera's own filters
are available too — `trim_start_matches(pat="#")` strips the prefix from an
`alpha` result, which is what swaylock needs.

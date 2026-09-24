# Development

```
crates/acid-palette/src/lib.rs   the source of truth: every colour, once
crates/acidify/                  the renderer: templates in, ports out
palette.json                     generated; what non-Rust ports read
resources/ports.toml             the port registry
ports/<app>/*.tera               one template per port
ports/<app>/                     generated themes
docs/                            palette reference, generated
tests/                           each port checked by its own tool
scripts/publish.py               assembles and pushes the port repositories
```

A colour is edited only in `crates/acid-palette/src/lib.rs`. Everything else is
generated, so run `make` afterwards and commit the result.

```sh
make            # regenerate palette.json, the ports, and the docs
make check      # run the tests and fail if any generated file is stale
make test-ports # check each port with its own tool, in a container
make mirrors    # build each port repository's contents locally
```

`make check` is the gate: it runs the Rust tests, re-renders every template to
confirm the committed output matches, and validates the registry against the
working tree.

## Adding a port

1. Write `ports/<app>/<name>.tera`. See [PORTING.md](PORTING.md).
2. Add an entry to `resources/ports.toml`: repository name, description,
   template, the files to publish and where they land, and install instructions.
3. Add `tests/ports/<app>.sh`.
4. Add `previews/ports/<app>.sh`, and `preview_packages` to the registry entry.
5. Run `make && make check`, then `previews/run.sh <app>` to check it by eye.

The registry entry is not optional; `make check` fails without one. The port
repository's README and preview are assembled at publish time, so neither is
kept here.

## Port tests

Each port is checked by the program that will read its theme, inside a container
built from `tests/Containerfile`. Podman is required.

```sh
tests/run.sh            # every port
tests/run.sh neovim     # one port
tests/run.sh --build    # rebuild the image first
```

Every port test includes a negative control — a broken theme, an invented key, a
duplicate node — so a test that cannot fail is caught.

## Previews

A preview is a screenshot of the real program: terminal ports run under Xvfb in
Alacritty, Wayland ports under a headless wlroots compositor, both in a
container.

**Each port renders its own.** Publishing gives a port repository everything it
needs — `preview/render.sh`, `preview/Containerfile` built from the port's
`preview_packages`, the shared helpers, and a workflow — and that repository's
CI renders the images and commits them to its own `previews/` directory.

The steps are shared rather than copied: `.github/workflows/port-preview.yml`
here is a reusable workflow, and each port's generated workflow is a dozen lines
calling it. A change to the steps takes effect on every port's next run without
republishing.

```sh
previews/run.sh            # every port
previews/run.sh neovim     # one port
```

That builds the mirrors and renders from them, so it runs exactly what a port's
CI runs. Output goes to `target/previews/`, which is not committed: previews
belong to the port repositories.

Rendering is not deterministic to the pixel — font rasterisation and timing vary
— so previews are not part of `make check`.

## Versioning

One version covers the palette, the renderer and every port: a theme is only ever
released together with the palette it came from. It is set once, in
`[workspace.package]` in `Cargo.toml`, and reaches everything else from there —
`acid_palette::VERSION`, the `version` field in `palette.json`, the `{{ version }}`
template variable, and the header of every generated file.

| Change | Bump |
| --- | --- |
| A colour value | minor |
| A role added, removed or renamed | major |
| A port's mapping, or a new port | minor |
| A fix that leaves every generated file unchanged | patch |

A colour value is a minor bump rather than a patch because it changes every
port's output.

`make check` fails if the manifest and `palette.json` disagree, and if any
published file is missing the current version.

### Releasing

```sh
$EDITOR Cargo.toml     # bump [workspace.package] version
make                   # restamp and re-render everything
make check
git commit
git tag v0.2.0
git push --tags        # publish.yml mirrors the tag to every port repository
```

## Publishing

This repository is the hub and the only one edited. Each port also has a
repository in the [acid-theme](https://github.com/acid-theme) organisation
holding just that port's files at the root, so a single `curl` installs a theme
and `vim.pack` and `fisher` can consume the Neovim and fish ports directly.

Those repositories are mirrors. `scripts/publish.py` assembles each one from
`resources/ports.toml` — the themes, a README rendered from the registry's
install text, the previews, and the licence — and pushes one commit per change.
A pull request against a mirror cannot be merged; each generated README says so
and points back here.

Everything specific to one port lives in that port's repository. This one keeps
the sources: the palette, the templates, the registry and the generators.

```sh
make publish                                    # push everything that changed
make publish ARGS="--only neovim --dry-run"
```

Publishing is idempotent, and only ever pushes. Port repositories are created
once, by hand, and publishing fails with a clear error if one is missing.

## Continuous integration

`ci.yml` runs formatting, lints, tests, `make check`, the registry validation and
the port tests on every push, then uploads the built mirrors as an artifact.

`publish.yml` publishes on a `v*` tag, or on manual dispatch with optional `only`
and `dry_run` inputs, gating on the full check first.

`port-preview.yml` is not run here. It is the reusable workflow each port
repository calls to render its own preview.

Publishing needs a token that can write to the sibling repositories, because the
default `GITHUB_TOKEN` is scoped to this repository alone. Create a fine-grained
personal access token owned by the organisation with `Contents: Read and write`
on the port repositories, and store it as the `ACID_PUBLISH_TOKEN` secret.

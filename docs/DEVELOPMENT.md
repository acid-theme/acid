# Development

```
crates/acid-palette/src/lib.rs   the source of truth: every colour, once
crates/acidify/                  the renderer: templates in, ports out
palette.json                     generated; what non-Rust ports read
resources/ports.toml             the port registry
ports/<app>/*.tera               one template per port
ports/<app>/                     generated themes and README
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
4. Run `make && make check`.

The registry entry and the README are not optional — `make check` fails without
an entry, and the port's README is generated from the registry's install text.

## Port tests

Each port is checked by the program that will read its theme, inside a container
built from `tests/Containerfile`. Podman is required.

```sh
tests/run.sh            # every port
tests/run.sh nvim       # one port
tests/run.sh --build    # rebuild the image first
```

Every port test includes a negative control — a broken theme, an invented key, a
duplicate node — so a test that cannot fail is caught.

## Publishing

This repository is the hub and the only one edited. Each port also has a
repository in the [acid-theme](https://github.com/acid-theme) organisation
holding just that port's files at the root, so a single `curl` installs a theme
and `vim.pack` and `fisher` can consume the Neovim and fish ports directly.

Those repositories are mirrors. `scripts/publish.py` assembles each one from
`resources/ports.toml` and pushes one commit per change. A pull request against a
mirror cannot be merged; each generated README says so and points back here.

```sh
make publish                                    # push everything that changed
make publish ARGS="--only nvim --dry-run"
```

Publishing is idempotent, and only ever pushes. Port repositories are created
once, by hand, and publishing fails with a clear error if one is missing.

## Continuous integration

`ci.yml` runs formatting, lints, tests, `make check`, the registry validation and
the port tests on every push, then uploads the built mirrors as an artifact.

`publish.yml` publishes on a `v*` tag, or on manual dispatch with optional `only`
and `dry_run` inputs, gating on the full check first.

Publishing needs a token that can write to the sibling repositories, because the
default `GITHUB_TOKEN` is scoped to this repository alone. Create a fine-grained
personal access token owned by the organisation with `Contents: Read and write`
on the port repositories, and store it as the `ACID_PUBLISH_TOKEN` secret.

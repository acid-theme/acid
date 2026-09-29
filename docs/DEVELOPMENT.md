# Development

```
acid.toml                        where the ports are published
crates/acid-palette/src/lib.rs   the source of truth: every colour, once
crates/acid/                     the one tool: render, check, test, preview, publish
palette.json                     generated; what other tools read
ports/<name>/                    everything about one port
docs/                            this, and the generated palette reference
previews/                        the sample files a preview may use
```

A colour is edited only in `crates/acid-palette/src/lib.rs`. Everything else is
generated, so run `make` afterwards and commit the result.

```sh
make            # regenerate palette.json, the ports and the docs
make check      # formatting, lints, tests, and everything generated current
make test-ports # check each port with its own program, in a container
make previews   # render each port's preview
make publish    # push each port to its own repository
```

## The tool

One binary does everything, and one language: Rust. Templates are compiled into
it, so a template that names a colour role wrongly does not build.

```sh
acid new <name>     start a port
acid render         render every theme into its dist directory
acid docs           the palette reference and the hub's generated tables
acid check          fail if anything generated is out of date or invalid
acid test [ports]   check each port with its own program
acid preview [ports]
acid publish [--only <port>] [--tag vX.Y.Z] [--dry-run]
```

`acid test` and `acid preview` have two halves. Outside, they build the port's
image and start the container; inside, they run that port's compiled checks. The
binary is mounted in, so the container needs no toolchain — and it is copied
first, so rebuilding while a long run is going does not break it.

## What `acid check` enforces

- formatting, clippy with warnings denied, and the Rust tests
- every generated file matches what its template produces now
- every manifest parses, with unknown fields refused
- **every published file names the version that produced it**
- **a preview script and its packages come together, or neither**
- every port has a test
- every variant of a matrix has a filename

The two in bold were each added after something slipped past: themes published
without a version stamp, and a port that named preview packages but had no
preview to run.

## Adding a port

1. `acid new <name>` and replace the stubs. See [PORTING.md](PORTING.md).
2. `make && make check`
3. `acid test <name>`, then `acid preview <name>` if it has one.

The hub's README lists the port automatically; the folder is the registry.

## Containers

One image per port, built from the packages its manifest names, so a run
installs that port's tool rather than every port's. The image is named after a
hash of its own definition and pulled from `ghcr.io` when one has been published,
so an unchanged definition is never rebuilt. Podman is required.

Every port's checks include a control — a deliberately broken theme that must be
rejected — because a check that cannot fail proves nothing.

## Versioning

One version covers the palette, the tool and every port: a theme is only ever
released together with the palette it came from. It is set once, in
`[workspace.package]` in `Cargo.toml`, and reaches everything else from there.

| Change | Bump |
| --- | --- |
| A colour value | minor |
| A role added, removed or renamed | major |
| A port's mapping, or a new port | minor |
| A fix that leaves every generated file unchanged | patch |

A colour value is a minor bump rather than a patch because it changes every
port's output.

### Releasing

```sh
$EDITOR Cargo.toml     # bump [workspace.package] version
make                   # restamp and re-render everything
make check
git commit
git tag v0.2.0
git push --tags        # publish.yml mirrors the tag to every port repository
```

`acid publish --tag` refuses a tag that disagrees with the palette's version, and
tags every port whether or not its files changed.

## Publishing

This repository is the hub and the only one edited. Each port also has a
repository in the [acid-theme](https://github.com/acid-theme) organisation
holding just that port's files at the root, so a single `curl` installs a theme
and `vim.pack` and `fisher` can consume the Neovim and fish ports directly.

Those repositories are mirrors: the themes, a README rendered from the port's own
`README.tera`, the licence, and a workflow. A pull request against one cannot be
merged; each generated README says so and points back here.

Previews are the exception — the port's own repository renders and commits them,
and publishing carries them over rather than replacing them. That is why a stale
local render cannot overwrite what a port's CI produced.

Publishing is idempotent, and only ever pushes. Port repositories are created
once, by hand.

## Continuous integration

`ci.yml` runs `make check` and the port checks on every push.

`publish.yml` publishes on every push to `main`, and on a `v*` tag, which
additionally moves that tag in each mirror. It needs a token that can write to
the sibling repositories, because the default `GITHUB_TOKEN` is scoped to this
repository alone: a fine-grained personal access token owned by the organisation
with `Contents: Read and write`, stored as the `ACID_PUBLISH_TOKEN` secret.

`port-preview.yml` is not run here. It is the reusable workflow each port
repository calls to render its own preview.

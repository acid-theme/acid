# Everything generated comes from crates/acid-palette/src/lib.rs and the ports'
# own templates. After touching a colour, run `make`.

ACID := cargo run --quiet -p acid --

.PHONY: all palette render docs check test test-ports previews publish fmt fmt-check lint clean

all: palette render docs

## Regenerate palette.json from the Rust source of truth.
palette:
	cargo run --quiet -p acid-palette --bin codegen

## Render every port's theme into its dist directory.
render:
	$(ACID) render

## Render the palette reference and the hub's generated tables.
docs:
	$(ACID) docs

## Everything CI checks: formatting, lints, tests, and stale output.
check: fmt-check lint test
	$(ACID) check

test:
	cargo test --workspace --quiet

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets --quiet -- --deny warnings

## Check each port with its own program, in a container. Needs podman.
## ARGS limits it, e.g. ARGS="neovim".
test-ports:
	$(ACID) test $(ARGS)

## Render a preview of each port by running the real program in a container.
previews:
	$(ACID) preview $(ARGS)

## Push each port to its own repository. Needs push rights on the org.
publish: all check
	$(ACID) publish $(ARGS)

clean:
	cargo clean

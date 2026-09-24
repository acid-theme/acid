# Everything downstream of crates/acid-palette/src/lib.rs is generated. After
# touching a colour, run `make`.

TEMPLATES := $(wildcard ports/*/*.tera) $(wildcard docs/*.tera)
ACIDIFY   := cargo run --quiet -p acidify --

.PHONY: all palette ports docs preview previews registry mirrors publish check test test-ports fmt fmt-check lint clean

all: palette ports docs

## Regenerate palette.json from the Rust source of truth.
palette:
	cargo run --quiet -p acid-palette --bin codegen

## Render every port template.
ports:
	$(ACIDIFY) $(wildcard ports/*/*.tera)

## Render the generated documentation.
docs:
	$(ACIDIFY) $(wildcard docs/*.tera)

## Print every colour slot the current terminal theme has loaded.
preview:
	@bash scripts/preview.sh

## Render a preview of each port by running the real program in a container.
## Needs podman. ARGS limits it, e.g. ARGS="nvim".
previews:
	@previews/run.sh $(ARGS)

## Run each port's tests in a container, against that port's own tool.
## Needs podman. ARGS limits it, e.g. ARGS="nvim".
test-ports:
	@tests/run.sh $(ARGS)

## Validate the port registry against the working tree.
registry:
	@python3 scripts/publish.py --check

## Build each port repository's contents locally, pushing nothing.
mirrors: all
	@python3 scripts/publish.py --out target/mirrors

## Push each port to its own repository. Needs push rights on the org.
publish: all check
	@python3 scripts/publish.py $(ARGS)

## Everything CI checks: formatting, lints, tests, stale output, the registry.
check: fmt-check lint test registry
	@mkdir -p target
	cargo run --quiet -p acid-palette --bin codegen target/palette.check.json
	diff -u palette.json target/palette.check.json
	$(ACIDIFY) --check $(TEMPLATES)

test:
	cargo test --workspace --quiet

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets --quiet -- --deny warnings

fmt:
	cargo fmt --all

clean:
	cargo clean

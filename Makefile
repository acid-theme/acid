# Everything downstream of crates/acid-palette/src/lib.rs is generated. After
# touching a colour, run `make`.

TEMPLATES := $(wildcard ports/*/*.tera) $(wildcard docs/*.tera)
ACIDIFY   := cargo run --quiet -p acidify --

.PHONY: all palette ports docs preview registry mirrors publish check test fmt clean

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

## Validate the port registry against the working tree.
registry:
	@python3 scripts/publish.py --check

## Build each port repository's contents locally, pushing nothing.
mirrors: all
	@python3 scripts/publish.py --out target/mirrors

## Push each port to its own repository. Needs push rights on the org.
publish: all check
	@python3 scripts/publish.py $(ARGS)

## Fail if any generated file is stale. For CI.
check: test registry
	@mkdir -p target
	cargo run --quiet -p acid-palette --bin codegen target/palette.check.json
	diff -u palette.json target/palette.check.json
	$(ACIDIFY) --check $(TEMPLATES)

test:
	cargo test --workspace --quiet

fmt:
	cargo fmt --all

clean:
	cargo clean

# Everything downstream of crates/acid-palette/src/lib.rs is generated. After
# touching a colour, run `make`.

TEMPLATES := $(wildcard ports/*/*.tera) $(wildcard docs/*.tera)
ACIDIFY   := cargo run --quiet -p acidify --

.PHONY: all palette ports docs preview check test fmt clean

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

## Fail if any generated file is stale. For CI.
check: test
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

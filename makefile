.PHONY: build test bench compare ci

build:
	cargo build --workspace --release

test:
	cargo test --workspace

bench:
	cargo bench -p truss-core

compare:
	bash scripts/compare-competitors.sh

ci: build test bench

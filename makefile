.PHONY: build test bench compare ci

build:
	cargo build --workspace --release

test:
	cargo test --workspace

bench:
	cargo bench -p truss-core

compare:
	hyperfine \
		--warmup 5 \
		--export-markdown benchmarks/hyperfine/compare.md \
		'./target/release/truss validate benchmarks/fixtures/large.yml' \
		'competitors/yaml-language-server/run.sh benchmarks/fixtures/large.yml'

ci: build test bench

# =========================
# Truss – Development Tasks
# =========================

# Configuration
set shell := ["bash", "-c"]

# Colors
GREEN := "\\033[0;32m"
RESET := "\\033[0m"

# -------------------------
# Default
# -------------------------
default:
	@just --list

# -------------------------
# Build
# -------------------------
build:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Building Truss (release){{RESET}}"'
	cargo build --workspace --release

build-debug:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Building Truss (debug){{RESET}}"'
	cargo build --workspace

# -------------------------
# Test
# -------------------------
test:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Running tests{{RESET}}"'
	cargo test --workspace

test-core:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Running core tests{{RESET}}"'
	cargo test -p truss-core

# -------------------------
# Bench (Rust / Criterion)
# -------------------------
bench:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Running core benchmarks (criterion){{RESET}}"'
	cargo bench -p truss-core

# -------------------------
# Bench (CLI / Hyperfine)
# -------------------------
bench-cli:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Running CLI benchmarks{{RESET}}"'
	hyperfine \
		--warmup 5 \
		'./target/release/truss validate benchmarks/fixtures/large.yml'

# -------------------------
# Compare (Truss vs competitors)
# -------------------------
compare:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Comparing Truss vs yaml-language-server{{RESET}}"'
	hyperfine \
		--warmup 5 \
		--export-markdown benchmarks/hyperfine/compare.md \
		'./target/release/truss validate benchmarks/fixtures/complex-dynamic.yml' \
		'competitors/yaml-language-server/run.sh benchmarks/fixtures/complex-dynamic.yml'

compare-smoke:
	hyperfine \
		--warmup 3 \
		'cargo run --release -- benchmarks/fixtures/simple.yml' \
		'actionlint benchmarks/fixtures/simple.yml' \
		'yamllint benchmarks/fixtures/simple.yml'

# -------------------------
# Full pipeline (local CI)
# -------------------------
ci:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Running local CI pipeline{{RESET}}"'
	just build
	just test
	just bench

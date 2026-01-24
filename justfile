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
	cargo test -p truss-core --no-fail-fast

test-validation:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Running all validation tests{{RESET}}"'
	@bash scripts/test-validation.sh

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
		'./target/release/truss validate --quiet benchmarks/fixtures/complex-dynamic.yml'

# -------------------------
# Compare (Truss vs competitors)
# -------------------------
compare:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Comparing Truss vs all competitors{{RESET}}"'
	@bash scripts/compare-competitors.sh

compare-smoke:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Quick smoke test comparison{{RESET}}"'
	@bash scripts/compare-competitors.sh benchmarks/fixtures/simple.yml 3 benchmarks/hyperfine/compare-smoke.md

# -------------------------
# Multifile Testing Framework
# -------------------------
test-multifile:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Running multifile test suite{{RESET}}"'
	@bash scripts/run-full-suite.sh

test-repo REPO:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Testing repository: {{REPO}}{{RESET}}"'
	@bash scripts/run-validation.sh test-suite/repos/{{REPO}}

setup-test-repos:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Setting up test repositories{{RESET}}"'
	@bash scripts/setup-test-repos.sh

compare-results:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Comparing validation results{{RESET}}"'
	@python3 scripts/compare-results.py test-suite/results test-suite/comparison/coverage.json

generate-report:
	@bash -eu -o pipefail -c 'echo -e "{{GREEN}}==> Generating comparison reports{{RESET}}"'
	@python3 scripts/generate-report.py test-suite/comparison/coverage.json test-suite/comparison/reports

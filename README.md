# Truss

High-performance CI/CD pipeline validation and analysis engine written in Rust. Truss provides real-time feedback, high performance, and reproducible results by detecting configuration errors, semantic inconsistencies, and autocomplete opportunities before a pipeline is executed.

## Performance Highlights

**Truss is 15-35x faster than existing GitHub Actions validation tools:**

| Tool | Language | Mean Time | Relative Speed |
|------|----------|-----------|----------------|
| **Truss** | Rust | **11.1ms** | 1.00x (baseline) |
| actionlint | Go | 165.7ms | **14.98x slower** |
| yaml-language-server | TypeScript | 381.7ms | **34.51x slower** |
| yamllint | Python | 210.9ms | **19.07x slower** |

*Benchmark: Complex dynamic workflow validation via Hyperfine. See [Performance](#performance) section for details.*

**Why this matters:** 11.1ms is fast enough for real-time LSP integration, enabling instant diagnostics as you type in your editor.

## Features

- **High Performance**: 15-35x faster than competitors (11.1ms average for complex workflows)
- **Semantic Validation**: Goes beyond syntax checking to validate semantic correctness
- **39 Validation Rules**: Comprehensive coverage of GitHub Actions workflow syntax and semantics
- **Modular Design**: Core engine with multiple adapter layers (CLI, LSP, WASM)
- **Measurable**: Comprehensive benchmarking infrastructure from day one
- **Incremental**: Designed for partial file edits and real-time feedback

## Quick Start

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- `just` (recommended) or `make` for build automation

### Installation

```bash
# Clone the repository
git clone https://github.com/JuanMarchetto/truss.git
cd truss

# Build the project
just build
# or: make build

# Run tests
just test
# or: make test
```

### Usage

#### CLI

```bash
# Validate a GitHub Actions workflow file
./target/release/truss validate path/to/workflow.yml

# Validate multiple files (processed in parallel)
./target/release/truss validate file1.yml file2.yml

# JSON output (includes timing and metadata)
./target/release/truss validate --json path/to/workflow.yml

# Quiet mode (suppress output, only exit code)
./target/release/truss validate --quiet path/to/workflow.yml
```

#### LSP Server

The Truss LSP server provides real-time diagnostics for GitHub Actions workflows in editors that support the Language Server Protocol.

```bash
# Run the LSP server (typically configured in your editor)
./target/release/truss-lsp
```

The LSP server supports:
- Real-time validation as you type
- Incremental parsing for performance
- Diagnostics for all 39 validation rules
- UTF-16 position handling for LSP compatibility

To use with your editor, configure it to use `truss-lsp` as the language server for YAML files (or specifically `.github/workflows/*.yml` files).

## Validation Rules

Truss implements **39 comprehensive validation rules** across 5 categories:

### Core & Structural (4 rules)
- **SyntaxRule** - YAML syntax validation via tree-sitter
- **NonEmptyRule** - Empty document detection
- **GitHubActionsSchemaRule** - Basic GitHub Actions workflow structure
- **WorkflowTriggerRule** - `on:` trigger configuration with 30+ event types

### Job-Level (9 rules)
- **JobNameRule** - Duplicate names, format, reserved words
- **JobNeedsRule** - Dependency validation with circular dependency detection
- **JobIfExpressionRule** - Conditional expression validation
- **JobOutputsRule** - Output reference validation
- **JobContainerRule** - Container image, ports, services
- **JobStrategyValidationRule** - Strategy structure validation
- **RunsOnRequiredRule** - Runner requirement enforcement
- **RunnerLabelRule** - GitHub-hosted runner label validation (22+ labels including ARM)
- **ReusableWorkflowCallRule** - Reusable workflow path and structure

### Step-Level (11 rules)
- **StepValidationRule** - Step structure (`uses` or `run` required)
- **StepNameRule** - Step name validation
- **StepIdUniquenessRule** - Unique step IDs within jobs
- **StepIfExpressionRule** - Step conditional expressions
- **StepOutputReferenceRule** - `steps.X.outputs.Y` reference validation
- **StepContinueOnErrorRule** - Boolean validation
- **StepTimeoutRule** - Timeout value validation
- **StepShellRule** - Shell type validation (bash, pwsh, python, sh, cmd, powershell)
- **StepWorkingDirectoryRule** - Working directory path validation
- **StepEnvValidationRule** - Environment variable name format
- **ArtifactValidationRule** - upload/download-artifact parameter validation

### Workflow-Level (9 rules)
- **WorkflowNameRule** - Workflow name validation
- **WorkflowInputsRule** - Workflow input types and requirements
- **WorkflowCallInputsRule** - Reusable workflow call input validation
- **WorkflowCallSecretsRule** - Secret definition and reference validation
- **WorkflowCallOutputsRule** - Workflow call output validation
- **TimeoutRule** - Job-level timeout validation
- **PermissionsRule** - Permission scope validation (15+ scopes)
- **ConcurrencyRule** - Concurrency group and cancel-in-progress
- **DefaultsValidationRule** - Default shell and working directory

### Expression & Reference (6 rules)
- **ExpressionValidationRule** - `${{ }}` syntax, functions, operators
- **ActionReferenceRule** - `owner/repo@ref` format validation
- **EventPayloadValidationRule** - Event-specific field validation (branches, tags, cron, types)
- **SecretsValidationRule** - Secret reference format and name validation
- **MatrixStrategyRule** - Matrix structure and key validation
- **EnvironmentRule** - Environment name format validation

See [docs/VALIDATION_RULES.md](docs/VALIDATION_RULES.md) for complete details on each rule.

## Performance

Truss is designed for speed and efficiency, making it ideal for real-time editor integration and CI/CD pipelines.

### Hyperfine Benchmark (CLI end-to-end)

Comparing `truss validate` against competitor tools on `benchmarks/fixtures/complex-dynamic.yml`:

| Tool | Mean Time | Min | Max | Relative to Truss |
|------|-----------|-----|-----|-------------------|
| **Truss** | **11.1ms** | 1.7ms | 17.9ms | 1.00x |
| actionlint | 165.7ms | 144.5ms | 189.7ms | 14.98x slower |
| yaml-language-server | 381.7ms | 263.9ms | 553.4ms | 34.51x slower |
| yamllint | 210.9ms | 94.6ms | 276.7ms | 19.07x slower |

### Criterion Benchmarks (Core engine)

| Fixture | Mean Time | Description |
|---------|-----------|-------------|
| Simple YAML | **225 us** | Minimal workflow (name + on: push) |
| Medium YAML | **984 us** | Standard workflow with multiple jobs |
| Complex static | **3.66 ms** | Large workflow with matrix, permissions, containers |
| Complex dynamic | **2.44 ms** | Dynamic workflow with expressions and reusable calls |

### Why Performance Matters

Truss achieves **15-35x better performance** than existing tools while providing comprehensive semantic validation:

- **Real-time editor feedback** - LSP integration with instant diagnostics (11.1ms is well under perceptible latency)
- **Large-scale validation** - Process hundreds of workflow files quickly via parallel processing
- **CI/CD integration** - Fast validation doesn't slow down pipelines
- **Better developer experience** - Instant feedback improves productivity

### Running Benchmarks

```bash
# Criterion benchmarks (core engine)
just bench
# or: cargo bench -p truss-core

# CLI benchmarks via Hyperfine
just bench-cli

# Compare against competitors
just compare
```

See [benchmarks/hyperfine/compare.md](benchmarks/hyperfine/compare.md) for detailed Hyperfine results.

## Project Structure

```
truss/
├── crates/
│   ├── truss-core/      # Core validation engine (editor-agnostic, deterministic)
│   │   ├── lib.rs        # Engine entry point with 39 registered rules
│   │   ├── parser.rs     # tree-sitter YAML parser with incremental support
│   │   ├── validation/   # Validation framework and 39 rule implementations
│   │   ├── tests/        # 40 integration test files (257+ tests)
│   │   └── benches/      # Criterion benchmarks
│   ├── truss-cli/        # Command-line interface (parallel file processing)
│   ├── truss-lsp/        # Language Server Protocol adapter
│   └── truss-wasm/       # WebAssembly bindings (placeholder)
├── benchmarks/           # Benchmark fixtures and Hyperfine results
├── competitors/          # Comparison benchmark scripts
├── test-suite/           # Multi-repo comparison testing framework
├── scripts/              # Build, test, and comparison automation
├── docs/                 # Architecture, rules, and test strategy docs
└── .github/workflows/    # CI/CD pipeline (check, test, clippy, fmt)
```

## Build Systems

Truss provides two build systems for flexibility:

### `justfile` (Recommended)

The primary build system with enhanced features:

```bash
just build          # Build release
just test           # Run all tests
just test-core      # Run core tests only
just bench          # Run Criterion benchmarks
just bench-cli      # Run Hyperfine CLI benchmarks
just compare        # Compare with competitors
just ci             # Full CI pipeline
```

**Installation:** `cargo install just` or see [just's documentation](https://github.com/casey/just)

### `makefile` (Fallback)

A minimal fallback for environments without `just`:

```bash
make build          # Build release
make test           # Run tests
make bench          # Run benchmarks
make compare        # Compare with competitors
make ci             # Full CI pipeline
```

## Development

### Building

```bash
# Debug build
just build-debug
# or: cargo build --workspace

# Release build
just build
# or: cargo build --workspace --release
```

### Testing

```bash
# Run all tests (257+ tests across 40 test files)
just test
# or: cargo test --workspace

# Run core tests only
just test-core
# or: cargo test -p truss-core
```

### CI Pipeline

The project uses GitHub Actions CI with 4 jobs:
- **Check** - `cargo check --workspace`
- **Test** - `cargo test --workspace` (257+ tests)
- **Clippy** - `cargo clippy --workspace -- -D warnings`
- **Format** - `cargo fmt --all -- --check`

All 4 jobs must pass on every push to `main` and on every pull request.

## Architecture

Truss follows a strict "Core First" architecture:

- **truss-core**: All critical logic lives here. Editor-agnostic and fully deterministic. Uses tree-sitter for YAML parsing and rayon for parallel rule execution.
- **Adapters** (truss-cli, truss-lsp, truss-wasm): Thin layers that adapt the core to different interfaces.

Key design principles:
- Performance is a first-class requirement, not an afterthought
- Validation rules are stateless and independently testable
- Results are deterministic and reproducible
- Incremental parsing enables real-time editor integration

For detailed architecture information, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Current Status

**MVP Complete:**
- 39 validation rules implemented and tested
- 257+ tests across 40 test files (all passing)
- LSP server with real-time diagnostics and incremental parsing
- CLI tool with parallel file processing and JSON output
- 15-35x faster than competitors
- Comprehensive benchmarking infrastructure
- Clean architecture (Core + adapters)
- CI/CD pipeline (check, test, clippy, fmt)

**Planned:**
- Contextual autocomplete
- WASM bindings (structure in place)
- `--version` flag and severity filtering
- Directory/glob scanning support

**Not Included (for now):**
- Azure Pipelines / GitLab CI support
- Advanced UI
- Complex configuration

## Documentation

- [Architecture Guide](docs/ARCHITECTURE.md) - Design principles and guidelines
- [Validation Rules](docs/VALIDATION_RULES.md) - Complete list of all 39 validation rules
- [Test Strategy](docs/TEST_STRATEGY.md) - Testing approach and organization

## Contributing

1. Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) to understand design principles
2. Ensure all tests pass: `just test`
3. Run clippy: `cargo clippy --workspace -- -D warnings`
4. Check formatting: `cargo fmt --all -- --check`
5. Follow the "Core First" principle: business logic belongs in `truss-core`

## License

Licensed under the MIT License. See [LICENSE-MIT](LICENSE-MIT) for details.

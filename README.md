# Truss

> **Heads up:** This project is still early. A good chunk of the code was AI-generated and is being actively reviewed, tested, and improved. All tests pass and the benchmarks are reproducible, but treat this as experimental for now. Bug reports and contributions are very welcome as we work toward a stable release.

A fast GitHub Actions workflow validator written in Rust. Truss catches configuration errors, semantic issues, and common mistakes in your CI/CD pipelines before you push — so you spend less time debugging failed runs.

## Why Truss?

**It's fast.** Validates any workflow in under 1ms:

| Fixture | Complexity | Mean Time |
|---------|-----------|-----------|
| Simple workflow | 14 lines, single job | **0.86 ms** |
| Complex workflow | Matrices, containers, dependencies | **0.88 ms** |
| 5 files combined | Batch processing, parallel validation | **0.89 ms** |

*Measured with Hyperfine (`--shell=none`, 100 runs) on the release binary. See [benchmarks/](#running-benchmarks).*

At sub-millisecond per file, Truss is fast enough to validate workflows as you type — zero perceptible lag in your editor.

## What It Catches

Truss ships with **41 validation rules** that go well beyond syntax checking. It validates job dependencies for circular references, checks that your `runs-on` labels are real GitHub-hosted runners, flags script injection risks, warns about deprecated workflow commands, verifies matrix strategies, validates cron expressions, and much more.

See the [full rule list](#validation-rules) below or check [docs/VALIDATION_RULES.md](docs/VALIDATION_RULES.md) for the details.

## Getting Started

### Prerequisites

- Rust 1.70+ ([rustup.rs](https://rustup.rs/))
- `just` (recommended) or `make`

### Install & Build

```bash
git clone https://github.com/JuanMarchetto/truss.git
cd truss
just build    # or: make build
just test     # or: make test
```

### Basic Usage

```bash
# Validate a workflow file
truss validate .github/workflows/ci.yml

# Validate everything in a directory
truss validate .github/workflows/

# Multiple files at once (parallel processing)
truss validate ci.yml deploy.yml release.yml

# Glob patterns work too
truss validate '.github/workflows/*.yml'

# Pipe from stdin
cat workflow.yml | truss validate -

# Only show errors (skip warnings)
truss validate --severity error ci.yml

# Machine-readable JSON output
truss validate --json ci.yml

# Quiet mode — just the exit code
truss validate --quiet ci.yml

# Version info
truss --version
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All files valid |
| 1 | Validation errors found |
| 2 | Bad arguments or no files given |
| 3 | I/O error (file not found, permission denied) |

### VS Code Extension

There's a VS Code extension in `editors/vscode/` that gives you real-time diagnostics as you type:

```bash
cargo build --release -p truss-lsp
cd editors/vscode
npm install && npm run compile
npx vsce package
code --install-extension truss-validator-0.1.0.vsix
```

It activates automatically on `.github/workflows/*.yml` files. See [editors/vscode/README.md](editors/vscode/README.md) for setup details.

### Other Editors

You can hook up `truss-lsp` to any editor that supports LSP:

```bash
./target/release/truss-lsp   # stdio transport
```

Point your editor's LSP client at this binary for `.github/workflows/*.yml` files. It supports incremental parsing, so re-validation after edits is near-instant.

## Validation Rules

41 rules across 5 categories:

### Core & Structural (4 rules)
| Rule | What it does |
|------|-------------|
| SyntaxRule | YAML syntax validation via tree-sitter |
| NonEmptyRule | Catches empty documents |
| GitHubActionsSchemaRule | Validates basic workflow structure |
| WorkflowTriggerRule | `on:` trigger config (30+ event types) |

### Job-Level (9 rules)
| Rule | What it does |
|------|-------------|
| JobNameRule | Duplicate names, invalid characters, reserved words |
| JobNeedsRule | Dependency validation, circular dependency detection |
| JobIfExpressionRule | Conditional expression validation |
| JobOutputsRule | Output reference validation |
| JobContainerRule | Container image, ports, services config |
| JobStrategyValidationRule | Strategy structure validation |
| RunsOnRequiredRule | Makes sure every job has `runs-on` |
| RunnerLabelRule | Validates GitHub-hosted runner labels (22+ labels) |
| ReusableWorkflowCallRule | Reusable workflow path and structure |

### Step-Level (11 rules)
| Rule | What it does |
|------|-------------|
| StepValidationRule | Step structure — must have `uses` or `run` (not both) |
| StepNameRule | Step name format validation |
| StepIdUniquenessRule | No duplicate step IDs within a job |
| StepIfExpressionRule | Step conditional expressions |
| StepOutputReferenceRule | `steps.X.outputs.Y` reference validation |
| StepContinueOnErrorRule | Boolean validation for `continue-on-error` |
| StepTimeoutRule | Timeout value validation |
| StepShellRule | Shell type validation (bash, pwsh, python, etc.) |
| StepWorkingDirectoryRule | Working directory path validation |
| StepEnvValidationRule | Env var names + reserved `GITHUB_` prefix detection |
| ArtifactValidationRule | upload/download-artifact parameter validation |

### Workflow-Level (9 rules)
| Rule | What it does |
|------|-------------|
| WorkflowNameRule | Workflow name validation |
| WorkflowInputsRule | Input types and requirements |
| WorkflowCallInputsRule | Reusable workflow call inputs |
| WorkflowCallSecretsRule | Secret definitions and references |
| WorkflowCallOutputsRule | Workflow call output validation |
| TimeoutRule | Job-level timeout validation |
| PermissionsRule | Permission scope validation (15+ scopes) |
| ConcurrencyRule | Concurrency groups and cancel-in-progress |
| DefaultsValidationRule | Default shell and working directory |

### Expression, Reference & Security (8 rules)
| Rule | What it does |
|------|-------------|
| ExpressionValidationRule | `${{ }}` syntax, functions, operators |
| ActionReferenceRule | `owner/repo@ref` format validation |
| EventPayloadValidationRule | Event fields, filter conflicts, cron ranges, activity types |
| SecretsValidationRule | Secret reference format and naming |
| MatrixStrategyRule | Matrix structure and key validation |
| EnvironmentRule | Environment name format |
| ScriptInjectionRule | Flags untrusted inputs used directly in `run:` blocks |
| DeprecatedCommandsRule | Warns about `::set-output`, `::set-env`, etc. |

## Performance

### CLI Benchmarks (Hyperfine)

End-to-end timing with `--shell=none` for accurate sub-millisecond measurement:

| Fixture | Mean | Min | Max |
|---------|------|-----|-----|
| Simple (14 lines) | **0.86 ms** | 0.75 ms | 1.05 ms |
| Complex (matrices, containers) | **0.88 ms** | 0.81 ms | 1.06 ms |
| Batch (5 files, parallel) | **0.89 ms** | 0.78 ms | 1.10 ms |

### Optimization History

Three rounds of performance optimization reduced end-to-end latency by **87%**:

| Tier | Key Changes | Simple | Complex | Batch (5) |
|------|-------------|--------|---------|-----------|
| Baseline | Initial release | 1.8 ms | 4.2 ms | 6.8 ms |
| Tier 1 | Cache workflow detection, skip rayon for single files | 1.6 ms | 3.9 ms | 5.9 ms |
| Tier 2 | Zero-copy `node_text()`, shared utilities, allocation elimination | 0.87 ms | 0.88 ms | 0.91 ms |
| Tier 3 | Borrowed collections, case-insensitive byte matching, cached lookups | **0.85 ms** | **0.88 ms** | **0.89 ms** |

Key techniques: zero-copy string slicing into source buffer (`&str` instead of `String`), shared `get_jobs_node()` and `clean_key()` utilities, `eq_ignore_ascii_case()` byte comparisons, borrowed `HashSet<&str>`/`HashMap<&str, Vec<&str>>` for dependency graphs, cached `find_value_for_key()` results, LTO + single codegen unit in release profile.

### Core Engine Benchmarks (Criterion)

Pure validation time (no I/O overhead):

| Fixture | Mean | Description |
|---------|------|-------------|
| Simple | 245 us | Minimal workflow |
| Medium | 1.08 ms | Multi-step with branching |
| Complex static | 3.71 ms | Matrix strategies, dependencies, containers |
| Complex dynamic | 2.48 ms | Expressions, reusable calls, dynamic matrices |

### Running Benchmarks

```bash
just bench          # Criterion (core engine)
just bench-cli      # Hyperfine (CLI end-to-end)
just compare        # Compare against competitors (requires actionlint, yamllint)
```

## Project Structure

```
truss/
├── crates/
│   ├── truss-core/      # Validation engine — editor-agnostic, deterministic
│   │   ├── lib.rs        # Engine with 41 registered rules
│   │   ├── parser.rs     # tree-sitter YAML parser (incremental)
│   │   ├── validation/   # 41 rule implementations
│   │   ├── tests/        # 44 test files, 347 tests
│   │   └── benches/      # Criterion benchmarks
│   ├── truss-cli/        # CLI — parallel processing, globs, stdin, JSON output
│   ├── truss-lsp/        # Language Server Protocol adapter
│   └── truss-wasm/       # WebAssembly bindings (placeholder)
├── editors/
│   └── vscode/           # VS Code extension
├── benchmarks/           # Fixtures and Hyperfine results
├── competitors/          # Comparison scripts for actionlint, yamllint, etc.
├── test-suite/           # Multi-repo comparison testing framework
├── scripts/              # Build, test, and comparison automation
├── docs/                 # Architecture, rules, test strategy docs
└── .github/workflows/    # CI pipeline
```

## Building & Testing

```bash
# Debug build
just build-debug      # or: cargo build --workspace

# Release build
just build            # or: cargo build --workspace --release

# Run all 347 tests
just test             # or: cargo test --workspace

# Core tests only
just test-core        # or: cargo test -p truss-core
```

### CI

Every push to `main` and every PR runs:
- `cargo check --workspace`
- `cargo test --workspace` (347 tests)
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --all -- --check`

## Architecture

Truss follows a "core first" design. All validation logic lives in `truss-core`, which is editor-agnostic and fully deterministic. The CLI, LSP server, and WASM crate are thin adapters that wrap the core for different interfaces.

Key ideas:
- **Performance is a feature**, not an afterthought — everything is benchmarked
- **Rules are stateless** — each rule gets the parsed tree and source, returns diagnostics
- **Results are deterministic** — same input always produces the same output
- **Incremental parsing** — the LSP server only re-parses what changed

More details in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Current Status

**What's working:**
- 41 validation rules, all tested (347 tests across 44 test files)
- LSP server with real-time diagnostics and incremental parsing
- VS Code extension
- CLI with parallel file processing, globs, stdin, severity filtering, JSON output
- Sub-1ms validation per file (release build, 87% faster than initial release)
- CI pipeline (check, test, clippy, fmt)

**Coming next:**
- Contextual autocomplete
- WASM bindings
- `cargo install truss-cli` (crates.io)
- Neovim and other editor integrations

**Out of scope (for now):**
- Azure Pipelines / GitLab CI
- Advanced UI
- Complex configuration

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — Design principles and guidelines
- [Validation Rules](docs/VALIDATION_RULES.md) — All 41 rules in detail
- [Test Strategy](docs/TEST_STRATEGY.md) — How we test
- [Planned Improvements](docs/PLANNED_IMPROVEMENTS.md) — What's on the roadmap

## Contributing

Contributions are welcome! See the [Contributing Guide](CONTRIBUTING.md) to get started.

## License

MIT. See [LICENSE-MIT](LICENSE-MIT).

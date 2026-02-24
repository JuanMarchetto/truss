# Truss

> **Heads up:** This project is still early. A good chunk of the code was AI-generated and is being actively reviewed, tested, and improved. All tests pass and the benchmarks are reproducible, but treat this as experimental for now. Bug reports and contributions are very welcome as we work toward a stable release.

A fast GitHub Actions workflow validator written in Rust. Truss catches configuration errors, semantic issues, and common mistakes in your CI/CD pipelines before you push — so you spend less time debugging failed runs.

## Why Truss?

**It's fast.** Like, really fast. 15-35x faster than the alternatives:

| Tool | Language | Mean Time | vs Truss |
|------|----------|-----------|----------|
| **Truss** | Rust | **11.1ms** | baseline |
| actionlint | Go | 165.7ms | 15x slower |
| yamllint | Python | 210.9ms | 19x slower |
| yaml-language-server | TypeScript | 381.7ms | 35x slower |

*Measured with Hyperfine on `complex-dynamic.yml`. See [benchmarks/hyperfine/compare.md](benchmarks/hyperfine/compare.md).*

At 11ms, Truss is fast enough to validate workflows as you type — no perceptible lag in your editor.

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

End-to-end timing of `truss validate` vs competitors on the complex dynamic workflow fixture:

| Tool | Mean | Min | Max | Relative |
|------|------|-----|-----|----------|
| **Truss** | **11.1ms** | 1.7ms | 17.9ms | 1.00x |
| actionlint | 165.7ms | 144.5ms | 189.7ms | 14.98x |
| yaml-language-server | 381.7ms | 263.9ms | 553.4ms | 34.51x |
| yamllint | 210.9ms | 94.6ms | 276.7ms | 19.07x |

### Core Engine Benchmarks (Criterion)

| Fixture | Mean | Description |
|---------|------|-------------|
| Simple | 225 us | Minimal workflow |
| Medium | 984 us | Multi-step with branching |
| Complex static | 3.66 ms | Matrix strategies, dependencies, containers |
| Complex dynamic | 2.44 ms | Expressions, reusable calls, dynamic matrices |

### Running Benchmarks

```bash
just bench          # Criterion (core engine)
just bench-cli      # Hyperfine (CLI end-to-end)
just compare        # Compare against competitors
```

## Project Structure

```
truss/
├── crates/
│   ├── truss-core/      # Validation engine — editor-agnostic, deterministic
│   │   ├── lib.rs        # Engine with 41 registered rules
│   │   ├── parser.rs     # tree-sitter YAML parser (incremental)
│   │   ├── validation/   # 41 rule implementations
│   │   ├── tests/        # 44 test files, 346 tests
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

# Run all 346 tests
just test             # or: cargo test --workspace

# Core tests only
just test-core        # or: cargo test -p truss-core
```

### CI

Every push to `main` and every PR runs:
- `cargo check --workspace`
- `cargo test --workspace` (346 tests)
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
- 41 validation rules, all tested (346 tests across 44 files)
- LSP server with real-time diagnostics and incremental parsing
- VS Code extension
- CLI with parallel file processing, globs, stdin, severity filtering, JSON output
- 15-35x faster than alternatives
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

# Truss

[![CI](https://github.com/JuanMarchetto/truss/actions/workflows/ci.yml/badge.svg)](https://github.com/JuanMarchetto/truss/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

> **Heads up:** This project is still early. A good chunk of the code was AI-generated and is being actively reviewed, tested, and improved. All tests pass and the benchmarks are reproducible, but treat this as experimental for now. Bug reports and contributions are very welcome as we work toward a stable release.

A fast GitHub Actions workflow validator written in Rust. Truss catches configuration errors, semantic issues, and common mistakes in your CI/CD pipelines before you push — so you spend less time debugging failed runs.

## Why Truss?

**It's fast and accurate.** We tested Truss, [actionlint](https://github.com/rhysd/actionlint), [yamllint](https://github.com/adrienverge/yamllint), and [yaml-language-server](https://github.com/redhat-developer/yaml-language-server) against **271 production workflow files** from 7 major open-source projects ([pytorch/pytorch](https://github.com/pytorch/pytorch), [rust-lang/rust](https://github.com/rust-lang/rust), [tensorflow/tensorflow](https://github.com/tensorflow/tensorflow), [facebook/react](https://github.com/facebook/react), [microsoft/TypeScript](https://github.com/microsoft/TypeScript), [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes), [actions/checkout](https://github.com/actions/checkout)):

| Tool | Language | Errors | False Positives | Avg Time | Total Time |
|------|----------|--------|-----------------|----------|------------|
| **Truss** | Rust | **0** | **0** | **1.7 ms** | **471 ms** |
| actionlint | Go | 0 | 0 | 6.8 ms | 1,840 ms |
| yaml-language-server | TypeScript | 0 | 0 | 101 ms | 27,277 ms |
| yamllint | Python | 0 | n/a (style only) | 123 ms | 33,453 ms |

All tools reported **zero errors** on 271 real-world files — a clean bill of health across the board. The key differentiator is speed: Truss processes each file in ~1.7ms on average, making it **3.9x faster** than actionlint, **58x faster** than yaml-language-server, and **71x faster** than yamllint.

*Measured on an Intel i5-7500T @ 2.70GHz, 8GB RAM, Linux 6.17. Your results may vary. See [test-suite/](#real-world-validation) for how to reproduce.*

## What It Catches

Truss ships with **41 validation rules** that go well beyond syntax checking. It validates job dependencies for circular references, checks that your `runs-on` labels are real GitHub-hosted runners, flags script injection risks, warns about deprecated workflow commands, verifies matrix strategies, validates cron expressions, and much more.

See the [full rule list](#validation-rules) below or check [docs/VALIDATION_RULES.md](docs/VALIDATION_RULES.md) for the details.

## How It Compares

### Feature Comparison

| Feature | Truss | [actionlint](https://github.com/rhysd/actionlint) | [yamllint](https://github.com/adrienverge/yamllint) | [yaml-language-server](https://github.com/redhat-developer/yaml-language-server) |
|---------|-------|------------|---------|----------------------|
| **Language** | Rust | Go | Python | TypeScript |
| **YAML syntax** | tree-sitter | Custom | Yes | JSON Schema |
| **GHA semantic validation** | 41 rules | Yes | No | Partial (schema) |
| **Expression validation** | `${{ }}` syntax, functions, operators | Strong type-checking | No | No |
| **Runner label checks** | 22+ labels | Yes | No | No |
| **Matrix validation** | Structure + keys | Structure + types | No | No |
| **Reusable workflows** | Inputs, outputs, secrets | Inputs, outputs, secrets | No | No |
| **Job dependency cycles** | Yes | Yes | No | No |
| **Script injection detection** | Yes | Yes (+ shellcheck) | No | No |
| **Cron validation** | Yes | Yes | No | No |
| **Action reference format** | Yes (incl. subpaths) | Yes | No | No |
| **LSP server** | Yes (incremental) | No ([requested](https://github.com/rhysd/actionlint/issues/229)) | No | Yes |
| **VS Code extension** | Yes | Yes (2 extensions) | No | Yes |
| **Avg time per file** | **1.7 ms** | 6.8 ms | 123 ms | 101 ms |

Other tools in the ecosystem: [zizmor](https://github.com/zizmorcore/zizmor) (Rust, security-focused, 24 audit rules), [action-validator](https://github.com/mpalmer/action-validator) (Rust, schema-based), [ghalint](https://github.com/suzuki-shunsuke/ghalint) (Go, security policies). These are complementary — they focus on security auditing or schema validation rather than semantic correctness.

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

### Real-World Validation

We cloned 7 major open-source repositories and ran every tool against all **271 workflow files** found across [pytorch/pytorch](https://github.com/pytorch/pytorch) (139 files), [rust-lang/rust](https://github.com/rust-lang/rust) (50), [tensorflow/tensorflow](https://github.com/tensorflow/tensorflow) (33), [facebook/react](https://github.com/facebook/react) (24), [microsoft/TypeScript](https://github.com/microsoft/TypeScript) (18), [actions/checkout](https://github.com/actions/checkout) (7):

| Tool | Errors | False Positives | Avg Time | Total Time | Speedup |
|------|--------|-----------------|----------|------------|---------|
| **Truss** | **0** | **0** | **1.7 ms** | **471 ms** | — |
| actionlint | 0 | 0 | 6.8 ms | 1,840 ms | Truss is 3.9x faster |
| yaml-language-server | 0 | 0 | 101 ms | 27,277 ms | Truss is 58x faster |
| yamllint | 0 | n/a | 123 ms | 33,453 ms | Truss is 71x faster |

All four tools report zero errors on these 271 files — the test repos' workflows are well-formed. The differentiator here is pure speed: Truss validates each file in about 1.7ms, while actionlint takes 6.8ms, and the Python/TypeScript-based tools take 101-123ms.

To reproduce: `cd test-suite && bash scripts/setup-test-repos.sh && bash scripts/run-full-suite.sh`

### CLI Benchmarks (Hyperfine)

End-to-end timing of `truss validate --quiet` with `--shell=none`, 200 runs:

| Fixture | Mean | Min | Max |
|---------|------|-----|-----|
| Simple (14 lines) | **1.7 ms** | 1.4 ms | 4.4 ms |
| Medium (multi-step) | **2.4 ms** | 2.1 ms | 5.9 ms |
| Complex dynamic (reusable calls) | **3.7 ms** | 3.3 ms | 5.1 ms |
| Complex static (matrices) | **4.9 ms** | 4.4 ms | 6.3 ms |
| All 4 files (directory scan) | **5.7 ms** | 5.0 ms | 7.8 ms |

### Head-to-Head: Truss vs. actionlint vs. yamllint

All tools benchmarked on the same machine (Intel i5-7500T @ 2.70GHz, 8GB RAM, Linux 6.17) with Hyperfine (`--shell=none`, 200 runs, `--warmup 10`). This is one particular benchmark on one particular machine — your results may vary.

| Fixture | Truss (Rust) | actionlint (Go) | yamllint (Python) |
|---------|-------------|-----------------|-------------------|
| Simple | **1.7 ms** | 2.6 ms | 103 ms |
| Complex dynamic | **3.7 ms** | 3.7 ms | 140 ms |
| Complex static | 4.9 ms | **4.0 ms** | 153 ms |

On individual fixtures, Truss and actionlint are in the same performance class (single-digit milliseconds). Truss is faster on simple files; actionlint is faster on complex ones. Both are 30-60x faster than yamllint. On larger batches (271 real-world files), Truss averages 1.7ms/file vs. actionlint's 6.8ms/file — a **3.9x advantage** that comes from Truss's lower per-file overhead, LTO-optimized binary, and parallel rule execution.

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
│   │   ├── tests/        # 45 test files, 403 tests
│   │   └── benches/      # Criterion benchmarks
│   ├── truss-cli/        # CLI — parallel processing, globs, stdin, JSON output
│   ├── truss-lsp/        # Language Server Protocol adapter
│   └── truss-wasm/       # WebAssembly bindings
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

# Run all 403 tests
just test             # or: cargo test --workspace

# Core tests only
just test-core        # or: cargo test -p truss-core
```

### CI

Every push to `main` and every PR runs:
- `cargo check --workspace`
- `cargo test --workspace` (403 tests)
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
- 41 validation rules with unique rule IDs, all tested (403 tests across 45 test files)
- Zero false positives on 271 production workflow files from pytorch, rust-lang, tensorflow, react, TypeScript, kubernetes, and checkout
- LSP server with real-time diagnostics and incremental parsing
- VS Code extension
- CLI with parallel file processing, globs, stdin, severity filtering, rule filtering (`--ignore-rules`, `--only-rules`), JSON output
- `.truss.yml` configuration file support (ignore paths, enable/disable rules per project)
- Sub-6ms validation per file, 3.9x faster than actionlint on real-world batches
- WASM bindings and online playground
- CI pipeline (check, test, clippy, fmt)

**Coming next:**
- Contextual autocomplete
- `cargo install truss-cli` (crates.io)
- Neovim and other editor integrations

**Out of scope (for now):**
- Azure Pipelines / GitLab CI
- Advanced UI

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — Design principles and guidelines
- [Validation Rules](docs/VALIDATION_RULES.md) — All 41 rules in detail
- [Test Strategy](docs/TEST_STRATEGY.md) — How we test
- [Planned Improvements](docs/PLANNED_IMPROVEMENTS.md) — What's on the roadmap

## Contributing

Contributions are welcome! See the [Contributing Guide](CONTRIBUTING.md) to get started.

## License

MIT. See [LICENSE-MIT](LICENSE-MIT).

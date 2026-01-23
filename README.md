# Truss

High-performance CI/CD pipeline validation and analysis engine written in Rust. Truss provides real-time feedback, high performance, and reproducible results by detecting configuration errors, semantic inconsistencies, and autocomplete opportunities before a pipeline is executed.

## 🚀 Performance Highlights

**Truss is 15-35x faster than existing GitHub Actions validation tools:**

| Tool | Language | Mean Time | Relative Speed |
|------|----------|-----------|----------------|
| **Truss** | Rust | **11.1ms** | 1.00x (baseline) |
| actionlint | Go | 165.7ms | **14.98x slower** |
| yaml-language-server | TypeScript | 381.7ms | **34.51x slower** |
| yamllint | Python | 210.9ms | **19.07x slower** |

*Benchmark: Complex dynamic workflow validation. See [Performance](#performance) section for details.*

**Why this matters:** 11.1ms is fast enough for real-time LSP integration, enabling instant diagnostics as you type in your editor.

## Features

- **High Performance**: 15-35x faster than competitors (11.1ms average for complex workflows)
- **Semantic Validation**: Goes beyond syntax checking to validate semantic correctness
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

# Validate multiple files
./target/release/truss validate file1.yml file2.yml

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
- Diagnostics for all validation rules

To use with your editor, configure it to use `truss-lsp` as the language server for YAML files (or specifically `.github/workflows/*.yml` files).

## Showcase & Demo

### Quick Demo

```bash
# Validate a workflow file
./target/release/truss validate .github/workflows/ci.yml

# Validate multiple files in parallel
./target/release/truss validate workflows/*.yml
```

### LSP Integration

Truss LSP provides real-time diagnostics in your editor:
- Instant validation as you type
- Precise error locations with helpful messages
- Support for all 12 validation rules
- Incremental parsing for optimal performance

### Performance Comparison

See the [Performance](#performance) section below for detailed benchmark results, or check out:
- [Performance Comparison Guide](docs/PERFORMANCE_COMPARISON.md) - Detailed metrics and visualization data
- [Showcase Guide](docs/SHOWCASE.md) - Demo scripts, talking points, and presentation materials

### Validation Rules

Truss implements **12 comprehensive validation rules**:
- ✅ Syntax validation
- ✅ Workflow trigger validation
- ✅ Job name and dependency validation
- ✅ Step structure validation
- ✅ Expression validation
- ✅ Permissions validation
- ✅ Environment validation
- ✅ Matrix strategy validation
- And more...

See [Validation Rules Documentation](docs/VALIDATION_RULES.md) for complete details.

## Performance

Truss is designed for speed and efficiency, making it ideal for real-time editor integration and CI/CD pipelines.

### Benchmark Results

**Complex Workflow Validation** (`benchmarks/fixtures/complex-dynamic.yml`):

| Tool | Mean Time | Min | Max | Relative to Truss |
|------|-----------|-----|-----|-------------------|
| **Truss** | **11.1ms** | 1.7ms | 17.9ms | 1.00x |
| actionlint | 165.7ms | 144.5ms | 189.7ms | 14.98x slower |
| yaml-language-server | 381.7ms | 263.9ms | 553.4ms | 34.51x slower |
| yamllint | 210.9ms | 94.6ms | 276.7ms | 19.07x slower |

**Core Engine Performance** (Criterion benchmarks):
- **Simple YAML**: < 1ms
- **Medium YAML**: ~2ms
- **Complex static workflow**: ~6.8ms
- **Complex dynamic workflow**: ~4.3ms

### Why Performance Matters

Truss achieves **15-35x better performance** than existing tools while providing comprehensive semantic validation. This performance improvement enables:

- ✅ **Real-time editor feedback** - LSP integration with instant diagnostics (11.1ms is fast enough)
- ✅ **Large-scale validation** - Process hundreds of workflow files quickly
- ✅ **CI/CD integration** - Fast validation doesn't slow down pipelines
- ✅ **Better developer experience** - Instant feedback improves productivity

### Running Benchmarks

```bash
# Compare with competitors
just compare

# Run CLI benchmarks
just bench-cli

# Run core engine benchmarks
just bench
```

See [benchmarks/hyperfine/compare.md](benchmarks/hyperfine/compare.md) for detailed results.

## Project Structure

```
truss/
├── crates/
│   ├── truss-core/      # Core validation engine (editor-agnostic)
│   ├── truss-cli/       # Command-line interface
│   ├── truss-lsp/       # Language Server Protocol adapter
│   └── truss-wasm/      # WebAssembly bindings (placeholder)
├── benchmarks/          # Benchmark fixtures and results
├── docs/                # Documentation
│   └── ARCHITECTURE.md  # Architecture and design guidelines
└── competitors/         # Comparison benchmarks
```

## Build Systems

Truss provides two build systems for flexibility:

### `justfile` (Recommended)

The primary build system with enhanced features:
- Colored output
- Additional commands (`bench-cli`, `compare-smoke`, `test-core`)
- Better developer experience

**Installation:** `cargo install just` or see [just's documentation](https://github.com/casey/just)

**Usage:**
```bash
just build          # Build release
just test           # Run tests
just bench          # Run benchmarks
just compare        # Compare with competitors
just ci             # Full CI pipeline
```

### `makefile` (Fallback)

A minimal fallback for environments without `just`:
- Universal availability
- Basic commands only
- Compatible with all Unix-like systems

**Usage:**
```bash
make build          # Build release
make test           # Run tests
make bench          # Run benchmarks
make compare        # Compare with competitors
make ci             # Full CI pipeline
```

**Note:** For the best experience, use `justfile`. The `makefile` is provided as a fallback for environments where `just` is not available.

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
# Run all tests
just test
# or: cargo test --workspace

# Run core tests only
just test-core
# or: cargo test -p truss-core
```

### Benchmarking

```bash
# Rust/Criterion benchmarks
just bench
# or: cargo bench -p truss-core

# CLI/Hyperfine benchmarks
just bench-cli

# Compare with competitors
just compare
```

## Architecture

Truss follows a strict "Core First" architecture:

- **truss-core**: All critical logic lives here. Editor-agnostic and fully deterministic.
- **Adapters** (truss-cli, truss-lsp, truss-wasm): Thin layers that adapt the core to different interfaces.

For detailed architecture information, design principles, and development guidelines, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Current Status

**✅ MVP Complete:**
- ✅ GitHub Actions workflow validation
- ✅ **12 validation rules** implemented and tested (63+ tests)
- ✅ LSP server with real-time diagnostics
- ✅ CLI tool with parallel file processing
- ✅ Incremental parsing support
- ✅ High-performance validation (**15-35x faster** than competitors)
- ✅ Comprehensive benchmarking infrastructure
- ✅ Clean architecture (Core + adapters)

**📋 Planned:**
- Contextual autocomplete
- WASM bindings (structure in place)

**🚫 Not Included (for now):**
- Azure Pipelines support
- Advanced UI
- Complex configuration
- Monetization

## Documentation

- [Architecture Guide](docs/ARCHITECTURE.md) - Design principles and guidelines
- [Validation Rules](docs/VALIDATION_RULES.md) - Complete list of validation rules
- [Test Strategy](docs/TEST_STRATEGY.md) - Testing approach and organization
- [Showcase Guide](docs/SHOWCASE.md) - Demo materials and presentation resources
- [Performance Comparison](docs/PERFORMANCE_COMPARISON.md) - Detailed benchmark data

## Contributing

1. Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) to understand design principles
2. Ensure all tests pass: `just test`
3. Run benchmarks to verify performance: `just bench`
4. Follow the "Core First" principle: business logic belongs in `truss-core`

## License

Licensed under either of:
- MIT License ([LICENSE-MIT](LICENSE-MIT))



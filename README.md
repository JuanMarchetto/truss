# Truss

High-performance CI/CD pipeline validation and analysis engine written in Rust. Truss provides real-time feedback, high performance, and reproducible results by detecting configuration errors, semantic inconsistencies, and autocomplete opportunities before a pipeline is executed.

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

## Performance

Truss is designed for speed and efficiency, making it ideal for real-time editor integration and CI/CD pipelines.

### Benchmark Results

**Complex Workflow Validation** (complex-dynamic.yml):
- **Truss**: 11.1ms average (1.7ms - 17.9ms range)
- **actionlint**: 165.7ms (14.98x slower)
- **yaml-language-server**: 381.7ms (34.51x slower)
- **yamllint**: 210.9ms (19.07x slower)

**Core Engine Performance** (Criterion benchmarks):
- Simple YAML: < 1ms
- Medium YAML: ~2ms
- Complex static workflow: ~6.8ms
- Complex dynamic workflow: ~4.3ms

Truss achieves **15-35x better performance** than existing tools while providing comprehensive semantic validation. This makes it suitable for:
- Real-time editor feedback (LSP integration)
- Large-scale CI/CD pipeline validation
- High-frequency validation in automated workflows

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

**MVP Scope:**
- GitHub Actions workflow validation
- Semantic validation (12 validation rules implemented)
- LSP server with real-time diagnostics
- Incremental parsing support
- High-performance validation (15-35x faster than competitors)
- Contextual autocomplete (planned)

**Not Included (for now):**
- Azure Pipelines
- Advanced UI
- Complex configuration
- Monetization

## Contributing

1. Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) to understand design principles
2. Ensure all tests pass: `just test`
3. Run benchmarks to verify performance: `just bench`
4. Follow the "Core First" principle: business logic belongs in `truss-core`

## License

Licensed under either of:
- MIT License ([LICENSE-MIT](LICENSE-MIT))



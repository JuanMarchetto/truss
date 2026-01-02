# Truss

High-performance CI/CD pipeline validation and analysis engine written in Rust. Truss provides real-time feedback, high performance, and reproducible results by detecting configuration errors, semantic inconsistencies, and autocomplete opportunities before a pipeline is executed.

## Features

- **High Performance**: Optimized for speed and scalability
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

```bash
# Validate a GitHub Actions workflow file
./target/release/truss validate path/to/workflow.yml

# Analyze a workflow file (legacy syntax)
./target/release/truss path/to/workflow.yml
```

## Project Structure

```
truss/
├── crates/
│   ├── truss-core/      # Core validation engine (editor-agnostic)
│   ├── truss-cli/       # Command-line interface
│   ├── truss-lsp/       # Language Server Protocol adapter (placeholder)
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
- Basic semantic validation
- Contextual autocomplete (planned)
- Incremental parsing (planned)

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



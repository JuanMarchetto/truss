//! Golden file regression tests for benchmark fixtures.
//!
//! These tests run TrussEngine::analyze() against real-world GitHub Actions
//! workflow fixtures derived from major open-source projects. They serve as
//! regression guards to ensure we don't introduce false positives.
//!
//! Fixture provenance:
//! - simple.yml: actions/checkout (licensed.yml)
//! - medium.yml: Azure/azure-sdk-for-js (scheduled event processor)
//! - complex-static.yml: microsoft/TypeScript (CI with matrix strategies)
//! - complex-dynamic.yml: rust-lang/rust (CI with dynamic matrices)

use truss_core::{Severity, TrussEngine};

/// Load fixture content at compile time so tests fail fast if fixtures are missing.
const SIMPLE_YML: &str = include_str!("../../../benchmarks/fixtures/simple.yml");
const MEDIUM_YML: &str = include_str!("../../../benchmarks/fixtures/medium.yml");
const COMPLEX_STATIC_YML: &str = include_str!("../../../benchmarks/fixtures/complex-static.yml");
const COMPLEX_DYNAMIC_YML: &str = include_str!("../../../benchmarks/fixtures/complex-dynamic.yml");

// ---------------------------------------------------------------------------
// simple.yml — actions/checkout (licensed.yml)
// Single job, linear execution, baseline for correctness.
// ---------------------------------------------------------------------------

#[test]
fn fixture_simple_no_errors() {
    let mut engine = TrussEngine::new();
    let result = engine.analyze(SIMPLE_YML);

    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();

    assert!(
        errors.is_empty(),
        "simple.yml should produce zero errors. Got {} error(s):\n{}",
        errors.len(),
        errors
            .iter()
            .map(|d| format!("  - [ERROR] {}", d.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn fixture_simple_is_valid() {
    let mut engine = TrussEngine::new();
    let result = engine.analyze(SIMPLE_YML);
    assert!(
        result.is_ok(),
        "simple.yml should be valid (is_ok() == true)"
    );
}

// ---------------------------------------------------------------------------
// medium.yml — Azure/azure-sdk-for-js (scheduled event processor)
// Schedule triggers with multiple cron entries and comments between them.
// Previously caused false positive: "schedule event missing 'cron' field"
// ---------------------------------------------------------------------------

#[test]
fn fixture_medium_no_errors() {
    let mut engine = TrussEngine::new();
    let result = engine.analyze(MEDIUM_YML);

    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();

    assert!(
        errors.is_empty(),
        "medium.yml should produce zero errors after Phase 1 fixes. Got {} error(s):\n{}",
        errors.len(),
        errors
            .iter()
            .map(|d| format!(
                "  - [ERROR] {} ({}..{})",
                d.message, d.span.start, d.span.end
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn fixture_medium_is_valid() {
    let mut engine = TrussEngine::new();
    let result = engine.analyze(MEDIUM_YML);
    assert!(
        result.is_ok(),
        "medium.yml should be valid (is_ok() == true)"
    );
}

// ---------------------------------------------------------------------------
// complex-static.yml — microsoft/TypeScript
// Matrix strategies, cross-job dependencies, bare if: expressions,
// self-hosted runner arrays. Previously caused 3 false positives:
// 1. Matrix key 'config' value parsed as comment text
// 2. Bare if: expression flagged as missing ${{ }} wrapper
// 3. Self-hosted runner array flagged as unknown runner label
// ---------------------------------------------------------------------------

#[test]
fn fixture_complex_static_no_errors() {
    let mut engine = TrussEngine::new();
    let result = engine.analyze(COMPLEX_STATIC_YML);

    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();

    assert!(
        errors.is_empty(),
        "complex-static.yml should produce zero errors after Phase 1 fixes. Got {} error(s):\n{}",
        errors.len(),
        errors
            .iter()
            .map(|d| format!(
                "  - [ERROR] {} ({}..{})",
                d.message, d.span.start, d.span.end
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn fixture_complex_static_is_valid() {
    let mut engine = TrussEngine::new();
    let result = engine.analyze(COMPLEX_STATIC_YML);
    assert!(
        result.is_ok(),
        "complex-static.yml should be valid (is_ok() == true). Errors: {:?}",
        result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// complex-dynamic.yml — rust-lang/rust
// Dynamic matrices via fromJSON(), concurrency with comments,
// expressions in YAML comments. Previously caused 3 false positives:
// 1. "Concurrency missing 'group' field" (comments before group key)
// 2. "Invalid expression syntax: 'github'" (expression in YAML comment)
// 3. "Unknown function 'fromJSON'" (missing from valid functions list)
// ---------------------------------------------------------------------------

#[test]
fn fixture_complex_dynamic_no_errors() {
    let mut engine = TrussEngine::new();
    let result = engine.analyze(COMPLEX_DYNAMIC_YML);

    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();

    assert!(
        errors.is_empty(),
        "complex-dynamic.yml should produce zero errors after Phase 1 fixes. Got {} error(s):\n{}",
        errors.len(),
        errors
            .iter()
            .map(|d| format!(
                "  - [ERROR] {} ({}..{})",
                d.message, d.span.start, d.span.end
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn fixture_complex_dynamic_is_valid() {
    let mut engine = TrussEngine::new();
    let result = engine.analyze(COMPLEX_DYNAMIC_YML);
    assert!(
        result.is_ok(),
        "complex-dynamic.yml should be valid (is_ok() == true). Errors: {:?}",
        result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Cross-fixture: Determinism tests
// Ensure analyzing the same fixture twice produces identical diagnostics.
// ---------------------------------------------------------------------------

#[test]
fn fixture_analysis_is_deterministic() {
    let mut engine = TrussEngine::new();

    let fixtures = [
        ("simple.yml", SIMPLE_YML),
        ("medium.yml", MEDIUM_YML),
        ("complex-static.yml", COMPLEX_STATIC_YML),
        ("complex-dynamic.yml", COMPLEX_DYNAMIC_YML),
    ];

    for (name, content) in &fixtures {
        let result_a = engine.analyze(content);
        let result_b = engine.analyze(content);

        assert_eq!(
            result_a.diagnostics.len(),
            result_b.diagnostics.len(),
            "Analyzing {} twice should produce the same number of diagnostics ({} vs {})",
            name,
            result_a.diagnostics.len(),
            result_b.diagnostics.len()
        );

        for (i, (a, b)) in result_a
            .diagnostics
            .iter()
            .zip(result_b.diagnostics.iter())
            .enumerate()
        {
            assert_eq!(
                a.message, b.message,
                "Diagnostic #{} for {} differs between runs: '{}' vs '{}'",
                i, name, a.message, b.message
            );
            assert_eq!(
                a.severity, b.severity,
                "Severity of diagnostic #{} for {} differs between runs",
                i, name
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Cross-fixture: All fixtures diagnostic summary
// A single test that runs all fixtures and prints a summary.
// Useful for debugging when individual tests pass but coverage is unclear.
// ---------------------------------------------------------------------------

#[test]
fn fixture_all_diagnostic_summary() {
    let mut engine = TrussEngine::new();

    let fixtures = [
        ("simple.yml", SIMPLE_YML),
        ("medium.yml", MEDIUM_YML),
        ("complex-static.yml", COMPLEX_STATIC_YML),
        ("complex-dynamic.yml", COMPLEX_DYNAMIC_YML),
    ];

    let mut total_errors = 0;

    for (name, content) in &fixtures {
        let result = engine.analyze(content);
        let errors = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let warnings = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();
        total_errors += errors;

        // Print summary for each fixture (visible with `cargo test -- --nocapture`)
        eprintln!(
            "[{}] {} diagnostics ({} errors, {} warnings)",
            name,
            result.diagnostics.len(),
            errors,
            warnings
        );
        for d in &result.diagnostics {
            eprintln!(
                "  [{:?}] {} ({}..{})",
                d.severity, d.message, d.span.start, d.span.end
            );
        }
    }

    assert_eq!(
        total_errors, 0,
        "All benchmark fixtures combined should produce 0 errors, but got {}",
        total_errors
    );
}

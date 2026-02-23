# Test Strategy for Validation Rules

## Overview

This document outlines the TDD approach for implementing validation rules in Truss.

## Test Organization

### Unit Tests (Same File)
**Location:** `crates/truss-core/validation.rs` (in `#[cfg(test)]` module)

**Purpose:**
- Test individual rule logic
- Test private helper functions
- Fast, isolated tests
- Can access private implementation details

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rule_specific_behavior() {
        let rule = MyRule;
        // Test rule directly
    }
}
```

### Integration Tests (Separate Files)
**Location:** `crates/truss-core/tests/validation_*.rs`

**Purpose:**
- Test rules through public API (`TrussEngine`)
- Test rule interactions
- Test with real YAML fixtures
- Verify end-to-end behavior

**Naming Convention:**
- `validation_syntax.rs` - Tests for SyntaxRule
- `validation_non_empty.rs` - Tests for NonEmptyRule
- `validation_schema.rs` - Tests for GitHubActionsSchemaRule
- `validation_job_name.rs` - Tests for JobNameRule
- `validation_job_needs.rs` - Tests for JobNeedsRule
- `validation_step.rs` - Tests for StepValidationRule
- etc.

### Test Structure
```
crates/truss-core/
├── lib.rs                         (unit tests in #[cfg(test)] module)
├── validation/                    (rules organized by module)
│   ├── mod.rs
│   └── rules/
│       ├── syntax.rs
│       ├── non_empty.rs
│       ├── schema.rs
│       └── ... (all 19 rules)
└── tests/
    ├── validation_syntax.rs       (SyntaxRule tests)
    ├── validation_non_empty.rs    (NonEmptyRule tests)
    ├── validation_schema.rs       (GitHubActionsSchemaRule tests)
    ├── validation_workflow_trigger.rs
    ├── validation_job_name.rs
    ├── validation_job_needs.rs
    ├── validation_step.rs
    ├── validation_expression.rs
    ├── validation_permissions.rs
    ├── validation_environment.rs
    ├── validation_workflow_name.rs
    ├── validation_matrix.rs
    ├── validation_runs_on.rs
    ├── validation_secrets.rs
    ├── validation_timeout.rs
    ├── validation_workflow_inputs.rs
    ├── validation_job_outputs.rs
    ├── validation_concurrency.rs
    └── validation_action_reference.rs
```

## Test Structure for Each Rule

Each validation rule should have tests covering:

### 1. Valid Cases ✅
Tests that should pass without errors:
```rust
#[test]
fn test_rule_valid_cases() {
    let mut engine = TrussEngine::new();
    let valid_inputs = vec![
        "valid: yaml",
        "another: valid",
    ];
    
    for input in valid_inputs {
        let result = engine.analyze(input);
        // Assert no errors for this rule
    }
}
```

### 2. Error Cases ❌
Tests that should produce specific diagnostics:
```rust
#[test]
#[ignore = "RuleName not yet implemented"]
fn test_rule_error_cases() {
    let mut engine = TrussEngine::new();
    let invalid_input = "invalid: yaml";
    
    let result = engine.analyze(invalid_input);
    // Assert specific error is produced
}
```

### 3. Edge Cases 🔍
Boundary conditions and special cases:
```rust
#[test]
fn test_rule_edge_cases() {
    // Test empty, very long, special characters, etc.
}
```

### 4. Determinism 🔄
Verify rule produces consistent results:
```rust
#[test]
fn test_rule_deterministic() {
    let mut engine = TrussEngine::new();
    let input = "test: input";
    
    let result1 = engine.analyze(input);
    let result2 = engine.analyze(input);
    
    assert_eq!(result1.diagnostics.len(), result2.diagnostics.len());
}
```

## TDD Workflow

### Step 1: Write Tests First
1. Identify the validation rule to implement
2. Write comprehensive tests covering all cases
3. Mark error-case tests with `#[ignore = "RuleName not yet implemented"]`
4. Run tests (valid cases pass, ignored tests skipped)

### Step 2: Implement Rule
1. Implement the rule to make tests pass
2. Remove `#[ignore]` from tests
3. Run tests (they should pass - Green)

### Step 3: Refactor
1. Improve code quality
2. Ensure tests still pass
3. Add more edge cases if needed

## Best Practices

1. **One test file per rule** - Easy to find and maintain
2. **Descriptive test names** - `test_rule_name_what_it_tests`
3. **Test through public API** - Integration tests use `TrussEngine`
4. **Isolate test cases** - Each test should be independent
5. **Test determinism** - Rules must produce consistent results
6. **Test error messages** - Verify messages are clear and actionable
7. **Use `#[ignore]`** - Mark tests for unimplemented rules

## Running Tests

```bash
# Run all tests (unit + integration, ignored tests skipped)
cargo test -p truss-core

# Run including ignored tests (shows pending work)
cargo test -p truss-core -- --include-ignored

# Run specific test file
cargo test -p truss-core --test validation_syntax

# Run with output
cargo test -p truss-core -- --nocapture

# Use justfile commands
just test-core
just test-validation
```

## Continuous Integration

All tests must pass before merging:
- Unit tests in `lib.rs` and `validation.rs`
- Integration tests in `tests/`
- Ignored tests are skipped (CI passes)
- Benchmarks should not regress

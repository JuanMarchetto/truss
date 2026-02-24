# Test Strategy for Validation Rules

## Overview

Truss follows a test-driven development (TDD) approach for all validation rules. Every rule gets its own integration test file exercised through the public `TrussEngine` API, so tests reflect real-world usage rather than internal implementation details. At the time of writing the suite covers **41 rules** across **346 tests** spread over **44 test files**.

## How Tests Are Organized

### Unit tests (same file)

**Location:** `crates/truss-core/validation.rs` (inside a `#[cfg(test)]` module)

These live alongside the code they test. They are useful for verifying private helpers and individual rule logic without going through the engine, so they run fast and stay isolated.

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

### Integration tests (separate files)

**Location:** `crates/truss-core/tests/validation_*.rs`

Each file tests one rule (or a closely related group) through the public API. The tests parse real YAML, run it through `TrussEngine`, and assert on the diagnostics that come back. This is where most of the coverage lives.

Naming follows the pattern `validation_<rule_name>.rs`:

- `validation_syntax.rs` -- SyntaxRule
- `validation_non_empty.rs` -- NonEmptyRule
- `validation_schema.rs` -- GitHubActionsSchemaRule
- `validation_job_name.rs` -- JobNameRule
- `validation_job_needs.rs` -- JobNeedsRule
- `validation_step.rs` -- StepValidationRule
- ... and so on for the remaining rules.

### Full test structure

```
crates/truss-core/
|-- lib.rs                              (unit tests in #[cfg(test)] module)
|-- validation/                         (rules organized by module)
|   |-- mod.rs
|   +-- rules/
|       |-- syntax.rs
|       |-- non_empty.rs
|       |-- schema.rs
|       +-- ... (all 41 rules)
+-- tests/
    |-- validation_syntax.rs            (4 tests)
    |-- validation_non_empty.rs         (3 tests)
    |-- validation_schema.rs            (5 tests)
    |-- validation_workflow_trigger.rs   (6 tests)
    |-- validation_job_name.rs          (5 tests)
    |-- validation_job_needs.rs         (6 tests)
    |-- validation_job_if_expression.rs (5 tests)
    |-- validation_job_outputs.rs       (5 tests)
    |-- validation_job_container.rs     (5 tests)
    |-- validation_job_strategy.rs      (5 tests)
    |-- validation_step.rs              (8 tests -- includes 3 mutual exclusion tests)
    |-- validation_step_name.rs         (5 tests)
    |-- validation_step_id_uniqueness.rs(5 tests)
    |-- validation_step_if_expression.rs(5 tests)
    |-- validation_step_output_reference.rs (5 tests)
    |-- validation_step_continue_on_error.rs (5 tests)
    |-- validation_step_timeout.rs      (5 tests)
    |-- validation_step_shell.rs        (5 tests)
    |-- validation_step_working_directory.rs (5 tests)
    |-- validation_step_env.rs          (8 tests -- includes 3 GITHUB_ prefix tests)
    |-- validation_expression.rs        (6 tests)
    |-- validation_expression_edge_cases.rs (5 tests)
    |-- validation_permissions.rs       (5 tests)
    |-- validation_environment.rs       (5 tests)
    |-- validation_workflow_name.rs     (5 tests)
    |-- validation_workflow_inputs.rs   (5 tests)
    |-- validation_workflow_call_inputs.rs (5 tests)
    |-- validation_workflow_call_secrets.rs (5 tests)
    |-- validation_workflow_call_outputs.rs (5 tests)
    |-- validation_reusable_workflow_call.rs (5 tests)
    |-- validation_matrix.rs            (5 tests)
    |-- validation_runs_on.rs           (5 tests)
    |-- validation_runner_label.rs      (5 tests)
    |-- validation_secrets.rs           (5 tests)
    |-- validation_timeout.rs           (5 tests)
    |-- validation_concurrency.rs       (5 tests)
    |-- validation_defaults.rs          (5 tests)
    |-- validation_action_reference.rs  (5 tests)
    |-- validation_artifact.rs          (5 tests)
    |-- validation_event_payload.rs     (17 tests -- includes filter conflicts, cron ranges, PR types)
    |-- validation_comment_handling.rs  (5 tests)
    |-- validation_deprecated_commands.rs (7 tests)
    |-- validation_script_injection.rs  (9 tests)
    +-- validation_benchmark_fixtures.rs (5 tests)
```

## What Every Rule's Tests Should Cover

### 1. Valid cases

Feed the rule well-formed input and make sure it produces zero diagnostics. This is the happy path -- it should always be the first thing you write.

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

### 2. Error cases

Deliberately break something and confirm the rule catches it with the right diagnostic.

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

### 3. Edge cases

Push the boundaries: empty strings, extremely long values, special characters, unusual but technically valid YAML. These tests tend to uncover the bugs that slip through normal testing.

```rust
#[test]
fn test_rule_edge_cases() {
    // Test empty, very long, special characters, etc.
}
```

### 4. Determinism

Run the same input through the same rule twice and compare results. Non-deterministic diagnostics would be a serious problem for users, so every rule needs at least one check like this.

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

### Step 1 -- Write the tests first

1. Pick the validation rule you want to implement.
2. Write tests that cover valid input, invalid input, edge cases, and determinism.
3. Tag the error-case tests with `#[ignore = "RuleName not yet implemented"]` so CI stays green while the rule does not exist yet.
4. Run the suite. Valid-case tests should pass; the ignored ones get skipped.

### Step 2 -- Make them pass

1. Implement the rule.
2. Remove the `#[ignore]` attributes.
3. Run the suite again -- everything should be green.

### Step 3 -- Clean up

1. Refactor for clarity and performance.
2. Confirm the tests still pass.
3. Add any extra edge cases that came to mind during implementation.

## Best Practices

1. **One test file per rule.** It keeps things easy to find and easy to maintain.
2. **Descriptive test names.** Follow the pattern `test_rule_name_what_it_tests` so failures are self-explanatory.
3. **Test through the public API.** Integration tests should go through `TrussEngine`, not call internal functions.
4. **Keep tests independent.** No test should rely on side effects from another test.
5. **Verify determinism.** Rules must produce the same output every time for the same input.
6. **Check the error messages.** A diagnostic that fires but says something confusing is almost as bad as one that does not fire at all.
7. **Use `#[ignore]` for pending work.** It signals intent without breaking CI.

## Running Tests

```bash
# Run all tests (unit + integration, ignored tests skipped)
cargo test -p truss-core

# Run including ignored tests (shows pending work)
cargo test -p truss-core -- --include-ignored

# Run a specific test file
cargo test -p truss-core --test validation_syntax

# Run with stdout visible
cargo test -p truss-core -- --nocapture

# Use justfile commands
just test-core
just test-validation
```

## Continuous Integration

All tests must pass before a PR can merge. Ignored tests are skipped so they do not block CI. Benchmark fixtures are included in the suite to catch performance regressions early.

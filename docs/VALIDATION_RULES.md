# GitHub Actions Validation Rules

This document defines all validation rules that should be implemented for GitHub Actions workflows, following TDD principles.

## Current Rules (Implemented)

### 1. SyntaxRule ✅
**Purpose:** Validates YAML syntax correctness  
**Status:** Implemented  
**Tests:** `validation_syntax.rs` (3 tests)

### 2. NonEmptyRule ✅
**Purpose:** Warns on empty documents  
**Status:** Implemented  
**Tests:** `validation_non_empty.rs` (4 tests)

### 3. GitHubActionsSchemaRule ✅
**Purpose:** Validates basic GitHub Actions workflow structure  
**Status:** Implemented (basic)  
**Tests:** `validation_schema.rs` (5 tests)

## Required Rules (To Implement)

### 4. WorkflowTriggerRule
**Purpose:** Validates `on:` trigger configuration  
**Tests:** `validation_workflow_trigger.rs` (6 tests, 3 ignored)  
**Test Cases:**
- ✅ Valid: `on: push`
- ✅ Valid: `on: [push, pull_request]`
- ✅ Valid: `on: { push: { branches: [main] } }`
- ❌ Error: Missing `on:` field (ignored)
- ❌ Error: Invalid event type (ignored)
- ❌ Error: Invalid trigger syntax (ignored)

### 5. JobNameRule
**Purpose:** Validates job names  
**Tests:** `validation_job_name.rs` (5 tests, 3 ignored)  
**Test Cases:**
- ✅ Valid: `build`, `test`, `deploy`
- ✅ Valid: `build-and-test`
- ❌ Error: Duplicate job names (ignored)
- ❌ Error: Invalid characters (ignored)
- ❌ Error: Reserved names (ignored)

### 6. JobNeedsRule
**Purpose:** Validates job dependencies (`needs:`)  
**Tests:** `validation_job_needs.rs` (5 tests, 3 ignored)  
**Test Cases:**
- ✅ Valid: `needs: [build, test]`
- ✅ Valid: `needs: build`
- ❌ Error: Reference to non-existent job (ignored)
- ❌ Error: Circular dependency (ignored)
- ❌ Error: Self-reference (ignored)

### 7. StepValidationRule
**Purpose:** Validates step structure  
**Tests:** `validation_step.rs` (5 tests, 2 ignored)  
**Test Cases:**
- ✅ Valid: Step with `uses:`
- ✅ Valid: Step with `run:`
- ✅ Valid: Multiple steps with both
- ❌ Error: Missing both `uses` and `run` (ignored)
- ❌ Error: Invalid action reference (ignored)

### 8. ExpressionValidationRule
**Purpose:** Validates GitHub Actions expressions  
**Tests:** `validation_expression.rs` (8 tests, 3 ignored)  
**Test Cases:**
- ✅ Valid: `${{ github.event.pull_request.number }}`
- ✅ Valid: `${{ matrix.os }}`
- ✅ Valid: Conditional expressions
- ✅ Valid: Nested property access
- ✅ Valid: workflow_dispatch inputs
- ❌ Error: Invalid expression syntax (ignored)
- ❌ Error: Undefined context variable (ignored)
- ❌ Error: Unclosed expression (ignored)

### 9. PermissionsRule
**Purpose:** Validates permissions configuration  
**Tests:** `validation_permissions.rs` (8 tests, 2 ignored)  
**Test Cases:**
- ✅ Valid: `permissions: read-all`
- ✅ Valid: `permissions: write-all`
- ✅ Valid: `permissions: { contents: read }`
- ✅ Valid: Empty permissions `{}`
- ✅ Valid: Job-level permissions
- ✅ Valid: `none` value
- ❌ Error: Invalid permission scope (ignored)
- ❌ Error: Invalid permission value (ignored)

### 10. EnvironmentRule
**Purpose:** Validates environment references  
**Tests:** `validation_environment.rs` (7 tests, 2 ignored)  
**Test Cases:**
- ✅ Valid: `environment: production`
- ✅ Valid: `environment: { name: prod, url: ... }`
- ✅ Valid: Workflow-level env variables
- ✅ Valid: Step-level env variables
- ✅ Valid: Environment with URL
- ❌ Error: Invalid environment name (ignored)
- ❌ Error: Invalid protection rules (ignored)

### 11. WorkflowNameRule
**Purpose:** Validates workflow name field  
**Tests:** `validation_workflow_name.rs` (7 tests, 2 ignored)  
**Test Cases:**
- ✅ Valid: `name: CI`
- ✅ Valid: `name: ${{ github.event.pull_request.title }}`
- ✅ Valid: Optional (missing name is OK)
- ✅ Valid: Special characters in quotes
- ✅ Valid: Unicode characters
- ❌ Error: Empty name `name: ""` (ignored)
- ❌ Error: Name too long (ignored)

### 12. MatrixStrategyRule
**Purpose:** Validates matrix strategy syntax  
**Status:** Not started  
**Test Cases:**
- ✅ Valid: `matrix: { os: [ubuntu, windows] }`
- ✅ Valid: `matrix: { include: [...] }`
- ❌ Error: Invalid matrix syntax
- ❌ Error: Empty matrix
- ❌ Error: Invalid include/exclude

## Test Organization

### Test File Structure
```
crates/truss-core/tests/
├── validation_syntax.rs          ✅ (3 tests passing)
├── validation_non_empty.rs       ✅ (4 tests passing)
├── validation_schema.rs          ✅ (5 tests passing)
├── validation_workflow_trigger.rs 🔴 (3 passing, 3 ignored)
├── validation_job_name.rs        🔴 (2 passing, 3 ignored)
├── validation_job_needs.rs       🔴 (2 passing, 3 ignored)
├── validation_step.rs            🔴 (3 passing, 2 ignored)
├── validation_expression.rs      🔴 (5 passing, 3 ignored)
├── validation_permissions.rs     🔴 (6 passing, 2 ignored)
├── validation_environment.rs     🔴 (5 passing, 2 ignored)
└── validation_workflow_name.rs   🔴 (5 passing, 2 ignored)
```

## Implementation Priority

1. **WorkflowTriggerRule** - Foundational, validates `on:` triggers
2. **JobNameRule** - Needed before JobNeedsRule
3. **JobNeedsRule** - Depends on JobNameRule for validation
4. **StepValidationRule** - Core workflow validation
5. **ExpressionValidationRule** - Requires expression parser
6. **PermissionsRule** - Straightforward validation
7. **EnvironmentRule** - Environment protection validation
8. **WorkflowNameRule** - Simple but useful
9. **MatrixStrategyRule** - Complex, needs tests first

## Running Tests

```bash
# Run all tests (ignored tests are skipped)
cargo test -p truss-core

# Run including ignored tests (see pending work)
cargo test -p truss-core -- --include-ignored

# Run specific rule tests
cargo test -p truss-core --test validation_job_name

# Use justfile
just test-core
```

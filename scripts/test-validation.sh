#!/bin/bash
# Run all validation tests and show results

echo "Running all validation test files..."

for test in validation_syntax validation_non_empty validation_schema validation_workflow_trigger validation_job_name validation_job_needs validation_step validation_expression validation_permissions validation_environment validation_workflow_name; do
    echo ""
    echo "=== $test ==="
    cargo test -p truss-core --test "$test" 2>&1 | grep "test result" || true
done

echo ""
echo "Note: Some tests will fail until rules are implemented (expected TDD behavior)"
echo "Run 'cargo test -p truss-core' to see all tests with full details"


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
**Status:** Implemented  
**Tests:** `validation_schema.rs` (5 tests)

### 4. WorkflowTriggerRule ✅
**Purpose:** Validates `on:` trigger configuration  
**Status:** Implemented  
**Tests:** `validation_workflow_trigger.rs` (6 tests)  
**Test Cases:**
- ✅ Valid: `on: push`
- ✅ Valid: `on: [push, pull_request]`
- ✅ Valid: `on: { push: { branches: [main] } }`
- ✅ Error: Missing `on:` field
- ✅ Error: Invalid event type
- ✅ Error: Invalid trigger syntax

### 5. JobNameRule ✅
**Purpose:** Validates job names  
**Status:** Implemented  
**Tests:** `validation_job_name.rs` (5 tests)  
**Test Cases:**
- ✅ Valid: `build`, `test`, `deploy`
- ✅ Valid: `build-and-test`
- ✅ Error: Duplicate job names
- ✅ Error: Invalid characters (spaces)
- ✅ Error: Reserved names (`if`, `else`, etc.)

### 6. JobNeedsRule ✅
**Purpose:** Validates job dependencies (`needs:`)  
**Status:** Implemented  
**Tests:** `validation_job_needs.rs` (5 tests)  
**Test Cases:**
- ✅ Valid: `needs: [build, test]`
- ✅ Valid: `needs: build`
- ✅ Error: Reference to non-existent job
- ✅ Error: Circular dependency
- ✅ Error: Self-reference

### 7. StepValidationRule ✅
**Purpose:** Validates step structure  
**Status:** Implemented  
**Tests:** `validation_step.rs` (5 tests)  
**Test Cases:**
- ✅ Valid: Step with `uses:`
- ✅ Valid: Step with `run:`
- ✅ Valid: Multiple steps with both
- ✅ Error: Missing both `uses` and `run`
- ✅ Warning: Invalid action reference format

### 8. ExpressionValidationRule ✅
**Purpose:** Validates GitHub Actions expressions  
**Status:** Implemented  
**Tests:** `validation_expression.rs` (8 tests)  
**Test Cases:**
- ✅ Valid: `${{ github.event.pull_request.number }}`
- ✅ Valid: `${{ matrix.os }}`
- ✅ Valid: Conditional expressions
- ✅ Valid: Nested property access
- ✅ Valid: workflow_dispatch inputs
- ✅ Error: Invalid expression syntax
- ✅ Warning: Undefined context variable
- ✅ Error: Unclosed expression

### 9. PermissionsRule ✅
**Purpose:** Validates permissions configuration  
**Status:** Implemented  
**Tests:** `validation_permissions.rs` (8 tests)  
**Test Cases:**
- ✅ Valid: `permissions: read-all`
- ✅ Valid: `permissions: write-all`
- ✅ Valid: `permissions: { contents: read }`
- ✅ Valid: Empty permissions `{}`
- ✅ Valid: Job-level permissions
- ✅ Valid: `none` value
- ✅ Error: Invalid permission scope
- ✅ Error: Invalid permission value

### 10. EnvironmentRule ✅
**Purpose:** Validates environment references  
**Status:** Implemented  
**Tests:** `validation_environment.rs` (7 tests)  
**Test Cases:**
- ✅ Valid: `environment: production`
- ✅ Valid: `environment: { name: prod, url: ... }`
- ✅ Valid: Workflow-level env variables
- ✅ Valid: Step-level env variables
- ✅ Valid: Environment with URL
- ✅ Error: Invalid environment name (spaces)
- ✅ Error: Invalid protection rules (not supported in workflow YAML)

### 11. WorkflowNameRule ✅
**Purpose:** Validates workflow name field  
**Status:** Implemented  
**Tests:** `validation_workflow_name.rs` (7 tests)  
**Test Cases:**
- ✅ Valid: `name: CI`
- ✅ Valid: `name: ${{ github.event.pull_request.title }}`
- ✅ Valid: Optional (missing name is OK)
- ✅ Valid: Special characters in quotes
- ✅ Valid: Unicode characters
- ✅ Error: Empty name `name: ""`
- ✅ Error: Name too long (>255 characters)

### 12. MatrixStrategyRule ✅
**Purpose:** Validates matrix strategy syntax  
**Status:** Implemented  
**Tests:** `validation_matrix.rs` (7 tests)  
**Test Cases:**
- ✅ Valid: `matrix: { os: [ubuntu, windows] }`
- ✅ Valid: `matrix: { include: [...] }`
- ✅ Valid: `matrix: { exclude: [...] }`
- ✅ Error: Empty matrix
- ✅ Error: Invalid include syntax
- ✅ Error: Invalid exclude syntax

### 13. RunsOnRequiredRule ✅
**Purpose:** Validates that `runs-on` is required for all jobs  
**Status:** Implemented  
**Tests:** `validation_runs_on.rs` (7 tests)  
**Test Cases:**
- ✅ Valid: Job with `runs-on: ubuntu-latest`
- ✅ Valid: Job with `runs-on: ${{ matrix.os }}` (expression)
- ✅ Valid: Multiple jobs with `runs-on`
- ✅ Valid: Job with `runs-on` and other fields
- ✅ Error: Job missing `runs-on` field
- ✅ Error: Job with empty `runs-on: ""`
- ✅ Error: One of multiple jobs missing `runs-on`

### 14. SecretsValidationRule ✅
**Purpose:** Validates secrets.* references  
**Status:** Implemented  
**Tests:** `validation_secrets.rs` (7 tests)  
**Test Cases:**
- ✅ Valid: `${{ secrets.GITHUB_TOKEN }}`
- ✅ Valid: `${{ secrets.MY_SECRET }}`
- ✅ Valid: Secret reference in env variables
- ✅ Valid: Multiple secret references
- ✅ Valid: Secret reference in conditional
- ✅ Error: Invalid syntax (singular 'secret' instead of 'secrets')
- ✅ Error: Missing dot (secretsMY_SECRET instead of secrets.MY_SECRET)

### 15. TimeoutRule ✅
**Purpose:** Validates that `timeout-minutes` is a positive number  
**Status:** Implemented  
**Tests:** `validation_timeout.rs` (8 tests)  
**Test Cases:**
- ✅ Valid: `timeout-minutes: 60`
- ✅ Valid: `timeout-minutes: ${{ matrix.timeout }}` (expression)
- ✅ Valid: No timeout (optional field)
- ✅ Valid: Large positive number
- ✅ Valid: Decimal number (GitHub Actions accepts decimals)
- ✅ Error: `timeout-minutes: -5` (negative)
- ✅ Error: `timeout-minutes: 0` (zero)
- ✅ Error: `timeout-minutes: "60"` (string instead of number)

### 16. WorkflowInputsRule ✅
**Purpose:** Validates workflow_dispatch inputs  
**Status:** Implemented  
**Tests:** `validation_workflow_inputs.rs` (8 tests)  
**Test Cases:**
- ✅ Valid: `workflow_dispatch` with string input
- ✅ Valid: `workflow_dispatch` with choice input
- ✅ Valid: `workflow_dispatch` with boolean input
- ✅ Valid: `workflow_dispatch` with environment input
- ✅ Valid: `workflow_dispatch` without inputs
- ✅ Valid: Multiple inputs
- ✅ Error: Input reference to undefined input
- ✅ Error: Invalid input type

### 17. JobOutputsRule ✅
**Purpose:** Validates that job outputs reference valid step IDs  
**Status:** Implemented  
**Tests:** `validation_job_outputs.rs` (10 tests)  
**Test Cases:**
- ✅ Valid: `outputs: { result: ${{ steps.build.outputs.result }} }` where `build` step exists
- ✅ Valid: Multiple outputs referencing same step ID
- ✅ Valid: Output with conditional expression
- ✅ Error: Reference to non-existent step ID
- ✅ Error: Reference to step in different job
- ✅ Error: Reference to step without `id` field
- ✅ Error: Invalid output syntax (missing output name)

### 18. ConcurrencyRule ✅
**Purpose:** Validates concurrency syntax  
**Status:** Implemented  
**Tests:** `validation_concurrency.rs` (11 tests)  
**Test Cases:**
- ✅ Valid: `concurrency: { group: 'ci-${{ github.ref }}', cancel-in-progress: true }`
- ✅ Valid: Workflow-level concurrency with cancel-in-progress: false
- ✅ Valid: Job-level concurrency with expression group
- ✅ Error: Missing required `group` field (workflow level)
- ✅ Error: Missing required `group` field (job level)
- ✅ Error: Invalid cancel-in-progress value (string instead of boolean)
- ✅ Error: Invalid group type (number instead of string/expression)

### 19. ActionReferenceRule ✅
**Purpose:** Validates action reference format (owner/repo@ref)  
**Status:** Implemented  
**Tests:** `validation_action_reference.rs` (14 tests)  
**Test Cases:**
- ✅ Valid: `uses: actions/checkout@v3` (tag)
- ✅ Valid: `uses: actions/checkout@main` (branch)
- ✅ Valid: `uses: actions/checkout@abc123def456` (SHA)
- ✅ Valid: Local path `uses: ./.github/actions/my-action`
- ✅ Valid: Docker `uses: docker://alpine:3.18`
- ✅ Valid: Composite action `uses: my-org/my-composite-action@v1`
- ✅ Error: `uses: invalid-action` (missing @ref)
- ✅ Error: `uses: owner/repo` (missing @ref)
- ✅ Error: `uses: checkout@v3` (missing owner)
- ✅ Error: `uses: actionscheckout@v3` (missing slash)
- ✅ Error: Invalid owner format (spaces)

### 20. StepIdUniquenessRule ✅
**Purpose:** Validates that step IDs are unique within a job  
**Status:** Implemented  
**Tests:** `validation_step_id_uniqueness.rs` (7 tests)  
**Test Cases:**
- ✅ Valid: Unique step IDs within a job
- ✅ Valid: Steps without IDs (no conflict)
- ✅ Valid: Different jobs can have same step IDs
- ✅ Error: Duplicate step ID in same job
- ✅ Error: Multiple steps with same `id` field

### 21. StepOutputReferenceRule ✅
**Purpose:** Validates that step output references (`steps.<step_id>.outputs.<output_name>`) reference valid outputs  
**Status:** Implemented  
**Tests:** `validation_step_output_reference.rs` (9 tests)  
**Test Cases:**
- ✅ Valid: Reference to existing step output
- ✅ Valid: Reference in job outputs
- ✅ Valid: Reference in step `if` conditions
- ✅ Valid: Reference in step `env` variables
- ✅ Error: Reference to non-existent output name
- ✅ Error: Reference to output from step without `id`
- ✅ Error: Reference to step in different job

### 22. JobStrategyValidationRule ✅
**Purpose:** Validates `strategy` field syntax and constraints (max-parallel, fail-fast)  
**Status:** Implemented  
**Tests:** `validation_job_strategy.rs` (8 tests)  
**Test Cases:**
- ✅ Valid: `strategy: { max-parallel: 3, fail-fast: true }`
- ✅ Valid: Strategy with matrix
- ✅ Error: `max-parallel: -1` (must be positive)
- ✅ Error: `fail-fast: "true"` (must be boolean, not string)

### 23. StepIfExpressionRule ✅
**Purpose:** Validates `if` condition expressions in steps  
**Status:** Implemented  
**Tests:** `validation_step_if_expression.rs` (7 tests)  
**Test Cases:**
- ✅ Valid: `if: ${{ github.ref == 'refs/heads/main' }}`
- ✅ Valid: Complex conditional expressions
- ✅ Error: `if: github.ref == 'refs/heads/main'` (missing `${{ }}` wrapper)
- ✅ Error: Invalid expression syntax in `if` condition

### 24. JobIfExpressionRule ✅
**Purpose:** Validates `if` condition expressions in jobs  
**Status:** Implemented  
**Tests:** `validation_job_if_expression.rs` (6 tests)  
**Test Cases:**
- ✅ Valid: `if: ${{ github.ref == 'refs/heads/main' }}`
- ✅ Valid: Job-level conditional expressions
- ✅ Error: Invalid expression syntax in `if` condition

### 25. WorkflowCallInputsRule ✅
**Purpose:** Validates `workflow_call` inputs and their usage  
**Status:** Implemented  
**Tests:** `validation_workflow_call_inputs.rs` (8 tests)  
**Test Cases:**
- ✅ Valid: `workflow_call` with properly defined inputs
- ✅ Valid: Input reference matches defined input
- ✅ Error: Input reference to undefined input
- ✅ Error: Invalid input type

### 26. WorkflowCallSecretsRule ✅
**Purpose:** Validates `workflow_call` secrets and their usage  
**Status:** Implemented  
**Tests:** `validation_workflow_call_secrets.rs` (6 tests)  
**Test Cases:**
- ✅ Valid: `workflow_call` with properly defined secrets
- ✅ Valid: Secret reference matches defined secret
- ✅ Error: Secret reference to undefined secret

### 27. ReusableWorkflowCallRule ✅
**Purpose:** Validates `uses:` workflow calls reference valid reusable workflows  
**Status:** Implemented  
**Tests:** `validation_reusable_workflow_call.rs` (7 tests)  
**Test Cases:**
- ✅ Valid: `uses: owner/repo/.github/workflows/reusable.yml@main`
- ✅ Valid: Reusable workflow call with inputs
- ✅ Error: Invalid workflow call format
- ✅ Error: Missing required fields

### 28. WorkflowCallOutputsRule ✅
**Purpose:** Validates `workflow_call` outputs are properly defined  
**Status:** Implemented  
**Tests:** `validation_workflow_call_outputs.rs` (6 tests)  
**Test Cases:**
- ✅ Valid: `workflow_call` output references valid job output
- ✅ Error: Output references non-existent job
- ✅ Error: Output references non-existent job output

### 29. StepContinueOnErrorRule ✅
**Purpose:** Validates `continue-on-error` is a boolean  
**Status:** Implemented  
**Tests:** `validation_step_continue_on_error.rs` (4 tests)  
**Test Cases:**
- ✅ Valid: `continue-on-error: true`
- ✅ Valid: `continue-on-error: false`
- ✅ Error: `continue-on-error: "true"` (string instead of boolean)

### 30. StepTimeoutRule ✅
**Purpose:** Validates `timeout-minutes` at step level  
**Status:** Implemented  
**Tests:** `validation_step_timeout.rs` (6 tests)  
**Test Cases:**
- ✅ Valid: `timeout-minutes: 30` at step level
- ✅ Error: `timeout-minutes: -5` (must be positive)
- ✅ Error: `timeout-minutes: 0` (must be positive)

### 31. StepShellRule ✅
**Purpose:** Validates `shell` field values  
**Status:** Implemented  
**Tests:** `validation_step_shell.rs` (8 tests)  
**Test Cases:**
- ✅ Valid: `shell: bash`, `shell: pwsh`, `shell: python`
- ✅ Valid: Custom shell with inline script
- ✅ Error: Invalid shell syntax

### 32. StepWorkingDirectoryRule ✅
**Purpose:** Validates `working-directory` paths  
**Status:** Implemented  
**Tests:** `validation_step_working_directory.rs` (4 tests)  
**Test Cases:**
- ✅ Valid: `working-directory: ./src`
- ✅ Valid: Absolute and relative paths
- ✅ Warning: Potentially invalid paths (basic format validation)

### 33. ArtifactValidationRule ✅
**Purpose:** Validates `actions/upload-artifact` and `actions/download-artifact` usage  
**Status:** Implemented  
**Tests:** `validation_artifact.rs` (5 tests)  
**Test Cases:**
- ✅ Valid: Artifact with valid name and path
- ✅ Error: Empty artifact name
- ✅ Warning: Potentially invalid paths

### 34. EventPayloadValidationRule ✅
**Purpose:** Validates event-specific fields in `on:` triggers  
**Status:** Implemented  
**Tests:** `validation_event_payload.rs` (6 tests)  
**Test Cases:**
- ✅ Valid: Event-specific fields for each event type
- ✅ Error: Invalid field for event type
- ✅ Error: Invalid event type value

### 35. RunnerLabelRule ✅
**Purpose:** Validates `runs-on` labels are valid GitHub-hosted runners or self-hosted runner groups  
**Status:** Implemented  
**Tests:** `validation_runner_label.rs` (6 tests)  
**Test Cases:**
- ✅ Valid: Known GitHub-hosted runners (`ubuntu-latest`, `windows-latest`, etc.)
- ✅ Valid: Self-hosted runner labels
- ✅ Warning: Unknown runner labels (basic format validation)

### 36. StepEnvValidationRule ✅
**Purpose:** Validates environment variable names and values at step level  
**Status:** Implemented  
**Tests:** `validation_step_env.rs` (5 tests)  
**Test Cases:**
- ✅ Valid: `env: { VALID_NAME: value }`
- ✅ Valid: Environment variables with expressions
- ✅ Error: Invalid env var name format

### 37. JobContainerRule ✅
**Purpose:** Validates `container` and `services` configurations  
**Status:** Implemented  
**Tests:** `validation_job_container.rs` (6 tests)  
**Test Cases:**
- ✅ Valid: Container with valid image and ports
- ✅ Valid: Services configuration
- ✅ Error: Invalid port mapping format
- ✅ Error: Invalid container configuration

### 38. StepNameRule ✅
**Purpose:** Validates step `name` field format  
**Status:** Implemented  
**Tests:** `validation_step_name.rs` (6 tests)  
**Test Cases:**
- ✅ Valid: Step with valid name
- ✅ Valid: Step name with expressions
- ✅ Warning: Empty step name
- ✅ Warning: Very long step name

### 39. DefaultsValidationRule ✅
**Purpose:** Validates `defaults` configuration at workflow and job levels  
**Status:** Implemented  
**Tests:** `validation_defaults.rs` (6 tests)  
**Test Cases:**
- ✅ Valid: Defaults with valid shell and working-directory
- ✅ Valid: Workflow-level defaults
- ✅ Valid: Job-level defaults
- ✅ Error: Invalid shell in defaults
- ✅ Error: Invalid working-directory in defaults

## Test Organization

### Test File Structure
```
crates/truss-core/tests/
├── validation_syntax.rs                  ✅ (3 tests)
├── validation_non_empty.rs              ✅ (4 tests)
├── validation_schema.rs                 ✅ (5 tests)
├── validation_workflow_trigger.rs        ✅ (6 tests)
├── validation_job_name.rs                ✅ (5 tests)
├── validation_job_needs.rs               ✅ (5 tests)
├── validation_job_if_expression.rs       ✅ (6 tests)
├── validation_job_outputs.rs             ✅ (10 tests)
├── validation_job_container.rs           ✅ (6 tests)
├── validation_job_strategy.rs            ✅ (8 tests)
├── validation_step.rs                    ✅ (5 tests)
├── validation_step_name.rs               ✅ (6 tests)
├── validation_step_id_uniqueness.rs      ✅ (7 tests)
├── validation_step_if_expression.rs      ✅ (7 tests)
├── validation_step_output_reference.rs   ✅ (9 tests)
├── validation_step_continue_on_error.rs  ✅ (4 tests)
├── validation_step_timeout.rs            ✅ (6 tests)
├── validation_step_shell.rs              ✅ (8 tests)
├── validation_step_working_directory.rs  ✅ (4 tests)
├── validation_step_env.rs                ✅ (5 tests)
├── validation_expression.rs             ✅ (8 tests)
├── validation_expression_edge_cases.rs  ✅ (11 tests)
├── validation_permissions.rs             ✅ (8 tests)
├── validation_environment.rs             ✅ (7 tests)
├── validation_workflow_name.rs           ✅ (7 tests)
├── validation_workflow_inputs.rs         ✅ (8 tests)
├── validation_workflow_call_inputs.rs    ✅ (8 tests)
├── validation_workflow_call_secrets.rs   ✅ (6 tests)
├── validation_workflow_call_outputs.rs   ✅ (6 tests)
├── validation_reusable_workflow_call.rs  ✅ (7 tests)
├── validation_matrix.rs                  ✅ (7 tests)
├── validation_runs_on.rs                 ✅ (7 tests)
├── validation_runner_label.rs            ✅ (6 tests)
├── validation_secrets.rs                 ✅ (7 tests)
├── validation_timeout.rs                 ✅ (8 tests)
├── validation_concurrency.rs             ✅ (11 tests)
├── validation_defaults.rs                ✅ (6 tests)
├── validation_action_reference.rs        ✅ (14 tests)
├── validation_artifact.rs                ✅ (5 tests)
├── validation_event_payload.rs           ✅ (6 tests)
├── validation_comment_handling.rs        ✅ (8 tests)
└── validation_benchmark_fixtures.rs      ✅ (10 tests)
```

**Total: 294 tests across 40 test files (all passing)**

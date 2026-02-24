# GitHub Actions Validation Rules

This document covers every validation rule implemented in Truss. Each rule was developed test-first, so the test cases listed below double as a living spec for what the rule accepts and rejects.

## Current Rules

### 1. SyntaxRule
Catches malformed YAML before anything else runs. If the file can't be parsed, there's no point running further rules.

**Tests:** `validation_syntax.rs` (3 tests)

### 2. NonEmptyRule
Flags empty or effectively-blank documents. Usually means someone committed a placeholder file by accident.

**Tests:** `validation_non_empty.rs` (4 tests)

### 3. GitHubActionsSchemaRule
Checks the basic shape of a workflow file -- does it have `on:` and `jobs:`, are the top-level keys what GitHub expects, etc.

**Tests:** `validation_schema.rs` (5 tests)

### 4. WorkflowTriggerRule
Validates the `on:` trigger block. Supports the shorthand string form, array form, and full object form with branch/path filters.

**Tests:** `validation_workflow_trigger.rs` (6 tests)
**Test cases:**
- ✅ `on: push` (simple string)
- ✅ `on: [push, pull_request]` (array)
- ✅ `on: { push: { branches: [main] } }` (object with filters)
- ✅ Error on missing `on:` field entirely
- ✅ Error on unrecognized event types
- ✅ Error on malformed trigger syntax

### 5. JobNameRule
Makes sure job IDs are valid identifiers. GitHub is surprisingly strict here -- no spaces, no reserved words.

**Tests:** `validation_job_name.rs` (5 tests)
**Test cases:**
- ✅ Standard names like `build`, `test`, `deploy`
- ✅ Hyphenated names like `build-and-test`
- ✅ Error on duplicate job names
- ✅ Error on names with spaces or special characters
- ✅ Error on reserved words (`if`, `else`, etc.)

### 6. JobNeedsRule
Validates the `needs:` dependency graph between jobs. Catches dangling references, cycles, and self-references that would cause GitHub to reject the workflow at runtime.

**Tests:** `validation_job_needs.rs` (5 tests)
**Test cases:**
- ✅ `needs: [build, test]` (array form)
- ✅ `needs: build` (string form)
- ✅ Error when referencing a job that doesn't exist
- ✅ Error on circular dependencies
- ✅ Error on self-references

### 7. StepValidationRule
Every step needs either `uses:` or `run:` -- this rule enforces that, and also checks that action references in `uses:` look reasonable.

**Tests:** `validation_step.rs` (8 tests)
**Test cases:**
- ✅ Step with `uses:`
- ✅ Step with `run:`
- ✅ Multiple steps mixing both forms
- ✅ Error when a step has neither `uses` nor `run`
- ✅ Warning on invalid action reference format

### 8. ExpressionValidationRule
Parses `${{ ... }}` expressions and checks that context references (like `github.event.pull_request.number` or `matrix.os`) are plausible.

**Tests:** `validation_expression.rs` (8 tests)
**Test cases:**
- ✅ Property access: `${{ github.event.pull_request.number }}`
- ✅ Matrix references: `${{ matrix.os }}`
- ✅ Conditional expressions
- ✅ Nested property access chains
- ✅ `workflow_dispatch` input references
- ✅ Error on broken expression syntax
- ✅ Warning on undefined context variables
- ✅ Error on unclosed `${{ }}`

### 9. PermissionsRule
Validates the `permissions:` block at both workflow and job levels. GitHub supports `read-all`, `write-all`, `none`, or a map of individual scopes.

**Tests:** `validation_permissions.rs` (8 tests)
**Test cases:**
- ✅ `permissions: read-all` and `permissions: write-all`
- ✅ Scoped map: `permissions: { contents: read }`
- ✅ Empty permissions `{}` (effectively no permissions)
- ✅ Job-level permissions override
- ✅ `none` value
- ✅ Error on invalid permission scope names
- ✅ Error on invalid permission values (anything other than `read`, `write`, `none`)

### 10. EnvironmentRule
Checks environment references and environment variable definitions at the workflow, job, and step levels.

**Tests:** `validation_environment.rs` (7 tests)
**Test cases:**
- ✅ Simple string: `environment: production`
- ✅ Object form: `environment: { name: prod, url: ... }`
- ✅ Workflow-level and step-level `env:` blocks
- ✅ Environment with URL
- ✅ Error on names with invalid characters
- ✅ Error on protection rules (not supported in workflow YAML)

### 11. WorkflowNameRule
The `name:` field is optional, but if present it should be non-empty and not absurdly long.

**Tests:** `validation_workflow_name.rs` (7 tests)
**Test cases:**
- ✅ `name: CI`
- ✅ Expression in name: `name: ${{ github.event.pull_request.title }}`
- ✅ Missing name is fine (it's optional)
- ✅ Special characters and Unicode are allowed
- ✅ Error on empty string `name: ""`
- ✅ Error on names longer than 255 characters

### 12. MatrixStrategyRule
Validates `strategy.matrix` blocks including `include` and `exclude` modifiers.

**Tests:** `validation_matrix.rs` (7 tests)
**Test cases:**
- ✅ `matrix: { os: [ubuntu, windows] }`
- ✅ `matrix: { include: [...] }`
- ✅ `matrix: { exclude: [...] }`
- ✅ Error on empty matrix
- ✅ Error on invalid `include`/`exclude` syntax

### 13. RunsOnRequiredRule
Every job needs a `runs-on` value. This rule catches jobs that are missing it or have it set to an empty string, which would fail silently on GitHub.

**Tests:** `validation_runs_on.rs` (7 tests)
**Test cases:**
- ✅ `runs-on: ubuntu-latest`
- ✅ Expression form: `runs-on: ${{ matrix.os }}`
- ✅ Multiple jobs each with their own `runs-on`
- ✅ Error when `runs-on` is missing
- ✅ Error when `runs-on` is empty
- ✅ Error when one job in a multi-job workflow is missing it

### 14. SecretsValidationRule
Catches common typos in `secrets.*` references -- the most frequent being `secret.` (singular) instead of `secrets.` (plural).

**Tests:** `validation_secrets.rs` (7 tests)
**Test cases:**
- ✅ `${{ secrets.GITHUB_TOKEN }}` and `${{ secrets.MY_SECRET }}`
- ✅ Secret references inside `env:` blocks
- ✅ Multiple secret references in one file
- ✅ Secret reference in `if:` conditionals
- ✅ Error on `secret.` (missing the "s")
- ✅ Error on `secretsMY_SECRET` (missing the dot)

### 15. TimeoutRule
Validates `timeout-minutes` at the job level. Must be a positive number -- GitHub silently accepts strings and zeros, but they don't behave the way you'd expect.

**Tests:** `validation_timeout.rs` (8 tests)
**Test cases:**
- ✅ `timeout-minutes: 60`
- ✅ Expression form: `timeout-minutes: ${{ matrix.timeout }}`
- ✅ No timeout specified (it's optional)
- ✅ Large values and decimals (GitHub does accept these)
- ✅ Error on negative values
- ✅ Error on zero
- ✅ Error on string values like `timeout-minutes: "60"`

### 16. WorkflowInputsRule
Validates `workflow_dispatch` inputs: their types, required flags, default values, and whether references to them actually exist.

**Tests:** `validation_workflow_inputs.rs` (8 tests)
**Test cases:**
- ✅ String, choice, boolean, and environment input types
- ✅ `workflow_dispatch` with no inputs (valid)
- ✅ Multiple inputs
- ✅ Error on references to inputs that aren't defined
- ✅ Error on unrecognized input types

### 17. JobOutputsRule
Checks that job-level `outputs:` actually reference step IDs that exist in that job. A surprisingly common source of broken workflows when steps get renamed or moved.

**Tests:** `validation_job_outputs.rs` (10 tests)
**Test cases:**
- ✅ `outputs: { result: ${{ steps.build.outputs.result }} }` where `build` step exists
- ✅ Multiple outputs pointing at the same step
- ✅ Outputs with conditional expressions
- ✅ Error on references to non-existent step IDs
- ✅ Error on cross-job step references
- ✅ Error on references to steps that lack an `id` field
- ✅ Error on malformed output syntax

### 18. ConcurrencyRule
Validates concurrency groups at workflow and job levels. The `group` field is required when using the object form -- without it, GitHub will reject the workflow.

**Tests:** `validation_concurrency.rs` (11 tests)
**Test cases:**
- ✅ `concurrency: { group: 'ci-${{ github.ref }}', cancel-in-progress: true }`
- ✅ `cancel-in-progress: false` at workflow level
- ✅ Job-level concurrency with expression-based groups
- ✅ Error on missing `group` field (both workflow and job level)
- ✅ Error on `cancel-in-progress` being a string instead of boolean
- ✅ Error on `group` being a number instead of string/expression

### 19. ActionReferenceRule
Validates the format of `uses:` references. Handles the various forms: `owner/repo@ref`, local paths, Docker images, and composite actions.

**Tests:** `validation_action_reference.rs` (14 tests)
**Test cases:**
- ✅ Tag ref: `uses: actions/checkout@v3`
- ✅ Branch ref: `uses: actions/checkout@main`
- ✅ SHA ref: `uses: actions/checkout@abc123def456`
- ✅ Local path: `uses: ./.github/actions/my-action`
- ✅ Docker: `uses: docker://alpine:3.18`
- ✅ Composite: `uses: my-org/my-composite-action@v1`
- ✅ Error on missing `@ref` suffix
- ✅ Error on `owner/repo` without version
- ✅ Error on missing owner (`checkout@v3`)
- ✅ Error on missing slash (`actionscheckout@v3`)
- ✅ Error on spaces in owner name

### 20. StepIdUniquenessRule
Step IDs must be unique within a job. Different jobs can reuse the same IDs -- that's fine -- but duplicates within a single job will confuse output references.

**Tests:** `validation_step_id_uniqueness.rs` (7 tests)
**Test cases:**
- ✅ Unique step IDs within a job
- ✅ Steps without IDs (no conflict possible)
- ✅ Same step ID in different jobs (allowed)
- ✅ Error on duplicate step IDs in the same job

### 21. StepOutputReferenceRule
Validates `steps.<step_id>.outputs.<output_name>` references. Checks that the step ID exists, belongs to the current job, and actually has an `id` field.

**Tests:** `validation_step_output_reference.rs` (9 tests)
**Test cases:**
- ✅ Reference to existing step output
- ✅ References in job outputs, `if:` conditions, and `env:` blocks
- ✅ Error on non-existent output names
- ✅ Error on references to steps without an `id`
- ✅ Error on cross-job step references

### 22. JobStrategyValidationRule
Validates the `strategy` block beyond just the matrix -- checks `max-parallel` and `fail-fast` types and values.

**Tests:** `validation_job_strategy.rs` (8 tests)
**Test cases:**
- ✅ `strategy: { max-parallel: 3, fail-fast: true }`
- ✅ Strategy combined with matrix
- ✅ Error on negative `max-parallel`
- ✅ Error on `fail-fast` being a string instead of boolean

### 23. StepIfExpressionRule
Validates `if:` conditions on steps. GitHub actually allows bare expressions without `${{ }}` wrappers in `if:` fields, but we warn about it since it's a common source of confusion and inconsistency.

**Tests:** `validation_step_if_expression.rs` (7 tests)
**Test cases:**
- ✅ `if: ${{ github.ref == 'refs/heads/main' }}`
- ✅ Complex conditionals with logical operators
- ✅ Error on missing `${{ }}` wrapper
- ✅ Error on invalid expression syntax

### 24. JobIfExpressionRule
Same as StepIfExpressionRule, but for job-level `if:` conditions.

**Tests:** `validation_job_if_expression.rs` (6 tests)
**Test cases:**
- ✅ `if: ${{ github.ref == 'refs/heads/main' }}`
- ✅ Job-level conditional expressions
- ✅ Error on invalid expression syntax

### 25. WorkflowCallInputsRule
For reusable workflows (`workflow_call`), validates that declared inputs match their usage and have valid types.

**Tests:** `validation_workflow_call_inputs.rs` (8 tests)
**Test cases:**
- ✅ Properly defined inputs with matching references
- ✅ Error on references to undefined inputs
- ✅ Error on invalid input types

### 26. WorkflowCallSecretsRule
Same idea as WorkflowCallInputsRule, but for `workflow_call` secrets. Makes sure secret references point to something that's actually declared.

**Tests:** `validation_workflow_call_secrets.rs` (6 tests)
**Test cases:**
- ✅ Properly defined secrets with matching references
- ✅ Error on references to undefined secrets

### 27. ReusableWorkflowCallRule
Validates the `uses:` field when calling a reusable workflow (as opposed to an action). The format is `owner/repo/.github/workflows/file.yml@ref`.

**Tests:** `validation_reusable_workflow_call.rs` (7 tests)
**Test cases:**
- ✅ `uses: owner/repo/.github/workflows/reusable.yml@main`
- ✅ Workflow call with input passthrough
- ✅ Error on invalid format
- ✅ Error on missing required fields

### 28. WorkflowCallOutputsRule
Checks that `workflow_call` output mappings point to jobs and job outputs that actually exist.

**Tests:** `validation_workflow_call_outputs.rs` (6 tests)
**Test cases:**
- ✅ Output referencing a valid job output
- ✅ Error on references to non-existent jobs
- ✅ Error on references to non-existent job outputs

### 29. StepContinueOnErrorRule
Simple type check: `continue-on-error` must be a boolean (or an expression). Strings like `"true"` are a common mistake.

**Tests:** `validation_step_continue_on_error.rs` (4 tests)
**Test cases:**
- ✅ `continue-on-error: true` and `continue-on-error: false`
- ✅ Error on `continue-on-error: "true"` (string)

### 30. StepTimeoutRule
Like TimeoutRule (#15), but at the step level. Same constraints: must be a positive number.

**Tests:** `validation_step_timeout.rs` (6 tests)
**Test cases:**
- ✅ `timeout-minutes: 30` on a step
- ✅ Error on negative values and zero

### 31. StepShellRule
Validates the `shell:` field on `run:` steps. Recognizes the built-in shells (`bash`, `pwsh`, `python`, `sh`, `cmd`, `powershell`) and allows custom shell strings.

**Tests:** `validation_step_shell.rs` (8 tests)
**Test cases:**
- ✅ `shell: bash`, `shell: pwsh`, `shell: python`
- ✅ Custom shell with inline script
- ✅ Error on invalid shell syntax

### 32. StepWorkingDirectoryRule
Basic sanity check on `working-directory` paths. Accepts relative and absolute paths, warns on anything that looks malformed.

**Tests:** `validation_step_working_directory.rs` (4 tests)
**Test cases:**
- ✅ `working-directory: ./src`
- ✅ Absolute and relative paths
- ✅ Warning on suspicious path formats

### 33. ArtifactValidationRule
Validates `actions/upload-artifact` and `actions/download-artifact` usage, checking that artifact names and paths are present and well-formed.

**Tests:** `validation_artifact.rs` (5 tests)
**Test cases:**
- ✅ Artifact with valid name and path
- ✅ Error on empty artifact name
- ✅ Warning on potentially invalid paths

### 34. EventPayloadValidationRule
Goes deeper than WorkflowTriggerRule by validating event-specific fields -- for example, making sure `branches` filters are only used on events that support them.

**Tests:** `validation_event_payload.rs` (18 tests)
**Test cases:**
- ✅ Event-specific fields matching their event types
- ✅ Error on fields that don't belong to a given event type
- ✅ Error on invalid event type values

### 35. RunnerLabelRule
Validates `runs-on` labels against known GitHub-hosted runners. Self-hosted labels are allowed too, but unknown labels get a warning since they're a frequent source of "workflow queued forever" issues.

**Tests:** `validation_runner_label.rs` (6 tests)
**Test cases:**
- ✅ Known runners: `ubuntu-latest`, `windows-latest`, `macos-latest`, etc.
- ✅ Self-hosted runner labels
- ✅ Warning on unrecognized labels

### 36. StepEnvValidationRule
Validates environment variable names and values at the step level. Env var names must follow the standard `[A-Z_][A-Z0-9_]*` convention.

**Tests:** `validation_step_env.rs` (8 tests)
**Test cases:**
- ✅ `env: { VALID_NAME: value }`
- ✅ Environment variables with expression values
- ✅ Error on invalid env var name format

### 37. JobContainerRule
Validates `container:` and `services:` blocks on jobs. Checks image names, port mappings, and the overall structure.

**Tests:** `validation_job_container.rs` (6 tests)
**Test cases:**
- ✅ Container with valid image and ports
- ✅ Services configuration with multiple containers
- ✅ Error on malformed port mappings
- ✅ Error on invalid container configuration

### 38. StepNameRule
Validates the optional `name:` field on steps. It's not required, but if present it shouldn't be empty or excessively long.

**Tests:** `validation_step_name.rs` (6 tests)
**Test cases:**
- ✅ Step with a descriptive name
- ✅ Name containing expressions
- ✅ Warning on empty name
- ✅ Warning on very long names

### 39. DefaultsValidationRule
Validates `defaults.run` at both the workflow and job levels. Mostly checks that `shell` and `working-directory` contain sensible values.

**Tests:** `validation_defaults.rs` (6 tests)
**Test cases:**
- ✅ Defaults with valid shell and working-directory
- ✅ Workflow-level and job-level defaults
- ✅ Error on invalid shell in defaults
- ✅ Error on invalid working-directory in defaults

### 40. DeprecatedCommandsRule
Warns when a `run:` block uses deprecated workflow commands. GitHub removed support for `::set-output`, `::save-state`, `::set-env`, and `::add-path` due to security concerns -- workflows using them will fail or behave unexpectedly.

**Tests:** `validation_deprecated_commands.rs` (7 tests)
**Test cases:**
- ✅ Detects `::set-output` and suggests `GITHUB_OUTPUT`
- ✅ Detects `::save-state` and suggests `GITHUB_STATE`
- ✅ Detects `::set-env` and suggests `GITHUB_ENV`
- ✅ Detects `::add-path` and suggests `GITHUB_PATH`
- ✅ Handles multiline `run:` blocks correctly
- ✅ Detects multiple deprecated commands in a single block
- ✅ No false positives on modern `GITHUB_OUTPUT`-style syntax

### 41. ScriptInjectionRule
Detects potential script injection vulnerabilities in `run:` blocks. When untrusted inputs (like PR titles, issue bodies, or branch names) are interpolated directly via `${{ }}` expressions, an attacker can inject arbitrary shell commands.

**Tests:** `validation_script_injection.rs` (9 tests)
**Test cases:**
- ✅ Detects `${{ github.event.pull_request.title }}` in run blocks
- ✅ Detects `${{ github.event.pull_request.body }}`
- ✅ Detects `${{ github.event.pull_request.head.ref }}`
- ✅ Detects `${{ github.event.issue.body }}`
- ✅ Detects `${{ github.event.comment.body }}`
- ✅ Detects `${{ github.head_ref }}`
- ✅ No false positive on `${{ github.sha }}` (safe context)
- ✅ No false positive on env var or secrets references
- ✅ Recognizes safe usage through environment variable indirection

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
├── validation_step.rs                    ✅ (8 tests)
├── validation_step_name.rs               ✅ (6 tests)
├── validation_step_id_uniqueness.rs      ✅ (7 tests)
├── validation_step_if_expression.rs      ✅ (7 tests)
├── validation_step_output_reference.rs   ✅ (9 tests)
├── validation_step_continue_on_error.rs  ✅ (4 tests)
├── validation_step_timeout.rs            ✅ (6 tests)
├── validation_step_shell.rs              ✅ (8 tests)
├── validation_step_working_directory.rs  ✅ (4 tests)
├── validation_step_env.rs                ✅ (8 tests)
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
├── validation_event_payload.rs           ✅ (17 tests)
├── validation_deprecated_commands.rs     ✅ (7 tests)
├── validation_script_injection.rs        ✅ (9 tests)
├── validation_comment_handling.rs        ✅ (8 tests)
└── validation_benchmark_fixtures.rs      ✅ (10 tests)
```

**Total: 347 tests across 44 test files (all passing)**

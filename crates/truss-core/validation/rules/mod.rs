//! Validation rules for GitHub Actions workflows.

pub mod non_empty;
pub mod schema;
pub mod syntax;
pub mod workflow_trigger;
pub mod job_name;
pub mod job_needs;
pub mod step;
pub mod expression;
pub mod permissions;
pub mod environment;
pub mod workflow_name;
pub mod matrix;
pub mod runs_on;
pub mod secrets;
pub mod timeout;
pub mod workflow_inputs;

// Re-export all rules for easy importing
pub use non_empty::NonEmptyRule;
pub use schema::GitHubActionsSchemaRule;
pub use syntax::SyntaxRule;
pub use workflow_trigger::WorkflowTriggerRule;
pub use job_name::JobNameRule;
pub use job_needs::JobNeedsRule;
pub use step::StepValidationRule;
pub use expression::ExpressionValidationRule;
pub use permissions::PermissionsRule;
pub use environment::EnvironmentRule;
pub use workflow_name::WorkflowNameRule;
pub use matrix::MatrixStrategyRule;
pub use runs_on::RunsOnRequiredRule;
pub use secrets::SecretsValidationRule;
pub use timeout::TimeoutRule;
pub use workflow_inputs::WorkflowInputsRule;


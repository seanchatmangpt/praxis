pub mod air;
pub mod compile;
pub mod lower;
pub mod normalizer;
pub mod parse;
pub mod resolve;
pub mod temporal;

use thiserror::Error;

/// Core Refusal type for wasm4pm-arazzo, following strict core team discipline.
///
/// # PROJ-754 taxonomy mapping (Rail B: Arazzo -> AIR compiler refusal coverage)
///
/// PROJ-754 (`docs/jira/v26.7.11/tickets/index.md`) requires a specific, tested `Refusal`
/// variant for every construct `crate::lower::lower_description` cannot compile. Two of the
/// five required constructs are already covered by pre-existing, pre-PROJ-754 variants and are
/// deliberately reused rather than duplicated (a fresh variant carrying the same meaning would
/// itself violate the "specific, not generic" rule this taxonomy exists to enforce):
///
/// - **missing operationId** (a step declaring none of `operationId`/`operationPath`/
///   `channelPath`/`workflowId`) -> [`Refusal::MissingIdentity`], tested by
///   `lower::tests::refuses_step_with_no_identity` and
///   `tests/end_to_end_lowering.rs::hand_written_production_arazzo_without_operation_identity_is_refused_at_lowering`.
/// - **unresolvable reference** (a success/failure action `$ref` that does not dereference
///   against the document's own `components`, or -- new in PROJ-754 -- a `Step.depends_on`
///   entry naming a step id absent from the workflow) -> [`Refusal::UnresolvableReference`].
///
/// The remaining three constructs are new in PROJ-754, added below:
/// [`Refusal::CyclicStepDependency`], [`Refusal::UnsupportedCriterion`],
/// [`Refusal::MalformedRetryPolicy`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Refusal {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("URI resolution error: {0}")]
    UriResolution(String),
    #[error("Invalid arazzo version: {0}")]
    InvalidVersion(String),
    #[error("Missing identity: {0}")]
    MissingIdentity(String),
    #[error("Cyclic dependency or unresolvable cross-reference: {0}")]
    UnresolvableReference(String),
    #[error("Invalid workflow: {0}")]
    InvalidWorkflow(String),
    /// A workflow's `Step.depends_on` graph contains a cycle (including a step naming itself):
    /// no valid readiness order exists, so the workflow can never start. Fired by
    /// `lower::validate_step_dependencies`, called from `lower::lower_workflow` before any step
    /// is lowered to AIR.
    #[error("Cyclic step dependency: {0}")]
    CyclicStepDependency(String),
    /// A `Criterion.type` (Arazzo `expressionType`) names a selector shape -- JSONPath, XPath,
    /// regex, or a versioned selector object -- that this bridge has no evaluator for. Only the
    /// spec's default (`simple`, or `type` omitted) lowers to an AIR-evaluable condition; see
    /// `lower::classify_criterion`. Fired at lowering time rather than left to silently
    /// misclassify an unevaluatable selector as a plain boolean condition downstream.
    #[error("Unsupported criterion expression shape: {0}")]
    UnsupportedCriterion(String),
    /// A step's `timeout` (milliseconds) or a `type: retry` failure action's `retryLimit` /
    /// `retryAfter` carries a value that can never be satisfied: a zero-millisecond timeout, a
    /// zero retry limit on an action whose entire purpose is to retry, or a negative/
    /// non-finite `retryAfter` delay. Fired by `lower::validate_step_timeout` /
    /// `lower::validate_retry_policy`.
    #[error("Malformed timeout or retry policy: {0}")]
    MalformedRetryPolicy(String),
}

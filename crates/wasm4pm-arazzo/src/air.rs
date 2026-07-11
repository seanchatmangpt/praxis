use bumpalo::collections::{String as BumpString, Vec as BumpVec};

/// The root program in Arazzo Intermediate Representation (AIR).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirProgram<'bump> {
    /// Workflows defined in this program.
    pub workflows: BumpVec<'bump, AirWorkflow<'bump>>,
}

/// An individual workflow within an `AirProgram`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirWorkflow<'bump> {
    /// The name of the workflow.
    pub name: BumpString<'bump>,
    /// Ordered steps that compose the workflow.
    pub steps: BumpVec<'bump, AirStep<'bump>>,
}

/// A discrete step executed within a workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirStep<'bump> {
    /// Step identifier or name.
    pub name: BumpString<'bump>,
    /// Target execution environment or endpoint.
    pub target: AirTarget<'bump>,
    /// The action to be performed on the target.
    pub action: AirAction<'bump>,
    /// Routing rules evaluated after this step's action completes successfully, in the
    /// Arazzo document's own declaration order (first-match-wins is a caller/runtime
    /// concern, not encoded here). Empty when the source Arazzo step declared no
    /// `onSuccess` entries (and, for a workflow-level step list, no workflow-level
    /// `successActions`).
    pub on_success: BumpVec<'bump, AirRouting<'bump>>,
    /// Routing rules evaluated after this step's action fails, same ordering contract as
    /// [`AirStep::on_success`].
    pub on_failure: BumpVec<'bump, AirRouting<'bump>>,
}

/// Describes the destination or endpoint of a step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirTarget<'bump> {
    /// Target URL or identifier.
    pub url: BumpString<'bump>,
    /// Protocol method (e.g., GET, POST).
    pub method: BumpString<'bump>,
}

/// Describes the inputs provided to and outputs expected from an operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirAction<'bump> {
    /// Given inputs for the action.
    pub inputs: BumpVec<'bump, AirExpr<'bump>>,
    /// Expected outputs or bindings resulting from the action.
    pub outputs: BumpVec<'bump, AirExpr<'bump>>,
}

/// Expressions for representing values or references within AIR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AirExpr<'bump> {
    /// A constant literal value.
    Literal(BumpString<'bump>),
    /// A reference to a variable or binding.
    Variable(BumpString<'bump>),
}

/// One routing rule reached after a step's action resolves: what happens next (end,
/// retry, or transfer control), gated by zero or more criteria.
///
/// Lowered from Arazzo's `SuccessAction`/`FailureAction` objects
/// (`wasm4pm_compat::arazzo::{SuccessAction, FailureAction}`) by
/// `crate::lower::lower_description`; see that module for the exact field mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirRouting<'bump> {
    /// The routing rule's own name (Arazzo `SuccessAction.name` / `FailureAction.name`).
    pub name: BumpString<'bump>,
    /// What happens when this routing rule is taken.
    pub outcome: AirRoutingOutcome<'bump>,
    /// Criteria (Arazzo `Criterion.condition` expression text, verbatim) that must all
    /// hold for this routing rule to apply. Empty means unconditional.
    pub criteria: BumpVec<'bump, AirExpr<'bump>>,
}

/// What a routing rule does when it applies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AirRoutingOutcome<'bump> {
    /// End the workflow (Arazzo `type: end`).
    End,
    /// Retry the current step (Arazzo failure-action `type: retry`).
    Retry,
    /// Transfer control to a named step in the current workflow (Arazzo `type: goto`
    /// with `stepId` set).
    GotoStep(BumpString<'bump>),
    /// Transfer control to a named workflow (Arazzo `type: goto` with `workflowId` set).
    GotoWorkflow(BumpString<'bump>),
}

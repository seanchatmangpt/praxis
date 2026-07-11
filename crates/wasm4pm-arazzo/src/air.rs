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

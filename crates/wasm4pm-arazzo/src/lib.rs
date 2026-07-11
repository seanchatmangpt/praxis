pub mod air;
pub mod compile;
pub mod parse;
pub mod quantum;
pub mod resolve;
pub mod temporal;

use thiserror::Error;

/// Core Refusal type for wasm4pm-arazzo, following strict core team discipline.
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
}

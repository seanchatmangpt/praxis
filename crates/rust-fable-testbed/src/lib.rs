//! `rust-fable-testbed` — a deterministic Rust-eval pipeline for Claude models.
//!
//! Task specs are authored once as RDF/Turtle ontologies ([`spec`]) and serve double
//! duty as:
//!
//! 1. **Eval-harness inputs**: compiled into deterministic, hash-addressed prompts
//!    ([`prompt`]) via `ggen_core::prompt_mfg`, sent to a Claude model
//!    ([`model_client`]), applied to a sandboxed scratch copy of a fixture project
//!    ([`sandbox`]), and scored through a `cargo build`/`test`/`clippy`/safety-audit
//!    pipeline ([`pipeline`]) built on `praxis_core::verify::VerifyGuard`. Every run
//!    appends a BLAKE3-chained JSON-lines receipt ([`receipt`]).
//! 2. **Spec-driven-dev inputs**: the same task ontology renders `spec.md`/`tasks.md`
//!    ([`specdriven`]) for a human- or model-driven spec -> plan -> tasks -> implement
//!    workflow, sharing the identical verification pipeline.
//!
//! One underlying system, two entry points.

pub mod model_client;
pub mod pipeline;
pub mod prompt;
pub mod prompt_mfg;
pub mod receipt;
pub mod sandbox;
pub mod spec;
pub mod specdriven;

/// Crate-wide error type.
///
/// Individual modules define narrower error variants where useful (e.g.
/// [`model_client::ModelError`]) and convert into this type at module boundaries via
/// `?` + `#[from]`, or the caller wraps a module-specific error as a [`Error::Other`]
/// leaf when a `#[from]` conversion isn't warranted.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failure loading or parsing a task spec `.ttl` file.
    #[error("spec error: {0}")]
    Spec(String),

    /// Failure compiling a task spec into a prompt.
    #[error("prompt compile error: {0}")]
    Prompt(#[from] prompt_mfg::PromptError),

    /// Failure talking to the model API.
    #[error("model client error: {0}")]
    Model(String),

    /// Failure staging or writing into the sandbox.
    #[error("sandbox error: {0}")]
    Sandbox(String),

    /// Failure appending or reading the receipt ledger.
    #[error("receipt error: {0}")]
    Receipt(String),

    /// Underlying I/O failure.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Underlying JSON (de)serialization failure.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Catch-all for context-rich failures from `anyhow`.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, Error>;

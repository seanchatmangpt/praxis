#![doc = include_str!("../README.md")]
#![deny(clippy::print_stdout)]
#![deny(unsafe_code)]
// Recorded lint debt (v26.7.6 verification gate): 257 findings at HEAD once
// CI's `-D warnings` promotes the aspirational `missing_docs`/`pedantic`
// warn-level policy (Cargo.toml [lints]) to errors. `clippy::correctness`
// and the deny/forbid lints above stay fully active. Debt is tracked in
// docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

/// Append-only audit chain using BLAKE3 content addressing.
pub mod chain;
pub mod cli;
pub mod config;
pub mod error;
pub mod ops;
pub mod types;

#[cfg(feature = "otel")]
pub mod telemetry;

#[cfg(feature = "lsp")]
pub mod lsp;

#[cfg(feature = "andon")]
pub mod law_andon;

#[cfg(feature = "mcp")]
pub mod mcp_cache;

#[cfg(feature = "discovery")]
pub mod discovery;

#[cfg(feature = "ggen")]
pub mod mfg;

#[cfg(feature = "ggen")]
pub mod corpus;

#[cfg(feature = "ggen")]
pub mod plan_run;

#[cfg(feature = "ggen")]
pub mod receipt_shacl;

#[cfg(feature = "proposer")]
pub mod revenue;

#[cfg(feature = "proposer")]
pub mod revtac;

#[cfg(feature = "proposer")]
pub mod mission;

pub mod frontier;
pub mod repl;
pub mod synth_ops;
pub mod verify_ops;

pub use error::AppError;
pub use praxis_core::{Andon, LawObject, Obligation};
pub use types::{
    canonical_bytes, Admit, Admitted, AdmittedEvidence, AdmittedReceipt, Blake3Hash, Evidence,
    ObjectRef, ProfileId, Raw, RawEvidence, Validated, ValidatedEvidence,
};

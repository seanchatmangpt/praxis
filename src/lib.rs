#![doc = include_str!("../README.md")]
#![deny(clippy::print_stdout)]
#![deny(unsafe_code)]

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
pub mod receipt_shacl;

#[cfg(feature = "proposer")]
pub mod revenue;

#[cfg(feature = "proposer")]
pub mod revtac;

#[cfg(feature = "proposer")]
pub mod mission;

pub mod frontier;
pub mod repl;
pub mod verify_ops;

pub use error::AppError;
pub use types::{
    canonical_bytes, Admit, Admitted, AdmittedEvidence, AdmittedReceipt, Blake3Hash, Evidence,
    ObjectRef, ProfileId, Raw, RawEvidence, Validated, ValidatedEvidence,
};
pub use praxis_core::{Andon, LawObject, Obligation};

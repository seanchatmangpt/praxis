#![doc = include_str!("../README.md")]
#![deny(clippy::print_stdout)]
#![deny(unsafe_code)]

/// Append-only audit chain using BLAKE3 content addressing.
pub mod chain;
pub mod cli;
pub mod error;
pub mod types;

#[cfg(feature = "otel")]
pub mod telemetry;

#[cfg(feature = "lsp")]
pub mod lsp;

#[cfg(feature = "discovery")]
pub mod discovery;

pub mod repl;

pub use error::AppError;
pub use types::{
    canonical_bytes, Admit, Admitted, AdmittedEvidence, AdmittedReceipt, Blake3Hash, Evidence,
    ObjectRef, ProfileId, Raw, RawEvidence, Validated, ValidatedEvidence,
};
pub use praxis_core::{Andon, LawObject, Obligation};

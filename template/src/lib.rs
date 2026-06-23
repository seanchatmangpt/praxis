#![doc = include_str!("../README.md")]
#![deny(clippy::print_stdout)]
#![deny(unsafe_code)]

pub mod chain;
pub mod cli;
pub mod error;
pub mod types;
pub mod validation;
pub mod discovery;

#[cfg(feature = "otel")]
pub mod telemetry;

#[cfg(feature = "lsp")]
pub mod lsp;

pub mod repl;

pub use error::AppError;
pub use types::{Blake3Hash, ObjectRef, canonical_bytes, ProfileId, Evidence, Admit, Raw, Validated, Admitted, RawEvidence, ValidatedEvidence, AdmittedEvidence, AdmittedReceipt};

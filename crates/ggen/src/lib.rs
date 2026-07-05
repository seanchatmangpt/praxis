#![doc = include_str!("../README.md")]
#![deny(clippy::print_stdout)]
#![deny(unsafe_code)]

pub mod chain;
pub mod config;
pub mod error;
pub mod graph;
pub mod lint;
pub mod pack;
pub mod shell_safety;
pub mod sync;
pub mod template;
pub mod types;
pub mod write;

#[cfg(feature = "otel")]
pub mod telemetry;

#[cfg(feature = "lsp")]
pub mod lsp;

#[cfg(feature = "discovery")]
pub mod discovery;

pub mod repl;
pub mod verbs;
pub mod watch;

pub use error::AppError;
pub use types::{
    canonical_bytes, Admit, Admitted, AdmittedEvidence, AdmittedReceipt, Blake3Hash, Evidence,
    ObjectRef, ProfileId, Raw, RawEvidence, Validated, ValidatedEvidence,
};

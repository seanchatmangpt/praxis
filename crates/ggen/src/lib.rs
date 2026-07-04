#![doc = include_str!("../README.md")]
#![deny(clippy::print_stdout)]
#![deny(unsafe_code)]

pub mod chain;
pub mod cli;
pub mod config;
pub mod error;
pub mod graph;
pub mod lint;
pub mod pack;
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
pub use types::{Blake3Hash, ObjectRef, canonical_bytes, ProfileId, Evidence, Admit, Raw, Validated, Admitted, RawEvidence, ValidatedEvidence, AdmittedEvidence, AdmittedReceipt};

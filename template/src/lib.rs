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
pub use types::{
    Admitted, AdmittedEvidence, AdmittedReceipt, Blake3Hash, Evidence, ONTOLOGY_VERB_IDS,
    ObjectRef, Pending, ProfileId, Raw, RawEvidence, ReceiptRefusal, Rejected, Sealed, State,
    Validated, ValidatedEvidence, Verified, assert_unique_ids, canonical_bytes,
};

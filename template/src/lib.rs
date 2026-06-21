#![doc = include_str!("../README.md")]
#![deny(clippy::print_stdout)]
#![deny(unsafe_code)]

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

pub use error::AppError;
pub use types::{Blake3Hash, ObjectRef, canonical_bytes};

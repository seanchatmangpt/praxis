//! Core error types for law object lifecycle.

use thiserror::Error;

use crate::law::Obligation;

/// Errors that can occur during law object operations.
#[derive(Debug, Error, Clone)]
pub enum CoreError {
    /// One or more obligations were not met during judgment.
    #[error("obligations unmet: {0:?}")]
    ObligationUnmet(Vec<Obligation>),

    /// Signature verification failed.
    #[error("signature invalid")]
    SignatureInvalid,

    /// Chain hash verification failed (prev hash mismatch).
    #[error("chain hash mismatch")]
    ChainMismatch,

    /// Payload could not be serialized to canonical bytes for hashing.
    #[error("payload serialization failed: {0}")]
    SerializationFailed(String),

    /// Ed25519 signing failed (missing/invalid key, or the underlying
    /// signing primitive returned an error).
    #[error("signing failed: {0}")]
    SigningFailed(String),

    /// A hex-encoded field (payload/chain hash) failed to decode: wrong
    /// length, invalid hex characters, or not exactly 32 bytes.
    #[error("hex decode failed: {0}")]
    HexDecodeFailed(String),

    /// Filesystem I/O failed while reading or writing the receipt ledger.
    #[error("io error: {0}")]
    Io(String),
}

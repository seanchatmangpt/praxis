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

    /// PROJ-752 (PRD.md sec.7.4-7.5, `A_z = T(Q(W))`): the Tera template
    /// engine refused to parse or render the Rail A Arazzo-manufacture
    /// template. Carries Tera's own error text (formatted with `{e:?}` at
    /// the call site so the underlying cause -- e.g. a malformed template
    /// expression or a JSON-encoding failure on a projected value -- is not
    /// silently dropped).
    #[error("arazzo template render failed: {0}")]
    TemplateRenderFailed(String),

    /// PROJ-752: a Q-stage `ProjectionRow` referenced a `childModel`/region
    /// element IRI that does not appear as the `elementId` of any row in
    /// the same projection result set -- an internal-consistency violation
    /// of the row set being rendered. PROJ-751's own tests cover round-trip
    /// consistency of the real `run_render_model_projection` output; this
    /// variant exists so a malformed/hand-built row set fails loud here
    /// too, rather than silently manufacturing an incomplete Arazzo
    /// document.
    #[error("arazzo projection: unresolved projected element {0}")]
    UnresolvedProjectionElement(String),
}

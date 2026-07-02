//! `law` verb dispatcher — judge, admit, receipt, show, promote.
//!
//! Dispatches to the real admission/receipt/standing crates for the praxis
//! law object lifecycle: Raw → Validated → Admitted → Receipted.
//!
//! Each subcommand accepts a JSON payload via stdin or argument, wraps it in
//! a [`praxis_core::LawObject`], runs [`praxis_core::DefaultLaw`]'s
//! `Judge`/`Admit` implementations (and, when an `atom`/`rule` is present,
//! prolog8 admission), and returns the result as JSON.
//!
//! Malformed input (bad JSON, bad hex, an unparseable standing name, an
//! `atom`/`rule` without a `catalog`) is a hard `Err`. A *domain* denial
//! (halted obligations, a prolog8 rejection, a missing auditor) is `Ok(json)`
//! with a `status`/`verdict` field describing the denial.
//!
//! These verbs are thin wrappers: all pure logic (input schemas, parsing,
//! and the `*_payload` functions) lives in [`my_conforming_project::ops`],
//! the single source of truth shared with future non-CLI callers (e.g. an
//! MCP server).

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::{arg, verb};
use my_conforming_project::ops;
use serde_json::Value;

/// Judge a raw LawObject: Raw → Validated or Halted.
///
/// Reads a JSON payload, wraps it in a `LawObject`, applies `DefaultLaw`'s
/// `Judge` to validate obligations (and, if `atom`/`rule` are present, runs
/// prolog8 admission), and returns the Validated/Halted verdict.
#[verb]
pub fn judge(
    payload: String,
    #[arg(default_value = "default", help = "Policy name to judge against")] law: String,
) -> Result<Value> {
    ops::judge_payload(&payload, &law).map_err(NounVerbError::argument_error)
}

/// Admit a validated LawObject: Validated → Admitted.
///
/// Transitions a payload through judgment and admission via `DefaultLaw`.
/// Denial (halted at judge or admit) is returned as `Ok` with a `denied` shape.
#[verb]
pub fn admit(
    payload: String,
    #[arg(default_value = "default", help = "Admission policy name")] policy: String,
) -> Result<Value> {
    ops::admit_payload(&payload, &policy).map_err(NounVerbError::argument_error)
}

/// Generate a receipt for an admitted LawObject.
///
/// Runs the full judge → admit → receipt pipeline and produces a
/// BLAKE3 chain hash bound to both the payload and the previous link.
#[verb]
pub fn receipt(payload: String) -> Result<Value> {
    ops::receipt_payload(&payload).map_err(NounVerbError::argument_error)
}

/// Show a law object receipt as JSON or human-readable format.
#[verb]
pub fn show(
    payload: String,
    #[arg(default_value = "json", help = "Output format: json or text")] format: String,
) -> Result<Value> {
    ops::show_payload(&payload, &format).map_err(NounVerbError::argument_error)
}

/// Promote a law object via the `BreedStanding` ladder.
///
/// Promotions to `Replayable` or `Certified` require a non-empty `auditor`.
#[verb]
pub fn promote(
    payload: String,
    #[arg(default_value = "", help = "Auditor name endorsing the promotion")] auditor: String,
) -> Result<Value> {
    ops::promote_payload(&payload, &auditor).map_err(NounVerbError::argument_error)
}

/// Verify an ed25519 signature over a receipt's chain hash.
///
/// Input: `{"chain_hash": "<64 hex>", "signed_receipt": {...}, "verifying_key": "<64 hex, optional>"}`.
/// `signed_receipt` is the object `receipt` emits under this same feature
/// (chain_hash hex, base64 signature, verifying_key hex). If `verifying_key`
/// is omitted, verification trusts the key embedded in `signed_receipt`
/// (integrity only); if present, it also checks authenticity against that
/// specific key. Returns `Ok(json)` with `"status": "valid"`/`"invalid"` in
/// both the success and failure case — a signature that doesn't verify is a
/// legitimate answer, not an error. Malformed input (bad hex, unparseable
/// `signed_receipt`) is a hard `Err`.
#[cfg(feature = "law-signed")]
#[verb("verify-signature")]
pub fn verify_signature(payload: String) -> Result<Value> {
    ops::verify_signature_payload(&payload).map_err(NounVerbError::argument_error)
}

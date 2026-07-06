//! `receipt` verb dispatcher — issue, validate, show, replay, export-ocel.
//!
//! Distinct from `law receipt` (`src/verbs/law.rs`), which runs the judge ->
//! admit -> receipt pipeline and returns a one-shot JSON receipt: these verbs
//! persist the resulting `praxis_core::ReceiptRecord`s to an append-only
//! JSONL ledger and operate on that ledger (validate/show/replay/export
//! -ocel). `praxis receipt` with no verb defaults to `validate` (see
//! `inject_default_verbs` in `src/main.rs`).
//!
//! These are thin wrappers: all pure logic lives in
//! [`my_conforming_project::ops`]'s `receipt_*_payload` functions, the same
//! single source of truth shared with future non-CLI callers (e.g. an MCP
//! server).

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
use my_conforming_project::ops;
use serde_json::Value;

/// Resolve the receipts ledger directory: the admitted `PraxisConfig`'s
/// `receipts.dir` if config loads cleanly, otherwise the hardcoded default
/// `"receipts"` (config admission failures — e.g. running outside a checked
/// -out praxis tree — must never block a receipt operation from at least
/// falling back to a sane default).
fn receipts_dir() -> String {
    my_conforming_project::config::config()
        .map(|admitted| admitted.value().receipts.dir.clone())
        .unwrap_or_else(|_| "receipts".to_string())
}

/// Run the judge -> admit -> receipt pipeline and append the resulting
/// `ReceiptRecord` to the JSONL ledger.
///
/// Input is the same `LawInput`+`ReceiptFields` JSON shape as `law receipt`
/// (`{"value": ..., "obligations": [...], "prev_chain_hash": "...", ...}`).
/// `prev_chain_hash`, if given, takes precedence; otherwise the ledger's own
/// last chain hash is used automatically.
#[verb]
pub fn issue(payload: String) -> Result<Value> {
    ops::receipt_issue_payload(&payload, &receipts_dir()).map_err(NounVerbError::argument_error)
}

/// Validate the full receipt ledger: schema, chain-tamper detection
/// (recompute), chain linkage, monotonicity, and POWL token-replay
/// conformance. On success, archives the ledger to `data/validated_receipts/`.
#[verb]
pub fn validate(
    #[arg(
        default_value = "",
        help = "Receipts directory (defaults to the configured receipts.dir)"
    )]
    dir: String,
) -> Result<Value> {
    let dir = if dir.is_empty() { receipts_dir() } else { dir };
    ops::receipt_validate_payload(&dir).map_err(NounVerbError::argument_error)
}

/// Show the trailing `last` receipts in the ledger (all of them if `last` is `0`).
///
/// `last` is typed `u32` rather than `usize`: this macro infers a bare
/// `usize` parameter as a `Count`-action flag (for `-v`/`-vv`-style repeated
/// flags), which would make `--last N` an error (`unexpected value ...`) —
/// see `plan.rs`'s `attention_capacity: u32` for the same convention.
#[verb]
pub fn show(
    #[arg(
        default_value = "0",
        help = "Number of trailing receipts to show (0 = all)"
    )]
    last: u32,
) -> Result<Value> {
    ops::receipt_show_payload(&receipts_dir(), last as usize).map_err(NounVerbError::argument_error)
}

/// Replay every receipt's judge -> admit -> receipt lifecycle through the
/// fixed POWL token model and report per-receipt conformance metrics.
#[verb]
pub fn replay() -> Result<Value> {
    ops::receipt_replay_payload(&receipts_dir()).map_err(NounVerbError::argument_error)
}

/// Export the full receipt ledger as an OCEL 2.0 event log.
#[verb("export-ocel")]
pub fn export_ocel(
    #[arg(
        default_value = "",
        help = "Optional output file path for the OCEL 2.0 JSON"
    )]
    out: String,
) -> Result<Value> {
    let out = if out.is_empty() {
        None
    } else {
        Some(out.as_str())
    };
    ops::receipt_export_ocel_payload(&receipts_dir(), out).map_err(NounVerbError::argument_error)
}

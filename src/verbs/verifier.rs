//! `verify` verb — affidavit-style receipt certification pipeline.
//!
//! Reads a receipts ledger (JSONL, one [`praxis_core::ReceiptRecord`] per
//! line — or, for convenience, a single JSON object/array of records) and
//! runs [`praxis_core::verify::run_pipeline`] over it: `check_format`,
//! `chain_integrity`, `continuity`, `verify_commitments`, `evaluate_profile`,
//! each timed via `VerifyGuard` and reported as one `CheckOutcome` in the
//! resulting `Verdict`. A `decode` outcome (the only I/O in the pipeline,
//! reading and parsing the file) is prepended so the JSON output always
//! documents the same stage sequence a hand ledger inspection would.
//!
//! Distinct from `receipt validate` (`src/verbs/receipt.rs`), which runs
//! `praxis_core::receipt_validator::ReceiptValidator` (schema, chain
//! recompute, linkage, monotonic, POWL token replay — no timing metrics, no
//! named `profile`) directly against the configured receipts directory: this
//! verb is the timing-instrumented, profile-aware certify pipeline over an
//! explicit file path, matching the CPHY_ROADMAP's verification lane.
//!
//! This is a thin wrapper: all decode/pipeline logic lives in
//! [`my_conforming_project::verify_ops`], the single source of truth shared
//! with `tests/snapshots_verbs.rs`.
//!
//! The verdict is always returned as `Ok(json)`, whether accepted or
//! rejected — a rejected verdict is a legitimate, fully-documented answer,
//! not an error. Callers (CI, `dod`) should check the `accepted` field
//! themselves rather than relying on a nonzero process exit code.

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
use my_conforming_project::verify_ops::run_verify_pipeline;
use praxis_core::verify::VerifyMetrics;
use serde_json::Value;

fn print_metrics(metrics: &VerifyMetrics, timings: bool) {
    if timings {
        for s in &metrics.stages {
            let status = if s.passed { "PASS" } else { "FAIL" };
            eprintln!(
                "  [{status}] {} ({:.2}ms)",
                s.name,
                s.duration.as_secs_f64() * 1_000.0
            );
        }
    }
    eprintln!("{}", metrics.summary_line());
}

/// Verify a receipts ledger using the affidavit-style certify pipeline
/// (`decode` -> `check_format` -> `chain_integrity` -> `continuity` ->
/// `verify_commitments` -> `evaluate_profile`).
///
/// Returns the full `Verdict` as JSON, whether accepted or rejected;
/// callers should inspect `.accepted` rather than the process exit code.
#[verb]
pub fn verify(
    #[arg(help = "Path to the receipts JSONL file (or a single JSON receipt)")] path: String,
    #[arg(default_value = "default", help = "Verification profile")] profile: String,
    #[arg(help = "Print per-stage timing breakdown to stderr")] timings: bool,
) -> Result<Value> {
    let (verdict, metrics) = run_verify_pipeline(&path, &profile);
    print_metrics(&metrics, timings);
    serde_json::to_value(&verdict)
        .map_err(|e| NounVerbError::execution_error(format!("could not serialize verdict: {e}")))
}

//! BLAKE3-chained receipt ledger — a flat JSONL append log, not the full
//! `law.rs` Raw→Validated→Admitted→Receipted lifecycle (a deliberate v1
//! simplification per the plan; `law.rs`'s receipt pattern is inspiration only, this
//! chain is new code).

use std::io::Write as _;
use std::path::Path;

use serde::{Deserialize, Serialize};

use praxis_core::verify::VerifyMetrics;

use crate::{Error, Result};

/// One entry in the testbed's BLAKE3-chained receipt ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestbedReceipt {
    /// Task identifier this receipt is for.
    pub task_id: String,
    /// Hash of the compiled prompt sent to the model (from `CompiledPrompt::hash`).
    pub prompt_hash: String,
    /// Model identifier actually used.
    pub model: String,
    /// One-line pipeline summary (`VerifyMetrics::summary_line`).
    pub metrics_summary: String,
    /// Chain hash of the previous receipt (`"0".repeat(64)` for the genesis entry).
    pub prev_chain_hash: String,
    /// `blake3(prev_chain_hash || json(task_id, prompt_hash, model, metrics_summary,
    /// prev_chain_hash))`, hex-encoded.
    pub chain_hash: String,
}

/// Payload hashed to produce a receipt's `chain_hash`. A separate (de)serializable
/// type so the hash input excludes `chain_hash` itself.
#[derive(Debug, Serialize)]
struct ReceiptPayload<'a> {
    task_id: &'a str,
    prompt_hash: &'a str,
    model: &'a str,
    metrics_summary: &'a str,
    prev_chain_hash: &'a str,
}

/// Genesis chain hash used when the ledger has no prior entries: 64 `'0'` characters
/// (the same width as a BLAKE3 hex digest).
#[must_use]
pub fn genesis_chain_hash() -> String {
    "0".repeat(64)
}

/// Build a [`TestbedReceipt`] chaining to `prev_chain_hash`.
///
/// `chain_hash = blake3(prev_chain_hash || json({task_id, prompt_hash, model,
/// metrics_summary, prev_chain_hash}))`, hex-encoded.
///
/// # Errors
///
/// Returns [`Error::Json`] if `payload` fails to serialize. In practice this cannot
/// happen for a struct of plain `&str` fields (no maps, no non-finite floats), but the
/// fallible path is threaded through via `?` rather than asserted away with
/// `.expect()`.
pub fn chain_receipt(
    prev_chain_hash: &str, task_id: &str, prompt_hash: &str, model: &str, metrics: &VerifyMetrics,
) -> Result<TestbedReceipt> {
    let metrics_summary = metrics.summary_line();
    let payload = ReceiptPayload { task_id, prompt_hash, model, metrics_summary: &metrics_summary, prev_chain_hash };

    let payload_json = serde_json::to_string(&payload)?;

    let mut preimage = String::with_capacity(prev_chain_hash.len() + payload_json.len());
    preimage.push_str(prev_chain_hash);
    preimage.push_str(&payload_json);
    let chain_hash = blake3::hash(preimage.as_bytes()).to_hex().to_string();

    Ok(TestbedReceipt {
        task_id: task_id.to_string(),
        prompt_hash: prompt_hash.to_string(),
        model: model.to_string(),
        metrics_summary,
        prev_chain_hash: prev_chain_hash.to_string(),
        chain_hash,
    })
}

/// Append `receipt` as one JSON line to `ledger_path`, creating the file if it doesn't
/// exist.
///
/// # Errors
///
/// Returns [`Error::Io`] if the file can't be opened/written, or [`Error::Json`] if
/// serialization fails.
pub fn append_receipt(ledger_path: &Path, receipt: &TestbedReceipt) -> Result<()> {
    let line = serde_json::to_string(receipt)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger_path)
        .map_err(Error::Io)?;
    writeln!(file, "{line}").map_err(Error::Io)?;
    Ok(())
}

/// Read `ledger_path`'s last line and return its `chain_hash`, or
/// [`genesis_chain_hash`] if the ledger doesn't exist yet (or is empty).
///
/// # Errors
///
/// Returns [`Error::Io`] if an existing ledger can't be read, or [`Error::Json`] if
/// its last line isn't a valid [`TestbedReceipt`].
pub fn last_chain_hash(ledger_path: &Path) -> Result<String> {
    if !ledger_path.exists() {
        return Ok(genesis_chain_hash());
    }
    let content = std::fs::read_to_string(ledger_path).map_err(Error::Io)?;
    let Some(last_line) = content.lines().rev().find(|l| !l.trim().is_empty()) else {
        return Ok(genesis_chain_hash());
    };
    let receipt: TestbedReceipt = serde_json::from_str(last_line)?;
    Ok(receipt.chain_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis_core::verify::VerifyGuard;

    fn sample_metrics() -> VerifyMetrics {
        let mut guard = VerifyGuard::new();
        guard.begin_stage("cargo_build");
        guard.end_stage(true);
        guard.finish()
    }

    #[test]
    fn chain_receipt_is_deterministic_for_same_inputs() {
        let metrics = sample_metrics();
        let a = chain_receipt(&genesis_chain_hash(), "t1", "hash1", "claude-opus-4-8", &metrics)
            .expect("chain_receipt");
        let b = chain_receipt(&genesis_chain_hash(), "t1", "hash1", "claude-opus-4-8", &metrics)
            .expect("chain_receipt");
        assert_eq!(a.chain_hash, b.chain_hash);
    }

    #[test]
    fn chain_receipt_differs_when_prev_hash_differs() {
        let metrics = sample_metrics();
        let a = chain_receipt(&genesis_chain_hash(), "t1", "hash1", "claude-opus-4-8", &metrics)
            .expect("chain_receipt");
        let b = chain_receipt("deadbeef", "t1", "hash1", "claude-opus-4-8", &metrics)
            .expect("chain_receipt");
        assert_ne!(a.chain_hash, b.chain_hash);
    }

    #[test]
    fn append_and_read_back_last_chain_hash() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ledger = dir.path().join("testbed_receipts.jsonl");

        assert_eq!(last_chain_hash(&ledger).expect("genesis read"), genesis_chain_hash());

        let metrics = sample_metrics();
        let r1 = chain_receipt(&genesis_chain_hash(), "t1", "h1", "claude-opus-4-8", &metrics)
            .expect("chain_receipt r1");
        append_receipt(&ledger, &r1).expect("append r1");
        assert_eq!(last_chain_hash(&ledger).expect("read after r1"), r1.chain_hash);

        let r2 = chain_receipt(&r1.chain_hash, "t2", "h2", "claude-opus-4-8", &metrics)
            .expect("chain_receipt r2");
        append_receipt(&ledger, &r2).expect("append r2");
        assert_eq!(last_chain_hash(&ledger).expect("read after r2"), r2.chain_hash);
        assert_ne!(r1.chain_hash, r2.chain_hash);
    }
}

//! Pure domain logic backing the `verify` verb: decode a receipts ledger and
//! run `praxis_core::verify::run_pipeline` over it.
//!
//! Kept separate from [`crate::ops`] (which is exclusively `law`/`receipt`
//! noun payload logic) since this is the one place JSONL-vs-single-JSON
//! ledger decoding lives; [`crate::verbs::verifier`] is a thin wrapper that
//! only adds CLI plumbing (arg parsing, stderr timing printout, JSON
//! serialization for the verb return value).

use praxis_core::verify::{run_pipeline, CheckOutcome, StageMetric, Verdict, VerifyMetrics};
use praxis_core::ReceiptRecord;
use serde_json::Value;
use std::time::Instant;

/// Read `path` and parse it into a sequence of [`ReceiptRecord`]s.
///
/// Tries JSONL first (one record per non-empty line, the ledger format
/// [`praxis_core::receipt_store::ReceiptStore`] writes); if any line fails to
/// parse as a standalone record, falls back to treating the whole file as a
/// single JSON value — either a bare `ReceiptRecord` object or a JSON array
/// of them (the shape a hand-written `receipt.json` or an exported batch
/// might use). An empty (or whitespace-only) file decodes to zero records,
/// not an error.
pub fn decode_records(path: &str) -> Result<Vec<ReceiptRecord>, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("could not read {path}: {e}"))?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut jsonl_records = Vec::new();
    let mut jsonl_ok = true;
    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<ReceiptRecord>(line) {
            Ok(record) => jsonl_records.push(record),
            Err(_) => {
                jsonl_ok = false;
                break;
            }
        }
    }
    if jsonl_ok && !jsonl_records.is_empty() {
        return Ok(jsonl_records);
    }

    // Fallback: a single JSON object, or a JSON array of objects.
    let value: Value = serde_json::from_str(trimmed).map_err(|e| format!("invalid JSON in {path}: {e}"))?;
    match value {
        Value::Array(items) => items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                serde_json::from_value::<ReceiptRecord>(item)
                    .map_err(|e| format!("invalid receipt record at index {i}: {e}"))
            })
            .collect(),
        single @ Value::Object(_) => {
            let record: ReceiptRecord =
                serde_json::from_value(single).map_err(|e| format!("invalid receipt record: {e}"))?;
            Ok(vec![record])
        }
        _ => Err(format!("{path}: expected JSONL, a JSON object, or a JSON array of receipt records")),
    }
}

/// Decode `path` and run the affidavit-style verify pipeline against
/// `profile`, returning the resulting [`Verdict`] and [`VerifyMetrics`].
///
/// A decode failure (unreadable file, malformed JSON/JSONL) is itself
/// reported as a single failing `decode` [`CheckOutcome`] rather than an
/// `Err` — there's nothing further to check, but the shape of the answer
/// stays uniform with every other rejection.
pub fn run_verify_pipeline(path: &str, profile: &str) -> (Verdict, VerifyMetrics) {
    let start = Instant::now();
    let decoded = decode_records(path);
    let decode_duration = start.elapsed();
    let decode_ok = decoded.is_ok();
    let decode_detail = match &decoded {
        Ok(records) => format!("decoded {} record(s) from {path}", records.len()),
        Err(e) => e.clone(),
    };
    let decode_outcome =
        CheckOutcome { stage: "decode".to_string(), passed: decode_ok, detail: decode_detail.clone() };
    let decode_stage_metric =
        StageMetric { name: "decode".to_string(), passed: decode_ok, duration: decode_duration };

    let records = match decoded {
        Ok(records) => records,
        Err(_) => {
            let metrics = VerifyMetrics {
                stages: vec![decode_stage_metric],
                total_duration: decode_duration,
                passed_count: 0,
                failed_count: 1,
            };
            let verdict = Verdict {
                accepted: false,
                profile: profile.to_string(),
                outcomes: vec![decode_outcome],
                reason: Some(decode_detail),
            };
            return (verdict, metrics);
        }
    };

    let (mut verdict, mut metrics) = run_pipeline(&records, profile);
    verdict.outcomes.insert(0, decode_outcome);
    metrics.stages.insert(0, decode_stage_metric);
    metrics.total_duration += decode_duration;
    metrics.passed_count += 1; // decode always passed on this branch
    (verdict, metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_ledger(records: &[ReceiptRecord]) -> tempfile::NamedTempFile {
        let file = tempfile::NamedTempFile::new().expect("tempfile");
        let mut body = String::new();
        for r in records {
            body.push_str(&serde_json::to_string(r).expect("serialize record"));
            body.push('\n');
        }
        std::fs::write(file.path(), body).expect("write ledger");
        file
    }

    fn chained_records(n: u64) -> Vec<ReceiptRecord> {
        let mut records = Vec::new();
        let mut prev = [0u8; 32];
        for i in 1..=n {
            let payload_hash_hex = format!("{i:02x}").repeat(32)[..64].to_string();
            let mut record = ReceiptRecord {
                version: 1,
                instruction_id: i,
                activity_idx: 0,
                activity: None,
                node_kind: 0,
                ts_ns: i * 1000,
                payload_hash_hex,
                prev_chain_hash_hex: hex::encode(prev),
                chain_hash_hex: String::new(),
                andon: praxis_core::Andon::Green,
                obligation_count: 0,
                object_ids: vec![format!("law:instr{i}")],
            };
            let chain_hash = record.recompute_chain_hash().expect("recompute");
            record.chain_hash_hex = hex::encode(chain_hash);
            prev = chain_hash;
            records.push(record);
        }
        records
    }

    #[test]
    fn accepts_a_lawful_jsonl_ledger() {
        let records = chained_records(3);
        let file = write_ledger(&records);
        let (verdict, metrics) =
            run_verify_pipeline(file.path().to_str().expect("utf8 path"), "default");
        assert!(verdict.accepted, "verdict: {verdict:?}");
        assert_eq!(verdict.outcomes.len(), 6, "decode + 5 core stages");
        assert_eq!(metrics.stages.len(), 6);
    }

    #[test]
    fn rejects_a_tampered_chain_hash() {
        let mut records = chained_records(3);
        records[1].chain_hash_hex = "ff".repeat(32);
        let file = write_ledger(&records);
        let (verdict, _metrics) =
            run_verify_pipeline(file.path().to_str().expect("utf8 path"), "default");
        assert!(!verdict.accepted);
        assert!(verdict.reason.is_some());
    }

    #[test]
    fn missing_file_is_a_failing_decode_outcome_not_a_hard_error() {
        let (verdict, metrics) = run_verify_pipeline("/no/such/receipts.jsonl", "default");
        assert!(!verdict.accepted);
        assert_eq!(verdict.outcomes.len(), 1);
        assert_eq!(verdict.outcomes[0].stage, "decode");
        assert!(!verdict.outcomes[0].passed);
        assert_eq!(metrics.stages.len(), 1);
    }

    #[test]
    fn empty_file_decodes_to_zero_records_and_still_accepts() {
        let file = tempfile::NamedTempFile::new().expect("tempfile");
        let (verdict, _metrics) =
            run_verify_pipeline(file.path().to_str().expect("utf8 path"), "default");
        assert!(verdict.accepted, "verdict: {verdict:?}");
    }

    #[test]
    fn single_json_object_fallback_is_accepted() {
        let records = chained_records(1);
        let file = tempfile::NamedTempFile::new().expect("tempfile");
        let json = serde_json::to_string_pretty(&records[0]).expect("serialize");
        std::fs::write(file.path(), json).expect("write");
        let (verdict, _metrics) =
            run_verify_pipeline(file.path().to_str().expect("utf8 path"), "default");
        assert!(verdict.accepted, "verdict: {verdict:?}");
    }

    #[test]
    fn unknown_profile_is_rejected() {
        let records = chained_records(1);
        let file = write_ledger(&records);
        let (verdict, _metrics) =
            run_verify_pipeline(file.path().to_str().expect("utf8 path"), "nonstandard");
        assert!(!verdict.accepted);
        assert_eq!(verdict.profile, "nonstandard");
    }
}

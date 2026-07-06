//! Pure domain logic backing the `verify` verb: decode a receipts ledger and
//! run `praxis_core::verify::run_pipeline` over it.
//!
//! Kept separate from [`crate::ops`] (which is exclusively `law`/`receipt`
//! noun payload logic) since this is the one place JSONL-vs-single-JSON
//! ledger decoding lives; [`crate::verbs::verifier`] is a thin wrapper that
//! only adds CLI plumbing (arg parsing, stderr timing printout, JSON
//! serialization for the verb return value).

use std::time::Instant;

use praxis_core::{
    verify::{run_pipeline, CheckOutcome, StageMetric, Verdict, VerifyMetrics},
    ReceiptRecord,
};
use serde_json::Value;

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
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("could not read {path}: {e}"))?;
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
    let value: Value =
        serde_json::from_str(trimmed).map_err(|e| format!("invalid JSON in {path}: {e}"))?;
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
            let record: ReceiptRecord = serde_json::from_value(single)
                .map_err(|e| format!("invalid receipt record: {e}"))?;
            Ok(vec![record])
        }
        _ => Err(format!(
            "{path}: expected JSONL, a JSON object, or a JSON array of receipt records"
        )),
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
    let decode_outcome = CheckOutcome {
        stage: "decode".to_string(),
        passed: decode_ok,
        detail: decode_detail.clone(),
    };
    let decode_stage_metric = StageMetric {
        name: "decode".to_string(),
        passed: decode_ok,
        duration: decode_duration,
    };

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

/// A Little's Law (`L = λ·W`) snapshot over a parsed receipt ledger.
///
/// Pure function of the records (invariant 3: no wall clock) — the
/// observation window is spanned by the records' own `ts_ns` values and the
/// cycle time comes from their recorded `duration_ms` spans (falling back to
/// mean inter-completion time when no record carries a duration).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct LittleLawSnapshot {
    /// Number of receipts (completions) observed in the window.
    pub completed: usize,
    /// Observation window width: `max(ts_ns) - min(ts_ns)`.
    pub window_ns: u64,
    /// Throughput λ, completions per second over the window.
    pub throughput_per_s: f64,
    /// Cycle time W in milliseconds (mean recorded `duration_ms`, else mean
    /// inter-completion time `window / completed`).
    pub cycle_time_ms: f64,
    /// Work in process L = λ·W (Little's Law), dimensionless item count.
    pub wip: f64,
}

/// Compute a [`LittleLawSnapshot`] from an in-memory receipt ledger.
///
/// Refusals (typed, by name — never a silent default):
/// * empty ledger — there is no window to measure;
/// * zero-width window (`ts_ns` identical across records) — λ is undefined.
pub fn little_law_snapshot(records: &[ReceiptRecord]) -> Result<LittleLawSnapshot, String> {
    if records.is_empty() {
        return Err("little-law snapshot refused: empty receipt ledger".to_string());
    }
    let mut min_ts = u64::MAX;
    let mut max_ts = 0u64;
    let mut dur_sum_ms = 0u64;
    let mut dur_count = 0usize;
    for r in records {
        min_ts = min_ts.min(r.ts_ns);
        max_ts = max_ts.max(r.ts_ns);
        if let Some(d) = r.duration_ms {
            dur_sum_ms += d;
            dur_count += 1;
        }
    }
    let window_ns = max_ts - min_ts;
    if window_ns == 0 {
        return Err(format!(
            "little-law snapshot refused: zero-width observation window \
             ({} record(s) all at ts_ns={min_ts})",
            records.len()
        ));
    }
    let completed = records.len();
    let window_s = window_ns as f64 / 1e9;
    let throughput_per_s = completed as f64 / window_s;
    let cycle_time_ms = if dur_count > 0 {
        dur_sum_ms as f64 / dur_count as f64
    } else {
        (window_ns as f64 / 1e6) / completed as f64
    };
    let wip = throughput_per_s * (cycle_time_ms / 1e3);
    Ok(LittleLawSnapshot {
        completed,
        window_ns,
        throughput_per_s,
        cycle_time_ms,
        wip,
    })
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
                duration_ms: None,
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
    fn little_law_snapshot_refuses_empty_and_zero_window_by_name() {
        let err = little_law_snapshot(&[]).expect_err("empty ledger must refuse");
        assert!(err.contains("empty receipt ledger"), "{err}");
        let mut records = chained_records(2);
        records[1].ts_ns = records[0].ts_ns;
        let err = little_law_snapshot(&records).expect_err("zero window must refuse");
        assert!(err.contains("zero-width observation window"), "{err}");
    }

    #[test]
    fn little_law_snapshot_computes_l_equals_lambda_w() {
        // 4 completions over 3 µs window; each carrying a 2 ms duration.
        let mut records = chained_records(4);
        for r in &mut records {
            r.duration_ms = Some(2);
        }
        let snap = little_law_snapshot(&records).expect("snapshot");
        assert_eq!(snap.completed, 4);
        assert_eq!(snap.window_ns, 3000);
        let lambda = 4.0 / (3000.0 / 1e9);
        assert!((snap.throughput_per_s - lambda).abs() < 1e-6);
        assert!((snap.cycle_time_ms - 2.0).abs() < 1e-12);
        assert!((snap.wip - lambda * 0.002).abs() < 1e-6, "L = λ·W");
    }

    #[test]
    fn little_law_snapshot_falls_back_to_inter_completion_cycle_time() {
        let records = chained_records(4); // duration_ms: None on all
        let snap = little_law_snapshot(&records).expect("snapshot");
        // window 3000 ns = 0.003 ms over 4 completions.
        assert!((snap.cycle_time_ms - 0.003 / 4.0).abs() < 1e-12);
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

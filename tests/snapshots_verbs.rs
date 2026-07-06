//! Deterministic insta snapshots over the pure `*_payload` functions backing
//! the `law` and `verify` verbs.
//!
//! Every input here is fixed (`ts_ns: 42`, a zeroed `prev_chain_hash`), so
//! these snapshots are stable across runs and machines. Fields that are
//! inherently wall-clock-derived (`Andon::Halted`'s `at: u64` millisecond
//! timestamp, stamped by `ops::now_ms()` rather than accepting a caller
//! -supplied value) are redacted rather than asserted verbatim.

use my_conforming_project::{ops, verify_ops};
use prolog8::{Atom8, Catalog, CatalogId, PredicateId, PredicateMeta, PredicateProofPolicy};

/// A single-predicate catalog (`p/0`), matching `ops.rs`'s own test helper.
fn build_catalog() -> Catalog {
    let mut catalog = Catalog::new(CatalogId(1));
    catalog.add_predicate(PredicateMeta {
        pred_id: PredicateId(1),
        label: "p".to_string(),
        arity: 0,
        access_orders: vec![],
        proof_policy: PredicateProofPolicy::OnRequest,
        materialized: false,
    });
    catalog
}

// ── judge ─────────────────────────────────────────────────────────────────

#[test]
fn law_judge_validated() {
    let result = ops::judge_payload(r#"{"value":{"id":1}}"#, "default").expect("should judge");
    insta::assert_json_snapshot!("law_judge_validated", result);
}

#[test]
fn law_judge_halted_blocking_constraint() {
    let payload =
        r#"{"value":{"id":1},"obligations":[{"type":"blocking_constraint","reason":"stop"}]}"#;
    let result = ops::judge_payload(payload, "default").expect("should judge");
    insta::assert_json_snapshot!("law_judge_halted_blocking_constraint", result, {
        ".andon.Halted.at" => "[ts]",
    });
}

#[test]
fn law_judge_prolog8_admitted() {
    let catalog = build_catalog();
    let atom = Atom8::new(PredicateId(1), 0, &[]);
    let payload = serde_json::json!({
        "value": {"id": 1},
        "atom": atom,
        "catalog": catalog,
    });
    let result = ops::judge_payload(&payload.to_string(), "default").expect("should judge");
    insta::assert_json_snapshot!("law_judge_prolog8_admitted", result);
}

// ── admit ─────────────────────────────────────────────────────────────────

#[test]
fn law_admit_admitted() {
    let result = ops::admit_payload(r#"{"value":{"id":1}}"#, "default").expect("should admit");
    insta::assert_json_snapshot!("law_admit_admitted", result);
}

#[test]
fn law_admit_denied_halted() {
    let payload =
        r#"{"value":{"id":1},"obligations":[{"type":"blocking_constraint","reason":"stop"}]}"#;
    let result = ops::admit_payload(payload, "default").expect("should admit");
    insta::assert_json_snapshot!("law_admit_denied_halted", result, {
        ".andon.Halted.at" => "[ts]",
    });
}

// ── receipt ───────────────────────────────────────────────────────────────
//
// The two receipt snapshots pin the UNSIGNED output shape, so they are
// gated `cfg(not(feature = "law-signed"))`: under `law-signed` (part of
// `all-features`) `receipt_payload` signs fail-closed and the output gains
// a key-derived `signed` block that cannot match the unsigned snapshot —
// and skipping the env key would abort with "no signing key available".
// The signed path is covered by `ops::tests` (deterministic env-key guard)
// and by `law verify-signature`'s own tests; the chain-hash determinism
// asserted here is byte-identical in both configurations.

#[cfg(not(feature = "law-signed"))]
#[test]
fn law_receipt_deterministic() {
    let payload = format!(
        r#"{{"value":{{"id":1}},"prev_chain_hash":"{}","ts_ns":42}}"#,
        "11".repeat(32)
    );
    let result = ops::receipt_payload(&payload).expect("should receipt");
    insta::assert_json_snapshot!("law_receipt_deterministic", result);
}

#[cfg(not(feature = "law-signed"))]
#[test]
fn law_receipt_default_prev() {
    // No `prev_chain_hash` supplied: defaults to the zero/genesis hash.
    // `ts_ns` is still fixed so the chain hash is fully deterministic.
    let payload = r#"{"value":{"id":1},"ts_ns":42}"#;
    let result = ops::receipt_payload(payload).expect("should receipt");
    insta::assert_json_snapshot!("law_receipt_default_prev", result);
}

// ── promote ───────────────────────────────────────────────────────────────

#[test]
fn law_promote_promoted() {
    // NAMED -> REGISTERED does not require an auditor.
    let result = ops::promote_payload(r#"{"standing":"NAMED"}"#, "").expect("should promote");
    insta::assert_json_snapshot!("law_promote_promoted", result);
}

#[test]
fn law_promote_denied_needs_auditor() {
    // CANONICAL -> REFUSABLE also does not require an auditor yet, but
    // REFUSABLE -> REPLAYABLE does; use that rung to get a denial.
    let result = ops::promote_payload(r#"{"standing":"REFUSABLE"}"#, "").expect("should promote");
    insta::assert_json_snapshot!("law_promote_denied_needs_auditor", result);
}

// ── verify ────────────────────────────────────────────────────────────────

/// Build `n` chained `ReceiptRecord`s deterministically (fixed `ts_ns` per
/// step, genesis-zero prev hash), and write them as a JSONL ledger.
fn write_chained_ledger(n: u64) -> tempfile::NamedTempFile {
    let mut records = Vec::new();
    let mut prev = [0u8; 32];
    for i in 1..=n {
        let payload_hash_hex = format!("{i:02x}").repeat(32)[..64].to_string();
        let mut record = praxis_core::ReceiptRecord {
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

    let file = tempfile::NamedTempFile::new().expect("tempfile");
    let mut body = String::new();
    for r in &records {
        body.push_str(&serde_json::to_string(r).expect("serialize record"));
        body.push('\n');
    }
    std::fs::write(file.path(), body).expect("write ledger");
    file
}

#[test]
fn verify_verdict_accepted() {
    let file = write_chained_ledger(2);
    let (verdict, _metrics) =
        verify_ops::run_verify_pipeline(file.path().to_str().expect("utf8 path"), "default");
    assert!(verdict.accepted, "verdict: {verdict:?}");
    // The `decode` stage's detail embeds the tempfile's randomly-generated
    // path; redact it so the snapshot is stable across runs/machines.
    insta::assert_json_snapshot!("verify_verdict_accepted", verdict, {
        ".outcomes[0].detail" => "[decode detail]",
    });
}

#[test]
fn verify_verdict_reject_chain_integrity() {
    let file = write_chained_ledger(2);
    let content = std::fs::read_to_string(file.path()).expect("read ledger");
    let mut lines: Vec<serde_json::Value> = content
        .lines()
        .map(|l| serde_json::from_str(l).expect("parse line"))
        .collect();
    // Tamper with the first record's chain_hash directly (not via recompute).
    lines[0]["chain_hash_hex"] = serde_json::json!("ff".repeat(32));
    let tampered = lines
        .iter()
        .map(|v| serde_json::to_string(v).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(file.path(), tampered).expect("write tampered ledger");

    let (verdict, _metrics) =
        verify_ops::run_verify_pipeline(file.path().to_str().expect("utf8 path"), "default");
    assert!(!verdict.accepted);
    insta::assert_json_snapshot!("verify_verdict_reject_chain_integrity", verdict, {
        ".outcomes[0].detail" => "[decode detail]",
    });
}

#[test]
fn verify_verdict_reject_continuity() {
    let file = write_chained_ledger(2);
    let content = std::fs::read_to_string(file.path()).expect("read ledger");
    let mut lines: Vec<serde_json::Value> = content
        .lines()
        .map(|l| serde_json::from_str(l).expect("parse line"))
        .collect();
    // Make the second record's instruction_id go backward.
    let first_instruction_id = lines[0]["instruction_id"].clone();
    lines[1]["instruction_id"] = first_instruction_id;
    let tampered = lines
        .iter()
        .map(|v| serde_json::to_string(v).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(file.path(), tampered).expect("write tampered ledger");

    let (verdict, _metrics) =
        verify_ops::run_verify_pipeline(file.path().to_str().expect("utf8 path"), "default");
    assert!(!verdict.accepted);
    insta::assert_json_snapshot!("verify_verdict_reject_continuity", verdict, {
        ".outcomes[0].detail" => "[decode detail]",
    });
}

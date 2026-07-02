//! Genesis Day 2 — Revenue Physics end-to-end pipe conformance.
//!
//! Drives the whole observation→proposal→plan→admission→receipt chain in
//! process through [`my_conforming_project::revenue::run_demo`] (which itself
//! runs over the shared `ops::*_payload` functions), and pins the invariants
//! Day 2 exits on:
//!
//! - the pipe stays green end to end (a broken seam is a hard `Err`);
//! - it is deterministic under a fixed `ts_ns` — the closing `chain_hash` is
//!   byte-stable and pinned to a literal;
//! - the receipt binds back to the admitted proposal's `proposal_hash`
//!   (AR-9 closure);
//! - the proposer's lawfulness pre-filter and the `law admit` gate **agree**:
//!   an account missing `legal_approved` is both never proposed past
//!   `proposal` *and* refused by `admit` if forced further.
#![cfg(feature = "proposer")]

use my_conforming_project::revenue;
use serde_json::json;

/// Fixed timestamp shared by every run in this file, so the chain hash is
/// reproducible. Matches nothing external — determinism is the only contract.
const TS_NS: u64 = 1_751_328_000_000_000_000;

/// Under `--features law-signed`, `receipt_payload` fails closed without a
/// signing key. Set a fixed test key once so the pipe stays green under
/// `--all-features` too. Signing does not feed back into the chain hash (it
/// signs *over* it), so [`EXPECTED_CHAIN_HASH`] is identical either way.
fn ensure_signing_key() {
    #[cfg(feature = "law-signed")]
    {
        use std::sync::Once;
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            std::env::set_var(
                "PRAXIS_SIGNING_KEY",
                "8bb5514c228cf4275a64aba09f3da77ef7de8b74a4424d670e71c26b0557e293",
            );
        });
    }
}

/// The stable chain hash of the Day-2 mission receipt under [`TS_NS`]. If this
/// changes, either the pipe's bound payload changed (proposal hash, goal,
/// admitted plan, objective identity) or determinism regressed — both are
/// things this test exists to catch.
const EXPECTED_CHAIN_HASH: &str =
    "229a4fe9c0ede59fbc4d20640ee5a7a48746f5a91aebf1504c175724ea1863f8";

#[test]
fn full_pipe_runs_green_and_binds_proposal_hash() {
    ensure_signing_key();
    let t = revenue::run_demo(TS_NS).expect("revenue pipe must run green end to end");

    // Step 1: ranked proposals with rationale + 64-hex proposal_hash.
    let proposals = t["step_1_proposals"]["proposals"].as_array().expect("proposals array");
    assert!(proposals.len() >= 3, "expected several ranked proposals, got {}", proposals.len());
    let top = &proposals[0];
    assert_eq!(top["proposal_hash"].as_str().expect("hash").len(), 64);
    assert!(top["rationale"].as_array().expect("rationale").len() >= 3);

    // Step 2/3: the top goal drives a multi-action, evidence-gated plan.
    let plan = t["step_3_plan"]["plan"].as_array().expect("plan array");
    assert!(plan.len() >= 2, "top proposal should need a multi-step gated plan, got {plan:?}");

    // Step 4: every plan action was judged validated and admitted.
    for adm in t["step_4_admissions"].as_array().expect("admissions array") {
        assert_eq!(adm["judge_verdict"], json!("validated"), "action not validated: {adm}");
        assert_eq!(adm["admit_status"], json!("admitted"), "action not admitted: {adm}");
    }

    // Step 5: the receipt binds the top proposal_hash and yields a chain hash.
    let bound = t["step_5_receipt"]["binds_proposal_hash"].as_str().expect("bound hash");
    assert_eq!(bound, top["proposal_hash"].as_str().expect("top hash"), "receipt must bind top proposal");
    assert_eq!(t["chain_hash"].as_str().expect("chain hash").len(), 64);
}

#[test]
fn chain_hash_is_deterministic_and_pinned() {
    ensure_signing_key();
    let a = revenue::run_demo(TS_NS).expect("run a");
    let b = revenue::run_demo(TS_NS).expect("run b");
    assert_eq!(a["chain_hash"], b["chain_hash"], "same ts_ns must give the same chain hash");

    let got = a["chain_hash"].as_str().expect("chain hash");
    assert_eq!(
        got, EXPECTED_CHAIN_HASH,
        "chain hash drifted from the pinned value (update the pin only if the bound mission payload legitimately changed)"
    );
}

/// The seam the whole task turns on: the proposal lawfulness pre-filter and
/// the admission gate must agree. An account missing `legal_approved` must be
/// BOTH never proposed past `proposal` AND refused by `admit` if forced.
#[test]
fn missing_legal_approved_is_never_proposed_past_proposal_and_refused_if_forced() {
    ensure_signing_key();
    let t = revenue::run_demo(TS_NS).expect("run");
    let proposals = t["step_1_proposals"]["proposals"].as_array().expect("proposals");

    // (a) pre-filter: acct-legal-gap never appears with a target beyond proposal.
    let past_proposal = ["procurement", "closed-won"];
    let leaked: Vec<_> = proposals
        .iter()
        .filter(|p| p["target_account"] == json!("acct-legal-gap"))
        .filter(|p| {
            past_proposal.contains(&p["target_stage"].as_str().unwrap_or_default())
        })
        .collect();
    assert!(
        leaked.is_empty(),
        "acct-legal-gap (missing legal_approved) was proposed past proposal: {leaked:?}"
    );
    // Sanity: it *is* proposed up to proposal (the pre-filter isn't just empty).
    assert!(
        proposals.iter().any(|p| p["target_account"] == json!("acct-legal-gap")),
        "acct-legal-gap should still have a lawful proposal up to proposal"
    );

    // (b) admission gate: forcing the same account into procurement is refused,
    //     for the same missing-evidence reason the pre-filter used.
    let forced = revenue::forced_admit_by_id("acct-legal-gap", "procurement")
        .expect("forced admit should not hard-error");
    assert_eq!(forced["status"], json!("denied"), "forced move must be refused: {forced}");
    let unmet = forced["unmet"].as_array().expect("unmet array");
    let cites_legal = unmet.iter().any(|o| {
        o["evidence_type"] == json!("legal_approved")
            || o.to_string().contains("legal_approved")
    });
    assert!(cites_legal, "refusal should cite the missing legal_approved evidence: {unmet:?}");

    // A fully-evidenced account, by contrast, IS admitted into procurement.
    let ok = revenue::forced_admit_by_id("acct-apex", "procurement").expect("admit apex");
    assert_eq!(ok["status"], json!("admitted"), "apex has full evidence and must admit: {ok}");
}

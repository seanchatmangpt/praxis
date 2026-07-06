//! End-to-end tests for the `plan run` vertical slice (feature `ggen`).
//!
//! Covers: workflow dry-run (solve + POWL compile, no side effects), the
//! full loop over the `examples/v26_7_6_after_neon` fixture (artifact +
//! ledger receipt), and two-run determinism (identical `powl_chain_hash`).
#![cfg(feature = "ggen")]

use my_conforming_project::{mfg, ops, plan_run};
use serde_json::json;

const GOAL_TTL_PATH: &str = "examples/v26_7_6_after_neon/goal.ttl";
const GOAL_TTL: &str = include_str!("../examples/v26_7_6_after_neon/goal.ttl");

const EXPECTED_PLAN: &[&str] = &[
    "grant-standing",
    "ground-blueprint",
    "manufacture-artifact",
    "fold-receipt",
];

/// Under `--features law-signed`, the ledger receipt fails closed without a
/// signing key. Set the fixed house test key once (same key and pattern as
/// `tests/revenue_pipe.rs`) so the full loop stays green under
/// `--all-features`. Signing signs *over* the chain hash and does not feed
/// into it, so the determinism assertions are unaffected.
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

/// Workflow dry-run: manufacture + solve + compile to POWL without touching
/// the filesystem or the receipt ledger.
#[test]
fn dry_run_solve_and_powl_compile() {
    let manufactured = mfg::manufacture(GOAL_TTL, GOAL_TTL_PATH).expect("fixture manufactures");
    let payload = json!({
        "domain": manufactured.domain_text,
        "problem": manufactured.problem_text,
        "mode": "classical",
    })
    .to_string();
    let solved = ops::plan_solve_payload(&payload).expect("solve does not hard-error");
    assert_eq!(solved["admitted"], json!(true), "fixture goal is reachable");

    let steps: Vec<String> = solved["plan"]["ops"]
        .as_array()
        .expect("plan.ops array")
        .iter()
        .map(|s| {
            s["action"]["schema_name"]
                .as_str()
                .expect("schema name")
                .to_string()
        })
        .collect();
    assert_eq!(steps, EXPECTED_PLAN, "golden after-neon plan");

    let tape = plan_run::compile_plan_to_powl(&steps).expect("plan compiles to POWL");
    assert_eq!(tape.len as usize, EXPECTED_PLAN.len());
}

/// Full loop: graph -> plan -> POWL -> receipted execution -> artifact ->
/// ledger receipt, in one call, into a temp sandbox.
#[test]
fn full_loop_after_neon_fixture() {
    ensure_signing_key();
    let tmp = tempfile::tempdir().expect("tempdir");
    let out_dir = tmp.path().join("artifact");
    let receipts_dir = tmp.path().join("receipts");

    let result = plan_run::plan_run_payload(
        GOAL_TTL_PATH,
        out_dir.to_str().expect("utf-8 out dir"),
        receipts_dir.to_str().expect("utf-8 receipts dir"),
    )
    .expect("full loop does not hard-error");

    assert_eq!(result["admitted"], json!(true), "result: {result:#}");
    let plan: Vec<&str> = result["solve"]["plan"]
        .as_array()
        .expect("plan")
        .iter()
        .map(|v| v.as_str().expect("name"))
        .collect();
    assert_eq!(plan, EXPECTED_PLAN);
    assert_eq!(result["execution"]["fired"], result["solve"]["plan"]);

    let chain_hash = result["execution"]["powl_chain_hash"]
        .as_str()
        .expect("chain hash");
    assert!(chain_hash.starts_with("blake3:"), "got {chain_hash}");

    for f in ["domain.pddl", "problem.pddl", "plan.json"] {
        assert!(out_dir.join(f).is_file(), "artifact file {f} missing");
    }
    // The artifact records the same chain hash the run reported.
    let plan_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out_dir.join("plan.json")).expect("read plan.json"),
    )
    .expect("plan.json parses");
    assert_eq!(plan_json["powl_chain_hash"], json!(chain_hash));

    // The ledger receipt was appended: exactly one record in the fresh ledger.
    let shown = ops::receipt_show_payload(receipts_dir.to_str().expect("utf-8"), 0)
        .expect("ledger readable");
    assert_eq!(shown["total"], json!(1), "ledger: {shown:#}");
}

/// Two full runs over the same goal produce identical POWL chain hashes:
/// nothing time- or environment-dependent enters the frame hash path.
#[test]
fn two_runs_identical_chain_hashes() {
    ensure_signing_key();
    let hash_of_run = || {
        let tmp = tempfile::tempdir().expect("tempdir");
        let result = plan_run::plan_run_payload(
            GOAL_TTL_PATH,
            tmp.path().join("artifact").to_str().expect("utf-8"),
            tmp.path().join("receipts").to_str().expect("utf-8"),
        )
        .expect("run does not hard-error");
        assert_eq!(result["admitted"], serde_json::json!(true));
        result["execution"]["powl_chain_hash"]
            .as_str()
            .expect("chain hash")
            .to_string()
    };
    let a = hash_of_run();
    let b = hash_of_run();
    assert_eq!(a, b, "two-run determinism: identical chain hashes");
}

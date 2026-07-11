#![cfg(test)]

//! Multi-engine serve/resume tests (PROJ-722/723/724), in-process only:
//! contract execution from a sorted inbox scan, quiescence-file loop end,
//! deterministic EngineIdentity, resume-from-ledger with chain-prefix
//! verification, and the torn-ledger-tail refusal. The multi-PROCESS
//! harness (CARGO_BIN_EXE spawn of real second binaries) is PROJ-728 and
//! deliberately not here.

use std::fs;
use std::path::PathBuf;

use chicago_tdd_tools::prelude::*;
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use super::{engine_resume, engine_serve, EngineBundle, EngineIdentity, ENGINE_VERSION};
use crate::bench::dispatch::{disp_object, workday_contract, ExecutionClass, DISP_PREFIX};
use crate::bench::fill_template;
use crate::powl::CngRefusal;

/// Scratch root for this test file. O(1).
fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/engine")
        .join(test_name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// Crate-root path helper. O(1).
fn crate_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Renders one shape-valid dispatch contract into the engine's inbox and
/// returns its dispatch id. O(|template|).
fn seed_inbox_contract(bundle: &EngineBundle, set_id: &str) -> String {
    let template = fs::read_to_string(crate_path("templates/dispatch-contract.template.ttl"))
        .expect("contract template reads");
    let mut contract = workday_contract(
        set_id,
        "software-delivery",
        1,
        ExecutionClass::ExternalMachineDispatch,
    );
    contract.recursive_depth = 0;
    contract.closure_law = None;
    let rendered = contract.render(&template).expect("contract renders");
    let path = bundle
        .inbox_dir()
        .join(format!("{}.ttl", contract.dispatch_id));
    fs::write(&path, &rendered).expect("inbox contract writes");
    contract.dispatch_id
}

test!(engine_identity_is_deterministic_and_engine_distinct, {
    // Arrange + Act: same (id, seed) twice; a different id once.
    let a1 = EngineIdentity::new("H", 42);
    let a2 = EngineIdentity::new("H", 42);
    let b = EngineIdentity::new("M", 42);

    // Assert: byte-stable identity, no PID/wall-clock input; distinct
    // engines get distinct nonces from the same seed.
    assert_eq!(a1.instance_nonce, a2.instance_nonce);
    assert_eq!(a1.engine_version, ENGINE_VERSION);
    assert_ne!(a1.instance_nonce, b.instance_nonce);
});

test!(serve_executes_inbox_contract_and_writes_consequence, {
    // Arrange: one engine bundle with one shape-valid contract in inbox.
    let root = scratch_dir("serve_executes");
    let bundle = EngineBundle::new(&root, "H").expect("bundle constructs");
    let dispatch_id = seed_inbox_contract(&bundle, "srv0");

    // Act: bounded serve (no quiesce file — the poll budget ends it).
    let report = engine_serve(&root, "H", 42, 3, None).expect("serve completes");

    // Assert: the contract was admitted + executed through the real chain
    // and its consequence landed in the outbox, correlation intact.
    assert_eq!(report.contracts_executed, 1);
    assert!(!report.quiesced);
    assert!(!report.resumed);
    assert_eq!(report.polls, 3);
    let consequence_path = bundle.outbox_dir().join(format!("{dispatch_id}.ttl"));
    let consequence_ttl = fs::read_to_string(&consequence_path).expect("consequence exists");
    let store = Store::new().expect("store");
    store
        .load_from_slice(
            RdfParser::from_format(RdfFormat::Turtle),
            consequence_ttl.as_bytes(),
        )
        .expect("consequence parses");
    let correlation = disp_object(&store, "consequenceOf", DISP_PREFIX)
        .expect("scan runs")
        .expect("consequenceOf present");
    assert!(correlation.starts_with("corr-"), "got {correlation}");
    // Ledger: three remote-side transitions were appended durably.
    let entries = crate::bench::dispatch::read_ledger_entries(
        &bundle.ledger_dir().join(format!("{dispatch_id}.ttl")),
    )
    .expect("ledger reads");
    let to_states: Vec<&str> = entries.iter().map(|e| e.to_state.as_str()).collect();
    assert_eq!(
        to_states,
        vec!["REMOTE_STARTED", "REMOTE_IN_PROGRESS", "RESULT_AVAILABLE"]
    );
});

test!(shacl_validated_quiescence_file_ends_the_loop, {
    // Arrange: a bundle whose control dir already holds a valid quiesce.ttl.
    let root = scratch_dir("quiesce_ends");
    let bundle = EngineBundle::new(&root, "H").expect("bundle constructs");
    let template = fs::read_to_string(crate_path("templates/dispatch-quiesce.template.ttl"))
        .expect("quiesce template reads");
    let body = fill_template(
        &template,
        &[
            ("SUBJECT", "quiesce-h"),
            ("ENGINE_ID", "H"),
            ("REASON", "test-shutdown"),
        ],
    );
    fs::write(bundle.quiesce_path(), &body).expect("quiesce writes");

    // Act.
    let report = engine_serve(&root, "H", 42, 8, None).expect("serve completes");

    // Assert: the loop ended on the validated control file at poll 1.
    assert!(report.quiesced);
    assert_eq!(report.polls, 1);
    assert_eq!(report.contracts_executed, 0);
});

test!(
    resume_verifies_ledger_prefix_and_skips_processed_contracts,
    {
        // Arrange: one served pass (contract executed, ledger + processed set
        // durable), simulating work done before a kill.
        let root = scratch_dir("resume_continues");
        let bundle = EngineBundle::new(&root, "H").expect("bundle constructs");
        let dispatch_id = seed_inbox_contract(&bundle, "res0");
        let first = engine_serve(&root, "H", 42, 2, None).expect("first serve completes");
        assert_eq!(first.contracts_executed, 1);

        // Act: resume over the same bundle — the inbox contract is still on
        // disk, but its idempotency key is in the durable processed set.
        let resumed = engine_resume(&root, "H", 42, 2, None).expect("resume completes");

        // Assert: chain-prefix verified (3 ledgered transitions), the
        // already-processed contract was SKIPPED (no double execution), and
        // the receipt-chain digest reflects zero new executions.
        assert!(resumed.resumed);
        assert_eq!(resumed.ledger_entries_verified, 3);
        assert_eq!(resumed.contracts_executed, 0);
        // The prior consequence is still exactly where the first pass put it.
        assert!(bundle
            .outbox_dir()
            .join(format!("{dispatch_id}.ttl"))
            .is_file());
    }
);

test!(torn_ledger_tail_refuses_cng_r11_on_resume, {
    // Arrange: a served bundle, then a truncated (torn) last ledger entry —
    // the crash-mid-write case atomic rename normally prevents.
    let root = scratch_dir("torn_tail");
    let bundle = EngineBundle::new(&root, "H").expect("bundle constructs");
    let dispatch_id = seed_inbox_contract(&bundle, "torn0");
    engine_serve(&root, "H", 42, 2, None).expect("serve completes");
    let ledger_path = bundle.ledger_dir().join(format!("{dispatch_id}.ttl"));
    let full = fs::read_to_string(&ledger_path).expect("ledger reads");
    let torn = &full[..full.len() - 40];
    fs::write(&ledger_path, torn).expect("torn tail writes");

    // Act.
    let result = engine_resume(&root, "H", 42, 2, None);

    // Assert: typed CNG_R11 AuditMismatch naming the torn ledger — a torn
    // tail is refused lawfully, never repaired silently.
    match result {
        Err(CngRefusal::AuditMismatch(msg)) => {
            assert!(msg.contains("ledger"), "got {msg}");
            assert_eq!(CngRefusal::AuditMismatch(msg).code(), "CNG_R11");
        }
        other => panic!("expected AuditMismatch, got {other:?}"),
    }
});

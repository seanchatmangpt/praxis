//! Closure work (v26.7.10-revised): the continuous admit-to-decompose-to-
//! dispatch narrative — Track P (`crates/cng/src/bench/decomp/`, no-LLM goal
//! decomposition, PROJ-702..710) feeding Track E (`crates/cng/src/bench/
//! {dispatch,engine}.rs`, multi-engine execution, PROJ-720..724) — driven
//! end to end through the new bridge
//! (`crates/cng/src/bench/decomp/dispatch_bridge.rs`).
//!
//! Fixture: the canonical potato example (`examples/pddl-strips-potato.ttl`,
//! same bridge pattern as `tests/cng_decomp.rs`) currently SELECTS THE
//! SINGLE-ACTOR PLAN under the real `decompose()` law — verified directly
//! against its emitted `decomposition-result.ttl`
//! (`decomp:outcome "NoAdmissibleDecomposition"`, `decomp:subworkflowCount
//! "1"`) before this file was written. A single-actor result has nothing to
//! dispatch across two engines, so the MULTI-subworkflow case this test
//! exists to prove is exercised with the same typed-struct "kitchen
//! two-chain" fixture `crates/cng/src/bench/decomp/decomp_test.rs` already
//! uses in-crate (`kitchen_two_chain_selects_a_two_actor_decomposition`) —
//! two independent goal chains over disjoint objects, guaranteed by
//! construction (not cherry-picked) to split into `helper` ∥ `main`. The
//! fixture cannot be imported from an external integration test (it is
//! `#[cfg(test)]`-private to the crate), so it is reconstructed here from
//! the same public `bcinr_pddl` struct types, field-for-field identical.
//!
//! What this test proves and what it does NOT: see the assertions at the
//! bottom and the module doc on `dispatch_bridge` for the stated boundary
//! (the remote engine executes its OWN dispatch-id-seeded synthetic
//! manufacture, not the dispatched subworkflow's PDDL plan — no payload-
//! carrying contract exists yet, PROJ-710 → PROJ-723 is still open).

#![cfg(feature = "bench")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use chicago_tdd_tools::prelude::*;

use bcinr_pddl::{Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8Problem};

use cng::bench::decomp::dispatch_bridge::{
    collect_subworkflow_consequence, dispatch_subworkflow_to_engine,
};
use cng::bench::decomp::{decompose, DecompositionOutcome};

fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/decompose-to-dispatch-it")
        .join(test_name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn atom(pred: &str, args: &[&str]) -> Pddl8Atom {
    Pddl8Atom {
        pred: pred.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
    }
}

fn schema(
    name: &str,
    pre: Vec<Pddl8Atom>,
    add: Vec<Pddl8Atom>,
    del: Vec<Pddl8Atom>,
) -> Pddl8ActionSchema {
    Pddl8ActionSchema {
        name: name.to_string(),
        params: vec!["?x".to_string()],
        preconditions: pre,
        add_effects: add,
        del_effects: del,
        typed_params: Vec::new(),
        condition: None,
        effects: Vec::new(),
        numeric_effects: Vec::new(),
    }
}

/// Two independent goal chains over disjoint objects (fetch+cook the
/// potato, fetch+place the fork) — field-for-field the same fixture as
/// `decomp_test.rs`'s `kitchen_domain`/`kitchen_problem`
/// (`kitchen_two_chain_selects_a_two_actor_decomposition`), reconstructed
/// here because that test module is crate-private. `held` is the admitted
/// resource predicate (`rules/decomp-resources.dl`), released by both
/// `cook` and `place`.
fn kitchen_domain() -> Pddl8Domain {
    Pddl8Domain {
        name: "kitchen-two-chain".to_string(),
        predicates: vec![
            ("in-pantry".to_string(), 1),
            ("in-drawer".to_string(), 1),
            ("held".to_string(), 1),
            ("cooked".to_string(), 1),
            ("placed".to_string(), 1),
        ],
        actions: vec![
            schema(
                "fetch-pantry",
                vec![atom("in-pantry", &["?x"])],
                vec![atom("held", &["?x"])],
                vec![atom("in-pantry", &["?x"])],
            ),
            schema(
                "fetch-drawer",
                vec![atom("in-drawer", &["?x"])],
                vec![atom("held", &["?x"])],
                vec![atom("in-drawer", &["?x"])],
            ),
            schema(
                "cook",
                vec![atom("held", &["?x"])],
                vec![atom("cooked", &["?x"])],
                vec![atom("held", &["?x"])],
            ),
            schema(
                "place",
                vec![atom("held", &["?x"])],
                vec![atom("placed", &["?x"])],
                vec![atom("held", &["?x"])],
            ),
        ],
        types: Vec::new(),
        functions: Vec::new(),
        durative_actions: Vec::new(),
        derived: Vec::new(),
        constraints: Vec::new(),
        processes: Vec::new(),
        events: Vec::new(),
    }
}

fn kitchen_problem() -> Pddl8Problem {
    Pddl8Problem {
        name: "kitchen-two-chain-1".to_string(),
        domain: "kitchen-two-chain".to_string(),
        objects: vec!["potato".to_string(), "fork".to_string()],
        init: vec![atom("in-pantry", &["potato"]), atom("in-drawer", &["fork"])],
        goal: vec![atom("cooked", &["potato"]), atom("placed", &["fork"])],
        object_types: Vec::new(),
        fn_values: Vec::new(),
        timed_inits: Vec::new(),
        preferences: Vec::new(),
        metric: None,
    }
}

/// Runs the compiled `cng` binary to completion. O(child runtime).
fn run_cng(args: &[&str]) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_cng"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn cng binary");
    (
        String::from_utf8(output.stdout).expect("utf-8 stdout"),
        String::from_utf8(output.stderr).expect("utf-8 stderr"),
        output.status.success(),
    )
}

/// Runs one serialized `cng engine serve` pass to completion (mirrors
/// `tests/cng_multi_engine.rs`'s `serve_to_budget`): NoWait (no
/// `--poll-wait-ms`), small bounded `--max-polls` so the single contract
/// already sitting in the inbox is picked up on poll 0 and the process
/// exits promptly once the budget is exhausted.
fn serve_to_budget(root: &Path, engine_id: &str, max_polls: &str) -> String {
    let (stdout, stderr, ok) = run_cng(&[
        "engine",
        "serve",
        "--root",
        root.to_str().expect("utf-8 root"),
        "--engine-id",
        engine_id,
        "--seed",
        "616",
        "--max-polls",
        max_polls,
    ]);
    assert!(ok, "engine serve {engine_id} failed: {stderr}");
    stdout
}

test!(kitchen_decomposition_splits_into_helper_and_main, {
    // Arrange + Act: same fixture the dispatch stage below reuses — proves
    // the split independently of the dispatch machinery.
    let domain = kitchen_domain();
    let problem = kitchen_problem();
    let out = scratch_dir("kitchen-split-only");

    let result = decompose(
        &domain,
        &problem,
        &out,
        "urn:cng:test:decompose-to-dispatch",
    )?;

    // Assert: derived (not hardcoded) two-actor decomposition.
    assert!(matches!(
        result.outcome,
        DecompositionOutcome::Selected {
            subworkflows: 2,
            ..
        }
    ));
    assert_eq!(result.subworkflows.len(), 2);
    assert_eq!(result.subworkflows[0].role, "helper");
    assert_eq!(result.subworkflows[1].role, "main");
});

test!(
    decomposed_subworkflows_dispatch_to_real_engines_and_are_admitted,
    {
        // Arrange: decompose the guaranteed-split fixture into helper + main.
        let domain = kitchen_domain();
        let problem = kitchen_problem();
        let decomp_out = scratch_dir("kitchen-decompose");
        let result = decompose(
            &domain,
            &problem,
            &decomp_out,
            "urn:cng:test:decompose-to-dispatch-live",
        )?;
        let DecompositionOutcome::Selected {
            subworkflows: 2, ..
        } = result.outcome
        else {
            panic!(
                "fixture must select a 2-subworkflow split, got {:?}",
                result.outcome
            );
        };
        assert_eq!(result.subworkflows.len(), 2);
        let helper = &result.subworkflows[0];
        let main = &result.subworkflows[1];
        assert_eq!(helper.role, "helper");
        assert_eq!(main.role, "main");
        // Each subworkflow carries its own manufactured PDDL problem + digest —
        // the content the bridge below identifies (not yet the content the
        // engine executes; see the module doc on `dispatch_bridge`).
        assert!(!helper.problem_pddl.is_empty());
        assert!(!main.problem_pddl.is_empty());
        assert_ne!(helper.problem_digest, main.problem_digest);

        // Act 1 (dispatch): bridge each subworkflow into a real dispatch
        // contract, addressed to a DISTINCT target engine, written directly
        // into that engine's real filesystem inbox.
        let root = scratch_dir("kitchen-dispatch-root");
        let handle_h = dispatch_subworkflow_to_engine(&root, helper, "H")?;
        let handle_m = dispatch_subworkflow_to_engine(&root, main, "M")?;
        assert_eq!(handle_h.role, "helper");
        assert_eq!(handle_h.target_engine, "H");
        assert_eq!(handle_m.role, "main");
        assert_eq!(handle_m.target_engine, "M");
        assert!(root
            .join("engines/H/inbox")
            .join(format!("{}.ttl", handle_h.dispatch_id))
            .is_file());
        assert!(root
            .join("engines/M/inbox")
            .join(format!("{}.ttl", handle_m.dispatch_id))
            .is_file());

        // Act 2 (execute): spawn REAL H and M engine OS processes
        // (CARGO_BIN_EXE_cng engine serve, mirroring tests/cng_multi_engine.rs's
        // spawn_engine/serve_to_budget helpers), each running its bounded serve
        // loop to completion. The contract is already in the inbox before the
        // process starts, so it is admitted on poll 0.
        serve_to_budget(&root, "H", "4");
        serve_to_budget(&root, "M", "4");

        // Act 3 (collect): bounded poll for each engine's real outbox
        // consequence, then the same lawful re-entry pipeline the multi-engine
        // coordinator uses.
        let outcome_h = collect_subworkflow_consequence(&root, &handle_h, 4, None)?;
        let outcome_m = collect_subworkflow_consequence(&root, &handle_m, 4, None)?;

        // Assert: both dispatches round-tripped through a REAL second process
        // and were lawfully admitted; both consequences are receipted
        // (non-empty content digest) and durable on each engine's outbox.
        assert!(
            outcome_h.admitted,
            "helper consequence must be admitted: {outcome_h:?}"
        );
        assert!(
            outcome_m.admitted,
            "main consequence must be admitted: {outcome_m:?}"
        );
        assert!(outcome_h.consequence_digest.is_some());
        assert!(outcome_m.consequence_digest.is_some());
        assert!(root
            .join("engines/H/outbox")
            .join(format!("{}.ttl", handle_h.dispatch_id))
            .is_file());
        assert!(root
            .join("engines/M/outbox")
            .join(format!("{}.ttl", handle_m.dispatch_id))
            .is_file());
        // Each engine's own serve report is durable (a real process ran).
        assert!(root.join("engines/H/receipts/serve-report.json").is_file());
        assert!(root.join("engines/M/receipts/serve-report.json").is_file());

        // --- What this test does NOT prove (stated honestly, not asserted
        // past): the H/M engine processes above each derive their OWN
        // deterministic PDDL artifact set from `blake3(dispatch_id)`
        // (`engine.rs::run_serve_loop`, `write_set` seeded, category hardcoded
        // to "email-routing") — dispatch contracts do not yet carry the
        // subworkflow's `problem_pddl`/tape as an executable payload. So this
        // test proves: (1) a real `decompose()` run derives a genuine 2-actor
        // split (not hardcoded), (2) each subworkflow's identity converts
        // deterministically into a valid, shape-conformant dispatch contract
        // via the new bridge, (3) that contract round-trips through a REAL,
        // independently-spawned second OS process's admission + lawful
        // re-entry pipeline (provenance/correlation/authority/structural/
        // semantic, all five stages), and (4) the receipts of that round trip
        // are durable on disk. It does NOT prove that engine H executed
        // `helper`'s specific plan/PDDL, nor that combining the two engines'
        // outputs reconstructs or closes the ORIGINAL kitchen problem's goal —
        // no machinery in this crate today makes that claim checkable, because
        // no payload-carrying contract exists yet (PROJ-710 → PROJ-723).
    }
);

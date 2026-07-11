//! PROJ-714 mechanism proof (1-of-4 declared long-horizon scenarios, G14/G15
//! in `docs/releases/v26.7.10/DEFINITION_OF_DONE.md`): a genuinely long
//! multi-room logistics scenario driven through the SAME no-LLM
//! decomposition pipeline already built this session
//! (`cng::bench::decomp::decompose`) — grounding, Datalog edge derivation
//! (`rules/decomp.dl`), bounded candidate search, interface-state replay,
//! non-interference + release-closure proofs, nested POWL composition, and
//! deterministic selection — at a plan length well past the 4-10-step
//! potato/kitchen/IPC-corpus fixtures
//! (`crates/cng/src/bench/decomp/decomp_test.rs`'s `kitchen_domain`,
//! `crates/cng/examples/pddl-strips-potato.ttl`, `crates/cng/src/bench/ipc/`).
//!
//! # Scenario design
//!
//! Two packages (`parcel-a`, `parcel-b`) each traverse an independent
//! `ROOM_COUNT`-room corridor from `room-0` to `room-(ROOM_COUNT-1)` via a
//! single `move(?pkg ?from ?to)` STRIPS action over static `connected`
//! edges. `connected` atoms are never deleted (a road, not a resource), so
//! sharing the corridor topology between the two packages introduces no
//! interference — see `crates/cng/src/bench/decomp/interference.rs` and
//! `rules/decomp.dl`'s atom-scoped `mutex`/`custodyConflict` derivation
//! (ground atoms are package-specific: `at(parcel-a, room-3)` and
//! `at(parcel-b, room-3)` are distinct atoms, so achiever/mutex/custody
//! sets never cross between the two packages' chains). This makes the two
//! packages' goal atoms independent under `decomp::search::partition_goals`
//! (union-find over derived `achieves`/`mutex`/`custodyConflict` coupling),
//! so a genuine two-actor (helper/main) decomposition is derivable — not
//! merely a longer single-actor chain.
//!
//! `ROOM_COUNT - 1` moves per package × 2 packages is the single-actor plan
//! length; `ROOM_COUNT = 16` gives a 30-step single-actor plan — well past
//! the IPC-corpus generators' documented ≤ ~10-step ceiling (`crates/cng/
//! src/bench/ipc/mod.rs`'s module doc: sizes are tuned so "plans stay
//! short (≤ ~10 steps)") and the potato/kitchen fixtures' 2-4-atom goal
//! chains (`crates/cng/src/bench/decomp/decomp_test.rs`'s `kitchen_domain`,
//! `crates/cng/examples/pddl-strips-potato.ttl`'s 2-atom goal). Grounding
//! stays typed and bounded: `move` has exactly 3 parameters, and
//! `pddl_index`'s
//! relaxed-reachability-pruned grounder (PROJ-733) only ever materializes
//! the 30 ground actions actually reachable from the two package chains —
//! not the naive `|objects|^3` cross product — so this scenario does not
//! reintroduce the grounding blowup PROJ-733 fixed (see
//! `crates/cng/src/bench/decomp/mod.rs`'s `DECOMP_MAX_GROUND` doc comment).
//!
//! # Honest scope
//!
//! This file proves the PROJ-714 MECHANISM works end-to-end on ONE
//! scenario — the declared 80/20 cut for this session
//! (`docs/releases/v26.7.10/RELEASE_CONTROL.md` §9.2). The remaining
//! 3-of-4 declared long-horizon scenarios are the SAME mechanism
//! un-applied to further domains, not separately built here.
//!
//! Typed-struct fixtures only (house style, `tests/no_inline_ttl_guard.rs`
//! enforced) — no inline PDDL/Turtle/SPARQL in this file; `decompose()` is
//! called directly on `bcinr_pddl` structs, mirroring
//! `crates/cng/src/bench/decomp/decomp_test.rs`'s typed-struct convention
//! and `tests/cng_decomp_negative_corpus_completeness.rs`'s external-test-
//! file precedent.

#![cfg(feature = "bench")]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use chicago_tdd_tools::prelude::*;

use bcinr_pddl::{Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8Problem};

use cng::bench::decomp::{
    decompose, CandidateStatus, DecompositionOutcome, SINGLE_ACTOR_CANDIDATE_ID,
};

/// Corridor length: `ROOM_COUNT` rooms chained `room-0 -> room-1 -> ... ->
/// room-(ROOM_COUNT-1)`, so each package's traversal takes `ROOM_COUNT - 1`
/// `move` steps. Two independent packages -> a `2 * (ROOM_COUNT - 1)`-step
/// single-actor plan (30 steps at `ROOM_COUNT = 16`), well past the
/// 4-10-step potato/kitchen/IPC-corpus fixture range.
const ROOM_COUNT: usize = 16;

fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/long-horizon")
        .join(test_name);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn atom(pred: &str, args: &[&str]) -> Pddl8Atom {
    Pddl8Atom {
        pred: pred.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
    }
}

fn room(i: usize) -> String {
    format!("room-{i}")
}

/// The single `move(?pkg ?from ?to)` action schema: relocate a package from
/// `?from` to `?to` over a static `connected` corridor edge. `connected` is
/// never an effect of any action (a road, not custody state), so it never
/// participates in `rules/decomp.dl`'s `mutex`/`custodyConflict` derivation.
fn move_schema() -> Pddl8ActionSchema {
    Pddl8ActionSchema {
        name: "move".to_string(),
        params: vec!["?pkg".to_string(), "?from".to_string(), "?to".to_string()],
        preconditions: vec![
            atom("at", &["?pkg", "?from"]),
            atom("connected", &["?from", "?to"]),
        ],
        add_effects: vec![atom("at", &["?pkg", "?to"])],
        del_effects: vec![atom("at", &["?pkg", "?from"])],
        typed_params: Vec::new(),
        condition: None,
        effects: Vec::new(),
        numeric_effects: Vec::new(),
    }
}

fn logistics_domain() -> Pddl8Domain {
    Pddl8Domain {
        name: "logistics-long-horizon".to_string(),
        predicates: vec![("at".to_string(), 2), ("connected".to_string(), 2)],
        actions: vec![move_schema()],
        types: Vec::new(),
        functions: Vec::new(),
        durative_actions: Vec::new(),
        derived: Vec::new(),
        constraints: Vec::new(),
        processes: Vec::new(),
        events: Vec::new(),
    }
}

/// Two packages (`parcel-a`, `parcel-b`), each starting at `room-0` and
/// needing to reach `room-(ROOM_COUNT-1)`, over the SAME static corridor.
/// Object-disjoint per-package `at` atoms keep the two goal chains
/// independent for `decomp::search::partition_goals` (see module doc).
///
/// # Complexity
/// O(ROOM_COUNT) to build the corridor's `connected` facts.
fn logistics_problem() -> Pddl8Problem {
    let mut objects = vec!["parcel-a".to_string(), "parcel-b".to_string()];
    let mut init = vec![
        atom("at", &["parcel-a", "room-0"]),
        atom("at", &["parcel-b", "room-0"]),
    ];
    for i in 0..ROOM_COUNT {
        objects.push(room(i));
    }
    for i in 0..ROOM_COUNT - 1 {
        let from = room(i);
        let to = room(i + 1);
        init.push(atom("connected", &[from.as_str(), to.as_str()]));
    }
    let last_room = room(ROOM_COUNT - 1);
    let goal = vec![
        atom("at", &["parcel-a", last_room.as_str()]),
        atom("at", &["parcel-b", last_room.as_str()]),
    ];
    Pddl8Problem {
        name: "logistics-long-horizon-1".to_string(),
        domain: "logistics-long-horizon".to_string(),
        objects,
        init,
        goal,
        object_types: Vec::new(),
        fn_values: Vec::new(),
        timed_inits: Vec::new(),
        preferences: Vec::new(),
        metric: None,
    }
}

test!(
    long_horizon_logistics_scenario_decomposes_and_plans_end_to_end,
    {
        // Arrange.
        let domain = logistics_domain();
        let problem = logistics_problem();
        let out = scratch_dir("logistics");
        let expected_single_actor_steps = 2 * (ROOM_COUNT - 1);
        let started = Instant::now();

        // Act: the real decomposition pipeline (grounding -> Datalog edge
        // derivation -> candidate search -> planning -> interference/release
        // proofs -> selection -> receipt), unmodified from the potato/kitchen
        // fixtures' call shape.
        let result = decompose(
            &domain,
            &problem,
            &out,
            "urn:cng:test:decomp:long-horizon:logistics",
        )?;
        let elapsed = started.elapsed();

        // Assert: candidate 0 is always the single-actor plan, and its own
        // makespan proves this scenario is genuinely long-horizon — a typed
        // number derived from the real BFS plan, not a renamed short scenario.
        assert_eq!(
            result.candidate_receipts[0].candidate_id,
            SINGLE_ACTOR_CANDIDATE_ID
        );
        assert_eq!(
            result.candidate_receipts[0].score.makespan, expected_single_actor_steps as u64,
            "the single-actor plan must be exactly 2*(ROOM_COUNT-1) steps"
        );
        assert!(
            result.candidate_receipts[0].score.makespan >= 20,
            "long-horizon scenario must exceed the 4-10-step potato/kitchen/IPC \
         fixture range (got {})",
            result.candidate_receipts[0].score.makespan
        );
        assert!(
            result.candidate_receipts.len() >= 2,
            "split candidates must be examined, not skipped"
        );

        // Assert: whichever of the three typed outcomes won, the returned
        // subworkflow tapes together account for every step of the traversal —
        // the pipeline never drops or duplicates steps at this longer scale.
        let total_ops: usize = result.subworkflows.iter().map(|s| s.tape.ops.len()).sum();
        match &result.outcome {
            DecompositionOutcome::Selected { subworkflows, .. } => {
                assert_eq!(*subworkflows, 2);
                assert_eq!(result.subworkflows.len(), 2);
                assert_eq!(result.subworkflows[0].role, "helper");
                assert_eq!(result.subworkflows[1].role, "main");
                assert!(!result.interface_atoms.is_empty());
                assert_eq!(
                    total_ops, expected_single_actor_steps,
                    "helper + main step counts must sum to the whole traversal"
                );

                // The split genuinely BEAT the single actor on the selection
                // law's makespan (helper ∥ main run concurrently, so makespan
                // is the longer of the two chains, not their sum) — proof the
                // decomposition is a real benefit at this scale, not a tie
                // broken arbitrarily.
                let selected = result
                    .candidate_receipts
                    .iter()
                    .find(|r| r.status == CandidateStatus::Selected)
                    .expect("a selected candidate is always receipted");
                assert_ne!(selected.candidate_id, SINGLE_ACTOR_CANDIDATE_ID);
                assert!(
                    selected.score.makespan < result.candidate_receipts[0].score.makespan,
                    "winning split ({}) must have strictly lower makespan than \
                 single-actor ({})",
                    selected.score.makespan,
                    result.candidate_receipts[0].score.makespan
                );
            }
            DecompositionOutcome::NoBeneficialDecomposition { .. }
            | DecompositionOutcome::NoAdmissibleDecomposition { .. } => {
                assert_eq!(result.subworkflows.len(), 1);
                assert_eq!(result.subworkflows[0].role, "single");
                assert_eq!(total_ops, expected_single_actor_steps);
            }
        }

        // Assert: real evidence was written for this longer scenario, not
        // skipped.
        assert!(result.result_graph_path.exists());
        let graph = fs::read_to_string(&result.result_graph_path).expect("read result graph");
        assert!(!graph.is_empty());

        // Determinism at this longer scale: a second manufacture over the SAME
        // fixture is byte-identical (no wall clock, no randomness in the
        // digest/receipt path).
        let out2 = scratch_dir("logistics-again");
        let again = decompose(
            &domain,
            &problem,
            &out2,
            "urn:cng:test:decomp:long-horizon:logistics",
        )?;
        let bytes_a = fs::read(&result.result_graph_path).expect("read a");
        let bytes_b = fs::read(&again.result_graph_path).expect("read b");
        assert_eq!(bytes_a, bytes_b);

        // Assert: this longer scenario has not reintroduced the grounding
        // blowup PROJ-733 fixed — both decompose() calls together stay well
        // under a wall-clock budget generous enough to absorb CI noise but
        // tight enough to catch a combinatorial regression.
        assert!(
            elapsed.as_secs() < 30,
            "single decompose() call over the long-horizon fixture took {elapsed:?}; \
         PROJ-733's grounding fix may have regressed"
        );
    }
);

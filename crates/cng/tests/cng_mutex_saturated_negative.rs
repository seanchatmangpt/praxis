//! Closes doctrine §18 item 5 (`docs/releases/v26.7.10/DEFINITION_OF_DONE.md`):
//! "Mutex-saturated goals -> NO_ADMISSIBLE_DECOMPOSITION". `DOD_SIGNOFF.md`'s
//! §18 detail table marks this row "ALIVE (adjacent scenario)": the only
//! evidence cited is `decomp/decomp_test.rs::
//! single_atom_goal_yields_no_admissible_decomposition`, which proves the
//! `NoAdmissibleDecomposition { rejected: 0 }` TYPED-OUTCOME MECHANISM in
//! general, using a domain with exactly ONE goal atom — there is no
//! possible second partition component for a one-atom goal regardless of
//! mutex, so that fixture never exercises `rules/decomp.dl`'s `:mutex`
//! derivation or `search.rs::coupled` at all. This file closes the literal
//! gap: a fixture where TWO independently-achievable goal atoms exist (so a
//! split looks structurally possible at a glance) but genuine STRIPS
//! resource contention between their two DISTINCT, SOLE achievers —
//! `achieve-a` deletes the precondition atom `achieve-b` needs — is
//! Datalog-derived into a real `:mutex` edge, which `search.rs::coupled`
//! (achiever-set/mutex/custody coupling) turns into a partition-graph UNION
//! of the two goal atoms into ONE component, making a split structurally
//! inadmissible (never even candidate-enumerated) rather than a candidate
//! that was tried and rejected.
//!
//! This is deliberately a DIFFERENT mechanism from doctrine item 3
//! ("Interfering effect pair -> CNG_R22", already ALIVE via
//! `concurrent_clobber_refuses_cng_r22_interference`/
//! `interfering_parallel_actions_refuse_cng_r22`): item 3's `CNG_R22`
//! fires inside `interference::check_interference` against the CONCRETE
//! planned tapes of an ALREADY-ENUMERATED split candidate, and shows up as
//! an `Inadmissible` candidate receipt. Item 5's mutex-saturation instead
//! fires one stage earlier, inside `search::partition_goals` /
//! `search::coupled`, and — because the coupled goal atoms never even form
//! two separate components — produces ZERO split candidates and ZERO
//! rejected receipts: `result.candidate_receipts.len() == 1` (the
//! single-actor candidate alone). The tests below inspect exactly that
//! receipt ledger to confirm the empty-rejected-list shape is the true
//! signature of structural (partition-time) inadmissibility, and pair it
//! with direct inspection of the Datalog-derived `DerivedEdges` (real
//! `rules/decomp.dl` mutex rule, not a hand-fabricated `DerivedEdges`
//! struct) plus a structurally-identical CONTROL domain with the mutex
//! removed (same shape, disjoint resources) that DOES split — an A/B pair
//! isolating mutex-coupling, not domain shape, as the causal mechanism.
//!
//! Typed-struct fixtures only (house style, `tests/no_inline_ttl_guard.rs`
//! enforced) — no inline PDDL/Turtle/SPARQL; `rules/decomp.dl` and
//! `rules/decomp-resources.dl` are read from the real on-disk files (same
//! paths `decomp/mod.rs::load_rules_texts` reads internally), never
//! duplicated as string literals. Test-only file; no production source
//! under `crates/cng/src/` is touched by this file.

#![cfg(feature = "bench")]

use std::fs;
use std::path::{Path, PathBuf};

use chicago_tdd_tools::prelude::*;

use bcinr_pddl::{Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8Problem};
use pddl_index::ground::IndexedGroundProblem as GroundProblem;

use cng::bench::decomp::{
    decomp_queries_dir, decompose, derive_edges, lift_ground, partition_goals, CandidateStatus,
    DecompositionOutcome, DerivedEdges, DECOMP_MAX_GROUND, SINGLE_ACTOR_CANDIDATE_ID,
};
use cng::bench::QuerySet;

fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/mutex-saturated-negative")
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

/// One-parameter (`?x`) action schema — mirrors `decomp_test.rs`'s `schema`
/// helper and `cng_decomp_negative_corpus_completeness.rs`'s copy of it.
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

/// Reads the REAL `rules/decomp.dl` + `rules/decomp-resources.dl` from
/// disk — the same files `rules::derive_edges` is fed in production
/// (`decomp/mod.rs::load_rules_texts`, private to that module, so this test
/// reads the identical on-disk paths itself rather than hand-fabricating
/// rule text).
fn read_rules_texts() -> (String, String) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rules");
    let rules = fs::read_to_string(dir.join("decomp.dl")).expect("read rules/decomp.dl");
    let resources = fs::read_to_string(dir.join("decomp-resources.dl"))
        .expect("read rules/decomp-resources.dl");
    (rules, resources)
}

// ---------------------------------------------------------------------------
// Fixture A — mutex-saturated: two independently-achievable goal atoms whose
// SOLE achievers are genuinely STRIPS-mutex (one deletes the precondition
// the other's only achiever needs).
// ---------------------------------------------------------------------------

/// `shared(?x)` is a plain (non-resource-classified, per
/// `rules/decomp-resources.dl`) precondition atom both actions need.
/// `achieve-a` DELETES it (a real add/del STRIPS effect, not a coincidence);
/// `achieve-b` only reads it. `rules/decomp.dl`'s
/// `{?a :delEff ?p. ?b :pre ?p}=>{?a :mutex ?b}` rule therefore derives a
/// genuine `:mutex(achieve-a, achieve-b)` edge from the admitted facts of
/// THIS domain — nothing hand-inserted into a `DerivedEdges` struct.
fn mutex_saturated_domain() -> Pddl8Domain {
    Pddl8Domain {
        name: "goal-mutex-saturated".to_string(),
        predicates: vec![
            ("shared".to_string(), 1),
            ("done-a".to_string(), 1),
            ("done-b".to_string(), 1),
        ],
        actions: vec![
            schema(
                "achieve-a",
                vec![atom("shared", &["?x"])],
                vec![atom("done-a", &["?x"])],
                vec![atom("shared", &["?x"])],
            ),
            schema(
                "achieve-b",
                vec![atom("shared", &["?x"])],
                vec![atom("done-b", &["?x"])],
                Vec::new(),
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

fn mutex_saturated_problem() -> Pddl8Problem {
    Pddl8Problem {
        name: "goal-mutex-saturated-1".to_string(),
        domain: "goal-mutex-saturated".to_string(),
        objects: vec!["potato".to_string()],
        init: vec![atom("shared", &["potato"])],
        goal: vec![atom("done-a", &["potato"]), atom("done-b", &["potato"])],
        object_types: Vec::new(),
        fn_values: Vec::new(),
        timed_inits: Vec::new(),
        preferences: Vec::new(),
        metric: None,
    }
}

// ---------------------------------------------------------------------------
// Fixture B — control: structurally identical (same predicate arities, same
// two-action/one-object shape, same goal-atom count) EXCEPT each action has
// its OWN precondition atom (`ready-a`/`ready-b`), so no delete/precondition
// overlap exists between them and no `:mutex` edge is derivable. Isolates
// mutex-coupling — not the domain's general shape — as the causal mechanism.
// ---------------------------------------------------------------------------

fn independent_control_domain() -> Pddl8Domain {
    Pddl8Domain {
        name: "goal-independent-control".to_string(),
        predicates: vec![
            ("ready-a".to_string(), 1),
            ("ready-b".to_string(), 1),
            ("done-a".to_string(), 1),
            ("done-b".to_string(), 1),
        ],
        actions: vec![
            schema(
                "achieve-a",
                vec![atom("ready-a", &["?x"])],
                vec![atom("done-a", &["?x"])],
                vec![atom("ready-a", &["?x"])],
            ),
            schema(
                "achieve-b",
                vec![atom("ready-b", &["?x"])],
                vec![atom("done-b", &["?x"])],
                vec![atom("ready-b", &["?x"])],
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

fn independent_control_problem() -> Pddl8Problem {
    Pddl8Problem {
        name: "goal-independent-control-1".to_string(),
        domain: "goal-independent-control".to_string(),
        objects: vec!["potato".to_string()],
        init: vec![atom("ready-a", &["potato"]), atom("ready-b", &["potato"])],
        goal: vec![atom("done-a", &["potato"]), atom("done-b", &["potato"])],
        object_types: Vec::new(),
        fn_values: Vec::new(),
        timed_inits: Vec::new(),
        preferences: Vec::new(),
        metric: None,
    }
}

// ---------------------------------------------------------------------------
// Test 1 — direct mechanism inspection: the REAL Datalog derivation produces
// a genuine `:mutex` edge between the two goal atoms' sole achievers, and
// `partition_goals` unions them into exactly one component because of it
// (not achiever-set intersection, not custody conflict).
// ---------------------------------------------------------------------------

test!(
    genuine_datalog_mutex_between_sole_achievers_unions_both_goal_atoms_into_one_partition_component,
    {
        // Arrange: ground + lift the mutex-saturated surface exactly as
        // `decompose_with` does internally (PROJ-702/704 steps 1-3), reading
        // the real on-disk rule files.
        let domain = mutex_saturated_domain();
        let problem = mutex_saturated_problem();
        let ground = GroundProblem::build(&domain, &problem, Some(DECOMP_MAX_GROUND))
            .expect("mutex-saturated grounding");

        // Premise guard: TWO distinct goal atoms, not one — this is what
        // distinguishes this fixture from the "adjacent" single-atom-goal
        // scenario (`decomp_test.rs::
        // single_atom_goal_yields_no_admissible_decomposition`), which
        // never has a second component to union in the first place.
        assert_eq!(
            ground.goal.len(),
            2,
            "premise: this fixture needs two independently-labeled goal atoms, \
             unlike the adjacent single-atom-goal scenario"
        );

        let base = "urn:cng:test:decomp:negcorpus:mutex-saturated:edges";
        let store = lift_ground(&ground, &problem.objects, &domain.name, &problem.name, base)?;
        let queries = QuerySet::load(&decomp_queries_dir())?;
        let (rules_text, resources_text) = read_rules_texts();

        // Act: the REAL Datalog derivation (rules/decomp.dl), not a
        // hand-fabricated DerivedEdges.
        let edges: DerivedEdges =
            derive_edges(&store, &ground, base, &queries, &rules_text, &resources_text)?;

        // Assert: each goal atom has exactly one achiever, and those two
        // achievers ARE distinct actions (rules out the "same achiever
        // action" coupling path in `search::coupled`, isolating mutex).
        let achievers_a = edges
            .achievers
            .get("done-a(potato)")
            .expect("done-a(potato) must have a derived achiever");
        let achievers_b = edges
            .achievers
            .get("done-b(potato)")
            .expect("done-b(potato) must have a derived achiever");
        assert_eq!(achievers_a.len(), 1, "done-a(potato) has exactly one achiever");
        assert_eq!(achievers_b.len(), 1, "done-b(potato) has exactly one achiever");
        let achiever_a = achievers_a.iter().next().expect("achiever_a present");
        let achiever_b = achievers_b.iter().next().expect("achiever_b present");
        assert_eq!(achiever_a, "achieve-a(potato)");
        assert_eq!(achiever_b, "achieve-b(potato)");
        assert_ne!(
            achiever_a, achiever_b,
            "coupling must come from the mutex edge, not a shared achiever action"
        );

        // Assert: the REAL derived `:mutex` edge connects exactly those two
        // achievers (symmetric closure means either ordered pair is present
        // in the set — `rules.rs` inserts both directions).
        let mutex_pair_present = edges
            .mutex
            .contains(&(achiever_a.clone(), achiever_b.clone()))
            || edges
                .mutex
                .contains(&(achiever_b.clone(), achiever_a.clone()));
        assert!(
            mutex_pair_present,
            "rules/decomp.dl's delEff/pre mutex rule must derive a genuine mutex edge \
             between achieve-a(potato) and achieve-b(potato); derived mutex set = {:?}",
            edges.mutex
        );

        // Assert: NOT a custody conflict — isolates plain resource-consuming
        // mutex (item 5's mechanism) from custody-conflict coupling (a
        // different edge kind `search::coupled` also treats as coupling).
        // `shared` is not a `rules/decomp-resources.dl`-classified predicate,
        // so no `:custodyConflict` should ever be derivable here.
        assert!(
            !edges
                .custody
                .contains(&(achiever_a.clone(), achiever_b.clone()))
                && !edges
                    .custody
                    .contains(&(achiever_b.clone(), achiever_a.clone())),
            "this fixture's coupling must be pure mutex, not custody conflict"
        );

        // Act: partition the goal atoms exactly as `decompose_with` does
        // (PROJ-705 step 4).
        let components = partition_goals(&ground.goal, &edges);

        // Assert: exactly ONE component holding both goal atoms — the
        // mutex edge unioned them; a split is therefore structurally
        // inadmissible (never even candidate-enumerable), not merely
        // unbeneficial or rejected after being tried.
        assert_eq!(
            components.len(),
            1,
            "genuine mutex between the two goal atoms' sole achievers must union them into \
             ONE partition component, not leave two separate ones; components = {components:?}"
        );
        assert_eq!(components[0].len(), 2);
        let labels: Vec<String> = components[0].iter().map(|a| a.label()).collect();
        assert_eq!(labels, vec!["done-a(potato)".to_string(), "done-b(potato)".to_string()]);
    }
);

// ---------------------------------------------------------------------------
// Test 2 — end-to-end: `decompose()` on the mutex-saturated fixture lands on
// `NoAdmissibleDecomposition` with an EMPTY rejected-candidate ledger (the
// signature of structural, partition-time inadmissibility) rather than a
// ledger containing an `Inadmissible`/`CNG_R22` split receipt (which would
// mean the mutex was instead caught by `check_interference` on an
// already-enumerated candidate — a different failure mode, doctrine item 3).
// ---------------------------------------------------------------------------

test!(
    mutex_saturated_goals_force_no_admissible_decomposition_with_zero_candidates_ever_attempted,
    {
        // Arrange.
        let domain = mutex_saturated_domain();
        let problem = mutex_saturated_problem();
        assert_eq!(
            problem.goal.len(),
            2,
            "premise: two goal atoms, unlike the adjacent single-atom-goal scenario"
        );
        let out = scratch_dir("mutex-saturated-end-to-end");

        // Act.
        let result = decompose(
            &domain,
            &problem,
            &out,
            "urn:cng:test:decomp:negcorpus:mutex-saturated:e2e",
        )?;

        // Assert: EXACT typed outcome (not a loose `matches!`) — the same
        // outcome variant/rejected-count as the adjacent single-atom
        // scenario, but reached via a completely different, genuinely
        // mutex-coupled two-goal-atom domain.
        assert_eq!(
            result.outcome,
            DecompositionOutcome::NoAdmissibleDecomposition { rejected: 0 }
        );

        // Assert: the receipt ledger contains ONLY the single-actor
        // candidate — no split candidate was ever enumerated, let alone
        // tried and rejected. This is the load-bearing distinction from
        // doctrine item 3 (CNG_R22 interference on an enumerated split,
        // which would show up here as a SECOND, `Inadmissible` receipt
        // naming CNG_R22): a non-empty rejected list would mean this test
        // was accidentally exercising the wrong mechanism.
        assert_eq!(
            result.candidate_receipts.len(),
            1,
            "zero split candidates may be enumerated; the mutex-coupled goal atoms must never \
             separate into two components in the first place"
        );
        assert_eq!(
            result.candidate_receipts[0].candidate_id,
            SINGLE_ACTOR_CANDIDATE_ID
        );
        assert_eq!(
            result.candidate_receipts[0].status,
            CandidateStatus::Selected
        );
        assert_eq!(result.candidate_receipts[0].reason, "selected");
        assert!(
            !result.candidate_receipts[0]
                .reason
                .to_lowercase()
                .contains("cng_r22"),
            "the single surviving receipt must not carry an interference-refusal reason; the \
             mechanism under test is partition-time mutex coupling, not tape-level interference"
        );

        // Assert: the single-actor plan itself IS admitted (rules out
        // doctrine item 1, "unreachable helper goal" / an unsolvable
        // problem) — both goal atoms ARE jointly achievable, just never as
        // an admissible split. BFS must sequence achieve-b before
        // achieve-a, since achieve-a deletes the `shared(potato)`
        // precondition achieve-b needs.
        assert_eq!(result.subworkflows.len(), 1);
        assert_eq!(result.subworkflows[0].role, "single");
        let labels: Vec<&str> = result.subworkflows[0]
            .tape
            .ops
            .iter()
            .map(|op| op.action.label.as_str())
            .collect();
        assert_eq!(
            labels,
            vec!["achieve-b(potato)", "achieve-a(potato)"],
            "the only lawful total order runs achieve-b before achieve-a, forced by the \
             genuine resource-consumption mutex between them"
        );

        // Assert: real evidence written, never skipped for this outcome.
        assert!(result.result_graph_path.exists());
    }
);

// ---------------------------------------------------------------------------
// Test 3 — control: the structurally identical shape WITHOUT the mutex
// (disjoint per-goal preconditions) DOES enumerate a split and DOES select
// it, isolating mutex-coupling (not domain shape, not goal count, not object
// count) as the specific cause of Test 2's `NoAdmissibleDecomposition`.
// ---------------------------------------------------------------------------

test!(
    structurally_identical_goals_without_mutex_coupling_do_split_and_are_selected,
    {
        // Arrange.
        let domain = independent_control_domain();
        let problem = independent_control_problem();
        let out = scratch_dir("independent-control");

        // Act.
        let result = decompose(
            &domain,
            &problem,
            &out,
            "urn:cng:test:decomp:negcorpus:mutex-saturated:control",
        )?;

        // Assert: with the mutex removed, a 2-subworkflow split IS
        // admissible and wins the selection law (shorter makespan: 1 vs the
        // single actor's 2) — the exact opposite of Test 2's outcome, on a
        // domain of identical shape (2 actions, 1 object, 2 goal atoms).
        assert_eq!(
            result.outcome,
            DecompositionOutcome::Selected {
                candidate_id: "done-a(potato)".to_string(),
                subworkflows: 2,
            }
        );
        assert_eq!(result.subworkflows.len(), 2);
        assert_eq!(result.subworkflows[0].role, "helper");
        assert_eq!(result.subworkflows[1].role, "main");

        // Assert: BOTH goal-atom candidates were enumerated and receipted
        // admissible (single-actor + two splits) — proof the mutex-free
        // shape really does separate into two components, confirming Test
        // 2's single-component collapse was caused by the mutex edge and
        // not by this fixture family's general shape.
        assert_eq!(result.candidate_receipts.len(), 3);
        let split_ids: std::collections::BTreeSet<&str> = result
            .candidate_receipts
            .iter()
            .filter(|r| r.candidate_id != SINGLE_ACTOR_CANDIDATE_ID)
            .map(|r| r.candidate_id.as_str())
            .collect();
        assert_eq!(
            split_ids,
            std::collections::BTreeSet::from(["done-a(potato)", "done-b(potato)"])
        );
        for receipt in &result.candidate_receipts {
            assert_ne!(
                receipt.status,
                CandidateStatus::Inadmissible,
                "the mutex-free control must admit every candidate; {} was inadmissible: {}",
                receipt.candidate_id,
                receipt.reason
            );
        }

        assert!(result.result_graph_path.exists());
    }
);

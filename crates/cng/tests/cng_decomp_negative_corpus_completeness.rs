//! Closes doctrine §18 negative-corpus items 6 and 7
//! (`docs/releases/v26.7.10/DEFINITION_OF_DONE.md`), both marked PARTIAL in
//! `docs/releases/v26.7.10/DOD_SIGNOFF.md`'s §18 detail table:
//!
//! - Item 6 (`NO_BENEFICIAL_DECOMPOSITION`): prior evidence
//!   (`potato_decomposition_is_typed_receipted_and_replayable`,
//!   `ipc_corpus_seeds_plan_decompose_and_regenerate_byte_identically`)
//!   ACCEPTS the outcome as one of three legal match arms but never FORCES
//!   it — no fixture on disk was engineered so the algorithm actually lands
//!   there. This file adds one that does, with an exact-value assertion on
//!   the typed outcome (never a loose `matches!`).
//! - Item 7 (injected canned subgoal): the existing
//!   `no_canned_helper_subgoal_across_incompatible_variants`
//!   (`tests/cng_ipc_corpus.rs`) proves candidate-id disjointness across
//!   domains with entirely different vocabularies (potato's `cooked`/
//!   `placed` vs blocksworld's `on`/`clear`, etc.) — a coarse check a
//!   canned/keyword-matched decomposition rule would trivially pass, since
//!   the predicate names never collide in the first place. This file goes
//!   one step further: two domains sharing the EXACT SAME goal-atom label
//!   text (same predicate names, same object names) but a genuinely
//!   different achiever chain for one of them, and asserts the receipted
//!   output differs anyway — proof the engine re-derives from THIS run's
//!   admitted facts rather than returning a cached/canned answer keyed on
//!   the label string.
//!
//! Typed-struct fixtures only (house style, `tests/no_inline_ttl_guard.rs`
//! enforced) — no inline PDDL/Turtle/SPARQL. Test-only file; no production
//! source under `crates/cng/src/` is touched by this file.

#![cfg(feature = "bench")]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use chicago_tdd_tools::prelude::*;

use bcinr_pddl::{Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8Problem};

use cng::bench::decomp::{
    decompose, CandidateStatus, DecompositionOutcome, SINGLE_ACTOR_CANDIDATE_ID,
};

fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/decomp-negative-corpus-completeness")
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

/// One-parameter (`?x`) action schema — the shape every action in both
/// fixture domains below uses (mirrors `decomp_test.rs`'s `schema` helper).
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

// ---------------------------------------------------------------------------
// Item 6 — NO_BENEFICIAL_DECOMPOSITION forced.
// ---------------------------------------------------------------------------

/// Two independent goal chains (potato/cooked, fork/placed — same shape as
/// `decomp_test.rs`'s `kitchen_domain`) EXCEPT `fetch-drawer(?x)` carries an
/// extra literal (non-variable) precondition `cooked(potato)`: a genuine
/// STRIPS cross-chain dependency, not a schema variable, so it applies to
/// every grounding of `fetch-drawer` regardless of `?x`.
///
/// This forces every lawful plan — single-actor AND the `cooked(potato)`
/// split — through the identical 4-step total order `fetch-pantry(potato),
/// cook(potato), fetch-drawer(fork), place(fork)`: no plan can place
/// `fetch-drawer` before `cook` satisfies `cooked(potato)`. The split is
/// still admissible (interference: the one ordered pair `cook -> fetch-
/// drawer` is exempted by `check_interference`'s `mustPrecede` skip;
/// release closure: `held(potato)` is acquired and released entirely
/// within the helper, never surviving into s′) — but its selection score
/// TIES the single actor on makespan (both walk the same 4-node critical
/// path) and LOSES on dispatch cost (2 subworkflows' worth of
/// `DISPATCH_OVERHEAD_STEPS` vs 1). That is exactly obligation 6 of
/// `DEFINITION_OF_DONE.md` §18: a split admissible but never beneficial.
///
/// `partition_goals`'s coupling check (`search.rs::coupled`) only inspects
/// `achievers`/`mutex`/`custody` — not `mustPrecede` — so `cooked(potato)`
/// and `placed(fork)` remain separate goal components and the split stays
/// enumerable despite the forced cross-chain ordering; this is the precise
/// mechanism this fixture exploits to make an admissible-but-not-beneficial
/// split reachable at all.
fn forced_precedence_domain() -> Pddl8Domain {
    Pddl8Domain {
        name: "kitchen-forced-precedence".to_string(),
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
                // Extra literal (ground-constant) precondition, not `?x`:
                // every fetch-drawer grounding needs the potato chain done.
                vec![atom("in-drawer", &["?x"]), atom("cooked", &["potato"])],
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

fn forced_precedence_problem() -> Pddl8Problem {
    Pddl8Problem {
        name: "kitchen-forced-precedence-1".to_string(),
        domain: "kitchen-forced-precedence".to_string(),
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

test!(
    splits_admissible_but_not_beneficial_forces_no_beneficial_decomposition,
    {
        // Arrange.
        let domain = forced_precedence_domain();
        let problem = forced_precedence_problem();
        let out = scratch_dir("no-beneficial");

        // Act.
        let result = decompose(
            &domain,
            &problem,
            &out,
            "urn:cng:test:decomp:negcorpus:no-beneficial",
        )?;

        // Assert: the outcome is EXACTLY NoBeneficialDecomposition naming the
        // admissible-but-losing split — an exact-value comparison, not a loose
        // `matches!`, so this fails loudly if the algorithm ever lands on
        // NoAdmissibleDecomposition (no split passed proof obligations) or
        // Selected (the split somehow won the tie-break) instead.
        assert_eq!(
            result.outcome,
            DecompositionOutcome::NoBeneficialDecomposition {
                best_rejected_id: "cooked(potato)".to_string(),
            }
        );

        // Assert: the returned result still carries the SINGLE-ACTOR candidate's
        // own receipt and plan — never a silent fallback to some other
        // computation. The typed outcome variant above already IS the proof;
        // this additionally pins the exact plan it carries.
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
            vec![
                "fetch-pantry(potato)",
                "cook(potato)",
                "fetch-drawer(fork)",
                "place(fork)",
            ],
            "the forced precedence must produce the unique 4-step total order"
        );

        // Assert: candidate 0 is the single actor and the selection law ran and
        // marked it Selected (proof this was scored, not a NoAdmissible short
        // circuit before selection).
        assert_eq!(
            result.candidate_receipts[0].candidate_id,
            SINGLE_ACTOR_CANDIDATE_ID
        );
        assert_eq!(
            result.candidate_receipts[0].status,
            CandidateStatus::Selected
        );

        // Assert: the "cooked(potato)" split is receipted as ADMISSIBLE — it
        // passed every proof obligation — never Inadmissible, never silently
        // dropped from the ledger just because it lost the argmin.
        let split = result
            .candidate_receipts
            .iter()
            .find(|r| r.candidate_id == "cooked(potato)")
            .expect("the admissible split candidate must be receipted, not silently dropped");
        assert_eq!(split.status, CandidateStatus::Admissible);

        // Assert the exact lexicographic reason splitting lost: tied makespan,
        // strictly worse dispatch cost — the load-bearing precondition of this
        // scenario, pinned as exact numbers so a future scoring-law change that
        // silently breaks this fixture's premise fails loudly here.
        let single = &result.candidate_receipts[0];
        assert_eq!(single.score.makespan, 4);
        assert_eq!(
            split.score.makespan, 4,
            "split must TIE the single actor on makespan"
        );
        assert_eq!(single.score.dispatch_cost, 6);
        assert_eq!(
            split.score.dispatch_cost, 8,
            "split must LOSE on dispatch-cost overhead"
        );
        assert!(
        split.score > single.score,
        "the split's lexicographic score must be strictly worse, proving NoBeneficialDecomposition \
         is the correct argmin and not an accident of equal scores"
    );

        // Assert: real evidence was written (not skipped for a "success"
        // outcome that emits no artifact).
        assert!(result.result_graph_path.exists());
    }
);

// ---------------------------------------------------------------------------
// Item 7 — a MORE SUBTLE canned/hardcoded-subgoal bug than the existing
// cross-domain-vocabulary disjointness check.
// ---------------------------------------------------------------------------

/// Same two independent goal chains as `forced_precedence_domain` (minus the
/// forced cross-chain precondition), with `cooked(?x)` achieved in exactly
/// 2 steps: `fetch-pantry`, `cook`.
fn plain_domain() -> Pddl8Domain {
    Pddl8Domain {
        name: "kitchen-two-chain-plain".to_string(),
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

/// Same goal-atom labels as `plain_domain`'s problem — same predicate
/// names, same object names — but `cooked(?x)` now requires an extra
/// `heat(?x)` step first (`cook`'s precondition gains `heated(?x)`), so the
/// achiever chain for the IDENTICAL goal atom `cooked(potato)` is one
/// action longer: `fetch-pantry, heat, cook` instead of `fetch-pantry,
/// cook`. This is the "same goal predicate names, different achiever
/// structure" case named in the task: a decomposition engine that cached or
/// pattern-matched a candidate's receipt off the goal-atom LABEL TEXT alone
/// — rather than re-deriving achievers/scores from the admitted domain of
/// THIS run — would return the same score/receipt/graph bytes for both
/// domains. The existing `no_canned_helper_subgoal_across_incompatible_
/// variants` test (`tests/cng_ipc_corpus.rs`) cannot catch this: its
/// domains (potato vs blocksworld vs ...) never share a predicate name, so
/// candidate-id disjointness holds trivially regardless of whether the
/// engine actually re-derives anything.
fn heat_gated_domain() -> Pddl8Domain {
    Pddl8Domain {
        name: "kitchen-two-chain-heat-gated".to_string(),
        predicates: vec![
            ("in-pantry".to_string(), 1),
            ("in-drawer".to_string(), 1),
            ("held".to_string(), 1),
            ("heated".to_string(), 1),
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
                "heat",
                vec![atom("held", &["?x"])],
                vec![atom("heated", &["?x"])],
                Vec::new(),
            ),
            schema(
                "cook",
                vec![atom("held", &["?x"]), atom("heated", &["?x"])],
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

fn two_chain_problem(domain_name: &str) -> Pddl8Problem {
    Pddl8Problem {
        name: format!("{domain_name}-1"),
        domain: domain_name.to_string(),
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

test!(
    canned_subgoal_detection_catches_identical_goal_labels_with_different_achiever_structure,
    {
        // Arrange: two domains, one shared goal-atom-label vocabulary.
        let plain_domain = plain_domain();
        let plain_problem = two_chain_problem("kitchen-two-chain-plain");
        let heat_domain = heat_gated_domain();
        let heat_problem = two_chain_problem("kitchen-two-chain-heat-gated");

        // Sanity: the two problems really do share the exact goal-atom
        // label text a canned/keyword-matched rule would key off — the
        // premise this test depends on, pinned so a future fixture edit
        // that breaks it fails loudly here instead of silently passing.
        assert_eq!(
            plain_problem.goal, heat_problem.goal,
            "this test's premise requires identical goal atoms across structurally \
             different domains"
        );

        let out_plain = scratch_dir("canned-plain");
        let out_heat = scratch_dir("canned-heat");

        // Act: decompose both under the SAME base IRI, so only the admitted
        // domain/problem content — never the IRI namespace — can cause any
        // divergence in the emitted graphs.
        let base = "urn:cng:test:decomp:negcorpus:canned";
        let plain_result = decompose(&plain_domain, &plain_problem, &out_plain, base)?;
        let heat_result = decompose(&heat_domain, &heat_problem, &out_heat, base)?;

        // Assert: both runs enumerate a "cooked(potato)" split candidate —
        // the id text IS expected to match, since ids are derived purely
        // from goal-atom labels (`search.rs::candidate_id`), not achiever
        // structure. A bare id-SET comparison (the existing cross-domain
        // test's approach) would therefore NOT catch a canned-answer bug
        // here — this is exactly why this test inspects RECEIPT CONTENT
        // for the shared id, not just its presence/absence.
        let plain_cooked = plain_result
            .candidate_receipts
            .iter()
            .find(|r| r.candidate_id == "cooked(potato)")
            .expect("plain domain must enumerate the cooked(potato) split");
        let heat_cooked = heat_result
            .candidate_receipts
            .iter()
            .find(|r| r.candidate_id == "cooked(potato)")
            .expect("heat-gated domain must enumerate the cooked(potato) split");

        // Assert: the SAME candidate id carries a DIFFERENT receipt in each
        // run. The heat-gated variant's achiever chain for cooked(potato)
        // is one action longer (fetch-pantry, heat, cook vs fetch-pantry,
        // cook), so a correctly re-derived score must differ; pinned as
        // exact numbers, not just inequality, so the fixture's own claim
        // about WHY they differ stays checked.
        assert_eq!(plain_cooked.score.makespan, 2, "plain chain is 2 steps");
        assert_eq!(heat_cooked.score.makespan, 3, "heat-gated chain is 3 steps");
        assert_ne!(
            plain_cooked.score.makespan, heat_cooked.score.makespan,
            "identical candidate id must not mask a genuinely different achiever chain \
             (a canned/cached answer keyed on the id string would make these equal)"
        );
        assert_ne!(
            plain_cooked.score.dispatch_cost,
            heat_cooked.score.dispatch_cost
        );

        // Assert: the emitted receipt graphs differ byte-for-byte under the
        // SAME base IRI — no cached artifact reused across structurally
        // different domains despite the identical candidate id and
        // identical goal-atom labels.
        let plain_bytes =
            fs::read(&plain_result.result_graph_path).expect("read plain result graph");
        let heat_bytes = fs::read(&heat_result.result_graph_path).expect("read heat result graph");
        assert_ne!(
            plain_bytes, heat_bytes,
            "structurally different domains sharing goal-atom labels must not share a \
             canned result graph"
        );

        // Assert: this run's heat-gated domain derives its own candidate
        // set fresh (both still contain "cooked(potato)" — the components
        // are independent chains in both domains, so this is NOT expected
        // to differ; a same-set outcome here is the correct, non-canned
        // result, not evidence of caching, precisely because the RECEIPT
        // CONTENT above already proved fresh re-derivation).
        let plain_ids: BTreeSet<&str> = plain_result
            .candidate_receipts
            .iter()
            .map(|r| r.candidate_id.as_str())
            .collect();
        let heat_ids: BTreeSet<&str> = heat_result
            .candidate_receipts
            .iter()
            .map(|r| r.candidate_id.as_str())
            .collect();
        assert_eq!(
            plain_ids, heat_ids,
            "independent-chain topology is identical in both domains"
        );
    }
);

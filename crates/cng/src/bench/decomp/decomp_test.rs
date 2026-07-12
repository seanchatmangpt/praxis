//! Unit tests for the decomposition module tree (PROJ-702..710): typed
//! Rust-struct fixtures only — no inline Turtle, PDDL, or SPARQL. The
//! two-chain kitchen domain is constructed as `bcinr_pddl` structs; the
//! refusal tests (CNG_R21..R24) drive each gate directly.

use chicago_tdd_tools::prelude::*;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use bcinr_pddl::parse::problem_from_pddl;
use bcinr_pddl::{
    Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8GroundAction, Pddl8GroundAtom, Pddl8Problem,
    Pddl8Tape, Pddl8TapeOp,
};
use oxigraph::model::{GraphName, NamedNode, Quad, Term};
// PROJ-733: production code now grounds via pddl_index::ground::
// IndexedGroundProblem (see decomp/mod.rs's module doc) — these tests call
// into that production code with `&GroundProblem` params, so the fixture's
// own grounding must use the same aliased type, not bcinr_pddl's.
use pddl_index::ground::IndexedGroundProblem as GroundProblem;

use crate::bench::templates::QuerySet;
use crate::powl::{CngRefusal, Powl};

use super::*;

fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/chatman/cng-tests/decomp_{}", std::process::id()))
        .join(test_name);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn atom(pred: &str, args: &[&str]) -> Pddl8Atom {
    Pddl8Atom {
        pred: pred.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
    }
}

fn ground_atom(pred: &str, args: &[&str]) -> Pddl8GroundAtom {
    Pddl8GroundAtom {
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

/// Two independent goal chains over disjoint objects: fetch+cook the
/// potato, fetch+place the fork. `held` is the admitted resource predicate
/// (rules/decomp-resources.dl), released by both cook and place.
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

fn ground_action(
    label: &str,
    pre: Vec<Pddl8GroundAtom>,
    add: Vec<Pddl8GroundAtom>,
    del: Vec<Pddl8GroundAtom>,
) -> Pddl8GroundAction {
    Pddl8GroundAction {
        schema_name: label.split('(').next().unwrap_or(label).to_string(),
        label: label.to_string(),
        preconditions: pre,
        add_effects: add,
        del_effects: del,
    }
}

fn tape_of(actions: Vec<Pddl8GroundAction>) -> Pddl8Tape {
    Pddl8Tape {
        ops: actions
            .into_iter()
            .enumerate()
            .map(|(i, action)| Pddl8TapeOp {
                index: i as u8,
                label: action.label.clone(),
                pred_mask: if i == 0 { 0 } else { 1u64 << (i - 1) },
                action,
            })
            .collect(),
    }
}

test!(replay_verifies_preconditions_and_applies_effects, {
    // Arrange: one lawful step.
    let init: BTreeSet<Pddl8GroundAtom> = [ground_atom("in-pantry", &["potato"])]
        .into_iter()
        .collect();
    let tape = tape_of(vec![ground_action(
        "fetch-pantry(potato)",
        vec![ground_atom("in-pantry", &["potato"])],
        vec![ground_atom("held", &["potato"])],
        vec![ground_atom("in-pantry", &["potato"])],
    )]);

    // Act.
    let s_prime = replay_to_interface_state(&init, &tape)?;

    // Assert: state − del + add.
    assert!(s_prime.contains(&ground_atom("held", &["potato"])));
    assert!(!s_prime.contains(&ground_atom("in-pantry", &["potato"])));
});

test!(tampered_tape_refuses_cng_r23_interface_state_mismatch, {
    // Arrange: the step's precondition does not hold in init.
    let init: BTreeSet<Pddl8GroundAtom> = BTreeSet::new();
    let tape = tape_of(vec![ground_action(
        "cook(potato)",
        vec![ground_atom("held", &["potato"])],
        vec![ground_atom("cooked", &["potato"])],
        vec![ground_atom("held", &["potato"])],
    )]);

    // Act.
    let refusal = replay_to_interface_state(&init, &tape).unwrap_err();

    // Assert: typed CNG_R23 naming step and atom.
    assert_eq!(refusal.code(), "CNG_R23");
    assert!(matches!(
        refusal,
        CngRefusal::InterfaceStateMismatch { step: 0, .. }
    ));
});

test!(concurrent_clobber_refuses_cng_r22_interference, {
    // Arrange: helper deletes what main needs; no ordering edge.
    let helper = tape_of(vec![ground_action(
        "steal(fork)",
        vec![],
        vec![],
        vec![ground_atom("held", &["fork"])],
    )]);
    let main = tape_of(vec![ground_action(
        "place(fork)",
        vec![ground_atom("held", &["fork"])],
        vec![ground_atom("placed", &["fork"])],
        vec![],
    )]);
    let edges = DerivedEdges::default();

    // Act.
    let refusal = check_interference(&helper, &main, &edges).unwrap_err();

    // Assert: typed CNG_R22.
    assert_eq!(refusal.code(), "CNG_R22");
});

test!(ordered_pair_is_not_interference, {
    // Arrange: same clobber, but a derived mustPrecede edge orders it.
    let helper = tape_of(vec![ground_action(
        "steal(fork)",
        vec![],
        vec![],
        vec![ground_atom("held", &["fork"])],
    )]);
    let main = tape_of(vec![ground_action(
        "place(fork)",
        vec![ground_atom("held", &["fork"])],
        vec![ground_atom("placed", &["fork"])],
        vec![],
    )]);
    let mut edges = DerivedEdges::default();
    edges
        .must_precede
        .insert(("place(fork)".to_string(), "steal(fork)".to_string()));

    // Act + Assert: ordered pairs are exempt.
    check_interference(&helper, &main, &edges)?;
});

test!(unreleased_resource_refuses_cng_r24, {
    // Arrange: helper leaves held(potato) (resource) in s′ beyond init;
    // main never consumes it.
    let init: BTreeSet<Pddl8GroundAtom> = BTreeSet::new();
    let s_prime: BTreeSet<Pddl8GroundAtom> =
        [ground_atom("held", &["potato"])].into_iter().collect();
    let helper = tape_of(vec![ground_action(
        "fetch-pantry(potato)",
        vec![],
        vec![ground_atom("held", &["potato"])],
        vec![],
    )]);
    let main = tape_of(vec![ground_action("noop(x)", vec![], vec![], vec![])]);
    let mut edges = DerivedEdges::default();
    edges.resource_atoms.insert("held(potato)".to_string());

    // Act.
    let refusal = check_release_closure(&s_prime, &init, &helper, &main, &edges).unwrap_err();

    // Assert: typed CNG_R24 naming the resource.
    assert_eq!(refusal.code(), "CNG_R24");
    assert!(matches!(
        refusal,
        CngRefusal::ResourceUnreleased { ref resource, .. } if resource == "held(potato)"
    ));
});

test!(forced_inadmissible_candidate_refuses_cng_r21, {
    // Arrange: one admissible single-actor receipt, one inadmissible split.
    let mut receipts = vec![
        CandidateReceipt {
            candidate_id: SINGLE_ACTOR_CANDIDATE_ID.to_string(),
            status: CandidateStatus::Admissible,
            reason: "admissible".to_string(),
            score: score_single(4),
        },
        CandidateReceipt {
            candidate_id: "cooked(potato)".to_string(),
            status: CandidateStatus::Inadmissible,
            reason: "CNG_R04: subproblem admits no plan".to_string(),
            score: Score {
                makespan: 0,
                dispatch_cost: 0,
                risk: 0,
            },
        },
    ];

    // Act: demand the inadmissible candidate.
    let refusal = select(&mut receipts, Some("cooked(potato)")).unwrap_err();

    // Assert: typed CNG_R21, never a silent fallback.
    assert_eq!(refusal.code(), "CNG_R21");
});

test!(cyclic_composed_order_refuses_cng_r21, {
    // Arrange: a 2-node cycle.
    let edges: BTreeSet<(usize, usize)> = [(0, 1), (1, 0)].into_iter().collect();

    // Act.
    let refusal = longest_path_nodes(2, &edges, "cyclic-candidate").unwrap_err();

    // Assert.
    assert_eq!(refusal.code(), "CNG_R21");
});

test!(single_actor_is_always_candidate_zero, {
    // Arrange: two trivially independent components.
    let components = vec![
        vec![ground_atom("cooked", &["potato"])],
        vec![ground_atom("placed", &["fork"])],
    ];

    // Act.
    let candidates = enumerate_candidates(&components);

    // Assert: candidate 0 is the single-actor candidate; splits follow in
    // canonical id order; bound respected.
    assert_eq!(candidates[0].id, SINGLE_ACTOR_CANDIDATE_ID);
    assert!(candidates[0].helper_goal.is_empty());
    assert!(candidates.len() <= DECOMP_MAX_CANDIDATES);
    let ids: Vec<&str> = candidates[1..].iter().map(|c| c.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "split candidates must be canonically ordered");
});

test!(lift_render_round_trip_preserves_atom_sets, {
    // Arrange: ground the kitchen surface and lift it.
    let domain = kitchen_domain();
    let problem = kitchen_problem();
    let ground = GroundProblem::build(&domain, &problem, Some(DECOMP_MAX_GROUND))
        .expect("kitchen grounding");
    let base = "urn:cng:test:decomp:roundtrip";
    let store = lift_ground(&ground, &problem.objects, &domain.name, &problem.name, base)?;
    let queries = QuerySet::load(&decomp_queries_dir())?;
    let template = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/decomp-problem.template.pddl"),
    )
    .expect("problem template");

    // Act: render the lifted problem back to PDDL text and parse it.
    let source_iri = problem_iri(base, &problem.name);
    let text = render_problem(&store, &source_iri, &queries, &template)?;
    let parsed = problem_from_pddl(&text)
        .map_err(|e| CngRefusal::MalformedTtl(format!("round-trip parse: {e:?}")))?;

    // Assert: lift ∘ render is identity on the object/init/goal sets.
    let set = |atoms: &[Pddl8Atom]| -> BTreeSet<String> {
        atoms
            .iter()
            .map(|a| {
                if a.args.is_empty() {
                    a.pred.clone()
                } else {
                    format!("{}({})", a.pred, a.args.join(","))
                }
            })
            .collect()
    };
    assert_eq!(set(&parsed.init), set(&problem.init));
    assert_eq!(set(&parsed.goal), set(&problem.goal));
    let objects: BTreeSet<&String> = parsed.objects.iter().collect();
    let expected: BTreeSet<&String> = problem.objects.iter().collect();
    assert_eq!(objects, expected);

    // Determinism: rendering twice is byte-identical.
    let again = render_problem(&store, &source_iri, &queries, &template)?;
    assert_eq!(text, again);
});

test!(kitchen_two_chain_selects_a_two_actor_decomposition, {
    // Arrange.
    let domain = kitchen_domain();
    let problem = kitchen_problem();
    let out = scratch_dir("kitchen-two-chain");

    // Act.
    let result = decompose(&domain, &problem, &out, "urn:cng:test:decomp:kitchen")?;

    // Assert: derived (not hardcoded) two-actor decomposition — the
    // independent chains split; helper is the lexicographically least
    // component; both subworkflow tapes are 2 ops; parallel (no cross
    // edges, zero risk); receipts cover single + both splits.
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
    assert_eq!(result.subworkflows[0].tape.ops.len(), 2);
    assert_eq!(result.subworkflows[1].tape.ops.len(), 2);
    assert!(result.cross_edges.is_empty());
    assert!(!result.interface_atoms.is_empty());
    assert_eq!(result.candidate_receipts.len(), 3);
    assert_eq!(
        result.candidate_receipts[0].candidate_id,
        SINGLE_ACTOR_CANDIDATE_ID
    );
    // The composed model is a nested PartialOrder with two subworkflow
    // children and an EMPTY root order (helper ∥ main).
    let Powl::PartialOrder { children, order } = &result.composed_model else {
        panic!("composed model must be a PartialOrder");
    };
    assert_eq!(children.len(), 2);
    assert!(order.is_empty(), "independent chains must compose parallel");
    assert!(result.result_graph_path.exists());
});

test!(single_atom_goal_yields_no_admissible_decomposition, {
    // Arrange: one goal atom — no split is enumerable.
    let domain = kitchen_domain();
    let mut problem = kitchen_problem();
    problem.goal = vec![atom("cooked", &["potato"])];
    let out = scratch_dir("single-atom-goal");

    // Act.
    let result = decompose(&domain, &problem, &out, "urn:cng:test:decomp:single")?;

    // Assert: typed success outcome, single-actor POWL carried, receipted.
    assert_eq!(
        result.outcome,
        DecompositionOutcome::NoAdmissibleDecomposition { rejected: 0 }
    );
    assert_eq!(result.subworkflows.len(), 1);
    assert_eq!(result.subworkflows[0].role, "single");
    assert_eq!(result.candidate_receipts.len(), 1);
    assert!(result.result_graph_path.exists());
});

test!(decompose_is_deterministic_across_runs, {
    // Arrange.
    let domain = kitchen_domain();
    let problem = kitchen_problem();
    let out_a = scratch_dir("determinism-a");
    let out_b = scratch_dir("determinism-b");

    // Act: same inputs, two manufactures.
    let a = decompose(&domain, &problem, &out_a, "urn:cng:test:decomp:det")?;
    let b = decompose(&domain, &problem, &out_b, "urn:cng:test:decomp:det")?;

    // Assert: byte-identical result graphs (CNG_R08 territory otherwise).
    let bytes_a = std::fs::read(&a.result_graph_path).expect("read result a");
    let bytes_b = std::fs::read(&b.result_graph_path).expect("read result b");
    assert_eq!(bytes_a, bytes_b);
    assert_eq!(a.outcome, b.outcome);
});

// PROJ-728/729-followup Gap B: `rules::derive_edges`'s `append_pair_facts`
// (rules.rs) already carries a `CNG_R09 HardcodingSuspicion` refusal for
// when the lifted working store names an action/atom IRI absent from the
// admitted ground surface — a lifted-graph/ground-surface detachment, i.e.
// a fact the real PDDL admission never produced (a canned/fabricated
// artifact injected past the lifter). Every other decomp/ test exercises
// only the SUCCESS path of `derive_edges` (through `decompose()`); grep
// across `crates/cng/{src,tests}` found zero tests forcing this refusal to
// actually fire before this one. This directly closes the DOD_EVIDENCE_MAP
// citation for `CNG_R09` inside decomp/ ("wired but untested" — distinct
// from PROJ-713's `no_canned_helper_subgoal_across_incompatible_variants`,
// which structurally proves a DIFFERENT canned-artifact concern: candidate
// ids in `search.rs` are a pure function of the admitted goal-atom set, so
// there is no runtime boundary there for a `CNG_R09` refusal to guard —
// see the session report for the full reasoning).
test!(
    detached_graph_action_refuses_cng_r09_hardcoding_suspicion,
    {
        // Arrange: ground + lift the kitchen surface exactly as the
        // lift-render round-trip test does, and confirm `derive_edges`
        // succeeds on the untampered graph first (so a later refusal is
        // provably caused by the tamper, not a broken fixture).
        let domain = kitchen_domain();
        let problem = kitchen_problem();
        let ground = GroundProblem::build(&domain, &problem, Some(DECOMP_MAX_GROUND))
            .expect("kitchen grounding");
        let base = "urn:cng:test:decomp:detached";
        let store = lift_ground(&ground, &problem.objects, &domain.name, &problem.name, base)?;
        let queries = QuerySet::load(&decomp_queries_dir())?;
        let (rules_text, resources_text) = load_rules_texts()?;
        derive_edges(
            &store,
            &ground,
            base,
            &queries,
            &rules_text,
            &resources_text,
        )?;

        // Act: insert a lawfully-SHAPED `pddl:Action`/`pddl:precondition` fact
        // whose action IRI names an action this problem never admitted
        // ("roast-potato" is not a kitchen-domain schema instantiation) —
        // pointing at a REAL atom (`held(potato)`) so the only tampered axis is
        // the action identity, isolating exactly the failure `append_pair_facts`
        // is written to catch.
        let pddl = PDDL_STRIPS_PREFIX;
        let fake_action =
            NamedNode::new(format!("{base}/action/roast-potato")).expect("fake action IRI");
        let real_atom = NamedNode::new(atom_iri(base, &ground_atom("held", &["potato"])))
            .expect("real atom IRI");
        store
            .insert(&Quad::new(
                fake_action.clone(),
                NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
                    .expect("rdf:type IRI"),
                Term::NamedNode(NamedNode::new(format!("{pddl}Action")).expect("Action class IRI")),
                GraphName::DefaultGraph,
            ))
            .expect("insert fake action rdf:type");
        store
            .insert(&Quad::new(
                fake_action,
                NamedNode::new(format!("{pddl}precondition")).expect("precondition predicate IRI"),
                Term::NamedNode(real_atom),
                GraphName::DefaultGraph,
            ))
            .expect("insert fake precondition fact");

        // Assert: typed CNG_R09, never a silent/derived fallback.
        let refusal = derive_edges(
            &store,
            &ground,
            base,
            &queries,
            &rules_text,
            &resources_text,
        )
        .unwrap_err();
        assert_eq!(refusal.code(), "CNG_R09");
        assert!(matches!(refusal, CngRefusal::HardcodingSuspicion(_)));
    }
);

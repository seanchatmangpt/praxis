//! PROJ-711/712/713 — clean-room IPC corpus generators, doctrine §13
//! negative corpus, and the anti-hardcoding gate.
//!
//! - Corpus law (PROJ-711): every domain × seed regenerates byte-identically,
//!   plans under the blind-BFS bound at the gated size, and decomposes to a
//!   TYPED outcome. The unit test covers seeds 0..3 per domain; the full
//!   20-seed corpus is the benchmark run, not this test.
//! - Negative corpus (PROJ-712, doctrine §13): each named failure scenario
//!   refuses with its exact `CNG_Rxx` code — via on-disk `.pddl` fixtures
//!   under `tests/fixtures/decomp-negative/` for plan-level scenarios, and
//!   typed-struct proofs against the decomposition proof obligations for
//!   the replay/interference/custody scenarios. No inline PDDL/Turtle/SPARQL.
//! - Anti-hardcoding gate (PROJ-713): `SwappedGoalIdentities` permutations
//!   are GUARANTEED (by generator construction) to change the goal text;
//!   the emitted decomposition receipt graphs and plan tapes must change
//!   causally, and helper subgoal candidate ids must never repeat across
//!   incompatible domain variants (canned-subgoal detection).

#![cfg(feature = "bench")]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use chicago_tdd_tools::prelude::*;

use bcinr_pddl::parse::{domain_from_pddl, problem_from_pddl};
use bcinr_pddl::{Pddl8GroundAction, Pddl8GroundAtom, Pddl8Tape};

use cng::bench::decomp::{
    check_interference, check_release_closure, decompose, decompose_with,
    replay_to_interface_state, DerivedEdges,
};
use cng::bench::ipc::{
    generate, generate_solvable, generate_variant, max_size, parse_surface, plan, IpcProblem,
    IpcVariant, IPC_DOMAINS,
};

fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/ipc-corpus")
        .join(test_name);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn negative_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("decomp-negative")
}

/// Parses one on-disk negative fixture pair (`<stem>.domain.pddl`,
/// `<stem>.problem.pddl`) through the unchanged bcinr parser.
fn negative_surface(stem: &str) -> (bcinr_pddl::Pddl8Domain, bcinr_pddl::Pddl8Problem) {
    let dir = negative_dir();
    let domain_text = fs::read_to_string(dir.join(format!("{stem}.domain.pddl")))
        .expect("negative domain fixture must be readable");
    let problem_text = fs::read_to_string(dir.join(format!("{stem}.problem.pddl")))
        .expect("negative problem fixture must be readable");
    let domain = domain_from_pddl(&domain_text).expect("negative domain fixture parses");
    let problem = problem_from_pddl(&problem_text).expect("negative problem fixture parses");
    (domain, problem)
}

/// Typed ground-atom constructor (no PDDL text).
fn ground_atom(pred: &str, args: &[&str]) -> Pddl8GroundAtom {
    Pddl8GroundAtom {
        pred: pred.to_string(),
        args: args.iter().map(|a| (*a).to_string()).collect(),
    }
}

/// Typed ground-action constructor (no PDDL text).
fn ground_action(
    label: &str,
    pre: Vec<Pddl8GroundAtom>,
    add: Vec<Pddl8GroundAtom>,
    del: Vec<Pddl8GroundAtom>,
) -> Pddl8GroundAction {
    let schema = match label.split('(').next() {
        Some(name) => name.to_string(),
        None => label.to_string(),
    };
    Pddl8GroundAction {
        schema_name: schema,
        label: label.to_string(),
        preconditions: pre,
        add_effects: add,
        del_effects: del,
    }
}

/// Empty derived-edge surface for typed-struct proof tests.
fn empty_edges() -> DerivedEdges {
    DerivedEdges {
        achievers: BTreeMap::new(),
        mutex: BTreeSet::new(),
        custody: BTreeSet::new(),
        must_precede: BTreeSet::new(),
        resource_atoms: BTreeSet::new(),
    }
}

/// Ordered action labels of a tape (plan-identity evidence).
fn tape_labels(tape: &Pddl8Tape) -> Vec<String> {
    tape.ops.iter().map(|op| op.action.label.clone()).collect()
}

/// Split-candidate ids (everything except `0-single`) from a decomposition
/// receipt list.
fn split_candidate_ids(problem: &IpcProblem, test_slug: &str) -> BTreeSet<String> {
    let (domain, parsed) = parse_surface(problem).expect("generated surface parses");
    let out = scratch_dir(test_slug);
    let result = decompose(
        &domain,
        &parsed,
        &out,
        &format!("urn:cng:test:ipc:{test_slug}"),
    )
    .expect("decompose yields a typed result");
    result
        .candidate_receipts
        .iter()
        .map(|r| r.candidate_id.clone())
        .filter(|id| id != "0-single")
        .collect()
}

// ---------------------------------------------------------------------------
// PROJ-711 — corpus law: solvable at the gated size, typed decomposition
// outcome, byte-identical regeneration.
// ---------------------------------------------------------------------------

test!(
    ipc_corpus_seeds_plan_decompose_and_regenerate_byte_identically,
    {
        // Arrange: seeds 0..3 per domain (the full 20-seed corpus is the
        // benchmark run, not this unit test).
        for domain_name in IPC_DOMAINS {
            for seed in 0..3u64 {
                // Act: size-backoff solvability gate + plan.
                let (problem, gated_size) =
                    generate_solvable(domain_name, seed, max_size(domain_name)?)?;
                let tape = plan(&problem)?;

                // Assert: a real plan exists at the gated size.
                assert!(
                    !tape.ops.is_empty(),
                    "{domain_name} seed {seed}: empty plan"
                );
                assert_eq!(problem.meta.size, gated_size);
                assert_eq!(problem.meta.domain, domain_name);
                assert_eq!(problem.meta.seed, seed);

                // Assert: same-seed regeneration is byte-identical.
                let again = generate(domain_name, seed, gated_size)?;
                assert_eq!(problem.domain_pddl, again.domain_pddl);
                assert_eq!(problem.problem_pddl, again.problem_pddl);
                assert_eq!(problem.meta, again.meta);

                // Assert: decompose returns one of the three TYPED outcomes
                // (never a refusal, never a silent fallback) for a solvable
                // corpus problem.
                let (parsed_domain, parsed_problem) = parse_surface(&problem)?;
                let out = scratch_dir(&format!("corpus-{domain_name}-{seed}"));
                let result = decompose(
                    &parsed_domain,
                    &parsed_problem,
                    &out,
                    &format!("urn:cng:test:ipc:{domain_name}:{seed}"),
                )?;
                assert!(
                    [
                        "Selected",
                        "NoAdmissibleDecomposition",
                        "NoBeneficialDecomposition"
                    ]
                    .contains(&result.outcome.as_str()),
                    "{domain_name} seed {seed}: unexpected outcome {:?}",
                    result.outcome
                );
                assert_eq!(result.candidate_receipts[0].candidate_id, "0-single");
            }
        }
    }
);

// ---------------------------------------------------------------------------
// PROJ-713 — anti-hardcoding gate.
// ---------------------------------------------------------------------------

test!(
    permuted_goal_identities_change_plans_and_receipts_causally,
    {
        for domain_name in IPC_DOMAINS {
            // Arrange: canonical problem at the gated size, plus its
            // guaranteed-distinct identity permutation.
            let (canonical, gated_size) =
                generate_solvable(domain_name, 0, max_size(domain_name)?)?;
            let permuted = generate_variant(
                domain_name,
                0,
                gated_size,
                IpcVariant::SwappedGoalIdentities,
            )?;

            // Assert: the permutation changed the problem text (guaranteed by
            // generator construction — never a coincidence of random draws).
            assert_ne!(
                canonical.problem_pddl, permuted.problem_pddl,
                "{domain_name}: identity permutation must change the problem"
            );

            // Act: both plans; the permutation must change the plan causally.
            let canonical_tape = plan(&canonical)?;
            let permuted_tape = plan(&permuted)?;
            assert_ne!(
                tape_labels(&canonical_tape),
                tape_labels(&permuted_tape),
                "{domain_name}: permuted identities must change the plan"
            );

            // Act: both decompositions under the SAME base IRI so only content
            // can differ; the emitted receipt graphs must differ causally.
            let base = format!("urn:cng:test:ipc:permute:{domain_name}");
            let (cd, cp) = parse_surface(&canonical)?;
            let (pd, pp) = parse_surface(&permuted)?;
            let canonical_result = decompose(
                &cd,
                &cp,
                &scratch_dir(&format!("permute-canon-{domain_name}")),
                &base,
            )?;
            let permuted_result = decompose(
                &pd,
                &pp,
                &scratch_dir(&format!("permute-swap-{domain_name}")),
                &base,
            )?;
            let canonical_bytes =
                fs::read(&canonical_result.result_graph_path).expect("read canonical result graph");
            let permuted_bytes =
                fs::read(&permuted_result.result_graph_path).expect("read permuted result graph");
            assert_ne!(
                canonical_bytes, permuted_bytes,
                "{domain_name}: permuted identities must change the receipt graph"
            );
        }
    }
);

test!(no_canned_helper_subgoal_across_incompatible_variants, {
    // Arrange/Act: split-candidate id sets per domain (seed 0, gated size).
    // Candidate ids are built from goal-atom labels, so a canned helper
    // subgoal reused across incompatible domains would show up as a
    // repeated id.
    let mut ids_by_domain: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for domain_name in IPC_DOMAINS {
        let (problem, _size) = generate_solvable(domain_name, 0, max_size(domain_name)?)?;
        ids_by_domain.insert(
            domain_name,
            split_candidate_ids(&problem, &format!("canned-{domain_name}")),
        );
    }

    // Assert: pairwise disjoint across incompatible domain variants.
    let domains: Vec<&str> = ids_by_domain.keys().cloned().collect();
    for i in 0..domains.len() {
        for j in (i + 1)..domains.len() {
            let a = &ids_by_domain[domains[i]];
            let b = &ids_by_domain[domains[j]];
            let shared: Vec<&String> = a.intersection(b).collect();
            assert!(
                shared.is_empty(),
                "canned helper subgoal shared between {} and {}: {shared:?}",
                domains[i],
                domains[j]
            );
        }
    }
});

// ---------------------------------------------------------------------------
// PROJ-712 — doctrine §13 negative corpus. Each named scenario refuses with
// its exact code.
// ---------------------------------------------------------------------------

test!(subgoal_not_contributing_refuses_cng_r21, {
    // Arrange: a solvable corpus problem; demand a candidate whose helper
    // subgoal contributes nothing (it was never enumerated from the goal).
    let (problem, _size) = generate_solvable("blocksworld", 0, max_size("blocksworld")?)?;
    let (domain, parsed) = parse_surface(&problem)?;
    let out = scratch_dir("subgoal-not-contributing");

    // Act.
    let refusal = decompose_with(
        &domain,
        &parsed,
        &out,
        "urn:cng:test:ipc:negative:subgoal-not-contributing",
        Some("not-a-contributing-subgoal"),
    )
    .unwrap_err();

    // Assert: typed CNG_R21 DecompositionInadmissible, never a fallback.
    assert_eq!(refusal.code(), "CNG_R21");
});

test!(helper_unreachable_refuses_cng_r04, {
    // Arrange: fixture whose delivered(c2) subgoal has no reachable
    // achiever (the pipeline plans candidate #0 first, so an unreachable
    // helper subgoal surfaces at the whole-problem gate).
    let (domain, problem) = negative_surface("helper-unreachable");
    let out = scratch_dir("helper-unreachable");

    // Act.
    let refusal = decompose(
        &domain,
        &problem,
        &out,
        "urn:cng:test:ipc:negative:helper-unreachable",
    )
    .unwrap_err();

    // Assert.
    assert_eq!(refusal.code(), "CNG_R04");
});

test!(main_unreachable_after_helper_refuses_cng_r23, {
    // Arrange (typed structs, no PDDL text): the helper consumed atom
    // `crane-free` that the main tape's first step still requires; the
    // verified replay of the main tape from s' must refuse at step 0.
    let crane_free = ground_atom("crane-free", &[]);
    let staged = ground_atom("staged", &["cargo1"]);
    let init: BTreeSet<Pddl8GroundAtom> = [crane_free.clone()].into_iter().collect();
    let helper_tape = Pddl8Tape::from_plan(vec![ground_action(
        "stage(cargo1)",
        vec![crane_free.clone()],
        vec![staged.clone()],
        vec![crane_free.clone()],
    )]);
    let main_tape = Pddl8Tape::from_plan(vec![ground_action(
        "lift(cargo2)",
        vec![crane_free.clone()],
        vec![ground_atom("lifted", &["cargo2"])],
        vec![],
    )]);

    // Act: s' from the helper replay, then main replay from s'.
    let s_prime = replay_to_interface_state(&init, &helper_tape)?;
    let refusal = replay_to_interface_state(&s_prime, &main_tape).unwrap_err();

    // Assert: typed CNG_R23 naming the failing step.
    assert_eq!(refusal.code(), "CNG_R23");
});

test!(helper_retains_resource_refuses_cng_r24, {
    // Arrange (typed structs): the helper acquired custody of pot1
    // (holding(pot1) true in s' but not in init, classified as a resource
    // atom) and no main-side precondition ever consumes it.
    let holding = ground_atom("holding", &["pot1"]);
    let init: BTreeSet<Pddl8GroundAtom> = BTreeSet::new();
    let s_prime: BTreeSet<Pddl8GroundAtom> = [holding.clone()].into_iter().collect();
    let helper_tape = Pddl8Tape::from_plan(vec![ground_action(
        "acquire(pot1)",
        vec![],
        vec![holding.clone()],
        vec![],
    )]);
    let main_tape = Pddl8Tape::from_plan(vec![ground_action(
        "slice(potato1)",
        vec![ground_atom("unsliced", &["potato1"])],
        vec![ground_atom("sliced", &["potato1"])],
        vec![ground_atom("unsliced", &["potato1"])],
    )]);
    let mut edges = empty_edges();
    edges.resource_atoms.insert(holding.label());

    // Act.
    let refusal =
        check_release_closure(&s_prime, &init, &helper_tape, &main_tape, &edges).unwrap_err();

    // Assert: typed CNG_R24 naming the retained resource.
    assert_eq!(refusal.code(), "CNG_R24");
});

test!(interfering_parallel_actions_refuse_cng_r22, {
    // Arrange (typed structs): a helper action deletes the main action's
    // protected precondition and no mustPrecede edge orders the pair.
    let stove_off = ground_atom("stovetop-off", &[]);
    let helper_tape = Pddl8Tape::from_plan(vec![ground_action(
        "ignite(stovetop1)",
        vec![stove_off.clone()],
        vec![ground_atom("stovetop-on", &[])],
        vec![stove_off.clone()],
    )]);
    let main_tape = Pddl8Tape::from_plan(vec![ground_action(
        "clean(stovetop1)",
        vec![stove_off.clone()],
        vec![ground_atom("clean", &["stovetop1"])],
        vec![],
    )]);
    let edges = empty_edges();

    // Act.
    let refusal = check_interference(&helper_tape, &main_tape, &edges).unwrap_err();

    // Assert: typed CNG_R22 naming the clobbering pair.
    assert_eq!(refusal.code(), "CNG_R22");
});

test!(actor_lacks_capability_refuses_cng_r05, {
    // Arrange: fixture whose only achiever demands an admitted capability
    // fact the actor does not carry, and no action ever grants it.
    let (domain, problem) = negative_surface("actor-lacks-capability");
    let out = scratch_dir("actor-lacks-capability");

    // Act.
    let refusal = decompose(
        &domain,
        &problem,
        &out,
        "urn:cng:test:ipc:negative:actor-lacks-capability",
    )
    .unwrap_err();

    // Assert: PROJ-733's relaxed-reachability grounder (bcinr-pddl) detects
    // the unreachable capability atom at GROUNDING time — the only schema
    // prunes to zero ground actions (`GroundError::EmptyGrounding`) — an
    // earlier, more precise catch of the same scenario than the naive
    // grounder's old behavior (ground everything regardless of reachability,
    // then fail the BFS search and refuse CNG_R04 PlanUnsolvable). Both are
    // typed, loud refusals for "actor lacks capability"; the pruned grounder
    // just proves it sooner, as CNG_R05 UnsupportedConstruct (grounding
    // failed) rather than CNG_R04.
    assert_eq!(refusal.code(), "CNG_R05");
});

test!(depth_or_cost_bound_exceeded_refuses_cng_r05, {
    // Arrange: fixture grounding to 8^5 = 32768 actions, above
    // DECOMP_MAX_GROUND = 16384.
    let (domain, problem) = negative_surface("bound-exceeded");
    let out = scratch_dir("bound-exceeded");

    // Act.
    let refusal = decompose(
        &domain,
        &problem,
        &out,
        "urn:cng:test:ipc:negative:bound-exceeded",
    )
    .unwrap_err();

    // Assert: the bound refusal is loud and typed, never an open-ended
    // search.
    assert_eq!(refusal.code(), "CNG_R05");
});

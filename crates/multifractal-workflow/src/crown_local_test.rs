//! End-to-end tests for the composed LOCAL crown-witness prefix (F02 -> F03 -> F08 -> F09 ->
//! F10), driven by [`super::drive_local_witness_prefix`] over a single real admitted observation
//! graph. Every fixture is real: a SHACL-backed admission policy, real STRIPS PDDL text, a real
//! F19 hook-pack catalog, and a real open recursive-socket closure -- no mocks, no test doubles.

use std::collections::{BTreeMap, BTreeSet};

use powl2_decompose::{ParentChildClosure, Powl, SocketKind, SocketPath, WorkflowSocketId};
use praxis_graphlaw::chatman::closure::{ClosureLaw, RecursiveSocketClosure};
use praxis_graphlaw::triples::{BodyLiteral, Rule, Triple};

use super::{drive_local_witness_prefix, LocalWitnessRefused, LocalWitnessRun};
use crate::f02_observation_admission::{AdmissionLedger, AdmissionPolicy, AdmissionState};
use crate::f03_semantic_contraction::ContractionState;
use crate::f05_datalog_closure::RulePack;
use crate::f08_pddl_planning::projector::{
    HOOK_PACK_PREDICATE, PDDL_DOMAIN_PREDICATE, PDDL_PROBLEM_PREDICATE,
};
use crate::f18_broker_law::ActionId;

// --------------------------------------------------------------------------
// Real fixture constants
// --------------------------------------------------------------------------

const SOURCE: &str = "crown-planner-1";
const PRINCIPAL: &str = "https://planners.example.org/crown-planner-1";
const SUBJECT: &str = "urn:mfw:crown:snapshot-1";

/// F02 re-admission (F19 -> F02) source identity: the local runtime itself, distinct from
/// `SOURCE` (the external planner) -- see `crown_local.rs`'s module doc F19->F02 nuance.
const ACTUATION_SOURCE: &str = "crown-local-runtime";
const ACTUATION_PRINCIPAL: &str = "urn:mfw:crown:local-runtime";

/// The exact STRIPS domain F08's own `run_pipeline` end-to-end test uses (proven to plan+bind on
/// F08's side) and F09's `resolve_continuation_goal` test parses (proven to parse on F09's side).
const DOMAIN_TEXT: &str = r#"
(define (domain mfw-crown-local)
  (:requirements :strips)
  (:predicates (at ?x) (goal-reached))
  (:action move
    :parameters (?x)
    :precondition (at ?x)
    :effect (and (goal-reached))))
"#;

const PROBLEM_TEXT: &str = r#"
(define (problem mfw-crown-local-problem)
  (:domain mfw-crown-local)
  (:objects a)
  (:init (at a))
  (:goal (and (goal-reached))))
"#;

/// Real F19-shaped hook pack covering the `move` action (same shape as F08's own fixture).
const HOOK_PACK: &str = r#"
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/crown#> .
ex:hook-move a kh:Hook ;
  kh:name "move-hook" ;
  kh:kind "delta" ;
  kh:var "http://example.org/crown#actuates-move" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:move> ;
  kh:reason "crown-local-authority" ;
  kh:priority 1 .
"#;

/// A hook pack that is valid Turtle and a real catalog, but covers the wrong action (`fly`, not
/// `move`) -- so F02 admits it but F08's Action-Hook Binder refuses (proving F08 really runs).
const WRONG_ACTION_HOOK_PACK: &str = r#"
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/crown#> .
ex:hook-fly a kh:Hook ;
  kh:name "fly-hook" ;
  kh:kind "delta" ;
  kh:var "http://example.org/crown#actuates-fly" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:fly> ;
  kh:reason "crown-local-authority" ;
  kh:priority 1 .
"#;

/// SHACL shapes that conform vacuously (a NodeShape whose target class no node carries) -- used
/// both as F02's policy shapes and F03's contraction shapes. Real shapes, real (empty) target
/// set, real conformance.
const VACUOUS_SHAPES: &str = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <urn:mfw:crown#> .
ex:PlanningSnapshotShape a sh:NodeShape ;
    sh:targetClass ex:AbsentClass .
"#;

// --------------------------------------------------------------------------
// Real fixture builders
// --------------------------------------------------------------------------

fn crown_policy() -> AdmissionPolicy {
    let mut known_principals = BTreeMap::new();
    known_principals.insert(SOURCE.to_string(), PRINCIPAL.to_string());
    known_principals.insert(
        ACTUATION_SOURCE.to_string(),
        ACTUATION_PRINCIPAL.to_string(),
    );

    let mut authorized = BTreeSet::new();
    authorized.insert(PDDL_DOMAIN_PREDICATE.to_string());
    authorized.insert(PDDL_PROBLEM_PREDICATE.to_string());
    authorized.insert(HOOK_PACK_PREDICATE.to_string());
    let mut authorized_predicates = BTreeMap::new();
    authorized_predicates.insert(SOURCE.to_string(), authorized);

    // The local runtime (re-admission principal) is authorized only for the F19->F02 actuation
    // predicates -- never the F08 planning predicates, which remain the external planner's alone.
    let mut actuation_authorized = BTreeSet::new();
    actuation_authorized.insert("urn:mfw:f19#actuatedHookName".to_string());
    actuation_authorized.insert("urn:mfw:f19#actuationReceiptHash".to_string());
    actuation_authorized.insert("urn:mfw:f18#brokerReceiptHash".to_string());
    authorized_predicates.insert(ACTUATION_SOURCE.to_string(), actuation_authorized);

    AdmissionPolicy::new(
        known_principals,
        authorized_predicates,
        vec![
            "urn:chatman:engine#".to_string(),
            "urn:mfw:f08#".to_string(),
            "urn:mfw:f19#".to_string(),
            "urn:mfw:f18#".to_string(),
        ],
        vec!["urn:".to_string(), "https://".to_string()],
        VACUOUS_SHAPES,
    )
    .expect("valid SHACL shapes in crown fixture policy")
}

/// A real, open recursive-socket closure over a 2-leaf `PartialOrder` root (nothing admitted, so
/// not already closed -- F09's `semantic_closure_check` passes).
fn open_growth_root_and_closure() -> (Powl, RecursiveSocketClosure) {
    let children = (0..2)
        .map(|i| Powl::Leaf(Some(format!("leaf-{i}"))))
        .collect();
    let root = Powl::PartialOrder {
        children,
        order: BTreeSet::new(),
    };
    let pcc = ParentChildClosure::from_model(&root);
    let socket = WorkflowSocketId {
        path: SocketPath::root(),
        kind: SocketKind::PartialOrder,
    };
    let closure = RecursiveSocketClosure::declare(&pcc, socket, ClosureLaw::AllRequired)
        .expect("declare open closure over 2 leaves");
    (root, closure)
}

/// A real, stratifiable, single-rule Datalog pack that fires only on `crown#Widget` typed nodes
/// (of which the admitted planning graph has none) -- so F03's closure is a genuine, non-vacuous
/// materialization pass that derives nothing here. Mirrors F03's own `widget_rule_pack` fixture.
/// (An *empty* pack is not usable: praxis-graphlaw's stratifier reports a spurious cycle for a
/// zero-rule ruleset, which F05's `close_datalog` faithfully surfaces as a refusal.)
fn harmless_rule_pack() -> RulePack {
    let rule = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/crown#derivedFlag".to_string(),
            "\"yes\"".to_string(),
        ),
        body: vec![BodyLiteral {
            pattern: Triple::from(
                "?x".to_string(),
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                "http://example.org/crown#Widget".to_string(),
            ),
            negated: false,
        }],
    };
    RulePack::new("crown-local-widget-pack", vec![rule])
}

fn base_run<'a>(
    policy: &'a AdmissionPolicy,
    ledger: &'a AdmissionLedger,
    root: Powl,
    closure: RecursiveSocketClosure,
    correlation_id: &str,
    hook_pack: &str,
) -> LocalWitnessRun<'a> {
    LocalWitnessRun {
        policy,
        ledger,
        correlation_id: correlation_id.to_string(),
        source_id: SOURCE.to_string(),
        subject_iri: SUBJECT.to_string(),
        source_principal_iri: PRINCIPAL.to_string(),
        pddl_domain: DOMAIN_TEXT.to_string(),
        pddl_problem: PROBLEM_TEXT.to_string(),
        hook_pack_turtle: hook_pack.to_string(),
        datalog_rule_pack: harmless_rule_pack(),
        f03_shacl_shapes: VACUOUS_SHAPES.to_string(),
        growth_root: root,
        growth_closure: closure,
        socket_blocked: true,
        descent_budget: 4,
        closure_law: ClosureLaw::AllRequired,
        broker_secret: [7u8; 32],
        action: ActionId::new(SUBJECT, "move", correlation_id),
        actor: "crown-local-test-actor".to_string(),
        has_standing: true,
        standing_reason: "crown-local fixture: caller-asserted standing".to_string(),
        local_run_id: [9u8; 32],
        local_max_ticks: 16,
        actuation_source_id: ACTUATION_SOURCE.to_string(),
        actuation_principal_iri: ACTUATION_PRINCIPAL.to_string(),
    }
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

/// The load-bearing test: one admitted observation graph drives F02 -> F03 -> F08 -> F09 -> F10
/// -> F11 -> F18 -> F19 -> F02(re-admit) end to end, and every stage's real output is present
/// and correct.
#[test]
fn crown_local_prefix_drives_f02_through_f02_readmit_end_to_end() {
    let policy = crown_policy();
    let ledger = AdmissionLedger::new();
    let (root, closure) = open_growth_root_and_closure();
    let run = base_run(&policy, &ledger, root, closure, "crown-corr-1", HOOK_PACK);

    let outcome = drive_local_witness_prefix(run)
        .expect("full LOCAL crown prefix must compose end to end on a real fixture");

    // --- F02: the observation graph was really admitted ---
    assert_eq!(outcome.admission.state, AdmissionState::Admitted);
    assert_eq!(outcome.admission.correlation_id, "crown-corr-1");
    assert!(!outcome.admission.receipt_hash.is_empty());

    // --- F03: the admitted world contracted to a Plannable planning state ---
    assert_eq!(outcome.planning_state.state, ContractionState::Plannable);

    // --- F08: a real plan reached the goal, over the admitted graph ---
    assert!(
        outcome.plan.receipt.goal_reached,
        "F08 must reach the goal for the admitted STRIPS problem"
    );
    assert_eq!(
        outcome.plan.tape.ops.len(),
        1,
        "the one-hop move problem plans to a single op"
    );
    assert!(outcome
        .plan
        .capability_map
        .iri
        .starts_with("urn:mfw:f08:action_capability_map:"));

    // --- F09: a child was manufactured and grafted under the blocked socket ---
    match &outcome.growth.new_root {
        Powl::PartialOrder { children, .. } => assert_eq!(
            children.len(),
            3,
            "2 original leaves + 1 manufactured child"
        ),
        other => panic!("F09 new_root must be a PartialOrder, got {other:?}"),
    }

    // --- F10: real geometry was produced (inside F09) and passes its shape report ---
    assert!(
        outcome.growth.geometry_shape.leaves >= 1,
        "F10 geometry must have at least one leaf socket"
    );
    assert!(
        outcome.growth.geometry_turtle.contains("truex.io"),
        "F10 geometry Turtle must be real serialized output under F09's growth base IRI"
    );

    // --- F11 -> F18: real local execution actually ran and was really broker-dispatched ---
    assert_eq!(outcome.broker_receipt.correlation_id, "crown-corr-1");
    assert!(
        !outcome.broker_receipt.consequence_hash_hex.is_empty(),
        "F18's captured consequence must be F11's real Local Receipt chain hash, not empty"
    );
    assert!(!outcome.broker_receipt.receipt_hash_hex.is_empty());
    assert!(!outcome.broker_receipt.authority_token_hex.is_empty());

    // --- F18 -> F19: the actuated action really resolved to exactly one registered hook ---
    assert_eq!(
        outcome.hook_resolution.state,
        crate::f19_hooks::HookResolutionState::Replayable
    );
    assert_eq!(outcome.hook_resolution.binding.hook_name, "move-hook");
    assert_eq!(
        outcome.hook_resolution.declared_authority,
        "crown-local-authority"
    );
    assert!(!outcome.hook_resolution.receipt_hash.is_empty());

    // --- F19 -> F02 (re-admit): the actuation consequence was really re-admitted through the
    //     same F02 gate pipeline, under the distinct local-runtime principal, as a new ledger
    //     entry (not colliding with or replaying the original planning admission) ---
    assert_eq!(outcome.actuation_admission.state, AdmissionState::Admitted);
    assert_eq!(
        outcome.actuation_admission.correlation_id,
        "crown-corr-1-actuation"
    );
    assert_eq!(outcome.actuation_admission.source_id, ACTUATION_SOURCE);
    assert_ne!(
        outcome.actuation_admission.receipt_hash, outcome.admission.receipt_hash,
        "the re-admitted actuation consequence must be a distinct admission, not a replay of the \
         original planning observation"
    );
    assert!(!outcome.actuation_admission.receipt_hash.is_empty());

    // --- Crown receipt is a real 64-hex BLAKE3 fold over every stage's digest ---
    assert_eq!(outcome.crown_receipt.len(), 64);
    assert!(outcome.crown_receipt.chars().all(|c| c.is_ascii_hexdigit()));

    // --- F08 <-> F09 shared-problem edge: both planned the same admitted problem to the same
    //     number of ops (F10's geometry is built from F09's plan tape) ---
    assert_eq!(
        outcome.growth.geometry.source_action_count,
        outcome.plan.tape.ops.len(),
        "F09's plan (feeding F10) and F08's plan must agree on op count for the shared problem"
    );
}

/// Same inputs -> byte-identical crown receipt (repo invariant #5: deterministic under fixed
/// inputs, no wall clock, no randomness).
#[test]
fn crown_local_prefix_is_deterministic() {
    let policy = crown_policy();

    let ledger_a = AdmissionLedger::new();
    let (root_a, closure_a) = open_growth_root_and_closure();
    let run_a = base_run(
        &policy,
        &ledger_a,
        root_a,
        closure_a,
        "crown-det",
        HOOK_PACK,
    );
    let out_a = drive_local_witness_prefix(run_a).expect("first run");

    let ledger_b = AdmissionLedger::new();
    let (root_b, closure_b) = open_growth_root_and_closure();
    let run_b = base_run(
        &policy,
        &ledger_b,
        root_b,
        closure_b,
        "crown-det",
        HOOK_PACK,
    );
    let out_b = drive_local_witness_prefix(run_b).expect("second run");

    assert_eq!(
        out_a.crown_receipt, out_b.crown_receipt,
        "same inputs must produce a byte-identical crown receipt"
    );
    assert_eq!(out_a.admission.receipt_hash, out_b.admission.receipt_hash);
    assert_eq!(
        out_a.planning_state.receipt_head,
        out_b.planning_state.receipt_head
    );
}

/// F02 short-circuits the whole prefix: an unknown source is refused at admission, and no
/// downstream stage runs (the returned refusal is the F02 one, proving F03/F08/F09 never ran).
#[test]
fn crown_local_prefix_short_circuits_when_f02_refuses() {
    let policy = crown_policy();
    let ledger = AdmissionLedger::new();
    let (root, closure) = open_growth_root_and_closure();
    let mut run = base_run(
        &policy,
        &ledger,
        root,
        closure,
        "crown-bad-source",
        HOOK_PACK,
    );
    run.source_id = "not-a-registered-source".to_string();

    let err = drive_local_witness_prefix(run)
        .expect_err("an unknown source must refuse at F02, before any downstream stage");
    assert!(
        matches!(err, LocalWitnessRefused::Admission(_)),
        "expected an F02 admission refusal, got {err:?}"
    );
    // Nothing was admitted into the ledger.
    assert_eq!(ledger.len().expect("ledger evaluable"), 0);
}

/// F08 really runs and really gates: with a hook pack that covers the wrong action, F02 admits
/// (the pack is valid) and F03 contracts, but F08's Action-Hook Binder refuses -- so F09 never
/// runs. Proves the F03 -> F08 edge is a real, gating call, not a decorative pass-through.
#[test]
fn crown_local_prefix_refuses_at_f08_when_no_hook_covers_the_action() {
    let policy = crown_policy();
    let ledger = AdmissionLedger::new();
    let (root, closure) = open_growth_root_and_closure();
    let run = base_run(
        &policy,
        &ledger,
        root,
        closure,
        "crown-wrong-hook",
        WRONG_ACTION_HOOK_PACK,
    );

    let err = drive_local_witness_prefix(run)
        .expect_err("an uncovered action must refuse at F08's Action-Hook Binder");
    assert!(
        matches!(err, LocalWitnessRefused::Planning(_)),
        "expected an F08 planning refusal, got {err:?}"
    );
    // F02 still admitted the observation (the refusal is strictly downstream of admission).
    assert_eq!(ledger.len().expect("ledger evaluable"), 1);
}

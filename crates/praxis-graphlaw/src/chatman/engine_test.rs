#![cfg(test)]

//! Chicago-style state-based tests over a real in-memory store: one
//! happy path, one refusal per stage, determinism (byte-identical
//! receipt roots across independent engines), actuation, and replay.

use super::*;
use crate::chatman::abi::{InputHandles, InvocationId, OperatorId, ProfileId};

const SNAPSHOT_IRI: &str = "urn:chatman:snapshot:test";
const PROFILE_IRI: &str = "profile:engine-test";

/// Snapshot fixture: a tiny RDFS hierarchy (so OWL RL derives a fact),
/// a one-step PDDL world, and a conforming one-event OCEL trace.
const SNAPSHOT_TTL: &str = r#"
@prefix ex: <http://example.org/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ceng: <urn:chatman:engine#> .

ex:Employee rdfs:subClassOf ex:Person .
ex:alice a ex:Employee .

ex:world ceng:pddlDomain """
(define (domain chatman-min)
  (:requirements :strips)
  (:predicates (ready ?x) (done ?x))
  (:action finish
    :parameters (?x)
    :precondition (and (ready ?x))
    :effect (and (done ?x) (not (ready ?x)))))
""" .
ex:world ceng:pddlProblem """
(define (problem chatman-min-p)
  (:domain chatman-min)
  (:objects a)
  (:init (ready a))
  (:goal (done a))
)
""" .
ex:world ceng:ocelLog """{"run_id":1,"sealed":true,"objects":[{"id":"case-1","otype":"case"}],"events":[{"id":"e1","activity":"finish(a)","op_index":0,"at_ns":1,"objects":["case-1"]}]}""" .
"#;

/// Same world, but the trace fires op 0 twice (DuplicateFire).
const SNAPSHOT_TTL_DUPLICATE_FIRE: &str = r#"
@prefix ex: <http://example.org/> .
@prefix ceng: <urn:chatman:engine#> .
ex:world ceng:pddlDomain """
(define (domain chatman-min)
  (:requirements :strips)
  (:predicates (ready ?x) (done ?x))
  (:action finish
    :parameters (?x)
    :precondition (and (ready ?x))
    :effect (and (done ?x) (not (ready ?x)))))
""" .
ex:world ceng:pddlProblem """
(define (problem chatman-min-p)
  (:domain chatman-min)
  (:objects a)
  (:init (ready a))
  (:goal (done a))
)
""" .
ex:world ceng:ocelLog """{"run_id":1,"sealed":true,"objects":[{"id":"case-1","otype":"case"}],"events":[{"id":"e1","activity":"finish(a)","op_index":0,"at_ns":1,"objects":["case-1"]},{"id":"e2","activity":"finish(a)","op_index":0,"at_ns":2,"objects":["case-1"]}]}""" .
"#;

/// Same world, but the goal is unreachable (no action derives it).
const SNAPSHOT_TTL_INFEASIBLE: &str = r#"
@prefix ex: <http://example.org/> .
@prefix ceng: <urn:chatman:engine#> .
ex:world ceng:pddlDomain """
(define (domain chatman-min)
  (:requirements :strips)
  (:predicates (ready ?x) (done ?x) (never ?x))
  (:action finish
    :parameters (?x)
    :precondition (and (ready ?x))
    :effect (and (done ?x) (not (ready ?x)))))
""" .
ex:world ceng:pddlProblem """
(define (problem chatman-min-p)
  (:domain chatman-min)
  (:objects a)
  (:init (ready a))
  (:goal (never a))
)
""" .
ex:world ceng:ocelLog """{"run_id":1,"sealed":true,"objects":[{"id":"case-1","otype":"case"}],"events":[{"id":"e1","activity":"finish(a)","op_index":0,"at_ns":1,"objects":["case-1"]}]}""" .
"#;

fn test_profile() -> Result<EngineProfile, Refusal> {
    let profile_id = ProfileId::new(PROFILE_IRI);
    let gates = ProfileGates::new(profile_id.clone(), ProfileGates::DEFAULT_ENABLED_MASK, 0, 8)?;
    let symbol_table = ProfileSymbolTable::build(
        profile_id,
        vec![
            "<urn:chatman:t0>".to_string(),
            "<urn:chatman:t1>".to_string(),
        ],
    )?;
    Ok(EngineProfile {
        gates,
        symbol_table,
        admission: AdmissionSpec {
            constraint_names: vec!["c0".to_string()],
            required_mask: 0,
            forbidden_mask: 0,
            set_on_admit: 0,
            clear_on_admit: 0,
        },
        breed_permits: Vec::new(),
    })
}

fn envelope() -> InvocationEnvelope {
    InvocationEnvelope {
        invocation_id: InvocationId::new("inv-1"),
        snapshot_id: GraphSnapshotId::new(SNAPSHOT_IRI),
        profile_id: ProfileId::new(PROFILE_IRI),
        operator_id: OperatorId::new("op-1"),
        input_handles: InputHandles::default(),
    }
}

fn engine_with(turtle: &str) -> Result<ChatmanEngine, Refusal> {
    let mut engine = ChatmanEngine::in_memory(test_profile()?)?;
    engine.load_snapshot(&GraphSnapshotId::new(SNAPSHOT_IRI), turtle)?;
    Ok(engine)
}

fn admit(turtle: &str) -> Result<AdmittedTransition, Refusal> {
    let mut engine = engine_with(turtle)?;
    engine.admit_transition(envelope())
}

fn refusal_check(result: Result<AdmittedTransition, Refusal>, want: &str) -> Result<(), Refusal> {
    match result {
        Err(refusal) if refusal.name() == want => Ok(()),
        other => Err(Refusal::ValidationFailed(format!(
            "wanted refusal {want}, got {other:?}"
        ))),
    }
}

#[test]
fn happy_path_admits_and_seals_nine_digests() -> Result<(), Refusal> {
    // Arrange + Act
    let transition = admit(SNAPSHOT_TTL)?;
    let receipt = transition.receipt();

    // Assert: every digest is non-empty 64-hex material and the root
    // recomputes over the nine carried digests.
    for digest in [
        &receipt.graph_snapshot,
        &receipt.profile,
        &receipt.symbol_table,
        &receipt.projection,
        &receipt.admission_table,
        &receipt.route_decision,
        &receipt.tape,
        &receipt.hook_event,
        &receipt.engine_version,
    ] {
        assert_eq!(digest.0.len(), 64, "digest must be 64 hex chars");
    }
    assert_eq!(receipt.recompute_root(), receipt.receipt_root);
    // Canonical material is sorted (receipt law).
    let lines: Vec<&str> = receipt.canon_nquads.lines().collect();
    let mut sorted = lines.clone();
    sorted.sort_unstable();
    assert_eq!(lines, sorted, "canonical N-Quads must be sorted");
    assert!(!receipt.canon_nquads.is_empty());
    // The hookless fixture yields no boundary requests.
    assert!(transition.boundary_requests().is_empty());
    Ok(())
}

#[test]
fn owl_closure_lands_in_sibling_graph_not_snapshot() -> Result<(), Refusal> {
    // Arrange
    let mut engine = engine_with(SNAPSHOT_TTL)?;
    let (_, before) = engine.fetch_snapshot(&GraphSnapshotId::new(SNAPSHOT_IRI))?;

    // Act
    let transition = engine.admit_transition(envelope())?;

    // Assert: the derived alice-is-a-Person fact exists in the closure
    // graph and the snapshot graph is byte-identical to before.
    let closure = NamedNode::new(format!("{SNAPSHOT_IRI}#closure"))
        .map_err(|e| Refusal::ValidationFailed(format!("closure IRI: {e}")))?;
    let mut derived_lines = Vec::new();
    for quad in engine
        .store
        .quads_for_pattern(None, None, None, Some(closure.as_ref().into()))
    {
        let quad = quad.map_err(|e| Refusal::ValidationFailed(format!("storage: {e}")))?;
        derived_lines.push(format!(
            "{} {} {}",
            quad.subject, quad.predicate, quad.object
        ));
    }
    assert!(
        derived_lines
            .iter()
            .any(|l| l.contains("alice") && l.contains("Person")),
        "OWL RL closure must derive alice rdf:type Person, got {derived_lines:?}"
    );
    let (_, after) = engine.fetch_snapshot(&GraphSnapshotId::new(SNAPSHOT_IRI))?;
    assert_eq!(before, after, "the input snapshot graph is immutable");
    assert_eq!(
        transition.receipt().recompute_root(),
        transition.receipt().receipt_root
    );
    Ok(())
}

#[test]
fn double_run_is_byte_identical() -> Result<(), Refusal> {
    // Arrange + Act: two independent engines over the same snapshot.
    let first = admit(SNAPSHOT_TTL)?;
    let second = admit(SNAPSHOT_TTL)?;

    // Assert: receipts agree byte for byte.
    assert_eq!(first.receipt(), second.receipt());
    assert_eq!(
        first.receipt().receipt_root.0,
        second.receipt().receipt_root.0
    );
    Ok(())
}

/// Gate F determinism, extended for PROJ-SEC-01: five independent engines
/// over the same snapshot must produce byte-identical end-of-run receipts
/// (the existing check, carried over from `double_run_is_byte_identical`
/// above but run 5x instead of 2x to match the "5 consecutive runs" bar
/// documented in `.claude/rules/rust-agi-core-team.md` §1) *and*
/// byte-identical PROJ-SEC-01 per-transition seals at every S1→S2, S2→S3,
/// S3→S4, and S4→S5 boundary. `double_run_is_byte_identical` above is left
/// as-is (its name predates this ticket); this test is the 5-run
/// extension the ticket asked for, added alongside rather than in place of
/// the original.
#[test]
fn five_consecutive_runs_receipts_and_stage_seals_are_byte_identical() -> Result<(), Refusal> {
    let mut receipts = Vec::with_capacity(5);
    let mut seal_runs = Vec::with_capacity(5);
    // O(1) fixed iteration count (5), each iteration bounded by one S1-S6
    // admission; not a hot-path loop.
    for _ in 0..5 {
        let mut engine = engine_with(SNAPSHOT_TTL)?;
        let stages = engine.run_stages(&envelope())?;
        seal_runs.push(stages.stage_seals);
        let transition = engine.admit_transition(envelope())?;
        receipts.push(transition.receipt().clone());
    }
    for r in &receipts[1..] {
        assert_eq!(
            &receipts[0], r,
            "receipt must be byte-identical across runs"
        );
    }
    for s in &seal_runs[1..] {
        assert_eq!(
            &seal_runs[0], s,
            "PROJ-SEC-01 stage seals must be byte-identical across runs"
        );
    }
    Ok(())
}

/// PROJ-SEC-01 negative test: a stage entered with a seal that does not
/// verify against the digest it is paired with must refuse with
/// [`Refusal::StageSealMismatch`], not silently proceed. This calls S2
/// (`apply_owl_closure`) directly, bypassing `run_stages`'s honest wiring,
/// with a seal computed over a tampered digest — proving the check is a
/// real recompute-and-compare seam, not a no-op that always agrees with
/// itself.
#[test]
fn stage_seal_mismatch_refuses_tampered_transition() -> Result<(), Refusal> {
    let mut engine = engine_with(SNAPSHOT_TTL)?;
    let snapshot_id = GraphSnapshotId::new(SNAPSHOT_IRI);
    let (_, real_graph_snapshot) = engine.fetch_snapshot(&snapshot_id)?;

    // A seal computed over a *different* digest than the one about to be
    // passed as `prior_digest` — simulating corruption between S1 and S2.
    let tampered_digest = Digest::new("tampered-not-what-s1-produced".to_string());
    let seal_over_tampered = StageSeal::of("S1", &tampered_digest);

    let result = engine.apply_owl_closure(&snapshot_id, &seal_over_tampered, &real_graph_snapshot);
    match result {
        Err(Refusal::StageSealMismatch(_)) => Ok(()),
        other => Err(Refusal::ValidationFailed(format!(
            "wanted Err(StageSealMismatch), got {other:?}"
        ))),
    }
}

/// Companion negative test: even a seal that verifies against the *real*
/// digest tag from the wrong stage name must refuse (seals are stage-name
/// tagged, so an S2 seal cannot be replayed as an S1 seal for the same
/// digest bytes).
#[test]
fn stage_seal_wrong_stage_name_refuses() -> Result<(), Refusal> {
    let mut engine = engine_with(SNAPSHOT_TTL)?;
    let snapshot_id = GraphSnapshotId::new(SNAPSHOT_IRI);
    let (_, real_graph_snapshot) = engine.fetch_snapshot(&snapshot_id)?;

    // Seal computed under the wrong stage tag ("S2" instead of "S1") over
    // the *real* digest — still must not verify at S2's entry, which
    // checks against "S1".
    let wrongly_tagged_seal = StageSeal::of("S2", &real_graph_snapshot);

    let result = engine.apply_owl_closure(&snapshot_id, &wrongly_tagged_seal, &real_graph_snapshot);
    match result {
        Err(Refusal::StageSealMismatch(_)) => Ok(()),
        other => Err(Refusal::ValidationFailed(format!(
            "wanted Err(StageSealMismatch), got {other:?}"
        ))),
    }
}

#[test]
fn s1_unknown_snapshot_refuses_snapshot_not_found() -> Result<(), Refusal> {
    // Arrange: engine with no snapshot loaded.
    let mut engine = ChatmanEngine::in_memory(test_profile()?)?;
    // Act
    let result = engine.admit_transition(envelope());
    // Assert
    refusal_check(result, "SnapshotNotFound")
}

#[test]
fn s1_quoted_triple_text_refuses_triple_term() -> Result<(), Refusal> {
    // Arrange
    let mut engine = ChatmanEngine::in_memory(test_profile()?)?;
    // Act: RDF 1.2 quoted-triple syntax at the load boundary.
    let result = engine.load_snapshot(
        &GraphSnapshotId::new(SNAPSHOT_IRI),
        "<< <urn:s> <urn:p> <urn:o> >> <urn:q> <urn:r> .",
    );
    // Assert
    match result {
        Err(Refusal::TripleTermInSnapshot(_)) => Ok(()),
        other => Err(Refusal::ValidationFailed(format!(
            "wanted TripleTermInSnapshot, got {other:?}"
        ))),
    }
}

#[test]
fn s2_owl_disabled_refuses_via_router() -> Result<(), Refusal> {
    // Arrange: profile whose gates never enable OwlRl.
    let profile_id = ProfileId::new(PROFILE_IRI);
    let hot_only = ProfileGates::new(
        profile_id.clone(),
        crate::chatman::router::Dialect::Triple8Pattern.mask_bit(),
        0,
        8,
    )?;
    let base = test_profile()?;
    let profile = EngineProfile {
        gates: hot_only,
        symbol_table: base.symbol_table,
        admission: base.admission,
        breed_permits: base.breed_permits,
    };
    let mut engine = ChatmanEngine::in_memory(profile)?;
    engine.load_snapshot(&GraphSnapshotId::new(SNAPSHOT_IRI), SNAPSHOT_TTL)?;
    // Act
    let result = engine.admit_transition(envelope());
    // Assert: the OWL RL shape has no enabled dialect >= its floor.
    refusal_check(result, "UnsupportedDialect")
}

#[test]
fn s3_missing_pddl_refuses_plan_infeasible() -> Result<(), Refusal> {
    // Arrange: snapshot with graph data but no PDDL literals.
    let result = admit(
        r#"@prefix ex: <http://example.org/> .
ex:a ex:knows ex:b ."#,
    );
    // Assert
    refusal_check(result, "PlanInfeasible")
}

#[test]
fn s3_unreachable_goal_refuses_plan_infeasible() -> Result<(), Refusal> {
    refusal_check(admit(SNAPSHOT_TTL_INFEASIBLE), "PlanInfeasible")
}

#[test]
fn s4_duplicate_fire_refuses_trace_unlawful() -> Result<(), Refusal> {
    refusal_check(admit(SNAPSHOT_TTL_DUPLICATE_FIRE), "TraceUnlawful")
}

#[test]
fn s4_missing_trace_refuses_trace_unlawful() -> Result<(), Refusal> {
    // Arrange: PDDL present, OCEL literal absent.
    let ttl = r#"
@prefix ex: <http://example.org/> .
@prefix ceng: <urn:chatman:engine#> .
ex:world ceng:pddlDomain """
(define (domain chatman-min)
  (:requirements :strips)
  (:predicates (ready ?x) (done ?x))
  (:action finish
    :parameters (?x)
    :precondition (and (ready ?x))
    :effect (and (done ?x) (not (ready ?x)))))
""" .
ex:world ceng:pddlProblem """
(define (problem chatman-min-p)
  (:domain chatman-min)
  (:objects a)
  (:init (ready a))
  (:goal (done a))
)
""" .
"#;
    refusal_check(admit(ttl), "TraceUnlawful")
}

#[test]
fn envelope_naming_wrong_profile_refuses_profile_hash_mismatch() -> Result<(), Refusal> {
    // Arrange
    let mut engine = engine_with(SNAPSHOT_TTL)?;
    let mut env = envelope();
    env.profile_id = ProfileId::new("profile:someone-else");
    // Act + Assert
    refusal_check(engine.admit_transition(env), "ProfileHashMismatch")
}

#[test]
fn actuate_registers_post_graph_and_dedups() -> Result<(), Refusal> {
    // Arrange
    let mut engine = engine_with(SNAPSHOT_TTL)?;
    let transition = engine.admit_transition(envelope())?;
    let root = transition.receipt().receipt_root.0.clone();

    // Act
    let record = engine.actuate(transition)?;

    // Assert: the post graph is named by the receipt root and exists.
    assert!(record.post_graph.contains(&root));
    assert_eq!(record.duplicates_skipped, 0);
    let post = NamedNode::new(&record.post_graph)
        .map_err(|e| Refusal::ValidationFailed(format!("post IRI: {e}")))?;
    let present = engine
        .store
        .contains_named_graph(post.as_ref())
        .map_err(|e| Refusal::ValidationFailed(format!("storage: {e}")))?;
    assert!(present, "actuation must register the post graph");
    Ok(())
}

#[test]
fn verify_replay_accepts_faithful_and_refuses_tampered() -> Result<(), Refusal> {
    // Arrange
    let transition = admit(SNAPSHOT_TTL)?;
    let inputs = ReplayInputs {
        envelope: envelope(),
        snapshot_turtle: SNAPSHOT_TTL.to_string(),
        profile: test_profile()?,
    };

    // Act + Assert: the faithful receipt replays clean.
    match ChatmanEngine::verify_replay(transition.receipt(), &inputs) {
        Ok(()) => {}
        Err(mismatch) => {
            return Err(Refusal::ValidationFailed(format!(
                "faithful replay must verify, got {mismatch}"
            )))
        }
    }

    // Tamper with digest #7: the fail-fast enum names the tape field.
    let mut tampered = transition.receipt().clone();
    tampered.tape = Digest::new("0".repeat(64));
    match ChatmanEngine::verify_replay(&tampered, &inputs) {
        Err(ReplayMismatch::Tape { .. }) => {}
        other => {
            return Err(Refusal::ValidationFailed(format!(
                "tampered tape digest must fail as ReplayMismatch::Tape, got {other:?}"
            )))
        }
    }

    // Tamper with the root only: every field matches, the root recompute
    // catches the drift.
    let mut root_tampered = transition.receipt().clone();
    root_tampered.receipt_root = Digest::new("f".repeat(64));
    match ChatmanEngine::verify_replay(&root_tampered, &inputs) {
        Err(ReplayMismatch::ReceiptRoot { .. }) => Ok(()),
        other => Err(Refusal::ValidationFailed(format!(
            "tampered root must fail as ReplayMismatch::ReceiptRoot, got {other:?}"
        ))),
    }
}

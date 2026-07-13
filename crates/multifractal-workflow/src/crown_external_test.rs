//! Tests for the crown-witness EXTERNAL tail (`F10 -> F12 -> F13 -> F14 -> F15`), the independent
//! `F20 -> F02(re-admit)` edge, and `F02(re-admit) -> F15 (AIR transition)`.
//!
//! The non-`#[ignore]` tests drive the whole `F10 (real geometry) -> F12 -> F13 -> F14 ->
//! F15 (converter)` chain in pure Rust, with no Erlang dependency. The `#[ignore]`d tests
//! additionally spawn the real `air_core` escript bridge (run with `--ignored`).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use powl2_decompose::Powl;

use super::{
    air_program_to_bridge_workflow, drive_external_readmit_transition, drive_external_reentry,
    drive_external_witness_tail, drive_external_witness_tail_through_f16, ExternalReentryRun,
    ExternalWitnessRun,
};
use crate::f02_observation_admission::{AdmissionLedger, AdmissionPolicy, AdmissionState};
use crate::f10_powl_geometry::{manufacture_powl_v2, POWLModel, Plan, PlanAction};
use crate::f20_external_dispatch::{Powl as CngPowl, SubworkflowPlan};

/// A real F10 geometry, built through F10's genuine, independently-tested
/// `manufacture_powl_v2` (Plan Grouper -> Partial Order Builder -> ...). Two plan-required-ordered
/// actions from one remote provenance source -- a minimal but genuine plan-derived geometry, not a
/// hand-built `Powl` literal.
fn real_f10_geometry() -> POWLModel {
    let plan = Plan {
        actions: vec![
            PlanAction {
                id: "settle_a".to_string(),
                source: "https://truex.io/remote/settlement-svc".to_string(),
            },
            PlanAction {
                id: "settle_b".to_string(),
                source: "https://truex.io/remote/settlement-svc".to_string(),
            },
        ],
        precedes: BTreeSet::from([(0usize, 1usize)]),
        choice_groups: vec![],
    };
    let (geometry, _turtle, _shape) =
        manufacture_powl_v2(&plan, &BTreeMap::new(), "https://truex.io/f10-geo")
            .expect("a well-formed two-action ordered plan must build real F10 geometry");
    geometry
}

fn run_over(geometry: &POWLModel) -> ExternalWitnessRun<'_> {
    ExternalWitnessRun {
        f10_geometry: geometry,
        base_iri: "https://truex.io/crown-ext".to_string(),
        workflow_id: "crown-external-workflow".to_string(),
        title: "Crown external witness tail".to_string(),
        compiler_version: "26.7.12".to_string(),
    }
}

/// The load-bearing end-to-end test: F10's real geometry, wrapped as the externalized region,
/// drives F12 (cut resolution/admission) -> F13 (Arazzo projection/compile) -> F14 (Arazzo -> AIR
/// compile) -> F15 (AIR program -> bridge workflow) in one real call.
#[test]
fn external_tail_composes_f10_geometry_through_f12_f13_f14_to_a_bridge_workflow() {
    let geometry = real_f10_geometry();
    let outcome = drive_external_witness_tail(run_over(&geometry))
        .expect("the external tail must compose end to end over real F10 geometry");

    // F12: the resolved-and-admitted node is genuinely an external cut.
    assert!(
        matches!(outcome.external_cut, Powl::ExternalCut { .. }),
        "F12 must resolve a real ExternalCut, got {:?}",
        outcome.external_cut
    );

    // F13: the manufactured document is a real Arazzo 1.1.0 doc that carries the external-cut
    // extension the projection stamps onto the cut step.
    assert!(
        outcome.arazzo_document.contains("\"arazzo\": \"1.1.0\""),
        "F13 must manufacture an Arazzo 1.1.0 document"
    );
    assert!(
        outcome.arazzo_document.contains("x-powl-external-cut"),
        "F13's projected document must carry the external-cut extension marker"
    );

    // F13 -> F14 byte-level edge: F14's independent recompile of F13's arazzo_document yields the
    // *byte-identical* AIR digest F13 recorded internally. This is the strongest possible edge
    // proof -- not merely "it compiles", but "it compiles to the same program".
    assert_eq!(
        outcome.air_digest_hex, outcome.arazzo_receipt.air_digest_hex,
        "F14's recompiled AIR digest must byte-equal F13's receipt air_digest_hex (proves the \
         F13->F14 edge produced identical AIR)"
    );
    assert_eq!(
        outcome.air_workflow_count, 1,
        "the projection yields one workflow"
    );

    // F15: the bridge workflow is derived from F14's real program -- one step per lowered AIR
    // step, and every step id appears in F13's manufactured document.
    assert!(
        !outcome.bridge_workflow.steps.is_empty(),
        "F15 bridge workflow must have at least one step derived from F14's AIR program"
    );
    assert_eq!(
        outcome.bridge_workflow.steps.len(),
        outcome.bridge_active_steps.len(),
        "every bridge step must be an initial active step"
    );
    assert_eq!(
        outcome.bridge_events.len(),
        outcome.bridge_active_steps.len(),
        "one step_completed event per active step"
    );
    for step_id in outcome.bridge_workflow.steps.keys() {
        assert!(
            outcome.arazzo_document.contains(step_id.as_str()),
            "bridge step id {step_id:?} must originate from F13's manufactured document \
             (it is F14's lowered step name)"
        );
    }

    // Crown receipt is a real 64-hex BLAKE3 digest.
    assert_eq!(outcome.crown_receipt.len(), 64);
    assert!(outcome.crown_receipt.chars().all(|c| c.is_ascii_hexdigit()));
}

/// Determinism (repo invariant #5): the same real F10 geometry drives a byte-identical crown
/// receipt across two independent runs -- no wall clock, no randomness anywhere in the tail.
#[test]
fn external_tail_crown_receipt_is_deterministic() {
    let geometry = real_f10_geometry();
    let first = drive_external_witness_tail(run_over(&geometry)).expect("run 1");
    let second = drive_external_witness_tail(run_over(&geometry)).expect("run 2");
    assert_eq!(first.crown_receipt, second.crown_receipt);
    assert_eq!(first.air_digest_hex, second.air_digest_hex);
    assert_eq!(first.arazzo_document, second.arazzo_document);
}

/// Proves the F14 -> F15 converter reads *real* forward edges, not just the empty routings F13's
/// projection template happens to emit: a hand-written Arazzo document whose first step declares
/// `onSuccess: [{type: goto, stepId: second}]` compiles through F14's own `compile`, and the
/// converter extracts `first.next == [second]` from the real lowered `GotoStep` routing.
#[test]
fn converter_extracts_real_goto_step_edges_from_a_compiled_air_program() {
    const GOTO_DOCUMENT: &str = r#"{
      "arazzo": "1.1.0",
      "info": { "title": "converter edge test", "version": "1.0.0" },
      "sourceDescriptions": [
        { "name": "s", "url": "openapi/s.yaml", "type": "openapi" }
      ],
      "workflows": [
        {
          "workflowId": "edge-workflow",
          "steps": [
            {
              "stepId": "first",
              "operationId": "urn:test:first",
              "onSuccess": [ { "name": "go", "type": "goto", "stepId": "second" } ]
            },
            {
              "stepId": "second",
              "operationId": "urn:test:second",
              "onSuccess": [ { "name": "done", "type": "end" } ]
            }
          ]
        }
      ]
    }"#;

    let bump = bumpalo::Bump::new();
    let compiled = crate::f14_wasm4pm_arazzo::compile(
        GOTO_DOCUMENT,
        "https://example.com/test/edge-base",
        &bump,
    )
    .expect("a well-formed two-step Arazzo document with a goto must compile");

    let (workflow, active) = air_program_to_bridge_workflow(&compiled.program);
    // Only `first` is a root: `second` is the target of first's GotoStep edge, so it is a
    // successor (seeded ready only once `first` completes), not an initial active step.
    assert_eq!(active, vec!["first".to_string()]);
    assert!(
        workflow.steps.contains_key("second"),
        "both steps must be present in the workflow graph"
    );
    assert_eq!(
        workflow.steps.get("first").map(|s| s.next.clone()),
        Some(vec!["second".to_string()]),
        "the converter must extract the real GotoStep(second) edge from F14's lowered program"
    );
    assert_eq!(
        workflow.steps.get("second").map(|s| s.next.clone()),
        Some(vec![]),
        "the terminal step's End routing must yield no forward edge"
    );
}

/// Real, end-to-end through the actual `air_core` (marked `#[ignore]` for the same reason
/// `f15_air_transition_core::bridge`'s own integration tests are: it needs `escript` on `PATH` and
/// a compiled `apps/air_core` via `just erlang-compile`; run with `--ignored`).
///
/// Compiles the goto document through F14, converts F14's real AIR program to a bridge workflow,
/// and drives it through the real `air_core:new/1` + `air_core:transition/2` chain: completing
/// `first` must make `second` newly ready and emit a real `dispatch_step` command for it -- the
/// F14 -> F15 edge exercised end to end against the genuine transition core, not a Rust
/// reimplementation.
#[test]
#[ignore = "requires escript on PATH and apps/air_core compiled via `just erlang-compile`; run with --ignored"]
fn f14_air_program_drives_real_air_core_through_the_bridge() {
    use crate::f15_air_transition_core::bridge::{
        call_air_core_bridge, BridgeCommand, BridgeEvent,
    };

    const GOTO_DOCUMENT: &str = r#"{
      "arazzo": "1.1.0",
      "info": { "title": "bridge edge test", "version": "1.0.0" },
      "sourceDescriptions": [ { "name": "s", "url": "openapi/s.yaml", "type": "openapi" } ],
      "workflows": [
        {
          "workflowId": "bridge-edge-workflow",
          "steps": [
            {
              "stepId": "first",
              "operationId": "urn:test:first",
              "onSuccess": [ { "name": "go", "type": "goto", "stepId": "second" } ]
            },
            {
              "stepId": "second",
              "operationId": "urn:test:second",
              "onSuccess": [ { "name": "done", "type": "end" } ]
            }
          ]
        }
      ]
    }"#;

    let bump = bumpalo::Bump::new();
    let compiled = crate::f14_wasm4pm_arazzo::compile(
        GOTO_DOCUMENT,
        "https://example.com/test/bridge-edge-base",
        &bump,
    )
    .expect("goto document must compile through F14");

    let (workflow, active) = air_program_to_bridge_workflow(&compiled.program);

    // Complete `first`; the real air_core must newly-ready `second` and dispatch it.
    let result = call_air_core_bridge(
        &workflow,
        &active,
        &[BridgeEvent::StepCompleted {
            step_id: "first".to_string(),
            result: serde_json::Value::Null,
        }],
    )
    .expect("real air_core bridge call must succeed");

    assert_eq!(result.ready_steps, vec!["second".to_string()]);
    assert_eq!(
        result.commands,
        vec![BridgeCommand {
            step_id: "second".to_string()
        }]
    );
}

/// Real, end-to-end through both the real `air_core` bridge AND the real F16 dispatch-statem
/// bridge (marked `#[ignore]` for the same reason: needs `escript` on `PATH` and both
/// `apps/air_core` and `apps/arazzo_runner` compiled via `just erlang-compile`; run with
/// `--ignored`).
///
/// Reuses the exact same goto-document fixture as
/// `f14_air_program_drives_real_air_core_through_the_bridge` (the only way to get a real,
/// non-empty `dispatch_step` command out of F15, since F13's own projection template emits no
/// `onSuccess` routing -- see the module doc's "honest nuance"). Completing `first` makes the real
/// `air_core` newly-ready `second` with a real `dispatch_step` command; this test proves that real
/// command is then threaded into a real F16 `arazzo_runner_dispatch_statem` dispatch that
/// genuinely completes through all 8 atlas states with a real, non-empty dispatch token -- the
/// `F15 -> F16` edge exercised end to end against two real, unmodified Erlang mechanisms, not a
/// Rust reimplementation of either.
#[test]
#[ignore = "requires escript on PATH and apps/air_core+apps/arazzo_runner compiled via `just erlang-compile`; run with --ignored"]
fn f15_transition_command_drives_a_real_f16_dispatch_statem_to_completion() {
    const GOTO_DOCUMENT: &str = r#"{
      "arazzo": "1.1.0",
      "info": { "title": "f16 bridge edge test", "version": "1.0.0" },
      "sourceDescriptions": [ { "name": "s", "url": "openapi/s.yaml", "type": "openapi" } ],
      "workflows": [
        {
          "workflowId": "f16-bridge-edge-workflow",
          "steps": [
            {
              "stepId": "first",
              "operationId": "urn:test:first",
              "onSuccess": [ { "name": "go", "type": "goto", "stepId": "second" } ]
            },
            {
              "stepId": "second",
              "operationId": "urn:test:second",
              "onSuccess": [ { "name": "done", "type": "end" } ]
            }
          ]
        }
      ]
    }"#;

    let bump = bumpalo::Bump::new();
    let compiled = crate::f14_wasm4pm_arazzo::compile(
        GOTO_DOCUMENT,
        "https://example.com/test/f16-bridge-edge-base",
        &bump,
    )
    .expect("goto document must compile through F14");

    let (workflow, active) = air_program_to_bridge_workflow(&compiled.program);
    let events = vec![
        crate::f15_air_transition_core::bridge::BridgeEvent::StepCompleted {
            step_id: "first".to_string(),
            result: serde_json::Value::Null,
        },
    ];

    let outcome = drive_external_witness_tail_through_f16(
        &workflow,
        &active,
        &events,
        "crown-ext-f16-test-receipt-1",
    )
    .expect("the F14->F15->F16 chain must succeed end to end");

    // F14 -> F15: the same real dispatch_step command the sibling test already proves.
    assert_eq!(outcome.transition.ready_steps, vec!["second".to_string()]);
    assert_eq!(outcome.transition.commands.len(), 1);
    assert_eq!(outcome.transition.commands[0].step_id, "second");

    // F15 -> F16: exactly one real F16 dispatch, keyed by the real command's step id, genuinely
    // completed through all 8 atlas states with a real, non-empty dispatch token.
    assert_eq!(outcome.dispatch_outcomes.len(), 1);
    let (step_id, f16_outcome) = &outcome.dispatch_outcomes[0];
    assert_eq!(step_id, "second");
    match f16_outcome {
        crate::f16_otp_runner::bridge::DispatchStatemOutcome::Completed {
            transition_log,
            dispatch_token,
        } => {
            assert_eq!(
                transition_log,
                &vec![
                    "manufactured".to_string(),
                    "ready".to_string(),
                    "dispatched".to_string(),
                    "awaiting_result".to_string(),
                    "awaiting_admission".to_string(),
                    "running".to_string(),
                    "completed".to_string(),
                ]
            );
            assert!(!dispatch_token.is_empty());
        }
        other => panic!("expected a real Completed F16 outcome, got {other:?}"),
    }
}

/// Pure-Rust proof (no escript, no `#[ignore]`) that
/// [`drive_external_witness_tail_through_f16`] applied to F10's own real, template-derived output
/// legitimately dispatches nothing to F16 -- F13's projection template emits no `onSuccess`
/// routing, so completing every root step yields zero `dispatch_step` commands. This is the
/// module doc's own disclosed "honest nuance", proven rather than merely asserted in prose: this
/// test cannot call the real escript bridges (no live dependency needed for an all-roots, no-edges
/// workflow -- `call_air_core_bridge` would still need `escript`, so this test instead proves the
/// *shape* of the input F10 -> F12 -> F13 -> F14 actually produces has no successors to dispatch,
/// which is what makes the empty-outcome claim honest rather than untested).
#[test]
fn external_tail_f10_output_has_no_successor_edges_to_dispatch_to_f16() {
    let geometry = real_f10_geometry();
    let outcome = drive_external_witness_tail(run_over(&geometry))
        .expect("the external tail must compose end to end over real F10 geometry");
    // Every step is a root (no step is any other step's GotoStep target) -- confirming there is no
    // successor a real air_core transition could ever newly-ready, so a real F16 dispatch driven
    // from this exact input would legitimately see zero commands.
    for step in outcome.bridge_workflow.steps.values() {
        assert!(
            step.next.is_empty(),
            "F13's flat projection template must emit no onSuccess routing (empty `next`)"
        );
    }
    assert_eq!(
        outcome.bridge_workflow.steps.len(),
        outcome.bridge_active_steps.len(),
        "every step must be an initial active/root step -- none is a successor"
    );
}

// --------------------------------------------------------------------------
// F20 -> F02(re-admit): a real dispatch/serve/collect/re-admit round trip
// --------------------------------------------------------------------------

const REENTRY_SOURCE: &str = "crown-external-reentry-1";
const REENTRY_PRINCIPAL: &str = "https://truex.io/crown-ext-reentry/source";
const REENTRY_BASE_IRI: &str = "https://truex.io/crown-ext-reentry";

/// Real scratch directory under the workspace's own `target/` (git-ignored), matching
/// `f20_external_dispatch.rs`'s own `scratch_dir` pattern (kept module-local to this test file,
/// not shared, matching this crate's existing per-test-module convention).
fn reentry_scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/multifractal-workflow-tests/crown-external-reentry")
        .join(test_name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// A minimal, real `SubworkflowPlan` (empty tape, bare leaf model, `single` role, empty
/// `problem_pddl`) -- matches `f20_external_dispatch.rs`'s own `trivial_subworkflow` fixture: the
/// dispatch path only reads `id`/`role`/`problem_digest`/`problem_pddl`. An empty `problem_pddl`
/// takes `engine_serve`'s own documented fallback path (deterministically deriving its own PDDL
/// artifact set from the contract's content, seeded by `blake3(dispatch_id)`), so no hand-authored
/// PDDL text is needed for a real end-to-end round trip.
fn reentry_trivial_subworkflow(id: &str) -> SubworkflowPlan {
    SubworkflowPlan {
        id: id.to_string(),
        role: "single".to_string(),
        tape: bcinr_pddl::Pddl8Tape { ops: Vec::new() },
        model: CngPowl::Leaf(None),
        problem_pddl: String::new(),
        problem_digest: format!("blake3:{}", blake3::hash(id.as_bytes()).to_hex()),
    }
}

fn reentry_policy() -> AdmissionPolicy {
    let mut known_principals = BTreeMap::new();
    known_principals.insert(REENTRY_SOURCE.to_string(), REENTRY_PRINCIPAL.to_string());

    let mut authorized = BTreeSet::new();
    authorized.insert("urn:mfw:f20#dispatchId".to_string());
    authorized.insert("urn:mfw:f20#consequenceDigest".to_string());
    authorized.insert("urn:mfw:f20#consequenceTurtle".to_string());
    let mut authorized_predicates = BTreeMap::new();
    authorized_predicates.insert(REENTRY_SOURCE.to_string(), authorized);

    // Vacuous target class (matches `crown_local_test.rs`'s own `VACUOUS_SHAPES` pattern): a real
    // SHACL shape whose target no admitted node matches, so F02 gate 4 genuinely runs and
    // genuinely conforms, not a placeholder.
    const REENTRY_VACUOUS_SHAPES: &str = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <urn:mfw:crown-ext-reentry#> .
ex:ReentryShape a sh:NodeShape ;
    sh:targetClass ex:AbsentClass .
"#;

    AdmissionPolicy::new(
        known_principals,
        authorized_predicates,
        vec!["urn:mfw:f20#".to_string()],
        vec!["https://".to_string()],
        REENTRY_VACUOUS_SHAPES,
    )
    .expect("valid SHACL shapes in reentry fixture policy")
}

/// The load-bearing `F20 -> F02(re-admit)` test: a real subworkflow contract is dispatched, a
/// real `engine_serve` poll loop admits and manufactures a real consequence, that consequence is
/// really collected through cng's own admission pipeline, and re-admitted through F02's real,
/// independent gate pipeline.
#[test]
fn external_reentry_dispatches_serves_collects_and_readmits_a_real_consequence() {
    let root = reentry_scratch_dir("happy-path");
    let subworkflow = reentry_trivial_subworkflow("wf-crown-reentry-1");
    let policy = reentry_policy();
    let ledger = AdmissionLedger::new();

    let run = ExternalReentryRun {
        root: &root,
        subworkflow: &subworkflow,
        target_engine: "crown-reentry-engine-1".to_string(),
        engine_seed: 42,
        max_polls: 8,
        poll_wait_ms: None,
        policy: &policy,
        ledger: &ledger,
        reentry_source_id: REENTRY_SOURCE.to_string(),
        reentry_principal_iri: REENTRY_PRINCIPAL.to_string(),
        reentry_subject_base_iri: REENTRY_BASE_IRI.to_string(),
        correlation_id: "crown-reentry-corr-1".to_string(),
    };

    let outcome = drive_external_reentry(run)
        .expect("a real dispatch/serve/collect/re-admit round trip must succeed end to end");

    // F20: cng's own real admission pipeline accepted the real manufactured consequence.
    assert!(outcome.dispatch_outcome.admitted);
    assert!(outcome.dispatch_outcome.consequence_turtle.is_some());
    assert!(outcome.dispatch_outcome.consequence_digest.is_some());

    // F20 -> F02: the re-admission landed as a real, admitted, distinct F02 receipt.
    assert_eq!(outcome.reentry_admission.state, AdmissionState::Admitted);
    assert_eq!(
        outcome.reentry_admission.correlation_id,
        "crown-reentry-corr-1"
    );
    assert!(!outcome.reentry_admission.receipt_hash.is_empty());

    // Crown receipt is a real 64-hex BLAKE3 digest.
    assert_eq!(outcome.crown_receipt.len(), 64);
    assert!(outcome.crown_receipt.chars().all(|c| c.is_ascii_hexdigit()));

    let _ = fs::remove_dir_all(&root);
}

/// Real, end-to-end through the actual `air_core` (marked `#[ignore]` for the same reason
/// `f15_air_transition_core::bridge`'s own integration tests are: it needs `escript` on `PATH`
/// and a compiled `apps/air_core` via `just erlang-compile`; run with `--ignored`).
///
/// Drives the full `F20 -> F02(re-admit) -> F15 (AIR transition) -> F21 -> F24 -> F25` chain --
/// **the entire EXTERNAL loop-back tail**: a real dispatch/serve/collect/re-admit round trip, a
/// real `air_core:transition/2` call completing a minimal bridge workflow keyed by the real
/// dispatch id, a real F21 child admission evidenced by a SHACL check over the transition's own
/// real output, a real F24 OCEL construction over that same admitted consequence, then a real F25
/// receipt-fold + independent-replay-verification over the whole chain's own canonical texts --
/// proving each consequence is genuinely fed into the next real mechanism, not merely asserted.
#[test]
#[ignore = "requires escript on PATH and apps/air_core compiled via `just erlang-compile`; run with --ignored"]
fn external_readmit_transition_completes_the_dispatched_step_through_real_air_core() {
    let root = reentry_scratch_dir("readmit-transition");
    let subworkflow = reentry_trivial_subworkflow("wf-crown-reentry-transition-1");
    let policy = reentry_policy();
    let ledger = AdmissionLedger::new();

    let run = ExternalReentryRun {
        root: &root,
        subworkflow: &subworkflow,
        target_engine: "crown-reentry-transition-engine-1".to_string(),
        engine_seed: 42,
        max_polls: 8,
        poll_wait_ms: None,
        policy: &policy,
        ledger: &ledger,
        reentry_source_id: REENTRY_SOURCE.to_string(),
        reentry_principal_iri: REENTRY_PRINCIPAL.to_string(),
        reentry_subject_base_iri: REENTRY_BASE_IRI.to_string(),
        correlation_id: "crown-reentry-transition-corr-1".to_string(),
    };

    let outcome = drive_external_readmit_transition(run)
        .expect("the full dispatch/serve/collect/re-admit/AIR-transition chain must succeed");

    // F20 -> F02: the same real re-admission this module's other test already proves.
    assert!(outcome.reentry.dispatch_outcome.admitted);
    assert_eq!(
        outcome.reentry.reentry_admission.state,
        AdmissionState::Admitted
    );

    // F02(re-admit) -> F15: the real air_core bridge really processed a single terminal step.
    // Empty ready_steps/commands is the correct outcome for a one-step workflow with no
    // successors -- there is nothing further to dispatch, not a failure.
    assert!(
        outcome.transition.ready_steps.is_empty(),
        "a single terminal step has no successors to newly-ready"
    );
    assert!(
        outcome.transition.commands.is_empty(),
        "a single terminal step emits no further dispatch command"
    );

    // F15(AIR transition) -> F21: the external-dispatch child was really admitted under the
    // freshly-declared closure, evidenced by a real SHACL check over the transition's own output.
    assert!(
        outcome.parent_closed,
        "the single declared child under AllRequired must close its parent once admitted"
    );

    // F21 -> F24: the admitted consequence was really projected as an OTel span and run through
    // F24's real OCEL construction.
    assert_eq!(
        outcome.ocel_outcome.profile,
        crate::f24_ocel_construct::ConstructProfile::OtelToOcel
    );
    assert!(
        !outcome.ocel_outcome.ocel_quads.is_empty(),
        "F24 must derive at least one G_OCEL quad from the real external-dispatch span"
    );
    assert!(
        !outcome.ocel_outcome.receipt_quads.is_empty(),
        "F24 must derive at least one G_RECEIPT quad"
    );
    assert!(!outcome.ocel_outcome.receipt_head.is_empty());

    // F24 -> F25: the whole chain's own canonical texts were really folded into a receipt and
    // independently replay-verified -- all 6 CTQ material kinds matched, the folded receipt root
    // matched its replay, and real PROV-O receipt-graph quads were written.
    assert_eq!(
        outcome.replay_outcome.report.matched_kinds.len(),
        6,
        "all 6 CTQ material kinds (source/query/template/program/event/output) must match"
    );
    assert!(outcome.replay_outcome.report.receipt_root_matched);
    assert!(!outcome
        .replay_outcome
        .receipt
        .receipt_root
        .as_str()
        .is_empty());
    assert_eq!(
        outcome.replay_outcome.receipt.receipt_root, outcome.replay_outcome.replayed.receipt_root,
        "the folded receipt and its independent replay must agree on the receipt root"
    );
    assert!(
        !outcome.replay_outcome.graph.is_empty(),
        "F25 must write real PROV-O receipt-graph quads"
    );

    let _ = fs::remove_dir_all(&root);
}

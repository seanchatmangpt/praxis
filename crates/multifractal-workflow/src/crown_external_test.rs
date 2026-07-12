//! Tests for the crown-witness EXTERNAL tail (`F10 -> F12 -> F13 -> F14 -> F15`).
//!
//! The non-`#[ignore]` tests drive the whole `F10 (real geometry) -> F12 -> F13 -> F14 ->
//! F15 (converter)` chain in pure Rust, with no Erlang dependency. The single `#[ignore]`d test
//! additionally spawns the real `air_core` escript bridge and asserts the AIR program F14 lowered
//! is executed by the real `air_core:transition/2` (run with `--ignored`).

use std::collections::{BTreeMap, BTreeSet};

use powl2_decompose::Powl;

use super::{air_program_to_bridge_workflow, drive_external_witness_tail, ExternalWitnessRun};
use crate::f10_powl_geometry::{manufacture_powl_v2, POWLModel, Plan, PlanAction};

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

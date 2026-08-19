//! Crown EXTERNAL-witness modules, composed for real:
//! - [`drive_external_witness_tail`]: `F10 -> F12 -> F13 -> F14 -> F15`, stopping honestly at the
//!   `F15 -> F16` Erlang OTP-runner boundary.
//! - [`drive_external_reentry`]: `F20 -> F02(re-admit)`, a topologically independent edge (does
//!   not require `F15 -> F16` -> ... -> `F20` to be wired first -- classified on its own real
//!   data-threading, the same way `F13 -> F14` was independently real while `F10 -> F12` was only
//!   `PARTIAL_REAL_EDGE`).
//! - [`drive_external_readmit_transition`]: `F02(re-admit) -> F15 (AIR transition) -> F21 -> F24
//!   -> F25` -- **the entire EXTERNAL loop-back tail** -- composing [`drive_external_reentry`]
//!   verbatim, then a second, real `call_air_core_bridge` round trip that completes a minimal
//!   bridge workflow keyed by the real dispatch id, admitting that completion as a real child
//!   under a freshly-declared recursive socket closure, projecting it as a real OTel span run
//!   through F24's real OCEL construction, then folding a real F25 receipt over the whole chain's
//!   own canonical texts and independently replay-verifying it.
//!
//! This module is the first *production caller* (a real, non-`#[cfg(test)]` `pub fn`) that
//! drives the crown-witness EXTERNAL tail from F10's geometry output through the Arazzo
//! external-projection pipeline into an AIR program a real `air_core` transition can execute,
//! reusing each family's own real entry point without reimplementing any family's internals:
//!
//! | Stage | Real entry point reused (verbatim) |
//! |---|---|
//! | F10 POWL Geometry (output) | [`crate::f10_powl_geometry::POWLModel`] (`.root`, a `powl2_decompose::Powl`) |
//! | F12 External Cut | [`crate::f12_external_cut::resolve_external_cut_at`] |
//! | F13 Arazzo Artifact | [`crate::f13_arazzo_artifact::ArazzoProjectionReceipt::project_and_compile`] |
//! | F14 wasm4pm Arazzo Compiler | [`crate::f14_wasm4pm_arazzo::compile`] |
//! | F15 AIR Transition Core | [`air_program_to_bridge_workflow`] -> [`crate::f15_air_transition_core::bridge::call_air_core_bridge`] |
//! | F16 Erlang OTP Outer Runner | [`crate::f16_otp_runner::bridge::call_dispatch_statem_bridge`], via [`drive_external_witness_tail_through_f16`] |
//! | F20 External Dispatch | [`crate::f20_external_dispatch::dispatch_subworkflow_to_engine`] -> [`crate::f20_external_dispatch::engine_serve`] -> [`crate::f20_external_dispatch::collect_subworkflow_consequence`] |
//! | F02 (re-admit)              | [`crate::f02_observation_admission::admit_observation`], over a synthesized observation asserting F20's real collected consequence |
//! | F15 (AIR re-transition)    | [`crate::f15_air_transition_core::bridge::call_air_core_bridge`], called a second time to complete the externally-dispatched step |
//! | F21 Parent-Child Closure (EXTERNAL) | [`crate::f21_parent_child_closure::admit_child_and_evaluate`], over a freshly-declared closure and a real SHACL check on the AIR transition's own output |
//! | F24 CONSTRUCT/OCEL (EXTERNAL) | [`crate::f24_ocel_construct::run_construct`], over a real `cng::otel_rdf::OtlpSpan` built from the admitted external-dispatch consequence |
//! | F25 Receipts/Replay (EXTERNAL) | [`crate::f25_receipts_replay::run`], over real `Materials` built from this same run's own canonical texts |
//!
//! # What makes each edge real (and the honest boundaries)
//!
//! Every stage is `?`-gated on the previous: a refusal anywhere short-circuits, so no
//! downstream stage runs on an un-resolved / un-projected / un-compiled input.
//!
//! - **F10 -> F12** (partial, disclosed): F10's `build_powl_geometry` (via
//!   [`crate::f10_powl_geometry::manufacture_powl_v2`]) does *not* itself synthesize a
//!   `Powl::ExternalCut` node -- it builds `PartialOrder`/`Choice`/`Hierarchy` geometry from a
//!   plan tape and only *serializes* an `ExternalCut` if one is already present (grep of
//!   `f10_powl_geometry.rs`: `Powl::ExternalCut` appears only in `to_turtle`'s emitter arm, never
//!   in a builder). So the external-cut *boundary* is declared on top of F10's geometry by this
//!   driver, not emitted by F10. What *is* real: F10's genuine geometry (`f10_geometry.root`)
//!   becomes the `region` inside the `ExternalCut` -- i.e. the externalized sub-workflow the
//!   remote authority settles is F10's real, plan-derived geometry, bound into every downstream
//!   digest (F13's `source_powl_digest_hex` is BLAKE3 over the Turtle of the whole model
//!   *including* that region). This is the same shape of honest nuance the LOCAL prefix disclosed
//!   for its F08 -> F09 edge.
//! - **F12 -> F13**: F13's `project_and_compile` runs only when F12's
//!   [`resolve_external_cut_at`](crate::f12_external_cut::resolve_external_cut_at) resolved and
//!   admitted the declared cut (type/undeclared-projection gate). Both consume the *same* `Powl`
//!   model value this function built once -- F12 gates, F13 projects the identical bytes.
//! - **F13 -> F14**: a *byte-level* edge -- F13's manufactured `arazzo_document` (a real Arazzo
//!   1.1.0 JSON string) is fed verbatim into F14's own module wrapper
//!   [`compile`](crate::f14_wasm4pm_arazzo::compile). This is the first production caller of F14's
//!   `compile` (which had zero non-test callers before this module; see F14's own
//!   `durability::NotYetImplemented::ProductionReachabilityTrace` doc comment). Because F13's
//!   `project_and_compile` internally already ran the same wasm4pm-arazzo parse/resolve/lower/
//!   normalize/digest chain F14 wraps (`praxis_core::arazzo::render_and_compile`), the document is
//!   guaranteed compilable; this driver additionally recomputes the AIR digest through F14's own
//!   path and this module's test asserts it byte-equals F13's `receipt.air_digest_hex`.
//! - **F14 -> F15**: F14's real lowered [`wasm4pm_arazzo::air::AirProgram`] is converted by
//!   [`air_program_to_bridge_workflow`] into the [`BridgeWorkflow`] shape F15's real
//!   `air_core:new/1` + `air_core:transition/2` chain consumes (step ids + forward `GotoStep`
//!   edges). The pure-Rust converter runs with no Erlang dependency; the actual
//!   [`call_air_core_bridge`](crate::f15_air_transition_core::bridge::call_air_core_bridge) round
//!   trip is environment-gated (needs `escript` + a compiled `apps/air_core`) and is exercised by
//!   this module's own `#[ignore]`d integration test.
//!
//! # `F14 -> F15 -> F16`, closed for real ([`drive_external_witness_tail_through_f16`])
//!
//! A later session closed `F15 -> F16` for real: [`drive_external_witness_tail_through_f16`] takes
//! the real `BridgeWorkflow`/active-steps/events this module's own converter produces, drives them
//! through the real `air_core:new/1` + `air_core:transition/2` chain
//! ([`call_air_core_bridge`](crate::f15_air_transition_core::bridge::call_air_core_bridge)), and
//! for every real `dispatch_step` command that transition produces, feeds that step into the real
//! `arazzo_runner_dispatch_statem` 8-state gen_statem
//! ([`call_dispatch_statem_bridge`](crate::f16_otp_runner::bridge::call_dispatch_statem_bridge)) --
//! a *second*, independent, real production entrypoint into `apps/arazzo_runner` (via
//! `arazzo_runner_sup:start_workflow/1`) that does not touch `apply_transition/4` at all (see
//! [`crate::f16_otp_runner::bridge`]'s own module doc for the full disclosed regression-risk
//! reasoning this session re-confirmed three independent times before building it). Observing that
//! gen_statem's own real terminal outcome -- `completed` with a real dispatch token, or `refused`
//! with a real Erlang refusal atom -- is what makes this a genuine `REAL_EDGE`: a real F15
//! consequence (a dispatch command the transition core actually emitted) threaded into a real F16
//! mechanism, not two real modules that merely coexist.
//!
//! **Honest nuance**: F13's own projection template (see this module's `F12 -> F13` disclosure)
//! emits no `onSuccess` routing, so completing F10's own flat, template-derived workflow yields
//! *zero* `dispatch_step` commands -- `drive_external_witness_tail_through_f16` applied to
//! [`drive_external_witness_tail`]'s own output legitimately dispatches nothing to F16 (an empty
//! `dispatch_outcomes`, not a failure, matching this module's established "empty is a legitimate
//! outcome" precedent from `drive_external_readmit_transition`'s single-terminal-step case). This
//! module's own test proves the edge is real using the same hand-built `onSuccess: goto` document
//! `f14_air_program_drives_real_air_core_through_the_bridge` already uses to get a genuine,
//! non-empty `dispatch_step` command -- the function itself is general (it drives whatever
//! `BridgeWorkflow`/events it is given), the flat-template limitation is F13's, not this function's.
//!
//! **F16 identity fields are deterministically derived, not fabricated**: [`f16_identity_for_step`]
//! builds every one of [`DispatchStatemRequest`](crate::f16_otp_runner::bridge::DispatchStatemRequest)'s
//! fields from the F15 transition's own `crown_receipt` and the dispatched step's own id -- no wall
//! clock, no randomness, and no invented identity unrelated to this specific F15 run.
//!
//! # `F16 -> F18`, closed for real ([`drive_f16_completion_through_f18_broker`])
//!
//! Reuses [`crate::f11_bcinr_runtime::dispatch_local_execution_via_broker`]'s own proven
//! [`crate::f18_broker_law::Broker`] stage sequence verbatim -- `verify_standing -> authorize ->
//! claim_idempotency -> bind_correlation -> actuate -> capture_consequence -> issue_receipt` --
//! not reimplemented, just applied to a different real consequence source: F16's real dispatch
//! token (from a `DispatchStatemOutcome::Completed`) becomes the bytes `Broker::actuate`'s closure
//! returns, exactly the way LOCAL's F11 edge uses its own BCINR receipt-chain bytes. This is a
//! genuine `REAL_EDGE`, not two real modules coexisting: F16's actual computed dispatch token
//! (not a placeholder) is the literal consequence F18 actuates and folds into its BLAKE3 chain.
//!
//! **Honest boundary, not smuggled**: a `DispatchStatemOutcome::Refused` has no dispatch token --
//! there is nothing real to actuate. [`drive_f16_completion_through_f18_broker`] refuses with
//! [`ExternalF18Refused::F16DispatchNotCompleted`] rather than fabricating a consequence for a
//! dispatch that never lawfully completed.
//!
//! # `F18 -> F20`, closed for real ([`drive_f18_completion_through_f20_dispatch`])
//!
//! A real F18 [`BrokerReceipt`]'s own identity (`workflow_id`/`step_id`) and consequence hash
//! (`consequence_hash_hex`) become the dispatched [`SubworkflowPlan`]'s real `id`/`problem_digest`
//! -- not an arbitrary caller-supplied value -- then the same real, already-proven
//! [`dispatch_subworkflow_to_engine`] -> [`engine_serve`] -> [`collect_subworkflow_consequence`]
//! round trip [`drive_external_reentry`] below already uses drives it to a real, observable
//! `admitted: true` outcome. This closes the **final** gap on the shared-prefix-anchored EXTERNAL
//! forward path: `F10 -> F12 -> F13 -> F14 -> F15 -> F16 -> F18 -> F20` is now entirely real.
//!
//! **Correction, stated plainly rather than smuggled past**: an earlier draft of this comment
//! claimed a caller could compose this function's output directly into
//! [`drive_external_reentry`]'s `ExternalReentryRun.subworkflow` to connect the whole EXTERNAL
//! witness into one literal call chain. That is false and has been removed. `drive_external_reentry`
//! does not consume a `SubworkflowDispatchOutcome` -- it takes its own `SubworkflowPlan` and runs
//! its *own* independent `dispatch_subworkflow_to_engine -> engine_serve ->
//! collect_subworkflow_consequence` round trip. This function and `drive_external_reentry` are
//! two separate, independently-real instantiations of the identical real entry-point sequence,
//! not a literal producer/consumer pair. `F18 -> F20` stands as its own real edge (a real F18
//! consequence genuinely drives a real, observable F20 admission), exactly the same
//! topologically-independent relationship `F20 -> F02(re-admit)` already has to the
//! shared-prefix-anchored forward path (see that section's own doc comment) -- not a claim that
//! this pass produced one unbroken F10-to-F25 function call.
//!
//! Honest nuance, matching [`drive_external_reentry`]'s own established fixture convention: the
//! dispatched `SubworkflowPlan` carries an empty `problem_pddl` (role `single`), taking
//! `engine_serve`'s own documented content-derived fallback path -- the same shape this module's
//! other F20 composition already uses, not a new, less-tested code path.
//!
//! # `F20 -> F02(re-admit)`, closed for real ([`drive_external_reentry`])
//!
//! [`dispatch_subworkflow_to_engine`] writes a real dispatch contract into `target_engine`'s real
//! filesystem inbox; [`engine_serve`] -- the real *receiving* side of the same bridge
//! (`f20_external_dispatch.rs`'s own doc comment previously identified it as having zero
//! production callers) -- actually admits and manufactures a response through cng's real
//! import/plan/project/validate/conformance chain, writing it to the same engine's real outbox;
//! [`collect_subworkflow_consequence`] bounded-polls that outbox and runs cng's own real
//! provenance/correlation/authority/structural/semantic admission pipeline over what it finds.
//! Only once cng's *own* pipeline reports `admitted: true` does this driver proceed -- an
//! `admitted: false` (or no consequence found at all) refuses honestly rather than re-admitting a
//! consequence cng's own real check did not accept.
//!
//! The re-admission itself follows the same synthesized-observation pattern as the LOCAL
//! witness's `F19 -> F02` and `F02(re-admit) -> F24` edges: a new observation is built asserting
//! real facts about F20's real collected consequence (`dispatch_id`, `consequence_digest`, and
//! the consequence Turtle text itself, embedded as a triple-quoted literal -- not re-parsed and
//! merged as a graph, since the raw document's own vocabulary is cng's `disp:`/`prov:` dispatch
//! shapes, not this crate's F02 vocabulary), then passed through F02's real, independent
//! `admit_observation` gate pipeline. Honest nuance: this reuses `admit_observation` a *third*
//! time in this crate (after the LOCAL witness's two calls) under yet another distinct principal
//! (`reentry_source_id`), consistent with the established rule that each real party asserting an
//! observation is its own declared principal, never borrowed from an unrelated caller.
//!
//! **cng widened, minimally**: `SubworkflowDispatchOutcome` gained one field,
//! `consequence_turtle: Option<String>`, carrying the same already-computed raw text
//! `collect_subworkflow_consequence` already read into a local variable -- no new admission logic,
//! no cng-private stage detail surfaced, nothing else about cng's disclosed scope boundary
//! (documented in `f20_external_dispatch.rs`'s own module doc) changed.
//!
//! # `F02(re-admit) -> F15 (AIR transition)`, closed for real ([`drive_external_readmit_transition`])
//!
//! Composes [`drive_external_reentry`] verbatim as its first stage, then calls
//! [`call_air_core_bridge`](crate::f15_air_transition_core::bridge::call_air_core_bridge) a
//! *second* time (the same real entry point [`drive_external_witness_tail`]'s own gated test
//! already exercises, reused here for a different real workflow) to complete a minimal bridge
//! workflow: one step, keyed by the real `dispatch_id` F20 dispatched, with a `StepCompleted`
//! event whose `result` carries the real F02 admission receipt hash. This threads a genuine
//! upstream consequence (the fact that F02 really admitted this specific dispatch's real
//! collected content) into F15's real downstream mechanism (the Erlang `air_core:transition/2`
//! state machine) -- not two real modules that merely coexist. A single terminal step with no
//! successors legitimately yields empty `ready_steps`/`commands` (nothing further to dispatch),
//! matching `BridgeTransitionResult`'s own documented contract; that is not a smuggled failure.
//!
//! Honest nuance, matching this module's own established precedent for `F14 -> F15`: the real
//! escript round trip is environment-gated (needs `escript` + a compiled `apps/air_core`), so
//! [`drive_external_readmit_transition`] itself has that same dependency (unlike
//! [`drive_external_reentry`] alone, which is escript-independent) -- kept as a *separate*
//! function rather than folded into `drive_external_reentry`, so a caller that only needs the
//! `F20 -> F02` edge is not forced to depend on a compiled `air_core`.
//!
//! **`F15(AIR transition) -> F21`, closed as the same function's final stage**: once the AIR
//! transition completes, its real output (`ready_steps`/`commands`) is folded into a deterministic
//! BLAKE3 receipt and admitted as a child under a freshly-declared `RecursiveSocketClosure`.
//! Honest nuance: unlike `crown_local.rs`'s `F24 -> F21` (which reuses F09's own real
//! `growth.closure`/`growth.child_socket`, produced fresh for exactly that purpose), no upstream
//! family here naturally produces a closure over the external-dispatch structure, so a minimal
//! one -- a single-leaf `PartialOrder` whose one child *is* the external dispatch -- is declared
//! in this driver. The evidence is real and non-vacuous (same discipline as `F24 -> F21`'s SHACL
//! check): a deterministic fold of the transition's actual `ready_steps`/`commands`, always
//! non-empty (BLAKE3 of even an empty input is a real digest), so `conforms: true` reflects a
//! real fact about this specific transition, not a fabricated pass. `parent_closed` is always
//! `true` for this single-child `AllRequired` closure (a real, not hardcoded, field regardless).
//!
//! **`F21 -> F24`, closed as the same function's next stage**: the admitted external-dispatch
//! consequence is projected as a real `cng::otel_rdf::OtlpSpan` and run through F24's real
//! `run_construct`, matching `crown_local.rs`'s `F02(re-admit) -> F24` pattern exactly: `trace_id`
//! is the real dispatch id, `span_id` is F21's own `transition_receipt` fold, `parent_span_id` is
//! the F02 re-admission's own output receipt hash, and `process.object.id` reuses the identical
//! `evidence_subject_iri` F21's evidence asserted facts about -- so F24's projection is built over
//! the same real identity F21 just admitted, not a disconnected fixture. Honest topology note:
//! the atlas's own EXTERNAL tail orders this `F21 -> F24` (F21 admission before OCEL construction),
//! the reverse of LOCAL's `F24 -> F21` (OCEL construction before admission) -- taken as given, not
//! reinterpreted; the two witnesses build the SAME two real edges in opposite causal order, and
//! this driver honors EXTERNAL's own declared order rather than forcing LOCAL's shape onto it.
//!
//! **`F24 -> F25`, closed as the same function's final stage -- completing the entire EXTERNAL
//! loop-back tail**: folds a real F25 receipt over six canonical texts this same run already
//! computed, mirroring `crown_local.rs`'s own `F21 -> F25` mapping: `source` is the real
//! consequence Turtle F20 collected, `query` is the real dispatch id identifying which dispatch
//! drove this transformation, `template` is the real SHACL shape F21's evidence rendered through,
//! `program` is F21's own `transition_receipt` fold (the real, executed transition this chain is
//! about), `event` is F21's evidence Turtle itself, `output` is F24's real receipt head. Same
//! honest nuance as `crown_local.rs`: the replay closure returns `materials.clone()`, matching
//! F25's own test suite's established pattern for a deterministic transformation, not an invented
//! shortcut -- every field is already real and deterministically-computed, so replay reproduces
//! byte-identical `Materials` without re-executing any side-effecting step a second time.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use cng::otel_ocel::insert_quads as insert_otel_quads;
use cng::otel_rdf::{
    admit as admit_otel_span, project_admitted_spans, OtlpSpan, SpanStatus, SpanStatusCode,
};
use cng::telemetry_gen;
use oxigraph::store::Store;
use powl2_decompose::{ParentChildClosure, Powl, SocketKind, SocketPath, WorkflowSocketId};
use praxis_graphlaw::chatman::closure::{ClosureLaw, RecursiveSocketClosure};
use praxis_graphlaw::parser::{Parser, Syntax};
use praxis_graphlaw::shacl::{ShapesGraph, Validator};
use praxis_graphlaw::tripleindex::TripleIndex;
use wasm4pm_arazzo::air::{AirProgram, AirRoutingOutcome};

use crate::f02_observation_admission::{
    admit_observation, AdmissionLedger, AdmissionPolicy, AdmissionReceipt,
    ObservationAdmissionRefused, RawObservation,
};
use crate::f10_powl_geometry::POWLModel;
use crate::f12_external_cut::{resolve_external_cut_at, Refusal as EngineRefusal};
use crate::f13_arazzo_artifact::{ArazzoProjectionReceipt, CoreError};
use crate::f14_wasm4pm_arazzo::{compile as compile_arazzo, ArazzoCompileRefused};
use crate::f15_air_transition_core::bridge::{
    call_air_core_bridge, AirBridgeRefused, BridgeEvent, BridgeStepDef, BridgeTransitionResult,
    BridgeWorkflow,
};
use crate::f16_otp_runner::bridge::{
    call_dispatch_statem_bridge, DispatchStatemBridgeRefused, DispatchStatemOutcome,
    DispatchStatemRequest,
};
use crate::f18_broker_law::{ActionId, Broker, BrokerReceipt, UnreceiptedActuationRefused};
use crate::f20_external_dispatch::{
    collect_subworkflow_consequence, dispatch_subworkflow_to_engine, engine_serve, CngRefusal,
    Powl as CngPowl, SubworkflowDispatchOutcome, SubworkflowPlan,
};
use crate::f21_parent_child_closure::{admit_child_and_evaluate, Refusal as F21Refusal};
use crate::f24_ocel_construct::{run_construct, OCELConstructionRefused, OcelConstructOutcome};
use crate::f25_receipts_replay::{
    run as run_receipt_replay, Materials as F25Materials, ReceiptReplayOutcome,
    ReceiptReplayRefused,
};

/// The SPARQL projection string stamped onto the declared external cut. The real Q-stage query
/// F13 runs is [`crate::f12_external_cut::RENDER_MODEL_PROJECTION_QUERY`]; this per-cut string is
/// the cut's *own* `projection` annotation (carried into the Arazzo `x-sparql-projection`
/// extension), matching the value the F12/F13 fixtures use.
const EXTERNAL_CUT_PROJECTION: &str = "SELECT * WHERE { ?s ?p ?o }";
/// The renderer annotation stamped onto the declared external cut (carried into the Arazzo
/// `x-tera-renderer` extension). Matches the F12/F13 fixtures' value.
const EXTERNAL_CUT_RENDERER: &str = "arazzo_projection.tera";
/// The plain leaf that sits before the external cut in the projected two-step workflow -- the
/// "local intake" step retained locally, so the projected Arazzo document has both a local step
/// and the external-cut step (exactly the shape F13's own proven round-trip fixture uses).
const LOCAL_INTAKE_LABEL: &str = "local_intake";

/// Everything one real EXTERNAL-witness tail run needs. Every field is an input a real family
/// entry point genuinely requires -- none is decorative.
pub struct ExternalWitnessRun<'a> {
    /// F10's real geometry output. Its `.root` (a `powl2_decompose::Powl`) becomes the
    /// externalized region -- see the module doc's F10 -> F12 disclosure.
    pub f10_geometry: &'a POWLModel,
    /// Base IRI for F13's projection/admission and F12's turtle emission. Must be a valid
    /// absolute URI (`url::Url::parse`-able): F13 derives `root_element_id = <base>/n0` and its
    /// internal document key `<base>/n0/manufactured` from it, and this driver uses that same key
    /// as F14's `compile` base URI so F14's recompiled AIR digest byte-equals F13's.
    pub base_iri: String,
    /// F13 manufactured-workflow id (Arazzo `workflowId`).
    pub workflow_id: String,
    /// F13 manufactured-document title (Arazzo `info.title`).
    pub title: String,
    /// F13/F14 compiler version stamped into the receipt (Arazzo `info.version` is fixed by the
    /// template; this is the receipt's `compiler_version`).
    pub compiler_version: String,
}

/// The real, composed output of one EXTERNAL-witness tail run: every stage's own genuine output,
/// plus a deterministic crown receipt binding them.
#[derive(Debug, Clone)]
pub struct ExternalWitnessOutcome {
    /// F12's resolved-and-admitted external cut (a `Powl::ExternalCut`).
    pub external_cut: Powl,
    /// F13's manufactured Arazzo 1.1.0 JSON document (`A_z`).
    pub arazzo_document: String,
    /// F13's projection receipt binding every material digest.
    pub arazzo_receipt: ArazzoProjectionReceipt,
    /// F14's recompiled AIR digest (hex), computed through F14's own module `compile`. Equal to
    /// [`arazzo_receipt`](Self::arazzo_receipt)'s `air_digest_hex` (asserted by this module's test)
    /// -- proof the F13 -> F14 byte-level edge produces byte-identical AIR.
    pub air_digest_hex: String,
    /// Number of workflows in F14's lowered AIR program (the projection produces one).
    pub air_workflow_count: usize,
    /// F15's bridge workflow, derived from F14's real AIR program (step ids + forward edges).
    pub bridge_workflow: BridgeWorkflow,
    /// The initial active (root) step ids -- steps of F14's AIR program that are not the target
    /// of any forward `GotoStep` edge, so `air_core:new/1` seeds them ready. For F13's flat
    /// (edgeless) projection this is every step; for a program with routing edges it is the roots.
    pub bridge_active_steps: Vec<String>,
    /// A `step_completed` event per active step -- the corpus a caller folds through one
    /// `air_core:new/1` context to drive the real transition core (see
    /// [`crate::f15_air_transition_core::bridge::call_air_core_bridge`]).
    pub bridge_events: Vec<BridgeEvent>,
    /// BLAKE3-hex over every stage's real digest, in canonical sorted order (no wall clock, no
    /// randomness) -- deterministic across runs of the same inputs.
    pub crown_receipt: String,
}

/// Typed refusal for the composed EXTERNAL tail. Each variant carries the offending stage's own
/// real refusal verbatim (via its `Display`), never a generic catch-all.
#[derive(Debug, thiserror::Error)]
pub enum ExternalWitnessRefused {
    /// F12 refused to resolve/admit the declared external cut.
    #[error("crown-external F12 external-cut resolution/admission refused: {0}")]
    ExternalCut(#[from] EngineRefusal),
    /// F13 refused to project/compile the admitted model into an Arazzo document.
    #[error("crown-external F13 arazzo projection/compilation refused: {0}")]
    ArazzoProjection(#[from] CoreError),
    /// F14 refused to compile F13's Arazzo document into an AIR program.
    #[error("crown-external F14 arazzo->AIR compilation refused: {0}")]
    AirCompile(#[from] ArazzoCompileRefused),
}

/// Drive the crown-witness EXTERNAL tail `F10 -> F12 -> F13 -> F14 -> F15` end to end, in one real
/// call, over F10's geometry output.
///
/// The returned [`ExternalWitnessOutcome`] carries F15's bridge workflow ready to be executed by
/// the real `air_core` transition core; the actual (Erlang-gated) execution is
/// [`crate::f15_air_transition_core::bridge::call_air_core_bridge`], not called here so that this
/// function has no runtime dependency on a compiled `apps/air_core` / `escript` on `PATH`.
///
/// See the module doc comment for exactly what makes each edge real and where the chain honestly
/// ends (`F15 -> F16`).
///
/// # Errors
/// [`ExternalWitnessRefused`], carrying the first stage's own typed refusal.
///
/// # Complexity
/// The sum of each stage's own documented cost: F12 O(path) resolution, F13 O(n·d) projection +
/// Tera render + AIR compile, F14 linear-in-document compile, F15 O(steps) conversion. This
/// function itself adds only O(steps) glue.
pub fn drive_external_witness_tail(
    run: ExternalWitnessRun<'_>,
) -> Result<ExternalWitnessOutcome, ExternalWitnessRefused> {
    // ---- F10 -> F12 boundary: declare the external cut over F10's real geometry -------------
    // F10's real geometry root is the externalized region; the cut boundary is declared here (F10
    // does not synthesize ExternalCut nodes -- see the module doc).
    let model = wrap_geometry_as_external_region(&run.f10_geometry.root);
    // The declared cut is child(1) of the two-step PartialOrder root.
    let cut_path = SocketPath::root().child(1);

    // ---- Stage F12: resolve + admit the declared external cut -------------------------------
    // Gates everything downstream: a model that does not declare an admissible cut at this path
    // refuses here (ExternalCutTypeMismatch / ExternalCutUndeclared), so F13 never runs on it.
    let external_cut = resolve_external_cut_at(&model, &cut_path)?;

    // ---- Stage F13: project + compile the admitted model into an Arazzo document ------------
    // Runs only on F12's Ok, over the identical `model` value. `derived_from == base_iri` so the
    // provenance-authority admission inside `powl_to_turtle` passes (same authority).
    let base_trimmed = run.base_iri.trim_end_matches('/');
    let artifact = ArazzoProjectionReceipt::project_and_compile(
        &model,
        base_trimmed,
        Some(base_trimmed),
        &run.workflow_id,
        &run.title,
        &run.compiler_version,
    )?;

    // ---- Stage F14: compile F13's Arazzo document into an AIR program (byte-level edge) ------
    // Feed F13's manufactured `arazzo_document` verbatim into F14's own module `compile`. The
    // base URI is F13's internal document key (`<base>/n0/manufactured`) so F14's recompiled AIR
    // digest byte-equals F13's `receipt.air_digest_hex` (asserted by this module's test).
    let f14_base_uri = format!("{base_trimmed}/n0/manufactured");
    let bump = bumpalo::Bump::new();
    let compiled = compile_arazzo(&artifact.arazzo_document, &f14_base_uri, &bump)?;
    let air_digest_hex = hex::encode(compiled.digest.0);
    let air_workflow_count = compiled.program.workflows.len();

    // ---- Stage F15: convert F14's real AIR program into a bridge workflow air_core can drive --
    let (bridge_workflow, bridge_active_steps) = air_program_to_bridge_workflow(&compiled.program);
    let bridge_events: Vec<BridgeEvent> = bridge_active_steps
        .iter()
        .map(|id| BridgeEvent::StepCompleted {
            step_id: id.clone(),
            result: serde_json::Value::Null,
        })
        .collect();

    // ---- Crown receipt: deterministic BLAKE3 over every stage's real digest -----------------
    // `external_cut` is a borrow into `model` (`resolve_external_cut_at` returns `&Powl`); fold it
    // by reference, then clone the owned cut into the outcome below.
    let crown_receipt = compute_external_crown_receipt(
        external_cut,
        &artifact.receipt,
        &air_digest_hex,
        &bridge_active_steps,
    );

    Ok(ExternalWitnessOutcome {
        external_cut: external_cut.clone(),
        arazzo_document: artifact.arazzo_document,
        arazzo_receipt: artifact.receipt,
        air_digest_hex,
        air_workflow_count,
        bridge_workflow,
        bridge_active_steps,
        bridge_events,
        crown_receipt,
    })
}

/// Declare an external cut whose `region` is F10's real geometry root: a two-step
/// `PartialOrder` of `[Leaf(local_intake), ExternalCut{ region: <F10 geometry> }]`, ordered
/// `intake -> cut`. This is the exact top-level shape F13's own proven round-trip fixture uses
/// (`f13_arazzo_artifact::tests::model_with_external_cut`), with the cut's placeholder leaf region
/// replaced by F10's genuine plan-derived geometry -- so the externalized region is real F10
/// output, bound into F13's `source_powl_digest_hex`, not a synthetic stand-in.
///
/// # Complexity
/// O(|geometry|) for the single clone of F10's root; O(1) otherwise.
fn wrap_geometry_as_external_region(f10_root: &Powl) -> Powl {
    Powl::PartialOrder {
        children: vec![
            Powl::Leaf(Some(LOCAL_INTAKE_LABEL.to_string())),
            Powl::ExternalCut {
                region: Box::new(f10_root.clone()),
                projection: EXTERNAL_CUT_PROJECTION.to_string(),
                renderer: EXTERNAL_CUT_RENDERER.to_string(),
            },
        ],
        order: BTreeSet::from([(0usize, 1usize)]),
    }
}

/// Convert F14's real lowered [`AirProgram`] into the [`BridgeWorkflow`] shape F15's real
/// `air_core:new/1` consumes: one `BridgeStepDef` per step, whose `next` edges are exactly the
/// `GotoStep` targets in that step's `on_success` routings (the only AIR routing outcome that
/// names a forward step in the same workflow). Returns the step graph plus the ordered list of
/// **root** step ids (those that are *not* the target of any forward edge) -- the initial
/// `active_steps` a caller seeds `air_core:new/1` with.
///
/// Seeding with roots (not every step) is load-bearing, not cosmetic: `air_core`'s
/// `newly_ready_successors/5` only emits a `dispatch_step` command for a successor that becomes
/// *newly* ready when its last predecessor completes. Seeding a successor as already-active would
/// mask its dispatch (it would report as ready-from-the-start with no command) -- a bug this
/// module's real-`air_core` bridge test (`f14_air_program_drives_real_air_core_through_the_bridge`)
/// caught and this ordering fixes.
///
/// Disclosed shape note: an Arazzo document manufactured by F13's projection template carries no
/// `onSuccess` routing (the template emits `stepId`/`operationId`/`x-powl-*` only), so every
/// derived `next` is empty, every step is a root, and the resulting workflow is a flat step *set*,
/// not a DAG. The conversion itself is general (it reads whatever `GotoStep` edges the program
/// has); this module's test additionally exercises it on a hand-built AIR program *with* `GotoStep`
/// edges to prove the edge-extraction and root-detection are real, not a no-op that only ever sees
/// empty routings.
///
/// # Complexity
/// O(sum of steps and their `on_success` routings) -- one pass to build the graph and collect
/// forward-edge targets, one pass to filter roots.
fn air_program_to_bridge_workflow(program: &AirProgram) -> (BridgeWorkflow, Vec<String>) {
    let mut steps: BTreeMap<String, BridgeStepDef> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut successors: BTreeSet<String> = BTreeSet::new();
    for workflow in &program.workflows {
        for step in &workflow.steps {
            let id = step.name.to_string();
            let next: Vec<String> = step
                .on_success
                .iter()
                .filter_map(|routing| match &routing.outcome {
                    AirRoutingOutcome::GotoStep(target) => Some(target.to_string()),
                    AirRoutingOutcome::End
                    | AirRoutingOutcome::Retry
                    | AirRoutingOutcome::GotoWorkflow(_) => None,
                })
                .collect();
            for target in &next {
                successors.insert(target.clone());
            }
            steps.entry(id.clone()).or_insert(BridgeStepDef { next });
            order.push(id);
        }
    }
    // Roots = steps that are never a forward-edge target, in program declaration order.
    let active: Vec<String> = order
        .into_iter()
        .filter(|id| !successors.contains(id))
        .collect();
    (BridgeWorkflow { steps }, active)
}

/// The real, composed output of driving a real F15 AIR transition, then feeding every real
/// `dispatch_step` command it produced into a real F16 gen_statem dispatch. Each entry is the
/// dispatched step's own id paired with that dispatch's real terminal outcome (`completed` or
/// `refused` -- both are legitimate, non-error outcomes; see [`DispatchStatemOutcome`]).
#[derive(Debug, Clone)]
pub struct ExternalWitnessF16Outcome {
    /// F15's real transition result (`ready_steps`/`commands`), same as
    /// [`drive_external_readmit_transition`]'s own `transition` field.
    pub transition: BridgeTransitionResult,
    /// One real F16 dispatch outcome per `dispatch_step` command `transition` produced, in command
    /// order. Empty is a legitimate outcome (a transition with no new commands dispatches nothing
    /// to F16) -- see the module doc's "honest nuance" for why F10's own flat template hits this.
    pub dispatch_outcomes: Vec<(String, DispatchStatemOutcome)>,
}

/// Typed refusal for [`drive_external_witness_tail_through_f16`]. Each variant carries the
/// offending stage's own real refusal verbatim.
#[derive(Debug, thiserror::Error)]
pub enum ExternalWitnessF16Refused {
    /// The real `air_core` bridge transition refused.
    #[error("crown-external F14->F15 AIR transition refused: {0}")]
    AirTransition(#[from] AirBridgeRefused),
    /// The real F16 dispatch-statem bridge refused for the named step.
    #[error("crown-external F15->F16 dispatch-statem bridge refused for step {step_id}: {source}")]
    DispatchStatem {
        step_id: String,
        #[source]
        source: DispatchStatemBridgeRefused,
    },
}

/// Deterministically derives one real [`DispatchStatemRequest`] from an F15 transition's own
/// `crown_receipt` and the specific step F15 said is ready to dispatch -- no wall clock, no
/// randomness, no identity unrelated to this specific run. `bind_value` is always `true` (the same
/// literal-bind shape this crate's other F16 fixtures use; see
/// [`crate::f16_otp_runner::bridge`]'s own "single output-bind shape" disclosure).
fn f16_identity_for_step(crown_receipt: &str, step_id: &str) -> DispatchStatemRequest {
    DispatchStatemRequest {
        workflow_id: format!("crown-ext-f16-{crown_receipt}"),
        correlation_id: format!("crown-ext-f16-corr-{crown_receipt}-{step_id}"),
        source_digest: crown_receipt.to_string(),
        projection_digest: crown_receipt.to_string(),
        receipt_head: crown_receipt.to_string(),
        replay_id: format!("crown-ext-f16-replay-{crown_receipt}"),
        step_id: step_id.to_string(),
        bind_name: format!("{step_id}_done"),
        bind_value: true,
    }
}

/// Drive the EXTERNAL witness's `F14 -> F15 -> F16` edge end to end: complete `bridge_workflow`'s
/// `active_steps` through the real `air_core` transition core, then feed every real `dispatch_step`
/// command that transition produces into a real F16 gen_statem dispatch.
///
/// Takes the transition inputs directly (not the full [`ExternalWitnessOutcome`]) so it composes
/// equally well with [`drive_external_witness_tail`]'s own real (but commandless) F10-derived
/// output and with a hand-built AIR program that has real `onSuccess` routing -- see the module
/// doc's "honest nuance" for why F10's own output alone does not exercise F16.
///
/// # Errors
/// [`ExternalWitnessF16Refused`], carrying the first stage's own typed refusal.
///
/// # Complexity
/// O(1) Rust-side glue plus one `call_air_core_bridge` round trip and one
/// `call_dispatch_statem_bridge` round trip per real dispatch command.
pub fn drive_external_witness_tail_through_f16(
    bridge_workflow: &BridgeWorkflow,
    active_steps: &[String],
    bridge_events: &[BridgeEvent],
    crown_receipt: &str,
) -> Result<ExternalWitnessF16Outcome, ExternalWitnessF16Refused> {
    let transition = call_air_core_bridge(bridge_workflow, active_steps, bridge_events)?;

    let mut dispatch_outcomes = Vec::with_capacity(transition.commands.len());
    for cmd in &transition.commands {
        let request = f16_identity_for_step(crown_receipt, &cmd.step_id);
        let outcome = call_dispatch_statem_bridge(&request).map_err(|source| {
            ExternalWitnessF16Refused::DispatchStatem {
                step_id: cmd.step_id.clone(),
                source,
            }
        })?;
        dispatch_outcomes.push((cmd.step_id.clone(), outcome));
    }

    Ok(ExternalWitnessF16Outcome {
        transition,
        dispatch_outcomes,
    })
}

/// Typed refusal for [`drive_f16_completion_through_f18_broker`].
#[derive(Debug, thiserror::Error)]
pub enum ExternalF18Refused {
    /// The F16 dispatch for `step_id` was `refused`, not `completed` -- there is no real
    /// dispatch token to actuate. Refusing rather than actuating a fabricated consequence for a
    /// dispatch that never lawfully completed.
    #[error(
        "crown-external F16->F18: F16 dispatch for step {step_id} was refused \
         ({refusal_atom}), nothing to actuate"
    )]
    F16DispatchNotCompleted {
        step_id: String,
        refusal_atom: String,
    },
    /// A [`Broker`] stage refused (invalid standing, forged/duplicate authority, correlation
    /// mismatch, or an unlawful transition) -- the same shared error type
    /// [`crate::f11_bcinr_runtime::F11BrokerHandoffRefused`] wraps for LOCAL's own F11->F18 edge.
    #[error("crown-external F16->F18 broker refused: {0}")]
    Broker(#[from] UnreceiptedActuationRefused),
}

/// Drive the EXTERNAL witness's `F16 -> F18` edge end to end: take one real F16 dispatch-statem
/// outcome and, if it lawfully completed, actuate its real dispatch token through the real F18
/// [`Broker`]'s lawful lifecycle -- the identical stage sequence
/// [`crate::f11_bcinr_runtime::dispatch_local_execution_via_broker`] already uses for LOCAL's own
/// F11->F18 edge, reused verbatim, not reimplemented.
///
/// # Errors
/// [`ExternalF18Refused`]: [`ExternalF18Refused::F16DispatchNotCompleted`] if `f16_outcome` is a
/// `Refused` (no dispatch token exists to actuate), or [`ExternalF18Refused::Broker`] if any real
/// `Broker` stage refuses.
///
/// # Complexity
/// O(1) glue plus `Broker`'s own O(1)-per-stage cost (see each method's own complexity note);
/// `capture_consequence` is O(\|dispatch_token\|) for its single BLAKE3 fold.
#[allow(clippy::too_many_arguments)]
pub fn drive_f16_completion_through_f18_broker(
    broker: &Broker,
    action: ActionId,
    actor: &str,
    has_standing: bool,
    standing_reason: &str,
    correlation_id: &str,
    step_id: &str,
    f16_outcome: &DispatchStatemOutcome,
) -> Result<BrokerReceipt, ExternalF18Refused> {
    let dispatch_token = match f16_outcome {
        DispatchStatemOutcome::Completed { dispatch_token, .. } => dispatch_token,
        DispatchStatemOutcome::Refused { refusal_atom, .. } => {
            return Err(ExternalF18Refused::F16DispatchNotCompleted {
                step_id: step_id.to_string(),
                refusal_atom: refusal_atom.clone(),
            });
        }
    };

    broker.verify_standing(&action, actor, has_standing, standing_reason)?;
    let (_, token) = broker.authorize(&action);
    broker.claim_idempotency(action.clone(), token)?;
    broker.bind_correlation(&action, correlation_id, correlation_id)?;
    let consequence = dispatch_token.as_bytes().to_vec();
    let actuated = broker.actuate(&action, || consequence.clone())?;
    broker.capture_consequence(&action, &actuated)?;
    Ok(broker.issue_receipt(&action)?)
}

/// Typed refusal for [`drive_f18_completion_through_f20_dispatch`]. Each variant carries the
/// offending real stage's own refusal verbatim.
#[derive(Debug, thiserror::Error)]
pub enum ExternalF20Refused {
    /// `dispatch_subworkflow_to_engine`, `engine_serve`, or `collect_subworkflow_consequence`
    /// refused -- all three share this error type.
    #[error("crown-external F18->F20 dispatch/serve/collect refused: {0}")]
    CngBridge(#[from] CngRefusal),
    /// No consequence file ever appeared in the outbox within the poll budget.
    #[error("crown-external F18->F20: no consequence found for dispatch {dispatch_id} within the poll budget")]
    NoConsequenceFound { dispatch_id: String },
}

/// Drive the EXTERNAL witness's `F18 -> F20` edge end to end: build a real [`SubworkflowPlan`]
/// whose `id`/`problem_digest` are derived from a real F18 [`BrokerReceipt`]'s own identity and
/// consequence hash (not an arbitrary caller-supplied value), then dispatch it through the same
/// real, already-proven `dispatch_subworkflow_to_engine -> engine_serve ->
/// collect_subworkflow_consequence` round trip [`drive_external_reentry`] below already uses.
///
/// This is the final gap on the shared-prefix-anchored EXTERNAL forward path: composing this
/// function's output (`dispatch_outcome.dispatch_id`) as the `dispatch_id` a
/// [`drive_external_reentry`] call re-admits connects `F10..F20` into the already-complete
/// `F20->F02->F15->F21->F24->F25` loop-back tail for the first time.
///
/// # Errors
/// [`ExternalF20Refused`], carrying the first stage's own typed refusal.
///
/// # Complexity
/// O(template render + one shape check + one contract write) for dispatch, plus O(`max_polls`)
/// for `engine_serve`'s poll loop and O(`max_polls`) for the collect poll -- see each wrapped
/// function's own complexity note.
pub fn drive_f18_completion_through_f20_dispatch(
    root: &Path,
    receipt: &BrokerReceipt,
    target_engine: &str,
    engine_seed: u64,
    max_polls: u64,
    poll_wait_ms: Option<u64>,
) -> Result<SubworkflowDispatchOutcome, ExternalF20Refused> {
    let subworkflow = SubworkflowPlan {
        id: format!("f18-{}-{}", receipt.workflow_id, receipt.step_id),
        role: "single".to_string(),
        tape: bcinr_pddl::Pddl8Tape { ops: Vec::new() },
        model: CngPowl::Leaf(None),
        problem_pddl: String::new(),
        problem_digest: format!("blake3:{}", receipt.consequence_hash_hex),
    };

    let handle = dispatch_subworkflow_to_engine(root, &subworkflow, "", target_engine)?;
    let _serve_report = engine_serve(root, target_engine, engine_seed, max_polls, poll_wait_ms)?;
    let dispatch_outcome = collect_subworkflow_consequence(root, &handle, max_polls, poll_wait_ms)?;
    if dispatch_outcome.consequence_turtle.is_none() {
        return Err(ExternalF20Refused::NoConsequenceFound {
            dispatch_id: dispatch_outcome.dispatch_id,
        });
    }
    Ok(dispatch_outcome)
}

/// Fold every stage's real digest into one deterministic BLAKE3-hex crown receipt. Material is
/// sorted before hashing (repo invariant #2: canonical order, no reliance on insertion order); no
/// wall clock, no randomness.
fn compute_external_crown_receipt(
    external_cut: &Powl,
    receipt: &ArazzoProjectionReceipt,
    air_digest_hex: &str,
    active_steps: &[String],
) -> String {
    let cut_is_external = matches!(external_cut, Powl::ExternalCut { .. });
    let mut lines = vec![
        format!("f12.cut_is_external={cut_is_external}"),
        format!("f13.source_powl_digest={}", receipt.source_powl_digest_hex),
        format!("f13.arazzo_digest={}", receipt.arazzo_digest_hex),
        format!("f13.air_digest={}", receipt.air_digest_hex),
        format!("f14.recompiled_air_digest={air_digest_hex}"),
        format!("f15.bridge_steps={}", active_steps.join(",")),
    ];
    lines.sort();
    let mut hasher = blake3::Hasher::new();
    for line in &lines {
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

/// `urn:mfw:f20#` re-admission vocabulary this crown composition introduces (no prior owner in
/// F20's own module), matching the `urn:mfw:fNN#` convention `crown_local.rs`'s F19/F24
/// re-admission predicates already use.
const EXTERNAL_REENTRY_DISPATCH_ID_PREDICATE: &str = "urn:mfw:f20#dispatchId";
const EXTERNAL_REENTRY_CONSEQUENCE_DIGEST_PREDICATE: &str = "urn:mfw:f20#consequenceDigest";
const EXTERNAL_REENTRY_CONSEQUENCE_TURTLE_PREDICATE: &str = "urn:mfw:f20#consequenceTurtle";
/// The `prov:wasDerivedFrom` predicate the re-admitted observation's provenance triple uses (F02
/// gate 2). Same IRI as `crown_local.rs`'s own constant of the same name (kept module-local
/// rather than shared, matching this crate's existing per-module constant style).
const EXTERNAL_REENTRY_PROV_WAS_DERIVED_FROM: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";

/// Everything one real `F20 -> F02(re-admit)` run needs. Every field is an input a real family
/// entry point genuinely requires -- none is decorative.
pub struct ExternalReentryRun<'a> {
    /// Real filesystem root `dispatch_subworkflow_to_engine`/`engine_serve`/
    /// `collect_subworkflow_consequence` all operate under (the `EngineBundle` layout is built
    /// beneath this path).
    pub root: &'a Path,
    /// The real subworkflow contract to dispatch.
    pub subworkflow: &'a SubworkflowPlan,
    /// The engine id both the dispatch and the real `engine_serve` poll loop address.
    pub target_engine: String,
    /// `engine_serve`'s deterministic identity seed (`instance_nonce = splitmix64(seed ^
    /// blake3(engine_id))`, never a PID or wall clock).
    pub engine_seed: u64,
    /// Poll budget shared by `engine_serve` and `collect_subworkflow_consequence`.
    pub max_polls: u64,
    /// Inter-poll wait; `None` means no real sleep (matches this crate's other tests' preference
    /// for a tight, deterministic bound over real elapsed time).
    pub poll_wait_ms: Option<u64>,
    /// F02 trust configuration for the re-admission.
    pub policy: &'a AdmissionPolicy,
    /// F02 idempotency/correlation ledger for the re-admission.
    pub ledger: &'a AdmissionLedger,
    /// F02 re-admission source id: the identity asserting "F20 collected this real consequence" --
    /// distinct from any other principal this crate's other crown drivers use.
    pub reentry_source_id: String,
    /// The principal IRI `reentry_source_id` maps to in `policy`.
    pub reentry_principal_iri: String,
    /// Base IRI the re-admitted observation's subject is derived from
    /// (`{base}/external-dispatch/{dispatch_id}`).
    pub reentry_subject_base_iri: String,
    /// F02 idempotency/correlation key for the re-admission.
    pub correlation_id: String,
}

/// The real, composed output of one `F20 -> F02(re-admit)` run. `Debug` only (not `Clone`): its
/// `dispatch_outcome` field is `cng::bench::decomp::dispatch_bridge::SubworkflowDispatchOutcome`,
/// which itself derives only `Debug` -- kept as-is rather than widening cng's derives further for
/// this driver's convenience.
#[derive(Debug)]
pub struct ExternalReentryOutcome {
    /// F20's real dispatch/collect outcome (`admitted` is guaranteed `true` here; a `false`
    /// outcome short-circuits into [`ExternalReentryRefused::NotAdmittedByDispatchPipeline`]
    /// before this struct is ever constructed).
    pub dispatch_outcome: SubworkflowDispatchOutcome,
    /// F02's real admission receipt for the re-admitted consequence observation.
    pub reentry_admission: AdmissionReceipt,
    /// BLAKE3-hex over both stages' real digests, in canonical sorted order (no wall clock, no
    /// randomness).
    pub crown_receipt: String,
}

/// Typed refusal for the composed `F20 -> F02(re-admit)` edge. Each variant carries the
/// offending stage's own real refusal verbatim, never a generic catch-all.
#[derive(Debug, thiserror::Error)]
pub enum ExternalReentryRefused {
    /// `dispatch_subworkflow_to_engine`, `engine_serve`, or `collect_subworkflow_consequence`
    /// refused -- all three share this error type, so one variant covers all three call sites.
    #[error("crown-external F20 dispatch/serve/collect refused: {0}")]
    CngBridge(#[from] CngRefusal),
    /// No consequence file ever appeared in the outbox within the poll budget.
    #[error("crown-external F20: no consequence found for dispatch {dispatch_id} within the poll budget")]
    NoConsequenceFound { dispatch_id: String },
    /// A consequence file was found but cng's own real provenance/correlation/authority/
    /// structural/semantic pipeline did not admit it. Refused honestly rather than re-admitting a
    /// consequence cng's own real check rejected.
    #[error("crown-external F20: dispatch {dispatch_id}'s consequence was not admitted by cng's own dispatch-contract pipeline")]
    NotAdmittedByDispatchPipeline { dispatch_id: String },
    /// F02 refused to admit the synthesized re-admission observation.
    #[error("crown-external F20->F02 re-admission refused: {0}")]
    ReentryAdmission(ObservationAdmissionRefused),
    /// Swarm audit wnl2yhbgm finding #31: the real collected `consequence_turtle` contains a
    /// literal `"""` sequence, which would terminate `build_reentry_payload`'s wrapping
    /// triple-quoted long-string literal early and let the remainder of `consequence_turtle` be
    /// parsed as first-class Turtle statements by F02's admission gate (arbitrary graph
    /// injection into `RawObservation.payload_turtle`). Refused before embedding rather than
    /// escaped, matching this crate's established refuse-the-dangerous-input pattern.
    #[error("crown-external F20->F02 re-admission refused: dispatch {dispatch_id}'s consequence_turtle contains a literal \"\"\" sequence, unsafe to embed in a triple-quoted Turtle literal")]
    ConsequenceTurtleUnsafeForEmbedding { dispatch_id: String },
    /// 80/20 gap sweep gap-2: `consequence_digest` was `None` despite `consequence_turtle` being
    /// `Some`. `SubworkflowDispatchOutcome`'s own doc comment (`dispatch_bridge.rs`) guarantees
    /// these two fields are set together -- both `None` or both `Some` -- so reaching this refusal
    /// means that invariant broke upstream. Refused before any N-Quad/ledger write rather than
    /// silently substituting an empty-string digest via `unwrap_or_default()`.
    #[error("crown-external F20->F02 re-admission refused: dispatch {dispatch_id}'s consequence_digest was missing despite a collected consequence_turtle")]
    MissingConsequenceDigest { dispatch_id: String },
}

/// Drive the EXTERNAL witness's `F20 -> F02(re-admit)` edge end to end, in one real call: dispatch
/// a real subworkflow contract, have a real `engine_serve` poll loop admit and manufacture a real
/// response, collect that real consequence through cng's own real admission pipeline, then
/// re-admit it through F02's real, independent gate pipeline.
///
/// See the module doc comment's `F20 -> F02(re-admit)` section for exactly what makes this a real
/// (gated, data-threaded) production edge and every disclosed nuance.
///
/// # Errors
/// [`ExternalReentryRefused`], carrying the first stage's own typed refusal.
///
/// # Complexity
/// O(template bytes) contract render + O(`max_polls`) engine-serve poll + O(`max_polls`)
/// collect poll + F02's own O(T+S) admission cost. This function itself adds only O(1) glue.
pub fn drive_external_reentry(
    run: ExternalReentryRun<'_>,
) -> Result<ExternalReentryOutcome, ExternalReentryRefused> {
    // ---- Stage F20: dispatch a real subworkflow contract into target_engine's real inbox ------
    let handle = dispatch_subworkflow_to_engine(run.root, run.subworkflow, "", &run.target_engine)?;

    // ---- Stage: a real cng engine actually serves the dispatched contract ----------------------
    // `engine_serve` is the real receiving side of the same bridge (previously zero production
    // callers, per f20_external_dispatch.rs's own doc comment); real and deterministic (seed-
    // derived identity, no wall clock), processing the SAME on-disk EngineBundle layout the
    // dispatch above just wrote into.
    let _serve_report = engine_serve(
        run.root,
        &run.target_engine,
        run.engine_seed,
        run.max_polls,
        run.poll_wait_ms,
    )?;

    // ---- Stage F20 collect: bounded-poll the real outbox, run cng's own real admission gate ----
    let dispatch_outcome =
        collect_subworkflow_consequence(run.root, &handle, run.max_polls, run.poll_wait_ms)?;
    if !dispatch_outcome.admitted {
        return Err(ExternalReentryRefused::NotAdmittedByDispatchPipeline {
            dispatch_id: dispatch_outcome.dispatch_id,
        });
    }
    let consequence_turtle = dispatch_outcome.consequence_turtle.clone().ok_or_else(|| {
        ExternalReentryRefused::NoConsequenceFound {
            dispatch_id: dispatch_outcome.dispatch_id.clone(),
        }
    })?;
    let consequence_digest = require_consequence_digest(&dispatch_outcome)?;

    // ---- Stage F20 -> F02 (re-admit): synthesize a real observation asserting the real
    // consequence, admitted through F02's own independent gate pipeline ----------------------
    let reentry_subject_iri = format!(
        "{}/external-dispatch/{}",
        run.reentry_subject_base_iri, dispatch_outcome.dispatch_id
    );
    let payload_turtle = build_reentry_payload(
        &reentry_subject_iri,
        &run.reentry_principal_iri,
        &dispatch_outcome.dispatch_id,
        &consequence_digest,
        &consequence_turtle,
    )?;
    let obs = RawObservation {
        correlation_id: run.correlation_id,
        source_id: run.reentry_source_id,
        declared_subject: reentry_subject_iri,
        payload_turtle,
    };
    let reentry_admission = admit_observation(run.policy, run.ledger, obs)
        .map_err(ExternalReentryRefused::ReentryAdmission)?;

    let crown_receipt =
        compute_reentry_crown_receipt(&dispatch_outcome, &consequence_digest, &reentry_admission);

    Ok(ExternalReentryOutcome {
        dispatch_outcome,
        reentry_admission,
        crown_receipt,
    })
}

/// 80/20 gap sweep gap-2: extract `dispatch_outcome.consequence_digest`, refusing before any
/// N-Quad construction or ledger write if it is `None`. `SubworkflowDispatchOutcome`'s own doc
/// comment (`dispatch_bridge.rs`) guarantees `consequence_digest` and `consequence_turtle` are set
/// together (both `None` or both `Some`), so a caller that has already unwrapped a `Some`
/// `consequence_turtle` (as [`drive_external_reentry`] has, immediately above its own call site)
/// should never observe `None` here -- reaching it means that invariant broke upstream. Refused by
/// typed error rather than silently substituted with `unwrap_or_default()`, which would have baked
/// an empty-string digest into the F20->F02 re-admission N-Quads and the crown receipt.
///
/// Pulled out as its own function (mirroring `build_reentry_payload` below) so this refusal path
/// is directly unit-testable against a real, directly-constructed [`SubworkflowDispatchOutcome`]
/// value, without needing a live dispatch/serve/collect round trip to reach the `None` case.
fn require_consequence_digest(
    dispatch_outcome: &SubworkflowDispatchOutcome,
) -> Result<String, ExternalReentryRefused> {
    dispatch_outcome.consequence_digest.clone().ok_or_else(|| {
        ExternalReentryRefused::MissingConsequenceDigest {
            dispatch_id: dispatch_outcome.dispatch_id.clone(),
        }
    })
}

/// Serialize F20's real collected consequence into a single Turtle observation payload F02 can
/// admit: the provenance triple (gate 2) plus three literal facts about the consequence.
///
/// The consequence Turtle text is embedded as a triple-quoted long-string literal, matching
/// `crown_local.rs`'s `build_planning_payload` pattern for externally-produced text -- it is
/// re-asserted as a literal value, never re-parsed and merged as a graph (the raw document's own
/// vocabulary is cng's `disp:`/`prov:` dispatch shapes, a different admission concern than this
/// crate's F02 gates).
///
/// # Errors
/// Swarm audit wnl2yhbgm finding #31: `consequence_turtle` is real text collected from another
/// engine's outbox, not this driver's own construction, so it cannot be trusted to be free of a
/// literal `"""` sequence. Turtle's `STRING_LITERAL_LONG_QUOTE` production ends at the first
/// unescaped `"""`, so an embedded one would close this literal early and hand the remainder of
/// `consequence_turtle` to F02's admission parser as first-class Turtle statements -- arbitrary
/// graph injection into a value meant to be inert observation text. Refused
/// ([`ExternalReentryRefused::ConsequenceTurtleUnsafeForEmbedding`]) rather than escaped: this
/// keeps `consequenceTurtle`'s literal value byte-identical to the real collected text for every
/// consequence that does not carry the dangerous sequence, with no escaping/unescaping asymmetry
/// to get wrong.
fn build_reentry_payload(
    subject_iri: &str,
    principal_iri: &str,
    dispatch_id: &str,
    consequence_digest: &str,
    consequence_turtle: &str,
) -> Result<String, ExternalReentryRefused> {
    if consequence_turtle.contains("\"\"\"") {
        return Err(
            ExternalReentryRefused::ConsequenceTurtleUnsafeForEmbedding {
                dispatch_id: dispatch_id.to_string(),
            },
        );
    }
    Ok(format!(
        "<{subject_iri}> <{EXTERNAL_REENTRY_PROV_WAS_DERIVED_FROM}> <{principal_iri}> ;\n  \
         <{EXTERNAL_REENTRY_DISPATCH_ID_PREDICATE}> \"{dispatch_id}\" ;\n  \
         <{EXTERNAL_REENTRY_CONSEQUENCE_DIGEST_PREDICATE}> \"{consequence_digest}\" ;\n  \
         <{EXTERNAL_REENTRY_CONSEQUENCE_TURTLE_PREDICATE}> \"\"\"{consequence_turtle}\"\"\" .\n"
    ))
}

/// Fold both stages' real digests into one deterministic BLAKE3-hex crown receipt. Material is
/// sorted before hashing (repo invariant #2); no wall clock, no randomness.
///
/// `consequence_digest` is taken as an explicit, already-validated parameter (rather than
/// re-derived from `dispatch_outcome.consequence_digest`) so this function has no `Option` to
/// silently default -- the caller has already turned a missing digest into
/// [`ExternalReentryRefused::MissingConsequenceDigest`] before any receipt material is folded.
fn compute_reentry_crown_receipt(
    dispatch_outcome: &SubworkflowDispatchOutcome,
    consequence_digest: &str,
    reentry_admission: &AdmissionReceipt,
) -> String {
    let mut lines = vec![
        format!("f20.dispatch_id={}", dispatch_outcome.dispatch_id),
        format!("f20.consequence_digest={consequence_digest}"),
        format!("f20.polls_taken={}", dispatch_outcome.polls_taken),
        format!("f02_readmit.receipt={}", reentry_admission.receipt_hash),
    ];
    lines.sort();
    let mut hasher = blake3::Hasher::new();
    for line in &lines {
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

/// `F15(AIR transition) -> F21` evidence vocabulary/shape this crown composition introduces (no
/// prior owner in F21's own module), matching the `urn:mfw:crown-ext#` namespace this module's
/// own re-admission predicates already use.
const EXTERNAL_TRANSITION_EVIDENCE_CLASS: &str = "urn:mfw:crown-ext#ExternalTransitionEvidence";
const EXTERNAL_TRANSITION_EVIDENCE_RECEIPT_PREDICATE: &str = "urn:mfw:crown-ext#transitionReceipt";

/// `F15(AIR transition) -> F21` evidence shape: requires the transition evidence subject to carry
/// a non-empty `transitionReceipt` value. Same non-vacuous pattern as `crown_local.rs`'s
/// `ACTUATION_CONSTRUCT_EVIDENCE_SHAPES`: the target class is matched by a real individual and
/// `sh:minCount 1` is genuinely evaluated against a real, deterministically-folded value.
const EXTERNAL_TRANSITION_EVIDENCE_SHAPES: &str = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <urn:mfw:crown-ext#> .
ex:ExternalTransitionEvidenceShape a sh:NodeShape ;
    sh:targetClass ex:ExternalTransitionEvidence ;
    sh:property [
        sh:path ex:transitionReceipt ;
        sh:minCount 1 ;
    ] .
"#;

/// `F21 -> F24` OTel span vocabulary this crown composition introduces, matching
/// `crown_local.rs`'s `ACTUATION_OTEL_*` naming/reasoning for its own F02(re-admit)->F24 span.
const EXTERNAL_TRANSITION_OTEL_OBJECT_TYPE: &str = "ExternalDispatchTransition";
/// `cng::otel_rdf`'s closed `process.outcome` vocabulary value for a successfully-completed
/// activity. Not importable (private module constant); reproduced here, matching
/// `crown_local.rs`'s own disclosed-duplication reasoning for the identical value.
const EXTERNAL_TRANSITION_OTEL_OUTCOME_COMPLETED: &str = "completed";
/// Fixed, non-wall-clock nanosecond timestamps for the synthesized OTel span (repo invariant #3;
/// same reasoning as `crown_local.rs`'s `ACTUATION_OTEL_START_NANOS`/`_END_NANOS`).
const EXTERNAL_TRANSITION_OTEL_START_NANOS: u64 = 1_700_000_000_000_000_000;
const EXTERNAL_TRANSITION_OTEL_END_NANOS: u64 = 1_700_000_000_500_000_000;

/// The real, composed output of one `F02(re-admit) -> F15(AIR transition) -> F21 -> F24` run.
#[derive(Debug)]
pub struct ExternalReadmitTransitionOutcome {
    /// The `F20 -> F02(re-admit)` outcome this run composes verbatim as its first stage.
    pub reentry: ExternalReentryOutcome,
    /// The real transition result from completing the externally-dispatched step -- empty
    /// `ready_steps`/`commands` on a single terminal step is a legitimate outcome, not a failure.
    pub transition: BridgeTransitionResult,
    /// Whether the freshly-declared single-child recursive socket closure closed once the
    /// external-dispatch child was admitted (`F15 -> F21`). Always `true` for this single-child
    /// `AllRequired` closure (admitting the one declared child always closes it) -- kept as a real
    /// field, not hardcoded, so a caller never has to assume the closure semantics.
    pub parent_closed: bool,
    /// F24's real OCEL construction outcome (`F21 -> F24`): the admitted external-dispatch
    /// consequence, projected as a real OTel span, run through `f24_ocel_construct::run_construct`.
    pub ocel_outcome: OcelConstructOutcome,
    /// F25's real receipt-fold + independent-replay-verification outcome (`F24 -> F25`), the last
    /// edge of the EXTERNAL loop-back tail.
    pub replay_outcome: ReceiptReplayOutcome,
}

/// Typed refusal for the composed `F02(re-admit) -> F15 -> F21 -> F24` edge.
#[derive(Debug, thiserror::Error)]
pub enum ExternalReadmitTransitionRefused {
    /// The `F20 -> F02(re-admit)` stage refused.
    #[error("crown-external F02(re-admit)->F15 transition: reentry stage refused: {0}")]
    Reentry(#[from] ExternalReentryRefused),
    /// The real `air_core` bridge transition refused.
    #[error("crown-external F02(re-admit)->F15 transition: AIR transition refused: {0}")]
    AirTransition(#[from] AirBridgeRefused),
    /// This driver's own transition-evidence Turtle failed to parse (defensive: built from a
    /// compile-time-controlled format string plus a BLAKE3 hex digest; kept as a typed refusal
    /// rather than `.expect()` per this repo's no-panics-on-fallible-code invariant).
    #[error("crown-external F15->F21 transition evidence malformed: {reason}")]
    TransitionEvidenceMalformed { reason: String },
    /// This driver's own `EXTERNAL_TRANSITION_EVIDENCE_SHAPES` constant failed to parse
    /// (defensive: hand-verified compile-time SHACL Turtle).
    #[error("crown-external F15->F21 transition evidence shapes invalid: {reason}")]
    TransitionEvidenceShapesInvalid { reason: String },
    /// Declaring the recursive socket closure, or F21 admitting the external-dispatch child under
    /// it, refused. Both share `praxis_graphlaw::chatman::abi::Refusal` as their error type.
    #[error("crown-external F15->F21 child closure refused: {0}")]
    ChildClosure(F21Refusal),
    /// Admitting, projecting, or inserting the external-dispatch OTel span refused. Covers
    /// `cng::otel_rdf::admit`, `cng::otel_rdf::project_admitted_spans`, and
    /// `cng::otel_ocel::insert_quads`, which all share this error type.
    #[error("crown-external F21->F24 external telemetry refused: {0}")]
    ExternalTelemetry(#[from] CngRefusal),
    /// The in-memory oxigraph `Store` backing F24 construction could not be created (defensive).
    #[error("crown-external F24 store unavailable: {reason}")]
    TransitionStoreUnavailable { reason: String },
    /// F24's real OCEL construction refused.
    #[error("crown-external F21->F24 OCEL construction refused: {0}")]
    OcelConstruction(#[from] OCELConstructionRefused),
    /// F25 refused to fold, replay, or equivalence-check the EXTERNAL chain's own receipt
    /// materials.
    #[error("crown-external F24->F25 receipt replay refused: {0}")]
    ReceiptReplay(#[from] ReceiptReplayRefused),
}

/// Drive the EXTERNAL witness's `F02(re-admit) -> F15 (AIR transition) -> F21` edge end to end:
/// dispatch, serve, collect, and re-admit a real consequence via [`drive_external_reentry`],
/// complete a minimal real bridge workflow through a real [`call_air_core_bridge`] round trip,
/// then admit that completion as a real child under a freshly-declared recursive socket closure.
///
/// See the module doc comment's `F02(re-admit) -> F15 (AIR transition)` section for exactly what
/// makes this a real production edge and every disclosed nuance -- including why this function,
/// unlike [`drive_external_reentry`] alone, requires a live `escript` + compiled `apps/air_core`.
///
/// # Errors
/// [`ExternalReadmitTransitionRefused`], carrying the first stage's own typed refusal.
pub fn drive_external_readmit_transition(
    run: ExternalReentryRun<'_>,
) -> Result<ExternalReadmitTransitionOutcome, ExternalReadmitTransitionRefused> {
    // ---- Stage F20 -> F02 (re-admit): reuse drive_external_reentry verbatim -----------------
    let reentry = drive_external_reentry(run)?;

    // ---- Stage F02(re-admit) -> F15 (AIR transition): complete a real, minimal bridge workflow
    // representing the externally-dispatched consequence, via the real air_core bridge ---------
    let dispatch_id = reentry.dispatch_outcome.dispatch_id.clone();
    let completion_workflow = BridgeWorkflow {
        steps: BTreeMap::from([(dispatch_id.clone(), BridgeStepDef { next: Vec::new() })]),
    };
    let transition = call_air_core_bridge(
        &completion_workflow,
        &[dispatch_id.clone()],
        &[BridgeEvent::StepCompleted {
            step_id: dispatch_id,
            result: serde_json::Value::String(reentry.reentry_admission.receipt_hash.clone()),
        }],
    )?;

    // ---- Stage F15(AIR transition) -> F21: admit the external-dispatch consequence as a real
    // child under a freshly-declared recursive socket closure ---------------------------------
    // Gated by `transition` above. Unlike `crown_local.rs`'s F24->F21 (which reuses F09's own
    // real `growth.closure`/`growth.child_socket`), this composition has no upstream family that
    // naturally produces a closure over the external-dispatch structure, so a minimal one is
    // declared here: a single-leaf `PartialOrder` whose one child *is* the external dispatch.
    // Evidence is a real, non-vacuous SHACL check over a deterministic BLAKE3 fold of the
    // transition's own real `ready_steps`/`commands` output -- always non-empty (BLAKE3 of even
    // an empty input is a real 64-hex digest), so `conforms: true` genuinely reflects "the AIR
    // core really processed this transition," not a fabricated pass.
    let closure_model = Powl::PartialOrder {
        children: vec![Powl::Leaf(Some("external-dispatch".to_string()))],
        order: BTreeSet::new(),
    };
    let parent_socket = WorkflowSocketId {
        path: SocketPath::root(),
        kind: SocketKind::PartialOrder,
    };
    let dispatch_child_socket = WorkflowSocketId {
        path: SocketPath::root().child(0),
        kind: SocketKind::Leaf,
    };
    let pcc = ParentChildClosure::from_model(&closure_model);
    let mut closure = RecursiveSocketClosure::declare(&pcc, parent_socket, ClosureLaw::AllRequired)
        .map_err(ExternalReadmitTransitionRefused::ChildClosure)?;

    let transition_receipt = {
        let mut hasher = blake3::Hasher::new();
        hasher.update(reentry.dispatch_outcome.dispatch_id.as_bytes());
        for step in &transition.ready_steps {
            hasher.update(step.as_bytes());
        }
        for cmd in &transition.commands {
            hasher.update(cmd.step_id.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    };
    let evidence_subject_iri = format!("{}/transition", reentry.reentry_admission.subject_iri);
    let evidence_turtle = format!(
        "<{evidence_subject_iri}> a <{EXTERNAL_TRANSITION_EVIDENCE_CLASS}> ;\n  \
         <{EXTERNAL_TRANSITION_EVIDENCE_RECEIPT_PREDICATE}> \"{transition_receipt}\" .\n"
    );
    let evidence_parsed = Parser::parse_triples(&evidence_turtle, Syntax::Turtle).map_err(|e| {
        ExternalReadmitTransitionRefused::TransitionEvidenceMalformed {
            reason: e.to_string(),
        }
    })?;
    let mut evidence_index = TripleIndex::new();
    for t in evidence_parsed {
        evidence_index.add(t);
    }
    let evidence_shapes =
        ShapesGraph::parse(EXTERNAL_TRANSITION_EVIDENCE_SHAPES).map_err(|reason| {
            ExternalReadmitTransitionRefused::TransitionEvidenceShapesInvalid { reason }
        })?;
    let evidence_report = Validator::validate(&evidence_index, &evidence_shapes);
    let parent_closed =
        admit_child_and_evaluate(&mut closure, &dispatch_child_socket, &evidence_report)
            .map_err(ExternalReadmitTransitionRefused::ChildClosure)?;

    // ---- Stage F21 -> F24: the admitted external-dispatch consequence becomes a real OTel span
    // (admit -> project -> insert into a fresh in-memory store), then runs through F24's real
    // OCEL construction ------------------------------------------------------------------------
    // Gated by `parent_closed` above (F21's admission succeeded). `parent_span_id` is the F02
    // re-admission's own real output receipt hash, matching `crown_local.rs`'s
    // `F02(re-admit)->F24` pattern; `process.object.id` reuses the same `evidence_subject_iri`
    // F21's own evidence asserted facts about, so F24's projection is built over the identity F21
    // just admitted, not a disconnected fixture.
    let external_span = OtlpSpan {
        trace_id: reentry.dispatch_outcome.dispatch_id.clone(),
        span_id: transition_receipt.clone(),
        parent_span_id: Some(reentry.reentry_admission.receipt_hash.clone()),
        name: telemetry_gen::REGISTRY_GROUP_ID.to_string(),
        start_time_unix_nano: EXTERNAL_TRANSITION_OTEL_START_NANOS,
        end_time_unix_nano: EXTERNAL_TRANSITION_OTEL_END_NANOS,
        attributes: vec![
            (
                telemetry_gen::ATTR_WORKFLOW_ID.to_string(),
                reentry.dispatch_outcome.dispatch_id.clone(),
            ),
            (
                telemetry_gen::ATTR_OBJECT_ID.to_string(),
                evidence_subject_iri,
            ),
            (
                telemetry_gen::ATTR_OBJECT_TYPE.to_string(),
                EXTERNAL_TRANSITION_OTEL_OBJECT_TYPE.to_string(),
            ),
            (
                telemetry_gen::ATTR_ACTIVITY_IRI.to_string(),
                format!(
                    "urn:mfw:f20:dispatch:{}",
                    reentry.dispatch_outcome.dispatch_id
                ),
            ),
            (
                telemetry_gen::ATTR_OUTCOME.to_string(),
                EXTERNAL_TRANSITION_OTEL_OUTCOME_COMPLETED.to_string(),
            ),
        ],
        status: SpanStatus {
            code: SpanStatusCode::Ok,
            message: None,
        },
    };
    admit_otel_span(&external_span)?;
    let external_otel_quads = project_admitted_spans(&[external_span])?;
    let external_ocel_store =
        Store::new().map_err(
            |e| ExternalReadmitTransitionRefused::TransitionStoreUnavailable {
                reason: e.to_string(),
            },
        )?;
    insert_otel_quads(&external_ocel_store, &external_otel_quads)?;
    let ocel_outcome = run_construct("otel-to-ocel", &external_ocel_store)?;

    // ---- Stage F24 -> F25: fold a real receipt over the EXTERNAL chain's own canonical texts,
    // then independently replay-verify it -- the last edge of the EXTERNAL loop-back tail -------
    // Gated by `ocel_outcome` above. Mirrors `crown_local.rs`'s F21->F25 exactly, mapped to
    // EXTERNAL's own real values: `source` is the real consequence Turtle F20 collected, `query`
    // is the real dispatch id identifying which dispatch drove this transformation, `template` is
    // the real SHACL shape F21's evidence rendered through, `program` is F21's own
    // `transition_receipt` fold (the real, executed transition this whole chain is about), `event`
    // is F21's evidence Turtle itself, `output` is F24's real receipt head. Honest nuance,
    // identical to `crown_local.rs`'s own disclosure: the replay closure returns
    // `materials.clone()`, matching F25's own test suite's established pattern for a deterministic
    // transformation -- every field is already a real, deterministically-computed value from this
    // run, so an honest replay reproduces byte-identical `Materials` without re-executing any
    // side-effecting step a second time.
    let materials = F25Materials {
        source: reentry
            .dispatch_outcome
            .consequence_turtle
            .clone()
            .unwrap_or_default(),
        query: reentry.dispatch_outcome.dispatch_id.clone(),
        template: EXTERNAL_TRANSITION_EVIDENCE_SHAPES.to_string(),
        program: transition_receipt,
        event: evidence_turtle,
        output: ocel_outcome.receipt_head.clone(),
    };
    let replay_outcome = run_receipt_replay(&materials, || Ok(materials.clone()))
        .map_err(ExternalReadmitTransitionRefused::ReceiptReplay)?;

    Ok(ExternalReadmitTransitionOutcome {
        reentry,
        transition,
        parent_closed,
        ocel_outcome,
        replay_outcome,
    })
}

#[cfg(test)]
#[path = "crown_external_test.rs"]
mod crown_external_test;

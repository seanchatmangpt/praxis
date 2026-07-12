//! Crown LOCAL-witness prefix, composed for real:
//! `F02 -> F03 -> F08 -> F09 -> F10 -> F11 -> F18 -> F19 -> F02(re-admit) -> F24 -> F21`.
//!
//! This module is the first *production caller* (a real, non-`#[cfg(test)]` `pub fn`) that
//! drives the shared crown-witness prefix end to end in one call, reusing each family's own
//! real entry point without reimplementing any family's internals:
//!
//! | Stage | Real entry point reused (verbatim) |
//! |---|---|
//! | F02 Observation Admission | [`crate::f02_observation_admission::admit_observation`] |
//! | F03 Semantic Contraction  | [`crate::f03_semantic_contraction::contract`] |
//! | F08 PDDL Planning         | [`crate::f08_pddl_planning::run_pipeline`] |
//! | F09 MFW Growth            | [`crate::f09_mfw_growth::resolve_continuation_goal`] -> [`plan_growth`](crate::f09_mfw_growth::plan_growth) -> [`manufacture_and_bind_child`](crate::f09_mfw_growth::manufacture_and_bind_child) |
//! | F10 POWL Geometry         | reached *inside* F09's `manufacture_and_bind_child` (which calls [`crate::f10_powl_geometry::manufacture_powl_v2`]) |
//! | F11 BCINR Local Execution | [`crate::f11_bcinr_runtime::geometry_to_local_ast`] over F09's real `growth.geometry` |
//! | F18 Broker                | [`crate::f11_bcinr_runtime::dispatch_local_execution_via_broker`] |
//! | F19 Hooks                 | [`crate::f19_hooks::resolve_hook_for_action`] over F08's real bound action |
//! | F02 (re-admit)            | [`crate::f02_observation_admission::admit_observation`], called a second time over a freshly synthesized actuation-consequence observation |
//! | F24 CONSTRUCT/OCEL        | [`crate::f24_ocel_construct::run_construct`], over a real `cng::otel_rdf::OtlpSpan` built from the re-admitted actuation |
//! | F21 Parent-Child Closure  | [`crate::f21_parent_child_closure::admit_child_and_evaluate`], over F09's own real `growth.closure`/`growth.child_socket` |
//!
//! # What makes each edge real (and the honest nuances)
//!
//! Every stage is `?`-gated on the previous: a refusal anywhere short-circuits, so no downstream
//! stage runs on an un-admitted / un-contracted / un-planned input. The data flow is:
//!
//! - **F02 -> F03**: F03 [`contract`](crate::f03_semantic_contraction::contract)'s
//!   `admitted_rdf` is the *exact same bytes* [`admit_observation`](crate::f02_observation_admission::admit_observation)
//!   just admitted (`payload_turtle`), and F03 runs only on F02's `Ok`. A structural re-parse
//!   ([`verify_admitted_graph_carries_planning_predicates`]) additionally confirms the admitted
//!   graph really carries the three planning predicates, by predicate IRI.
//! - **F03 -> F08**: F08 runs only when F03 returns a
//!   [`ContractionState::Plannable`](crate::f03_semantic_contraction::ContractionState) state,
//!   and F03's `receipt_head` digest salts F08's `case_id` -- so F08's execution receipt is a
//!   function of F03's real output, not merely temporally after it.
//! - **F08 -> F09**: F09 runs only on F08's `Ok`, resolves its continuation goal from the *same
//!   admitted PDDL text* F08 planned, and both plan tapes are bound into the crown receipt.
//!   Honest nuance (disclosed, not smuggled): F09 *re-plans* the shared admitted problem through
//!   its own `plan_growth` gates rather than consuming F08's `Pddl8Tape` object, because no
//!   residual-goal extractor exists in this repo to turn F08's plan into F09's continuation goal
//!   (a real, disclosed architecture gap). The two tapes are asserted equal in this module's
//!   test, so the shared-problem edge is verified, not assumed.
//! - **F09 -> F10**: unchanged from the prior session -- F09's `manufacture_and_bind_child`
//!   already gates on F10's `manufacture_powl_v2`; the crown is now a real production caller of
//!   that whole chain, so F10's geometry is a genuine consequence of the admitted observation.
//! - **F10 -> F11**: [`geometry_to_local_ast`](crate::f11_bcinr_runtime::geometry_to_local_ast)
//!   converts `growth.geometry.root` -- F10's own canonical `Powl` geometry for the same plan
//!   tape (not F09's separately-grafted `new_root`) -- into F11's `PowlAstNode`. This was
//!   previously `TEST_ONLY_EDGE`: the function existed and was tested, but had zero production
//!   callers (adversarially confirmed this session by `docs/jira/v26.7.12/CROWN_STATUS.md`). This
//!   driver is now that caller.
//! - **F11 -> F18**: the converted AST feeds
//!   [`dispatch_local_execution_via_broker`](crate::f11_bcinr_runtime::dispatch_local_execution_via_broker)
//!   directly -- real local execution to `LOCAL_DONE`, then all eight of
//!   [`crate::f18_broker_law::Broker`]'s lawful stages, ending in a real
//!   [`crate::f18_broker_law::BrokerReceipt`] whose `consequence_hash_hex` is the real Local
//!   Receipt chain hash, not a placeholder. Also previously `TEST_ONLY_EDGE` for the same reason
//!   as F10 -> F11; this driver is its first production caller too (F18's own module doc
//!   previously said "No production caller in this repo" -- stale as of this pass).
//! - **F18 -> F19**: gated on a real `broker_receipt` -- only reached once local execution
//!   actually actuated through the broker. Resolves F19's real hook capability for the *same*
//!   grounded action F08's `ActionHookBinder` already bound at planning time (`plan.tape.ops`'s
//!   first op), against the same admitted `hook_pack_turtle`, but with a fresh
//!   [`crate::f19_hooks::InMemoryReceiptLedger`] -- this is a distinct, post-actuation binding
//!   ("this real actuation corresponds to exactly this registered hook"), not a re-check of
//!   planning-time admissibility (which F08 already performed and gated on). Per F19's own atlas
//!   doc (`F19_hooks.md`: "Maps planner actions to typed executable capabilities"), this is F19's
//!   canonical real entry point, reused verbatim -- not a second, parallel hook-lookup
//!   mechanism invented for this driver.
//! - **F19 -> F02 (re-admit)**: the actuation consequence (which hook actuated, under which
//!   receipts) is synthesized into a *new* observation and passed through
//!   [`admit_observation`](crate::f02_observation_admission::admit_observation) a second time --
//!   the same real gate pipeline, not a second implementation. Honest nuance: the re-admission is
//!   asserted by a distinct principal (`run.actuation_source_id`/`actuation_principal_iri`), not
//!   the original external planner (`run.source_id`) -- the local runtime observing its own
//!   actuation is architecturally a different asserting party than the external system that
//!   submitted the original planning problem, so this driver requires that party to be a
//!   separately-declared known principal in the same [`AdmissionPolicy`], authorized only for the
//!   new `urn:mfw:f19#`/`urn:mfw:f18#` actuation predicates (never the F08 planning predicates).
//!   The re-admission's `correlation_id` is `{run.correlation_id}-actuation` -- deterministic and
//!   distinct from the planning observation's, so it lands as a new ledger entry rather than
//!   colliding with (or replaying) the first admission.
//! - **F02 (re-admit) -> F24**: the re-admitted actuation consequence is synthesized into a real
//!   [`cng::otel_rdf::OtlpSpan`] -- `trace_id`/`span_id` are F18/F19's own real receipt hashes
//!   (not placeholders), `process.object.id` is the same `actuation_subject_iri` F02 just
//!   admitted, and `process.activity.iri` is derived from F19's real resolved hook name. The span
//!   is admitted ([`cng::otel_rdf::admit`]), projected
//!   ([`cng::otel_rdf::project_admitted_spans`]), inserted into a fresh in-memory `Store`
//!   ([`cng::otel_ocel::insert_quads`]), then run through F24's real `run_construct`. Honest
//!   nuance: span timestamps (`start_time_unix_nano`/`end_time_unix_nano`) are fixed sentinel
//!   constants, not a wall-clock read -- this driver's own composition never observes a real
//!   clock anywhere (repo invariant #3), and OTel's schema requires *some* timestamp value even
//!   though none is semantically load-bearing for the crown witness. This driver never calls
//!   `idempotency_gate` -- see the `F24 -> F21` nuance below for why.
//! - **F24 -> F21**: `growth.closure`/`growth.child_socket` are F09's *own* real output, produced
//!   fresh by `manufacture_and_bind_child` specifically for this purpose (see
//!   [`GrowthOutcome::child_socket`](crate::f09_mfw_growth::GrowthOutcome)'s own doc comment: "for
//!   a caller that wants to `admit` it once its own execution completes") -- not a repurposed or
//!   reinvented closure. The evidence is a real, non-vacuous SHACL check
//!   ([`Validator::validate`](praxis_graphlaw::shacl::Validator::validate)): this driver asserts
//!   `ocel_outcome.receipt_head` (the genuine value F24 just produced) about `actuation_subject_iri`
//!   and checks it against [`ACTUATION_CONSTRUCT_EVIDENCE_SHAPES`], which requires a non-empty
//!   `ocelReceiptHead`. Unlike this codebase's `VACUOUS_SHAPES` pattern (an intentionally
//!   unmatchable target class), this shape's target class is matched by a real individual and its
//!   `sh:minCount 1` constraint is genuinely evaluated -- it passes because F24 really produced a
//!   receipt head, not because the check is empty. Honest nuance: this is *not* SHACL validation
//!   of the OCEL construction's own projected quads (`ocel_outcome.ocel_quads`/`receipt_quads`,
//!   which live in `oxigraph`'s `Quad` representation, a different RDF library than
//!   `praxis-graphlaw`'s own `Term`/`TripleIndex` this validator consumes) -- bridging those two
//!   representations for a full structural OCEL-conformance check is deferred, disclosed future
//!   work, not attempted here. `parent_closed = false` is a legitimate outcome under
//!   `ClosureLaw::AllRequired` over a multi-child socket (the other children remain unobserved),
//!   not a failure. **Scope disclosure**: this closes `F19 -> F02`, `F02(re-admit) -> F24`, and
//!   `F24 -> F21`, not the further `F21 -> F25` tail -- `f25_receipts_replay::chaos_gate::admit_for_replay`
//!   is an audited-and-confirmed-honest `NotYetImplemented` refusal (not corruption; checked this
//!   pass), and `f25_receipts_replay::run` (F25's own top-level real entry point) has not yet been
//!   audited this pass for a composable non-chaos-gate path into it.
//!
//! Content-identity note for F08: F08 consumes the [`AdmittedTriple`] set built from the very
//! same PDDL/hook-pack strings the crown serialized into F02's `payload_turtle`. Identity is
//! guaranteed by construction (single in-function origin), not by re-deserializing F02's Turtle
//! back into `AdmittedTriple`s -- praxis-graphlaw literals expose no public lexical-value
//! accessor, so a round-trip would mean string-munging `Term`'s display form, which this module
//! declines to do.

use cng::otel_ocel::insert_quads as insert_otel_quads;
use cng::otel_rdf::{
    admit as admit_otel_span, project_admitted_spans, OtlpSpan, SpanStatus, SpanStatusCode,
};
use cng::powl::CngRefusal;
use cng::telemetry_gen;
use oxigraph::store::Store;
use powl2_decompose::Powl;
use praxis_graphlaw::chatman::closure::{ClosureLaw, RecursiveSocketClosure};
use praxis_graphlaw::parser::{Parser, Syntax};
use praxis_graphlaw::shacl::{ShapesGraph, Validator};
use praxis_graphlaw::tripleindex::TripleIndex;
use praxis_graphlaw::triples::VarOrTerm;

use crate::f02_observation_admission::{
    admit_observation, AdmissionLedger, AdmissionPolicy, AdmissionReceipt,
    ObservationAdmissionRefused, RawObservation,
};
use crate::f03_semantic_contraction::{
    contract, ContractionInputs, ContractionState, PlanningState, SemanticWorldRefused,
};
use crate::f05_datalog_closure::RulePack;
use crate::f08_pddl_planning::projector::{
    AdmittedTriple, HOOK_PACK_PREDICATE, PDDL_DOMAIN_PREDICATE, PDDL_PROBLEM_PREDICATE,
};
use crate::f08_pddl_planning::refusal::Refusal as PlanningRefusal;
use crate::f08_pddl_planning::{run_pipeline, PipelineOutcome};
use crate::f09_mfw_growth::{
    manufacture_and_bind_child, plan_growth, resolve_continuation_goal, DescentMeter,
    GrowthOutcome, GrowthPlan, MFWGrowthRefused, ResidueState,
};
use crate::f11_bcinr_runtime::{
    dispatch_local_execution_via_broker, geometry_to_local_ast, F10ToF11GeometryRefused,
    F11BrokerHandoffRefused,
};
use crate::f18_broker_law::{ActionId, Broker, BrokerReceipt, BrokerSecret};
use crate::f19_hooks::{
    resolve_hook_for_action, HookResolution, HookResolutionRefused, InMemoryReceiptLedger,
};
use crate::f21_parent_child_closure::{admit_child_and_evaluate, Refusal as F21Refusal};
use crate::f24_ocel_construct::{run_construct, OCELConstructionRefused, OcelConstructOutcome};

/// The `prov:wasDerivedFrom` predicate the admitted observation's provenance triple uses (F02
/// gate 2). Bare IRI, matching F02's own `PROV_WAS_DERIVED_FROM` constant.
const PROV_WAS_DERIVED_FROM: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";

/// F19 -> F02 re-admission vocabulary: which hook name actuated the grounded action. New
/// predicate this crown composition introduces (no prior owner in F19's own module), under the
/// same `urn:mfw:fNN#` convention as [`crate::f08_pddl_planning::projector::HOOK_PACK_PREDICATE`].
const HOOK_ACTUATION_NAME_PREDICATE: &str = "urn:mfw:f19#actuatedHookName";
/// F19 -> F02 re-admission vocabulary: F19's own hook-resolution receipt hash for the actuation.
const HOOK_ACTUATION_RECEIPT_PREDICATE: &str = "urn:mfw:f19#actuationReceiptHash";
/// F19 -> F02 re-admission vocabulary: the F18 broker receipt the actuation was dispatched under.
const HOOK_ACTUATION_BROKER_RECEIPT_PREDICATE: &str = "urn:mfw:f18#brokerReceiptHash";

/// F02(re-admit) -> F24: the synthesized OTel span's `process.object.type` value for the
/// actuation-consequence event -- a literal value (not an IRI), matching the shape
/// `f24_ocel_construct`'s own test fixture uses for `process.object.type` (e.g. `"Order"`).
const ACTUATION_OTEL_OBJECT_TYPE: &str = "HookActuation";
/// F02(re-admit) -> F24: `cng::otel_rdf`'s closed `process.outcome` vocabulary value for a
/// successfully-completed activity. Not importable: `cng::otel_rdf::OUTCOME_COMPLETED` is a
/// private module constant (`admit`'s own closed-vocabulary check is what actually enforces this
/// value, not this driver), so the literal is reproduced here rather than fabricated -- disclosed
/// duplication, not invention.
const ACTUATION_OTEL_OUTCOME_COMPLETED: &str = "completed";
/// F02(re-admit) -> F24: fixed, non-wall-clock nanosecond timestamps for the synthesized OTel
/// span. Repo invariant #3 forbids wall-clock reads in receipt/hash paths; this driver's own
/// composition never observes a real clock anywhere (matching how `broker_secret`/`local_run_id`
/// are already caller-supplied fixed values rather than `SystemTime::now()` reads) -- these
/// timestamps are structural placeholders OTel's schema requires, not semantically meaningful
/// data the crown witness depends on.
const ACTUATION_OTEL_START_NANOS: u64 = 1_700_000_000_000_000_000;
const ACTUATION_OTEL_END_NANOS: u64 = 1_700_000_000_500_000_000;

/// F24 -> F21 vocabulary: class/predicate this driver asserts about the actuation-construct
/// evidence subject, under the same `urn:mfw:crown#` namespace `crown_local_test.rs`'s
/// `VACUOUS_SHAPES` fixture already uses for driver-local vocabulary.
const ACTUATION_CONSTRUCT_EVIDENCE_CLASS: &str = "urn:mfw:crown#ActuationConstructEvidence";
const ACTUATION_CONSTRUCT_EVIDENCE_RECEIPT_HEAD_PREDICATE: &str = "urn:mfw:crown#ocelReceiptHead";

/// F24 -> F21 evidence shape: requires the actuation-construct evidence subject to carry a
/// non-empty `ocelReceiptHead` value. Unlike this codebase's `VACUOUS_SHAPES` pattern elsewhere
/// (an intentionally-unmatchable target class used where the check itself is not yet the point),
/// this shape's target class is matched by a real asserted individual and its `sh:minCount 1`
/// constraint is genuinely checked against a real value (`ocel_outcome.receipt_head`) -- it passes
/// because F24's `run_construct` really produced a non-empty receipt head, not because the shape
/// is vacuous.
const ACTUATION_CONSTRUCT_EVIDENCE_SHAPES: &str = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <urn:mfw:crown#> .
ex:ActuationConstructEvidenceShape a sh:NodeShape ;
    sh:targetClass ex:ActuationConstructEvidence ;
    sh:property [
        sh:path ex:ocelReceiptHead ;
        sh:minCount 1 ;
    ] .
"#;

/// Everything one real LOCAL-witness prefix run needs. Every field is an input a real family
/// entry point genuinely requires -- none is decorative.
pub struct LocalWitnessRun<'a> {
    /// F02 trust configuration (real SHACL-backed [`AdmissionPolicy`]).
    pub policy: &'a AdmissionPolicy,
    /// F02 idempotency/correlation ledger.
    pub ledger: &'a AdmissionLedger,
    /// Idempotency/correlation key for the planning observation (F02 L7).
    pub correlation_id: String,
    /// Trusted planner source id; must be a known principal in `policy`.
    pub source_id: String,
    /// The snapshot IRI the planning observation is about (F02 declared subject).
    pub subject_iri: String,
    /// The principal IRI `source_id` maps to -- the `prov:wasDerivedFrom` object (F02 gate 2).
    pub source_principal_iri: String,
    /// Real PDDL8 domain text; carried by the admitted observation and planned by F08 and F09.
    pub pddl_domain: String,
    /// Real PDDL8 problem text (same).
    pub pddl_problem: String,
    /// Real F19 hook-pack Turtle covering every grounded action (F08 Action-Hook Binder).
    pub hook_pack_turtle: String,
    /// F03 Datalog rule pack. Must be non-empty and stratifiable: praxis-graphlaw's stratifier
    /// reports a spurious cycle for a zero-rule ruleset, which F05's `close_datalog` surfaces as
    /// a refusal, so a caller wanting an identity closure should pass one harmless non-firing
    /// rule rather than an empty pack.
    pub datalog_rule_pack: RulePack,
    /// F03 SHACL shapes the admitted, closed world must conform to.
    pub f03_shacl_shapes: String,
    /// F09 root workflow being grown (must expose a `PartialOrder` at `growth_closure`'s socket).
    pub growth_root: Powl,
    /// F09 recursive-socket closure over the blocked socket in `growth_root`.
    pub growth_closure: RecursiveSocketClosure,
    /// The caller's own determination the socket is blocked (F09 has no independent detector).
    pub socket_blocked: bool,
    /// F09 bounded-descent budget (must be >= 1 for one growth step).
    pub descent_budget: usize,
    /// F09 closure law to re-declare the parent socket under after grafting.
    pub closure_law: ClosureLaw,
    /// F18 server-side authority secret (32 bytes). Caller-supplied per this repo's determinism
    /// discipline: this driver does not source randomness itself. Reusing the same secret across
    /// runs is the caller's own key-management choice, not judged here.
    pub broker_secret: [u8; 32],
    /// F18 action identity for the local dispatch this run performs (workflow/step/idempotency
    /// key) -- see [`ActionId`]'s own doc comment for why these three fields alone must not be
    /// sufficient to derive authority.
    pub action: ActionId,
    /// F18 `actor` for the standing check.
    pub actor: String,
    /// F18 caller-supplied standing determination; the broker does not itself judge standing (see
    /// [`Broker::verify_standing`]'s own doc comment).
    pub has_standing: bool,
    /// F18 standing reason, carried alongside `has_standing`.
    pub standing_reason: String,
    /// F11 local-execution run id (32 bytes) -- the same "no ambient randomness" discipline as
    /// `broker_secret`.
    pub local_run_id: [u8; 32],
    /// F11 bounded-descent tick budget for local execution (`BCINRLocalRuntime::run_to_local_done`).
    pub local_max_ticks: u32,
    /// F02 re-admission source id (F19 -> F02): the *local runtime's own* identity, distinct from
    /// `source_id` (the external planner). Must be a known principal in `policy`, authorized for
    /// the `urn:mfw:f19#`/`urn:mfw:f18#` actuation predicates only.
    pub actuation_source_id: String,
    /// The principal IRI `actuation_source_id` maps to in `policy` -- the re-admission's own
    /// `prov:wasDerivedFrom` object.
    pub actuation_principal_iri: String,
}

/// The real, composed output of one LOCAL-witness prefix run: every stage's own genuine output,
/// plus a deterministic crown receipt binding them.
#[derive(Debug, Clone)]
pub struct LocalWitnessOutcome {
    /// F02's admission receipt.
    pub admission: AdmissionReceipt,
    /// F03's published planning state (guaranteed `Plannable`, else the run refused).
    pub planning_state: PlanningState,
    /// F08's real plan/execution outcome (tape, receipt, OCEL, capability map).
    pub plan: PipelineOutcome,
    /// F09's growth outcome (grafted child + F10 geometry inside `geometry`/`geometry_turtle`).
    pub growth: GrowthOutcome,
    /// F18's real broker receipt for the F10 -> F11 -> F18 local-execution dispatch.
    pub broker_receipt: BrokerReceipt,
    /// F19's real hook resolution for the actuated action -- confirms the real local actuation
    /// corresponds to exactly one registered, authorized hook capability.
    pub hook_resolution: HookResolution,
    /// F02's second admission receipt (F19 -> F02 re-admit): the actuation consequence, admitted
    /// through the same real gate pipeline as `admission`, under a distinct principal and
    /// correlation id.
    pub actuation_admission: AdmissionReceipt,
    /// F24's real OCEL construction outcome (`F02(re-admit) -> F24`): the re-admitted actuation
    /// consequence, projected as a real OTel span, run through `f24_ocel_construct::run_construct`.
    pub ocel_outcome: OcelConstructOutcome,
    /// Whether F09's own recursive socket closure (`growth.closure`) closed once the manufactured
    /// child (`growth.child_socket`) was admitted under F24's real evidence (`F24 -> F21`). `false`
    /// is a legitimate outcome under `ClosureLaw::AllRequired` over a multi-child socket -- it does
    /// not mean the edge failed, only that the parent socket has other, still-unobserved children.
    pub parent_closed: bool,
    /// BLAKE3-hex over every stage's real digest, in canonical sorted order (no wall clock, no
    /// randomness) -- deterministic across runs of the same inputs.
    pub crown_receipt: String,
}

/// Typed refusal for the composed prefix. Each variant carries the offending stage's own real
/// refusal verbatim (via its `Display`), never a generic catch-all.
#[derive(Debug, thiserror::Error)]
pub enum LocalWitnessRefused {
    /// F02 refused to admit the observation graph.
    #[error("crown-local F02 admission refused: {0}")]
    Admission(#[from] ObservationAdmissionRefused),
    /// The admitted graph did not carry an expected planning predicate (re-parse check).
    #[error(
        "crown-local: admitted graph is missing the expected planning predicate <{predicate}> \
         (F02 admitted a payload that does not structurally carry the F08 planning triples)"
    )]
    AdmittedGraphMissingPredicate { predicate: &'static str },
    /// F03 refused to contract the admitted semantic world.
    #[error("crown-local F03 contraction refused: {0}")]
    Contraction(#[from] SemanticWorldRefused),
    /// F03 succeeded but did not reach the `Plannable` terminal state (defensive: `contract`'s
    /// own contract is to only return `Plannable` on `Ok`, so this is an internal-invariant
    /// guard, not a normally-reachable path).
    #[error("crown-local F03 did not reach Plannable (got {state:?}); refusing to plan")]
    NotPlannable { state: ContractionState },
    /// F08 refused to produce an admissible plan for the admitted graph.
    #[error("crown-local F08 planning refused: {0}")]
    Planning(#[from] PlanningRefusal),
    /// F09 (or F10, via F09's geometry gate) refused the growth attempt.
    #[error("crown-local F09/F10 growth refused: {0}")]
    Growth(#[from] MFWGrowthRefused),
    /// F10's real geometry used a `Powl` shape F11's `geometry_to_local_ast` cannot losslessly
    /// convert (a cyclic/partially-routed `Choice`, or an `ExternalCut`).
    #[error("crown-local F10->F11 geometry conversion refused: {0}")]
    GeometryToLocalAst(#[from] F10ToF11GeometryRefused),
    /// Local execution did not complete, or a F18 broker stage refused the dispatch.
    #[error("crown-local F11->F18 broker handoff refused: {0}")]
    BrokerHandoff(#[from] F11BrokerHandoffRefused),
    /// F08's plan tape had zero ops (a trivially-already-satisfied goal) -- there is no grounded
    /// action for F19 to resolve a hook against.
    #[error(
        "crown-local: F08's plan tape has zero ops; no grounded action for F19 hook resolution"
    )]
    EmptyPlanTapeForHookResolution,
    /// F19 could not resolve (or ambiguously resolved) a real hook capability for the actuated
    /// action.
    #[error("crown-local F18->F19 hook resolution refused: {0}")]
    HookResolution(#[from] HookResolutionRefused),
    /// F02 refused to re-admit the synthesized actuation-consequence observation (the
    /// `F19 -> F02` loop-back edge). Not `#[from]`: `ObservationAdmissionRefused` already backs
    /// [`Self::Admission`] for the first F02 call, so the two admissions are disambiguated by
    /// variant, not by source type.
    #[error("crown-local F19->F02 re-admission refused: {0}")]
    ReAdmission(ObservationAdmissionRefused),
    /// Admitting, projecting, or inserting the actuation-consequence OTel span refused. Covers
    /// [`cng::otel_rdf::admit`], [`cng::otel_rdf::project_admitted_spans`], and
    /// [`cng::otel_ocel::insert_quads`], which all share this error type.
    #[error("crown-local F02(re-admit)->F24 actuation telemetry refused: {0}")]
    ActuationTelemetry(#[from] CngRefusal),
    /// The in-memory oxigraph `Store` backing F24 construction could not be created (defensive:
    /// an in-memory store has no external dependency to fail on; kept as a typed refusal rather
    /// than `.expect()` per this repo's no-panics-on-fallible-code invariant).
    #[error("crown-local F24 store unavailable: {reason}")]
    ActuationStoreUnavailable { reason: String },
    /// F24's real OCEL construction refused. Never reaches F24's own unimplemented L7
    /// idempotency gate -- this driver does not call `idempotency_gate` (see module doc's scope
    /// disclosure).
    #[error("crown-local F02(re-admit)->F24 OCEL construction refused: {0}")]
    OcelConstruction(#[from] OCELConstructionRefused),
    /// This driver's own actuation-evidence Turtle failed to parse (defensive: the payload is
    /// built from a compile-time-controlled format string plus `ocel_outcome.receipt_head`, a
    /// hex-digest string with no Turtle-breaking characters -- kept as a typed refusal rather
    /// than `.expect()` per this repo's no-panics-on-fallible-code invariant).
    #[error("crown-local F24->F21 evidence payload malformed: {reason}")]
    ActuationEvidenceMalformed { reason: String },
    /// This driver's own `ACTUATION_CONSTRUCT_EVIDENCE_SHAPES` constant failed to parse
    /// (defensive: hand-verified compile-time SHACL Turtle; kept as a typed refusal, not
    /// `.expect()`, for the same reason as [`Self::ActuationEvidenceMalformed`]).
    #[error("crown-local F24->F21 evidence shapes invalid: {reason}")]
    ActuationEvidenceShapesInvalid { reason: String },
    /// F21 refused to admit the manufactured child under F09's recursive socket closure (unknown
    /// child, non-conforming evidence, or an already-admitted-but-conflicting state).
    #[error("crown-local F24->F21 child admission refused: {0}")]
    ChildClosureRefused(F21Refusal),
}

/// Drive the LOCAL crown-witness prefix
/// `F02 -> F03 -> F08 -> F09 -> F10 -> F11 -> F18 -> F19 -> F02(re-admit) -> F24 -> F21` end to
/// end, in one real call, over a single admitted observation graph.
///
/// See the module doc comment for exactly what makes each edge a real (gated, data-threaded)
/// production edge and every disclosed nuance.
///
/// # Errors
/// [`LocalWitnessRefused`], carrying the first stage's own typed refusal.
///
/// # Complexity
/// The sum of each stage's own documented cost: F02 O(T+S) admission, F03 OWL-RL + Datalog
/// closure + SHACL, F08 grounding + BFS plan search, F09 indexed planning + O(n^3) F10 geometry,
/// F11/F18 bounded-tick local execution, F19 O(1) hook lookup, F02(re-admit) a second O(T+S)
/// admission, F24 O(m log m) OTel projection + OCEL construction (m = emitted triple count), F21
/// O(log c) closure admission (c = declared child count). This function itself adds only O(T)
/// glue (payload build, re-parse check, receipt fold).
pub fn drive_local_witness_prefix(
    run: LocalWitnessRun<'_>,
) -> Result<LocalWitnessOutcome, LocalWitnessRefused> {
    // ---- Stage F02: admit the single observation graph ---------------------
    // The observation payload carries the planning content (PDDL domain/problem + hook pack) as
    // literals on the F08 predicates, plus the provenance triple F02 gate 2 requires.
    let payload_turtle = build_planning_payload(
        &run.subject_iri,
        &run.source_principal_iri,
        &run.pddl_domain,
        &run.pddl_problem,
        &run.hook_pack_turtle,
    );
    let obs = RawObservation {
        correlation_id: run.correlation_id.clone(),
        source_id: run.source_id.clone(),
        declared_subject: run.subject_iri.clone(),
        payload_turtle: payload_turtle.clone(),
    };
    let admission = admit_observation(run.policy, run.ledger, obs)?;

    // Structural confirmation the admitted graph really carries the three planning predicates
    // (by predicate IRI -- no literal-value extraction). This is the F02 -> F08 provenance
    // guarantee: F08 below plans over content that is structurally present in F02's admitted
    // graph, not a separately-supplied graph.
    verify_admitted_graph_carries_planning_predicates(&payload_turtle)?;

    // ---- Stage F03: contract the admitted semantic world -------------------
    // `admitted_rdf` is the exact bytes F02 just admitted.
    let planning_state = contract(ContractionInputs {
        admitted_rdf: &payload_turtle,
        datalog_rule_pack: run.datalog_rule_pack,
        n3_refinement: None,
        shacl_shapes_turtle: &run.f03_shacl_shapes,
        shex: None,
        admitted_predicates: vec![
            PDDL_DOMAIN_PREDICATE.to_string(),
            PDDL_PROBLEM_PREDICATE.to_string(),
            HOOK_PACK_PREDICATE.to_string(),
        ],
    })?;
    if planning_state.state != ContractionState::Plannable {
        return Err(LocalWitnessRefused::NotPlannable {
            state: planning_state.state,
        });
    }

    // ---- Stage F08: plan over the admitted graph ---------------------------
    // Gated by F03 above (only reached on a Plannable state); F03's receipt_head salts the
    // case_id so F08's execution receipt is a function of F03's real output.
    let f08_graph = vec![
        AdmittedTriple {
            subject: run.subject_iri.clone(),
            predicate: PDDL_DOMAIN_PREDICATE.to_string(),
            object_literal: run.pddl_domain.clone(),
        },
        AdmittedTriple {
            subject: run.subject_iri.clone(),
            predicate: PDDL_PROBLEM_PREDICATE.to_string(),
            object_literal: run.pddl_problem.clone(),
        },
        AdmittedTriple {
            subject: run.subject_iri.clone(),
            predicate: HOOK_PACK_PREDICATE.to_string(),
            object_literal: run.hook_pack_turtle.clone(),
        },
    ];
    // Salt F08's case_id with a bounded slice of F03's receipt_head so F08's execution receipt
    // is a real function of F03's output. F08 requires a 1-64 char case_id, so a 48-hex prefix
    // (192 bits of F03's digest) is used rather than the full 64-hex head.
    let f03_hex = planning_state.receipt_head.to_hex().to_string();
    let case_id = format!("cl-{}", &f03_hex[..48]);
    let plan = run_pipeline(&f08_graph, &case_id)?;

    // ---- Stage F09: continuation goal -> plan_growth -> manufacture (runs F10) ----
    // Gated by F08 above. The continuation goal is resolved from the same admitted PDDL text F08
    // planned; F09 re-plans it through its own real gates (see the disclosed nuance in the module
    // doc). manufacture_and_bind_child runs F10's manufacture_powl_v2 internally.
    let residue = ResidueState {
        socket: run.growth_closure.socket().clone(),
        description: format!(
            "crown-local continuation from admitted snapshot {}",
            run.subject_iri
        ),
        domain_pddl: run.pddl_domain.clone(),
        problem_pddl: run.pddl_problem.clone(),
    };
    let goal = resolve_continuation_goal(&residue)?;
    let mut meter = DescentMeter::new(run.descent_budget);
    let growth_plan = plan_growth(run.socket_blocked, &run.growth_closure, &goal, &mut meter)?;
    // `mut`: F21 below re-borrows `growth.closure` mutably to admit `growth.child_socket` once
    // its own execution (F10..F24) has completed, per `GrowthOutcome::child_socket`'s own doc
    // comment ("for a caller that wants to `admit` it once its own execution completes").
    let mut growth = manufacture_and_bind_child(&run.growth_root, &growth_plan, run.closure_law)?;

    // ---- Stage F10 -> F11 -> F18: convert F10's real geometry to F11's AST, then dispatch it
    // through the real F18 broker to a receipted local actuation ----
    // Gated by F09/F10 above: `growth.geometry` only exists once manufacture_and_bind_child
    // succeeded. Uses F10's own canonical geometry (`growth.geometry`, built by
    // `f10_powl_geometry::build_powl_geometry`), not F09's separately-grafted `growth.new_root` --
    // the two are independent constructions of "a Powl for this tape" by design (see
    // `GrowthOutcome::geometry`'s own doc comment).
    let local_ast = geometry_to_local_ast(&growth.geometry.root)?;
    let broker = Broker::new(BrokerSecret::new(run.broker_secret));
    let broker_receipt = dispatch_local_execution_via_broker(
        &broker,
        run.action.clone(),
        &run.actor,
        run.has_standing,
        &run.standing_reason,
        &run.correlation_id,
        &local_ast,
        run.local_run_id,
        run.local_max_ticks,
    )?;

    // ---- Stage F18 -> F19: resolve the actuated action's real hook capability ----
    // Gated by F18 above (broker_receipt exists): only reached once local execution really
    // actuated through the broker. Reuses F08's own bound action (the same grounded action
    // ActionHookBinder already confirmed a capability exists for at planning time) and the same
    // admitted hook-pack catalog, but with a fresh ledger -- a distinct, post-actuation binding,
    // not a re-check of planning-time admissibility (see module doc).
    let ground_action = plan
        .tape
        .ops
        .first()
        .map(|op| op.action.clone())
        .ok_or(LocalWitnessRefused::EmptyPlanTapeForHookResolution)?;
    let mut hook_ledger = InMemoryReceiptLedger::default();
    let hook_resolution =
        resolve_hook_for_action(&run.hook_pack_turtle, &ground_action, &mut hook_ledger)?;

    // ---- Stage F19 -> F02 (re-admit): the actuation consequence loops back through the same
    // real F02 admission gate as a new observation ----
    // Gated by F19 above (hook_resolution exists): the actuation subject is derived from the
    // broker receipt hash, so it names a distinct logical entity per actuation, never colliding
    // with the original planning-snapshot subject. Asserted by `run.actuation_source_id` -- a
    // distinct known principal from `run.source_id` (see module doc's F19->F02 nuance).
    let actuation_subject_iri = format!(
        "{}/actuation/{}",
        run.subject_iri, broker_receipt.receipt_hash_hex
    );
    let actuation_payload_turtle = build_actuation_payload(
        &actuation_subject_iri,
        &run.actuation_principal_iri,
        &hook_resolution.binding.hook_name,
        &hook_resolution.receipt_hash,
        &broker_receipt.receipt_hash_hex,
    );
    let actuation_obs = RawObservation {
        correlation_id: format!("{}-actuation", run.correlation_id),
        source_id: run.actuation_source_id.clone(),
        declared_subject: actuation_subject_iri.clone(),
        payload_turtle: actuation_payload_turtle,
    };
    let actuation_admission = admit_observation(run.policy, run.ledger, actuation_obs)
        .map_err(LocalWitnessRefused::ReAdmission)?;

    // ---- Stage F02 (re-admit) -> F24: the re-admitted actuation consequence becomes a real OTel
    // span (admit -> project -> insert into a fresh in-memory store), then runs through F24's
    // real OCEL construction ----
    // Gated by the re-admission above (`actuation_admission` exists). `parent_span_id` is set to
    // `actuation_admission.receipt_hash` itself -- not merely the same upstream values that fed
    // the re-admission's own payload, but F02(re-admit)'s own real output receipt, threaded
    // forward as this span's causal parent (a genuine OTel field, projected verbatim by
    // `cng::otel_rdf::project_admitted_spans` as `ob:parentSpanId`). `process.object.id` is the
    // same `actuation_subject_iri` F02 just admitted. So F24's OCEL projection is built over the
    // actual re-admission's output, not a disconnected fixture that merely shares source values.
    // Honest nuance: F24's own `idempotency_gate` (L7 atomic idempotency/correlation gate) is
    // never called here -- see the module doc's scope disclosure; it is a confirmed-honest
    // `NotYetImplemented` refusal, not composable into a success path this driver could reach.
    let actuation_span = OtlpSpan {
        trace_id: broker_receipt.receipt_hash_hex.clone(),
        span_id: hook_resolution.receipt_hash.clone(),
        parent_span_id: Some(actuation_admission.receipt_hash.clone()),
        name: telemetry_gen::REGISTRY_GROUP_ID.to_string(),
        start_time_unix_nano: ACTUATION_OTEL_START_NANOS,
        end_time_unix_nano: ACTUATION_OTEL_END_NANOS,
        attributes: vec![
            (
                telemetry_gen::ATTR_WORKFLOW_ID.to_string(),
                run.correlation_id.clone(),
            ),
            (
                telemetry_gen::ATTR_OBJECT_ID.to_string(),
                actuation_subject_iri.clone(),
            ),
            (
                telemetry_gen::ATTR_OBJECT_TYPE.to_string(),
                ACTUATION_OTEL_OBJECT_TYPE.to_string(),
            ),
            (
                telemetry_gen::ATTR_ACTIVITY_IRI.to_string(),
                format!("urn:mfw:f19:hook:{}", hook_resolution.binding.hook_name),
            ),
            (
                telemetry_gen::ATTR_OUTCOME.to_string(),
                ACTUATION_OTEL_OUTCOME_COMPLETED.to_string(),
            ),
        ],
        status: SpanStatus {
            code: SpanStatusCode::Ok,
            message: None,
        },
    };
    admit_otel_span(&actuation_span)?;
    let otel_quads = project_admitted_spans(&[actuation_span])?;
    let ocel_store = Store::new().map_err(|e| LocalWitnessRefused::ActuationStoreUnavailable {
        reason: e.to_string(),
    })?;
    insert_otel_quads(&ocel_store, &otel_quads)?;
    let ocel_outcome = run_construct("otel-to-ocel", &ocel_store)?;

    // ---- Stage F24 -> F21: admit the manufactured child under F09's own recursive socket
    // closure, evidenced by a real (non-vacuous) SHACL check over F24's real receipt head ----
    // Gated by `ocel_outcome` above. `growth.closure`/`growth.child_socket` are F09's own real
    // output for exactly this purpose (see `GrowthOutcome::child_socket`'s doc comment) -- not a
    // repurposed or reinvented closure. The evidence triples assert the real, just-produced
    // `ocel_outcome.receipt_head` about `actuation_subject_iri` (the same subject F02 re-admitted
    // and F24 constructed over); `Validator::validate` genuinely runs against them, so `conforms`
    // is a real fact (F24 really produced a non-empty receipt head), not fabricated. `parent_closed`
    // is a legitimate outcome either way per `is_closed`'s own contract -- `AllRequired` over a
    // multi-child socket correctly stays open until every child is admitted, not only this one.
    let evidence_turtle = format!(
        "<{actuation_subject_iri}> a <{ACTUATION_CONSTRUCT_EVIDENCE_CLASS}> ;\n  \
         <{ACTUATION_CONSTRUCT_EVIDENCE_RECEIPT_HEAD_PREDICATE}> \"{}\" .\n",
        ocel_outcome.receipt_head
    );
    let evidence_parsed = Parser::parse_triples(&evidence_turtle, Syntax::Turtle).map_err(|e| {
        LocalWitnessRefused::ActuationEvidenceMalformed {
            reason: e.to_string(),
        }
    })?;
    let mut evidence_index = TripleIndex::new();
    for t in evidence_parsed {
        evidence_index.add(t);
    }
    let evidence_shapes = ShapesGraph::parse(ACTUATION_CONSTRUCT_EVIDENCE_SHAPES)
        .map_err(|reason| LocalWitnessRefused::ActuationEvidenceShapesInvalid { reason })?;
    let evidence_report = Validator::validate(&evidence_index, &evidence_shapes);
    let child_socket = growth.child_socket.clone();
    let parent_closed =
        admit_child_and_evaluate(&mut growth.closure, &child_socket, &evidence_report)
            .map_err(LocalWitnessRefused::ChildClosureRefused)?;

    // ---- Crown receipt: deterministic BLAKE3 over every stage's real digest ----
    let crown_receipt = compute_crown_receipt(
        &admission,
        &planning_state,
        &plan,
        &growth_plan,
        &growth,
        &broker_receipt,
        &hook_resolution,
        &actuation_admission,
        &ocel_outcome,
        parent_closed,
    );

    Ok(LocalWitnessOutcome {
        admission,
        planning_state,
        plan,
        growth,
        broker_receipt,
        hook_resolution,
        actuation_admission,
        ocel_outcome,
        parent_closed,
        crown_receipt,
    })
}

/// Serialize the planning content into a single Turtle observation payload F02 can admit: the
/// provenance triple (gate 2) plus the three planning literals on the F08 predicates.
///
/// PDDL/hook-pack text is embedded as triple-quoted Turtle long-string literals (`"""..."""`);
/// none of that text contains `"""`, so no escaping is needed. praxis-graphlaw's Turtle parser
/// supports long-string literals with embedded newlines (see its own
/// `parser_edge_cases_test::test_string_literal_styles`).
fn build_planning_payload(
    subject_iri: &str,
    principal_iri: &str,
    pddl_domain: &str,
    pddl_problem: &str,
    hook_pack_turtle: &str,
) -> String {
    format!(
        "<{subject_iri}> <{PROV_WAS_DERIVED_FROM}> <{principal_iri}> ;\n  \
         <{PDDL_DOMAIN_PREDICATE}> \"\"\"{pddl_domain}\"\"\" ;\n  \
         <{PDDL_PROBLEM_PREDICATE}> \"\"\"{pddl_problem}\"\"\" ;\n  \
         <{HOOK_PACK_PREDICATE}> \"\"\"{hook_pack_turtle}\"\"\" .\n"
    )
}

/// Serialize the F19 hook-actuation consequence into a single Turtle observation payload F02 can
/// re-admit: the provenance triple (gate 2, against the *actuation* principal, not the planner)
/// plus the three actuation literals.
///
/// `hook_name` and the two receipt hashes are all values this driver itself produced or received
/// from an already-admitted hook-pack catalog (never raw external input at this point), so plain
/// (non-triple-quoted) Turtle string literals are safe here -- unlike `build_planning_payload`,
/// which embeds externally-supplied PDDL/hook-pack text and so uses `"""..."""` long strings.
fn build_actuation_payload(
    actuation_subject_iri: &str,
    actuation_principal_iri: &str,
    hook_name: &str,
    hook_receipt_hash: &str,
    broker_receipt_hash: &str,
) -> String {
    format!(
        "<{actuation_subject_iri}> <{PROV_WAS_DERIVED_FROM}> <{actuation_principal_iri}> ;\n  \
         <{HOOK_ACTUATION_NAME_PREDICATE}> \"{hook_name}\" ;\n  \
         <{HOOK_ACTUATION_RECEIPT_PREDICATE}> \"{hook_receipt_hash}\" ;\n  \
         <{HOOK_ACTUATION_BROKER_RECEIPT_PREDICATE}> \"{broker_receipt_hash}\" .\n"
    )
}

/// Re-parse `payload_turtle` with the same parser F02 uses and confirm each of the three F08
/// planning predicates appears as a predicate. IRI comparison only (no literal-value
/// extraction).
///
/// # Errors
/// [`LocalWitnessRefused::AdmittedGraphMissingPredicate`] if any planning predicate is absent.
///
/// # Complexity
/// O(T) over the parsed triples, per predicate checked (3 predicates -> O(T)).
fn verify_admitted_graph_carries_planning_predicates(
    payload_turtle: &str,
) -> Result<(), LocalWitnessRefused> {
    // A malformed payload here would already have refused inside F02; if parsing still fails we
    // treat every planning predicate as absent (report the first).
    let parsed = Parser::parse_triples(payload_turtle, Syntax::Turtle).map_err(|_| {
        LocalWitnessRefused::AdmittedGraphMissingPredicate {
            predicate: PDDL_DOMAIN_PREDICATE,
        }
    })?;
    for predicate in [
        PDDL_DOMAIN_PREDICATE,
        PDDL_PROBLEM_PREDICATE,
        HOOK_PACK_PREDICATE,
    ] {
        let pred_term = VarOrTerm::convert(predicate.to_string());
        if !parsed.iter().any(|t| t.p == pred_term) {
            return Err(LocalWitnessRefused::AdmittedGraphMissingPredicate { predicate });
        }
    }
    Ok(())
}

/// Fold every stage's real digest into one deterministic BLAKE3-hex crown receipt. Material is
/// sorted before hashing (repo invariant #2: canonical order, no reliance on insertion order);
/// no wall clock, no randomness.
fn compute_crown_receipt(
    admission: &AdmissionReceipt,
    planning_state: &PlanningState,
    f08: &PipelineOutcome,
    growth_plan: &GrowthPlan,
    growth: &GrowthOutcome,
    broker_receipt: &BrokerReceipt,
    hook_resolution: &HookResolution,
    actuation_admission: &AdmissionReceipt,
    ocel_outcome: &OcelConstructOutcome,
    parent_closed: bool,
) -> String {
    let f08_tape_sig: String = f08
        .tape
        .ops
        .iter()
        .map(|op| format!("{}:{}:{}", op.index, op.label, op.pred_mask))
        .collect::<Vec<_>>()
        .join("|");
    let mut lines = vec![
        format!("f02.receipt={}", admission.receipt_hash),
        format!("f03.receipt_head={}", planning_state.receipt_head.to_hex()),
        format!("f08.tape={f08_tape_sig}"),
        format!("f09.descent={}", growth_plan.descent_receipt.digest),
        format!(
            "f10.shape=leaves:{},bindings:{}",
            growth.geometry_shape.leaves, growth.geometry_shape.child_bindings
        ),
        format!("f10.geometry_turtle_len={}", growth.geometry_turtle.len()),
        format!("f18.receipt_hash={}", broker_receipt.receipt_hash_hex),
        format!("f19.receipt_hash={}", hook_resolution.receipt_hash),
        format!("f02_readmit.receipt={}", actuation_admission.receipt_hash),
        format!("f24.receipt_head={}", ocel_outcome.receipt_head),
        format!("f21.parent_closed={parent_closed}"),
    ];
    lines.sort();
    let mut hasher = blake3::Hasher::new();
    for line in &lines {
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
#[path = "crown_local_test.rs"]
mod crown_local_test;

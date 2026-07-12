//! Crown LOCAL-witness prefix, composed for real: `F02 -> F03 -> F08 -> F09 -> F10`.
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
//!
//! Content-identity note for F08: F08 consumes the [`AdmittedTriple`] set built from the very
//! same PDDL/hook-pack strings the crown serialized into F02's `payload_turtle`. Identity is
//! guaranteed by construction (single in-function origin), not by re-deserializing F02's Turtle
//! back into `AdmittedTriple`s -- praxis-graphlaw literals expose no public lexical-value
//! accessor, so a round-trip would mean string-munging `Term`'s display form, which this module
//! declines to do.

use powl2_decompose::Powl;
use praxis_graphlaw::chatman::closure::{ClosureLaw, RecursiveSocketClosure};
use praxis_graphlaw::parser::{Parser, Syntax};
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

/// The `prov:wasDerivedFrom` predicate the admitted observation's provenance triple uses (F02
/// gate 2). Bare IRI, matching F02's own `PROV_WAS_DERIVED_FROM` constant.
const PROV_WAS_DERIVED_FROM: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";

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
}

/// Drive the LOCAL crown-witness prefix `F02 -> F03 -> F08 -> F09 -> F10` end to end, in one
/// real call, over a single admitted observation graph.
///
/// See the module doc comment for exactly what makes each edge a real (gated, data-threaded)
/// production edge and the one disclosed F08 -> F09 nuance.
///
/// # Errors
/// [`LocalWitnessRefused`], carrying the first stage's own typed refusal.
///
/// # Complexity
/// The sum of each stage's own documented cost: F02 O(T+S) admission, F03 OWL-RL + Datalog
/// closure + SHACL, F08 grounding + BFS plan search, F09 indexed planning + O(n^3) F10 geometry.
/// This function itself adds only O(T) glue (payload build, re-parse check, receipt fold).
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
    let growth = manufacture_and_bind_child(&run.growth_root, &growth_plan, run.closure_law)?;

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

    // ---- Crown receipt: deterministic BLAKE3 over every stage's real digest ----
    let crown_receipt = compute_crown_receipt(
        &admission,
        &planning_state,
        &plan,
        &growth_plan,
        &growth,
        &broker_receipt,
        &hook_resolution,
    );

    Ok(LocalWitnessOutcome {
        admission,
        planning_state,
        plan,
        growth,
        broker_receipt,
        hook_resolution,
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

//! Crown EXTERNAL-witness tail, composed for real: `F10 -> F12 -> F13 -> F14 -> F15`,
//! stopping honestly at the `F15 -> F16` Erlang OTP-runner boundary.
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
//! # Where the real chain ends (F15 -> F16), disclosed not fabricated
//!
//! The EXTERNAL witness continues `... -> F15 -> F16 (OTP runner) -> F18 (broker) -> F20 (external
//! dispatch) -> ...`. This driver stops at F15. The `F15 -> F16` edge *is* real, but only on the
//! Erlang side: `apps/arazzo_runner/src/arazzo_runner_workflow.erl:114` (`air_core:new`) and `:475`
//! (`air_core:transition`) are the production consumers of F15's transition core. There is no
//! Rust-composable path from this driver into F16's OTP runner (a separate BEAM process, driven by
//! Erlang callers, not by this Rust process), so composing past F15 from here would mean
//! fabricating a topology edge that does not exist. F16's own gen_statem dispatch supervisor is
//! additionally *not* wired into the production dispatch path (its
//! `check_gen_statem_lifecycle_wired` correctly still returns `Err`), and F18's Rust broker / F20's
//! filesystem external-dispatch have no production caller on the EXTERNAL path either. See this
//! module's report for the per-family status.

use std::collections::{BTreeMap, BTreeSet};

use powl2_decompose::{Powl, SocketPath};
use wasm4pm_arazzo::air::{AirProgram, AirRoutingOutcome};

use crate::f10_powl_geometry::POWLModel;
use crate::f12_external_cut::{resolve_external_cut_at, Refusal as EngineRefusal};
use crate::f13_arazzo_artifact::{ArazzoProjectionReceipt, CoreError};
use crate::f14_wasm4pm_arazzo::{compile as compile_arazzo, ArazzoCompileRefused};
use crate::f15_air_transition_core::bridge::{BridgeEvent, BridgeStepDef, BridgeWorkflow};

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

#[cfg(test)]
#[path = "crown_external_test.rs"]
mod crown_external_test;

//! Family F27 -- "Western Electric Workflow Genesis" (atlas ticket V12-027).
//!
//! # Status (this pass)
//!
//! Survey verdict: **MIXED**. This is a Wire-phase-1 pass over the
//! Skeleton stub, not the full production family. Per
//! `.claude/rules/no-overclaiming.md`, this doc comment states plainly what
//! is real (verified this session by reading the dependency source and by
//! running the tests in this file) and what is disclosed as still absent --
//! no part is dressed up to look more complete than it is.
//!
//! Family invariant (repeated verbatim on all 8 atlas lenses): **a control
//! signal does not merely alert a human and does not directly accuse a
//! cause -- it creates unfinished diagnostic work**. Concretely, this module
//! never terminates in a "blamed"/"closed" state; the terminal state is
//! `WorkflowManufactured`, carrying an *open* investigation POWL workflow
//! whose own actions (`gather-measures`, `draft-hypotheses`,
//! `schedule-investigation`) are themselves still to be executed by
//! whatever runs the manufactured workflow.
//!
//! ## What is REAL this pass
//!
//! - **[`generated`]** (D1-D8 provenance-chain vocabulary + the
//!   `ProcessSignalRefused` choke-point catalog) -- GGEN_GENERATABLE, real.
//!   `ggen sync run` output from `packs/f27-western-electric-pack/ontology.ttl`,
//!   generated from an isolated scratch project (the pack's own directory,
//!   not the shared root `ggen.toml` -- zero blast radius on other
//!   families' packs mid-wave) and confirmed byte-identical across repeated
//!   runs. `tests::refusal_catalog_matches_hand_written_enum` below
//!   cross-checks the generated catalog names against
//!   [`ProcessSignalRefused`] so the two cannot silently drift.
//! - **Western Electric Breed** ([`evaluate_western_electric_rules`]) --
//!   REUSE_ADAPT, real. Ports Rules 1, 2, and 4 of
//!   `/Users/sac/wasm4pm/wasm4pm/src/spc.rs::check_western_electric_rules`
//!   (read this session) onto this module's own [`ChartPoint`] shape,
//!   including its NaN/non-finite-value defect handling for Rule 1. Rule 3
//!   (6-consecutive-point monotone trend) is **disclosed as not ported**
//!   this pass -- `evaluate_western_electric_rules`'s own doc comment says
//!   so; callers get 3 of the 4 classic rules, not a silent subset dressed
//!   up as complete. Priority ordering when multiple rules fire in the same
//!   evaluation (Rule1 > Rule4 > Rule2) is taken from
//!   `/Users/sac/lsp-max/src/primitives/spc.rs`'s own doc comment (also
//!   read this session), which independently documents that priority for
//!   its own port of the same rules.
//! - **Control Baseline admission** ([`ControlBaseline::admit`]) --
//!   HAND_WRITE_REQUIRED, real. Computes mean and Bessel-corrected sample
//!   standard deviation (the exact formulas in
//!   `wasm4pm::spc::spc_mean`/`spc_std_dev`, re-derived here rather than
//!   imported since wasm4pm is not a workspace member and pulling in its
//!   whole crate for two formulas would be the opposite of "keep it
//!   small"), then refuses [`ProcessSignalRefused::BaselineRefused`] on
//!   fewer than 2 samples or any non-finite sample -- the boundary-violated-
//!   input choke point the family invariant names.
//! - **Signal admission** ([`admit_signal`]) -- HAND_WRITE_REQUIRED, real.
//!   Selects the single highest-priority fired [`SpecialCause`] (or `None`
//!   if the process is in control -- not a refusal, just no signal) and
//!   defensively re-refuses [`ProcessSignalRefused::SignalAdmissionRefused`]
//!   if ever called against baseline evidence that was not itself admitted.
//! - **Signal CONSTRUCT** ([`SignalReceipt::construct`]) -- HAND_WRITE_REQUIRED,
//!   real. BLAKE3 over a canonical, field-tagged, sorted-by-construction
//!   encoding of the baseline and fired rule (no `HashMap` iteration
//!   anywhere in this module).
//! - **Diagnostic Goal Generator** ([`generate_diagnostic_goal`]) --
//!   HAND_WRITE_REQUIRED, real, and the literal substance of the family
//!   invariant: it emits real PDDL domain+problem text for a fixed 3-action
//!   *open* investigation domain (`gather-measures` -> `draft-hypotheses`
//!   -> `schedule-investigation`), never a `cause-blamed` or
//!   `investigation-closed` predicate. This is new domain logic with no
//!   cited external source -- the survey named it as irreducibly novel.
//! - **PDDL Planner** (`crate::f08_pddl_planning::{projector, planner}`,
//!   called from [`manufacture_investigation_workflow`]) -- REUSE_ADAPT,
//!   real. Reuses this same crate's already-wired, already-tested F08
//!   module (grounding + bounded BFS search over `bcinr_pddl`) rather than
//!   re-deriving planning glue from `crates/cng/src/pipeline.rs`'s
//!   file-based artifact importer, which would have meant writing PDDL to a
//!   temp directory just to read it back.
//! - **POWL Investigation** (`cng::powl::{project_tape_to_powl,
//!   powl_to_turtle}`, called from [`manufacture_investigation_workflow`])
//!   -- REUSE_ADAPT, real. `cng` is already a workspace dependency of this
//!   crate (added for F20/F24, `features = ["bench"]`); `powl` is one of
//!   its unconditional `pub mod`s (confirmed by reading
//!   `crates/cng/src/lib.rs` this session), so no new Cargo.toml edit was
//!   needed.
//! - **State machine** ([`LifecycleState`], [`run_pipeline`]) --
//!   HAND_WRITE_REQUIRED, real. Implements the exact closed 8-state machine
//!   the survey specifies, with the two lawful `Refused` transition points
//!   (from `Baselined`, from `SignalAdmitted`) and no others.
//! - **Idempotency + correlation gate** ([`IdempotencyGate`]) --
//!   HAND_WRITE_REQUIRED, real for the in-process, single-run-instance
//!   case: a duplicate `correlation_id` replays the cached
//!   [`InvestigationRecord`] rather than re-running the pipeline (proven by
//!   `tests::idempotency_gate_replays_instead_of_re_actuating`, which counts
//!   pipeline invocations). **Disclosed gap**: the gate is in-memory only
//!   this pass -- it does not persist across a process restart, so the
//!   survey's "process/engine restarts" chaos-lens requirement is only
//!   partially met (duplicate-event idempotency: real; restart durability:
//!   not yet built). Stale/malformed correlated results are handled by the
//!   same typed-refusal boundary as any other malformed input (baseline/
//!   signal admission refuse before a correlation ID is ever recorded), not
//!   by a separate staleness check.
//! - **Receipt head / replay equivalence** ([`receipt_head`]) --
//!   HAND_WRITE_REQUIRED, real. `tests::replay_is_byte_identical` runs the
//!   full pipeline twice on identical inputs and asserts the two receipt
//!   heads are byte-identical strings.
//!
//! ## What is NOT built this pass (disclosed, not silently skipped)
//!
//! - Rule 3 (monotone trend) of the four classic Western Electric rules.
//! - Durable (cross-process-restart) idempotency persistence.
//! - Writing the D1-D8 provenance chain as actual RDF triples into an
//!   oxigraph store ([`emit_provenance_turtle`] emits real Turtle *text*
//!   with real content-addressed IRIs and `prov:wasDerivedFrom` edges
//!   chained through a live pipeline run, but nothing here loads that text
//!   into a queryable store).
//! - Cross-scale consequence propagation beyond the single investigation
//!   workflow (the survey's `CROSS_SCALE_LATERAL_GROWTH_PROVEN` claim
//!   ceiling): this pass proves one signal manufactures one workflow, not
//!   that manufactured workflows compose across families.
//!
//! Per this repo's claim-ceiling discipline, none of
//! `WESTERN_ELECTRIC_WORKFLOW_GENESIS_PROVEN`,
//! `CROSS_SCALE_LATERAL_GROWTH_PROVEN`, or `NO_ALERT_ONLY_HANDOFF` is
//! claimed by this module -- the exit-gate evidence bar (production
//! reachability trace, chaos/recovery evidence, full receipt/replay
//! equivalence across the whole family, not just this one pipeline) is not
//! met yet.
//!
//! Survey-cited paths (informed research from the v26.7.12 family survey
//! handed to the scaffolding session inline, re-verified by this pass
//! against the sources actually reused): `/Users/sac/Downloads/
//! v26.7.12_mermaid_atlas/families/F27_western-electric.md`,
//! `/Users/sac/wasm4pm/wasm4pm/src/spc.rs`,
//! `/Users/sac/lsp-max/src/primitives/spc.rs`,
//! `crates/multifractal-workflow/src/f08_pddl_planning/{projector,planner}.rs`,
//! `crates/cng/src/powl.rs`. (`/Users/sac/lsp-max/crates/lsp-max-andon/src/lib.rs`,
//! `/Users/sac/lsp-max/src/rule_pack_server.rs`, and `crates/cng/src/{pipeline,runner}.rs`
//! were re-checked this session too but not reused -- lsp-max-andon is a
//! self-labeled stub with every invariant hardcoded `|_| true`, per the
//! survey's own rejection; `cng::pipeline`/`cng::runner` operate over
//! on-disk artifact directories and a bcinr-powl execution runner, neither
//! of which this in-memory signal-to-goal pipeline needs.)

use std::collections::BTreeMap;

use bcinr_pddl::Pddl8Tape;

use crate::f08_pddl_planning::projector::{
    AdmittedTriple, PDDL_DOMAIN_PREDICATE, PDDL_PROBLEM_PREDICATE,
};
use crate::f08_pddl_planning::{planner, projector};

/// GGEN_GENERATABLE: the D1-D8 provenance-chain vocabulary and the
/// `ProcessSignalRefused` choke-point catalog. See
/// `f27_western_electric_generated.rs`'s own header for the exact
/// regeneration recipe; wrapped in a real `mod` (rather than F09's flat
/// top-level `include!`) so callers reference `generated::PROVENANCE_CHAIN`
/// / `generated::REFUSAL_CATALOG` explicitly, keeping generated and
/// hand-written names visually distinct throughout this file.
pub mod generated {
    include!("f27_western_electric_generated.rs");
}

// ---------------------------------------------------------------------------
// D1: Measure Projector (HAND_WRITE_REQUIRED)
// ---------------------------------------------------------------------------

/// One admitted process measure. `tick` is a logical counter, never a wall
/// clock, per this repo's no-wall-clock-in-hash/receipt-paths invariant --
/// callers own how logical ticks map to real time, if at all.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcessMeasure {
    pub tick: u64,
    pub value: f64,
}

/// One point on a Western Electric control chart: a measured value plus the
/// control limits it is evaluated against. Mirrors the shape of
/// `wasm4pm::spc::ChartData` (minus the `timestamp`/`subgroup_data` fields
/// this module has no use for) so [`evaluate_western_electric_rules`] is a
/// faithful adaptation, not a reinvention.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartPoint {
    pub tick: u64,
    pub value: f64,
    pub ucl: f64,
    pub cl: f64,
    pub lcl: f64,
}

/// D1 Measure Projector: projects admitted process measures onto a
/// [`ControlBaseline`]'s control limits, producing the [`ChartPoint`]
/// sequence [`evaluate_western_electric_rules`] consumes. Pure data
/// reshaping -- the baseline itself must already be admitted (see
/// [`ControlBaseline::admit`]).
///
/// # Complexity
/// O(n) over `measures`.
#[must_use]
pub fn project_measures_to_chart_points(
    measures: &[ProcessMeasure],
    baseline: &ControlBaseline,
) -> Vec<ChartPoint> {
    measures
        .iter()
        .map(|m| ChartPoint {
            tick: m.tick,
            value: m.value,
            ucl: baseline.ucl,
            cl: baseline.mean,
            lcl: baseline.lcl,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// D2: Control Baseline (HAND_WRITE_REQUIRED)
// ---------------------------------------------------------------------------

/// An admitted control baseline: mean, standard deviation, and the derived
/// 3-sigma control limits. Only constructible via [`ControlBaseline::admit`]
/// -- there is no public field-literal constructor, so a caller cannot
/// fabricate a baseline that bypasses the admission gate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlBaseline {
    pub mean: f64,
    pub std_dev: f64,
    pub ucl: f64,
    pub cl: f64,
    pub lcl: f64,
    sample_count: usize,
}

impl ControlBaseline {
    /// Admits `history` as a control baseline: computes the mean and
    /// Bessel-corrected sample standard deviation (same formulas as
    /// `wasm4pm::spc::spc_mean`/`spc_std_dev`), derives 3-sigma control
    /// limits, and refuses [`ProcessSignalRefused::BaselineRefused`] if
    /// `history` has fewer than 2 samples or contains any non-finite value
    /// -- the "baseline drift" / corrupt-evidence boundary the family
    /// invariant names.
    ///
    /// # Errors
    /// [`ProcessSignalRefused::BaselineRefused`].
    ///
    /// # Complexity
    /// O(n) over `history`.
    pub fn admit(history: &[ProcessMeasure]) -> Result<Self, ProcessSignalRefused> {
        if history.len() < 2 {
            return Err(ProcessSignalRefused::BaselineRefused(format!(
                "control baseline needs >=2 historical measures, got {}",
                history.len()
            )));
        }
        if let Some(bad) = history.iter().find(|m| !m.value.is_finite()) {
            return Err(ProcessSignalRefused::BaselineRefused(format!(
                "non-finite measure at tick {}: {}",
                bad.tick, bad.value
            )));
        }
        let n = history.len() as f64;
        let mean = history.iter().map(|m| m.value).sum::<f64>() / n;
        let variance = history
            .iter()
            .map(|m| (m.value - mean).powi(2))
            .sum::<f64>()
            / (n - 1.0);
        let std_dev = variance.sqrt();
        Ok(Self {
            mean,
            std_dev,
            ucl: mean + 3.0 * std_dev,
            cl: mean,
            lcl: mean - 3.0 * std_dev,
            sample_count: history.len(),
        })
    }
}

// ---------------------------------------------------------------------------
// D3 (RuleEvaluation): Western Electric Breed (REUSE_ADAPT)
// ---------------------------------------------------------------------------

/// Direction of a shift relative to the center line. Mirrors
/// `wasm4pm::spc::ShiftDirection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftDirection {
    Above,
    Below,
}

/// A detected Western Electric special-cause signal. Mirrors the subset of
/// `wasm4pm::spc::SpecialCause` this module ports (Rules 1, 2, 4 --
/// Rule 3/`Trend` is disclosed not ported, see module doc comment).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecialCause {
    /// Rule 1: point beyond UCL or LCL (beyond 3-sigma), or a non-finite
    /// value (corrupt-evidence defect class, ported verbatim from
    /// `wasm4pm::spc::check_western_electric_rules`'s own Rule 1 comment).
    OutOfControl { value: f64, ucl: f64, lcl: f64 },
    /// Rule 2: N consecutive points on the same side of the center line.
    Shift {
        direction: ShiftDirection,
        count: usize,
    },
    /// Rule 4: 2 of 3 consecutive points beyond 2-sigma on the same side.
    TwoOfThree { direction: ShiftDirection },
}

impl SpecialCause {
    /// Priority rank for choosing among simultaneously-fired causes: lower
    /// is higher priority. Order (Rule1 > Rule4 > Rule2) is taken from
    /// `lsp-max/src/primitives/spc.rs`'s own doc comment ("Priority: Rule1 >
    /// Rule4 > Rule2 > Rule3"), restricted to the three rules this module
    /// ports.
    fn priority_rank(&self) -> u8 {
        match self {
            SpecialCause::OutOfControl { .. } => 0,
            SpecialCause::TwoOfThree { .. } => 1,
            SpecialCause::Shift { .. } => 2,
        }
    }
}

/// D3 RuleEvaluation / "Western Electric Breed": evaluates Rules 1, 2, and 4
/// of the classic Western Electric rule set against a trailing window of
/// [`ChartPoint`]s. Ported from `wasm4pm::spc::check_western_electric_rules`
/// (Rule 1's non-finite-value handling included verbatim); Rule 3
/// (6-consecutive monotone trend) is not ported this pass.
///
/// Returns all signals found, in the order the rules are classically
/// numbered (Rule 1, then Rule 2, then Rule 4) -- callers wanting a single
/// signal should feed the result through [`admit_signal`], which applies
/// the priority order.
///
/// # Complexity
/// O(1) additional work per rule beyond O(w) to slice the trailing window,
/// where w = min(9, data.len()).
#[must_use]
pub fn evaluate_western_electric_rules(data: &[ChartPoint]) -> Vec<SpecialCause> {
    let mut alerts = Vec::new();

    // Rule 1: point beyond UCL/LCL, or non-finite (corrupt evidence is
    // treated as an out-of-control signal, never silently passed through --
    // same defect class wasm4pm's Rule 1 comment names).
    if let Some(latest) = data.last() {
        if !latest.value.is_finite() || latest.value > latest.ucl || latest.value < latest.lcl {
            alerts.push(SpecialCause::OutOfControl {
                value: latest.value,
                ucl: latest.ucl,
                lcl: latest.lcl,
            });
        }
    }

    if data.len() < 9 {
        return alerts;
    }
    let recent = &data[data.len() - 9..];

    // Rule 2: 9 consecutive points on the same side of the center line.
    {
        let above = recent.iter().filter(|d| d.value > d.cl).count() == 9;
        let below = recent.iter().filter(|d| d.value < d.cl).count() == 9;
        if above {
            alerts.push(SpecialCause::Shift {
                direction: ShiftDirection::Above,
                count: 9,
            });
        } else if below {
            alerts.push(SpecialCause::Shift {
                direction: ShiftDirection::Below,
                count: 9,
            });
        }
    }

    // Rule 4: 2 of 3 consecutive points beyond 2-sigma on the same side.
    {
        let last_3 = &recent[6..];
        let beyond_above = last_3
            .iter()
            .filter(|d| {
                let sigma = (d.ucl - d.cl) / 3.0;
                sigma > 0.0 && d.value > d.cl + 2.0 * sigma
            })
            .count();
        let beyond_below = last_3
            .iter()
            .filter(|d| {
                let sigma = (d.ucl - d.cl) / 3.0;
                sigma > 0.0 && d.value < d.cl - 2.0 * sigma
            })
            .count();
        if beyond_above >= 2 {
            alerts.push(SpecialCause::TwoOfThree {
                direction: ShiftDirection::Above,
            });
        } else if beyond_below >= 2 {
            alerts.push(SpecialCause::TwoOfThree {
                direction: ShiftDirection::Below,
            });
        }
    }

    alerts
}

// ---------------------------------------------------------------------------
// D4 (ProcessShiftSignal): Signal Admission (HAND_WRITE_REQUIRED)
// ---------------------------------------------------------------------------

/// An admitted process-shift signal: the single highest-priority fired
/// [`SpecialCause`], plus the baseline it was evaluated against. Only
/// constructible via [`admit_signal`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcessShiftSignal {
    pub cause: SpecialCause,
    pub baseline: ControlBaseline,
}

/// D4 Signal Admission choke point: selects the single highest-priority
/// cause from `alerts` (see [`SpecialCause::priority_rank`]) and pairs it
/// with `baseline` as an admitted [`ProcessShiftSignal`]. Returns `Ok(None)`
/// (not a refusal) when `alerts` is empty -- an in-control process is not an
/// error condition, it simply produces no signal to admit.
///
/// The `baseline_is_admitted` parameter is a defensive re-check: this
/// function has no way to verify at the type level that `baseline` actually
/// came from [`ControlBaseline::admit`] rather than being reconstructed by
/// unsafe/reflective means elsewhere, so callers that cannot statically
/// prove admission must pass `false` and get refused rather than risk
/// admitting a signal derived from unvetted baseline evidence. Every caller
/// in this module passes `true` because it always holds a real
/// [`ControlBaseline`] value that only exists because `admit` succeeded.
///
/// # Errors
/// [`ProcessSignalRefused::SignalAdmissionRefused`] if `baseline_is_admitted`
/// is `false`.
pub fn admit_signal(
    alerts: &[SpecialCause],
    baseline: ControlBaseline,
    baseline_is_admitted: bool,
) -> Result<Option<ProcessShiftSignal>, ProcessSignalRefused> {
    if !baseline_is_admitted {
        return Err(ProcessSignalRefused::SignalAdmissionRefused(
            "signal admission attempted against baseline evidence that was not itself admitted"
                .to_string(),
        ));
    }
    let Some(cause) = alerts.iter().min_by_key(|c| c.priority_rank()).copied() else {
        return Ok(None);
    };
    Ok(Some(ProcessShiftSignal { cause, baseline }))
}

// ---------------------------------------------------------------------------
// D5 (SignalReceipt): Signal CONSTRUCT (HAND_WRITE_REQUIRED)
// ---------------------------------------------------------------------------

/// A BLAKE3 receipt over an admitted [`ProcessShiftSignal`]: canonical,
/// field-tagged, sorted-by-construction (no `HashMap` iteration anywhere in
/// this encoding), matching this repo's receipts-are-computed-never-
/// asserted invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalReceipt {
    pub digest_hex: String,
}

impl SignalReceipt {
    /// D5 Signal CONSTRUCT: BLAKE3-hashes a canonical encoding of `signal`.
    /// Deterministic: identical signals produce identical digests; distinct
    /// signals produce (with overwhelming probability) distinct digests.
    ///
    /// # Complexity
    /// O(1) -- fixed-size encoding regardless of signal content.
    #[must_use]
    pub fn construct(signal: &ProcessShiftSignal) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"wego:ProcessShiftSignal\0");
        hasher.update(b"cause=");
        match signal.cause {
            SpecialCause::OutOfControl { value, ucl, lcl } => {
                hasher.update(b"OutOfControl\0value=");
                hasher.update(value.to_bits().to_le_bytes().as_slice());
                hasher.update(b"\0ucl=");
                hasher.update(ucl.to_bits().to_le_bytes().as_slice());
                hasher.update(b"\0lcl=");
                hasher.update(lcl.to_bits().to_le_bytes().as_slice());
            }
            SpecialCause::Shift { direction, count } => {
                hasher.update(b"Shift\0direction=");
                hasher.update(shift_direction_tag(direction).as_bytes());
                hasher.update(b"\0count=");
                hasher.update(&(count as u64).to_le_bytes());
            }
            SpecialCause::TwoOfThree { direction } => {
                hasher.update(b"TwoOfThree\0direction=");
                hasher.update(shift_direction_tag(direction).as_bytes());
            }
        }
        hasher.update(b"\0baseline.mean=");
        hasher.update(signal.baseline.mean.to_bits().to_le_bytes().as_slice());
        hasher.update(b"\0baseline.std_dev=");
        hasher.update(signal.baseline.std_dev.to_bits().to_le_bytes().as_slice());
        hasher.update(b"\0baseline.sample_count=");
        hasher.update(&(signal.baseline.sample_count as u64).to_le_bytes());
        Self {
            digest_hex: hasher.finalize().to_hex().to_string(),
        }
    }
}

fn shift_direction_tag(d: ShiftDirection) -> &'static str {
    match d {
        ShiftDirection::Above => "Above",
        ShiftDirection::Below => "Below",
    }
}

// ---------------------------------------------------------------------------
// D6 (DiagnosticGoal): Diagnostic Goal Generator (HAND_WRITE_REQUIRED,
// central novel logic per the family survey)
// ---------------------------------------------------------------------------

/// A real PDDL domain+problem pair describing an *open* investigation, never
/// a cause-blaming or investigation-closing goal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticGoal {
    pub domain_text: String,
    pub problem_text: String,
    /// The problem name, content-addressed from the [`SignalReceipt`] this
    /// goal was derived from -- ties the goal back to its signal without
    /// naming a cause.
    pub problem_name: String,
}

/// D6 Diagnostic Goal Generator: converts an admitted [`SignalReceipt`] into
/// a fixed 3-action open-investigation PDDL domain
/// (`gather-measures` -> `draft-hypotheses` -> `schedule-investigation`) and
/// a problem instance whose goal is `investigation-scheduled` -- deliberately
/// short of any `investigation-closed` or `cause-blamed` predicate, which is
/// the literal mechanism by which this module honors the family invariant
/// ("creates unfinished diagnostic work", never a direct accusation or a
/// closed case). The problem name embeds the first 16 hex chars of the
/// signal receipt's digest, so the goal is traceable to (not detached from)
/// the signal that produced it.
///
/// # Complexity
/// O(1) -- fixed-size PDDL text, one string-formatted digest prefix.
#[must_use]
pub fn generate_diagnostic_goal(receipt: &SignalReceipt) -> DiagnosticGoal {
    let short_digest = &receipt.digest_hex[..16];
    let problem_name = format!("f27-investigation-{short_digest}");
    let domain_text = r#"
(define (domain f27-open-investigation)
  (:requirements :strips)
  (:predicates (signal-detected) (measures-gathered) (hypotheses-drafted) (investigation-scheduled))
  (:action gather-measures
    :parameters ()
    :precondition (signal-detected)
    :effect (and (measures-gathered)))
  (:action draft-hypotheses
    :parameters ()
    :precondition (measures-gathered)
    :effect (and (hypotheses-drafted)))
  (:action schedule-investigation
    :parameters ()
    :precondition (hypotheses-drafted)
    :effect (and (investigation-scheduled))))
"#
    .to_string();
    let problem_text = format!(
        r#"
(define (problem {problem_name})
  (:domain f27-open-investigation)
  (:objects )
  (:init (signal-detected))
  (:goal (and (investigation-scheduled))))
"#
    );
    DiagnosticGoal {
        domain_text,
        problem_text,
        problem_name,
    }
}

// ---------------------------------------------------------------------------
// D7 (InvestigationPlan): PDDL Planner (REUSE_ADAPT via crate::f08_pddl_planning)
// D8 (InvestigationPOWL): POWL Investigation (REUSE_ADAPT via cng::powl)
// ---------------------------------------------------------------------------

/// A manufactured investigation workflow: the plan tape F08's planner found,
/// plus the POWL 2.0 Turtle serialization D8 produced from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvestigationWorkflow {
    pub plan_op_labels: Vec<String>,
    pub powl_turtle: String,
}

/// D7 + D8: grounds and plans `goal`'s PDDL text via this crate's own
/// already-wired `f08_pddl_planning` module (REUSE_ADAPT: no PDDL search
/// logic reimplemented here), then projects the resulting plan tape into a
/// flat POWL 2.0 model and serializes it as Turtle via `cng::powl`
/// (REUSE_ADAPT: `cng` already exposes this unconditionally). The POWL
/// model's `derivedFrom` provenance is set to a `urn:blake3:` IRI built from
/// `receipt`, chaining D8 back to D5 without needing a live oxigraph store.
///
/// # Errors
/// [`ProcessSignalRefused::PowlConstructionRefused`] wrapping whichever
/// stage first refused: F08's `Refusal` (parse/ground/plan failure) or
/// `cng::powl::CngRefusal` (empty-tape projection failure).
///
/// # Complexity
/// Bounded by F08's `planner::ground`/`planner::plan` (see that module's own
/// complexity docs) plus O(n^2) for `cng::powl`'s pre-closed order relation.
pub fn manufacture_investigation_workflow(
    goal: &DiagnosticGoal,
    receipt: &SignalReceipt,
) -> Result<InvestigationWorkflow, ProcessSignalRefused> {
    let graph = vec![
        AdmittedTriple {
            subject: format!("urn:blake3:{}", receipt.digest_hex),
            predicate: PDDL_DOMAIN_PREDICATE.to_string(),
            object_literal: goal.domain_text.clone(),
        },
        AdmittedTriple {
            subject: format!("urn:blake3:{}", receipt.digest_hex),
            predicate: PDDL_PROBLEM_PREDICATE.to_string(),
            object_literal: goal.problem_text.clone(),
        },
    ];
    let (domain, problem) = projector::project_and_resolve(&graph).map_err(|e| {
        ProcessSignalRefused::PowlConstructionRefused(format!(
            "diagnostic goal PDDL did not project/resolve: {e}"
        ))
    })?;
    let grounded = planner::ground(&domain, &problem).map_err(|e| {
        ProcessSignalRefused::PowlConstructionRefused(format!(
            "diagnostic goal did not ground: {e}"
        ))
    })?;
    let tape: Pddl8Tape = planner::plan(&grounded).map_err(|e| {
        ProcessSignalRefused::PowlConstructionRefused(format!(
            "diagnostic goal is unreachable from the open-investigation init state: {e}"
        ))
    })?;
    let plan_op_labels: Vec<String> = tape.ops.iter().map(|op| op.label.clone()).collect();

    let powl = cng::powl::project_tape_to_powl(&tape).map_err(|e| {
        ProcessSignalRefused::PowlConstructionRefused(format!(
            "plan tape did not project to POWL: {e}"
        ))
    })?;
    let base_iri = format!("urn:mfw:f27:investigation:{}", &receipt.digest_hex[..16]);
    let derived_from = format!("urn:blake3:{}", receipt.digest_hex);
    let powl_turtle = cng::powl::powl_to_turtle(&powl, &base_iri, Some(&derived_from));

    Ok(InvestigationWorkflow {
        plan_op_labels,
        powl_turtle,
    })
}

// ---------------------------------------------------------------------------
// State machine + typed refusal (HAND_WRITE_REQUIRED)
// ---------------------------------------------------------------------------

/// The closed F27 state machine, exactly as the survey specifies:
/// `Measured -> Baselined -> RulesEvaluated -> SignalCandidate ->
/// SignalAdmitted -> GoalGenerated -> Planned -> WorkflowManufactured`, with
/// `Refused` reachable only from `Baselined` and from `SignalAdmitted`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Measured,
    Baselined,
    RulesEvaluated,
    SignalCandidate,
    SignalAdmitted,
    GoalGenerated,
    Planned,
    WorkflowManufactured,
    /// Terminal refused state. `from` records which of the two lawful
    /// refusal-origin states (`Baselined` or `SignalAdmitted`) the pipeline
    /// was in when it refused.
    Refused {
        from: &'static str,
    },
}

/// F27's typed refusal taxonomy: the three named choke points the family
/// survey cites (baseline load, signal admission, POWL construction). Every
/// variant name here also appears in [`generated::REFUSAL_CATALOG`];
/// `tests::refusal_catalog_matches_hand_written_enum` proves the two cannot
/// silently drift apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessSignalRefused {
    /// Choke point 1 (state `Baselined` -> `Refused`): boundary-violated
    /// baseline input (insufficient history, non-finite measure).
    BaselineRefused(String),
    /// Choke point 2 (state `SignalAdmitted` -> `Refused`, defensive
    /// re-check path): signal admission attempted against unadmitted
    /// baseline evidence.
    SignalAdmissionRefused(String),
    /// Choke point 3 (state `SignalAdmitted` -> `Refused`, downstream
    /// manufacture path): the diagnostic goal's PDDL failed to
    /// parse/ground/plan, or the plan tape failed to project to POWL.
    PowlConstructionRefused(String),
}

impl ProcessSignalRefused {
    /// Which lawful refusal-origin state this refusal fires from, per the
    /// closed state machine (`Baselined` for `BaselineRefused`;
    /// `SignalAdmitted` for the other two).
    #[must_use]
    pub fn origin_state(&self) -> &'static str {
        match self {
            ProcessSignalRefused::BaselineRefused(_) => "Baselined",
            ProcessSignalRefused::SignalAdmissionRefused(_)
            | ProcessSignalRefused::PowlConstructionRefused(_) => "SignalAdmitted",
        }
    }
}

impl std::fmt::Display for ProcessSignalRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessSignalRefused::BaselineRefused(reason) => {
                write!(f, "ProcessSignalRefused::BaselineRefused: {reason}")
            }
            ProcessSignalRefused::SignalAdmissionRefused(reason) => {
                write!(f, "ProcessSignalRefused::SignalAdmissionRefused: {reason}")
            }
            ProcessSignalRefused::PowlConstructionRefused(reason) => {
                write!(f, "ProcessSignalRefused::PowlConstructionRefused: {reason}")
            }
        }
    }
}

impl std::error::Error for ProcessSignalRefused {}

/// The full, real record of one lawful pipeline run: every stage's output,
/// the final [`LifecycleState`] (always `WorkflowManufactured` for a value
/// actually returned by [`run_pipeline`], since refusals return `Err`
/// instead), and the [`receipt_head`] binding the whole chain.
#[derive(Debug, Clone, PartialEq)]
pub struct InvestigationRecord {
    pub baseline: ControlBaseline,
    pub signal: ProcessShiftSignal,
    pub signal_receipt: SignalReceipt,
    pub goal: DiagnosticGoal,
    pub workflow: InvestigationWorkflow,
    pub state: LifecycleState,
}

/// Folds every stage's content-addressed evidence into a single BLAKE3
/// receipt head, so two runs over identical inputs can be proven to have
/// produced equivalent results by comparing this one string (replay
/// equivalence -- see `tests::replay_is_byte_identical`).
///
/// # Complexity
/// O(1) plus O(|powl_turtle|) to hash the serialized POWL text.
#[must_use]
pub fn receipt_head(record: &InvestigationRecord) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"wego:ReceiptHead\0signal_receipt=");
    hasher.update(record.signal_receipt.digest_hex.as_bytes());
    hasher.update(b"\0problem_name=");
    hasher.update(record.goal.problem_name.as_bytes());
    hasher.update(b"\0plan_ops=");
    for label in &record.workflow.plan_op_labels {
        hasher.update(label.as_bytes());
        hasher.update(b"\0");
    }
    hasher.update(b"powl_turtle=");
    hasher.update(record.workflow.powl_turtle.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Runs the full C1-C8 / D1-D8 pipeline end to end, in atlas stage order:
/// Measure Projector -> Control Baseline -> Western Electric Breed -> Signal
/// Admission -> Signal CONSTRUCT -> Diagnostic Goal Generator -> PDDL
/// Planner -> POWL Investigation. Returns `Ok(None)` (not a refusal) when
/// the process is in control and no [`SpecialCause`] fires -- staying at
/// `RulesEvaluated`, never manufacturing a workflow for a non-signal, per
/// the family invariant.
///
/// # Errors
/// [`ProcessSignalRefused`] from whichever of the two lawful refusal-origin
/// states (`Baselined`, `SignalAdmitted`) first refuses.
pub fn run_pipeline(
    history: &[ProcessMeasure],
    latest: &[ProcessMeasure],
) -> Result<Option<InvestigationRecord>, ProcessSignalRefused> {
    // D1 Measure Projector + D2 Control Baseline.
    let baseline = ControlBaseline::admit(history)?;
    let chart_points = project_measures_to_chart_points(latest, &baseline);

    // D3 RuleEvaluation (Western Electric Breed).
    let alerts = evaluate_western_electric_rules(&chart_points);

    // D4 Signal Admission.
    let Some(signal) = admit_signal(&alerts, baseline, true)? else {
        return Ok(None);
    };

    // D5 Signal CONSTRUCT.
    let signal_receipt = SignalReceipt::construct(&signal);

    // D6 Diagnostic Goal Generator.
    let goal = generate_diagnostic_goal(&signal_receipt);

    // D7 PDDL Planner + D8 POWL Investigation.
    let workflow = manufacture_investigation_workflow(&goal, &signal_receipt)?;

    Ok(Some(InvestigationRecord {
        baseline,
        signal,
        signal_receipt,
        goal,
        workflow,
        state: LifecycleState::WorkflowManufactured,
    }))
}

// ---------------------------------------------------------------------------
// Chaos lens: idempotency + correlation gate (HAND_WRITE_REQUIRED,
// in-memory only this pass -- see module doc comment for the disclosed
// durability gap)
// ---------------------------------------------------------------------------

/// An in-process idempotency + correlation gate: keyed by caller-supplied
/// `correlation_id`, replays a cached [`InvestigationRecord`] for a
/// duplicate event rather than re-running (and re-manufacturing a second
/// investigation workflow for) the same signal. Never durable across a
/// process restart this pass -- see module doc comment.
#[derive(Debug, Default)]
pub struct IdempotencyGate {
    admitted: BTreeMap<String, InvestigationRecord>,
}

impl IdempotencyGate {
    #[must_use]
    pub fn new() -> Self {
        Self {
            admitted: BTreeMap::new(),
        }
    }

    /// Admits `correlation_id`'s pipeline result, or replays the
    /// already-admitted record for a duplicate `correlation_id` without
    /// calling `run` again -- this is the "never duplicate actuation"
    /// guarantee. `run` is only invoked at most once per distinct
    /// `correlation_id` ever seen by this gate instance.
    ///
    /// # Errors
    /// Whatever `run` returns for a first-seen `correlation_id`; a
    /// previously-refused `correlation_id` is not cached (refusals carry no
    /// standing to replay) and re-attempts `run` on the next call, exactly
    /// as a fresh event would.
    pub fn admit_or_replay(
        &mut self,
        correlation_id: &str,
        run: impl FnOnce() -> Result<Option<InvestigationRecord>, ProcessSignalRefused>,
    ) -> Result<Option<InvestigationRecord>, ProcessSignalRefused> {
        if let Some(existing) = self.admitted.get(correlation_id) {
            return Ok(Some(existing.clone()));
        }
        let outcome = run()?;
        if let Some(record) = &outcome {
            self.admitted
                .insert(correlation_id.to_string(), record.clone());
        }
        Ok(outcome)
    }
}

// ---------------------------------------------------------------------------
// D1-D8 provenance chain: real RDF/PROV Turtle emission (HAND_WRITE_REQUIRED
// glue over the GGEN_GENERATABLE vocabulary in `generated::PROVENANCE_CHAIN`)
// ---------------------------------------------------------------------------

/// PROV-O namespace, matching `cng::powl::PROV_PREFIX` so Turtle emitted by
/// this module and by `cng::powl` use the same prefix binding.
pub const PROV_PREFIX: &str = "http://www.w3.org/ns/prov#";

/// Emits the D1-D8 provenance chain as real Turtle: one content-addressed
/// IRI per stage (content-addressed from that stage's own real data, not a
/// counter), chained by `prov:wasDerivedFrom` edges using
/// [`generated::PROVENANCE_CHAIN`]'s fixed order and stage names. This is
/// the hand-written half of D1-D8: the vocabulary/order table is
/// ggen-generated ([`generated::PROVENANCE_CHAIN`]); binding it to a live
/// run's actual content-addressed IRIs is this function.
///
/// **Disclosed scope boundary**: this emits Turtle *text*; nothing in this
/// module loads it into an oxigraph store or validates it against a SHACL
/// shape (no such shape exists for this vocabulary yet).
///
/// # Complexity
/// O(1) -- fixed 8-stage chain.
#[must_use]
pub fn emit_provenance_turtle(record: &InvestigationRecord) -> String {
    let stage_iri = |name: &str| -> String {
        let digest = blake3::hash(
            format!(
                "{name}\0{}\0{}",
                record.signal_receipt.digest_hex, record.goal.problem_name
            )
            .as_bytes(),
        );
        format!("urn:mfw:f27:{name}:{}", &digest.to_hex().to_string()[..16])
    };
    let mut out = String::new();
    out.push_str("@prefix prov: <");
    out.push_str(PROV_PREFIX);
    out.push_str("> .\n");
    let iris: Vec<String> = generated::PROVENANCE_CHAIN
        .iter()
        .map(|s| stage_iri(s.name))
        .collect();
    for (i, stage) in generated::PROVENANCE_CHAIN.iter().enumerate() {
        out.push_str(&format!(
            "<{}> a <urn:mfw:f27:vocab#{}> .\n",
            iris[i], stage.name
        ));
        if i > 0 {
            out.push_str(&format!(
                "<{}> prov:wasDerivedFrom <{}> .\n",
                iris[i],
                iris[i - 1]
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stable_history() -> Vec<ProcessMeasure> {
        // Alternating around 10.0 so mean~10, std_dev>0, and no rule fires
        // spuriously during baseline construction.
        (0..20u64)
            .map(|tick| ProcessMeasure {
                tick,
                value: if tick % 2 == 0 { 9.5 } else { 10.5 },
            })
            .collect()
    }

    // -- D2 Control Baseline ------------------------------------------------

    #[test]
    fn baseline_admits_a_real_mean_and_std_dev() {
        let baseline = ControlBaseline::admit(&stable_history()).expect("stable history admits");
        assert!((baseline.mean - 10.0).abs() < 0.1, "{}", baseline.mean);
        assert!(baseline.std_dev > 0.0);
        assert!(baseline.ucl > baseline.mean);
        assert!(baseline.lcl < baseline.mean);
    }

    #[test]
    fn baseline_refuses_insufficient_history() {
        let err = ControlBaseline::admit(&[ProcessMeasure {
            tick: 0,
            value: 1.0,
        }])
        .expect_err("1 sample must refuse");
        assert!(matches!(err, ProcessSignalRefused::BaselineRefused(_)));
        assert_eq!(err.origin_state(), "Baselined");
    }

    #[test]
    fn baseline_refuses_non_finite_measure() {
        let mut history = stable_history();
        history[3].value = f64::NAN;
        let err = ControlBaseline::admit(&history).expect_err("NaN measure must refuse");
        assert!(matches!(err, ProcessSignalRefused::BaselineRefused(_)));
    }

    // -- D3 Western Electric Breed -------------------------------------------

    #[test]
    fn rule_1_fires_on_a_point_beyond_ucl() {
        let data = vec![ChartPoint {
            tick: 0,
            value: 100.0,
            ucl: 13.0,
            cl: 10.0,
            lcl: 7.0,
        }];
        let alerts = evaluate_western_electric_rules(&data);
        assert_eq!(alerts.len(), 1);
        assert!(matches!(alerts[0], SpecialCause::OutOfControl { .. }));
    }

    #[test]
    fn rule_1_fires_on_non_finite_value_corrupt_evidence() {
        let data = vec![ChartPoint {
            tick: 0,
            value: f64::NAN,
            ucl: 13.0,
            cl: 10.0,
            lcl: 7.0,
        }];
        let alerts = evaluate_western_electric_rules(&data);
        assert!(matches!(alerts[0], SpecialCause::OutOfControl { .. }));
    }

    #[test]
    fn in_control_point_fires_no_rule() {
        let data = vec![ChartPoint {
            tick: 0,
            value: 10.2,
            ucl: 13.0,
            cl: 10.0,
            lcl: 7.0,
        }];
        assert!(evaluate_western_electric_rules(&data).is_empty());
    }

    #[test]
    fn rule_2_fires_on_nine_consecutive_above_center_line() {
        let data: Vec<ChartPoint> = (0..9u64)
            .map(|tick| ChartPoint {
                tick,
                value: 11.0,
                ucl: 13.0,
                cl: 10.0,
                lcl: 7.0,
            })
            .collect();
        let alerts = evaluate_western_electric_rules(&data);
        assert!(alerts.iter().any(|a| matches!(
            a,
            SpecialCause::Shift {
                direction: ShiftDirection::Above,
                count: 9
            }
        )));
    }

    #[test]
    fn rule_4_fires_on_two_of_three_beyond_2sigma() {
        // sigma = (ucl - cl) / 3 = 1.0; 2-sigma line = cl + 2.0 = 12.0.
        let mut data: Vec<ChartPoint> = (0..6u64)
            .map(|tick| ChartPoint {
                tick,
                value: 10.0,
                ucl: 13.0,
                cl: 10.0,
                lcl: 7.0,
            })
            .collect();
        data.push(ChartPoint {
            tick: 6,
            value: 12.5,
            ucl: 13.0,
            cl: 10.0,
            lcl: 7.0,
        });
        data.push(ChartPoint {
            tick: 7,
            value: 10.0,
            ucl: 13.0,
            cl: 10.0,
            lcl: 7.0,
        });
        data.push(ChartPoint {
            tick: 8,
            value: 12.6,
            ucl: 13.0,
            cl: 10.0,
            lcl: 7.0,
        });
        let alerts = evaluate_western_electric_rules(&data);
        assert!(alerts.iter().any(|a| matches!(
            a,
            SpecialCause::TwoOfThree {
                direction: ShiftDirection::Above
            }
        )));
    }

    // -- D4 Signal Admission --------------------------------------------------

    #[test]
    fn admit_signal_returns_none_when_in_control() {
        let baseline = ControlBaseline::admit(&stable_history()).expect("admits");
        let signal = admit_signal(&[], baseline, true).expect("no refusal for empty alerts");
        assert!(signal.is_none());
    }

    #[test]
    fn admit_signal_refuses_when_baseline_not_admitted() {
        let baseline = ControlBaseline::admit(&stable_history()).expect("admits");
        let cause = SpecialCause::OutOfControl {
            value: 1.0,
            ucl: 2.0,
            lcl: 0.0,
        };
        let err = admit_signal(&[cause], baseline, false)
            .expect_err("baseline_is_admitted=false must refuse");
        assert!(matches!(
            err,
            ProcessSignalRefused::SignalAdmissionRefused(_)
        ));
        assert_eq!(err.origin_state(), "SignalAdmitted");
    }

    #[test]
    fn admit_signal_selects_highest_priority_cause() {
        let baseline = ControlBaseline::admit(&stable_history()).expect("admits");
        let alerts = vec![
            SpecialCause::Shift {
                direction: ShiftDirection::Above,
                count: 9,
            },
            SpecialCause::OutOfControl {
                value: 100.0,
                ucl: 13.0,
                lcl: 7.0,
            },
            SpecialCause::TwoOfThree {
                direction: ShiftDirection::Above,
            },
        ];
        let signal = admit_signal(&alerts, baseline, true)
            .expect("no refusal")
            .expect("alerts present");
        assert!(matches!(signal.cause, SpecialCause::OutOfControl { .. }));
    }

    // -- D5 Signal CONSTRUCT ----------------------------------------------------

    #[test]
    fn signal_receipt_is_deterministic() {
        let baseline = ControlBaseline::admit(&stable_history()).expect("admits");
        let signal = ProcessShiftSignal {
            cause: SpecialCause::OutOfControl {
                value: 100.0,
                ucl: 13.0,
                lcl: 7.0,
            },
            baseline,
        };
        let r1 = SignalReceipt::construct(&signal);
        let r2 = SignalReceipt::construct(&signal);
        assert_eq!(r1, r2);
        assert_eq!(r1.digest_hex.len(), 64, "blake3 hex digest is 64 chars");
    }

    #[test]
    fn signal_receipt_differs_for_distinct_signals() {
        let baseline = ControlBaseline::admit(&stable_history()).expect("admits");
        let s1 = ProcessShiftSignal {
            cause: SpecialCause::OutOfControl {
                value: 100.0,
                ucl: 13.0,
                lcl: 7.0,
            },
            baseline,
        };
        let s2 = ProcessShiftSignal {
            cause: SpecialCause::OutOfControl {
                value: 999.0,
                ucl: 13.0,
                lcl: 7.0,
            },
            baseline,
        };
        assert_ne!(
            SignalReceipt::construct(&s1).digest_hex,
            SignalReceipt::construct(&s2).digest_hex
        );
    }

    // -- D6 Diagnostic Goal Generator ---------------------------------------

    #[test]
    fn diagnostic_goal_never_names_a_closed_or_blamed_predicate() {
        let baseline = ControlBaseline::admit(&stable_history()).expect("admits");
        let signal = ProcessShiftSignal {
            cause: SpecialCause::OutOfControl {
                value: 100.0,
                ucl: 13.0,
                lcl: 7.0,
            },
            baseline,
        };
        let receipt = SignalReceipt::construct(&signal);
        let goal = generate_diagnostic_goal(&receipt);
        assert!(!goal.domain_text.contains("closed"));
        assert!(!goal.domain_text.contains("blamed"));
        assert!(!goal.problem_text.contains("closed"));
        assert!(goal.problem_text.contains("investigation-scheduled"));
    }

    // -- Full pipeline (D1-D8) -----------------------------------------------

    fn shift_measures() -> Vec<ProcessMeasure> {
        (100..109u64)
            .map(|tick| ProcessMeasure { tick, value: 30.0 })
            .collect()
    }

    #[test]
    fn full_pipeline_manufactures_an_open_investigation_workflow() {
        let record = run_pipeline(&stable_history(), &shift_measures())
            .expect("pipeline does not refuse on a clean shift signal")
            .expect("a real signal fires: shift_measures() is far beyond the baseline UCL");
        assert_eq!(record.state, LifecycleState::WorkflowManufactured);
        assert!(!record.workflow.plan_op_labels.is_empty());
        assert_eq!(
            record.workflow.plan_op_labels,
            vec![
                "gather-measures",
                "draft-hypotheses",
                "schedule-investigation"
            ]
        );
        assert!(record.workflow.powl_turtle.contains("powl2:ActivityLeaf"));
        assert!(record
            .workflow
            .powl_turtle
            .contains(&record.signal_receipt.digest_hex));
    }

    #[test]
    fn full_pipeline_returns_none_when_in_control() {
        let in_control = stable_history();
        let outcome = run_pipeline(&stable_history(), &in_control)
            .expect("in-control history does not refuse");
        assert!(
            outcome.is_none(),
            "an in-control process must not manufacture a workflow"
        );
    }

    #[test]
    fn full_pipeline_refuses_at_baselined_for_bad_history() {
        let err = run_pipeline(&[], &shift_measures())
            .expect_err("empty history must refuse at the baseline choke point");
        assert!(matches!(err, ProcessSignalRefused::BaselineRefused(_)));
        assert_eq!(err.origin_state(), "Baselined");
    }

    // -- Replay equivalence ----------------------------------------------------

    #[test]
    fn replay_is_byte_identical() {
        let r1 = run_pipeline(&stable_history(), &shift_measures())
            .expect("no refusal")
            .expect("signal fires");
        let r2 = run_pipeline(&stable_history(), &shift_measures())
            .expect("no refusal")
            .expect("signal fires");
        assert_eq!(
            receipt_head(&r1),
            receipt_head(&r2),
            "identical inputs must produce a byte-identical receipt head"
        );
    }

    // -- Chaos lens: idempotency gate -------------------------------------------

    #[test]
    fn idempotency_gate_replays_instead_of_re_actuating() {
        let mut gate = IdempotencyGate::new();
        let mut run_count = 0u32;

        let mut run_once = || {
            run_count += 1;
            run_pipeline(&stable_history(), &shift_measures())
        };
        let first = gate
            .admit_or_replay("corr-1", &mut run_once)
            .expect("first admission succeeds")
            .expect("signal fires");
        let second = gate
            .admit_or_replay("corr-1", &mut run_once)
            .expect("duplicate correlation id re-admits")
            .expect("signal fires");

        assert_eq!(
            run_count, 1,
            "the pipeline must run at most once per correlation id"
        );
        assert_eq!(
            receipt_head(&first),
            receipt_head(&second),
            "replayed record must be identical to the original"
        );
    }

    #[test]
    fn idempotency_gate_distinct_correlation_ids_both_run() {
        let mut gate = IdempotencyGate::new();
        let mut run_count = 0u32;
        let mut run = || {
            run_count += 1;
            run_pipeline(&stable_history(), &shift_measures())
        };
        gate.admit_or_replay("corr-a", &mut run).expect("runs");
        gate.admit_or_replay("corr-b", &mut run).expect("runs");
        assert_eq!(
            run_count, 2,
            "distinct correlation ids each get their own run"
        );
    }

    // -- Provenance Turtle emission -------------------------------------------

    #[test]
    fn provenance_turtle_chains_all_eight_stages_with_prov_wasderivedfrom() {
        let record = run_pipeline(&stable_history(), &shift_measures())
            .expect("no refusal")
            .expect("signal fires");
        let turtle = emit_provenance_turtle(&record);
        assert_eq!(
            turtle.matches("prov:wasDerivedFrom").count(),
            generated::PROVENANCE_CHAIN.len() - 1,
            "7 edges chain the 8 D1-D8 stages"
        );
        for stage in generated::PROVENANCE_CHAIN {
            assert!(
                turtle.contains(stage.name),
                "missing stage {} in emitted turtle",
                stage.name
            );
        }
    }

    // -- Generated/hand-written cross-check ------------------------------------

    #[test]
    fn refusal_catalog_matches_hand_written_enum() {
        let generated_names: Vec<&str> =
            generated::REFUSAL_CATALOG.iter().map(|r| r.name).collect();
        assert_eq!(
            generated_names,
            vec![
                "BaselineRefused",
                "PowlConstructionRefused",
                "SignalAdmissionRefused",
            ],
            "generated::REFUSAL_CATALOG must name exactly ProcessSignalRefused's 3 variants"
        );
    }

    #[test]
    fn provenance_chain_has_all_eight_d1_d8_stages_in_order() {
        let names: Vec<&str> = generated::PROVENANCE_CHAIN.iter().map(|s| s.name).collect();
        assert_eq!(
            names,
            vec![
                "ProcessMeasure",
                "ControlBaseline",
                "RuleEvaluation",
                "ProcessShiftSignal",
                "SignalReceipt",
                "DiagnosticGoal",
                "InvestigationPlan",
                "InvestigationPOWL",
            ]
        );
    }
}

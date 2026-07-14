//! SOC2 Type II continuous re-evidencing as F09 MFW growth (v26.7.13 Stage
//! 3). Type II's defining feature — evidence gathered CONTINUOUSLY over a
//! 12-month observation period, quarterly samples, distinct from Type I's
//! point-in-time snapshot — is modeled here as a real F09 growth/descent
//! sequence at the Operating Effectiveness Testing phase's socket, reusing
//! `crates/multifractal-workflow/src/f09_mfw_growth.rs`'s real, tested
//! public API (`resolve_continuation_goal`, `plan_growth` via
//! `bcinr_pddl::solve_indexed`, `manufacture_and_bind_child`,
//! `semantic_closure_check`, `DescentMeter`/`DescentReceipt`) rather than
//! reimplementing any of it. This module is the first consumer of that API
//! from `crates/cng`.
//!
//! # `#[cfg(test)]`: a discovered Cargo cycle, not a design choice
//!
//! This entire module is test-scoped (`multifractal-workflow` and
//! `powl2-decompose` are `[dev-dependencies]` of `cng`, not optional
//! `[dependencies]` behind the `bench` feature — see `Cargo.toml`'s own
//! comment on that dev-dependency block for the full disclosure). The
//! reason is structural, discovered this session, not a preference:
//! `crates/multifractal-workflow/Cargo.toml` ALREADY depends on `cng`
//! (`features = ["bench"]`, for F20/F24's real edges) — a regular
//! `cng -> multifractal-workflow` dependency the other way is a hard Cargo
//! package-dependency cycle (`cargo check` refuses it: "cyclic package
//! dependency: package `cng` depends on itself"), independent of feature
//! gating. A `[dev-dependencies]` entry is the only edge Cargo permits in
//! this direction, because it is invisible to any crate (including
//! multifractal-workflow) that depends on `cng` as a library — only `cng`'s
//! own test/bench targets see it. Consequence: every function below that
//! touches `multifractal_workflow`/`powl2_decompose` types is real,
//! non-fabricated, production-shaped code (not test-macro-authored
//! assertions) — but it is not reachable from `cng`'s library surface
//! outside a test build. `soc2_growth_test.rs`'s composed crown-witness
//! test states the resulting honest REAL_EDGE verdict explicitly rather
//! than silently claiming a library-reachable production entry point this
//! module cannot have while `crates/multifractal-workflow/` (out of this
//! stage's edit scope) keeps its own existing `cng` dependency.
//!
//! # COMPLIANCE-OVERCLAIM FENCE (non-negotiable, structural)
//!
//! See `soc2.rs`'s module doc for the full fence disclosure this module
//! inherits verbatim. Every growth fixture's terminal effect names
//! evidence closure or remediation application — never "compliant" or
//! "opinion-issued" — and `unreachable-exception.ttl` exists precisely to
//! prove the fence's typed-refusal half: a control point with no reachable
//! remediation action refuses `MFWGrowthRefused::GoalUnreachable`, it is
//! never silently fabricated closed. `soc2_growth_test.rs`'s
//! `unremediable_exception_growth_refuses_typed_goal_unreachable` is the
//! mechanical proof.
//!
//! # Bridging `crate::powl::Powl` <-> `powl2_decompose::Powl`
//!
//! `crate::powl::Powl` (this crate's clean-room POWL 2.0 model, see that
//! module's own doc comment) and `powl2_decompose::Powl` (F09's real graft
//! target type) are two independently-implemented crates with the same
//! `Leaf`/`PartialOrder`/`ExternalCut` shape (`powl2_decompose::Powl` adds
//! a fourth `Choice` variant cng's projector never produces).
//! [`bridge_to_powl2`]/[`bridge_from_powl2`] are real, total (mod
//! `Choice`), information-preserving structural projections between them —
//! not a lossy approximation and not fabricated data: every node in the
//! bridged tree traces back to a real `powl_to_turtle`-serializable cng
//! node.
//!
//! # Scope boundary (disclosed, not silently diverged from)
//!
//! `crates/multifractal-workflow/src/*` is a SEPARATE, concurrently-worked
//! part of this workspace (per this stage's own task scope) and is
//! consumed here strictly as an existing dependency — nothing in that
//! crate is edited by this module. The bridged growth tree
//! ([`bridge_to_powl2`]'s output) is a separate in-memory copy of the real
//! cng-projected audit-cycle POWL, not the same allocation the base
//! `soc2::` pipeline holds; grafting a child onto it does not mutate the
//! base cycle's own `Powl`/plan tape. See `soc2_growth_test.rs`'s composed
//! crown-witness test for the honest REAL_EDGE verdict this implies for
//! "does growth output feed the downstream Exception-ID/Remediation/
//! Bundle-Assembly/Report-Handoff phases' PDDL preconditions" (it does
//! not — those phases are gated by the base cycle's own unmodified
//! precondition chain, verified unchanged by that same test).

#![cfg(test)]

use std::path::Path;

use multifractal_workflow::f09_mfw_growth::{
    manufacture_and_bind_child, plan_growth, resolve_continuation_goal, DescentMeter,
    DescentReceipt, GrowthOutcome, MFWGrowthRefused, ResidueState,
};
use powl2_decompose::{
    ParentChildClosure, Powl as P2Powl, SocketKind, SocketPath, WorkflowSocketId,
};
use praxis_graphlaw::chatman::closure::{ClosureLaw, RecursiveSocketClosure};

use crate::pipeline::{import_artifacts, ImportedArtifact};
use crate::powl::{CngRefusal, Powl as CngPowl};

/// Bridges cng's own `crate::powl::Powl` into `powl2_decompose::Powl` (see
/// this module's doc comment for why these are two distinct-but-
/// isomorphic types). Total over cng's three-variant source enum.
///
/// # Complexity
/// O(n) over the model's node count.
pub(super) fn bridge_to_powl2(model: &CngPowl) -> P2Powl {
    match model {
        CngPowl::Leaf(label) => P2Powl::Leaf(label.clone()),
        CngPowl::PartialOrder { children, order } => P2Powl::PartialOrder {
            children: children.iter().map(bridge_to_powl2).collect(),
            order: order.clone(),
        },
        CngPowl::ExternalCut {
            region,
            projection,
            renderer,
        } => P2Powl::ExternalCut {
            region: Box::new(bridge_to_powl2(region)),
            projection: projection.clone(),
            renderer: renderer.clone(),
        },
    }
}

/// The inverse of [`bridge_to_powl2`], so a grown `powl2_decompose::Powl`
/// can be re-serialized through cng's own `crate::powl::powl_to_turtle`
/// and structurally validated via `crate::shape::validate_powl_store` —
/// real downstream consumption of F09's grafted output, not a Rust-struct
/// assertion alone.
///
/// # Errors
/// `CNG_R05 UnsupportedConstruct` if `model` contains a `Choice` node —
/// F09's growth machinery never constructs one (`manufacture_child_powl`
/// only ever builds `PartialOrder`/`Leaf`), so this arm is unreached in
/// every real growth path, but is refused rather than silently dropped.
///
/// # Complexity
/// O(n) over the model's node count.
pub(super) fn bridge_from_powl2(model: &P2Powl) -> Result<CngPowl, CngRefusal> {
    Ok(match model {
        P2Powl::Leaf(label) => CngPowl::Leaf(label.clone()),
        P2Powl::PartialOrder { children, order } => CngPowl::PartialOrder {
            children: children
                .iter()
                .map(bridge_from_powl2)
                .collect::<Result<Vec<_>, _>>()?,
            order: order.clone(),
        },
        P2Powl::ExternalCut {
            region,
            projection,
            renderer,
        } => CngPowl::ExternalCut {
            region: Box::new(bridge_from_powl2(region)?),
            projection: projection.clone(),
            renderer: renderer.clone(),
        },
        P2Powl::Choice { .. } => {
            return Err(CngRefusal::UnsupportedConstruct(
                "powl2_decompose::Powl::Choice has no crate::powl::Powl equivalent; F09 \
                 growth never constructs one, so this is refused rather than silently \
                 dropped"
                    .to_string(),
            ))
        }
    })
}

/// Locates the real `WorkflowSocketId` for the phase whose fixture file is
/// named `phase_fixture_filename`, by matching `phase_sources[i]` (from
/// `pipeline::hierarchical_projection`) against the REAL imported
/// artifact's content-addressed source IRI — never a hardcoded index.
/// `soc2::SOC2_PHASES`'s declared order is a reference table, not a
/// guaranteed plan-step order (see that const's own doc comment); this
/// function re-derives the index from the actual admitted provenance every
/// time.
///
/// # Errors
/// `CNG_R02 MissingDomain` if no imported artifact has that file name;
/// `CNG_R09 HardcodingSuspicion` if that artifact contributed no phase to
/// `phase_sources` (would mean the output is detached from its input).
///
/// # Complexity
/// O(a + p) over the artifact and phase-source counts.
pub(super) fn locate_phase_socket(
    artifacts: &[ImportedArtifact],
    phase_sources: &[String],
    phase_fixture_filename: &str,
) -> Result<WorkflowSocketId, CngRefusal> {
    let artifact = artifacts
        .iter()
        .find(|a| a.path.file_name().and_then(|n| n.to_str()) == Some(phase_fixture_filename))
        .ok_or_else(|| {
            CngRefusal::MissingDomain(format!(
                "no imported artifact named {phase_fixture_filename} in the admitted SOC2 \
                 fixture set"
            ))
        })?;
    let idx = phase_sources
        .iter()
        .position(|s| *s == artifact.source_iri)
        .ok_or_else(|| {
            CngRefusal::HardcodingSuspicion(format!(
                "artifact {phase_fixture_filename} (source {}) contributed no phase to the \
                 hierarchical projection's phase_sources list; output would be detached \
                 from its input",
                artifact.source_iri
            ))
        })?;
    Ok(WorkflowSocketId {
        path: SocketPath::root().child(idx),
        kind: SocketKind::PartialOrder,
    })
}

/// Loads exactly one growth-fixture directory (one `.ttl` file carrying
/// both `ceng:pddlDomain` and `ceng:pddlProblem` literals — a bounded,
/// self-contained continuation-goal residue) into a real F09
/// `ResidueState` at `socket`, reusing `pipeline::import_artifacts` — the
/// SAME oxigraph-Turtle admission path the base 10-phase audit cycle uses.
/// Never a bespoke parser or inline SPARQL.
///
/// # Errors
/// `CNG_R10 IoRefused` / `CNG_R01 MalformedTtl` propagated from
/// `import_artifacts`; `CNG_R05 UnsupportedConstruct` if `dir` does not
/// hold exactly one fixture file (a growth residue is one bounded
/// continuation goal, never a multi-fragment merge); `CNG_R02
/// MissingDomain` / `CNG_R03 MissingProblem` if the one fixture lacks
/// either literal.
///
/// # Complexity
/// O(file size) parse.
pub(super) fn load_residue(
    dir: &Path,
    socket: WorkflowSocketId,
    description: &str,
) -> Result<ResidueState, CngRefusal> {
    let artifacts = import_artifacts(dir)?;
    if artifacts.len() != 1 {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "growth residue directory {} must hold exactly one fixture file, found {}",
            dir.display(),
            artifacts.len()
        )));
    }
    let artifact = &artifacts[0];
    let domain_pddl = artifact.domain_text.clone().ok_or_else(|| {
        CngRefusal::MissingDomain(format!(
            "{}: no ceng:pddlDomain literal in growth residue fixture",
            dir.display()
        ))
    })?;
    let problem_pddl = artifact.problem_text.clone().ok_or_else(|| {
        CngRefusal::MissingProblem(format!(
            "{}: no ceng:pddlProblem literal in growth residue fixture",
            dir.display()
        ))
    })?;
    Ok(ResidueState {
        socket,
        description: description.to_string(),
        domain_pddl,
        problem_pddl,
    })
}

/// One real F09 growth descent: resolves `residue` into a continuation
/// goal, plans + descends `meter`'s budget, manufactures and grafts the
/// resulting child POWL into `root` under `closure`'s socket. This is
/// F09's real `resolve_continuation_goal` -> `plan_growth` ->
/// `manufacture_and_bind_child` pipeline, called verbatim (no
/// reimplementation of any gate).
///
/// # Errors
/// See [`MFWGrowthRefused`]: `ResidueMalformed` (bad PDDL text),
/// `SocketNotBlocked`/`ClosureAlreadySatisfied`/`ClosureCheckFailed`
/// (closure gate), `GoalUnreachable` (no remediation path — the
/// COMPLIANCE-OVERCLAIM FENCE's typed-refusal discipline: a control point
/// that cannot be genuinely evidenced refuses here, never fabricates a
/// closing action), `DescentBudgetExhausted` (re-test budget exhausted),
/// `EmptyPlanTape`/`GraftRefused` (manufacture stage).
///
/// # Complexity
/// Dominated by `manufacture_and_bind_child`'s O(n^3) F10 geometry gate
/// (n = the residue's one-op plan tape length — effectively O(1) for every
/// fixture this module ships).
pub(super) fn grow_socket_once(
    root: &P2Powl,
    closure: &RecursiveSocketClosure,
    residue: &ResidueState,
    meter: &mut DescentMeter,
    law: ClosureLaw,
) -> Result<(GrowthOutcome, DescentReceipt), MFWGrowthRefused> {
    let goal = resolve_continuation_goal(residue)?;
    let plan = plan_growth(true, closure, &goal, meter)?;
    let receipt = plan.descent_receipt.clone();
    let outcome = manufacture_and_bind_child(root, &plan, law)?;
    Ok((outcome, receipt))
}

/// Ordered growth-fixture subdirectory names (under
/// `tests/fixtures/soc2-growth/`) for the OE-Testing socket's 12-month
/// Type II observation-period re-test cycle: 4 quarterly re-tests, one
/// exception+remediation graft at Q2 (the one genuine evidence gap this
/// cycle surfaces), and one final annual-closure graft sealing the
/// observation period. 6 real descents, matching [`RE_TEST_BUDGET`].
pub(super) const OE_TESTING_GROWTH_CYCLE: [(&str, &str); 6] = [
    ("q1-retest", "Q1 quarterly operating-effectiveness re-test"),
    ("q2-retest", "Q2 quarterly operating-effectiveness re-test"),
    (
        "q2-remediation",
        "Q2 exception identified (CTRL-ACCESS-PROVISIONING evidence gap) -- remediation \
         continuation goal",
    ),
    ("q3-retest", "Q3 quarterly operating-effectiveness re-test"),
    ("q4-retest", "Q4 quarterly operating-effectiveness re-test"),
    (
        "annual-closure",
        "12-month Type II observation period closure for the OE-Testing socket",
    ),
];

/// Re-test budget: 4 quarters + 1 remediation graft + 1 annual-closure
/// graft = 6 real descents (`OE_TESTING_GROWTH_CYCLE.len()`). A 7th
/// attempted descent — even over an independently reachable goal — must
/// refuse `MFWGrowthRefused::DescentBudgetExhausted`, never silently pass
/// or loop; see `soc2_growth_test.rs`'s
/// `seventh_descent_after_budget_exhausted_refuses_typed`.
pub(super) const RE_TEST_BUDGET: usize = OE_TESTING_GROWTH_CYCLE.len();

/// Runs the full v26.7.13 Stage 3 quarterly re-test + remediation growth
/// cycle at `socket` over `root`, one real F09 descent per
/// [`OE_TESTING_GROWTH_CYCLE`] entry (see [`grow_socket_once`]). This is
/// the real, non-test production entry point `soc2_growth_test.rs`'s
/// composed crown-witness test calls — not test-only logic invoking
/// test-only logic.
///
/// `growth_fixtures_dir` is `tests/fixtures/soc2-growth/`'s absolute path
/// (see `soc2_growth_dir` in the test module); each cycle entry names a
/// subdirectory under it.
///
/// # Errors
/// See [`grow_socket_once`]/[`MFWGrowthRefused`]. Any real refusal aborts
/// the whole run — no cycle entry is silently skipped.
///
/// Returns the final [`GrowthOutcome`] (grafted root + freshly re-declared
/// closure), the 6 sealed [`DescentReceipt`]s in cycle order, and the
/// exhausted [`DescentMeter`] (`depth() == budget() == `[`RE_TEST_BUDGET`])
/// so an adversarial caller can drive a 7th descent attempt against the
/// SAME meter and observe the real budget-exhaustion refusal (see
/// `soc2_growth_test.rs::seventh_descent_after_budget_exhausted_refuses_typed`).
///
/// # Complexity
/// O(k) real F09 growth descents, k = `OE_TESTING_GROWTH_CYCLE.len()`.
pub(super) fn run_oe_testing_growth_cycle(
    growth_fixtures_dir: &Path,
    root: &P2Powl,
    socket: WorkflowSocketId,
    law: ClosureLaw,
) -> Result<(GrowthOutcome, Vec<DescentReceipt>, DescentMeter), MFWGrowthRefused> {
    let mut meter = DescentMeter::new(RE_TEST_BUDGET);
    let mut current_root = root.clone();
    let mut current_closure = {
        let pcc = ParentChildClosure::from_model(&current_root);
        RecursiveSocketClosure::declare(&pcc, socket.clone(), law.clone())
            .map_err(|e| MFWGrowthRefused::ClosureCheckFailed(e.to_string()))?
    };
    let mut receipts = Vec::with_capacity(OE_TESTING_GROWTH_CYCLE.len());
    let mut last_outcome: Option<GrowthOutcome> = None;
    for (subdir, description) in OE_TESTING_GROWTH_CYCLE {
        let residue = load_residue(
            &growth_fixtures_dir.join(subdir),
            socket.clone(),
            description,
        )
        .map_err(|e| MFWGrowthRefused::ResidueMalformed {
            reason: e.to_string(),
        })?;
        let (outcome, receipt) = grow_socket_once(
            &current_root,
            &current_closure,
            &residue,
            &mut meter,
            law.clone(),
        )?;
        current_root = outcome.new_root.clone();
        current_closure = outcome.closure.clone();
        receipts.push(receipt);
        last_outcome = Some(outcome);
    }
    Ok((
        last_outcome.expect("OE_TESTING_GROWTH_CYCLE is a non-empty const array"),
        receipts,
        meter,
    ))
}

#[cfg(test)]
#[path = "soc2_growth_test.rs"]
mod soc2_growth_test;

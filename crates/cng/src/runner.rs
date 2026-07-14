//! Runner connection: prove the projected POWL v2 workflow executes on the
//! published `bcinr-powl` runtime (crates.io 26.6.25).
//!
//! The adapter path is the smallest honest one the published crate exposes:
//! the linear `Powl::PartialOrder` model is lowered to
//! `bcinr_powl::compiler::PowlAstNode::PartialOrder`, compiled with
//! `bcinr_powl::compiler::compile_powl` (which runs bcinr-powl's own
//! Kahn's-algorithm acyclicity and reachability admission checks), and then
//! driven to completion with `bcinr_powl::scheduler::{PowlRunState,
//! scheduler_tick}`. `validated: true` is only reported when every activity
//! slot on the compiled tape actually fired under that scheduler.
//!
//! Determinism: the scheduler is branchless integer arithmetic over fixed
//! bitmasks; no wall clock, no randomness, no I/O. Same inputs produce the
//! same `RunnerReport` bytes.
//!
//! Hierarchical models (increment 1): the two-level PartialOrder-of-
//! PartialOrders shape is real in the serialized POWL artifact — phases and
//! their provenance survive in the Turtle. Execution against the published
//! bcinr-powl 26.6.25 runtime, however, goes through *lawful flattening*
//! ([`linearize_hierarchical`]), not native nested `PowlAstNode`
//! composition: the phase structure is depth-first linearized into the
//! exact label sequence and transitively closed edge set the equivalent
//! flat projection would produce, and that flat form is what
//! `compile_powl`/`scheduler_tick` admit and run.

use crate::powl::{CngRefusal, Powl};
use bcinr_pddl::Pddl8Tape;
use bcinr_powl::compiler::{compile_powl, CompileError, PowlAstNode};
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use bcinr_powl::tape::OpKind;

/// Version of the published runner crate this adapter targets. Kept as a
/// constant so the report string is not asserted ad hoc at call sites.
const RUNNER_VERSION: &str = "26.6.25";

/// Report of one admission + execution pass on the bcinr-powl runtime.
///
/// `validated` is `true` only when `compile_powl` accepted the workflow AND
/// the scheduler fired every activity slot; it is never asserted without a
/// real run.
#[derive(Debug, serde::Serialize)]
pub struct RunnerReport {
    /// Runner identity, e.g. `"bcinr-powl 26.6.25"`.
    pub runner: String,
    /// `true` iff admission (compile) and execution (scheduler) both succeeded.
    pub validated: bool,
    /// `true` iff the observed firing order was a linear extension of the
    /// projected order relation — the generated workflow is accepted as a
    /// conformance artifact, not merely executed.
    pub conformant: bool,
    /// Number of activity ops that actually fired on the scheduler.
    pub executed_ops: usize,
    /// What was actually proven, in one sentence.
    pub detail: String,
}

/// Extracted activity labels paired with their order-relation edges
/// ((from, to) index pairs into the label vector) — the shared return shape
/// of [`model_to_labels_and_edges`] and [`linearize_hierarchical`].
type LabelsAndEdges<'a> = (Vec<&'a str>, Vec<(usize, usize)>);

/// Extract the activity labels and order edges from the projected model.
///
/// Only the shape the linear projection produces is adapted: a top-level
/// `PartialOrder` whose children are all `Leaf(Some(label))`, or a single
/// `Leaf(Some(label))`. Any other shape is a typed refusal naming the gap —
/// no partial adaptation, no silent skipping of children.
///
/// # Errors
/// `CNG_R05 UnsupportedConstruct` for silent leaves and nested composite
/// children, or order pairs whose indices are out of range.
///
/// # Complexity
/// O(n + |order|) where n is the child count.
fn model_to_labels_and_edges(model: &Powl) -> Result<LabelsAndEdges<'_>, CngRefusal> {
    match model {
        Powl::Leaf(Some(label)) => Ok((vec![label.as_str()], Vec::new())),
        Powl::Leaf(None) => Err(CngRefusal::UnsupportedConstruct(
            "bcinr-powl runner adapter requires named activities; \
             POWL v2 model is a single silent leaf"
                .to_string(),
        )),
        Powl::ExternalCut { .. } => Err(CngRefusal::UnsupportedConstruct(
            "bcinr-powl runner adapter requires named activities; \
             POWL v2 model is an external cut"
                .to_string(),
        )),
        Powl::PartialOrder { children, order } => {
            let mut labels = Vec::with_capacity(children.len());
            for (i, child) in children.iter().enumerate() {
                match child {
                    Powl::Leaf(Some(label)) => labels.push(label.as_str()),
                    Powl::Leaf(None) => {
                        return Err(CngRefusal::UnsupportedConstruct(format!(
                            "bcinr-powl runner adapter requires named activity \
                             leaves; child {i} is a silent leaf"
                        )));
                    }
                    Powl::PartialOrder { .. } => {
                        return Err(CngRefusal::UnsupportedConstruct(format!(
                            "bcinr-powl 26.6.25 adapter covers the flat linear \
                             projection only; child {i} is a nested PartialOrder"
                        )));
                    }
                    Powl::ExternalCut { .. } => {
                        return Err(CngRefusal::UnsupportedConstruct(format!(
                            "bcinr-powl 26.6.25 adapter covers the flat linear \
                             projection only; child {i} is an external cut"
                        )));
                    }
                }
            }
            for &(from, to) in order.iter() {
                if from >= children.len() || to >= children.len() {
                    return Err(CngRefusal::InvalidPowl(format!(
                        "POWL v2 order pair ({from}, {to}) is out of range for \
                         {} children",
                        children.len()
                    )));
                }
            }
            Ok((labels, order.iter().copied().collect()))
        }
    }
}

/// Map a bcinr-powl `CompileError` to a typed refusal naming the exact
/// admission surface that rejected the workflow.
///
/// # Complexity
/// O(1).
fn compile_error_to_refusal(err: CompileError) -> CngRefusal {
    CngRefusal::RunnerMismatch(format!(
        "bcinr-powl 26.6.25 compile_powl refused the workflow: {err:?}"
    ))
}

/// Adapt the projected linear workflow to the bcinr-powl runtime and run it.
///
/// Pipeline (all on real published-crate surfaces, nothing hand-authored):
/// 1. Lower `model` to `PowlAstNode::PartialOrder { children, edges }` with
///    one `Atom` per activity leaf and the model's (transitively closed)
///    order relation as explicit edges.
/// 2. Admit via `bcinr_powl::compiler::compile_powl` — bcinr-powl's own
///    Kahn's-algorithm acyclicity and reachability checks run here.
/// 3. Execute via `bcinr_powl::scheduler::scheduler_tick` until the ready
///    set drains, bounded at `2 * tape.len + 2` ticks (the scheduler's own
///    documented termination bound for loop-free tapes).
/// 4. Cross-check: the fired activity count must equal both the model leaf
///    count and the source `Pddl8Tape` op count.
///
/// # Errors
/// `CNG_R04 PlanUnsolvable` for an empty tape;
/// `CNG_R07 RunnerMismatch` when the model/tape disagree, compile refuses,
/// or execution does not conform; `CNG_R05` when the model
/// exceeds the runtime's 64-slot tape, `compile_powl` refuses, or the
/// scheduler fails to fire every activity within the tick bound.
///
/// # Complexity
/// O(n²) in activity count n: the closed order relation has O(n²) pairs and
/// the bounded scheduler loop is O(n) ticks of O(n) work each.
pub fn validate_run(tape: &Pddl8Tape, model: &Powl) -> Result<RunnerReport, CngRefusal> {
    if tape.ops.is_empty() {
        return Err(CngRefusal::PlanUnsolvable(
            "empty PDDL plan tape: nothing to execute on the bcinr-powl runtime".to_string(),
        ));
    }
    let (labels, edges) = model_to_labels_and_edges(model)?;
    run_labels_and_edges(tape, labels, edges)
}

/// Shared post-extraction runner path for both the flat and the
/// hierarchical entry points: bind labels to the source tape, cap at the
/// 64-slot PowlTape, lower to bcinr-powl's AST, compile (admission), drive
/// the scheduler to completion, and compute the conformance verdict.
///
/// # Errors
/// `CNG_R07 RunnerMismatch` when the labels/tape disagree, compile refuses,
/// or execution does not conform; `CNG_R05 UnsupportedConstruct` past the
/// 64-slot tape cap.
///
/// # Complexity
/// O(n²) in activity count n: the closed order relation has O(n²) pairs and
/// the bounded scheduler loop is O(n) ticks of O(n) work each.
fn run_labels_and_edges(
    tape: &Pddl8Tape,
    labels: Vec<&str>,
    edges: Vec<(usize, usize)>,
) -> Result<RunnerReport, CngRefusal> {
    // Bind the model to its source tape: same op count, same labels in order.
    if labels.len() != tape.ops.len() {
        return Err(CngRefusal::RunnerMismatch(format!(
            "POWL v2 model has {} activity leaves but the source Pddl8Tape has \
             {} ops; refusing to run a model detached from its plan",
            labels.len(),
            tape.ops.len()
        )));
    }
    for (i, (label, op)) in labels.iter().zip(tape.ops.iter()).enumerate() {
        if *label != op.label {
            return Err(CngRefusal::RunnerMismatch(format!(
                "POWL v2 leaf {i} is '{label}' but Pddl8Tape op {i} is '{}'; \
                 model does not correspond to the plan",
                op.label
            )));
        }
    }

    // bcinr-powl 26.6.25's PowlTape is a fixed 64-slot bitmask tape.
    if labels.len() > 64 {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "bcinr-powl 26.6.25 PowlTape holds at most 64 ops; POWL v2 \
             PartialOrder provides {} activities",
            labels.len()
        )));
    }

    // 1. Lower to bcinr-powl's own AST.
    let children: Vec<PowlAstNode<'_>> = labels.iter().map(|&l| PowlAstNode::Atom(l)).collect();
    let order_pairs = edges.clone();
    let ast = PowlAstNode::PartialOrder { children, edges };

    // 2. Admission: bcinr-powl's compiler validates acyclicity + reachability.
    let compiled = compile_powl(&ast).map_err(compile_error_to_refusal)?;
    let slot_count = compiled.len as usize;

    // Bitmask of activity (Atom) slots; synthetic Silent/Join slots are
    // runtime plumbing, not plan ops.
    let mut atom_mask: u64 = 0;
    for i in 0..slot_count {
        if compiled.ops[i].kind == OpKind::Atom {
            atom_mask |= 1u64 << i;
        }
    }

    // 3. Execution: drive the branchless scheduler until the ready set
    // drains. Loop-free tapes terminate within 2 * len ticks (see
    // bcinr-powl scheduler tests); +2 gives slack for the final drain tick.
    // O(n) ticks of O(n) work each.
    // Predecessor masks over activity slots from the model's own order
    // relation: conformance requires that no activity fires before all of
    // its projected predecessors. The compiled tape places the n activity
    // Atoms in the first n slots in child order, so slot index == model
    // child index for activities. O(|order|).
    let mut pred_masks: Vec<u64> = vec![0; labels.len()];
    for &(from, to) in order_pairs.iter() {
        pred_masks[to] |= 1u64 << from;
    }

    let mut state = PowlRunState::new(&compiled);
    let mut fired_all: u64 = 0;
    let mut conformant = true;
    let max_ticks = 2 * slot_count + 2;
    for _ in 0..max_ticks {
        if state.check_mask == 0 {
            break;
        }
        let fired = scheduler_tick(&compiled.ops[..slot_count], &mut state);
        // Per-tick conformance: each activity fired this tick must have all
        // its projected predecessors fired in earlier ticks OR earlier in
        // this tick's cascade. The scheduler's AND-gate (`pred_satisfied`
        // over done_mask) only enables an op after its predecessors are
        // done, and a tick can cascade a whole ready chain; scanning fired
        // bits in ascending index order matches the projection's index-
        // ordered chain, so same-tick predecessors accumulate into
        // `credited` before their successors are checked.
        let mut credited = fired_all;
        let mut bits = fired.0 & atom_mask;
        while bits != 0 {
            let i = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            if i < pred_masks.len() && (pred_masks[i] & !credited) != 0 {
                conformant = false;
            }
            credited |= 1u64 << i;
        }
        fired_all |= fired.0;
    }

    // 4. Verdict is computed, never asserted: every activity slot must have
    // actually fired on the scheduler.
    let fired_atoms = (fired_all & atom_mask).count_ones() as usize;
    if fired_atoms != labels.len() || state.check_mask != 0 {
        return Err(CngRefusal::RunnerMismatch(format!(
            "bcinr-powl 26.6.25 scheduler fired {fired_atoms} of {} activity \
             ops within {max_ticks} ticks (residual check_mask {:#018x}); \
             workflow did not run to completion",
            labels.len(),
            state.check_mask
        )));
    }

    if !conformant {
        return Err(CngRefusal::RunnerMismatch(format!(
            "bcinr-powl 26.6.25 execution violated the projected order \
             relation: an activity fired before its predecessors among \
             {fired_atoms} ops; the workflow is not conformance-accepted"
        )));
    }

    Ok(RunnerReport {
        runner: format!("bcinr-powl {RUNNER_VERSION}"),
        validated: true,
        conformant,
        executed_ops: fired_atoms,
        detail: format!(
            "compile_powl admitted the projected workflow ({slot_count} tape \
             slots, Kahn acyclicity + reachability checks); scheduler_tick \
             fired all {fired_atoms} activity ops to completion; the observed \
             firing order was a linear extension of the projected precedes \
             relation (conformance-accepted); labels cross-checked against \
             the source Pddl8Tape"
        ),
    })
}

/// Lawfully flattens a two-level hierarchical POWL model (a root
/// `PartialOrder` whose every child is a phase `PartialOrder`, as produced
/// by `project_tape_to_powl_hierarchical`) into the flat label sequence and
/// transitively closed edge set the equivalent flat projection would
/// produce: phases are walked depth-first in root child-index order, leaves
/// within each phase in index order; edges are the intra-phase pairs plus
/// all cross-phase pairs — exactly what `model_to_labels_and_edges` yields
/// for the equivalent flat model.
///
/// Both order relations must be the full transitive closure of their total
/// order (all `(i, j)` with `i < j`); anything else would make flattening a
/// semantic choice rather than a lawful projection, so it is refused.
///
/// # Errors
/// `CNG_R05 UnsupportedConstruct` when the root is not a PartialOrder, a
/// top-level child is not a phase PartialOrder (naming the offending
/// shape), a phase contains a nested composite or a Silent/unlabelled leaf,
/// an order relation is not the full closure of its total order, or the
/// flattened leaf count exceeds bcinr-powl 26.6.25's 64-slot PowlTape.
///
/// # Complexity
/// O(n²) in flattened leaf count n (the emitted edge set is the full
/// closure, C(n, 2) pairs), plus O(p²) root-order verification over p
/// phases.
pub fn linearize_hierarchical(model: &Powl) -> Result<LabelsAndEdges<'_>, CngRefusal> {
    let Powl::PartialOrder {
        children: phases,
        order: root_order,
    } = model
    else {
        return Err(CngRefusal::UnsupportedConstruct(
            "hierarchical linearization requires a root PartialOrder of phase \
             PartialOrders; found a bare Leaf model"
                .to_string(),
        ));
    };

    // Root order must be the transitive closure of the phase total order:
    // exactly the C(p, 2) pairs (i, j) with i < j. O(p²).
    let p = phases.len();
    let full_root = p * p.saturating_sub(1) / 2;
    if root_order.len() != full_root
        || !(0..p).all(|i| ((i + 1)..p).all(|j| root_order.contains(&(i, j))))
    {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "hierarchical linearization requires the root order to be the full \
             transitive closure of the phase total order ({full_root} pairs over \
             {p} phases); found {} pairs",
            root_order.len()
        )));
    }

    let mut labels: Vec<&str> = Vec::new();
    let mut phase_ranges: Vec<(usize, usize)> = Vec::with_capacity(p);
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for (phase_idx, phase) in phases.iter().enumerate() {
        let Powl::PartialOrder {
            children: leaves,
            order: phase_order,
        } = phase
        else {
            return Err(CngRefusal::UnsupportedConstruct(format!(
                "hierarchical linearization requires every top-level child to be a \
                 phase PartialOrder; child {phase_idx} is a Leaf"
            )));
        };
        let start = labels.len();
        for (leaf_idx, leaf) in leaves.iter().enumerate() {
            match leaf {
                Powl::Leaf(Some(label)) => labels.push(label.as_str()),
                Powl::Leaf(None) => {
                    return Err(CngRefusal::UnsupportedConstruct(format!(
                        "hierarchical linearization requires named activity leaves; \
                         phase {phase_idx} leaf {leaf_idx} is a silent leaf"
                    )));
                }
                Powl::PartialOrder { .. } => {
                    return Err(CngRefusal::UnsupportedConstruct(format!(
                        "hierarchical linearization covers the two-level shape only; \
                         phase {phase_idx} child {leaf_idx} is a nested PartialOrder"
                    )));
                }
                Powl::ExternalCut { .. } => {
                    return Err(CngRefusal::UnsupportedConstruct(format!(
                        "hierarchical linearization covers the two-level shape only; \
                         phase {phase_idx} child {leaf_idx} is an external cut"
                    )));
                }
            }
        }
        // Phase order must be the full closure of the leaf total order.
        let n = leaves.len();
        let full_phase = n * n.saturating_sub(1) / 2;
        if phase_order.len() != full_phase
            || !(0..n).all(|i| ((i + 1)..n).all(|j| phase_order.contains(&(i, j))))
        {
            return Err(CngRefusal::UnsupportedConstruct(format!(
                "hierarchical linearization requires phase {phase_idx}'s order to be \
                 the full transitive closure of its leaf total order ({full_phase} \
                 pairs over {n} leaves); found {} pairs",
                phase_order.len()
            )));
        }
        // Intra-phase pairs at flat offsets. O(n²) per phase.
        for i in 0..n {
            for j in (i + 1)..n {
                edges.push((start + i, start + j));
            }
        }
        phase_ranges.push((start, n));
    }

    // bcinr-powl 26.6.25's PowlTape is a fixed 64-slot bitmask tape.
    if labels.len() > 64 {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "bcinr-powl 26.6.25 PowlTape holds at most 64 ops; hierarchical POWL v2 \
             model flattens to {} activities",
            labels.len()
        )));
    }

    // Cross-phase pairs: every op of an earlier phase precedes every op of
    // every later phase (the closure of the concatenated total order). O(n²).
    for a in 0..phase_ranges.len() {
        let (a_start, a_len) = phase_ranges[a];
        for &(b_start, b_len) in &phase_ranges[(a + 1)..] {
            for i in a_start..(a_start + a_len) {
                for j in b_start..(b_start + b_len) {
                    edges.push((i, j));
                }
            }
        }
    }
    edges.sort_unstable();

    Ok((labels, edges))
}

/// Runs a hierarchical POWL model on the bcinr-powl runtime by lawful
/// flattening: [`linearize_hierarchical`] produces the flat labels/edges,
/// then the exact same compile/schedule/conformance path as
/// [`validate_run`] executes them (shared via `run_labels_and_edges`).
///
/// # Errors
/// `CNG_R04 PlanUnsolvable` for an empty tape; the refusals of
/// [`linearize_hierarchical`] and the shared runner path otherwise.
///
/// # Complexity
/// O(n²) in flattened activity count n (linearization plus the bounded
/// scheduler loop).
pub fn validate_run_hierarchical(
    tape: &Pddl8Tape,
    model: &Powl,
) -> Result<RunnerReport, CngRefusal> {
    if tape.ops.is_empty() {
        return Err(CngRefusal::PlanUnsolvable(
            "empty PDDL plan tape: nothing to execute on the bcinr-powl runtime".to_string(),
        ));
    }
    let (labels, edges) = linearize_hierarchical(model)?;
    run_labels_and_edges(tape, labels, edges)
}

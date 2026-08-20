//! Family F09 -- "MFW Growth Operator" (atlas ticket V12-009).
//!
//! Survey verdict: **MIXED**. This module is a Wire-phase-1 pass, not the
//! full autonomic pipeline the atlas describes. Per
//! `.claude/rules/no-overclaiming.md`, this doc comment states plainly
//! which parts are real (verified this session by reading the dependency
//! source and by running the tests in this file) and which parts are
//! honest not-yet-implemented stubs -- no part is dressed up to look more
//! complete than it is.
//!
//! ## What is REAL (REUSE_ADAPT: genuinely calls existing, tested praxis code)
//!
//! - **Semantic Closure Check** ([`semantic_closure_check`]) delegates to
//!   [`praxis_graphlaw::chatman::closure::RecursiveSocketClosure::is_closed`]
//!   -- the real, tested Parent-Child Closure Law engine (PRD v26.7.11 §9,
//!   PROJ-759). This module does not reimplement closure semantics.
//! - **Reachability Gate + PDDL Planner** ([`reachability_gate`], folded
//!   into [`plan_growth`]) delegates to [`bcinr_pddl::solve_indexed`] --
//!   the real, differentially-verified-against-`bcinr_pddl` indexed PDDL8
//!   planner.
//! - **Descent Meter** ([`DescentMeter`]) is a real, tested bounded-descent
//!   budget check. Its check-then-refuse shape is adapted from
//!   `powl2_decompose::decompose`'s inlined `RefusalReason::BudgetExhausted`
//!   depth guard (`convert_rec`), promoted here to a standalone reusable
//!   type since F09 needs to carry a budget across a growth attempt rather
//!   than guard a single recursive call.
//! - **Descent Receipt** ([`DescentReceipt::seal`]) computes a real BLAKE3
//!   digest via [`wasm4pm_compat::hash::blake3_combined`] -- the same
//!   canonical field-tagged combination discipline
//!   `praxis_graphlaw::chatman::abi::InvocationEnvelope::envelope_hash`
//!   uses, not a bespoke hashing scheme.
//! - The L6 provenance-chain catalog and the `MFWGrowthRefused` catalog in
//!   `f09_mfw_growth_generated.rs` are real `ggen sync` output (not
//!   hand-typed) from `packs/f09-mfw-growth-pack/ontology.ttl`; see that
//!   file's own doc comment for the exact regenerate recipe. Generation was
//!   run twice this session and produced byte-identical output (checked via
//!   `diff`), and the pack was **not** registered in the shared root
//!   `ggen.toml` -- it was synced from an isolated scratch project so this
//!   change carries zero blast radius onto other families' packs mid-wave.
//!
//! ## What became REAL in the crash-recovery pass (fixed forward)
//!
//! - [`resolve_continuation_goal`] now really parses [`ResidueState`]'s
//!   admitted `domain_pddl`/`problem_pddl` text via the same
//!   `bcinr_pddl::parse` functions F08's `projector` module uses. It does
//!   not decide *what* the continuation goal is (that remains an upstream
//!   admission concern, not silently assumed here) -- it is the real,
//!   mechanical parse-and-validate step, refusing
//!   [`MFWGrowthRefused::ResidueMalformed`] on malformed text.
//! - [`manufacture_and_bind_child`] now really builds a child
//!   `powl2_decompose::Powl` from a plan tape (one `Leaf` per tape op, in a
//!   real total-order `PartialOrder`) and grafts it via [`graft_child`], a
//!   new tree-replace-at-path primitive this module adds because none
//!   existed anywhere in `powl2_decompose` (confirmed absent by grep;
//!   `Powl` had read-only `socket_at`, no mutator). Parent re-evaluation is
//!   real but disclosed-scoped: `RecursiveSocketClosure` only tracks
//!   children declared at `ParentChildClosure::from_model` time, so a
//!   freshly grafted child cannot be `admit`ted into the *caller's*
//!   pre-existing closure object -- this function instead re-derives a
//!   fresh `RecursiveSocketClosure` from the *post-graft* model and returns
//!   it alongside the new root, rather than silently pretending the old
//!   closure object updated itself.
//!
//! ## What became REAL in the F10 crown-edge pass (fixed forward)
//!
//! - [`manufacture_and_bind_child`] now really projects its `plan_tape`
//!   through F10's canonical POWL geometry pipeline
//!   ([`project_growth_plan_geometry`] -> `f10_powl_geometry::manufacture_powl_v2`)
//!   *before* grafting, and refuses the whole growth attempt (folded into
//!   [`MFWGrowthRefused::GraftRefused`], see that variant's reuse note) if
//!   F10 rejects the derived geometry. This is the real F09 -> F10
//!   production edge: F09's own confirmed real production entry point now
//!   has F10 as a genuine gate, not a parallel unused call -- see
//!   `tests::manufacture_and_bind_child_refuses_a_tape_with_self_referential_pred_mask`
//!   for the adversarial proof that this gate actually blocks a tape F09's
//!   own local logic alone would have grafted anyway.
//!
//! ## What remains an HONEST STUB (HAND_WRITE_REQUIRED, tracked under V12-009)
//!
//! - L7 (idempotent/duplicate-safe, restart-durable, chaos-tolerant
//!   re-admission with replay equivalence) is entirely unbuilt: there is no
//!   re-admission loop yet for it to guard.
//! - No production caller composes F08's `run_pipeline` output into this
//!   module's [`resolve_continuation_goal`]/[`plan_growth`] chain yet --
//!   that upstream (F08 -> F09) half of the crown path is tracked
//!   separately from this module's own (now real) F09 -> F10 half.
//! - Nothing yet calls [`manufacture_and_bind_child`] from outside this
//!   file's own test module: it is F09's real, tested production entry
//!   point, but no live orchestrator in this repo invokes it autonomously
//!   yet (disclosed, not silently implied).
//!
//! `MFW_AUTONOMIC_RESOLUTION_ALIVE` is **not** claimed by this module: no
//! single call composes admission through a manufactured, closure-admitted
//! child yet (each stage is real and tested in isolation -- see
//! `tests::plan_growth_succeeds_through_every_real_gate` for the closest
//! thing to a full cycle, which is real but test-driven, not wired to any
//! upstream/downstream production caller outside this module).
//! `BOUNDED_DESCENT_PROVEN` is claimed only for [`DescentMeter`] in
//! isolation (tested below), not for a full growth cycle under repeated
//! autonomic triggering.
//!
//! Survey-cited paths for F09 (informed research from the v26.7.12 family
//! survey handed to this wiring session inline):
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F09_mfw-growth.md
//! - /Users/sac/.claude/projects/-Users-sac-praxis/memory/autonomic-recursive-workflow.md
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/closure.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/closure_test.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/engine.rs
//! - /Users/sac/praxis/crates/pddl-index/src/lib.rs
//! - /Users/sac/praxis/crates/powl2-decompose/src/decompose.rs
//! - /Users/sac/praxis/crates/powl2-decompose/src/external_cut.rs
//! - /Users/sac/praxis/crates/powl2-decompose/src/recompose.rs
//! - /Users/sac/praxis/packs/f09-mfw-growth-pack/ (this wiring pass, new)

use std::collections::{BTreeMap, BTreeSet};

use bcinr_pddl::ground::lazy::GroundStats;
use powl2_decompose::{ParentChildClosure, Powl, WorkflowSocketId};
use praxis_graphlaw::chatman::closure::{ClosureLaw, RecursiveSocketClosure};
use wasm4pm_compat::hash::blake3_combined;
use wasm4pm_compat::pddl::{Pddl8Domain, Pddl8Problem, Pddl8Tape};

use crate::f10_powl_geometry;

include!("f09_mfw_growth_generated.rs");

/// F09's typed refusal taxonomy (atlas ticket V12-009). Every variant names
/// a concrete offender in its payload; no catch-all variant exists. See
/// `f09_mfw_growth_generated.rs::REFUSAL_CATALOG` for the ontology-sourced
/// description of each variant's meaning and which family invariant it
/// enforces (cross-checked against this enum by
/// `refusal_catalog_matches_enum_variants` below).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MFWGrowthRefused {
    /// Growth was attempted at a socket not reported blocked. Growth is
    /// autonomic (triggered only by a blocked socket + reachable goal), not
    /// pre-authored.
    #[error("socket {socket} is not blocked; MFW growth is only autonomic under a blocked socket")]
    SocketNotBlocked {
        /// The socket growth was attempted at.
        socket: String,
    },
    /// The socket's declared closure law already evaluates `Close(W) =
    /// true`. No child may be manufactured for already-closed truth.
    #[error(
        "semantic closure at socket {socket} already satisfies its declared {law} law; no \
         child may be manufactured for already-closed truth"
    )]
    ClosureAlreadySatisfied {
        /// The already-closed socket.
        socket: String,
        /// The declared closure law's name (see
        /// `praxis_graphlaw::chatman::closure::ClosureLaw::name`).
        law: &'static str,
    },
    /// The underlying closure-law evaluation itself refused (e.g. an
    /// unknown child); distinct from a definite closed/open verdict.
    #[error("closure check at socket {0} could not be evaluated")]
    ClosureCheckFailed(String),
    /// The indexed PDDL8 planner found no plan for the continuation goal;
    /// the goal remains unreachable, so growth is refused rather than
    /// manufacturing a child toward an impossible goal.
    #[error("continuation goal is not reachable: {reason}")]
    GoalUnreachable {
        /// The planner's own refusal reason.
        reason: String,
    },
    /// The descent meter has no budget remaining; one more descent step
    /// would be unbounded (`BOUNDED_DESCENT_PROVEN`).
    #[error("descent budget {budget} exhausted at depth {depth}; unbounded descent is refused")]
    DescentBudgetExhausted {
        /// The configured budget.
        budget: usize,
        /// The depth already reached.
        depth: usize,
    },
    /// A pipeline stage that is genuinely `HAND_WRITE_REQUIRED` and not yet
    /// built was reached. Fails loud rather than faking success -- see the
    /// module doc comment's "HONEST STUB" section.
    #[error(
        "F09 stage `{stage}` is HAND_WRITE_REQUIRED and not yet implemented (V12-009): {detail}"
    )]
    NotYetImplemented {
        /// The unbuilt stage's name.
        stage: &'static str,
        /// Why it is not built and what would be needed.
        detail: &'static str,
    },
    /// [`resolve_continuation_goal`]: the residue state's admitted PDDL
    /// domain/problem text failed to parse (`bcinr_pddl::parse`, the same
    /// real parser F08's `projector` module uses on admitted PDDL text).
    #[error("residue state PDDL text failed to parse: {reason}")]
    ResidueMalformed {
        /// The underlying `bcinr_pddl::Pddl8Error`, stringified.
        reason: String,
    },
    /// [`manufacture_and_bind_child`]: the plan tape produced zero ops, so
    /// there is no real child workflow to manufacture -- a plan that
    /// already satisfies the goal in zero steps grows nothing (distinct
    /// from a planning failure, which [`reachability_gate`] already
    /// refuses earlier as [`MFWGrowthRefused::GoalUnreachable`]).
    #[error("plan tape for socket {socket} has zero ops; nothing to manufacture")]
    EmptyPlanTape {
        /// The socket growth was attempted at.
        socket: String,
    },
    /// [`manufacture_and_bind_child`]: `parent_socket`'s path does not
    /// resolve inside the supplied root [`powl2_decompose::Powl`] model, or
    /// resolves to a node this module does not know how to graft a new
    /// child into (only [`powl2_decompose::Powl::PartialOrder`] targets are
    /// supported -- see [`graft_child`]).
    #[error("cannot graft a child at socket {socket}: {reason}")]
    GraftRefused {
        /// The socket growth was attempted at.
        socket: String,
        /// Why the graft was refused.
        reason: String,
    },
}

/// A resolved continuation goal: the concrete PDDL8 domain/problem a
/// blocked socket's growth must plan toward. REUSE_ADAPT: expressed
/// directly in `wasm4pm_compat`'s canonical PDDL8 types -- the same types
/// `bcinr_pddl` and `praxis_graphlaw::chatman::engine`'s S3 PDDL stage
/// already plan over. This module does not invent its own goal
/// representation.
#[derive(Debug, Clone)]
pub struct ContinuationGoal {
    /// The PDDL8 domain (action schemas) growth must plan over.
    pub domain: Pddl8Domain,
    /// The PDDL8 problem (objects, initial state, goal) to solve.
    pub problem: Pddl8Problem,
}

/// A blocked socket's residue state -- the L6 data-lens concept this
/// family's provenance chain names between `Socket` and `ContinuationGoal`.
///
/// `domain_pddl`/`problem_pddl` are real PDDL8 text, admitted by whatever
/// upstream mechanism produced this residue (this module does not invent
/// or infer them from `description`) -- the identical admission discipline
/// F08's `projector` module uses for its own admitted PDDL literals. This
/// module's contribution is the real, mechanical parse-and-validate step
/// in [`resolve_continuation_goal`], not a semantic derivation of *what*
/// the goal should be.
#[derive(Debug, Clone)]
pub struct ResidueState {
    /// The blocked socket this residue state describes.
    pub socket: WorkflowSocketId,
    /// Human-readable description of the residue (why the socket is
    /// blocked). Not machine-interpreted by anything in this module --
    /// audit/logging context only.
    pub description: String,
    /// Real PDDL8 domain text (action schemas available to resolve this
    /// residue), parsed via `bcinr_pddl::parse::domain_from_pddl`.
    pub domain_pddl: String,
    /// Real PDDL8 problem text (current state + the continuation goal),
    /// parsed via `bcinr_pddl::parse::problem_from_pddl`.
    pub problem_pddl: String,
}

/// Continuation Goal Resolver stage: parses `residue`'s admitted
/// `domain_pddl`/`problem_pddl` text into a [`ContinuationGoal`] via the
/// same real `bcinr_pddl::parse` functions F08's `projector` module uses on
/// admitted PDDL literals. What this module does *not* do -- and does not
/// pretend to do -- is decide *what* the continuation goal should be from
/// `description` or any other unstructured signal; that decision belongs
/// to whatever upstream mechanism populates `domain_pddl`/`problem_pddl`
/// on the [`ResidueState`] in the first place. A caller with only a
/// human-readable residue description and no admitted PDDL text has no
/// admissible input here, by design -- see the family invariant against
/// treating free-text prose as scenario authority (the same invariant
/// `crate::f26_ontology_self_play`'s Scenario Generator is written to
/// respect).
///
/// # Errors
/// [`MFWGrowthRefused::ResidueMalformed`] if either PDDL text fails to
/// parse.
pub fn resolve_continuation_goal(
    residue: &ResidueState,
) -> Result<ContinuationGoal, MFWGrowthRefused> {
    let domain = bcinr_pddl::parse::domain_from_pddl(&residue.domain_pddl).map_err(|e| {
        MFWGrowthRefused::ResidueMalformed {
            reason: format!("domain: {e}"),
        }
    })?;
    let problem = bcinr_pddl::parse::problem_from_pddl(&residue.problem_pddl).map_err(|e| {
        MFWGrowthRefused::ResidueMalformed {
            reason: format!("problem: {e}"),
        }
    })?;
    Ok(ContinuationGoal { domain, problem })
}

/// Bounded-descent budget for one MFW growth attempt (invariant: "No child
/// workflow is manufactured for ... unbounded descent"). REUSE_ADAPT: the
/// check-and-refuse shape is adapted from
/// `powl2_decompose::decompose::convert_rec`'s inlined
/// `RefusalReason::BudgetExhausted` depth guard, promoted here to a
/// standalone reusable type since F09 needs to carry the budget across the
/// growth pipeline (socket -> plan -> descent receipt), not just guard one
/// recursive call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DescentMeter {
    budget: usize,
    depth: usize,
}

impl DescentMeter {
    /// A fresh meter with `budget` descent steps available and zero depth
    /// reached.
    ///
    /// # Complexity
    /// O(1).
    #[must_use]
    pub fn new(budget: usize) -> Self {
        DescentMeter { budget, depth: 0 }
    }

    /// Checks budget remains before permitting one more descent step (one
    /// child-manufacture), then advances depth by one.
    ///
    /// # Errors
    /// [`MFWGrowthRefused::DescentBudgetExhausted`] if `depth` already
    /// equals `budget`.
    ///
    /// # Complexity
    /// O(1).
    pub fn descend(&mut self) -> Result<usize, MFWGrowthRefused> {
        if self.depth >= self.budget {
            return Err(MFWGrowthRefused::DescentBudgetExhausted {
                budget: self.budget,
                depth: self.depth,
            });
        }
        self.depth += 1;
        Ok(self.depth)
    }

    /// The configured budget.
    #[must_use]
    pub fn budget(&self) -> usize {
        self.budget
    }

    /// The depth reached so far.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Remaining descent steps before [`Self::descend`] would refuse.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.budget.saturating_sub(self.depth)
    }
}

/// A sealed receipt for one growth attempt's descent-budget state (L6 data
/// lens). REUSE_ADAPT: hashed via the same canonical, field-tagged BLAKE3
/// combination [`praxis_graphlaw::chatman::abi::InvocationEnvelope::envelope_hash`]
/// uses ([`wasm4pm_compat::hash::blake3_combined`]) -- not a bespoke
/// hashing scheme. This is a genuine, real digest (verified deterministic
/// by `descent_receipt_digest_is_deterministic` below), not a placeholder
/// string.
///
/// Full L6/L7 receiptedness (chaining into a receipt store, replay
/// verification against a historical log) is not built -- there is no
/// receipt store for this family yet. This type carries the sealed digest
/// only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescentReceipt {
    /// The parent socket this descent occurred under.
    pub parent_socket: WorkflowSocketId,
    /// The configured budget at seal time.
    pub budget: usize,
    /// The depth reached at seal time.
    pub depth_reached: usize,
    /// 64 lowercase hex chars: `blake3_combined` over the canonical,
    /// length-prefixed field tuple.
    pub digest: String,
}

impl DescentReceipt {
    /// Seals a receipt over `parent_socket` and `meter`'s current state.
    ///
    /// # Determinism
    /// Same `parent_socket`/`budget`/`depth` always produce the same
    /// digest: no wall clock, no randomness, field order fixed.
    ///
    /// # Complexity
    /// O(1) plus BLAKE3 over a small, fixed-shape byte string.
    #[must_use]
    pub fn seal(parent_socket: WorkflowSocketId, meter: &DescentMeter) -> Self {
        let digest = blake3_combined(&[
            "mfw:descent-receipt:v1",
            &parent_socket.to_string(),
            &meter.budget().to_string(),
            &meter.depth().to_string(),
        ]);
        DescentReceipt {
            parent_socket,
            budget: meter.budget(),
            depth_reached: meter.depth(),
            digest,
        }
    }
}

/// Semantic Closure Check (real; REUSE_ADAPT): refuses if `closure`'s
/// declared law already evaluates `Close(W) = true`, or if the closure
/// engine itself cannot evaluate the law.
///
/// # Errors
/// - [`MFWGrowthRefused::ClosureCheckFailed`] if
///   [`RecursiveSocketClosure::is_closed`] itself refuses.
/// - [`MFWGrowthRefused::ClosureAlreadySatisfied`] if it evaluates `true`.
///
/// # Complexity
/// O(c) worst case, c = declared direct children of the socket (delegates
/// to `RecursiveSocketClosure::is_closed`).
pub fn semantic_closure_check(closure: &RecursiveSocketClosure) -> Result<(), MFWGrowthRefused> {
    let is_closed = closure
        .is_closed()
        .map_err(|e| MFWGrowthRefused::ClosureCheckFailed(e.to_string()))?;
    if is_closed {
        return Err(MFWGrowthRefused::ClosureAlreadySatisfied {
            socket: closure.socket().to_string(),
            law: closure.law().name(),
        });
    }
    Ok(())
}

/// Real evidence that a continuation goal is reachable: the plan tape the
/// indexed PDDL8 planner found, plus its grounding statistics.
struct ReachabilityWitness {
    plan: Pddl8Tape,
    stats: GroundStats,
}

/// Reachability Gate + PDDL Planner (real; REUSE_ADAPT): runs
/// [`bcinr_pddl::solve_indexed`] over `goal`.
///
/// # Errors
/// [`MFWGrowthRefused::GoalUnreachable`] if no plan is found.
///
/// # Complexity
/// Bounded by `bcinr_pddl`'s indexed grounder (auto-selects over
/// `bcinr_pddl::GROUND_INDEX_THRESHOLD`) plus BFS plan search bounded by
/// `PDDL8_MAX_PLAN_DEPTH`.
fn reachability_gate(goal: &ContinuationGoal) -> Result<ReachabilityWitness, MFWGrowthRefused> {
    let gp =
        bcinr_pddl::ground::lazy::IndexedGroundProblem::build(&goal.domain, &goal.problem, None)
            .map_err(|e| MFWGrowthRefused::GoalUnreachable {
                reason: format!("pddl index failed: {e}"),
            })?;
    let stats = gp.stats();
    let plan = gp
        .find_plan()
        .into_result()
        .map_err(|e| MFWGrowthRefused::GoalUnreachable {
            reason: e.to_string(),
        })?;
    Ok(ReachabilityWitness { plan, stats })
}

/// The real, computed evidence produced once F09's REUSE_ADAPT gates
/// (Semantic Closure Check, Reachability Gate, PDDL Planner, Descent Meter)
/// have all passed for one growth attempt at `parent_socket`. This is a
/// genuine intermediate result -- no child has been manufactured or bound
/// yet; see [`manufacture_and_bind_child`].
#[derive(Debug, Clone)]
pub struct GrowthPlan {
    /// The socket this growth attempt is anchored to.
    pub parent_socket: WorkflowSocketId,
    /// The real plan tape [`bcinr_pddl::solve_indexed`] found.
    pub plan_tape: Pddl8Tape,
    /// Grounding statistics from the same planner call.
    pub ground_stats: GroundStats,
    /// The sealed descent-budget receipt for this attempt.
    pub descent_receipt: DescentReceipt,
}

/// Runs F09's real autonomic trigger + planning gates for one candidate
/// growth at `closure`'s socket: socket-blocked check, Semantic Closure
/// Check, Reachability Gate, PDDL Planner, Descent Meter (in that order,
/// short-circuiting on the first refusal).
///
/// `socket_blocked` is the caller's own determination that the socket is
/// currently blocked; this module has no independent way to detect that
/// (no existing praxis code surfaces "is this socket blocked" as a
/// queryable fact), so it is an explicit, honestly-named input rather than
/// something silently assumed `true`.
///
/// # Errors
/// See [`MFWGrowthRefused`]'s variants.
///
/// # Complexity
/// O(c) for the closure check (c = declared children of the socket) plus
/// the indexed grounder's cost for the reachability/planning stage; O(1)
/// for the descent-meter check and receipt seal.
pub fn plan_growth(
    socket_blocked: bool,
    closure: &RecursiveSocketClosure,
    goal: &ContinuationGoal,
    meter: &mut DescentMeter,
) -> Result<GrowthPlan, MFWGrowthRefused> {
    if !socket_blocked {
        return Err(MFWGrowthRefused::SocketNotBlocked {
            socket: closure.socket().to_string(),
        });
    }
    semantic_closure_check(closure)?;
    let witness = reachability_gate(goal)?;
    meter.descend()?;
    let descent_receipt = DescentReceipt::seal(closure.socket().clone(), meter);
    Ok(GrowthPlan {
        parent_socket: closure.socket().clone(),
        plan_tape: witness.plan,
        ground_stats: witness.stats,
        descent_receipt,
    })
}

/// POWL Manufacturer (real): projects a plan tape into a `Powl` child model
/// -- a total-order [`Powl::PartialOrder`] with one `Leaf(Some(op.label))`
/// per tape op, ordered `(i, i+1)` for every consecutive pair, since a plan
/// tape is inherently sequential (each op's `pred_mask` already encodes
/// that it depends on prior ops -- see `Pddl8TapeOp`). This is the one
/// well-defined, unambiguous POWL shape a linear plan projects to; a
/// planner that ever produces genuinely concurrent (non-sequential) tapes
/// would need a different projection, not built here (disclosed scope
/// boundary, matching the family survey's own finding that no such
/// projector exists anywhere in this repo).
///
/// # Errors
/// [`MFWGrowthRefused::EmptyPlanTape`] if `tape` has zero ops.
fn manufacture_child_powl(
    socket: &WorkflowSocketId,
    tape: &Pddl8Tape,
) -> Result<Powl, MFWGrowthRefused> {
    if tape.ops.is_empty() {
        return Err(MFWGrowthRefused::EmptyPlanTape {
            socket: socket.to_string(),
        });
    }
    let children: Vec<Powl> = tape
        .ops
        .iter()
        .map(|op| Powl::Leaf(Some(op.label.clone())))
        .collect();
    let order: BTreeSet<(usize, usize)> = (0..children.len().saturating_sub(1))
        .map(|i| (i, i + 1))
        .collect();
    Ok(Powl::PartialOrder { children, order })
}

/// Socket Binder (real): grafts `child` as a new, order-unconstrained
/// sibling under the [`Powl::PartialOrder`] node at `at.path` inside
/// `root`, returning the new root. `powl2_decompose::Powl` has no mutator
/// of its own (confirmed by the F09 survey and re-checked this pass: only
/// read-only `socket_at`/`sockets` exist) -- this is a genuinely new
/// tree-replace-at-path primitive, not adapted from anywhere.
///
/// The new child is added with no order constraint relative to existing
/// siblings (immediately available, not sequenced after them): growth
/// manufactures an *additional* way for the blocked socket to close, not a
/// continuation of whatever already-declared children exist there.
///
/// # Errors
/// [`MFWGrowthRefused::GraftRefused`] if `at.path` does not resolve inside
/// `root`, or resolves to a node that is not a [`Powl::PartialOrder`]
/// (`Leaf`/`Choice`/`ExternalCut` grafting is out of scope this pass,
/// disclosed rather than silently mishandled).
///
/// # Complexity
/// O(depth) to locate the node, O(n) to clone-and-rebuild the path back to
/// the root (immutable-tree update, consistent with `Powl: Clone`).
fn graft_child(root: &Powl, at: &WorkflowSocketId, child: Powl) -> Result<Powl, MFWGrowthRefused> {
    fn go(node: &Powl, remaining: &[usize], child: Powl) -> Result<Powl, String> {
        match remaining {
            [] => match node {
                Powl::PartialOrder { children, order } => {
                    let mut new_children = children.clone();
                    new_children.push(child);
                    Ok(Powl::PartialOrder {
                        children: new_children,
                        order: order.clone(),
                    })
                }
                Powl::Leaf(_) => Err("target is a Leaf, not a PartialOrder".to_string()),
                Powl::Choice { .. } => Err("target is a Choice, not a PartialOrder".to_string()),
                Powl::ExternalCut { .. } => {
                    Err("target is an ExternalCut, not a PartialOrder".to_string())
                }
            },
            [seg, rest @ ..] => match node {
                Powl::PartialOrder { children, order } => {
                    let idx = *seg;
                    let existing = children
                        .get(idx)
                        .ok_or_else(|| format!("child index {idx} out of range"))?;
                    let updated = go(existing, rest, child)?;
                    let mut new_children = children.clone();
                    new_children[idx] = updated;
                    Ok(Powl::PartialOrder {
                        children: new_children,
                        order: order.clone(),
                    })
                }
                Powl::Choice { children, graph } => {
                    let idx = *seg;
                    let existing = children
                        .get(idx)
                        .ok_or_else(|| format!("child index {idx} out of range"))?;
                    let updated = go(existing, rest, child)?;
                    let mut new_children = children.clone();
                    new_children[idx] = updated;
                    Ok(Powl::Choice {
                        children: new_children,
                        graph: graph.clone(),
                    })
                }
                Powl::ExternalCut {
                    region,
                    projection,
                    renderer,
                } => {
                    if *seg != 0 {
                        return Err(format!("ExternalCut has no child index {seg}"));
                    }
                    let updated = go(region, rest, child)?;
                    Ok(Powl::ExternalCut {
                        region: Box::new(updated),
                        projection: projection.clone(),
                        renderer: renderer.clone(),
                    })
                }
                Powl::Leaf(_) => Err("path descends past a Leaf".to_string()),
            },
        }
    }
    go(root, at.path.segments(), child).map_err(|reason| MFWGrowthRefused::GraftRefused {
        socket: at.to_string(),
        reason,
    })
}

/// The real, post-graft evidence [`manufacture_and_bind_child`] returns:
/// the new root model with the manufactured child attached, a freshly
/// re-derived closure over `plan.parent_socket` (see this function's
/// disclosed "Parent Re-evaluator" scope note), and the canonical F10 POWL
/// v2 geometry [`project_growth_plan_geometry`] independently derived from
/// the same plan tape (the real F09 -> F10 production edge -- see that
/// function's doc comment for what this is and is not).
#[derive(Debug, Clone)]
pub struct GrowthOutcome {
    /// The root `Powl` model after grafting the manufactured child.
    pub new_root: Powl,
    /// `RecursiveSocketClosure` re-declared over `new_root` at
    /// `plan.parent_socket`, reflecting the freshly grafted child.
    pub closure: RecursiveSocketClosure,
    /// The manufactured child's own socket id (a new leaf/child under
    /// `plan.parent_socket`), for a caller that wants to `admit` it once
    /// its own execution completes.
    pub child_socket: WorkflowSocketId,
    /// `crate::f10_powl_geometry`'s canonical [`f10_powl_geometry::POWLModel`]
    /// for the same plan tape, built by F10's real, independently-tested
    /// Plan Grouper -> Partial Order Builder -> Hierarchy Builder pipeline
    /// (`f10_powl_geometry::build_powl_geometry`), not by this module's own
    /// ad hoc [`manufacture_child_powl`]. Two independent constructions of
    /// "a Powl geometry for this tape" exist on this struct by design: F09's
    /// own (already grafted into `new_root`) and F10's canonical one (kept
    /// separate, not substituted in-place, since F10's Plan Grouper may
    /// legitimately shape the tree differently -- e.g. deeper phase nesting
    /// -- and swapping it in would change `new_root`'s shape out from under
    /// this function's own already-tested grafting contract).
    pub geometry: f10_powl_geometry::POWLModel,
    /// `geometry`'s canonical Turtle serialization
    /// ([`f10_powl_geometry::to_turtle`]).
    pub geometry_turtle: String,
    /// `geometry`'s structural shape report
    /// ([`f10_powl_geometry::validate_shape`]).
    pub geometry_shape: f10_powl_geometry::ShapeReport,
}

/// Base IRI [`project_growth_plan_geometry`] serializes F09's derived F10
/// geometry Turtle under. Fixed rather than a parameter of
/// [`manufacture_and_bind_child`]: this Turtle is provenance evidence, not
/// a publishable artifact with a caller-chosen namespace yet (no receipt
/// store exists for F09 to publish it into -- see the module doc's "HONEST
/// STUB" section on L6/L7). Revisit if/when that changes.
const F09_GROWTH_GEOMETRY_BASE_IRI: &str = "https://truex.io/ontology/mfw/f09-growth";

/// Converts a real [`GrowthPlan`]'s `plan_tape` into an F10
/// [`f10_powl_geometry::Plan`] -- the actual F09 -> F10 data bridge.
///
/// One [`f10_powl_geometry::PlanAction`] per tape op (`id` = `op.label`,
/// `source` = `socket.to_string()`, the growth socket that justifies every
/// op's inclusion -- never empty, since `WorkflowSocketId::to_string()`
/// always renders a non-empty `socket:...` address). `precedes` is decoded
/// directly from each op's `pred_mask` bitmask (`Pddl8TapeOp`'s own
/// documented meaning: "bitmask of ops that must complete before this one
/// is eligible") -- one `(predecessor_index, index)` pair per set bit, not
/// an assumed `(i-1, i)` chain, so a tape whose predecessor bits encode
/// something other than a simple chain is still decoded faithfully rather
/// than approximated. No choice groups: a PDDL8 plan tape is a linear
/// sequence, never a branch.
///
/// This function cannot itself fail -- any inconsistency in the decoded
/// `precedes` relation (an out-of-range or self-referential bit, a
/// contradiction) is caught downstream by
/// [`f10_powl_geometry::build_powl_geometry`]'s own typed refusals, not
/// pre-validated here (no duplicated validation logic).
///
/// # Complexity
/// O(ops * 64) worst case (64 = `u64::BITS`, decoding every `pred_mask`).
fn pddl_tape_to_f10_plan(socket: &WorkflowSocketId, tape: &Pddl8Tape) -> f10_powl_geometry::Plan {
    let actions = tape
        .ops
        .iter()
        .map(|op| f10_powl_geometry::PlanAction {
            id: op.label.clone(),
            source: socket.to_string(),
        })
        .collect();
    let mut precedes: BTreeSet<(usize, usize)> = BTreeSet::new();
    for op in &tape.ops {
        let i = op.index as usize;
        for bit in 0..u64::BITS {
            if op.pred_mask & (1u64 << bit) != 0 {
                precedes.insert((bit as usize, i));
            }
        }
    }
    f10_powl_geometry::Plan {
        actions,
        precedes,
        choice_groups: Vec::new(),
    }
}

/// The real F09 -> F10 production edge: projects `plan`'s real, planner-
/// produced `plan_tape` through F10's actual, independently-tested POWL
/// geometry pipeline ([`f10_powl_geometry::manufacture_powl_v2`]) -- Plan
/// Grouper -> Partial Order Builder -> Hierarchy Builder -> Serializer ->
/// Shape Validator -- and returns its real output. This is not a parallel,
/// unused call: [`manufacture_and_bind_child`] (F09's own confirmed real
/// production entry point) calls this before grafting and refuses the
/// entire growth attempt if F10 rejects the derived geometry, so F10 is a
/// genuine gate in F09's real pipeline, not decoration.
///
/// # Errors
/// [`MFWGrowthRefused::GraftRefused`], wrapping whatever
/// [`f10_powl_geometry::POWLGeometryRefused`] F10's pipeline raised (F10's
/// own `Display` renders the exact rejected invariant, e.g.
/// `F10_POWL_GEOMETRY_REFUSED[ORDER_DERIVED] InventedOrder: ...`). Reuses
/// `GraftRefused` rather than a new `MFWGrowthRefused` variant: this
/// module's refusal catalog is `ggen`-sourced from
/// `packs/f09-mfw-growth-pack/ontology.ttl` (see the module doc comment),
/// and adding a new ontology-backed variant for this one wiring pass was
/// judged out of scope; `GraftRefused`'s existing meaning ("cannot graft a
/// child at socket {socket}: {reason}") already covers "this plan tape's
/// geometry is rejected, so no child will be grafted for it" -- disclosed
/// here rather than silently reused without comment.
///
/// # Complexity
/// Dominated by [`f10_powl_geometry::build_powl_geometry`]'s O(n^3)
/// (n = `plan.plan_tape.ops.len()`).
fn project_growth_plan_geometry(
    plan: &GrowthPlan,
) -> Result<
    (
        f10_powl_geometry::POWLModel,
        String,
        f10_powl_geometry::ShapeReport,
    ),
    MFWGrowthRefused,
> {
    let f10_plan = pddl_tape_to_f10_plan(&plan.parent_socket, &plan.plan_tape);
    f10_powl_geometry::manufacture_powl_v2(
        &f10_plan,
        &BTreeMap::new(),
        F09_GROWTH_GEOMETRY_BASE_IRI,
    )
    .map_err(|e| MFWGrowthRefused::GraftRefused {
        socket: plan.parent_socket.to_string(),
        reason: format!(
            "F10 POWL geometry pipeline rejected this plan tape's derived process \
                 geometry: {e}"
        ),
    })
}

/// POWL Manufacturer + Socket Binder + Parent Re-evaluator -- the pipeline
/// stages after [`plan_growth`], real as of the crash-recovery pass. See
/// [`manufacture_child_powl`] and [`graft_child`] for what each stage does
/// and its disclosed scope boundary; see the module doc comment's "What
/// became REAL" section for why parent re-evaluation returns a *freshly
/// derived* closure rather than mutating `plan_growth`'s caller-supplied
/// one.
///
/// `law` is the closure law to re-declare the parent socket's closure
/// under after grafting -- the caller's own choice (mirroring
/// [`RecursiveSocketClosure::declare`]'s own signature), not silently
/// defaulted, since `plan_growth`'s original `&RecursiveSocketClosure`
/// only exposes a read accessor for its law
/// ([`RecursiveSocketClosure::law`]), not ownership of it.
///
/// # Errors
/// [`MFWGrowthRefused::EmptyPlanTape`]; [`MFWGrowthRefused::GraftRefused`]
/// from three distinct sources folded into one variant (see that variant's
/// reuse note on [`project_growth_plan_geometry`]): F10's POWL geometry
/// pipeline rejecting the derived plan geometry (checked first, before any
/// graft), the graft itself failing to resolve `plan.parent_socket`, or a
/// propagated [`RecursiveSocketClosure::declare`] failure (re-declaring a
/// closure over the grafted socket failed -- e.g. it still has zero
/// children, which cannot happen given a non-empty manufactured child was
/// just grafted, but is handled rather than unwrapped per this repo's
/// no-panic invariant).
pub fn manufacture_and_bind_child(
    root: &Powl,
    plan: &GrowthPlan,
    law: ClosureLaw,
) -> Result<GrowthOutcome, MFWGrowthRefused> {
    let child = manufacture_child_powl(&plan.parent_socket, &plan.plan_tape)?;
    // Real F09 -> F10 production edge: the same plan tape that justified
    // `child` above is independently projected through F10's canonical POWL
    // geometry pipeline and gates this whole growth attempt -- a tape F10
    // rejects is refused here, before any graft happens, not silently
    // grafted anyway. See `project_growth_plan_geometry`'s doc comment.
    let (geometry, geometry_turtle, geometry_shape) = project_growth_plan_geometry(plan)?;
    let new_root = graft_child(root, &plan.parent_socket, child)?;

    // The manufactured child is the new last child of parent_socket's
    // PartialOrder node -- socket_at + sockets() on new_root confirms its
    // exact address rather than assuming it.
    let target = new_root
        .socket_at(&plan.parent_socket.path)
        .ok_or_else(|| MFWGrowthRefused::GraftRefused {
            socket: plan.parent_socket.to_string(),
            reason: "grafted node not found by its own path immediately after graft \
                     (internal invariant violation)"
                .to_string(),
        })?;
    let new_child_index = match target {
        Powl::PartialOrder { children, .. } => children.len() - 1,
        _ => {
            return Err(MFWGrowthRefused::GraftRefused {
                socket: plan.parent_socket.to_string(),
                reason: "grafted node is not a PartialOrder immediately after graft \
                         (internal invariant violation)"
                    .to_string(),
            });
        }
    };
    // manufacture_child_powl always returns Powl::PartialOrder on success
    // (see that function) -- the manufactured child's own socket kind is
    // therefore always PartialOrder, not inspected via the (private)
    // SocketKind::of.
    let child_socket = WorkflowSocketId {
        path: plan.parent_socket.path.child(new_child_index),
        kind: powl2_decompose::SocketKind::PartialOrder,
    };

    let pcc = ParentChildClosure::from_model(&new_root);
    let closure =
        RecursiveSocketClosure::declare(&pcc, plan.parent_socket.clone(), law).map_err(|e| {
            MFWGrowthRefused::GraftRefused {
                socket: plan.parent_socket.to_string(),
                reason: format!("re-declaring closure over the grafted socket failed: {e}"),
            }
        })?;

    Ok(GrowthOutcome {
        new_root,
        closure,
        child_socket,
        geometry,
        geometry_turtle,
        geometry_shape,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use powl2_decompose::{ParentChildClosure, Powl, SocketKind, SocketPath};
    use praxis_graphlaw::chatman::closure::ClosureLaw;
    use wasm4pm_compat::pddl::{Pddl8ActionSchema, Pddl8Atom, Pddl8GroundAction, Pddl8TapeOp};

    use super::*;

    // -----------------------------------------------------------------
    // Fixture helpers (closure): same pattern as
    // praxis_graphlaw::chatman::closure_test, kept minimal for this
    // module's own tests.
    // -----------------------------------------------------------------

    fn root_partial_order_over(n: usize) -> Powl {
        let children = (0..n)
            .map(|i| Powl::Leaf(Some(format!("leaf-{i}"))))
            .collect();
        Powl::PartialOrder {
            children,
            order: BTreeSet::new(),
        }
    }

    fn root_socket() -> WorkflowSocketId {
        WorkflowSocketId {
            path: SocketPath::root(),
            kind: SocketKind::PartialOrder,
        }
    }

    fn leaf_socket(i: usize) -> WorkflowSocketId {
        WorkflowSocketId {
            path: SocketPath::root().child(i),
            kind: SocketKind::Leaf,
        }
    }

    fn open_closure() -> RecursiveSocketClosure {
        let model = root_partial_order_over(2);
        let pcc = ParentChildClosure::from_model(&model);
        RecursiveSocketClosure::declare(&pcc, root_socket(), ClosureLaw::AllRequired)
            .expect("declare over 2 leaves")
    }

    fn already_closed_closure() -> RecursiveSocketClosure {
        let mut rsc = open_closure();
        rsc.admit(&leaf_socket(0)).expect("admit leaf 0");
        rsc.admit(&leaf_socket(1)).expect("admit leaf 1");
        assert!(
            rsc.is_closed().expect("evaluable"),
            "fixture must be closed"
        );
        rsc
    }

    // -----------------------------------------------------------------
    // Fixture helpers (PDDL): same shape as pddl-index's own
    // tests/grounder.rs transport fixture, reduced to a 2-hop path so
    // solve_indexed runs in microseconds.
    // -----------------------------------------------------------------

    fn atom(pred: &str, args: &[&str]) -> Pddl8Atom {
        Pddl8Atom {
            pred: pred.into(),
            args: args.iter().map(|s| (*s).into()).collect(),
        }
    }

    fn move_schema() -> Pddl8ActionSchema {
        Pddl8ActionSchema {
            name: "move".into(),
            params: vec!["?from".into(), "?to".into()],
            preconditions: vec![atom("at", &["?from"]), atom("link", &["?from", "?to"])],
            add_effects: vec![atom("at", &["?to"])],
            del_effects: vec![atom("at", &["?from"])],
            typed_params: Vec::new(),
            condition: None,
            effects: Vec::new(),
            numeric_effects: Vec::new(),
        }
    }

    fn empty_domain(actions: Vec<Pddl8ActionSchema>) -> Pddl8Domain {
        Pddl8Domain {
            name: "mfw-f09-transport".into(),
            predicates: Vec::new(),
            actions,
            types: Vec::new(),
            functions: Vec::new(),
            durative_actions: Vec::new(),
            derived: Vec::new(),
            constraints: Vec::new(),
            processes: Vec::new(),
            events: Vec::new(),
        }
    }

    fn problem(objects: &[&str], init: Vec<Pddl8Atom>, goal: Vec<Pddl8Atom>) -> Pddl8Problem {
        Pddl8Problem {
            name: "mfw-f09-transport".into(),
            domain: "mfw-f09-transport".into(),
            objects: objects.iter().map(|s| (*s).into()).collect(),
            init,
            goal,
            object_types: Vec::new(),
            fn_values: Vec::new(),
            timed_inits: Vec::new(),
            preferences: Vec::new(),
            metric: None,
        }
    }

    /// A trivially solvable one-hop goal: `l0 --link--> l1`, `at(l0)` ->
    /// goal `at(l1)`.
    fn reachable_goal() -> ContinuationGoal {
        let domain = empty_domain(vec![move_schema()]);
        let problem = problem(
            &["l0", "l1"],
            vec![atom("at", &["l0"]), atom("link", &["l0", "l1"])],
            vec![atom("at", &["l1"])],
        );
        ContinuationGoal { domain, problem }
    }

    /// A genuinely unreachable goal: no `link` fact exists at all, so
    /// `move` can never fire and the goal `at(l1)` is unreachable.
    fn unreachable_goal() -> ContinuationGoal {
        let domain = empty_domain(vec![move_schema()]);
        let problem = problem(
            &["l0", "l1"],
            vec![atom("at", &["l0"])],
            vec![atom("at", &["l1"])],
        );
        ContinuationGoal { domain, problem }
    }

    // -----------------------------------------------------------------
    // DescentMeter
    // -----------------------------------------------------------------

    #[test]
    fn descent_meter_refuses_past_budget() {
        let mut meter = DescentMeter::new(2);
        assert_eq!(meter.remaining(), 2);
        assert_eq!(meter.descend(), Ok(1));
        assert_eq!(meter.descend(), Ok(2));
        assert_eq!(meter.remaining(), 0);
        assert_eq!(
            meter.descend(),
            Err(MFWGrowthRefused::DescentBudgetExhausted {
                budget: 2,
                depth: 2
            })
        );
    }

    #[test]
    fn descent_meter_zero_budget_refuses_immediately() {
        let mut meter = DescentMeter::new(0);
        assert_eq!(
            meter.descend(),
            Err(MFWGrowthRefused::DescentBudgetExhausted {
                budget: 0,
                depth: 0
            })
        );
    }

    #[test]
    fn descent_receipt_digest_is_deterministic() {
        let mut m1 = DescentMeter::new(3);
        m1.descend().expect("descend");
        let mut m2 = DescentMeter::new(3);
        m2.descend().expect("descend");
        let r1 = DescentReceipt::seal(root_socket(), &m1);
        let r2 = DescentReceipt::seal(root_socket(), &m2);
        assert_eq!(
            r1.digest, r2.digest,
            "same inputs must yield the same digest"
        );
        assert_eq!(
            r1.digest.len(),
            64,
            "blake3 hex digest is 64 lowercase hex chars"
        );
        assert!(r1.digest.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn descent_receipt_digest_changes_with_depth() {
        let mut meter = DescentMeter::new(3);
        let r0 = DescentReceipt::seal(root_socket(), &meter);
        meter.descend().expect("descend");
        let r1 = DescentReceipt::seal(root_socket(), &meter);
        assert_ne!(r0.digest, r1.digest, "digest must depend on depth");
    }

    // -----------------------------------------------------------------
    // semantic_closure_check
    // -----------------------------------------------------------------

    #[test]
    fn semantic_closure_check_passes_on_open_socket() {
        let closure = open_closure();
        assert_eq!(semantic_closure_check(&closure), Ok(()));
    }

    #[test]
    fn semantic_closure_check_refuses_already_closed_truth() {
        let closure = already_closed_closure();
        assert_eq!(
            semantic_closure_check(&closure),
            Err(MFWGrowthRefused::ClosureAlreadySatisfied {
                socket: root_socket().to_string(),
                law: "all_required",
            })
        );
    }

    // -----------------------------------------------------------------
    // plan_growth end-to-end
    // -----------------------------------------------------------------

    #[test]
    fn plan_growth_refuses_unblocked_socket() {
        let closure = open_closure();
        let goal = reachable_goal();
        let mut meter = DescentMeter::new(4);
        assert_eq!(
            plan_growth(false, &closure, &goal, &mut meter).unwrap_err(),
            MFWGrowthRefused::SocketNotBlocked {
                socket: root_socket().to_string(),
            }
        );
    }

    #[test]
    fn plan_growth_refuses_already_closed_truth() {
        let closure = already_closed_closure();
        let goal = reachable_goal();
        let mut meter = DescentMeter::new(4);
        assert_eq!(
            plan_growth(true, &closure, &goal, &mut meter).unwrap_err(),
            MFWGrowthRefused::ClosureAlreadySatisfied {
                socket: root_socket().to_string(),
                law: "all_required",
            }
        );
    }

    #[test]
    fn plan_growth_refuses_unreachable_goal() {
        let closure = open_closure();
        let goal = unreachable_goal();
        let mut meter = DescentMeter::new(4);
        let result = plan_growth(true, &closure, &goal, &mut meter);
        assert!(
            matches!(result, Err(MFWGrowthRefused::GoalUnreachable { .. })),
            "expected GoalUnreachable, got {result:?}"
        );
        assert_eq!(
            meter.depth(),
            0,
            "descent must not advance on an unreachable goal"
        );
    }

    #[test]
    fn plan_growth_refuses_exhausted_descent_budget() {
        let closure = open_closure();
        let goal = reachable_goal();
        let mut meter = DescentMeter::new(0);
        assert_eq!(
            plan_growth(true, &closure, &goal, &mut meter).unwrap_err(),
            MFWGrowthRefused::DescentBudgetExhausted {
                budget: 0,
                depth: 0
            }
        );
    }

    #[test]
    fn plan_growth_succeeds_through_every_real_gate() {
        let closure = open_closure();
        let goal = reachable_goal();
        let mut meter = DescentMeter::new(4);
        let plan = plan_growth(true, &closure, &goal, &mut meter).expect("all real gates pass");
        assert_eq!(plan.parent_socket, root_socket());
        assert_eq!(plan.plan_tape.ops.len(), 1, "one-hop plan is a single move");
        assert_eq!(plan.descent_receipt.depth_reached, 1);
        assert_eq!(plan.descent_receipt.budget, 4);
        assert_eq!(
            meter.depth(),
            1,
            "meter itself must reflect the real descent"
        );

        // Manufacture + bind is real as of the crash-recovery pass.
        let root = root_partial_order_over(2);
        let outcome = manufacture_and_bind_child(&root, &plan, ClosureLaw::AllRequired)
            .expect("a one-op plan tape grafts a real child under the root PartialOrder");
        assert!(matches!(outcome.new_root, Powl::PartialOrder { .. }));
        if let Powl::PartialOrder { children, .. } = &outcome.new_root {
            assert_eq!(
                children.len(),
                3,
                "2 original leaves + 1 manufactured child"
            );
            assert!(matches!(children[2], Powl::PartialOrder { .. }));
        }
        assert_eq!(outcome.closure.socket(), &root_socket());
    }

    // -----------------------------------------------------------------
    // resolve_continuation_goal (real: parses admitted PDDL text)
    // -----------------------------------------------------------------

    const RESIDUE_DOMAIN_TEXT: &str = r#"
(define (domain f09-residue-test)
  (:requirements :strips)
  (:predicates (at ?x) (goal-reached))
  (:action move
    :parameters (?x)
    :precondition (at ?x)
    :effect (and (goal-reached))))
"#;
    const RESIDUE_PROBLEM_TEXT: &str = r#"
(define (problem f09-residue-test-problem)
  (:domain f09-residue-test)
  (:objects a)
  (:init (at a))
  (:goal (and (goal-reached))))
"#;

    #[test]
    fn resolve_continuation_goal_parses_real_admitted_pddl_text() {
        let residue = ResidueState {
            socket: root_socket(),
            description: "test residue".into(),
            domain_pddl: RESIDUE_DOMAIN_TEXT.to_string(),
            problem_pddl: RESIDUE_PROBLEM_TEXT.to_string(),
        };
        let goal = resolve_continuation_goal(&residue).expect("real admitted PDDL text parses");
        assert_eq!(goal.domain.name, "f09-residue-test");
        assert_eq!(goal.problem.name, "f09-residue-test-problem");
    }

    #[test]
    fn resolve_continuation_goal_refuses_malformed_domain_text() {
        let residue = ResidueState {
            socket: root_socket(),
            description: "test residue".into(),
            domain_pddl: "not pddl at all".to_string(),
            problem_pddl: RESIDUE_PROBLEM_TEXT.to_string(),
        };
        let err = resolve_continuation_goal(&residue)
            .expect_err("malformed domain text must refuse, never fabricate a goal");
        assert!(matches!(err, MFWGrowthRefused::ResidueMalformed { .. }));
    }

    // -----------------------------------------------------------------
    // manufacture_and_bind_child (real: Powl projection + tree graft)
    // -----------------------------------------------------------------

    #[test]
    fn manufacture_and_bind_child_refuses_an_empty_plan_tape() {
        let closure = open_closure();
        let plan = GrowthPlan {
            parent_socket: root_socket(),
            plan_tape: wasm4pm_compat::pddl::Pddl8Tape { ops: Vec::new() },
            ground_stats: GroundStats {
                candidate_groundings: 0,
                materialized_groundings: 0,
                reachable_atoms: 0,
            },
            descent_receipt: DescentReceipt::seal(root_socket(), &DescentMeter::new(4)),
        };
        let root = root_partial_order_over(2);
        let err = manufacture_and_bind_child(&root, &plan, ClosureLaw::AllRequired)
            .expect_err("empty tape must refuse, never manufacture a placeholder child");
        assert!(matches!(err, MFWGrowthRefused::EmptyPlanTape { .. }));
        let _ = closure; // fixture parity with the other plan_growth tests
    }

    #[test]
    fn manufacture_and_bind_child_refuses_an_unresolvable_socket() {
        let closure = open_closure();
        let goal = reachable_goal();
        let mut meter = DescentMeter::new(4);
        let mut plan = plan_growth(true, &closure, &goal, &mut meter).expect("real gates pass");
        // Point at a socket that does not exist in `root` at all.
        plan.parent_socket = WorkflowSocketId {
            path: SocketPath::root().child(99),
            kind: powl2_decompose::SocketKind::Leaf,
        };
        let root = root_partial_order_over(2);
        let err = manufacture_and_bind_child(&root, &plan, ClosureLaw::AllRequired)
            .expect_err("a socket path that doesn't resolve must refuse, not silently graft");
        assert!(matches!(err, MFWGrowthRefused::GraftRefused { .. }));
    }

    #[test]
    fn manufacture_and_bind_child_is_deterministic() {
        let closure = open_closure();
        let goal = reachable_goal();
        let mut meter_a = DescentMeter::new(4);
        let plan_a = plan_growth(true, &closure, &goal, &mut meter_a).expect("gates pass");
        let mut meter_b = DescentMeter::new(4);
        let plan_b = plan_growth(true, &closure, &goal, &mut meter_b).expect("gates pass");
        let root = root_partial_order_over(2);
        let a =
            manufacture_and_bind_child(&root, &plan_a, ClosureLaw::AllRequired).expect("grafts");
        let b =
            manufacture_and_bind_child(&root, &plan_b, ClosureLaw::AllRequired).expect("grafts");
        assert_eq!(
            format!("{:?}", a.new_root),
            format!("{:?}", b.new_root),
            "same plan tape grafted onto the same root must produce the same tree"
        );
    }

    // -----------------------------------------------------------------
    // project_growth_plan_geometry / GrowthOutcome::geometry* (real F09 ->
    // F10 production edge)
    // -----------------------------------------------------------------

    #[test]
    fn manufacture_and_bind_child_populates_the_real_f10_geometry_projection() {
        let closure = open_closure();
        let goal = reachable_goal();
        let mut meter = DescentMeter::new(4);
        let plan = plan_growth(true, &closure, &goal, &mut meter).expect("all real gates pass");
        let root = root_partial_order_over(2);
        let outcome = manufacture_and_bind_child(&root, &plan, ClosureLaw::AllRequired)
            .expect("a one-op plan tape passes F10's geometry gate and grafts");
        // F10's canonical geometry is a real, independent projection of the
        // same tape (via crate::f10_powl_geometry::build_powl_geometry),
        // not a decorative copy of `outcome.new_root`.
        assert_eq!(outcome.geometry_shape.leaves, plan.plan_tape.ops.len());
        assert_eq!(
            outcome.geometry.source_action_count,
            plan.plan_tape.ops.len()
        );
        assert!(outcome.geometry_turtle.contains("a powl2:Model"));
        assert!(outcome.geometry_turtle.contains(&format!(
            "powl2:activityLabel \"{}\"",
            plan.plan_tape.ops[0].label
        )));
    }

    #[test]
    fn manufacture_and_bind_child_refuses_a_tape_with_self_referential_pred_mask() {
        // F09's own local `manufacture_child_powl` does not inspect
        // `pred_mask` at all -- it always builds a plain (i, i+1) chain
        // regardless of what `pred_mask` says (see that function's doc
        // comment). Without the F10 geometry gate this malformed tape (op 0
        // claims itself as its own predecessor) would graft anyway. This
        // proves the gate this pass adds has real bite: it independently
        // decodes `pred_mask` and refuses what F09's own local logic alone
        // would have silently accepted.
        let action = Pddl8GroundAction {
            schema_name: "self-loop".to_string(),
            label: "self-loop".to_string(),
            preconditions: Vec::new(),
            add_effects: Vec::new(),
            del_effects: Vec::new(),
        };
        let tape = wasm4pm_compat::pddl::Pddl8Tape {
            ops: vec![Pddl8TapeOp {
                index: 0,
                label: "self-loop".to_string(),
                pred_mask: 1, // bit 0 set on op 0 itself: "op 0 precedes op 0".
                action,
            }],
        };
        let plan = GrowthPlan {
            parent_socket: root_socket(),
            plan_tape: tape,
            ground_stats: GroundStats {
                candidate_groundings: 0,
                materialized_groundings: 0,
                reachable_atoms: 0,
            },
            descent_receipt: DescentReceipt::seal(root_socket(), &DescentMeter::new(4)),
        };
        let root = root_partial_order_over(2);
        let err = manufacture_and_bind_child(&root, &plan, ClosureLaw::AllRequired)
            .expect_err("a self-referential pred_mask must be refused by F10's geometry gate");
        assert!(matches!(err, MFWGrowthRefused::GraftRefused { .. }));
        let msg = err.to_string();
        assert!(
            msg.contains("F10 POWL geometry pipeline rejected"),
            "refusal must be attributable to the F10 gate, not a generic graft failure: {msg}"
        );
    }

    // -----------------------------------------------------------------
    // Generated-vs-hand-written consistency (ggen output cross-check)
    // -----------------------------------------------------------------

    #[test]
    fn refusal_catalog_matches_enum_variants() {
        let mut expected = vec![
            "ClosureAlreadySatisfied",
            "ClosureCheckFailed",
            "DescentBudgetExhausted",
            "EmptyPlanTape",
            "GoalUnreachable",
            "GraftRefused",
            "NotYetImplemented",
            "ResidueMalformed",
            "SocketNotBlocked",
        ];
        expected.sort_unstable();
        let mut actual: Vec<&str> = REFUSAL_CATALOG.iter().map(|e| e.name).collect();
        actual.sort_unstable();
        assert_eq!(
            actual, expected,
            "f09_mfw_growth_generated.rs::REFUSAL_CATALOG has drifted from MFWGrowthRefused's \
             actual variants; re-edit packs/f09-mfw-growth-pack/ontology.ttl and regenerate"
        );
    }

    #[test]
    fn provenance_chain_is_eight_stages_in_order() {
        assert_eq!(PROVENANCE_CHAIN.len(), 8);
        for (i, stage) in PROVENANCE_CHAIN.iter().enumerate() {
            assert_eq!(stage.chain_order as usize, i);
        }
        assert_eq!(PROVENANCE_CHAIN[0].name, "ParentWorkflow");
        assert_eq!(PROVENANCE_CHAIN[7].name, "ParentClosureDecision");
    }
}

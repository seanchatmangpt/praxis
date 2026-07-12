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
//!   into [`plan_growth`]) delegates to [`pddl_index::solve_indexed`] --
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
//! ## What is an HONEST STUB (HAND_WRITE_REQUIRED, tracked under V12-009)
//!
//! No existing praxis or `~/` code builds any of the following (confirmed
//! at survey time by a repo-wide grep for this family's own vocabulary --
//! zero hits) and they are not implemented here; each fails loud with
//! [`MFWGrowthRefused::NotYetImplemented`] rather than faking success:
//!
//! - [`resolve_continuation_goal`] -- the Continuation Goal Resolver
//!   (residue state -> concrete PDDL8 goal) is genuinely new judgment-laden
//!   logic. [`plan_growth`] takes an already-resolved [`ContinuationGoal`]
//!   as input precisely so the real downstream gates can be exercised and
//!   tested independently of this unbuilt stage.
//! - [`manufacture_and_bind_child`] -- POWL Manufacturer + Socket Binder +
//!   Parent Re-evaluator. Constructing a real child `powl2_decompose::Powl`
//!   from a `Pddl8Tape`, binding it at the parent's exact socket with L6
//!   provenance, and re-evaluating the parent's closure afterward is new
//!   control-flow logic with no existing code to adapt.
//! - L7 (idempotent/duplicate-safe, restart-durable, chaos-tolerant
//!   re-admission with replay equivalence) is entirely unbuilt: there is no
//!   re-admission loop yet for it to guard.
//!
//! `MFW_AUTONOMIC_RESOLUTION_ALIVE` is **not** claimed by this module: the
//! pipeline does not run end to end (it stops, honestly, at
//! `manufacture_and_bind_child`). `BOUNDED_DESCENT_PROVEN` is claimed only
//! for [`DescentMeter`] in isolation (tested below), not for a full growth
//! cycle, since no cycle completes yet.
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

use pddl_index::GroundStats;
use powl2_decompose::WorkflowSocketId;
use praxis_graphlaw::chatman::closure::RecursiveSocketClosure;
use wasm4pm_compat::hash::blake3_combined;
use wasm4pm_compat::pddl::{Pddl8Domain, Pddl8Problem, Pddl8Tape};

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
}

/// A resolved continuation goal: the concrete PDDL8 domain/problem a
/// blocked socket's growth must plan toward. REUSE_ADAPT: expressed
/// directly in `wasm4pm_compat`'s canonical PDDL8 types -- the same types
/// `pddl_index` and `praxis_graphlaw::chatman::engine`'s S3 PDDL stage
/// already plan over. This module does not invent its own goal
/// representation.
///
/// Producing one of these from a blocked socket's live state is
/// [`resolve_continuation_goal`]'s job, which is not yet implemented; a
/// caller of [`plan_growth`] must supply an already-resolved goal.
#[derive(Debug, Clone)]
pub struct ContinuationGoal {
    /// The PDDL8 domain (action schemas) growth must plan over.
    pub domain: Pddl8Domain,
    /// The PDDL8 problem (objects, initial state, goal) to solve.
    pub problem: Pddl8Problem,
}

/// A blocked socket's residue state -- the L6 data-lens concept this
/// family's provenance chain names between `Socket` and `ContinuationGoal`.
/// Carries no semantics of its own yet; see [`resolve_continuation_goal`].
#[derive(Debug, Clone)]
pub struct ResidueState {
    /// The blocked socket this residue state describes.
    pub socket: WorkflowSocketId,
    /// Human-readable description of the residue (why the socket is
    /// blocked). Not machine-interpreted by anything in this module.
    pub description: String,
}

/// HAND_WRITE_REQUIRED (V12-009): the Continuation Goal Resolver stage.
/// Converting a blocked socket's [`ResidueState`] into a concrete
/// [`ContinuationGoal`] requires real semantic introspection of live engine
/// state that this module does not yet have, and is genuinely new
/// judgment-laden logic per the F09 survey (no existing praxis code
/// performs this conversion). Fails loud rather than fabricating a goal.
///
/// # Errors
/// Always [`MFWGrowthRefused::NotYetImplemented`].
pub fn resolve_continuation_goal(
    _residue: &ResidueState,
) -> Result<ContinuationGoal, MFWGrowthRefused> {
    Err(MFWGrowthRefused::NotYetImplemented {
        stage: "continuation_goal_resolver",
        detail: "converting a blocked socket's residue state into a concrete PDDL8 \
                 domain/problem is HAND_WRITE_REQUIRED per the F09 survey and not \
                 yet built; plan_growth accepts an already-resolved ContinuationGoal \
                 so the real downstream gates can be exercised and tested \
                 independently of this stage",
    })
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
/// [`pddl_index::solve_indexed`] over `goal`.
///
/// # Errors
/// [`MFWGrowthRefused::GoalUnreachable`] if no plan is found.
///
/// # Complexity
/// Bounded by `pddl_index`'s indexed grounder (auto-selects over
/// `pddl_index::GROUND_INDEX_THRESHOLD`) plus BFS plan search bounded by
/// `PDDL8_MAX_PLAN_DEPTH`.
fn reachability_gate(goal: &ContinuationGoal) -> Result<ReachabilityWitness, MFWGrowthRefused> {
    let (plan, stats) = pddl_index::solve_indexed(&goal.domain, &goal.problem).map_err(|e| {
        MFWGrowthRefused::GoalUnreachable {
            reason: e.to_string(),
        }
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
    /// The real plan tape [`pddl_index::solve_indexed`] found.
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

/// HAND_WRITE_REQUIRED (V12-009): POWL Manufacturer + Socket Binder +
/// Parent Re-evaluator -- the pipeline stages after [`plan_growth`].
/// Constructing a real child `powl2_decompose::Powl` from `plan.plan_tape`,
/// binding it at `plan.parent_socket` with exact parent-socket/PDDL-
/// ancestry/descent-budget/closure-path provenance (L6), and re-evaluating
/// the parent's closure afterward is genuinely new, judgment-laden
/// control-flow logic with no existing praxis code to adapt -- confirmed
/// absent by the F09 survey's repo-wide grep for this vocabulary (zero
/// hits). This function fails loud rather than faking success.
///
/// # Errors
/// Always [`MFWGrowthRefused::NotYetImplemented`] currently.
pub fn manufacture_and_bind_child(_plan: &GrowthPlan) -> Result<(), MFWGrowthRefused> {
    Err(MFWGrowthRefused::NotYetImplemented {
        stage: "powl_manufacturer+socket_binder+parent_reevaluator",
        detail: "child-workflow construction from a PDDL plan tape, binding it at the \
                 parent's exact socket, and parent re-evaluation are HAND_WRITE_REQUIRED \
                 per the F09 survey and not yet built; plan_growth's real gates (closure, \
                 reachability, descent budget) must all pass before this is even reached",
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use powl2_decompose::{ParentChildClosure, Powl, SocketKind, SocketPath};
    use praxis_graphlaw::chatman::closure::ClosureLaw;
    use wasm4pm_compat::pddl::{Pddl8ActionSchema, Pddl8Atom};

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

        // The genuinely-unbuilt stage after this point fails loud, honestly.
        assert_eq!(
            manufacture_and_bind_child(&plan),
            Err(MFWGrowthRefused::NotYetImplemented {
                stage: "powl_manufacturer+socket_binder+parent_reevaluator",
                detail: "child-workflow construction from a PDDL plan tape, binding it at \
                         the parent's exact socket, and parent re-evaluation are \
                         HAND_WRITE_REQUIRED per the F09 survey and not yet built; \
                         plan_growth's real gates (closure, reachability, descent budget) \
                         must all pass before this is even reached",
            })
        );
    }

    // -----------------------------------------------------------------
    // resolve_continuation_goal (honest stub)
    // -----------------------------------------------------------------

    #[test]
    fn resolve_continuation_goal_is_honestly_unimplemented() {
        let residue = ResidueState {
            socket: root_socket(),
            description: "test residue".into(),
        };
        assert_eq!(
            resolve_continuation_goal(&residue).unwrap_err(),
            MFWGrowthRefused::NotYetImplemented {
                stage: "continuation_goal_resolver",
                detail: "converting a blocked socket's residue state into a concrete PDDL8 \
                         domain/problem is HAND_WRITE_REQUIRED per the F09 survey and not \
                         yet built; plan_growth accepts an already-resolved ContinuationGoal \
                         so the real downstream gates can be exercised and tested \
                         independently of this stage",
            }
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
            "GoalUnreachable",
            "NotYetImplemented",
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

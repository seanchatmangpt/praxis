//! Family F10 -- "POWL Recursive Process Geometry" (atlas ticket V12-010).
//!
//! Survey verdict: **MIXED**. This pass wires real content for all three of the
//! survey's slices; scope actually covered vs. genuinely deferred is disclosed
//! explicitly below, per `.claude/rules/no-overclaiming.md` -- nothing here is a
//! decorative re-export of a type that doesn't exist, and nothing masquerades a
//! not-yet-built stage as done.
//!
//! ## ALREADY_BUILT (real, re-exported, not re-implemented)
//!
//! [`WfNet`], [`decompose_wf_net`]/[`decompose_wf_net_with_budget`] thinly wrap
//! `powl2_decompose::{net::WfNet, decompose::convert, decompose::convert_with_budget}` --
//! the paper-faithful (Kourani et al.) WF-net -> POWL 2.0 decomposition. That crate's own
//! test suite (28/28, `just powl2-decompose-test`, per this family's survey) is the
//! evidence for that algorithm; this module does not re-verify it, only re-exposes it.
//! [`Powl`], [`ChoiceGraph`], [`GNode`], [`ParentChildClosure`], [`ParentChildEdge`],
//! [`SocketKind`], [`SocketPath`], [`WorkflowSocketId`] are the same re-export: the real
//! Hierarchy Builder + Child Binding Index (`Powl::parent_child_closure`, `Powl::sockets`)
//! this module's own pipeline (below) builds on top of, not a parallel reimplementation.
//!
//! ## REUSE_ADAPT (genuinely adapted, not a drop-in copy)
//!
//! [`to_turtle`] follows the exact Turtle/IRI conventions proven in
//! `crates/cng/src/powl.rs::powl_to_turtle` (`powl2:` prefix, `<base>/n0`, `/c<i>`,
//! `/binding/<i>` structural IRIs, `powl2:ChildBinding`/`childIndex`/`childModel`/
//! `precedes`) but is written fresh against `powl2_decompose::Powl` (which has a
//! `Choice` variant cng's own `Powl` enum does not carry -- cng's own doc comment on
//! `CngRefusal::UnsupportedConstruct` says nested/branching POWL is out of its scope).
//! New predicates ([`POWL2_CHOICE_CLASS`] and friends) needed for `Choice`/loop routing
//! are this module's own extension, disclosed as such, not asserted to be part of any
//! existing shipped `powl2-shapes.ttl` shape.
//!
//! [`validate_shape`] is a genuine but *reduced-scope* adaptation of
//! `crates/cng/src/shape.rs::validate_powl_store`: it checks the same class of structural
//! invariants (binding/node accounting, cyclic-choice coverage) but operates directly on
//! the in-memory [`Powl`] tree plus [`ParentChildClosure`], not via a parsed oxigraph
//! `Store` + SPARQL (`crates/cng/shapes/powl2-shapes.ttl` + the `.rq` query files this
//! module does not use). Adding the oxigraph/SPARQL dependency for a byte-identical port
//! of cng's validator was judged out of scope for a first wiring pass; this is a smaller,
//! real, independently-useful validator, not a stand-in pretending to be the SPARQL one.
//!
//! ## HAND_WRITE_REQUIRED (done this pass, genuinely new)
//!
//! [`Plan`]/[`PlanAction`]/[`ChoiceGroupSpec`] (the pipeline's input shape), the Plan
//! Grouper (`group_into_phases`, adjacency-by-source phase grouping, the same *algorithm*
//! `crates/cng/src/powl.rs::project_tape_to_powl_hierarchical` uses but reimplemented here
//! against this module's own `Plan` type, not `bcinr_pddl::Pddl8Tape` -- see the
//! "Explicitly NOT done" note below), the transitive-closure order derivation
//! (`transitive_closure`), the Partial Order / Choice Graph builders
//! (`build_phase_node`/`build_choice_node`), [`LoopBound`] (no crate in this repo had this
//! entity before this pass, per the family survey), the cycle detector
//! (`choice_graph_has_cycle`, a fresh DFS -- not the private `#[cfg(test)]`-only helper of
//! the same shape in `powl2_decompose::powl::socket_tests`), [`GeometryState`], and
//! [`POWLGeometryRefused`] (the typed refusal taxonomy, four variants, gated to the
//! ORDER_DERIVED / HIERARCHY_BUILT states the survey's L5 restricts REFUSED exits to) are
//! all real, hand-written, and exercised end to end by this file's `#[cfg(test)]` module.
//!
//! ## Explicitly NOT done this pass (disclosed, not silently skipped)
//!
//! - **No integration with an existing plan/provenance type.** [`Plan`] is a minimal,
//!   self-contained input shape (ordered actions + explicit precedence + explicit choice
//!   grouping + a per-action provenance-source string) covering what the pipeline's CTQ
//!   invariants need. It is *not* wired to `bcinr_pddl::Pddl8Tape` + `action_sources`
//!   (cng's real plan-provenance representation) or to any Chatman Engine action-source
//!   type. That integration is real, additional work, not attempted here.
//! - **No receipt head / replay equivalence (L6), no idempotency/correlation gate or
//!   durable receipt/replay state under duplicate/restart/stale-result chaos (L7).** This
//!   module produces an in-memory [`POWLModel`] + Turtle string + [`ShapeReport`] per call;
//!   it does not persist, hash into a receipt envelope, or replay-verify anything. No
//!   `POWLGeometryRefused` variant here stands in for that missing machinery -- it is
//!   simply absent, not faked as refused-by-design.
//! - **State-machine ordering ambiguity, resolved and disclosed, not silently picked.**
//!   The family survey's component pipeline lists "... Loop Bound Binder -> Hierarchy
//!   Builder ..." but its own state machine lists "... HIERARCHY_BUILT -> BOUND ..." (loop
//!   binding *after* hierarchy). This module follows the state machine (it is the more
//!   specific of the two, and is the one that names the REFUSED exits), so
//!   [`build_powl_geometry`] builds the hierarchy first and validates/attaches loop bounds
//!   as part of the same HIERARCHY_BUILT check, before a separate, non-refusing BOUND
//!   attachment step.
//! - **L8's joint claim ceiling is not claimed.** Nothing in this module or its tests
//!   asserts `POWL_RECURSIVE_PROCESS_MANUFACTURE_PROVEN`; that requires production
//!   reachability + chaos/recovery evidence this pass does not attempt to produce.
//!
//! Survey-cited paths for F10 (informed research from the v26.7.12 family survey handed
//! to the scaffolding session inline, not re-verified path-by-path by this wiring pass
//! beyond the powl2-decompose/cng files actually read and used above):
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F10_powl-geometry.md
//! - /Users/sac/praxis/crates/powl2-decompose/{Cargo.toml,src/lib.rs,src/powl.rs,src/decompose.rs}
//! - /Users/sac/praxis/crates/cng/{src/powl.rs,src/shape.rs,src/powl_test.rs,shapes/powl2-shapes.ttl}
//! - /Users/sac/praxis/crates/cng/src/queries/{shape-class-count.rq,shape-bad-precedes.rq}
//! - /Users/sac/praxis/docs/thesis_praxis/03_pddl_powl_functor.md
//! - /Users/sac/powlv2lsp/* (ruled out by the survey: incompatible acyclic-only semantics)
//! - /Users/sac/bcinr/crates/bcinr-pddl/src/powl_bridge.rs (ruled out: unconstructed stub)
//! - /Users/sac/bcinr/crates/bcinr-powl/src (ruled out: downstream execution, not geometry)

use std::collections::{BTreeMap, BTreeSet};

// ---------------------------------------------------------------------------------------
// ALREADY_BUILT: thin re-exports of powl2-decompose's real, tested types and algorithms.
// ---------------------------------------------------------------------------------------

pub use powl2_decompose::{
    convert as decompose_wf_net, convert_with_budget as decompose_wf_net_with_budget, ChoiceGraph,
    GNode, NetError, ParentChildClosure, ParentChildEdge, Powl, Refusal as WfNetDecomposeRefusal,
    RefusalReason as WfNetDecomposeRefusalReason, SocketKind, SocketPath, WfNet, WorkflowSocketId,
    DEFAULT_DEPTH_BUDGET,
};

/// The `▷`/`□` choice-graph sentinels, re-exported from `powl2_decompose::powl` (not
/// re-exported at that crate's own root -- only `pub mod powl` is public there).
use powl2_decompose::powl::{END, START};

// ---------------------------------------------------------------------------------------
// HAND_WRITE_REQUIRED: the pipeline's input shape (Plan Grouper's input).
// ---------------------------------------------------------------------------------------

/// One action in a validated plan, carrying the provenance source that justifies its
/// inclusion. An empty `source` is refused by [`build_powl_geometry`] as
/// [`POWLGeometryRefused::FlatOnlyProvenance`] -- see that variant's doc comment for why.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlanAction {
    /// The activity label this action becomes as a `Powl::Leaf`.
    pub id: String,
    /// The provenance source (e.g. an artifact IRI) this action was derived from.
    pub source: String,
}

/// A choice point: a set of mutually exclusive alternative actions (`members`, indices
/// into [`Plan::actions`]), zero or more of which (`loop_branches`, indices into
/// `members`) redo (route back to the choice point's start) instead of exiting.
#[derive(Debug, Clone, Default)]
pub struct ChoiceGroupSpec {
    /// Action indices that are alternatives of one another. Must have `>= 2` entries.
    pub members: Vec<usize>,
    /// Indices into `members` (not into `Plan::actions`) whose branch redoes the choice
    /// instead of exiting it. A non-empty set here makes the built `ChoiceGraph` cyclic,
    /// which [`build_powl_geometry`] then requires a matching [`LoopBound`] for.
    pub loop_branches: BTreeSet<usize>,
}

/// The F10 pipeline's input: "validated plan and action provenance" (family survey's
/// requirements-summary wording), captured as an ordered action list plus the
/// plan-required precedence and choice-alternative relations declared over it. Nothing
/// beyond what is declared here is ever added as order by [`build_powl_geometry`]
/// (CTQ: "preserving plan-required order without inventing order between independent
/// actions"; "retaining incomparability").
#[derive(Debug, Clone, Default)]
pub struct Plan {
    /// Actions in an arbitrary stable order (index is the identity used by `precedes` and
    /// `choice_groups`; it does not by itself imply execution order -- only `precedes`
    /// does).
    pub actions: Vec<PlanAction>,
    /// `(i, j)` means action `i` is plan-required strictly before action `j`. Need not be
    /// transitively closed; [`build_powl_geometry`] closes it.
    pub precedes: BTreeSet<(usize, usize)>,
    /// Declared choice points.
    pub choice_groups: Vec<ChoiceGroupSpec>,
}

// ---------------------------------------------------------------------------------------
// HAND_WRITE_REQUIRED: LoopBound (no crate in this repo had this entity before this pass).
// ---------------------------------------------------------------------------------------

/// An explicit iteration cap bound to one cyclic choice point (a recursive child binding
/// whose `ChoiceGraph` contains a routing cycle). Attaching a bound with
/// `max_iterations == 0` is refused (a zero-iteration loop is not a loop, it is a
/// contradiction of the choice point's own cyclic structure).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopBound {
    /// Maximum number of times the redo branch(es) may fire before the choice point must
    /// exit. Must be `>= 1`.
    pub max_iterations: u32,
}

// ---------------------------------------------------------------------------------------
// HAND_WRITE_REQUIRED: the pipeline's state machine and typed refusal taxonomy.
// ---------------------------------------------------------------------------------------

/// F10's pipeline state machine (family survey L5), restricted to the states this module
/// actually distinguishes in a refusal. `PLAN_RECEIVED`, `INCOMPARABILITY_PRESERVED`,
/// `CONTROL_FLOW_BUILT`, `BOUND`, `SERIALIZED`, and `VALIDATED` are pass-through states
/// this module reaches but never refuses at (per L5: "REFUSED exits only at ORDER_DERIVED
/// and HIERARCHY_BUILT"); they are omitted from this enum rather than added and left
/// dead, since [`POWLGeometryRefused`] only ever carries one of the two below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryState {
    /// The plan's provenance and declared order/choice structure is being validated and
    /// closed under transitivity.
    OrderDerived,
    /// The `Powl` tree, its parent-child closure, and cyclic-choice loop-bound coverage
    /// are being validated.
    HierarchyBuilt,
}

impl std::fmt::Display for GeometryState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GeometryState::OrderDerived => "ORDER_DERIVED",
            GeometryState::HierarchyBuilt => "HIERARCHY_BUILT",
        })
    }
}

/// Typed refusal for the F10 POWL geometry pipeline. Every variant is reachable and has
/// >= 1 end-to-end test in this file's `#[cfg(test)]` module (per
/// `.claude/rules/rust-agi-core-team.md` rule 5 / `.claude/rules/praxis-rust-discipline.md`
/// "Refusal Taxonomy").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum POWLGeometryRefused {
    /// The plan declares order that cannot be honored without fabricating it: a
    /// `precedes` pair references an out-of-range or self-referential action index, a
    /// choice group's own bookkeeping is malformed (fewer than 2 members, a duplicate
    /// member, an out-of-range `loop_branches` index), or a `precedes` pair orders two
    /// members of the *same* choice group (mutually exclusive alternatives cannot be
    /// sequenced relative to each other -- only one of them ever executes).
    InventedOrder {
        /// Always [`GeometryState::OrderDerived`].
        state: GeometryState,
        /// What was invented, specifically.
        detail: String,
    },
    /// The plan's declared `precedes` relation is self-contradictory: its transitive
    /// closure derives both `(i, j)` and `(j, i)` for some `i != j` (directly, or via a
    /// longer cycle) -- honoring it would require silently dropping one of the declared
    /// edges, i.e. losing precedence the plan required.
    LostPrecedence {
        /// Always [`GeometryState::OrderDerived`].
        state: GeometryState,
        /// Which pair/units contradict, specifically.
        detail: String,
    },
    /// The plan carries no usable action provenance to build a recursive process
    /// geometry from: it has zero actions, or at least one action has an empty
    /// provenance `source`. An empty source makes that action indistinguishable from "no
    /// provenance was ever attached" -- the input degenerates to a flat, undifferentiated
    /// list rather than genuine per-action provenance, which is exactly the CTQ's
    /// "flat-only provenance" refusal case. (This concretization of "flat-only
    /// provenance" is this module's own reading of an underspecified survey requirement,
    /// disclosed as such, not asserted as the only possible reading.)
    FlatOnlyProvenance {
        /// Always [`GeometryState::OrderDerived`].
        state: GeometryState,
        /// Which action (or "the plan") lacks provenance, specifically.
        detail: String,
    },
    /// A recursive child binding in the built hierarchy is not provenance-complete: a
    /// choice group has no exit branch (every member loops back to start, so the process
    /// can never terminate), a cyclic choice point has no matching [`LoopBound`] (or was
    /// given one with `max_iterations == 0`), or (defensive, see [`validate_shape`]) the
    /// built model's leaf/edge accounting does not match the input it was built from.
    UnboundChildBinding {
        /// Always [`GeometryState::HierarchyBuilt`].
        state: GeometryState,
        /// Which binding is unbound, specifically.
        detail: String,
    },
}

impl std::fmt::Display for POWLGeometryRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (kind, state, detail) = match self {
            POWLGeometryRefused::InventedOrder { state, detail } => {
                ("InventedOrder", state, detail)
            }
            POWLGeometryRefused::LostPrecedence { state, detail } => {
                ("LostPrecedence", state, detail)
            }
            POWLGeometryRefused::FlatOnlyProvenance { state, detail } => {
                ("FlatOnlyProvenance", state, detail)
            }
            POWLGeometryRefused::UnboundChildBinding { state, detail } => {
                ("UnboundChildBinding", state, detail)
            }
        };
        write!(f, "F10_POWL_GEOMETRY_REFUSED[{state}] {kind}: {detail}")
    }
}

impl std::error::Error for POWLGeometryRefused {}

// ---------------------------------------------------------------------------------------
// HAND_WRITE_REQUIRED: Plan Grouper.
// ---------------------------------------------------------------------------------------

/// A maximal run of plan-adjacent (in `Plan::actions` order), same-`source` action
/// indices among the actions not claimed by any choice group. Same *algorithm* as
/// `crates/cng/src/powl.rs::project_tape_to_powl_hierarchical`'s phase grouping,
/// reimplemented here against [`Plan`] rather than `bcinr_pddl::Pddl8Tape`.
#[derive(Debug, Clone)]
struct Phase {
    source: String,
    action_indices: Vec<usize>,
}

/// Groups `ungrouped` (action indices not claimed by any choice group, already in
/// `Plan::actions` order) into maximal same-source adjacent runs.
///
/// # Complexity
/// O(n) in `ungrouped.len()`.
fn group_into_phases(plan: &Plan, ungrouped: &[usize]) -> Vec<Phase> {
    let mut phases: Vec<Phase> = Vec::new();
    for &idx in ungrouped {
        let source = plan.actions[idx].source.clone();
        match phases.last_mut() {
            Some(p) if p.source == source => p.action_indices.push(idx),
            _ => phases.push(Phase {
                source,
                action_indices: vec![idx],
            }),
        }
    }
    phases
}

/// A top-level unit of the built model's root `PartialOrder`: either an ungrouped-action
/// phase, or a declared choice point.
#[derive(Debug, Clone)]
enum Unit {
    Phase(Phase),
    Choice { group_index: usize },
}

impl Unit {
    /// Original `Plan::actions` indices this unit covers.
    fn action_indices(&self, plan: &Plan) -> Vec<usize> {
        match self {
            Unit::Phase(p) => p.action_indices.clone(),
            Unit::Choice { group_index } => plan.choice_groups[*group_index].members.clone(),
        }
    }

    /// A deterministic sort key (not itself an execution-order claim -- only the
    /// `precedes`-derived `order` relation on the built `PartialOrder` is that).
    fn min_action_index(&self, plan: &Plan) -> usize {
        match self {
            Unit::Phase(p) => *p.action_indices.first().expect(
                "group_into_phases always creates a Phase with >= 1 action index (the match \
                 arm that constructs one always supplies `vec![idx]`)",
            ),
            Unit::Choice { group_index } => {
                let members = &plan.choice_groups[*group_index].members;
                members.iter().copied().min().expect(
                    "choice group members has >= 2 entries: validated in build_powl_geometry \
                     before any Unit is constructed",
                )
            }
        }
    }
}

/// Which choice group (if any) `action` is a member of.
///
/// # Complexity
/// O(g*m) worst case over `g` choice groups of `m` members each.
fn choice_group_of(plan: &Plan, action: usize) -> Option<usize> {
    plan.choice_groups
        .iter()
        .position(|g| g.members.contains(&action))
}

// ---------------------------------------------------------------------------------------
// HAND_WRITE_REQUIRED: order derivation (Partial Order Builder's closure step).
// ---------------------------------------------------------------------------------------

/// Transitive closure of `pairs` (repeated saturation to a fixpoint). Never adds a pair
/// not implied by `pairs` -- this is what makes "retaining incomparability" automatic:
/// any `(a, b)` absent from the closure simply stays unrelated in the built model.
///
/// # Complexity
/// O(n^3) worst case (bounded saturation passes over up to n^2 pairs each), acceptable
/// for the small plan/unit sizes this first-pass pipeline handles; revisit before use on
/// large-n plans.
fn transitive_closure(pairs: &BTreeSet<(usize, usize)>) -> BTreeSet<(usize, usize)> {
    let mut closure = pairs.clone();
    loop {
        let mut added = false;
        let snapshot: Vec<(usize, usize)> = closure.iter().copied().collect();
        for &(a, b) in &snapshot {
            for &(c, d) in &snapshot {
                if b == c && a != d && closure.insert((a, d)) {
                    added = true;
                }
            }
        }
        if !added {
            break;
        }
    }
    closure
}

// ---------------------------------------------------------------------------------------
// HAND_WRITE_REQUIRED: Partial Order Builder / Choice Graph Builder (per-unit Powl nodes).
// ---------------------------------------------------------------------------------------

/// Builds a phase's `Powl` node: a bare `Leaf` for a single-action phase, or a
/// `PartialOrder` over its actions' leaves with the order restricted (and re-indexed) from
/// `action_closure`.
///
/// # Complexity
/// O(k^2) in the phase's action count `k`.
fn build_phase_node(plan: &Plan, phase: &Phase, action_closure: &BTreeSet<(usize, usize)>) -> Powl {
    if phase.action_indices.len() == 1 {
        let idx = phase.action_indices[0];
        return Powl::Leaf(Some(plan.actions[idx].id.clone()));
    }
    let children: Vec<Powl> = phase
        .action_indices
        .iter()
        .map(|&idx| Powl::Leaf(Some(plan.actions[idx].id.clone())))
        .collect();
    let mut order = BTreeSet::new();
    for (li, &ai) in phase.action_indices.iter().enumerate() {
        for (lj, &aj) in phase.action_indices.iter().enumerate() {
            if li != lj && action_closure.contains(&(ai, aj)) {
                order.insert((li, lj));
            }
        }
    }
    Powl::PartialOrder { children, order }
}

/// Builds a choice group's `Powl::Choice` node: one leaf branch per member, routed
/// `START -> branch`, and `branch -> END` unless the branch is a declared loop branch, in
/// which case `branch -> START` (redo).
///
/// # Complexity
/// O(k) in the group's member count `k`.
fn build_choice_node(plan: &Plan, spec: &ChoiceGroupSpec) -> Powl {
    let children: Vec<Powl> = spec
        .members
        .iter()
        .map(|&idx| Powl::Leaf(Some(plan.actions[idx].id.clone())))
        .collect();
    let n = children.len();
    let mut edges = BTreeSet::new();
    for i in 0..n {
        edges.insert((START, GNode::Child(i)));
        if spec.loop_branches.contains(&i) {
            edges.insert((GNode::Child(i), START));
        } else {
            edges.insert((GNode::Child(i), END));
        }
    }
    Powl::Choice {
        children,
        graph: ChoiceGraph { n, edges },
    }
}

/// Detects whether a [`ChoiceGraph`]'s routing edges contain a cycle reachable from
/// `START` (plain DFS, recursion-stack / back-edge method). Written fresh for this
/// module's production Hierarchy Builder / Shape Validator checks -- the algorithm is the
/// standard textbook technique, not copied from `powl2_decompose::powl`'s private
/// `#[cfg(test)]`-only helper of the same shape (that one is not exported and exists only
/// to validate that crate's own test fixtures).
///
/// # Complexity
/// O(V + E) over the choice graph's nodes and edges.
fn choice_graph_has_cycle(graph: &ChoiceGraph) -> bool {
    fn visit(
        node: GNode,
        graph: &ChoiceGraph,
        visiting: &mut BTreeSet<GNode>,
        done: &mut BTreeSet<GNode>,
    ) -> bool {
        if done.contains(&node) {
            return false;
        }
        if !visiting.insert(node) {
            return true;
        }
        for next in graph.successors(node) {
            if visit(next, graph, visiting, done) {
                return true;
            }
        }
        visiting.remove(&node);
        done.insert(node);
        false
    }
    let mut visiting = BTreeSet::new();
    let mut done = BTreeSet::new();
    visit(START, graph, &mut visiting, &mut done)
}

// ---------------------------------------------------------------------------------------
// The manufactured artifact: POWLModel.
// ---------------------------------------------------------------------------------------

/// The "Canonical POWL v2 model" the family survey requires: a [`Powl`] tree (built from
/// real `powl2_decompose` types), its [`ParentChildClosure`] (Hierarchy Builder + Child
/// Binding Index output), and every attached [`LoopBound`] keyed by the cyclic choice
/// socket it binds.
#[derive(Debug, Clone)]
pub struct POWLModel {
    /// The built model.
    pub root: Powl,
    /// The model's parent-child closure (Hierarchy Builder + Child Binding Index).
    pub closure: ParentChildClosure,
    /// Every attached loop bound, keyed by the cyclic `Powl::Choice` socket it binds.
    pub loop_bounds: BTreeMap<WorkflowSocketId, LoopBound>,
    /// `Plan::actions.len()` this model was built from -- carried for
    /// [`validate_shape`]'s leaf-count check.
    pub source_action_count: usize,
}

/// Runs the F10 pipeline: Plan Grouper -> Partial Order Builder -> Choice Graph Builder ->
/// Hierarchy Builder -> Child Binding Index -> Loop Bound Binder, over `plan`, returning a
/// [`POWLModel`] or a [`POWLGeometryRefused`].
///
/// `loop_bounds` is keyed by index into `plan.choice_groups`; a bound is required for
/// every choice group whose declared `loop_branches` makes its `ChoiceGraph` cyclic
/// (independently re-confirmed by [`choice_graph_has_cycle`], not just trusted from
/// `loop_branches.is_empty()`), and is otherwise ignored (a bound supplied for a
/// non-cyclic choice group is inert, not an error).
///
/// # Errors
/// See [`POWLGeometryRefused`]'s variants.
///
/// # Complexity
/// Dominated by [`transitive_closure`]'s O(n^3) (n = `plan.actions.len()`), plus O(n log n)
/// for `Powl::sockets`/`Powl::parent_child_closure` over the built tree.
pub fn build_powl_geometry(
    plan: &Plan,
    loop_bounds: &BTreeMap<usize, LoopBound>,
) -> Result<POWLModel, POWLGeometryRefused> {
    // ---- ORDER_DERIVED: provenance completeness ----
    if plan.actions.is_empty() {
        return Err(POWLGeometryRefused::FlatOnlyProvenance {
            state: GeometryState::OrderDerived,
            detail: "plan has zero actions; there is no action provenance to derive a \
                     process geometry from"
                .to_string(),
        });
    }
    for (idx, action) in plan.actions.iter().enumerate() {
        if action.source.trim().is_empty() {
            return Err(POWLGeometryRefused::FlatOnlyProvenance {
                state: GeometryState::OrderDerived,
                detail: format!(
                    "action index {idx} ({:?}) has no provenance source; the plan's \
                     provenance is flat/undifferentiated, not recursively groupable",
                    action.id
                ),
            });
        }
    }

    // ---- ORDER_DERIVED: choice group bookkeeping ----
    let mut grouped: BTreeSet<usize> = BTreeSet::new();
    for (gi, group) in plan.choice_groups.iter().enumerate() {
        if group.members.len() < 2 {
            return Err(POWLGeometryRefused::InventedOrder {
                state: GeometryState::OrderDerived,
                detail: format!(
                    "choice group {gi} has {} member(s); a choice with < 2 alternatives is \
                     not a real decision",
                    group.members.len()
                ),
            });
        }
        for &m in &group.members {
            if m >= plan.actions.len() {
                return Err(POWLGeometryRefused::InventedOrder {
                    state: GeometryState::OrderDerived,
                    detail: format!(
                        "choice group {gi} references out-of-range action index {m} \
                         (plan has {} actions)",
                        plan.actions.len()
                    ),
                });
            }
            if !grouped.insert(m) {
                return Err(POWLGeometryRefused::InventedOrder {
                    state: GeometryState::OrderDerived,
                    detail: format!(
                        "action index {m} is a member of more than one choice group (or \
                         repeats within group {gi}); choice-group membership must be unique"
                    ),
                });
            }
        }
        for &b in &group.loop_branches {
            if b >= group.members.len() {
                return Err(POWLGeometryRefused::InventedOrder {
                    state: GeometryState::OrderDerived,
                    detail: format!(
                        "choice group {gi} loop_branches index {b} out of range \
                         ({} members)",
                        group.members.len()
                    ),
                });
            }
        }
    }

    // ---- ORDER_DERIVED: precedes bookkeeping + same-choice-group order rejection ----
    for &(i, j) in &plan.precedes {
        if i >= plan.actions.len() || j >= plan.actions.len() {
            return Err(POWLGeometryRefused::InventedOrder {
                state: GeometryState::OrderDerived,
                detail: format!(
                    "precedes pair ({i}, {j}) references an action index outside \
                     0..{}",
                    plan.actions.len()
                ),
            });
        }
        if i == j {
            return Err(POWLGeometryRefused::InventedOrder {
                state: GeometryState::OrderDerived,
                detail: format!("precedes pair ({i}, {i}) declares an action before itself"),
            });
        }
        if let (Some(gi), Some(gj)) = (choice_group_of(plan, i), choice_group_of(plan, j)) {
            if gi == gj {
                return Err(POWLGeometryRefused::InventedOrder {
                    state: GeometryState::OrderDerived,
                    detail: format!(
                        "precedes pair ({i}, {j}) orders two members of the same choice \
                         group {gi}; mutually exclusive alternatives cannot be sequenced \
                         relative to each other"
                    ),
                });
            }
        }
    }

    // ---- ORDER_DERIVED / INCOMPARABILITY_PRESERVED: action-level closure ----
    let closure = transitive_closure(&plan.precedes);
    for &(i, j) in &closure {
        if closure.contains(&(j, i)) {
            return Err(POWLGeometryRefused::LostPrecedence {
                state: GeometryState::OrderDerived,
                detail: format!(
                    "plan-required order contradicts itself: both ({i}, {j}) and \
                     ({j}, {i}) are derivable from the declared precedes relation"
                ),
            });
        }
    }

    // ---- Plan Grouper: build top-level units in a deterministic order ----
    let ungrouped: Vec<usize> = (0..plan.actions.len())
        .filter(|i| !grouped.contains(i))
        .collect();
    let phases = group_into_phases(plan, &ungrouped);

    let mut units: Vec<Unit> = phases.into_iter().map(Unit::Phase).collect();
    for gi in 0..plan.choice_groups.len() {
        units.push(Unit::Choice { group_index: gi });
    }
    units.sort_by_key(|u| u.min_action_index(plan));

    // action index -> unit index (partition, established above: every action is either in
    // exactly one phase via `ungrouped`, or exactly one choice group via `grouped`).
    let mut action_to_unit: Vec<Option<usize>> = vec![None; plan.actions.len()];
    for (ui, u) in units.iter().enumerate() {
        for a in u.action_indices(plan) {
            action_to_unit[a] = Some(ui);
        }
    }

    // ---- ORDER_DERIVED: aggregate unit-level order from the action-level closure ----
    let mut unit_pairs: BTreeSet<(usize, usize)> = BTreeSet::new();
    for &(i, j) in &closure {
        let ui = action_to_unit[i].expect(
            "action_to_unit[i] is Some for every i in 0..plan.actions.len(): every action \
             index was validated in-range above and is a member of exactly one phase (via \
             `ungrouped`) or exactly one choice group (via `grouped`), which jointly \
             partition 0..plan.actions.len()",
        );
        let uj = action_to_unit[j].expect("see the justification on the `ui` binding above");
        if ui == uj {
            continue; // internal to one unit; already reflected in that unit's own order.
        }
        if unit_pairs.contains(&(uj, ui)) {
            return Err(POWLGeometryRefused::LostPrecedence {
                state: GeometryState::OrderDerived,
                detail: format!(
                    "unit-level order contradiction between unit {ui} and unit {uj}: \
                     derived both directions from member actions {i}/{j}"
                ),
            });
        }
        unit_pairs.insert((ui, uj));
    }
    let root_order = transitive_closure(&unit_pairs);
    for &(a, b) in &root_order {
        if root_order.contains(&(b, a)) {
            return Err(POWLGeometryRefused::LostPrecedence {
                state: GeometryState::OrderDerived,
                detail: format!(
                    "root-level order contradiction between unit {a} and unit {b} after \
                     closure"
                ),
            });
        }
    }

    // ---- CONTROL_FLOW_BUILT: build each unit's Powl node ----
    let mut children: Vec<Powl> = Vec::with_capacity(units.len());
    // group_index -> (socket, is_cyclic), populated only for Unit::Choice units.
    let mut choice_sockets: BTreeMap<usize, (WorkflowSocketId, bool)> = BTreeMap::new();
    for (ui, u) in units.iter().enumerate() {
        match u {
            Unit::Phase(phase) => {
                children.push(build_phase_node(plan, phase, &closure));
            }
            Unit::Choice { group_index } => {
                let spec = &plan.choice_groups[*group_index];
                let node = build_choice_node(plan, spec);
                let is_cyclic = match &node {
                    Powl::Choice { graph, .. } => choice_graph_has_cycle(graph),
                    _ => false,
                };
                let socket = WorkflowSocketId {
                    path: SocketPath::root().child(ui),
                    kind: SocketKind::Choice,
                };
                choice_sockets.insert(*group_index, (socket, is_cyclic));
                children.push(node);
            }
        }
    }
    let root = Powl::PartialOrder {
        children,
        order: root_order,
    };

    // ---- HIERARCHY_BUILT: provenance-completeness of every recursive child binding ----
    let leaf_count = root
        .sockets()
        .iter()
        .filter(|s| s.kind == SocketKind::Leaf)
        .count();
    if leaf_count != plan.actions.len() {
        return Err(POWLGeometryRefused::UnboundChildBinding {
            state: GeometryState::HierarchyBuilt,
            detail: format!(
                "constructed model has {leaf_count} leaves but the plan declared \
                 {} actions; every action must become exactly one leaf, never invented \
                 or lost",
                plan.actions.len()
            ),
        });
    }
    for (&group_index, (socket, is_cyclic)) in &choice_sockets {
        let spec = &plan.choice_groups[group_index];
        if spec.loop_branches.len() == spec.members.len() {
            return Err(POWLGeometryRefused::UnboundChildBinding {
                state: GeometryState::HierarchyBuilt,
                detail: format!(
                    "choice group {group_index} (socket {socket}) has no exit branch; \
                     every member routes back to start, which never terminates"
                ),
            });
        }
        if *is_cyclic {
            match loop_bounds.get(&group_index) {
                None => {
                    return Err(POWLGeometryRefused::UnboundChildBinding {
                        state: GeometryState::HierarchyBuilt,
                        detail: format!(
                            "choice group {group_index} (socket {socket}) has a cyclic \
                             routing graph but no LoopBound was supplied"
                        ),
                    })
                }
                Some(bound) if bound.max_iterations == 0 => {
                    return Err(POWLGeometryRefused::UnboundChildBinding {
                        state: GeometryState::HierarchyBuilt,
                        detail: format!(
                            "choice group {group_index} (socket {socket}) was given a \
                             LoopBound with max_iterations = 0, which is not a valid bound"
                        ),
                    })
                }
                Some(_) => {}
            }
        }
    }

    let closure_index = root.parent_child_closure();

    // ---- BOUND: attach every validated loop bound (pure attachment, no refusal) ----
    let mut applied_loop_bounds: BTreeMap<WorkflowSocketId, LoopBound> = BTreeMap::new();
    for (group_index, (socket, is_cyclic)) in &choice_sockets {
        if *is_cyclic {
            if let Some(bound) = loop_bounds.get(group_index) {
                applied_loop_bounds.insert(socket.clone(), *bound);
            }
        }
    }

    Ok(POWLModel {
        root,
        closure: closure_index,
        loop_bounds: applied_loop_bounds,
        source_action_count: plan.actions.len(),
    })
}

// ---------------------------------------------------------------------------------------
// REUSE_ADAPT: POWL Serializer (adapted from crates/cng/src/powl.rs's Turtle conventions).
// ---------------------------------------------------------------------------------------

/// POWL 2.0 vocabulary namespace -- same IRI cng's serializer uses
/// (`crates/cng/src/powl.rs::POWL2_PREFIX`), so a `Powl::PartialOrder`/`Leaf`/
/// `ExternalCut` subtree serializes identically under both; `Choice`/loop terms below are
/// this module's own extension of that vocabulary.
pub const POWL2_PREFIX: &str = "https://truex.io/ontology/powl2#";
/// PROV-O namespace (reserved for future per-element provenance; not yet attached by
/// [`to_turtle`] -- see the module doc's "Explicitly NOT done" note).
pub const PROV_PREFIX: &str = "http://www.w3.org/ns/prov#";

/// Serializes a [`POWLModel`] as Turtle with deterministic structural IRIs:
/// `<base>/n0` is the root; `PartialOrder`/`Choice` children live at `/c<i>` with
/// `ChildBinding`s at `/binding/<i>`; a `Choice` node's routing graph is emitted as
/// `powl2:routes` edges between its `/start`, `/end`, and `/binding/<i>` terms; a cyclic
/// choice socket with an attached [`LoopBound`] gets one `powl2:loopMaxIterations`
/// literal.
///
/// # Complexity
/// O(n + |order| + |edges|) over the model tree.
pub fn to_turtle(model: &POWLModel, base_iri: &str) -> String {
    let base = base_iri.trim_end_matches('/');
    let mut out = String::new();
    out.push_str(&format!("@prefix powl2: <{POWL2_PREFIX}> .\n"));
    out.push_str(&format!("@prefix base: <{base}/> .\n\n"));
    out.push_str(&format!("<{base}/n0> a powl2:Model .\n"));
    emit_node(
        &model.root,
        base,
        "n0",
        &SocketPath::root(),
        &model.loop_bounds,
        &mut out,
    );
    out
}

fn emit_node(
    node: &Powl,
    base: &str,
    path_str: &str,
    socket_path: &SocketPath,
    loop_bounds: &BTreeMap<WorkflowSocketId, LoopBound>,
    out: &mut String,
) {
    match node {
        Powl::Leaf(None) => {
            out.push_str(&format!(
                "<{base}/{path_str}> a powl2:Leaf, powl2:SilentLeaf .\n"
            ));
        }
        Powl::Leaf(Some(label)) => {
            out.push_str(&format!(
                "<{base}/{path_str}> a powl2:Leaf, powl2:ActivityLeaf ;\n  powl2:activityLabel \"{}\" .\n",
                escape_turtle_literal(label)
            ));
        }
        Powl::PartialOrder { children, order } => {
            out.push_str(&format!("<{base}/{path_str}> a powl2:PartialOrder .\n"));
            emit_children(children, base, path_str, socket_path, loop_bounds, out);
            for (i, j) in order {
                out.push_str(&format!(
                    "<{base}/{path_str}/binding/{i}> powl2:precedes <{base}/{path_str}/binding/{j}> .\n"
                ));
            }
        }
        Powl::Choice { children, graph } => {
            out.push_str(&format!("<{base}/{path_str}> a powl2:Choice .\n"));
            out.push_str(&format!(
                "<{base}/{path_str}/start> a powl2:ChoiceStart .\n<{base}/{path_str}/end> a powl2:ChoiceEnd .\n"
            ));
            emit_children(children, base, path_str, socket_path, loop_bounds, out);
            for (u, v) in &graph.edges {
                out.push_str(&format!(
                    "{} powl2:routes {} .\n",
                    gnode_term(base, path_str, *u),
                    gnode_term(base, path_str, *v)
                ));
            }
            let socket = WorkflowSocketId {
                path: socket_path.clone(),
                kind: SocketKind::Choice,
            };
            if let Some(bound) = loop_bounds.get(&socket) {
                out.push_str(&format!(
                    "<{base}/{path_str}> powl2:loopMaxIterations {} .\n",
                    bound.max_iterations
                ));
            }
        }
        Powl::ExternalCut {
            region,
            projection,
            renderer,
        } => {
            out.push_str(&format!("<{base}/{path_str}> a powl2:ExternalCut .\n"));
            let region_path = format!("{path_str}/region");
            out.push_str(&format!(
                "<{base}/{path_str}> powl2:cutRegion <{base}/{region_path}> ;\n  powl2:cutProjection \"{}\" ;\n  powl2:cutRenderer \"{}\" .\n",
                escape_turtle_literal(projection),
                escape_turtle_literal(renderer)
            ));
            emit_node(
                region,
                base,
                &region_path,
                &socket_path.child(0),
                loop_bounds,
                out,
            );
        }
    }
}

fn emit_children(
    children: &[Powl],
    base: &str,
    path_str: &str,
    socket_path: &SocketPath,
    loop_bounds: &BTreeMap<WorkflowSocketId, LoopBound>,
    out: &mut String,
) {
    for (idx, child) in children.iter().enumerate() {
        let child_path = format!("{path_str}/c{idx}");
        let binding_path = format!("{path_str}/binding/{idx}");
        out.push_str(&format!(
            "<{base}/{path_str}> powl2:hasChild <{base}/{binding_path}> .\n"
        ));
        out.push_str(&format!(
            "<{base}/{binding_path}> a powl2:ChildBinding ;\n  powl2:childIndex {idx} ;\n  powl2:childModel <{base}/{child_path}> .\n"
        ));
        emit_node(
            child,
            base,
            &child_path,
            &socket_path.child(idx),
            loop_bounds,
            out,
        );
    }
}

fn gnode_term(base: &str, path_str: &str, node: GNode) -> String {
    match node {
        GNode::Start => format!("<{base}/{path_str}/start>"),
        GNode::End => format!("<{base}/{path_str}/end>"),
        GNode::Child(i) => format!("<{base}/{path_str}/binding/{i}>"),
    }
}

/// Escapes a string for use inside a double-quoted Turtle literal. Same escaping rule as
/// `crates/cng/src/powl.rs::escape_turtle_literal`, reimplemented here (a few lines,
/// standard Turtle string escaping -- not worth adding a dependency on cng to share).
///
/// # Complexity
/// O(len).
fn escape_turtle_literal(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }
    escaped
}

// ---------------------------------------------------------------------------------------
// REUSE_ADAPT: Shape Validator (tree-native, reduced-scope adaptation of cng's SPARQL one).
// ---------------------------------------------------------------------------------------

/// Outcome of a structural validation pass. See the module doc's REUSE_ADAPT note for how
/// this differs in scope from `crates/cng/src/shape.rs::ShapeReport`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeReport {
    /// Leaf socket count (must equal [`POWLModel::source_action_count`]).
    pub leaves: usize,
    /// `PartialOrder` socket count.
    pub partial_orders: usize,
    /// `Choice` socket count.
    pub choices: usize,
    /// Direct parent-child edge count (must equal total sockets minus one).
    pub child_bindings: usize,
    /// How many cyclic choice sockets carry an attached [`LoopBound`].
    pub bound_loops: usize,
    /// Human-readable statement of what was validated.
    pub shape: &'static str,
}

/// Validates a [`POWLModel`]'s structural shape: leaf count matches the plan it was built
/// from, the parent-child closure is a tree (edges == nodes - 1), and every cyclic choice
/// socket has an attached [`LoopBound`].
///
/// # Errors
/// [`POWLGeometryRefused::UnboundChildBinding`] naming the violated invariant. In normal
/// use through [`build_powl_geometry`] this is unreachable (the same checks already ran
/// during HIERARCHY_BUILT); it is real, independently useful coverage for a
/// hand-constructed or otherwise-obtained [`POWLModel`] (see this file's tests).
///
/// # Complexity
/// O(n log n) (dominated by `Powl::sockets`).
pub fn validate_shape(model: &POWLModel) -> Result<ShapeReport, POWLGeometryRefused> {
    let sockets = model.root.sockets();
    let leaves = sockets
        .iter()
        .filter(|s| s.kind == SocketKind::Leaf)
        .count();
    let partial_orders = sockets
        .iter()
        .filter(|s| s.kind == SocketKind::PartialOrder)
        .count();
    let choices = sockets
        .iter()
        .filter(|s| s.kind == SocketKind::Choice)
        .count();

    if leaves != model.source_action_count {
        return Err(POWLGeometryRefused::UnboundChildBinding {
            state: GeometryState::HierarchyBuilt,
            detail: format!(
                "shape violation: model has {leaves} leaves but source_action_count is \
                 {}; every action must become exactly one leaf, never invented or lost",
                model.source_action_count
            ),
        });
    }

    let total_nodes = sockets.len();
    let edges = model.closure.edges().len();
    if total_nodes == 0 || edges != total_nodes - 1 {
        return Err(POWLGeometryRefused::UnboundChildBinding {
            state: GeometryState::HierarchyBuilt,
            detail: format!(
                "shape violation: parent-child closure has {edges} edges but the model \
                 has {total_nodes} sockets; a tree must have exactly nodes-1 direct edges"
            ),
        });
    }

    let mut bound_loops = 0usize;
    for socket in sockets.iter().filter(|s| s.kind == SocketKind::Choice) {
        if let Some(Powl::Choice { graph, .. }) = model.root.socket_at(&socket.path) {
            let cyclic = choice_graph_has_cycle(graph);
            let bound = model.loop_bounds.contains_key(socket);
            if cyclic && !bound {
                return Err(POWLGeometryRefused::UnboundChildBinding {
                    state: GeometryState::HierarchyBuilt,
                    detail: format!(
                        "shape violation: choice socket {socket} has a cyclic routing \
                         graph with no bound LoopBound entry"
                    ),
                });
            }
            if cyclic && bound {
                bound_loops += 1;
            }
        }
    }

    Ok(ShapeReport {
        leaves,
        partial_orders,
        choices,
        child_bindings: total_nodes.saturating_sub(1),
        bound_loops,
        shape: "F10 tree-native structural validator: leaf-count == source action count, \
                closure edge-count == socket-count - 1, every cyclic choice socket has an \
                attached LoopBound",
    })
}

/// Convenience wrapper: [`build_powl_geometry`] -> [`to_turtle`] -> [`validate_shape`],
/// the full F10 pipeline in one call. Real end-to-end plumbing, not a facade -- every
/// stage it calls is defined in this module and independently unit-tested.
///
/// # Errors
/// Whatever [`build_powl_geometry`] or [`validate_shape`] returns.
pub fn manufacture_powl_v2(
    plan: &Plan,
    loop_bounds: &BTreeMap<usize, LoopBound>,
    base_iri: &str,
) -> Result<(POWLModel, String, ShapeReport), POWLGeometryRefused> {
    let model = build_powl_geometry(plan, loop_bounds)?;
    let turtle = to_turtle(&model, base_iri);
    let report = validate_shape(&model)?;
    Ok((model, turtle, report))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(id: &str, source: &str) -> PlanAction {
        PlanAction {
            id: id.to_string(),
            source: source.to_string(),
        }
    }

    // ---- Already-built re-export: real WF-net decomposition, not a decorative import ----

    #[test]
    fn already_built_decompose_wf_net_reexport_actually_decomposes_a_real_net() {
        // Mirrors powl2-decompose's own `net::tests` shape: a 3-transition sequential
        // WF-net, source -> a -> p1 -> b -> sink.
        let net = WfNet::new(
            ["source", "p1", "sink"].map(str::to_string),
            [
                ("a".to_string(), Some("a".to_string())),
                ("b".to_string(), Some("b".to_string())),
            ],
            [
                ("source".to_string(), "a".to_string()),
                ("p1".to_string(), "b".to_string()),
            ],
            [
                ("a".to_string(), "p1".to_string()),
                ("b".to_string(), "sink".to_string()),
            ],
            "source",
            "sink",
        )
        .expect("a well-formed sequential WF-net must construct");
        let model = decompose_wf_net(&net).expect("a sequential net is trivially separable");
        assert!(matches!(model, Powl::PartialOrder { .. } | Powl::Leaf(_)));
    }

    // ---- Plan Grouper + Partial Order Builder: order preserved, incomparability kept ----

    #[test]
    fn partial_order_builder_preserves_declared_order_and_retains_incomparability() {
        let plan = Plan {
            actions: vec![
                action("a0", "src"),
                action("a1", "src"),
                action("a2", "src"),
            ],
            precedes: BTreeSet::from([(0, 1)]),
            choice_groups: vec![],
        };
        let model = build_powl_geometry(&plan, &BTreeMap::new()).expect("valid plan");
        // One phase (all same source) wrapping all three leaves; order has (0,1) but
        // nothing relating action 2 to either -- incomparability retained.
        let Powl::PartialOrder { children, order } = &model.root else {
            panic!("expected root PartialOrder, got {:?}", model.root);
        };
        assert_eq!(children.len(), 1, "single phase, single top-level unit");
        let Powl::PartialOrder {
            order: phase_order, ..
        } = &children[0]
        else {
            panic!("expected the single unit to be the phase PartialOrder");
        };
        assert!(phase_order.contains(&(0, 1)));
        assert!(!phase_order.contains(&(0, 2)));
        assert!(!phase_order.contains(&(2, 0)));
        assert!(!phase_order.contains(&(1, 2)));
        assert!(order.is_empty(), "one unit: no root-level order to derive");
    }

    #[test]
    fn plan_grouper_splits_phases_by_source_adjacency() {
        let plan = Plan {
            actions: vec![
                action("a0", "src-A"),
                action("a1", "src-A"),
                action("a2", "src-B"),
            ],
            precedes: BTreeSet::from([(0, 1), (1, 2)]),
            choice_groups: vec![],
        };
        let model = build_powl_geometry(&plan, &BTreeMap::new()).expect("valid plan");
        let Powl::PartialOrder { children, order } = &model.root else {
            panic!("expected root PartialOrder");
        };
        assert_eq!(children.len(), 2, "two source-adjacency phases");
        assert_eq!(
            order.len(),
            1,
            "phase A before phase B, derived not invented"
        );
        assert!(order.contains(&(0, 1)));
    }

    // ---- FlatOnlyProvenance ----

    #[test]
    fn flat_only_provenance_refuses_empty_plan() {
        let plan = Plan::default();
        let err = build_powl_geometry(&plan, &BTreeMap::new()).unwrap_err();
        assert!(matches!(
            err,
            POWLGeometryRefused::FlatOnlyProvenance {
                state: GeometryState::OrderDerived,
                ..
            }
        ));
    }

    #[test]
    fn flat_only_provenance_refuses_action_with_empty_source() {
        let plan = Plan {
            actions: vec![action("a0", "")],
            precedes: BTreeSet::new(),
            choice_groups: vec![],
        };
        let err = build_powl_geometry(&plan, &BTreeMap::new()).unwrap_err();
        assert!(matches!(
            err,
            POWLGeometryRefused::FlatOnlyProvenance { .. }
        ));
    }

    // ---- InventedOrder ----

    #[test]
    fn invented_order_refuses_out_of_range_precedes() {
        let plan = Plan {
            actions: vec![action("a0", "src")],
            precedes: BTreeSet::from([(0, 5)]),
            choice_groups: vec![],
        };
        let err = build_powl_geometry(&plan, &BTreeMap::new()).unwrap_err();
        assert!(matches!(err, POWLGeometryRefused::InventedOrder { .. }));
    }

    #[test]
    fn invented_order_refuses_precedes_within_same_choice_group() {
        let plan = Plan {
            actions: vec![action("a0", "src"), action("a1", "src")],
            precedes: BTreeSet::from([(0, 1)]),
            choice_groups: vec![ChoiceGroupSpec {
                members: vec![0, 1],
                loop_branches: BTreeSet::new(),
            }],
        };
        let err = build_powl_geometry(&plan, &BTreeMap::new()).unwrap_err();
        assert!(matches!(err, POWLGeometryRefused::InventedOrder { .. }));
    }

    #[test]
    fn invented_order_refuses_single_member_choice_group() {
        let plan = Plan {
            actions: vec![action("a0", "src")],
            precedes: BTreeSet::new(),
            choice_groups: vec![ChoiceGroupSpec {
                members: vec![0],
                loop_branches: BTreeSet::new(),
            }],
        };
        let err = build_powl_geometry(&plan, &BTreeMap::new()).unwrap_err();
        assert!(matches!(err, POWLGeometryRefused::InventedOrder { .. }));
    }

    // ---- LostPrecedence ----

    #[test]
    fn lost_precedence_refuses_direct_contradiction() {
        let plan = Plan {
            actions: vec![action("a0", "src"), action("a1", "src")],
            precedes: BTreeSet::from([(0, 1), (1, 0)]),
            choice_groups: vec![],
        };
        let err = build_powl_geometry(&plan, &BTreeMap::new()).unwrap_err();
        assert!(matches!(err, POWLGeometryRefused::LostPrecedence { .. }));
    }

    #[test]
    fn lost_precedence_refuses_three_cycle() {
        let plan = Plan {
            actions: vec![
                action("a0", "src"),
                action("a1", "src"),
                action("a2", "src"),
            ],
            precedes: BTreeSet::from([(0, 1), (1, 2), (2, 0)]),
            choice_groups: vec![],
        };
        let err = build_powl_geometry(&plan, &BTreeMap::new()).unwrap_err();
        assert!(matches!(err, POWLGeometryRefused::LostPrecedence { .. }));
    }

    // ---- Choice Graph Builder + UnboundChildBinding ----

    fn two_way_choice_plan(loop_back: bool) -> Plan {
        let mut loop_branches = BTreeSet::new();
        if loop_back {
            loop_branches.insert(0usize);
        }
        Plan {
            actions: vec![action("branch-a", "planner"), action("branch-b", "planner")],
            precedes: BTreeSet::new(),
            choice_groups: vec![ChoiceGroupSpec {
                members: vec![0, 1],
                loop_branches,
            }],
        }
    }

    #[test]
    fn choice_graph_builder_builds_acyclic_choice_without_a_bound() {
        let plan = two_way_choice_plan(false);
        let model =
            build_powl_geometry(&plan, &BTreeMap::new()).expect("acyclic choice needs no bound");
        let Powl::PartialOrder { children, .. } = &model.root else {
            panic!("expected root PartialOrder");
        };
        assert_eq!(children.len(), 1);
        assert!(matches!(children[0], Powl::Choice { .. }));
        assert!(model.loop_bounds.is_empty());
    }

    #[test]
    fn unbound_child_binding_refuses_cyclic_choice_without_loop_bound() {
        let plan = two_way_choice_plan(true);
        let err = build_powl_geometry(&plan, &BTreeMap::new()).unwrap_err();
        assert!(matches!(
            err,
            POWLGeometryRefused::UnboundChildBinding {
                state: GeometryState::HierarchyBuilt,
                ..
            }
        ));
    }

    #[test]
    fn unbound_child_binding_refuses_zero_max_iterations_bound() {
        let plan = two_way_choice_plan(true);
        let bounds = BTreeMap::from([(0usize, LoopBound { max_iterations: 0 })]);
        let err = build_powl_geometry(&plan, &bounds).unwrap_err();
        assert!(matches!(
            err,
            POWLGeometryRefused::UnboundChildBinding { .. }
        ));
    }

    #[test]
    fn unbound_child_binding_refuses_choice_group_with_no_exit_branch() {
        let mut loop_branches = BTreeSet::new();
        loop_branches.insert(0usize);
        loop_branches.insert(1usize);
        let plan = Plan {
            actions: vec![action("branch-a", "planner"), action("branch-b", "planner")],
            precedes: BTreeSet::new(),
            choice_groups: vec![ChoiceGroupSpec {
                members: vec![0, 1],
                loop_branches,
            }],
        };
        let bounds = BTreeMap::from([(0usize, LoopBound { max_iterations: 3 })]);
        let err = build_powl_geometry(&plan, &bounds).unwrap_err();
        assert!(matches!(
            err,
            POWLGeometryRefused::UnboundChildBinding { .. }
        ));
    }

    #[test]
    fn loop_bound_binder_attaches_bound_to_the_correct_choice_socket() {
        let plan = two_way_choice_plan(true);
        let bounds = BTreeMap::from([(0usize, LoopBound { max_iterations: 5 })]);
        let model = build_powl_geometry(&plan, &bounds).expect("cyclic choice with a bound");
        assert_eq!(model.loop_bounds.len(), 1);
        let (socket, bound) = model
            .loop_bounds
            .iter()
            .next()
            .expect("loop_bounds has exactly one entry, checked above");
        assert_eq!(socket.kind, SocketKind::Choice);
        assert_eq!(bound.max_iterations, 5);
    }

    // ---- End-to-end: manufacture_powl_v2 (build -> serialize -> validate) ----

    #[test]
    fn end_to_end_manufacture_powl_v2_produces_turtle_and_passes_shape_validation() {
        let plan = Plan {
            actions: vec![
                action("gather", "planner-A"),
                action("review", "planner-A"),
                action("retry", "planner-B"),
                action("approve", "planner-B"),
            ],
            precedes: BTreeSet::from([(0, 1)]),
            choice_groups: vec![ChoiceGroupSpec {
                members: vec![2, 3],
                loop_branches: BTreeSet::from([0usize]), // member 0 == action 2 ("retry")
            }],
        };
        let bounds = BTreeMap::from([(0usize, LoopBound { max_iterations: 3 })]);
        let (model, turtle, report) =
            manufacture_powl_v2(&plan, &bounds, "https://example.org/proc")
                .expect("well-formed plan with a bounded loop must manufacture");

        assert_eq!(report.leaves, 4);
        assert_eq!(report.choices, 1);
        assert_eq!(report.bound_loops, 1);
        assert_eq!(report.child_bindings, model.root.sockets().len() - 1);

        assert!(turtle.contains("a powl2:Model"));
        assert!(turtle.contains("a powl2:PartialOrder"));
        assert!(turtle.contains("a powl2:Choice"));
        assert!(turtle.contains("powl2:activityLabel \"gather\""));
        assert!(turtle.contains("powl2:activityLabel \"retry\""));
        assert!(turtle.contains("powl2:loopMaxIterations 3"));
        assert!(turtle.contains("powl2:routes"));
        assert!(turtle.contains("powl2:precedes"));
    }

    #[test]
    fn manufacture_powl_v2_is_deterministic() {
        let plan = Plan {
            actions: vec![action("a0", "src"), action("a1", "src")],
            precedes: BTreeSet::from([(0, 1)]),
            choice_groups: vec![],
        };
        let (_, turtle_a, _) =
            manufacture_powl_v2(&plan, &BTreeMap::new(), "https://example.org/p").expect("ok");
        let (_, turtle_b, _) =
            manufacture_powl_v2(&plan, &BTreeMap::new(), "https://example.org/p").expect("ok");
        assert_eq!(turtle_a, turtle_b);
    }

    // ---- Shape Validator as an independent, defensive re-check ----

    #[test]
    fn validate_shape_flags_a_hand_constructed_model_with_an_unbound_cycle() {
        // Bypasses build_powl_geometry entirely: hand-builds a cyclic Choice model with
        // an empty loop_bounds map, proving validate_shape is a real, independent check,
        // not just a restatement of build_powl_geometry's own internal logic.
        let mut edges = BTreeSet::new();
        edges.insert((START, GNode::Child(0)));
        edges.insert((GNode::Child(0), START)); // cyclic: no exit at all
        edges.insert((START, GNode::Child(1)));
        edges.insert((GNode::Child(1), END));
        let root = Powl::Choice {
            children: vec![
                Powl::Leaf(Some("x".to_string())),
                Powl::Leaf(Some("y".to_string())),
            ],
            graph: ChoiceGraph { n: 2, edges },
        };
        let closure = root.parent_child_closure();
        let model = POWLModel {
            root,
            closure,
            loop_bounds: BTreeMap::new(),
            source_action_count: 2,
        };
        let err = validate_shape(&model).unwrap_err();
        assert!(matches!(
            err,
            POWLGeometryRefused::UnboundChildBinding {
                state: GeometryState::HierarchyBuilt,
                ..
            }
        ));
    }

    #[test]
    fn choice_graph_has_cycle_distinguishes_acyclic_from_cyclic() {
        let acyclic = ChoiceGraph {
            n: 1,
            edges: BTreeSet::from([(START, GNode::Child(0)), (GNode::Child(0), END)]),
        };
        assert!(!choice_graph_has_cycle(&acyclic));

        let cyclic = ChoiceGraph {
            n: 1,
            edges: BTreeSet::from([
                (START, GNode::Child(0)),
                (GNode::Child(0), START),
                (GNode::Child(0), END),
            ]),
        };
        assert!(choice_graph_has_cycle(&cyclic));
    }

    #[test]
    fn geometry_refused_display_names_state_and_kind() {
        let err = POWLGeometryRefused::LostPrecedence {
            state: GeometryState::OrderDerived,
            detail: "example".to_string(),
        };
        let rendered = err.to_string();
        assert!(rendered.contains("ORDER_DERIVED"));
        assert!(rendered.contains("LostPrecedence"));
        assert!(rendered.contains("example"));
    }
}

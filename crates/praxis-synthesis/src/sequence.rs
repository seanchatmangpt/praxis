//! Layer 2 — constraint-driven capability sequencing (the SMT lesson).
//!
//! Instead of hand-authoring PDDL, callers *declare* capabilities
//! (preconditions, effects, cost) and a solver discovers a valid execution
//! order **and** parameter bindings. Bindings are enumerated by joining
//! precondition patterns against Layer 1's saturated database — the
//! "Datalog feeds the solver" stack the research recommended.
//!
//! The bundled [`BoundedCsp`] is a deterministic branch-and-bound backtracking
//! solver in prolog8's byte-governor style: hard caps, receipted refusals, no
//! native dependencies. The [`Solver`] trait is the seam where a real SMT
//! backend (Z3, cvc5) plugs in later without touching callers.

use pddl_index::SymId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::datalog::{Atom, Program, Term, MAX_VARS};
use crate::Refusal;

/// Hard cap on plan length.
pub const MAX_STEPS: usize = 16;
/// Hard cap on parameter bindings considered per step per capability.
pub const MAX_BINDINGS_PER_STEP: usize = 256;
/// Hard cap on search nodes.
pub const MAX_NODES: u64 = 100_000;

/// A declared capability: what it needs, what it changes, what it costs.
/// Variables in `add`/`del` must be bound by `pre` (enforced at
/// [`SequenceProblem::new`]) — the same safety invariant as rule heads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    /// Unique name.
    pub name: String,
    /// Number of parameter variables (`Term::Var(0..params)`).
    pub params: u8,
    /// Precondition patterns, joined against the current state.
    pub pre: Vec<Atom>,
    /// Atoms added on execution.
    pub add: Vec<Atom>,
    /// Atoms deleted on execution.
    pub del: Vec<Atom>,
    /// Non-negative cost, summed and minimized.
    pub cost: u32,
}

/// The eight constraint kinds — the doctrine's namesake. Capability
/// references are by name; step indices are within the horizon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Constraint {
    /// Every occurrence of `a` precedes every occurrence of `b`.
    Before {
        /// Earlier capability.
        a: String,
        /// Later capability.
        b: String,
    },
    /// Sugar for `Before { a: b, b: a }`.
    After {
        /// Later capability.
        a: String,
        /// Earlier capability.
        b: String,
    },
    /// `a` may only occur at step `< k` (deadline).
    NotLater {
        /// Constrained capability.
        a: String,
        /// Exclusive step bound.
        k: u8,
    },
    /// `a` may only occur at step `>= k` (release time).
    NotEarlier {
        /// Constrained capability.
        a: String,
        /// Inclusive step bound.
        k: u8,
    },
    /// `a` and `b` may not both appear in one plan.
    Excludes {
        /// First capability.
        a: String,
        /// Second capability.
        b: String,
    },
    /// If `a` appears, `b` must also appear.
    Requires {
        /// Dependent capability.
        a: String,
        /// Required capability.
        b: String,
    },
    /// `a` may appear at most `n` times.
    AtMost {
        /// Constrained capability.
        a: String,
        /// Maximum occurrences.
        n: u8,
    },
    /// Total plan cost may not exceed `max`.
    Budget {
        /// Inclusive cost bound.
        max: u32,
    },
}

impl Constraint {
    /// Human/machine-legible rendering used in unsat cores.
    #[must_use]
    pub fn render(&self) -> String {
        match self {
            Constraint::Before { a, b } => format!("Before({a},{b})"),
            Constraint::After { a, b } => format!("After({a},{b})"),
            Constraint::NotLater { a, k } => format!("NotLater({a},{k})"),
            Constraint::NotEarlier { a, k } => format!("NotEarlier({a},{k})"),
            Constraint::Excludes { a, b } => format!("Excludes({a},{b})"),
            Constraint::Requires { a, b } => format!("Requires({a},{b})"),
            Constraint::AtMost { a, n } => format!("AtMost({a},{n})"),
            Constraint::Budget { max } => format!("Budget({max})"),
        }
    }

    fn names(&self) -> Vec<&str> {
        match self {
            Constraint::Before { a, b }
            | Constraint::After { a, b }
            | Constraint::Excludes { a, b }
            | Constraint::Requires { a, b } => vec![a, b],
            Constraint::NotLater { a, .. }
            | Constraint::NotEarlier { a, .. }
            | Constraint::AtMost { a, .. } => vec![a],
            Constraint::Budget { .. } => vec![],
        }
    }
}

/// A sequencing problem: capabilities + initial state (a saturated
/// [`Program`]'s facts) + goal patterns + horizon + constraints.
#[derive(Debug)]
pub struct SequenceProblem {
    pub(crate) caps: Vec<Capability>,
    pub(crate) init: StateDb,
    pub(crate) goal: Vec<Atom>,
    horizon: usize,
    pub(crate) constraints: Vec<Constraint>,
    /// Predicate id → interned name, snapshotted from the program's dict so
    /// refusal certificates can name facts, not just numbers.
    pub(crate) pred_names: BTreeMap<u32, String>,
    problem_hash: String,
}

/// One solved step: which capability, with which parameter values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundStep {
    /// Capability name.
    pub capability: String,
    /// Bound parameter values (interned IDs), one per parameter.
    pub binding: Vec<u32>,
}

/// Receipt for one solve: how hard the solver worked and what it found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolveReceipt {
    /// Search nodes explored.
    pub nodes_explored: u64,
    /// Branches pruned by branch-and-bound.
    pub pruned: u64,
    /// Content address of the problem (capabilities + init + goal + horizon).
    pub problem_hash: String,
    /// Content address of the discovered plan.
    pub plan_hash: String,
    /// Whether this plan was replayed from a fleet-shared solve cache
    /// rather than searched for.
    #[serde(default)]
    pub replayed: bool,
}

/// A discovered plan with its receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequencePlan {
    /// Ordered bound steps.
    pub steps: Vec<BoundStep>,
    /// Total cost.
    pub cost: u32,
    /// Solve receipt.
    pub receipt: SolveReceipt,
}

/// Content address of a step sequence — the single plan-hash formula.
/// Extracted from the solvers' `solve` bodies so solver and verifier cannot
/// drift: verified-replay recovery recomputes this over a cached plan's steps
/// and compares it to the receipt's claimed `plan_hash`.
#[must_use]
pub fn plan_hash_of(steps: &[BoundStep]) -> String {
    let canon = serde_json::to_string(steps).unwrap_or_default();
    chatman_common::provenance::content_address(canon.as_bytes())
}

/// The solver seam: [`BoundedCsp`] today, a real SMT backend later.
pub trait Solver {
    /// Discover a minimum-cost plan reaching the goal, or refuse with reason.
    fn solve(&self, problem: &SequenceProblem) -> Result<SequencePlan, Refusal>;
}

/// Mutable ground-atom state used during search (supports delete effects,
/// which `FactStore` deliberately does not).
#[derive(Debug, Clone, Default)]
pub(crate) struct StateDb {
    pub(crate) by_pred: BTreeMap<u32, BTreeSet<Vec<u32>>>,
}

impl StateDb {
    pub(crate) fn insert(&mut self, pred: u32, args: Vec<u32>) -> bool {
        self.by_pred.entry(pred).or_default().insert(args)
    }
    pub(crate) fn remove(&mut self, pred: u32, args: &[u32]) -> bool {
        self.by_pred.get_mut(&pred).is_some_and(|s| s.remove(args))
    }
    fn tuples_for(&self, pred: u32) -> impl Iterator<Item = &Vec<u32>> {
        self.by_pred.get(&pred).into_iter().flat_map(BTreeSet::iter)
    }
    /// Enumerate bindings for a positive conjunction. Deterministic order.
    pub(crate) fn join(&self, patterns: &[Atom], cap: usize) -> Vec<[Option<u32>; MAX_VARS]> {
        fn descend(
            db: &StateDb,
            patterns: &[Atom],
            idx: usize,
            binding: &mut [Option<u32>; MAX_VARS],
            out: &mut Vec<[Option<u32>; MAX_VARS]>,
            cap: usize,
        ) {
            if out.len() >= cap {
                return;
            }
            let Some(atom) = patterns.get(idx) else {
                out.push(*binding);
                return;
            };
            let tuples: Vec<Vec<u32>> = db.tuples_for(atom.pred.0).cloned().collect();
            for tuple in &tuples {
                if tuple.len() != atom.args.len() {
                    continue;
                }
                let mut newly: Vec<u8> = Vec::new();
                let mut ok = true;
                for (term, val) in atom.args.iter().zip(tuple.iter()) {
                    match term {
                        Term::Const(c) => {
                            if c.0 != *val {
                                ok = false;
                                break;
                            }
                        }
                        Term::Var(v) => match binding[usize::from(*v)] {
                            Some(b) if b != *val => {
                                ok = false;
                                break;
                            }
                            Some(_) => {}
                            None => {
                                binding[usize::from(*v)] = Some(*val);
                                newly.push(*v);
                            }
                        },
                    }
                }
                if ok {
                    descend(db, patterns, idx + 1, binding, out, cap);
                }
                for v in newly {
                    binding[usize::from(v)] = None;
                }
            }
        }
        let mut out = Vec::new();
        let mut binding = [None; MAX_VARS];
        descend(self, patterns, 0, &mut binding, &mut out, cap);
        out
    }
}

pub(crate) fn ground(atom: &Atom, binding: &[Option<u32>; MAX_VARS]) -> (u32, Vec<u32>) {
    let args = atom
        .args
        .iter()
        .map(|t| match t {
            Term::Const(c) => c.0,
            Term::Var(v) => binding[usize::from(*v)].unwrap_or(u32::MAX),
        })
        .collect();
    (atom.pred.0, args)
}

impl SequenceProblem {
    /// Build a problem from declared capabilities and a saturated program.
    /// `before` pairs are lowered to [`Constraint::Before`]; see
    /// [`with_constraints`](Self::with_constraints) for the full surface.
    pub fn new(
        program: &Program,
        capabilities: Vec<Capability>,
        goal: Vec<Atom>,
        horizon: usize,
        before: Vec<(String, String)>,
    ) -> Result<Self, Refusal> {
        let constraints = before
            .into_iter()
            .map(|(a, b)| Constraint::Before { a, b })
            .collect();
        Self::with_constraints(program, capabilities, goal, horizon, constraints)
    }

    /// Build a problem with the full eight-kind constraint surface.
    /// Validates capability safety, constraint references, and the per-problem
    /// constraint cap (64 = 8×8), and constructs the problem content address.
    pub fn with_constraints(
        program: &Program,
        capabilities: Vec<Capability>,
        goal: Vec<Atom>,
        horizon: usize,
        constraints: Vec<Constraint>,
    ) -> Result<Self, Refusal> {
        if horizon > MAX_STEPS {
            return Err(Refusal::InvalidInput {
                detail: format!("horizon {horizon} exceeds MAX_STEPS ({MAX_STEPS})"),
            });
        }
        if constraints.len() > 64 {
            return Err(Refusal::InvalidInput {
                detail: format!("{} constraints exceed the 64 (8x8) cap", constraints.len()),
            });
        }
        let names: BTreeSet<&str> = capabilities.iter().map(|c| c.name.as_str()).collect();
        if names.len() != capabilities.len() {
            return Err(Refusal::InvalidInput {
                detail: "duplicate capability names".into(),
            });
        }
        for c in &constraints {
            for n in c.names() {
                if !names.contains(n) {
                    return Err(Refusal::InvalidInput {
                        detail: format!("{} names undeclared capability {n}", c.render()),
                    });
                }
            }
        }
        for cap in &capabilities {
            for atom in cap.pre.iter().chain(cap.add.iter()).chain(cap.del.iter()) {
                for t in &atom.args {
                    if let Term::Var(v) = t {
                        if usize::from(*v) >= MAX_VARS || usize::from(*v) >= usize::from(cap.params)
                        {
                            return Err(Refusal::InvalidInput {
                                detail: format!(
                                    "capability {}: variable ?{v} is out of bounds (params={}, MAX_VARS={})",
                                    cap.name, cap.params, MAX_VARS
                                ),
                            });
                        }
                    }
                }
            }
            let bound: BTreeSet<u8> = cap
                .pre
                .iter()
                .flat_map(|a| {
                    a.args.iter().filter_map(|t| match t {
                        Term::Var(v) => Some(*v),
                        Term::Const(_) => None,
                    })
                })
                .collect();
            for atom in cap.add.iter().chain(cap.del.iter()) {
                for t in &atom.args {
                    if let Term::Var(v) = t {
                        if !bound.contains(v) {
                            return Err(Refusal::InvalidInput {
                                detail: format!(
                                    "capability {}: effect variable ?{v} not bound by any precondition",
                                    cap.name
                                ),
                            });
                        }
                    }
                }
            }
        }
        // Snapshot the saturated database into a mutable state.
        let mut init = StateDb::default();
        for pred in program.predicates() {
            for tuple in program
                .tuples_of(pred)
                .map(|(a, t)| t[..usize::from(a)].to_vec())
            {
                init.insert(pred.0, tuple.clone());
            }
        }
        let canon = serde_json::json!({
            "capabilities": capabilities,
            "goal": goal,
            "horizon": horizon,
            "constraints": constraints,
            "fixpoint": program.fixpoint_hash(),
        });
        let problem_hash =
            chatman_common::provenance::content_address(canon.to_string().as_bytes());
        // Snapshot every interned symbol (capability precondition predicates
        // may never appear as facts, but certificates must still name them).
        #[allow(clippy::cast_possible_truncation)]
        let pred_names: BTreeMap<u32, String> = (0..program.dict.len() as u32)
            .map(|i| (i, program.dict.resolve(SymId(i)).to_string()))
            .collect();
        Ok(Self {
            caps: capabilities,
            init,
            goal,
            horizon,
            constraints,
            pred_names,
            problem_hash,
        })
    }

    /// Content address of this problem.
    #[must_use]
    pub fn problem_hash(&self) -> &str {
        &self.problem_hash
    }

    /// The declared capability named `name`, if any.
    #[must_use]
    pub fn capability(&self, name: &str) -> Option<&Capability> {
        self.caps.iter().find(|c| c.name == name)
    }

    /// The declared horizon.
    #[must_use]
    pub fn horizon(&self) -> usize {
        self.horizon
    }

    pub(crate) fn goal_satisfied(&self, state: &StateDb) -> bool {
        !state.join(&self.goal, 1).is_empty() || self.goal.is_empty()
    }

    /// Replay a plan's steps from the initial state and report whether the
    /// goal holds afterward — the differential guard used by tests and the
    /// verifier (independent of the solver's own bookkeeping).
    #[must_use]
    pub fn replay_reaches_goal(&self, plan: &SequencePlan) -> bool {
        let mut state = self.init.clone();
        for step in &plan.steps {
            let Some(cap) = self.capability(&step.capability) else {
                return false;
            };
            // Re-derive a full binding consistent with the recorded params.
            let mut found = None;
            for b in state.join(&cap.pre, MAX_BINDINGS_PER_STEP) {
                let params: Vec<u32> = (0..cap.params)
                    .map(|i| b[usize::from(i)].unwrap_or(u32::MAX))
                    .collect();
                if params == step.binding {
                    found = Some(b);
                    break;
                }
            }
            let Some(binding) = found else {
                return false;
            };
            for atom in &cap.del {
                let (p, args) = ground(atom, &binding);
                state.remove(p, &args);
            }
            for atom in &cap.add {
                let (p, args) = ground(atom, &binding);
                state.insert(p, args);
            }
        }
        self.goal_satisfied(&state)
    }

    /// Check every ordering/occurrence/budget constraint against the plan's
    /// literal step sequence — O(steps × constraints), zero search. Covers
    /// all eight kinds: `Before`/`After`/`Excludes`/`AtMost`/`Requires`/
    /// `Budget`/`NotLater`/`NotEarlier`. A step naming an undeclared
    /// capability fails outright.
    #[must_use]
    pub fn plan_respects_constraints(&self, plan: &SequencePlan) -> bool {
        let steps = &plan.steps;
        if steps
            .iter()
            .any(|s| self.capability(&s.capability).is_none())
        {
            return false;
        }
        let positions = |name: &str| -> Vec<usize> {
            steps
                .iter()
                .enumerate()
                .filter(|(_, s)| s.capability == name)
                .map(|(i, _)| i)
                .collect()
        };
        self.constraints.iter().all(|c| match c {
            Constraint::Before { a, b } | Constraint::After { a: b, b: a } => {
                let pa = positions(a);
                positions(b).iter().all(|&j| pa.iter().any(|&i| i < j))
            }
            Constraint::NotLater { a, k } => positions(a).iter().all(|&i| i < usize::from(*k)),
            Constraint::NotEarlier { a, k } => positions(a).iter().all(|&i| i >= usize::from(*k)),
            Constraint::Excludes { a, b } => positions(a).is_empty() || positions(b).is_empty(),
            Constraint::Requires { a, b } => positions(a).is_empty() || !positions(b).is_empty(),
            Constraint::AtMost { a, n } => positions(a).len() <= usize::from(*n),
            Constraint::Budget { max } => {
                let cost = steps
                    .iter()
                    .filter_map(|s| self.capability(&s.capability))
                    .map(|c| c.cost)
                    .fold(0u32, u32::saturating_add);
                cost <= *max
            }
        })
    }

    /// Fragile precondition predicate names for a capability: preconditions
    /// whose predicate NO capability produces — the producer analysis
    /// Solver8's unsat certificates use, applied to runtime loss. The
    /// initial state is a one-time gift: if such a fact is lost mid-run,
    /// nothing in the plan can lawfully re-produce it (a fact with even one
    /// producer is recoverable by restarting that producer). The geometry
    /// layer wires these to AuthorityVacuum→Refuse branches *before runtime*.
    #[must_use]
    pub(crate) fn fragile_precondition_names(&self, capability: &str) -> Vec<String> {
        let Some(cap) = self.capability(capability) else {
            return Vec::new();
        };
        cap.pre
            .iter()
            .filter(|pre| {
                let pred = pre.pred.0;
                let producers = self
                    .caps
                    .iter()
                    .filter(|c| c.add.iter().any(|a| a.pred.0 == pred))
                    .count();
                producers == 0
            })
            .map(|pre| {
                self.pred_names
                    .get(&pre.pred.0)
                    .cloned()
                    .unwrap_or_else(|| format!("pred#{}", pre.pred.0))
            })
            .collect()
    }

    /// Ground add-effect atoms of a bound step (used by Layer 3 to derive
    /// data-dependency edges).
    #[must_use]
    pub fn step_effects(&self, step: &BoundStep) -> Vec<(u32, Vec<u32>)> {
        self.step_atoms(step, |c| &c.add)
    }

    /// Ground precondition atoms of a bound step (constants and parameter
    /// variables only; non-parameter variables are skipped).
    #[must_use]
    pub fn step_preconditions(&self, step: &BoundStep) -> Vec<(u32, Vec<u32>)> {
        self.step_atoms(step, |c| &c.pre)
    }

    fn step_atoms(
        &self,
        step: &BoundStep,
        select: impl Fn(&Capability) -> &Vec<Atom>,
    ) -> Vec<(u32, Vec<u32>)> {
        let Some(cap) = self.capability(&step.capability) else {
            return Vec::new();
        };
        let mut binding = [None; MAX_VARS];
        for (i, v) in step.binding.iter().enumerate() {
            if i < MAX_VARS {
                binding[i] = Some(*v);
            }
        }
        select(cap)
            .iter()
            .filter(|a| {
                a.args.iter().all(|t| match t {
                    Term::Const(_) => true,
                    Term::Var(v) => binding[usize::from(*v)].is_some(),
                })
            })
            .map(|a| ground(a, &binding))
            .collect()
    }
}

/// Bounded deterministic branch-and-bound backtracking solver.
#[derive(Debug, Default, Clone, Copy)]
pub struct BoundedCsp;

struct Search<'p> {
    problem: &'p SequenceProblem,
    nodes: u64,
    pruned: u64,
    best: Option<(Vec<BoundStep>, u32)>,
}

impl Search<'_> {
    fn dfs(
        &mut self,
        state: &mut StateDb,
        steps: &mut Vec<BoundStep>,
        cost: u32,
    ) -> Result<(), Refusal> {
        self.nodes += 1;
        if self.nodes > MAX_NODES {
            return Err(Refusal::BudgetExceeded {
                what: "search_nodes".into(),
                budget: MAX_NODES,
                spent: self.nodes,
                salvage: format!(
                    "best plan so far: {:?} (cost {:?}); depth {} at cutoff",
                    self.best.as_ref().map(|(s, _)| s.len()),
                    self.best.as_ref().map(|(_, c)| *c),
                    steps.len()
                ),
            });
        }
        if self.problem.goal_satisfied(state) {
            if self.best.as_ref().is_none_or(|(_, c)| cost < *c) {
                self.best = Some((steps.clone(), cost));
            }
            return Ok(());
        }
        if steps.len() >= self.problem.horizon {
            return Ok(());
        }
        // Deterministic order: declaration order, then sorted bindings.
        let caps = self.problem.caps.clone();
        for cap in &caps {
            if cost.saturating_add(cap.cost) >= self.best.as_ref().map_or(u32::MAX, |(_, c)| *c) {
                self.pruned += 1;
                continue;
            }
            // Before(a, b): b may not be placed until a has been.
            // After(a, b): a may not be placed until b has been.
            let blocked = self.problem.constraints.iter().any(|c| match c {
                Constraint::Before { a, b } => {
                    *b == cap.name && !steps.iter().any(|s| s.capability == *a)
                }
                Constraint::After { a, b } => {
                    *a == cap.name && !steps.iter().any(|s| s.capability == *b)
                }
                _ => false,
            });
            if blocked {
                continue;
            }
            let bindings = state.join(&cap.pre, MAX_BINDINGS_PER_STEP);
            for binding in bindings {
                let params: Vec<u32> = (0..cap.params)
                    .map(|i| binding[usize::from(i)].unwrap_or(u32::MAX))
                    .collect();
                // Apply effects, tracking exactly what changed for undo.
                let mut added: Vec<(u32, Vec<u32>)> = Vec::new();
                let mut removed: Vec<(u32, Vec<u32>)> = Vec::new();
                for atom in &cap.del {
                    let (p, args) = ground(atom, &binding);
                    if state.remove(p, &args) {
                        removed.push((p, args));
                    }
                }
                for atom in &cap.add {
                    let (p, args) = ground(atom, &binding);
                    if state.insert(p, args.clone()) {
                        added.push((p, args));
                    }
                }
                steps.push(BoundStep {
                    capability: cap.name.clone(),
                    binding: params,
                });
                self.dfs(state, steps, cost.saturating_add(cap.cost))?;
                steps.pop();
                for (p, args) in added {
                    state.remove(p, &args);
                }
                for (p, args) in removed {
                    state.insert(p, args);
                }
            }
        }
        Ok(())
    }
}

impl Solver for BoundedCsp {
    fn solve(&self, problem: &SequenceProblem) -> Result<SequencePlan, Refusal> {
        let mut search = Search {
            problem,
            nodes: 0,
            pruned: 0,
            best: None,
        };
        let mut state = problem.init.clone();
        let mut steps = Vec::new();
        search.dfs(&mut state, &mut steps, 0)?;
        let Some((steps, cost)) = search.best else {
            return Err(Refusal::Unsatisfiable {
                detail: format!(
                    "goal unreachable within horizon {} from {} initial atoms",
                    problem.horizon,
                    problem
                        .init
                        .by_pred
                        .values()
                        .map(BTreeSet::len)
                        .sum::<usize>()
                ),
                nodes_explored: search.nodes,
            });
        };
        let plan_hash = plan_hash_of(&steps);
        Ok(SequencePlan {
            steps,
            cost,
            receipt: SolveReceipt {
                nodes_explored: search.nodes,
                pruned: search.pruned,
                problem_hash: problem.problem_hash.clone(),
                plan_hash,
                replayed: false,
            },
        })
    }
}

/// Convenience: pattern atom from interned symbols.
#[must_use]
pub fn pat(pred: SymId, args: Vec<Term>) -> Atom {
    Atom::new(pred, args)
}

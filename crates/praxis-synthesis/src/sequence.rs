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

/// A sequencing problem: capabilities + initial state (a saturated
/// [`Program`]'s facts) + goal patterns + horizon.
#[derive(Debug)]
pub struct SequenceProblem {
    caps: Vec<Capability>,
    init: StateDb,
    goal: Vec<Atom>,
    horizon: usize,
    before: Vec<(String, String)>,
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

/// The solver seam: [`BoundedCsp`] today, a real SMT backend later.
pub trait Solver {
    /// Discover a minimum-cost plan reaching the goal, or refuse with reason.
    fn solve(&self, problem: &SequenceProblem) -> Result<SequencePlan, Refusal>;
}

/// Mutable ground-atom state used during search (supports delete effects,
/// which `FactStore` deliberately does not).
#[derive(Debug, Clone, Default)]
struct StateDb {
    by_pred: BTreeMap<u32, BTreeSet<Vec<u32>>>,
}

impl StateDb {
    fn insert(&mut self, pred: u32, args: Vec<u32>) -> bool {
        self.by_pred.entry(pred).or_default().insert(args)
    }
    fn remove(&mut self, pred: u32, args: &[u32]) -> bool {
        self.by_pred.get_mut(&pred).is_some_and(|s| s.remove(args))
    }
    fn tuples_for(&self, pred: u32) -> impl Iterator<Item = &Vec<u32>> {
        self.by_pred.get(&pred).into_iter().flat_map(BTreeSet::iter)
    }
    /// Enumerate bindings for a positive conjunction. Deterministic order.
    fn join(&self, patterns: &[Atom], cap: usize) -> Vec<[Option<u32>; MAX_VARS]> {
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

fn ground(atom: &Atom, binding: &[Option<u32>; MAX_VARS]) -> (u32, Vec<u32>) {
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
    /// Validates capability safety and constructs the problem content address.
    pub fn new(
        program: &Program,
        capabilities: Vec<Capability>,
        goal: Vec<Atom>,
        horizon: usize,
        before: Vec<(String, String)>,
    ) -> Result<Self, Refusal> {
        if horizon > MAX_STEPS {
            return Err(Refusal::InvalidInput {
                detail: format!("horizon {horizon} exceeds MAX_STEPS ({MAX_STEPS})"),
            });
        }
        let names: BTreeSet<&str> = capabilities.iter().map(|c| c.name.as_str()).collect();
        if names.len() != capabilities.len() {
            return Err(Refusal::InvalidInput { detail: "duplicate capability names".into() });
        }
        for (a, b) in &before {
            if !names.contains(a.as_str()) || !names.contains(b.as_str()) {
                return Err(Refusal::InvalidInput {
                    detail: format!("before({a},{b}) names an undeclared capability"),
                });
            }
        }
        for cap in &capabilities {
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
            for tuple in program.facts().tuples_for(pred) {
                init.insert(pred.0, tuple.clone());
            }
        }
        let canon = serde_json::json!({
            "capabilities": capabilities,
            "goal": goal,
            "horizon": horizon,
            "before": before,
            "fixpoint": program.fixpoint_hash(),
        });
        let problem_hash =
            chatman_common::provenance::content_address(canon.to_string().as_bytes());
        Ok(Self { caps: capabilities, init, goal, horizon, before, problem_hash })
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

    fn goal_satisfied(&self, state: &StateDb) -> bool {
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
            // before(a, b): b may not be placed until a has been.
            let blocked = self.problem.before.iter().any(|(a, b)| {
                *b == cap.name && !steps.iter().any(|s| s.capability == *a)
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
                steps.push(BoundStep { capability: cap.name.clone(), binding: params });
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
        let mut search = Search { problem, nodes: 0, pruned: 0, best: None };
        let mut state = problem.init.clone();
        let mut steps = Vec::new();
        search.dfs(&mut state, &mut steps, 0)?;
        let Some((steps, cost)) = search.best else {
            return Err(Refusal::Unsatisfiable {
                detail: format!(
                    "goal unreachable within horizon {} from {} initial atoms",
                    problem.horizon,
                    problem.init.by_pred.values().map(BTreeSet::len).sum::<usize>()
                ),
                nodes_explored: search.nodes,
            });
        };
        let plan_canon = serde_json::to_string(&steps).unwrap_or_default();
        let plan_hash = chatman_common::provenance::content_address(plan_canon.as_bytes());
        Ok(SequencePlan {
            steps,
            cost,
            receipt: SolveReceipt {
                nodes_explored: search.nodes,
                pruned: search.pruned,
                problem_hash: problem.problem_hash.clone(),
                plan_hash,
            },
        })
    }
}

/// Convenience: pattern atom from interned symbols.
#[must_use]
pub fn pat(pred: SymId, args: Vec<Term>) -> Atom {
    Atom::new(pred, args)
}

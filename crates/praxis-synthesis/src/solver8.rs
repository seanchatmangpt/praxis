//! Solver8 — the 8-constraint propagating kernel whose refusals are
//! certificates (P1 of the phase-change program).
//!
//! Structure, in prolog8's mold:
//!
//! 1. **Mask propagation.** Horizon ≤ 16, so each capability's feasible-step
//!    set is one `u16`. `NotLater`/`NotEarlier` carve windows;
//!    `Before`/`After` propagate earliest/latest bounds between masks.
//!    AC-3-style iteration to fixpoint, provably ≤ `16 × |caps|` rounds
//!    (each round must clear at least one bit somewhere or stop).
//! 2. **Mandatory analysis.** A capability that is the *sole producer* of a
//!    goal predicate must appear in any plan; the mandatory set closes under
//!    `Requires`. A mandatory capability whose mask empties is a proof of
//!    unsatisfiability — *before any search*.
//! 3. **Bounded MUS.** With ≤ 64 constraints, deletion-based minimal-core
//!    extraction is ≤ 64 propagation passes: drop each constraint, re-run;
//!    if still unsat, drop it permanently, else keep it. What remains is a
//!    minimal conflicting core — the refusal's named culprits. A second
//!    agent verifies impossibility by re-propagating the core alone.
//! 4. **Pruned search.** DFS runs only over steps propagation left alive,
//!    enforcing `Excludes`/`AtMost`/`Budget` incrementally and `Requires`
//!    at goal.
//!
//! [`CoreCache`] shares unsat cores fleet-wide: a dead end derived once is a
//! cache hit for every agent forever — the compounding effect no per-agent
//! solver has.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::sequence::{
    ground, BoundStep, Capability, Constraint, SequencePlan, SequenceProblem, SolveReceipt,
    Solver, MAX_BINDINGS_PER_STEP, MAX_NODES,
};
use crate::Refusal;

/// Fleet-shared solve cache, keyed by problem content address: certified
/// dead ends (unsat cores) *and* discovered plans both replay. A deliberation
/// performed once — either outcome — is a lookup for every later agent.
#[derive(Debug, Default, Clone)]
pub struct CoreCache {
    cores: HashMap<String, (String, Vec<String>)>,
    plans: HashMap<String, SequencePlan>,
    hits: u64,
    plan_hits: u64,
}

impl CoreCache {
    /// Empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Number of cached cores.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cores.len()
    }
    /// Whether the cache holds no cores (plans may still be cached).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cores.is_empty()
    }
    /// Number of cached plans.
    #[must_use]
    pub fn plan_count(&self) -> usize {
        self.plans.len()
    }
    /// How many refusals were answered from cache.
    #[must_use]
    pub fn hits(&self) -> u64 {
        self.hits
    }
    /// How many plans were answered from cache.
    #[must_use]
    pub fn plan_hits(&self) -> u64 {
        self.plan_hits
    }
    /// Discard any cached plan and unsat core for `problem_hash`, forcing the
    /// next `solve_cached` to genuinely re-derive. Node-fault injection uses
    /// this; returns true if anything was evicted.
    pub fn evict(&mut self, problem_hash: &str) -> bool {
        let core = self.cores.remove(problem_hash).is_some();
        let plan = self.plans.remove(problem_hash).is_some();
        core || plan
    }
}

/// The propagating solver. Stateless; pair with a [`CoreCache`] via
/// [`solve_cached`](Self::solve_cached) for fleet-wide dead-end sharing.
#[derive(Debug, Default, Clone, Copy)]
pub struct Solver8;

/// Window state: one feasible-step bitmask per capability index.
type Masks = Vec<u16>;

fn full_mask(horizon: usize) -> u16 {
    if horizon >= 16 {
        u16::MAX
    } else {
        (1u16 << horizon) - 1
    }
}

/// Apply one constraint to the masks. Returns true if any mask changed.
fn apply(c: &Constraint, idx: &BTreeMap<&str, usize>, masks: &mut Masks) -> bool {
    let mut changed = false;
    let mut narrow = |i: usize, keep: u16, masks: &mut Masks| {
        let next = masks[i] & keep;
        if next != masks[i] {
            masks[i] = next;
            changed = true;
        }
    };
    match c {
        Constraint::NotLater { a, k } => {
            let keep = if *k >= 16 { u16::MAX } else { (1u16 << k) - 1 };
            narrow(idx[a.as_str()], keep, masks);
        }
        Constraint::NotEarlier { a, k } => {
            let keep = if *k >= 16 { 0 } else { !((1u16 << k) - 1) };
            narrow(idx[a.as_str()], keep, masks);
        }
        Constraint::Before { a, b } | Constraint::After { b, a } => {
            let (ia, ib) = (idx[a.as_str()], idx[b.as_str()]);
            // earliest(b) > earliest(a): clear b's bits at or below a's earliest.
            if masks[ia] != 0 {
                let ea = masks[ia].trailing_zeros() as u16;
                let keep = if ea >= 15 { 0 } else { !((1u16 << (ea + 1)) - 1) };
                narrow(ib, keep, masks);
            }
            // latest(a) < latest(b): clear a's bits at or above b's latest.
            if masks[ib] != 0 {
                let lb = 15 - masks[ib].leading_zeros() as u16;
                let keep = if lb == 0 { 0 } else { (1u16 << lb) - 1 };
                narrow(ia, keep, masks);
            }
        }
        // Excludes / Requires / AtMost / Budget do not narrow step windows.
        _ => {}
    }
    changed
}

/// Propagate all window constraints to fixpoint. Returns final masks.
fn propagate(
    caps: &[Capability],
    constraints: &[Constraint],
    horizon: usize,
) -> Masks {
    let idx: BTreeMap<&str, usize> =
        caps.iter().enumerate().map(|(i, c)| (c.name.as_str(), i)).collect();
    let mut masks = vec![full_mask(horizon); caps.len()];
    // Each productive round clears ≥ 1 bit; ≤ 16 × |caps| bits exist.
    for _ in 0..=(16 * caps.len()) {
        let mut changed = false;
        for c in constraints {
            changed |= apply(c, &idx, &mut masks);
        }
        if !changed {
            break;
        }
    }
    masks
}

fn init_has(problem: &SequenceProblem, pred: u32) -> bool {
    problem.init.by_pred.get(&pred).is_some_and(|s| !s.is_empty())
}

fn producers_of(problem: &SequenceProblem, pred: u32) -> Vec<usize> {
    problem
        .caps
        .iter()
        .enumerate()
        .filter(|(_, c)| c.add.iter().any(|a| a.pred.0 == pred))
        .map(|(i, _)| i)
        .collect()
}

/// The mandatory set: sole producers of goal predicates, closed under
/// `Requires` and under sole-producer precondition support (a mandatory
/// capability's precondition predicate that the initial state lacks and only
/// one capability produces makes that producer mandatory too).
/// (Conservative: several producers make none individually mandatory.)
fn mandatory_set(problem: &SequenceProblem) -> BTreeSet<usize> {
    let caps = &problem.caps;
    let mut mandatory: BTreeSet<usize> = BTreeSet::new();
    for g in &problem.goal {
        if init_has(problem, g.pred.0) {
            continue;
        }
        let producers = producers_of(problem, g.pred.0);
        if producers.len() == 1 {
            mandatory.insert(producers[0]);
        }
    }
    loop {
        let mut grew = false;
        // Requires: mandatory(a) ∧ Requires(a,b) ⇒ mandatory(b).
        for c in &problem.constraints {
            if let Constraint::Requires { a, b } = c {
                let ia = caps.iter().position(|x| x.name == *a);
                let ib = caps.iter().position(|x| x.name == *b);
                if let (Some(ia), Some(ib)) = (ia, ib) {
                    if mandatory.contains(&ia) && mandatory.insert(ib) {
                        grew = true;
                    }
                }
            }
        }
        // Precondition support: mandatory(m) needs pred p, init lacks p, and
        // exactly one capability produces p ⇒ that producer is mandatory.
        let snapshot: Vec<usize> = mandatory.iter().copied().collect();
        for m in snapshot {
            for pre in &caps[m].pre {
                if init_has(problem, pre.pred.0) {
                    continue;
                }
                let producers = producers_of(problem, pre.pred.0);
                if producers.len() == 1 && mandatory.insert(producers[0]) {
                    grew = true;
                }
            }
        }
        if !grew {
            break;
        }
    }
    mandatory
}

/// Fact-level certification: a mandatory capability with a precondition
/// predicate that the initial state lacks and *no* capability produces is
/// unsatisfiable regardless of ordering — the certificate names the missing
/// fact itself.
fn missing_support(
    problem: &SequenceProblem,
    mandatory: &BTreeSet<usize>,
) -> Option<(String, String)> {
    for &m in mandatory {
        for pre in &problem.caps[m].pre {
            let pred = pre.pred.0;
            if !init_has(problem, pred) && producers_of(problem, pred).is_empty() {
                let name = problem
                    .pred_names
                    .get(&pred)
                    .cloned()
                    .unwrap_or_else(|| format!("pred#{pred}"));
                return Some((problem.caps[m].name.clone(), name));
            }
        }
    }
    None
}

/// Check whether `constraints` alone empty any mandatory capability's window.
/// Returns the first emptied mandatory capability's name if so.
fn empties_mandatory(
    problem: &SequenceProblem,
    constraints: &[Constraint],
    mandatory: &BTreeSet<usize>,
) -> Option<String> {
    let masks = propagate(&problem.caps, constraints, problem.horizon());
    mandatory
        .iter()
        .find(|i| masks[**i] == 0)
        .map(|i| problem.caps[*i].name.clone())
}

/// Deletion-based minimal unsatisfiable core: ≤ |constraints| propagation
/// passes (≤ 64 by the problem cap).
fn extract_core(
    problem: &SequenceProblem,
    mandatory: &BTreeSet<usize>,
) -> Vec<Constraint> {
    let mut core: Vec<Constraint> = problem.constraints.clone();
    let mut i = 0;
    while i < core.len() {
        let mut trial = core.clone();
        trial.remove(i);
        if empties_mandatory(problem, &trial, mandatory).is_some() {
            core = trial; // still unsat without it — drop permanently
        } else {
            i += 1; // needed for the conflict — keep
        }
    }
    core
}

impl Solver8 {
    /// Solve with the fleet-shared cache: a problem whose content address
    /// matches a cached dead end refuses immediately with the replayed
    /// certificate; one matching a cached plan returns it with
    /// `nodes_explored = 0` (this solve searched nothing) and
    /// `replayed = true`.
    pub fn solve_cached(
        &self,
        problem: &SequenceProblem,
        cache: &mut CoreCache,
    ) -> Result<SequencePlan, Refusal> {
        if let Some((detail, core)) = cache.cores.get(problem.problem_hash()) {
            cache.hits += 1;
            return Err(Refusal::UnsatProof {
                detail: detail.clone(),
                core: core.clone(),
                replayed: true,
            });
        }
        if let Some(plan) = cache.plans.get(problem.problem_hash()) {
            cache.plan_hits += 1;
            let mut replay = plan.clone();
            replay.receipt.nodes_explored = 0;
            replay.receipt.pruned = 0;
            replay.receipt.replayed = true;
            return Ok(replay);
        }
        let result = self.solve(problem);
        match &result {
            Err(Refusal::UnsatProof { detail, core, .. }) => {
                cache.cores.insert(
                    problem.problem_hash().to_string(),
                    (detail.clone(), core.clone()),
                );
            }
            Ok(plan) => {
                cache.plans.insert(problem.problem_hash().to_string(), plan.clone());
            }
            Err(_) => {}
        }
        result
    }
}

impl Solver for Solver8 {
    fn solve(&self, problem: &SequenceProblem) -> Result<SequencePlan, Refusal> {
        let caps = &problem.caps;
        let mandatory = mandatory_set(problem);

        // Fact-level certification: a mandatory capability whose precondition
        // nothing supplies. The core names the missing fact.
        if let Some((victim, missing)) = missing_support(problem, &mandatory) {
            return Err(Refusal::UnsatProof {
                detail: format!(
                    "mandatory capability '{victim}' requires '{missing}', which the \
                     initial state lacks and no capability produces"
                ),
                core: vec![format!("MissingFact({missing})")],
                replayed: false,
            });
        }

        // Pre-search certification: do the window constraints alone make a
        // mandatory capability impossible?
        if let Some(victim) = empties_mandatory(problem, &problem.constraints, &mandatory) {
            let core = extract_core(problem, &mandatory);
            return Err(Refusal::UnsatProof {
                detail: format!(
                    "mandatory capability '{victim}' has an empty feasible-step window \
                     within horizon {}",
                    problem.horizon()
                ),
                core: core.iter().map(Constraint::render).collect(),
                replayed: false,
            });
        }

        let masks = propagate(caps, &problem.constraints, problem.horizon());
        let budget: u32 = problem
            .constraints
            .iter()
            .filter_map(|c| match c {
                Constraint::Budget { max } => Some(*max),
                _ => None,
            })
            .min()
            .unwrap_or(u32::MAX);

        let mut search = Search8 {
            problem,
            masks: &masks,
            budget,
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
                    "goal unreachable within horizon {} under {} constraints \
                     (windows nonempty; exhausted pruned search)",
                    problem.horizon(),
                    problem.constraints.len()
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
                problem_hash: problem.problem_hash().to_string(),
                plan_hash,
                replayed: false,
            },
        })
    }
}

struct Search8<'p> {
    problem: &'p SequenceProblem,
    masks: &'p Masks,
    budget: u32,
    nodes: u64,
    pruned: u64,
    best: Option<(Vec<BoundStep>, u32)>,
}

impl Search8<'_> {
    fn requires_ok(&self, steps: &[BoundStep]) -> bool {
        self.problem.constraints.iter().all(|c| match c {
            Constraint::Requires { a, b } => {
                !steps.iter().any(|s| s.capability == *a)
                    || steps.iter().any(|s| s.capability == *b)
            }
            _ => true,
        })
    }

    #[allow(clippy::too_many_lines)]
    fn dfs(
        &mut self,
        state: &mut crate::sequence::StateDb,
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
                    "best plan so far: {:?} steps; depth {} at cutoff",
                    self.best.as_ref().map(|(s, _)| s.len()),
                    steps.len()
                ),
            });
        }
        if self.problem.goal_satisfied(state) && self.requires_ok(steps) {
            if self.best.as_ref().is_none_or(|(_, c)| cost < *c) {
                self.best = Some((steps.clone(), cost));
            }
            return Ok(());
        }
        let t = steps.len();
        if t >= self.problem.horizon() {
            return Ok(());
        }
        let caps = self.problem.caps.clone();
        for (ci, cap) in caps.iter().enumerate() {
            // Propagated window: capability infeasible at this step index.
            if self.masks[ci] & (1u16 << t) == 0 {
                self.pruned += 1;
                continue;
            }
            let next_cost = cost.saturating_add(cap.cost);
            if next_cost > self.budget
                || next_cost >= self.best.as_ref().map_or(u32::MAX, |(_, c)| *c)
            {
                self.pruned += 1;
                continue;
            }
            // Before(a,b): b needs a already placed.
            // Excludes(a,b): both directions. AtMost(a,n): occurrence cap.
            let violates = self.problem.constraints.iter().any(|c| match c {
                Constraint::Before { a, b } => {
                    *b == cap.name && !steps.iter().any(|s| s.capability == *a)
                }
                Constraint::After { a, b } => {
                    *a == cap.name && !steps.iter().any(|s| s.capability == *b)
                }
                Constraint::Excludes { a, b } => {
                    (*a == cap.name && steps.iter().any(|s| s.capability == *b))
                        || (*b == cap.name && steps.iter().any(|s| s.capability == *a))
                }
                Constraint::AtMost { a, n } => {
                    *a == cap.name
                        && steps.iter().filter(|s| s.capability == *a).count()
                            >= usize::from(*n)
                }
                _ => false,
            });
            if violates {
                self.pruned += 1;
                continue;
            }
            let bindings = state.join(&cap.pre, MAX_BINDINGS_PER_STEP);
            for binding in bindings {
                let params: Vec<u32> = (0..cap.params)
                    .map(|i| binding[usize::from(i)].unwrap_or(u32::MAX))
                    .collect();
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
                self.dfs(state, steps, next_cost)?;
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

/// Re-verify an unsat certificate without searching: parse the rendered core
/// back into constraints is the caller's job; this variant takes them
/// directly. Returns true iff the core alone still empties a mandatory
/// capability — i.e. the certificate checks.
#[must_use]
pub fn verify_core(problem: &SequenceProblem, core: &[Constraint]) -> bool {
    let mandatory = mandatory_set(problem);
    empties_mandatory(problem, core, &mandatory).is_some()
}

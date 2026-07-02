//! P2 — the fleet overlap curve: marginal deliberation cost as a function of
//! fleet novelty.
//!
//! The trillion-agent thesis predicts that a fleet's reasoning cost grows
//! with its *novelty*, not its *size*: when N agents deliberate over K
//! distinct problem shapes, content-addressed memoization plus shared unsat
//! cores should make the marginal cost of the Nth agent collapse as K/N
//! falls. That is a falsifiable curve, measurable on one machine. This module
//! measures it.
//!
//! Each pipeline's terminal state is projected to an 8-bit status byte in the
//! agent8 lane style, so a fleet run yields a `Vec<u8>` image consumable by
//! agent8's SWAR sweep.

use serde::{Deserialize, Serialize};

use crate::dag::{HashRunner, MemoCache};
use crate::datalog::{Atom, Program, Term};
use crate::sequence::{Capability, Constraint, SequenceProblem};
use crate::solver8::{CoreCache, Solver8};
use crate::verify::admit;
use crate::{Dag, Refusal};

/// Status-byte lanes (agent8 mapping).
pub mod lane {
    /// Saturation completed.
    pub const P_SATURATED: u8 = 1 << 0;
    /// A plan was discovered.
    pub const R_PLANNED: u8 = 1 << 1;
    /// The DAG executed.
    pub const C_EXECUTED: u8 = 1 << 2;
    /// Halted by a refusal (any kind).
    pub const H_HALTED: u8 = 1 << 3;
    /// The refusal was a *certified* unsat proof.
    pub const U_UNSAT_CERTIFIED: u8 = 1 << 4;
    /// A search/saturation budget was exhausted.
    pub const B_BUDGET: u8 = 1 << 5;
    /// Structural/input error.
    pub const E_ERROR: u8 = 1 << 6;
    /// Admitted by the verifier.
    pub const A_ADMITTED: u8 = 1 << 7;
}

/// One fleet run's measured outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetReport {
    /// Pipelines run.
    pub n: usize,
    /// Distinct templates drawn from.
    pub k: usize,
    /// Wall time for the whole fleet, nanoseconds.
    pub elapsed_ns: u128,
    /// DAG nodes actually computed (cold).
    pub executed_nodes: usize,
    /// DAG nodes served from the shared memo cache.
    pub replayed_nodes: usize,
    /// Solver search nodes across the fleet.
    pub solver_nodes: u64,
    /// Unsat proofs replayed from the shared core cache.
    pub core_hits: u64,
    /// Pipelines admitted end-to-end.
    pub admitted: usize,
    /// Pipelines refused (any refusal kind).
    pub refused: usize,
    /// The fleet image: one status byte per pipeline, agent8 lanes.
    pub bytes: Vec<u8>,
}

impl FleetReport {
    /// Deterministic work proxy: cold DAG nodes + solver nodes. Wall time is
    /// also reported, but this proxy is machine-independent.
    #[must_use]
    pub fn work(&self) -> u64 {
        self.executed_nodes as u64 + self.solver_nodes
    }
    /// Work per pipeline — the marginal-cost quantity the curve tracks.
    #[must_use]
    pub fn work_per_pipeline(&self) -> f64 {
        if self.n == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        {
            self.work() as f64 / self.n as f64
        }
    }
}

/// Build template `t`'s domain: a chain of `3 + (t % 6)` stages with
/// template-distinct predicate names. Templates where `t % 4 == 3` carry a
/// conflicting constraint triple — certified-unsat members that exercise the
/// shared core cache.
#[must_use]
pub fn template(t: usize) -> (Program, Vec<Capability>, Vec<Atom>, Vec<Constraint>) {
    let mut p = Program::new();
    let len = 3 + (t % 6); // 3..=8 stages, horizon cap respected
    let preds: Vec<_> =
        (0..=len).map(|i| p.intern(&format!("t{t}_stage{i}"))).collect();
    let obj = p.intern(&format!("t{t}_obj"));
    p.add_fact(preds[0], &[obj]).expect("fact");
    let v0 = Term::Var(0);
    let caps: Vec<Capability> = (0..len)
        .map(|i| Capability {
            name: format!("t{t}_step{i}"),
            params: 1,
            pre: vec![Atom::new(preds[i], vec![v0])],
            add: vec![Atom::new(preds[i + 1], vec![v0])],
            del: vec![],
            cost: 1,
        })
        .collect();
    let goal = vec![Atom::new(preds[len], vec![Term::Const(obj)])];
    let constraints = if t % 4 == 3 {
        // The final step must land impossibly early while its predecessor is
        // forced late — a certified dead end.
        vec![
            Constraint::NotLater { a: format!("t{t}_step{}", len - 1), k: 1 },
            Constraint::NotEarlier { a: format!("t{t}_step{}", len - 2), k: 2 },
            Constraint::Before {
                a: format!("t{t}_step{}", len - 2),
                b: format!("t{t}_step{}", len - 1),
            },
        ]
    } else {
        Vec::new()
    };
    (p, caps, goal, constraints)
}

/// Run a fleet of `n` pipelines drawn round-robin from `k` templates, with a
/// shared memo cache and a shared unsat-core cache.
///
/// # Panics
/// Panics only if a template is internally malformed (a bug, not an input).
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn run_fleet(
    n: usize,
    k: usize,
    memo: &mut MemoCache,
    cores: &mut CoreCache,
) -> FleetReport {
    let start = std::time::Instant::now();
    let mut report = FleetReport {
        n,
        k,
        elapsed_ns: 0,
        executed_nodes: 0,
        replayed_nodes: 0,
        solver_nodes: 0,
        core_hits: 0,
        admitted: 0,
        refused: 0,
        bytes: Vec::with_capacity(n),
    };
    let hits_before_all = cores.hits();
    for i in 0..n {
        let t = i % k.max(1);
        let (mut program, caps, goal, constraints) = template(t);
        let mut byte = 0u8;
        let outcome: Result<(), Refusal> = (|| {
            program.saturate()?;
            byte |= lane::P_SATURATED;
            let problem = SequenceProblem::with_constraints(
                &program,
                caps,
                goal,
                8,
                constraints,
            )?;
            let plan = Solver8.solve_cached(&problem, cores)?;
            byte |= lane::R_PLANNED;
            report.solver_nodes += plan.receipt.nodes_explored;
            let dag = Dag::from_plan(&plan, &problem);
            let dag_receipt = dag.execute(&mut HashRunner, memo)?;
            byte |= lane::C_EXECUTED;
            report.replayed_nodes += dag_receipt.replayed_count;
            report.executed_nodes +=
                dag_receipt.node_receipts.len() - dag_receipt.replayed_count;
            let verdict = admit(&mut program, &problem, &plan, &dag, &dag_receipt);
            if !verdict.ok {
                return Err(Refusal::VerificationFailed { failed: verdict.failed() });
            }
            byte |= lane::A_ADMITTED;
            Ok(())
        })();
        match outcome {
            Ok(()) => report.admitted += 1,
            Err(refusal) => {
                report.refused += 1;
                byte |= lane::H_HALTED;
                byte |= match refusal {
                    Refusal::UnsatProof { .. } => lane::U_UNSAT_CERTIFIED,
                    Refusal::BudgetExceeded { .. } | Refusal::TupleCapExceeded { .. } => {
                        lane::B_BUDGET
                    }
                    _ => lane::E_ERROR,
                };
            }
        }
        report.bytes.push(byte);
    }
    report.core_hits = cores.hits() - hits_before_all;
    report.elapsed_ns = start.elapsed().as_nanos();
    report
}

/// Sweep the overlap axis: fixed `n`, K/N descending. Fresh caches per point
/// (each point measures one fleet's internal sharing, not cross-point leakage).
#[must_use]
pub fn overlap_curve(n: usize, ks: &[usize]) -> Vec<FleetReport> {
    ks.iter()
        .map(|&k| {
            let mut memo = MemoCache::new();
            let mut cores = CoreCache::new();
            run_fleet(n, k, &mut memo, &mut cores)
        })
        .collect()
}

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

use chatman_common::provenance::{content_address, fold_event, genesis_seed};
use serde::{Deserialize, Serialize};

use crate::cell_supervise::splitmix64;
use crate::dag::{HashRunner, MemoCache};
use crate::datalog::{Atom, Program, Term};
use crate::sequence::{plan_hash_of, Capability, Constraint, SequenceProblem};
use crate::solver8::{rederive_unsat_certificate, CoreCache, Solver8};
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

    // ── Supervision combo lanes ─────────────────────────────────────────
    // All 8 bits are taken; supervision claims provably-unreachable
    // COMBINATIONS instead (exhaustive-legality test in cell_supervise):
    // the base run never co-sets H with A, so these composites are free.

    /// Halted at least once, then admitted: recovered under supervision.
    pub const S_RECOVERED: u8 = H_HALTED | A_ADMITTED;
    /// Halted on budget and parked (knhk semantics: a budget halt IS the
    /// receipted park action). Discriminate true parks from plain budget
    /// refusals via the roll-up's `parked` count, not the byte.
    pub const S_PARKED: u8 = H_HALTED | B_BUDGET;
    /// U and E co-set is impossible in the base run: claimed as the
    /// geometry-gap marker.
    pub const S_GEOMETRY_GAP: u8 = H_HALTED | U_UNSAT_CERTIFIED | E_ERROR;
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
    /// Pipelines that lost their cache to injected node faults this run.
    #[serde(default)]
    pub faults_injected: usize,
    /// Solver search nodes attributable to fault-forced re-solves.
    #[serde(default)]
    pub fault_resolve_nodes: u64,
    /// Faulted pipelines recovered by verified replay (no search).
    #[serde(default)]
    pub recovered_by_replay: usize,
    /// Cached artifacts that FAILED independent verification (poison detected)
    /// and were discarded + re-solved.
    #[serde(default)]
    pub replay_rejected: usize,
    /// Declared replay-verification work, in solver-node-equivalent units
    /// (cost model in the v2 receipt).
    #[serde(default)]
    pub replay_verify_cost: u64,
    /// Fold of every replay-verified plan_hash / core certificate over the
    /// genesis of "praxis-synthesis/replay-verify/v1", in pipeline order —
    /// the recovery's own verification hash. Empty under `BlindResolve`.
    #[serde(default)]
    pub replay_verification_root: String,
}

/// How a faulted pipeline recovers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryMode {
    /// v1 semantics (the receipted refutation's baseline): evict + re-solve.
    BlindResolve,
    /// Fetch the cached artifact and INDEPENDENTLY verify it against the
    /// freshly rebuilt problem; re-solve only on cache miss or verify failure.
    VerifiedReplay,
}

/// Domain seed for the replay-verification fold.
const REPLAY_VERIFY_DOMAIN: &str = "praxis-synthesis/replay-verify/v1";

/// Seed-deterministic node-fault script: a faulted pipeline loses its cached
/// plan and its memoized DAG outputs for the template and re-solves for real.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeFaults {
    /// Seed for the fault lottery.
    pub seed: u64,
    /// Probability (per mille) that a pipeline suffers a node fault.
    pub fault_per_mille: u16,
}

impl FleetReport {
    /// Deterministic work proxy: cold DAG nodes + solver nodes. Wall time is
    /// also reported, but this proxy is machine-independent.
    #[must_use]
    pub fn work(&self) -> u64 {
        self.executed_nodes as u64 + self.solver_nodes
    }
    /// v2 work proxy: solver search nodes plus declared replay-verification
    /// units (cost model stated in the v2 receipt). Deterministic.
    #[must_use]
    pub fn work_v2(&self) -> u64 {
        self.solver_nodes + self.replay_verify_cost
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

/// One pipeline's measured outcome, folded into the fleet report by callers.
struct PipelineOutcome {
    byte: u8,
    solver_nodes: u64,
    executed_nodes: usize,
    replayed_nodes: usize,
    admitted: bool,
    recovered_by_replay: bool,
    replay_rejected: bool,
    replay_verify_cost: u64,
    /// Hash folded into the fleet's replay-verification root on recovery.
    replay_fold: Option<String>,
}

/// How this pipeline treats the shared solve cache.
#[derive(Clone, Copy)]
enum FaultHandling {
    /// Not faulted: ordinary cached solve.
    None,
    /// Faulted, v1 semantics: evict, then genuinely re-solve.
    Evict,
    /// Faulted, verified replay: independently verify the cached artifact
    /// against the freshly rebuilt problem; re-solve on miss or poison.
    VerifiedReplay,
}

/// Run one pipeline over template `t` under the given fault handling.
fn run_pipeline(
    t: usize,
    memo: &mut MemoCache,
    cores: &mut CoreCache,
    fault: FaultHandling,
) -> PipelineOutcome {
    let (mut program, caps, goal, constraints) = template(t);
    let mut byte = 0u8;
    let mut out = PipelineOutcome {
        byte: 0,
        solver_nodes: 0,
        executed_nodes: 0,
        replayed_nodes: 0,
        admitted: false,
        recovered_by_replay: false,
        replay_rejected: false,
        replay_verify_cost: 0,
        replay_fold: None,
    };
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
        let plan = match fault {
            FaultHandling::None => Solver8.solve_cached(&problem, cores)?,
            FaultHandling::Evict => {
                cores.evict(problem.problem_hash());
                Solver8.solve_cached(&problem, cores)?
            }
            FaultHandling::VerifiedReplay => {
                // The problem hash is RECOMPUTED from the rebuilt problem,
                // never read from the cache. The cached artifact is never
                // trusted: all four checks run, no short-circuit.
                let h = problem.problem_hash().to_string();
                if let Some(cached) = cores.cached_plan(&h).cloned() {
                    // Recompute the plan's cost from the problem's own
                    // capability declarations — a forged cost field is
                    // poison like any other forged byte.
                    let recomputed_cost: Option<u32> = cached
                        .steps
                        .iter()
                        .map(|s| {
                            problem
                                .caps
                                .iter()
                                .find(|c| c.name == s.capability)
                                .map(|c| c.cost)
                        })
                        .sum();
                    let verified = cached.receipt.problem_hash == problem.problem_hash()
                        && plan_hash_of(&cached.steps) == cached.receipt.plan_hash
                        && recomputed_cost == Some(cached.cost)
                        && cached.steps.len() <= problem.horizon()
                        && problem.plan_respects_constraints(&cached)
                        && problem.replay_reaches_goal(&cached);
                    if verified {
                        out.recovered_by_replay = true;
                        // Cost model: one unit per step re-simulated, one per
                        // step x constraint checked by
                        // plan_respects_constraints, one for the plan-hash
                        // recomputation, one for the problem-hash equality,
                        // one for the cost recomputation.
                        out.replay_verify_cost += cached.steps.len() as u64
                            + (cached.steps.len() * problem.constraints.len()) as u64
                            + 3;
                        out.replay_fold = Some(cached.receipt.plan_hash.clone());
                        let mut plan = cached;
                        plan.receipt.nodes_explored = 0;
                        plan.receipt.pruned = 0;
                        plan.receipt.replayed = true;
                        plan
                    } else {
                        // Poison detected: discard, re-solve for real.
                        out.replay_rejected = true;
                        cores.evict(&h);
                        Solver8.solve_cached(&problem, cores)?
                    }
                } else if cores.cached_core(&h).is_some() {
                    // The cached certificate BODY is never served: the
                    // detail and core are re-derived from the problem by
                    // propagation, so a poisoned certificate text cannot
                    // survive replay (only the unsat FACT is cached).
                    if let Some((detail, core)) =
                        rederive_unsat_certificate(&problem)
                    {
                        out.recovered_by_replay = true;
                        out.replay_verify_cost += problem.constraints.len() as u64
                            + core.len() as u64
                            + 1;
                        out.replay_fold = Some(content_address(
                            core.join("\n").as_bytes(),
                        ));
                        return Err(Refusal::UnsatProof {
                            detail,
                            core,
                            replayed: true,
                        });
                    }
                    // Poison detected: the cached certificate no longer
                    // re-certifies. Discard, re-solve for real.
                    out.replay_rejected = true;
                    cores.evict(&h);
                    Solver8.solve_cached(&problem, cores)?
                } else {
                    // Genuine cache miss (novel template): only novelty
                    // re-solves.
                    Solver8.solve_cached(&problem, cores)?
                }
            }
        };
        byte |= lane::R_PLANNED;
        out.solver_nodes += plan.receipt.nodes_explored;
        let dag = Dag::from_plan(&plan, &problem);
        let dag_receipt = dag.execute(&mut HashRunner, memo)?;
        byte |= lane::C_EXECUTED;
        out.replayed_nodes += dag_receipt.replayed_count;
        out.executed_nodes +=
            dag_receipt.node_receipts.len() - dag_receipt.replayed_count;
        let verdict = admit(&mut program, &problem, &plan, &dag, &dag_receipt);
        if !verdict.ok {
            return Err(Refusal::VerificationFailed { failed: verdict.failed() });
        }
        byte |= lane::A_ADMITTED;
        Ok(())
    })();
    match outcome {
        Ok(()) => out.admitted = true,
        Err(refusal) => {
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
    out.byte = byte;
    out
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
    run_fleet_faulted(n, k, NodeFaults { seed: 0, fault_per_mille: 0 }, memo, cores)
}

/// [`run_fleet`] with a seed-deterministic node-fault lottery: a faulted
/// pipeline loses its cached plan/core for the template and its memoized DAG
/// outputs, so it re-solves for real and re-executes cold — the "retries are
/// novelty-bound" mechanism under test. At `fault_per_mille = 0` this is
/// byte-identical to `run_fleet` by construction.
///
/// # Panics
/// Panics only if a template is internally malformed (a bug, not an input).
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn run_fleet_faulted(
    n: usize,
    k: usize,
    faults: NodeFaults,
    memo: &mut MemoCache,
    cores: &mut CoreCache,
) -> FleetReport {
    run_fleet_faulted_recovering(n, k, faults, RecoveryMode::BlindResolve, memo, cores)
}

/// [`run_fleet_faulted`] with an explicit [`RecoveryMode`]: under
/// [`RecoveryMode::VerifiedReplay`] a faulted pipeline rebuilds its problem
/// from scratch, fetches the cached artifact, and INDEPENDENTLY verifies it
/// (hash equality, recomputed plan hash, constraint check, full O(plan)
/// re-simulation) — re-solving only on cache miss or verify failure. Under
/// [`RecoveryMode::BlindResolve`] this is byte-identical to v1. The faulted
/// pipeline's memoized DAG is still lost either way: its DAG runs against a
/// scratch memo, merged back, cold nodes counted honestly.
///
/// # Panics
/// Panics only if a template is internally malformed (a bug, not an input).
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn run_fleet_faulted_recovering(
    n: usize,
    k: usize,
    faults: NodeFaults,
    mode: RecoveryMode,
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
        faults_injected: 0,
        fault_resolve_nodes: 0,
        recovered_by_replay: 0,
        replay_rejected: 0,
        replay_verify_cost: 0,
        replay_verification_root: String::new(),
    };
    let mut replay_root = match mode {
        RecoveryMode::BlindResolve => String::new(),
        RecoveryMode::VerifiedReplay => genesis_seed(REPLAY_VERIFY_DOMAIN),
    };
    let hits_before_all = cores.hits();
    for i in 0..n {
        let t = i % k.max(1);
        let faulted = faults.fault_per_mille > 0
            && splitmix64(faults.seed ^ i as u64) % 1000
                < u64::from(faults.fault_per_mille);
        let out = if faulted {
            report.faults_injected += 1;
            // The faulted pipeline's DAG runs against a fresh scratch memo
            // (cold nodes counted honestly), merged back afterward: the
            // retry repopulates the shared cache.
            let mut scratch = MemoCache::new();
            let handling = match mode {
                RecoveryMode::BlindResolve => FaultHandling::Evict,
                RecoveryMode::VerifiedReplay => FaultHandling::VerifiedReplay,
            };
            let out = run_pipeline(t, &mut scratch, cores, handling);
            for (key, payload) in scratch.iter_raw() {
                memo.insert_raw(key.clone(), payload.clone());
            }
            report.fault_resolve_nodes += out.solver_nodes;
            if out.recovered_by_replay {
                report.recovered_by_replay += 1;
            }
            if out.replay_rejected {
                report.replay_rejected += 1;
            }
            report.replay_verify_cost += out.replay_verify_cost;
            if let Some(fold) = &out.replay_fold {
                replay_root = fold_event(&replay_root, fold.as_bytes());
            }
            out
        } else {
            run_pipeline(t, memo, cores, FaultHandling::None)
        };
        report.solver_nodes += out.solver_nodes;
        report.executed_nodes += out.executed_nodes;
        report.replayed_nodes += out.replayed_nodes;
        if out.admitted {
            report.admitted += 1;
        } else {
            report.refused += 1;
        }
        report.bytes.push(out.byte);
    }
    report.core_hits = cores.hits() - hits_before_all;
    report.replay_verification_root = replay_root;
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

/// [`overlap_curve`] under the node-fault lottery: fresh caches per point,
/// the same fault script applied at every K.
#[must_use]
pub fn overlap_curve_faulted(
    n: usize,
    ks: &[usize],
    faults: NodeFaults,
) -> Vec<FleetReport> {
    ks.iter()
        .map(|&k| {
            let mut memo = MemoCache::new();
            let mut cores = CoreCache::new();
            run_fleet_faulted(n, k, faults, &mut memo, &mut cores)
        })
        .collect()
}

/// [`overlap_curve_faulted`] with an explicit [`RecoveryMode`]: fresh caches
/// per point, the same fault script applied at every K.
#[must_use]
pub fn overlap_curve_recovering(
    n: usize,
    ks: &[usize],
    faults: NodeFaults,
    mode: RecoveryMode,
) -> Vec<FleetReport> {
    ks.iter()
        .map(|&k| {
            let mut memo = MemoCache::new();
            let mut cores = CoreCache::new();
            run_fleet_faulted_recovering(n, k, faults, mode, &mut memo, &mut cores)
        })
        .collect()
}

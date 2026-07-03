//! Supervised execution: the classify→actuate loop closed, tested end to
//! end on the lawobject plan with a deterministic fault injector.

mod common;

use std::collections::BTreeMap;

use common::lawobject_domain;
use praxis_synthesis::budget::Ticks;
use praxis_synthesis::dag::{
    DagNode, Disposition, FallibleRunner, NodeCrash, RunOutcome,
};
use praxis_synthesis::geometry::{FailureClass, FailureGeometry};
use praxis_synthesis::park::{ParkManager, ReAdmission};
use praxis_synthesis::supervise::{RestartPolicy, SupervisionTopology};
use praxis_synthesis::{
    BoundedCsp, Dag, HashRunner, MemoCache, NodeRunner, SequenceProblem, Solver,
};

/// Deterministic injector: crash capability `cap` on attempts `< fail_times`
/// with the given crash factory. Everything else delegates to HashRunner.
struct FaultRunner<F: Fn(u8) -> NodeCrash> {
    cap: String,
    fail_times: u8,
    crash: F,
    injected: u32,
}

impl<F: Fn(u8) -> NodeCrash> FallibleRunner for FaultRunner<F> {
    fn try_run(
        &mut self,
        node: &DagNode,
        inputs: &[Vec<u8>],
        attempt: u8,
    ) -> Result<(Vec<u8>, Ticks), NodeCrash> {
        if node.action.capability == self.cap && attempt < self.fail_times {
            self.injected += 1;
            return Err((self.crash)(attempt));
        }
        Ok((HashRunner.run(node, inputs), Ticks(1)))
    }
}

#[allow(clippy::type_complexity)]
fn setup() -> (Dag, SupervisionTopology, FailureGeometry, SequenceProblem) {
    let (mut p, caps, goal) = lawobject_domain();
    p.saturate().expect("saturation");
    let problem = SequenceProblem::new(&p, caps, goal, 6, Vec::new()).expect("problem");
    let plan = BoundedCsp.solve(&problem).expect("solvable");
    let topo = SupervisionTopology::derive(&plan, &problem, RestartPolicy::default_policy())
        .expect("topology");
    let geometry = FailureGeometry::derive(&topo, &plan, &problem);
    let dag = Dag::from_plan(&plan, &problem);
    (dag, topo, geometry, problem)
}

#[test]
fn no_crash_supervised_equals_plain_execution() {
    let (dag, topo, geometry, _) = setup();
    let plain = dag.execute(&mut HashRunner, &mut MemoCache::new()).expect("plain");
    let supervised = dag
        .execute_supervised(
            &topo,
            &geometry,
            &mut HashRunner,
            &mut MemoCache::new(),
            &mut ParkManager::new(),
            None,
            0,
        )
        .expect("supervised");
    assert_eq!(supervised.root_hash, plain.root_hash, "identical artifact");
    assert_eq!(supervised.outcome, RunOutcome::Completed);
    assert!(supervised.crash_receipts.is_empty());
    assert!(supervised.geometry_conformance);
    assert_eq!(supervised.restarts_total, 0);
}

#[test]
fn transient_crash_restarts_into_the_named_branch_and_completes_identically() {
    let (dag, topo, geometry, _) = setup();
    let clean = dag.execute(&mut HashRunner, &mut MemoCache::new()).expect("clean");
    let mut runner = FaultRunner {
        cap: "judge".into(),
        fail_times: 2,
        crash: |_| NodeCrash::Io { detail: "blip".into() },
        injected: 0,
    };
    let r = dag
        .execute_supervised(
            &topo,
            &geometry,
            &mut runner,
            &mut MemoCache::new(),
            &mut ParkManager::new(),
            None,
            0,
        )
        .expect("supervised");
    assert_eq!(runner.injected, 2, "two injected crashes");
    assert_eq!(r.crash_receipts.len(), 2);
    assert!(r.crash_receipts.iter().all(|c| c.matched));
    // First crash: TransientFault; second identical crash: Stall
    // (no progress) — both named, both restart.
    assert_eq!(r.crash_receipts[0].class, FailureClass::TransientFault);
    assert_eq!(r.crash_receipts[1].class, FailureClass::Stall);
    assert_eq!(r.outcome, RunOutcome::Completed);
    assert!(r.geometry_conformance);
    assert_eq!(r.restarts_total, 2);
    // The recovered artifact is byte-identical to the crash-free one.
    assert_eq!(r.root_hash, clean.root_hash, "recovery changes nothing");
    // The crash chain is anchored to the geometry.
    assert!(!r.crash_receipts[0].chain.is_empty());
}

#[test]
fn intensity_exhaustion_is_lawful_surrender_with_dependent_skips() {
    let (dag, topo, geometry, _) = setup();
    let mut runner = FaultRunner {
        cap: "judge".into(),
        fail_times: u8::MAX, // never recovers
        crash: |_| NodeCrash::Io { detail: "forever".into() },
        injected: 0,
    };
    let r = dag
        .execute_supervised(
            &topo,
            &geometry,
            &mut runner,
            &mut MemoCache::new(),
            &mut ParkManager::new(),
            None,
            0,
        )
        .expect("supervised run is Ok — surrender is a receipt, not an error");
    let RunOutcome::GaveUp { node_id } = &r.outcome else {
        panic!("expected GaveUp, got {:?}", r.outcome);
    };
    assert_eq!(r.dispositions.get(node_id), Some(&Disposition::GaveUp));
    // Downstream of judge (admit, receipt) is skipped with attribution.
    let skipped: Vec<_> = r
        .dispositions
        .values()
        .filter(|d| matches!(d, Disposition::SkippedBy { .. }))
        .collect();
    assert_eq!(skipped.len(), 2, "admit and receipt never ran");
    // Upstream completed: supply-evidence + clear-obligations.
    assert_eq!(r.node_receipts.len(), 2);
    // Restarts capped at the policy's intensity (3 = default).
    assert_eq!(r.restarts_total, u32::from(topo.policy.max_restarts));
}

#[test]
fn budget_breach_parks_and_readmission_completes_the_run_later() {
    let (dag, topo, geometry, _) = setup();
    let mut parks = ParkManager::new();
    let mut cache = MemoCache::new();
    // Run 0: judge is over budget → parked (AfterRuns(1)); dependents skip.
    let mut runner = FaultRunner {
        cap: "judge".into(),
        fail_times: u8::MAX,
        crash: |_| NodeCrash::OverBudget { ticks: 4 }, // time-tier breach
        injected: 0,
    };
    let r0 = dag
        .execute_supervised(&topo, &geometry, &mut runner, &mut cache, &mut parks, None, 0)
        .expect("run 0");
    assert!(r0
        .dispositions
        .values()
        .any(|d| matches!(d, Disposition::Parked { .. })));
    assert_eq!(parks.parked_count(), 1);
    let parked_entry = parks.iter().next().expect("entry");
    assert_eq!(parked_entry.readmission, ReAdmission::AfterRuns(1));
    assert_eq!(r0.node_receipts.len(), 2, "upstream still completed");

    // Run 1: the park re-admits at the boundary; judge now healthy.
    let r1 = dag
        .execute_supervised(
            &topo,
            &geometry,
            &mut HashRunner,
            &mut cache,
            &mut parks,
            None,
            1,
        )
        .expect("run 1");
    assert_eq!(parks.parked_count(), 0, "re-admitted");
    assert_eq!(r1.outcome, RunOutcome::Completed);
    // Upstream replays from memo; only judge + dependents compute cold.
    assert_eq!(r1.replayed_count, 2);
    let clean = dag.execute(&mut HashRunner, &mut MemoCache::new()).expect("clean");
    assert_eq!(r1.root_hash, clean.root_hash, "the healed run matches crash-free");
}

#[test]
fn authority_vacuum_refuses_with_the_fact_named() {
    // Release-style domain: push's preconditions have no producers.
    use praxis_synthesis::{Atom, Capability, Program, Term};
    let mut p = Program::new();
    let quiescent = p.intern("quiescent");
    let authorization = p.intern("authorization");
    let pushed = p.intern("pushed");
    let repo = p.intern("praxis");
    p.add_fact(quiescent, &[repo]).expect("fact");
    p.add_fact(authorization, &[repo]).expect("fact");
    p.saturate().expect("saturation");
    let v0 = Term::Var(0);
    let caps = vec![Capability {
        name: "push".into(),
        params: 1,
        pre: vec![Atom::new(quiescent, vec![v0]), Atom::new(authorization, vec![v0])],
        add: vec![Atom::new(pushed, vec![v0])],
        del: vec![],
        cost: 1,
    }];
    let goal = vec![Atom::new(pushed, vec![Term::Const(repo)])];
    let problem = SequenceProblem::new(&p, caps, goal, 2, Vec::new()).expect("problem");
    let plan = BoundedCsp.solve(&problem).expect("solvable");
    let topo = SupervisionTopology::derive(&plan, &problem, RestartPolicy::default_policy())
        .expect("topology");
    let geometry = FailureGeometry::derive(&topo, &plan, &problem);
    let dag = Dag::from_plan(&plan, &problem);

    // At runtime the authorization fact is revoked → PreconditionLost.
    let mut runner = FaultRunner {
        cap: "push".into(),
        fail_times: u8::MAX,
        crash: |_| NodeCrash::PreconditionLost { fact: "authorization".into() },
        injected: 0,
    };
    let r = dag
        .execute_supervised(
            &topo,
            &geometry,
            &mut runner,
            &mut MemoCache::new(),
            &mut ParkManager::new(),
            None,
            0,
        )
        .expect("run is Ok — refusal is a lawful outcome");
    let RunOutcome::Refused { core, .. } = &r.outcome else {
        panic!("expected Refused, got {:?}", r.outcome);
    };
    assert!(
        core.iter().any(|c| c.contains("authorization") || c.contains("quiescent")),
        "the certificate names the lost fact: {core:?}"
    );
    assert_eq!(r.crash_receipts.len(), 1, "one crash, zero futile retries");
    assert_eq!(r.crash_receipts[0].class, FailureClass::AuthorityVacuum);
}

#[test]
fn geometry_gap_is_receipted_and_the_run_still_completes() {
    let (dag, topo, geometry, _) = setup();
    // PreconditionLost on lawobject nodes has no derived branch (all facts
    // have producers) → the honest gap; default restart still heals it.
    let mut runner = FaultRunner {
        cap: "admit".into(),
        fail_times: 1,
        crash: |_| NodeCrash::PreconditionLost { fact: "validated".into() },
        injected: 0,
    };
    let r = dag
        .execute_supervised(
            &topo,
            &geometry,
            &mut runner,
            &mut MemoCache::new(),
            &mut ParkManager::new(),
            None,
            0,
        )
        .expect("supervised");
    assert_eq!(r.outcome, RunOutcome::Completed, "gap ≠ failure");
    assert!(!r.geometry_conformance, "the gap is REPORTED");
    let gap = &r.crash_receipts[0];
    assert_eq!(gap.class, FailureClass::GeometryGap);
    assert!(!gap.matched);
}

#[test]
fn dispositions_cover_every_node_exactly_once() {
    let (dag, topo, geometry, _) = setup();
    let mut runner = FaultRunner {
        cap: "judge".into(),
        fail_times: u8::MAX,
        crash: |_| NodeCrash::Io { detail: "forever".into() },
        injected: 0,
    };
    let r = dag
        .execute_supervised(
            &topo,
            &geometry,
            &mut runner,
            &mut MemoCache::new(),
            &mut ParkManager::new(),
            None,
            0,
        )
        .expect("supervised");
    let ids: BTreeMap<&String, &Disposition> = r.dispositions.iter().collect();
    assert_eq!(ids.len(), dag.nodes.len(), "total accounting: no silent nodes");
}

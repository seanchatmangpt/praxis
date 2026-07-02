//! Supervision-topology tests: derivation determinism, earned strategies,
//! cohort semantics, and the intensity doctrine.

mod common;

use common::lawobject_domain;
use praxis_synthesis::supervise::{
    RestartPolicy, Strategy, SupervisionTopology, MAX_RESTART_INTENSITY,
};
use praxis_synthesis::{BoundedCsp, Refusal, SequenceProblem, Solver};

fn solved() -> (praxis_synthesis::SequencePlan, SequenceProblem) {
    let (mut p, caps, goal) = lawobject_domain();
    p.saturate().expect("saturation");
    let problem = SequenceProblem::new(&p, caps, goal, 6, Vec::new()).expect("problem");
    let plan = BoundedCsp.solve(&problem).expect("solvable");
    (plan, problem)
}

#[test]
fn topology_derivation_is_deterministic() {
    let (plan, problem) = solved();
    let a = SupervisionTopology::derive(&plan, &problem, RestartPolicy::default_policy())
        .expect("derive");
    let b = SupervisionTopology::derive(&plan, &problem, RestartPolicy::default_policy())
        .expect("derive");
    assert_eq!(a.topology_hash, b.topology_hash, "same plan → same topology");
    assert_eq!(a.stages, b.stages);
}

#[test]
fn the_lawobject_chain_earns_rest_for_one_except_its_leaf() {
    // 5-step pure chain: every stage but the last has a dependent, so it
    // supervises RestForOne; the terminal stage has no consumers → OneForOne.
    let (plan, problem) = solved();
    let topo = SupervisionTopology::derive(&plan, &problem, RestartPolicy::default_policy())
        .expect("derive");
    assert_eq!(topo.stages.len(), 5, "one stage per dependency depth");
    assert_eq!(topo.node_count(), 5);
    let (last, body) = topo.stages.split_last().expect("stages");
    assert!(
        body.iter().all(|s| s.strategy == Strategy::RestForOne),
        "producers restart with their consumers"
    );
    assert_eq!(last.strategy, Strategy::OneForOne, "the leaf restarts alone");
}

#[test]
fn cohorts_are_the_transitive_downstream_plus_self() {
    let (plan, problem) = solved();
    let topo = SupervisionTopology::derive(&plan, &problem, RestartPolicy::default_policy())
        .expect("derive");
    // Root of the chain: its cohort is the whole plan.
    let root = &topo.stages[0].nodes[0];
    assert_eq!(topo.cohort(root).len(), 5, "root failure invalidates everything");
    assert_eq!(topo.dependents(root).len(), 4);
    // Leaf: cohort is itself alone.
    let leaf = &topo.stages[4].nodes[0];
    assert_eq!(topo.cohort(leaf), vec![leaf.clone()]);
    assert!(topo.dependents(leaf).is_empty());
    // Middle (depth 2, 'judge'): itself + 2 dependents.
    let mid = &topo.stages[2].nodes[0];
    assert_eq!(topo.cohort(mid).len(), 3);
}

#[test]
fn intensity_beyond_eight_is_refused_not_clamped() {
    let err = RestartPolicy::new(MAX_RESTART_INTENSITY + 1, 8).expect_err("must refuse");
    assert!(matches!(err, Refusal::InvalidInput { .. }));
    let msg = format!("{err}");
    assert!(msg.contains("refused, not clamped"), "{msg}");
    // The boundary itself is lawful.
    assert!(RestartPolicy::new(MAX_RESTART_INTENSITY, 8).is_ok());
}

#[test]
fn one_for_all_does_not_exist() {
    // The absence is the refusal: Strategy is a closed two-variant enum, so
    // this match is exhaustive — adding OneForAll would break compilation
    // here, forcing the doctrine conversation.
    let all = [Strategy::OneForOne, Strategy::RestForOne];
    for s in all {
        match s {
            Strategy::OneForOne | Strategy::RestForOne => {}
        }
    }
}

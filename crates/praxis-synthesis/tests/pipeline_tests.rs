//! End-to-end: facts → saturate → sequence → dag-execute → verify → receipt.

mod common;

use common::lawobject_domain;
use praxis_synthesis::{BoundedCsp, HashRunner, MemoCache, Synthesis};

#[test]
fn end_to_end_lawobject_synthesis_is_admitted() {
    let (mut p, caps, goal) = lawobject_domain();
    let receipt = Synthesis::run(
        &mut p,
        caps,
        goal,
        6,
        &BoundedCsp,
        &mut HashRunner,
        &mut MemoCache::new(),
    )
    .expect("full pipeline admits");

    assert_eq!(receipt.plan_steps, 5, "the discovered plan is the 5-step lawobject pipe");
    assert!(receipt.verdict.ok);
    assert_eq!(receipt.verdict.checks.len(), 6, "all six refinements ran");
    assert!(receipt.verdict.checks.iter().all(|c| c.ok), "{:?}", receipt.verdict);
    assert_eq!(receipt.dag.node_receipts.len(), 5);
    assert_eq!(receipt.chain.len(), 64, "one blake3 chain for the whole run");
}

#[test]
fn pipeline_receipt_is_deterministic() {
    let run = || {
        let (mut p, caps, goal) = lawobject_domain();
        let r = Synthesis::run(
            &mut p,
            caps,
            goal,
            6,
            &BoundedCsp,
            &mut HashRunner,
            &mut MemoCache::new(),
        )
        .expect("pipeline");
        (r.saturation.fixpoint_hash, r.plan_hash, r.dag.root_hash, r.chain)
    };
    assert_eq!(run(), run(), "the whole pipeline is a pure function of its input");
}

#[test]
fn warm_cache_run_is_fully_memoized_and_same_chain() {
    let (mut p1, caps1, goal1) = lawobject_domain();
    let mut cache = MemoCache::new();
    let cold = Synthesis::run(
        &mut p1, caps1, goal1, 6, &BoundedCsp, &mut HashRunner, &mut cache,
    )
    .expect("cold run");
    let (mut p2, caps2, goal2) = lawobject_domain();
    let warm = Synthesis::run(
        &mut p2, caps2, goal2, 6, &BoundedCsp, &mut HashRunner, &mut cache,
    )
    .expect("warm run");
    assert_eq!(cold.dag.replayed_count, 0);
    assert_eq!(warm.dag.replayed_count, warm.dag.node_receipts.len(), "100% replayed");
    assert_eq!(cold.chain, warm.chain, "replayed run produces the identical receipt");
}

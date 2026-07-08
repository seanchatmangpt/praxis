//! Layer 3 tests: topological execution, content-addressed reproducibility,
//! memoized replay, tamper evidence.

mod common;

use common::lawobject_domain;
use praxis_synthesis::{BoundedCsp, Dag, HashRunner, MemoCache, Refusal, SequenceProblem, Solver};

fn solved() -> (praxis_synthesis::SequencePlan, SequenceProblem) {
    let (mut p, caps, goal) = lawobject_domain();
    p.saturate().expect("saturation");
    let problem = SequenceProblem::new(&p, caps, goal, 6, Vec::new()).expect("problem");
    let plan = BoundedCsp.solve(&problem).expect("solvable");
    (plan, problem)
}

#[test]
fn dag_derives_the_causal_chain_and_executes_in_topo_order() {
    let (plan, problem) = solved();
    let dag = Dag::from_plan(&plan, &problem);
    assert_eq!(dag.nodes.len(), 5);
    // The lawobject pipe is a pure chain: each node after the first has
    // exactly one input.
    let indegrees: Vec<usize> = {
        let order = dag.topo_order().expect("acyclic");
        order.iter().map(|id| dag.nodes[id].inputs.len()).collect()
    };
    assert_eq!(
        indegrees.iter().sum::<usize>(),
        4,
        "4 data edges in a 5-node chain"
    );

    let receipt = dag
        .execute(&mut HashRunner, &mut MemoCache::new())
        .expect("executes");
    assert_eq!(receipt.node_receipts.len(), 5);
    assert_eq!(receipt.replayed_count, 0, "cold cache: nothing replayed");
}

#[test]
fn second_run_replays_entirely_from_the_memo_cache() {
    let (plan, problem) = solved();
    let dag = Dag::from_plan(&plan, &problem);
    let mut cache = MemoCache::new();
    let first = dag.execute(&mut HashRunner, &mut cache).expect("run 1");
    let second = dag.execute(&mut HashRunner, &mut cache).expect("run 2");
    assert_eq!(second.replayed_count, dag.nodes.len(), "100% memoized");
    assert_eq!(
        first.root_hash, second.root_hash,
        "content address is stable"
    );
}

#[test]
fn root_hash_is_independent_of_declaration_order() {
    // Two independent bound steps (different objects, no shared atoms) in
    // both orders must produce the same root hash.
    use praxis_synthesis::{Atom, Capability, Program, Term};
    let build = |flip: bool| {
        let mut p = Program::new();
        let raw = p.intern("raw");
        let done = p.intern("done");
        let o1 = p.intern("o1");
        let o2 = p.intern("o2");
        p.add_fact(raw, &[o1]).expect("fact");
        p.add_fact(raw, &[o2]).expect("fact");
        p.saturate().expect("saturation");
        let cap = |name: &str, obj| Capability {
            name: name.into(),
            params: 0,
            pre: vec![Atom::new(raw, vec![Term::Const(obj)])],
            add: vec![Atom::new(done, vec![Term::Const(obj)])],
            del: vec![],
            cost: 1,
        };
        let mut caps = vec![cap("work-o1", o1), cap("work-o2", o2)];
        if flip {
            caps.reverse();
        }
        let goal = vec![
            Atom::new(done, vec![Term::Const(o1)]),
            Atom::new(done, vec![Term::Const(o2)]),
        ];
        let problem = SequenceProblem::new(&p, caps, goal, 2, Vec::new()).expect("problem");
        let plan = BoundedCsp.solve(&problem).expect("solvable");
        let dag = Dag::from_plan(&plan, &problem);
        dag.execute(&mut HashRunner, &mut MemoCache::new())
            .expect("executes")
            .root_hash
    };
    assert_eq!(build(false), build(true), "root hash is order-independent");
}

#[test]
fn cycle_is_refused() {
    let (plan, problem) = solved();
    let mut dag = Dag::from_plan(&plan, &problem);
    // Introduce a back-edge from the first topo node to the last.
    let order = dag.topo_order().expect("acyclic");
    let (first, last) = (order[0].clone(), order[order.len() - 1].clone());
    dag.nodes.get_mut(&first).expect("node").inputs.push(last);
    let err = dag.topo_order().expect_err("cycle must refuse");
    assert!(matches!(err, Refusal::InvalidInput { .. }));
}

#[test]
fn tampered_receipt_breaks_chain_recompute() {
    use chatman_common::provenance::{fold_event, genesis_seed};
    let (plan, problem) = solved();
    let dag = Dag::from_plan(&plan, &problem);
    let mut receipt = dag
        .execute(&mut HashRunner, &mut MemoCache::new())
        .expect("executes");
    // Tamper with one node's recorded output hash.
    receipt.node_receipts[2].output_hash = "0".repeat(64);
    // Recompute the fold — it must no longer match the recorded final link.
    let mut chain = genesis_seed(praxis_synthesis::dag::DAG_CHAIN_DOMAIN);
    for nr in &receipt.node_receipts {
        let frame = serde_json::json!({
            "node_id": nr.node_id,
            "action_hash": nr.action_hash,
            "input_hashes": nr.input_hashes,
            "output_hash": nr.output_hash,
        })
        .to_string();
        chain = fold_event(&chain, frame.as_bytes());
    }
    let recorded = receipt.node_receipts.last().expect("nodes").chain.clone();
    assert_ne!(chain, recorded, "tamper is chain-evident");
}

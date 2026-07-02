//! P5 tests: the living ledger — incremental assertion resumes from the
//! delta, and successive fixpoint hashes chain.

use praxis_synthesis::datalog::IncrementReceipt;
use praxis_synthesis::{Atom, DlRule, Program, Refusal, Term};

fn tc_program() -> (Program, praxis_synthesis::datalog::SaturationReceipt) {
    let mut p = Program::new();
    let edge = p.intern("edge");
    let path = p.intern("path");
    let nodes: Vec<_> = (0..50).map(|i| p.intern(&format!("n{i}"))).collect();
    for w in nodes.windows(2) {
        p.add_fact(edge, &[w[0], w[1]]).expect("fact");
    }
    p.add_rule(DlRule {
        head: Atom::new(path, vec![Term::Var(0), Term::Var(1)]),
        body: vec![Atom::new(edge, vec![Term::Var(0), Term::Var(1)])],
        negative: vec![],
    })
    .expect("rule");
    p.add_rule(DlRule {
        head: Atom::new(path, vec![Term::Var(0), Term::Var(2)]),
        body: vec![
            Atom::new(path, vec![Term::Var(0), Term::Var(1)]),
            Atom::new(edge, vec![Term::Var(1), Term::Var(2)]),
        ],
        negative: vec![],
    })
    .expect("rule");
    let r = p.saturate().expect("saturation");
    (p, r)
}

#[test]
fn increment_equals_batch() {
    // Incremental: saturate a 50-node chain, then assert the bridge edge to
    // a second 50-node chain and the second chain's edges.
    let (mut inc, _) = tc_program();
    let edge = inc.dict.get("edge").expect("interned");
    let mut new_facts = Vec::new();
    let n49 = inc.dict.get("n49").expect("interned");
    let mut prev = n49;
    for i in 50..100 {
        let ni = inc.intern(&format!("n{i}"));
        new_facts.push((edge, vec![prev, ni]));
        prev = ni;
    }
    let receipt: IncrementReceipt = inc.assert_facts(&new_facts).expect("increment");
    assert_eq!(receipt.asserted, 50);
    assert!(receipt.derived > 0);
    assert_ne!(receipt.prev_fixpoint_hash, receipt.fixpoint_hash);

    // Batch: the whole 100-node chain from scratch.
    let mut batch = Program::new();
    let edge_b = batch.intern("edge");
    let path_b = batch.intern("path");
    let nodes: Vec<_> = (0..100).map(|i| batch.intern(&format!("n{i}"))).collect();
    for w in nodes.windows(2) {
        batch.add_fact(edge_b, &[w[0], w[1]]).expect("fact");
    }
    batch
        .add_rule(DlRule {
            head: Atom::new(path_b, vec![Term::Var(0), Term::Var(1)]),
            body: vec![Atom::new(edge_b, vec![Term::Var(0), Term::Var(1)])],
            negative: vec![],
        })
        .expect("rule");
    batch
        .add_rule(DlRule {
            head: Atom::new(path_b, vec![Term::Var(0), Term::Var(2)]),
            body: vec![
                Atom::new(path_b, vec![Term::Var(0), Term::Var(1)]),
                Atom::new(edge_b, vec![Term::Var(1), Term::Var(2)]),
            ],
            negative: vec![],
        })
        .expect("rule");
    let batch_receipt = batch.saturate().expect("saturation");

    // Same dict-insertion order → same ids → the *final states* must agree
    // exactly: incremental resume is extensionally equal to batch.
    assert_eq!(receipt.fixpoint_hash, batch_receipt.fixpoint_hash);
    assert_eq!(inc.len(), batch.len());
}

#[test]
fn increments_chain() {
    let (mut p, first) = tc_program();
    let chain_after_batch = p.chain().to_string();
    let edge = p.dict.get("edge").expect("interned");
    let n49 = p.dict.get("n49").expect("interned");
    let n50 = p.intern("n50");
    let r1 = p.assert_facts(&[(edge, vec![n49, n50])]).expect("inc 1");
    let n51 = p.intern("n51");
    let r2 = p.assert_facts(&[(edge, vec![n50, n51])]).expect("inc 2");
    // Ledger structure: each link folds the new fixpoint hash onto the last.
    assert_eq!(r1.prev_fixpoint_hash, first.fixpoint_hash);
    assert_eq!(r2.prev_fixpoint_hash, r1.fixpoint_hash);
    assert_ne!(chain_after_batch, r1.chain);
    assert_ne!(r1.chain, r2.chain);
    assert_eq!(p.chain(), r2.chain);
    // The chain recomputes: fold the hashes in order from the batch link.
    let recomputed = chatman_common::provenance::fold_event(
        &chatman_common::provenance::fold_event(
            &chain_after_batch,
            r1.fixpoint_hash.as_bytes(),
        ),
        r2.fixpoint_hash.as_bytes(),
    );
    assert_eq!(recomputed, r2.chain, "the ledger replays");
}

#[test]
fn empty_increment_is_a_noop_link() {
    let (mut p, first) = tc_program();
    let edge = p.dict.get("edge").expect("interned");
    let n0 = p.dict.get("n0").expect("interned");
    let n1 = p.dict.get("n1").expect("interned");
    // Re-asserting an existing fact: nothing arrives, nothing derives.
    let r = p.assert_facts(&[(edge, vec![n0, n1])]).expect("noop");
    assert_eq!(r.asserted, 0);
    assert_eq!(r.derived, 0);
    assert_eq!(r.fixpoint_hash, first.fixpoint_hash);
}

#[test]
fn increment_under_negation_is_refused_as_nonmonotonic() {
    let mut p = Program::new();
    let candidate = p.intern("candidate");
    let excluded = p.intern("excluded");
    let eligible = p.intern("eligible");
    let a = p.intern("a");
    p.add_fact(candidate, &[a]).expect("fact");
    p.add_rule(DlRule {
        head: Atom::new(eligible, vec![Term::Var(0)]),
        body: vec![Atom::new(candidate, vec![Term::Var(0)])],
        negative: vec![Atom::new(excluded, vec![Term::Var(0)])],
    })
    .expect("rule");
    p.saturate().expect("saturation");
    // Asserting excluded(a) NOW would have to retract eligible(a) — additive
    // increments cannot do that. The engine refuses rather than lies.
    let err = p.assert_facts(&[(excluded, vec![a])]).expect_err("must refuse");
    assert!(matches!(err, Refusal::InvalidInput { .. }));
    let msg = format!("{err}");
    assert!(msg.contains("nonmonotonic"), "reason names the pathology: {msg}");
}

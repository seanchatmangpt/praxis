//! Layer 1 tests: semi-naive saturation, stratified negation, receipted caps,
//! deterministic fixpoint hashes.

use praxis_synthesis::{Atom, DlRule, Program, Refusal, Term};

/// Transitive closure over a 100-node chain — the classic recursive program a
/// depth-bounded SLD engine cannot fully close, demonstrating the semi-naive
/// forward win.
#[test]
fn transitive_closure_over_100_node_chain_saturates() {
    let mut p = Program::new();
    let edge = p.intern("edge");
    let path = p.intern("path");
    let nodes: Vec<_> = (0..100).map(|i| p.intern(&format!("n{i}"))).collect();
    for w in nodes.windows(2) {
        p.add_fact(edge, &[w[0], w[1]]).expect("fact");
    }
    // path(X,Y) :- edge(X,Y).
    p.add_rule(DlRule {
        head: Atom::new(path, vec![Term::Var(0), Term::Var(1)]),
        body: vec![Atom::new(edge, vec![Term::Var(0), Term::Var(1)])],
        negative: vec![],
    })
    .expect("rule");
    // path(X,Z) :- path(X,Y), edge(Y,Z).
    p.add_rule(DlRule {
        head: Atom::new(path, vec![Term::Var(0), Term::Var(2)]),
        body: vec![
            Atom::new(path, vec![Term::Var(0), Term::Var(1)]),
            Atom::new(edge, vec![Term::Var(1), Term::Var(2)]),
        ],
        negative: vec![],
    })
    .expect("rule");

    let receipt = p.saturate().expect("saturation");
    // 100-node chain: C(100,2) = 4950 path facts.
    assert_eq!(p.count_for(path), 4950);
    assert_eq!(receipt.derived_count, 4950);
    assert!(
        receipt.iterations > 2,
        "recursive closure needs multiple rounds"
    );
    // Endpoint reachability.
    assert!(p.contains(path, &[nodes[0], nodes[99]]));
}

#[test]
fn stratified_negation_derives_complement() {
    let mut p = Program::new();
    let candidate = p.intern("candidate");
    let excluded = p.intern("excluded");
    let eligible = p.intern("eligible");
    let a = p.intern("a");
    let b = p.intern("b");
    p.add_fact(candidate, &[a]).expect("fact");
    p.add_fact(candidate, &[b]).expect("fact");
    p.add_fact(excluded, &[b]).expect("fact");
    // eligible(X) :- candidate(X), NOT excluded(X).
    p.add_rule(DlRule {
        head: Atom::new(eligible, vec![Term::Var(0)]),
        body: vec![Atom::new(candidate, vec![Term::Var(0)])],
        negative: vec![Atom::new(excluded, vec![Term::Var(0)])],
    })
    .expect("rule");

    let receipt = p.saturate().expect("saturation");
    assert!(p.contains(eligible, &[a]));
    assert!(!p.contains(eligible, &[b]));
    assert_eq!(receipt.strata, 2, "negation forces a second stratum");
}

#[test]
fn negation_cycle_is_refused_as_unstratifiable() {
    let mut p = Program::new();
    let win = p.intern("win");
    let m = p.intern("move");
    let x = p.intern("x");
    let y = p.intern("y");
    p.add_fact(m, &[x, y]).expect("fact");
    // win(X) :- move(X,Y), NOT win(Y).  — negation through its own predicate.
    p.add_rule(DlRule {
        head: Atom::new(win, vec![Term::Var(0)]),
        body: vec![Atom::new(m, vec![Term::Var(0), Term::Var(1)])],
        negative: vec![Atom::new(win, vec![Term::Var(1)])],
    })
    .expect("rule shape is valid; stratification is what fails");
    let err = p.saturate().expect_err("must refuse");
    assert!(matches!(err, Refusal::Unstratifiable { .. }), "got {err:?}");
}

#[test]
fn unsafe_rules_are_refused_at_add() {
    let mut p = Program::new();
    let q = p.intern("q");
    let r = p.intern("r");
    // Head var not bound by body.
    let err = p
        .add_rule(DlRule {
            head: Atom::new(q, vec![Term::Var(3)]),
            body: vec![Atom::new(r, vec![Term::Var(0)])],
            negative: vec![],
        })
        .expect_err("unsafe head");
    assert!(matches!(err, Refusal::InvalidInput { .. }));
    // Negated var not bound by body.
    let err = p
        .add_rule(DlRule {
            head: Atom::new(q, vec![Term::Var(0)]),
            body: vec![Atom::new(r, vec![Term::Var(0)])],
            negative: vec![Atom::new(r, vec![Term::Var(5)])],
        })
        .expect_err("unsafe negation");
    assert!(matches!(err, Refusal::InvalidInput { .. }));
}

#[test]
fn fixpoint_hash_is_deterministic_and_input_sensitive() {
    let build = |extra: bool| {
        let mut p = Program::new();
        let edge = p.intern("edge");
        let path = p.intern("path");
        let a = p.intern("a");
        let b = p.intern("b");
        let c = p.intern("c");
        p.add_fact(edge, &[a, b]).expect("fact");
        p.add_fact(edge, &[b, c]).expect("fact");
        if extra {
            let d = p.intern("d");
            p.add_fact(edge, &[c, d]).expect("fact");
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
        p.saturate().expect("saturation").fixpoint_hash
    };
    assert_eq!(build(false), build(false), "same input, same fixpoint hash");
    assert_ne!(build(false), build(true), "different input, different hash");
}

#[test]
fn saturated_program_is_closed() {
    let mut p = Program::new();
    let edge = p.intern("edge");
    let path = p.intern("path");
    let a = p.intern("a");
    let b = p.intern("b");
    p.add_fact(edge, &[a, b]).expect("fact");
    p.add_rule(DlRule {
        head: Atom::new(path, vec![Term::Var(0), Term::Var(1)]),
        body: vec![Atom::new(edge, vec![Term::Var(0), Term::Var(1)])],
        negative: vec![],
    })
    .expect("rule");
    p.saturate().expect("saturation");
    assert!(p.is_closed().expect("closure check"));
}

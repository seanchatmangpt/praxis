//! P3 — self-application: the praxis release procedure, derived.
//!
//! The pipeline that governs this repository (quiesce → test → commit → gate
//! → push) is expressed as capability declarations with the standing
//! doctrine as constraints, and synthesis derives the release plan. The
//! decisive case: when the `authorization` fact is absent, the solver does
//! not merely fail to schedule `push` — it refuses with a certificate that
//! **names the missing fact**. The Chatman equation eating its own tail:
//! the system proves why its own gated step cannot fire.

use praxis_synthesis::{
    Atom, BoundedCsp, Capability, Constraint, Program, Refusal, SequenceProblem, Solver, Solver8,
    Term,
};

/// The release domain. `authorized` controls whether the human go-ahead fact
/// is asserted.
fn release_domain(authorized: bool) -> (Program, Vec<Capability>, Vec<Atom>, Vec<Constraint>) {
    let mut p = Program::new();
    let uncommitted = p.intern("uncommitted");
    let tested = p.intern("tested");
    let committed = p.intern("committed");
    let quiescent = p.intern("quiescent");
    let authorization = p.intern("authorization");
    let gated = p.intern("gated");
    let pushed = p.intern("pushed");
    let repo = p.intern("praxis");
    p.add_fact(uncommitted, &[repo]).expect("fact");
    if authorized {
        p.add_fact(authorization, &[repo]).expect("fact");
    }
    let v0 = Term::Var(0);
    let caps = vec![
        Capability {
            name: "run-tests".into(),
            params: 1,
            pre: vec![Atom::new(uncommitted, vec![v0])],
            add: vec![Atom::new(tested, vec![v0])],
            del: vec![],
            cost: 3, // the expensive step
        },
        Capability {
            name: "commit".into(),
            params: 1,
            pre: vec![Atom::new(tested, vec![v0])],
            add: vec![Atom::new(committed, vec![v0])],
            del: vec![Atom::new(uncommitted, vec![v0])],
            cost: 1,
        },
        Capability {
            name: "quiesce".into(),
            params: 1,
            pre: vec![Atom::new(committed, vec![v0])],
            add: vec![Atom::new(quiescent, vec![v0])],
            del: vec![],
            cost: 1,
        },
        Capability {
            name: "gate-check".into(),
            params: 1,
            // The gate consumes BOTH quiescence and the human authorization
            // fact. Nothing in the domain manufactures authorization —
            // that is the entire point.
            pre: vec![
                Atom::new(quiescent, vec![v0]),
                Atom::new(authorization, vec![v0]),
            ],
            add: vec![Atom::new(gated, vec![v0])],
            del: vec![],
            cost: 1,
        },
        Capability {
            name: "push".into(),
            params: 1,
            pre: vec![Atom::new(gated, vec![v0])],
            add: vec![Atom::new(pushed, vec![v0])],
            del: vec![],
            cost: 1,
        },
    ];
    let goal = vec![Atom::new(pushed, vec![Term::Const(repo)])];
    // The standing doctrine, stated as constraints (redundant with causality
    // by design — doctrine should be enforced even if someone edits a
    // capability's preconditions later).
    let constraints = vec![
        Constraint::Before {
            a: "run-tests".into(),
            b: "commit".into(),
        },
        Constraint::Before {
            a: "commit".into(),
            b: "push".into(),
        },
        Constraint::Requires {
            a: "push".into(),
            b: "gate-check".into(),
        },
        Constraint::AtMost {
            a: "push".into(),
            n: 1,
        },
    ];
    (p, caps, goal, constraints)
}

#[test]
fn with_authorization_the_release_plan_is_derived() {
    let (mut p, caps, goal, constraints) = release_domain(true);
    p.saturate().expect("saturation");
    let problem =
        SequenceProblem::with_constraints(&p, caps, goal, 6, constraints).expect("problem");
    let plan = Solver8
        .solve(&problem)
        .expect("authorized release is derivable");
    let order: Vec<&str> = plan.steps.iter().map(|s| s.capability.as_str()).collect();
    assert_eq!(
        order,
        ["run-tests", "commit", "quiesce", "gate-check", "push"],
        "the release procedure, derived — not authored"
    );
    assert!(
        problem.replay_reaches_goal(&plan),
        "independent replay confirms"
    );
    // Differential: the brute oracle agrees.
    let brute = BoundedCsp
        .solve(&problem)
        .expect("oracle agrees it is solvable");
    assert_eq!(brute.steps, plan.steps);
}

#[test]
fn without_authorization_the_refusal_names_the_missing_fact() {
    let (mut p, caps, goal, constraints) = release_domain(false);
    p.saturate().expect("saturation");
    let problem =
        SequenceProblem::with_constraints(&p, caps, goal, 6, constraints).expect("problem");
    let err = Solver8.solve(&problem).expect_err("must refuse");
    let Refusal::UnsatProof {
        detail,
        core,
        replayed,
    } = err
    else {
        panic!("expected a certificate, got a plain refusal");
    };
    assert!(!replayed);
    // The certificate names the gated capability AND the missing fact.
    assert!(
        detail.contains("gate-check") && detail.contains("authorization"),
        "detail must name the victim and the missing fact: {detail}"
    );
    assert_eq!(
        core,
        vec!["MissingFact(authorization)".to_string()],
        "the core IS the missing fact"
    );
}

#[test]
fn the_unauthorized_refusal_is_pre_search_a_certificate_not_a_timeout() {
    // The brute oracle must exhaust search to conclude the same thing the
    // certificate states instantly — the difference between a proof and a
    // failure to find.
    let (mut p, caps, goal, constraints) = release_domain(false);
    p.saturate().expect("saturation");
    let problem =
        SequenceProblem::with_constraints(&p, caps, goal, 6, constraints).expect("problem");
    let brute = BoundedCsp.solve(&problem).expect_err("oracle also refuses");
    match brute {
        Refusal::Unsatisfiable { nodes_explored, .. } => {
            assert!(nodes_explored > 0, "the oracle had to search to fail");
        }
        other => panic!("oracle refuses without certificate: got {other:?}"),
    }
    // Solver8's refusal, by contrast, carries the fact-level proof (asserted
    // in the companion test) and required no search at all.
}

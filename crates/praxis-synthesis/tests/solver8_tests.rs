//! P1 tests: Solver8's plans agree with the brute oracle; its refusals are
//! minimal, verifiable, cacheable certificates.

mod common;

use common::lawobject_domain;
use praxis_synthesis::solver8::verify_core;
use praxis_synthesis::{
    BoundedCsp, Constraint, CoreCache, Refusal, SequenceProblem, Solver, Solver8,
};

fn saturated() -> (
    praxis_synthesis::Program,
    Vec<praxis_synthesis::Capability>,
    Vec<praxis_synthesis::Atom>,
) {
    let (mut p, caps, goal) = lawobject_domain();
    p.saturate().expect("saturation");
    (p, caps, goal)
}

// ── differential: the propagating solver against the brute oracle ──────────

#[test]
fn differential_solver8_equals_boundedcsp_without_constraints() {
    let (p, caps, goal) = saturated();
    let problem = SequenceProblem::new(&p, caps, goal, 6, Vec::new()).expect("problem");
    let brute = BoundedCsp.solve(&problem).expect("oracle solves");
    let smart = Solver8.solve(&problem).expect("solver8 solves");
    assert_eq!(brute.steps, smart.steps, "identical plans");
    assert_eq!(brute.cost, smart.cost);
    assert!(
        smart.receipt.nodes_explored <= brute.receipt.nodes_explored,
        "propagation must never search more than brute force"
    );
}

#[test]
fn differential_holds_under_satisfiable_constraints() {
    let (p, caps, goal) = saturated();
    let constraints = vec![
        Constraint::Before {
            a: "judge".into(),
            b: "receipt".into(),
        },
        Constraint::NotEarlier {
            a: "admit".into(),
            k: 3,
        },
        Constraint::Budget { max: 10 },
    ];
    let problem =
        SequenceProblem::with_constraints(&p, caps, goal, 6, constraints).expect("problem");
    let plan = Solver8.solve(&problem).expect("solvable");
    assert!(
        problem.replay_reaches_goal(&plan),
        "independent replay confirms"
    );
    // admit is step index 3 (0-based) in the 5-step chain — NotEarlier(3) holds.
    let admit_at = plan
        .steps
        .iter()
        .position(|s| s.capability == "admit")
        .expect("admit");
    assert!(admit_at >= 3);
}

// ── certificates: refusals with named culprits ──────────────────────────────

#[test]
fn conflicting_windows_produce_a_minimal_core_naming_the_culprits() {
    let (p, caps, goal) = saturated();
    // receipt is mandatory (sole producer of the goal atom). Force it early
    // and its prerequisite late: NotLater(receipt, 2) but Before-chain needs
    // 4 predecessors — plus decoy constraints that are NOT part of the
    // conflict and must be excluded from the core.
    let constraints = vec![
        Constraint::Before {
            a: "supply-evidence".into(),
            b: "clear-obligations".into(),
        }, // decoy
        Constraint::NotLater {
            a: "receipt".into(),
            k: 2,
        },
        Constraint::NotEarlier {
            a: "admit".into(),
            k: 3,
        },
        Constraint::Before {
            a: "admit".into(),
            b: "receipt".into(),
        },
        Constraint::AtMost {
            a: "judge".into(),
            n: 1,
        }, // decoy
    ];
    let problem =
        SequenceProblem::with_constraints(&p, caps, goal, 6, constraints).expect("problem");
    let err = Solver8.solve(&problem).expect_err("must refuse");
    let Refusal::UnsatProof {
        detail,
        core,
        replayed,
    } = err
    else {
        panic!("expected UnsatProof, got something else");
    };
    assert!(!replayed);
    assert!(detail.contains("receipt"), "the victim is named: {detail}");
    // The minimal conflict: receipt < 2 while admit >= 3 and admit before receipt.
    assert!(
        core.contains(&"NotLater(receipt,2)".to_string()),
        "core: {core:?}"
    );
    assert!(
        core.contains(&"NotEarlier(admit,3)".to_string()),
        "core: {core:?}"
    );
    assert!(
        core.contains(&"Before(admit,receipt)".to_string()),
        "core: {core:?}"
    );
    // Decoys excluded — the core is minimal.
    assert!(
        !core.iter().any(|c| c.contains("supply-evidence")),
        "core: {core:?}"
    );
    assert!(!core.iter().any(|c| c.contains("AtMost")), "core: {core:?}");
    assert_eq!(core.len(), 3, "exactly the conflicting triple: {core:?}");
}

#[test]
fn the_core_alone_reverifies_without_search() {
    let (p, caps, goal) = saturated();
    let core = vec![
        Constraint::NotLater {
            a: "receipt".into(),
            k: 2,
        },
        Constraint::NotEarlier {
            a: "admit".into(),
            k: 3,
        },
        Constraint::Before {
            a: "admit".into(),
            b: "receipt".into(),
        },
    ];
    let problem =
        SequenceProblem::with_constraints(&p, caps, goal, 6, core.clone()).expect("problem");
    assert!(
        verify_core(&problem, &core),
        "certificate checks by propagation alone"
    );
    // Dropping any single member breaks the certificate (minimality).
    for i in 0..core.len() {
        let mut sub = core.clone();
        sub.remove(i);
        assert!(
            !verify_core(&problem, &sub),
            "proper subset must not certify"
        );
    }
}

#[test]
fn core_cache_replays_the_dead_end_for_the_fleet() {
    let (p, caps, goal) = saturated();
    let constraints = vec![
        Constraint::NotLater {
            a: "receipt".into(),
            k: 2,
        },
        Constraint::NotEarlier {
            a: "admit".into(),
            k: 3,
        },
        Constraint::Before {
            a: "admit".into(),
            b: "receipt".into(),
        },
    ];
    let problem =
        SequenceProblem::with_constraints(&p, caps, goal, 6, constraints).expect("problem");
    let mut cache = CoreCache::new();
    // First agent derives the proof.
    let e1 = Solver8
        .solve_cached(&problem, &mut cache)
        .expect_err("unsat");
    assert!(matches!(
        e1,
        Refusal::UnsatProof {
            replayed: false,
            ..
        }
    ));
    assert_eq!(cache.len(), 1);
    // Every later agent replays it — no propagation, no MUS, no search.
    let e2 = Solver8
        .solve_cached(&problem, &mut cache)
        .expect_err("unsat");
    let Refusal::UnsatProof { replayed, core, .. } = e2 else {
        panic!("expected UnsatProof");
    };
    assert!(replayed, "second solve is a cache hit");
    assert_eq!(core.len(), 3);
    assert_eq!(cache.hits(), 1);
}

// ── the remaining constraint kinds ──────────────────────────────────────────

#[test]
fn excludes_forbids_coexistence() {
    let (p, caps, goal) = saturated();
    // judge and admit are both on the only path — excluding them kills it.
    let constraints = vec![Constraint::Excludes {
        a: "judge".into(),
        b: "admit".into(),
    }];
    let problem =
        SequenceProblem::with_constraints(&p, caps, goal, 6, constraints).expect("problem");
    assert!(matches!(
        Solver8.solve(&problem),
        Err(Refusal::Unsatisfiable { .. })
    ));
}

#[test]
fn budget_below_plan_cost_refuses() {
    let (p, caps, goal) = saturated();
    let constraints = vec![Constraint::Budget { max: 4 }]; // plan costs 5
    let problem =
        SequenceProblem::with_constraints(&p, caps, goal, 6, constraints).expect("problem");
    assert!(matches!(
        Solver8.solve(&problem),
        Err(Refusal::Unsatisfiable { .. })
    ));
}

#[test]
fn requires_is_enforced_at_goal() {
    // Two routes to the goal: cheap 'shortcut' or the judge route; requiring
    // shortcut -> audit forces the plan to include audit (or take the other
    // route — make the other route more expensive so shortcut wins only with
    // its rider).
    use praxis_synthesis::{Atom, Capability, Program, Term};
    let mut p = Program::new();
    let raw = p.intern("raw");
    let done = p.intern("done");
    let audited = p.intern("audited");
    let o1 = p.intern("o1");
    p.add_fact(raw, &[o1]).expect("fact");
    p.saturate().expect("saturation");
    let v0 = Term::Var(0);
    let caps = vec![
        Capability {
            name: "shortcut".into(),
            params: 1,
            pre: vec![Atom::new(raw, vec![v0])],
            add: vec![Atom::new(done, vec![v0])],
            del: vec![],
            cost: 1,
        },
        Capability {
            name: "audit".into(),
            params: 1,
            pre: vec![Atom::new(raw, vec![v0])],
            add: vec![Atom::new(audited, vec![v0])],
            del: vec![],
            cost: 1,
        },
        Capability {
            name: "long-route".into(),
            params: 1,
            pre: vec![Atom::new(raw, vec![v0])],
            add: vec![Atom::new(done, vec![v0])],
            del: vec![],
            cost: 10,
        },
    ];
    let goal = vec![Atom::new(done, vec![Term::Const(o1)])];
    let constraints = vec![Constraint::Requires {
        a: "shortcut".into(),
        b: "audit".into(),
    }];
    let problem =
        SequenceProblem::with_constraints(&p, caps, goal, 4, constraints).expect("problem");
    let plan = Solver8.solve(&problem).expect("solvable");
    let names: Vec<&str> = plan.steps.iter().map(|s| s.capability.as_str()).collect();
    assert!(
        names.contains(&"audit") || !names.contains(&"shortcut"),
        "shortcut without audit is unlawful: {names:?}"
    );
    assert_eq!(
        plan.cost, 2,
        "shortcut+audit (2) beats long-route (10): {names:?}"
    );
}

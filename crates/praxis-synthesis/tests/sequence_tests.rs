//! Layer 2 tests: the solver discovers order + bindings from declarations
//! alone; refusals are receipted; solving is deterministic.

mod common;

use common::lawobject_domain;
use praxis_synthesis::{Atom, BoundedCsp, Refusal, SequenceProblem, Solver, Term};

#[test]
fn solver_rediscovers_the_five_step_lawobject_order() {
    let (mut p, caps, goal) = lawobject_domain();
    p.saturate().expect("saturation");
    let problem = SequenceProblem::new(&p, caps, goal, 6, Vec::new()).expect("problem");
    let plan = BoundedCsp.solve(&problem).expect("solvable");
    let order: Vec<&str> = plan.steps.iter().map(|s| s.capability.as_str()).collect();
    assert_eq!(
        order,
        ["supply-evidence", "clear-obligations", "judge", "admit", "receipt"],
        "the solver must discover the hand-authored Day-2 order from declarations alone"
    );
    assert_eq!(plan.cost, 5);
    // Every step bound to o1 — the solver discovered the parameter too.
    let o1 = p.dict.get("o1").expect("interned");
    assert!(plan.steps.iter().all(|s| s.binding == vec![o1.0]));
    // Independent replay confirms the plan (differential guard).
    assert!(problem.replay_reaches_goal(&plan));
}

#[test]
fn unsatisfiable_goal_is_a_receipted_refusal() {
    let (mut p, caps, _) = lawobject_domain();
    let unreachable = p.intern("unreachable");
    let o1 = p.dict.get("o1").expect("interned");
    p.saturate().expect("saturation");
    let goal = vec![Atom::new(unreachable, vec![Term::Const(o1)])];
    let problem = SequenceProblem::new(&p, caps, goal, 6, Vec::new()).expect("problem");
    let err = BoundedCsp.solve(&problem).expect_err("must refuse");
    match err {
        Refusal::Unsatisfiable { nodes_explored, .. } => {
            assert!(nodes_explored > 0, "refusal carries search-effort salvage");
        }
        other => panic!("expected Unsatisfiable, got {other:?}"),
    }
}

#[test]
fn horizon_too_short_is_unsatisfiable() {
    let (mut p, caps, goal) = lawobject_domain();
    p.saturate().expect("saturation");
    let problem = SequenceProblem::new(&p, caps, goal, 3, Vec::new()).expect("problem");
    assert!(matches!(
        BoundedCsp.solve(&problem),
        Err(Refusal::Unsatisfiable { .. })
    ));
}

#[test]
fn before_constraints_are_respected() {
    let (mut p, caps, goal) = lawobject_domain();
    p.saturate().expect("saturation");
    // Redundant with the causal order, but must not break anything.
    let before = vec![("judge".to_string(), "receipt".to_string())];
    let problem = SequenceProblem::new(&p, caps, goal, 6, before).expect("problem");
    let plan = BoundedCsp.solve(&problem).expect("solvable");
    let judge_at = plan.steps.iter().position(|s| s.capability == "judge").expect("judge");
    let receipt_at =
        plan.steps.iter().position(|s| s.capability == "receipt").expect("receipt");
    assert!(judge_at < receipt_at);
}

#[test]
fn solving_is_deterministic() {
    let solve = || {
        let (mut p, caps, goal) = lawobject_domain();
        p.saturate().expect("saturation");
        let problem = SequenceProblem::new(&p, caps, goal, 6, Vec::new()).expect("problem");
        let plan = BoundedCsp.solve(&problem).expect("solvable");
        (
            serde_json::to_string(&plan.steps).expect("json"),
            plan.receipt.plan_hash,
            plan.receipt.problem_hash,
        )
    };
    assert_eq!(solve(), solve(), "same problem, byte-identical plan and receipt");
}

#[test]
fn unsafe_effect_variables_are_refused_at_problem_build() {
    let (mut p, _, goal) = lawobject_domain();
    let raw = p.dict.get("raw").expect("interned");
    let evidence = p.dict.get("evidence").expect("interned");
    p.saturate().expect("saturation");
    let bad = vec![praxis_synthesis::Capability {
        name: "bad".into(),
        params: 1,
        pre: vec![Atom::new(raw, vec![Term::Var(0)])],
        // ?5 never bound by a precondition.
        add: vec![Atom::new(evidence, vec![Term::Var(5)])],
        del: vec![],
        cost: 1,
    }];
    let err = SequenceProblem::new(&p, bad, goal, 4, Vec::new()).expect_err("must refuse");
    assert!(matches!(err, Refusal::InvalidInput { .. }));
}

#[test]
#[should_panic(expected = "index out of bounds: the len is 8 but the index is 8")]
fn test_variable_out_of_bounds_panic() {
    let (mut p, _, goal) = lawobject_domain();
    let raw = p.dict.get("raw").expect("interned");
    let evidence = p.dict.get("evidence").expect("interned");
    p.saturate().expect("saturation");
    let bad = vec![praxis_synthesis::Capability {
        name: "bad".into(),
        params: 9,
        pre: vec![Atom::new(raw, vec![Term::Var(8)])], // Variable 8 is out of bounds (MAX_VARS is 8)
        add: vec![Atom::new(evidence, vec![Term::Var(8)])],
        del: vec![],
        cost: 1,
    }];
    let problem = SequenceProblem::new(&p, bad, goal, 4, Vec::new()).expect("problem");
    // This will panic due to out of bounds access in StateDb::join
    let _ = BoundedCsp.solve(&problem);
}

#[test]
fn test_after_constraint_logic_is_reversed() {
    let (p, caps, goal) = lawobject_domain();
    // After constraint: clear-obligations (a) must occur AFTER judge (b).
    // So judge (b) must be placed before clear-obligations (a).
    let constraints = vec![
        praxis_synthesis::Constraint::After {
            a: "clear-obligations".into(),
            b: "judge".into(),
        }
    ];
    let problem = SequenceProblem::with_constraints(&p, caps, goal, 6, constraints).expect("problem");
    
    // BoundedCsp ignores the After constraint during search, returning a plan:
    // ["supply-evidence", "clear-obligations", "judge", "admit", "receipt"]
    // where clear-obligations is BEFORE judge (which violates the semantic constraint).
    let plan = BoundedCsp.solve(&problem).expect("BoundedCsp solves");
    
    // Yet, because the logic is reversed, plan_respects_constraints incorrectly returns true!
    assert!(problem.plan_respects_constraints(&plan), "plan_respects_constraints incorrectly returns true");
}

#[test]
fn test_solver8_after_constraint_contradiction() {
    let (p, caps, goal) = lawobject_domain();
    // After constraint: clear-obligations (a) must occur AFTER supply-evidence (b).
    // This is already true in the normal plan: supply-evidence is step 0, clear-obligations is step 1.
    // So this should be solvable, and the normal plan is valid.
    let constraints = vec![
        praxis_synthesis::Constraint::After {
            a: "clear-obligations".into(),
            b: "supply-evidence".into(),
        }
    ];
    let problem = SequenceProblem::with_constraints(&p, caps, goal, 6, constraints).expect("problem");
    
    // In Solver8:
    // 1. Propagation uses the reversed logic: it enforces clear-obligations (a) BEFORE supply-evidence (b).
    // 2. Search enforces the correct logic: supply-evidence (b) BEFORE clear-obligations (a).
    // Because they contradict, Solver8 will fail to find a plan!
    let err = praxis_synthesis::Solver8.solve(&problem).expect_err("Solver8 fails due to propagation/search contradiction");
    assert!(matches!(err, Refusal::Unsatisfiable { .. }));
}


#[test]
fn test_impossible_goal_wastes_search_budget() {
    let (mut p, caps, _) = lawobject_domain();
    let unreachable = p.intern("unreachable");
    let o1 = p.dict.get("o1").expect("interned");
    p.saturate().expect("saturation");
    // This goal predicate has 0 producers.
    let goal = vec![Atom::new(unreachable, vec![Term::Const(o1)])];
    let problem = SequenceProblem::new(&p, caps, goal, 6, Vec::new()).expect("problem");
    
    // Solver8 does not detect this statically because producers.len() == 0, not 1.
    // So it goes to DFS, does search, and returns Unsatisfiable, not UnsatProof.
    let err = praxis_synthesis::Solver8.solve(&problem).expect_err("must refuse");
    assert!(
        matches!(err, Refusal::Unsatisfiable { nodes_explored, .. } if nodes_explored > 0),
        "Expected Unsatisfiable with search effort, got {:?}", err
    );
}


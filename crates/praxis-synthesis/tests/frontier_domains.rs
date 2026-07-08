//! P6 — the falsifier loop: the generator hunts for solver disagreement,
//! and the frontier sweep receipts every cell (solved / refused / trap).
//!
//! The receipt is not "N domains passed"; it is "the generator actively
//! mutated toward the highest-discrepancy region for G generations and the
//! differential held."

use praxis_synthesis::gen::{generate, DomainSpec};
use praxis_synthesis::{BoundedCsp, Refusal, SequenceProblem, Solver, Solver8};

/// Outcome of one cell: both solvers' verdicts + discrepancy scoring.
struct Cell {
    spec: DomainSpec,
    outcome: &'static str,
    /// Falsifier fitness: how close the two solvers came to disagreeing.
    discrepancy: f64,
}

fn run_cell(spec: &DomainSpec) -> Cell {
    let (mut p, caps, goal, constraints) = generate(spec);
    if p.saturate().is_err() {
        return Cell {
            spec: *spec,
            outcome: "saturation-refused",
            discrepancy: 0.0,
        };
    }
    let horizon = spec.horizon().min(16);
    let Ok(problem) = SequenceProblem::with_constraints(&p, caps, goal, horizon, constraints)
    else {
        return Cell {
            spec: *spec,
            outcome: "problem-refused",
            discrepancy: 0.0,
        };
    };
    let smart = Solver8.solve(&problem);
    let brute = BoundedCsp.solve(&problem);
    match (smart, brute) {
        (Ok(s), Ok(b)) => {
            // HARD differential: identical plans and costs, and the smart
            // solver never searches more.
            assert_eq!(s.steps, b.steps, "PLAN DISAGREEMENT at {spec:?}");
            assert_eq!(s.cost, b.cost, "COST DISAGREEMENT at {spec:?}");
            assert!(
                s.receipt.nodes_explored <= b.receipt.nodes_explored,
                "Solver8 searched more than brute at {spec:?}"
            );
            // Both solved: verify independently.
            assert!(problem.replay_reaches_goal(&s), "replay fails at {spec:?}");
            // Fitness: node-ratio distance from 1 means the solvers worked
            // very differently — the region where bugs live.
            #[allow(clippy::cast_precision_loss)]
            let ratio =
                (b.receipt.nodes_explored.max(1)) as f64 / (s.receipt.nodes_explored.max(1)) as f64;
            Cell {
                spec: *spec,
                outcome: "solved-agree",
                discrepancy: ratio,
            }
        }
        (Err(_), Err(_)) => {
            // Both refuse — agreement. (Reasons may differ lawfully: Solver8
            // may certify what brute can only exhaust.)
            Cell {
                spec: *spec,
                outcome: "refused-agree",
                discrepancy: 0.5,
            }
        }
        (Ok(s), Err(Refusal::BudgetExceeded { .. })) => {
            // Brute ran out of budget where Solver8 (pruned) succeeded:
            // lawful asymmetry, but verify the plan independently.
            assert!(problem.replay_reaches_goal(&s), "replay fails at {spec:?}");
            Cell {
                spec: *spec,
                outcome: "smart-only",
                discrepancy: 3.0,
            }
        }
        (Err(Refusal::BudgetExceeded { .. }), Ok(b)) => {
            assert!(problem.replay_reaches_goal(&b), "replay fails at {spec:?}");
            Cell {
                spec: *spec,
                outcome: "brute-only",
                discrepancy: 3.0,
            }
        }
        (Ok(_), Err(e)) => {
            panic!("SOLVER8 FOUND A PLAN WHERE ORACLE PROVED NONE at {spec:?}: {e}")
        }
        (Err(e), Ok(_)) => {
            panic!("SOLVER8 REFUSED A SOLVABLE PROBLEM at {spec:?}: {e}")
        }
    }
}

#[test]
fn falsifier_sweep_holds_the_differential() {
    // Generation 0: 64 seeded specs across the space.
    let mut population: Vec<DomainSpec> = (0..64u64)
        .map(|s| DomainSpec::from_seed(s * 0x9E37_79B9 + 1))
        .collect();
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    let mut cells_run = 0usize;

    const GENERATIONS: usize = 4;
    for g in 0..GENERATIONS {
        let mut scored: Vec<Cell> = population.iter().map(run_cell).collect();
        cells_run += scored.len();
        for c in &scored {
            *counts.entry(c.outcome).or_default() += 1;
        }
        // Coordinate ascent on discrepancy: mutate the top quartile toward
        // every axis, both directions — the falsifier walks TOWARD the
        // regions where the solvers behaved most differently.
        scored.sort_by(|a, b| b.discrepancy.total_cmp(&a.discrepancy));
        let elite: Vec<DomainSpec> = scored.iter().take(16).map(|c| c.spec).collect();
        if g + 1 < GENERATIONS {
            population = elite
                .iter()
                .flat_map(|s| (0..4u8).map(move |ax| s.mutate(ax * 2 + (g as u8 % 2), g % 2 == 0)))
                .collect();
        }
    }

    // The receipt: the hunt ran and the differential never broke (any break
    // panics above). Coverage must include the interesting outcomes.
    assert!(
        cells_run >= 256,
        "at least 256 adversarial cells: ran {cells_run}"
    );
    assert!(counts.contains_key("solved-agree"), "outcomes: {counts:?}");
    assert!(
        counts.contains_key("refused-agree") || counts.contains_key("smart-only"),
        "the space must include refusals or asymmetric cells: {counts:?}"
    );
    // Persist the frontier report.
    let report = serde_json::json!({
        "what": "falsifier frontier sweep — differential-guided domain mutation",
        "generations": GENERATIONS,
        "cells_run": cells_run,
        "outcomes": counts.iter().map(|(k, v)| (k.to_string(), v)).collect::<std::collections::BTreeMap<_,_>>(),
        "differential_held": true,
    });
    std::fs::write(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../target/synthesis-frontier-report.json"
        ),
        serde_json::to_string_pretty(&report).expect("serialize"),
    )
    .expect("write report");
}

#[test]
fn interference_traps_are_navigated_not_fallen_into() {
    // A spec with interfering dead ends: taking the trap deletes the entry
    // token. The solver must route around it.
    let spec = DomainSpec {
        depth: 3,
        branching: 2,
        objects: 1,
        dead_ends: 2,
        interference: 2,
        rule_depth: 2,
        constraint_load: 0,
        goal_width: 2,
    };
    let (mut p, caps, goal, constraints) = generate(&spec);
    p.saturate().expect("saturation");
    let problem = SequenceProblem::with_constraints(&p, caps, goal, spec.horizon(), constraints)
        .expect("problem");
    let plan = Solver8.solve(&problem).expect("navigable despite traps");
    assert!(
        !plan.steps.iter().any(|s| s.capability.starts_with("dead-")),
        "an optimal plan never enters a dead end: {:?}",
        plan.steps.iter().map(|s| &s.capability).collect::<Vec<_>>()
    );
    assert!(problem.replay_reaches_goal(&plan));
}

#[test]
fn derived_predicates_feed_the_solver() {
    // rule_depth > 0: the entry precondition is a DERIVED predicate — only
    // saturation makes it true. Skipping Layer 1 must make the problem
    // unsolvable; running it must make it solvable.
    let spec = DomainSpec {
        depth: 2,
        branching: 1,
        objects: 1,
        dead_ends: 0,
        interference: 0,
        rule_depth: 3,
        constraint_load: 0,
        goal_width: 1,
    };
    let (mut p, caps, goal, constraints) = generate(&spec);
    // Without saturation: derived3 is absent → unsat (certified: nothing
    // produces the derived predicate).
    let unsat = SequenceProblem::with_constraints(
        &p,
        caps.clone(),
        goal.clone(),
        spec.horizon(),
        constraints.clone(),
    )
    .expect("problem");
    assert!(
        Solver8.solve(&unsat).is_err(),
        "underived preconditions must fail"
    );
    // With saturation: the Datalog→solver stack works.
    p.saturate().expect("saturation");
    let sat = SequenceProblem::with_constraints(&p, caps, goal, spec.horizon(), constraints)
        .expect("problem");
    let plan = Solver8
        .solve(&sat)
        .expect("derived predicates feed the solver");
    assert!(sat.replay_reaches_goal(&plan));
}

//! End-to-end grounder tests. These construct `Pddl8Domain`/`Pddl8Problem`
//! values directly (the parser lives in `bcinr-pddl`, which this crate does not
//! depend on) and check the join grounder's pruning and plan-finding.

use pddl_index::{candidate_estimate, IndexedGroundProblem};
use wasm4pm_compat::pddl::{
    Pddl8ActionSchema, Pddl8Atom, Pddl8Domain, Pddl8Problem,
};

fn atom(pred: &str, args: &[&str]) -> Pddl8Atom {
    Pddl8Atom { pred: pred.into(), args: args.iter().map(|s| (*s).into()).collect() }
}

fn schema(
    name: &str,
    params: &[&str],
    pre: Vec<Pddl8Atom>,
    add: Vec<Pddl8Atom>,
    del: Vec<Pddl8Atom>,
) -> Pddl8ActionSchema {
    Pddl8ActionSchema {
        name: name.into(),
        params: params.iter().map(|s| (*s).into()).collect(),
        preconditions: pre,
        add_effects: add,
        del_effects: del,
        typed_params: Vec::new(),
        condition: None,
        effects: Vec::new(),
        numeric_effects: Vec::new(),
    }
}

fn empty_domain(name: &str, actions: Vec<Pddl8ActionSchema>) -> Pddl8Domain {
    Pddl8Domain {
        name: name.into(),
        predicates: Vec::new(),
        actions,
        types: Vec::new(),
        functions: Vec::new(),
        durative_actions: Vec::new(),
        derived: Vec::new(),
        constraints: Vec::new(),
        processes: Vec::new(),
        events: Vec::new(),
    }
}

fn problem(name: &str, objects: &[&str], init: Vec<Pddl8Atom>, goal: Vec<Pddl8Atom>) -> Pddl8Problem {
    Pddl8Problem {
        name: name.into(),
        domain: name.into(),
        objects: objects.iter().map(|s| (*s).into()).collect(),
        init,
        goal,
        object_types: Vec::new(),
        fn_values: Vec::new(),
        timed_inits: Vec::new(),
        preferences: Vec::new(),
        metric: None,
    }
}

/// A transport domain: `move ?from ?to` requires `(at ?from)` and a static
/// `(link ?from ?to)`. Over N locations there are N² candidate `move`
/// groundings, but only the |links| that actually exist can ever fire.
fn transport(n: usize, links: &[(usize, usize)]) -> (Pddl8Domain, Pddl8Problem) {
    let mv = schema(
        "move",
        &["?from", "?to"],
        vec![atom("at", &["?from"]), atom("link", &["?from", "?to"])],
        vec![atom("at", &["?to"])],
        vec![atom("at", &["?from"])],
    );
    let domain = empty_domain("transport", vec![mv]);
    let names: Vec<String> = (0..n).map(|i| format!("l{i}")).collect();
    let obj_refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let mut init = vec![atom("at", &["l0"])];
    for &(a, b) in links {
        init.push(atom("link", &[&names[a], &names[b]]));
    }
    let goal = vec![atom("at", &[&names[n - 1]])];
    let prob = problem("transport", &obj_refs, init, goal);
    (domain, prob)
}

#[test]
fn prunes_dead_groundings_on_a_path() {
    // A directed path l0 -> l1 -> ... -> l39. 40 objects ⇒ 1600 candidate
    // `move` groundings, but only 39 links exist.
    let n = 40;
    let links: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
    let (domain, prob) = transport(n, &links);

    assert_eq!(candidate_estimate(&domain, &prob), n * n, "N² candidate groundings");

    let gp = IndexedGroundProblem::build(&domain, &prob, None).expect("build");
    let stats = gp.stats();
    assert_eq!(stats.candidate_groundings, n * n);
    // Only the 39 real links are reachable & materialized — far below 1600.
    assert_eq!(stats.materialized_groundings, n - 1);
    assert!(stats.materialization_ratio() < 0.05, "ratio {}", stats.materialization_ratio());

    let tape = gp.find_plan().expect("path is solvable");
    assert_eq!(tape.len(), n - 1, "shortest plan walks every link once");
}

#[test]
fn unreachable_goal_refuses() {
    // l0 links only to l1; goal is at l3 — unreachable.
    let (domain, prob) = transport(4, &[(0, 1)]);
    let gp = IndexedGroundProblem::build(&domain, &prob, None).expect("build");
    assert!(matches!(gp.find_plan(), Err(pddl_index::GroundError::NoAdmittedPlan)));
}

#[test]
fn no_precondition_schema_grounds_full_product() {
    // A parameterized no-precondition action must ground over every object,
    // exactly like the naive grounder (no pruning is possible).
    let s = schema("touch", &["?x"], vec![], vec![atom("seen", &["?x"])], vec![]);
    let domain = empty_domain("d", vec![s]);
    let prob = problem("d", &["a", "b", "c"], vec![], vec![atom("seen", &["c"])]);
    let gp = IndexedGroundProblem::build(&domain, &prob, None).expect("build");
    assert_eq!(gp.stats().materialized_groundings, 3);
    assert_eq!(gp.find_plan().expect("solvable").len(), 1);
}

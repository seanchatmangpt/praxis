//! Geometry tests: derived branches, fragile-fact mining, first-match
//! classification, and the unshadowable gap.

mod common;

use common::lawobject_domain;
use praxis_synthesis::fault::RuntimeClass;
use praxis_synthesis::geometry::{
    CrashKind, CrashSnapshot, FailureClass, FailureGeometry, LawfulResponse,
};
use praxis_synthesis::park::ReAdmission;
use praxis_synthesis::supervise::{RestartPolicy, SupervisionTopology};
use praxis_synthesis::{Atom, BoundedCsp, Capability, Program, SequenceProblem, Solver, Term};

fn derived() -> (FailureGeometry, SupervisionTopology, Vec<String>) {
    let (mut p, caps, goal) = lawobject_domain();
    p.saturate().expect("saturation");
    let problem = SequenceProblem::new(&p, caps, goal, 6, Vec::new()).expect("problem");
    let plan = BoundedCsp.solve(&problem).expect("solvable");
    let topo = SupervisionTopology::derive(&plan, &problem, RestartPolicy::default_policy())
        .expect("topology");
    let nodes: Vec<String> = topo.stages.iter().flat_map(|s| s.nodes.clone()).collect();
    let geometry = FailureGeometry::derive(&topo, &plan, &problem);
    (geometry, topo, nodes)
}

fn snapshot(node: &str, kind: CrashKind) -> CrashSnapshot {
    CrashSnapshot {
        node_id: node.into(),
        attempt: 0,
        ticks_used: 1,
        tick_budget: Some(8),
        tier: RuntimeClass::W1,
        kind,
        refusal_head: None,
        upstream_parked: false,
        progressed: true,
    }
}

#[test]
fn derivation_is_deterministic_and_anchored() {
    let (a, _, _) = derived();
    let (b, _, _) = derived();
    assert_eq!(a.geometry_hash, b.geometry_hash);
    assert!(a.branch_count() >= 40, "8+ branches per node x 5 nodes");
}

#[test]
fn predicted_classes_land_in_named_branches() {
    let (g, _, nodes) = derived();
    let n = &nodes[0];
    // Transient I/O → Restart.
    let c = g.classify(&snapshot(n, CrashKind::Io));
    assert_eq!((c.class, c.matched), (FailureClass::TransientFault, true));
    assert_eq!(c.response, LawfulResponse::Restart);
    // Over budget (time tier) → Park(AfterRuns(1)).
    let c = g.classify(&snapshot(n, CrashKind::OverBudget));
    assert_eq!(c.class, FailureClass::BudgetBreach);
    assert_eq!(c.response, LawfulResponse::Park(ReAdmission::AfterRuns(1)));
    // Tick breach (hot tier) → certified refusal, first-match beats Park.
    let mut s = snapshot(n, CrashKind::OverBudget);
    s.ticks_used = 9;
    let c = g.classify(&s);
    assert_eq!(c.class, FailureClass::BudgetBreach);
    assert!(matches!(c.response, LawfulResponse::Refuse { .. }));
    // Starved input → Park(OnInputChange).
    let mut s = snapshot(n, CrashKind::Io);
    s.upstream_parked = true;
    let c = g.classify(&s);
    assert_eq!(c.class, FailureClass::StarvedInput);
    // Runtime unsat certificate → Refuse.
    let mut s = snapshot(n, CrashKind::Refused);
    s.refusal_head = Some("unsat (certified): mandatory capability ...".into());
    let c = g.classify(&s);
    assert_eq!(c.class, FailureClass::CertifiedUnsat);
    // Bad output: first attempt restarts, second parks Manual.
    let c = g.classify(&snapshot(n, CrashKind::BadOutput));
    assert_eq!(
        (c.class, c.response),
        (FailureClass::LogicFault, LawfulResponse::Restart)
    );
    let mut s = snapshot(n, CrashKind::BadOutput);
    s.attempt = 1;
    let c = g.classify(&s);
    assert_eq!(c.response, LawfulResponse::Park(ReAdmission::Manual));
    // Stall: second attempt without progress restarts under the Stall class.
    let mut s = snapshot(n, CrashKind::Io);
    s.attempt = 1;
    s.progressed = false;
    let c = g.classify(&s);
    assert_eq!(
        c.class,
        FailureClass::Stall,
        "stall outranks transient by order"
    );
}

#[test]
fn fragile_preconditions_are_mined_into_authority_vacuum_refusals() {
    // The release domain: gate-check requires 'authorization', which the
    // initial state supplies (when authorized) and NO capability produces —
    // fragile by the sole-producer analysis. Build WITH the fact so a plan
    // exists, then confirm the derived geometry still guards its loss.
    let mut p = Program::new();
    let quiescent = p.intern("quiescent");
    let authorization = p.intern("authorization");
    let pushed = p.intern("pushed");
    let repo = p.intern("praxis");
    p.add_fact(quiescent, &[repo]).expect("fact");
    p.add_fact(authorization, &[repo]).expect("fact");
    p.saturate().expect("saturation");
    let v0 = Term::Var(0);
    let caps = vec![Capability {
        name: "push".into(),
        params: 1,
        pre: vec![
            Atom::new(quiescent, vec![v0]),
            Atom::new(authorization, vec![v0]),
        ],
        add: vec![Atom::new(pushed, vec![v0])],
        del: vec![],
        cost: 1,
    }];
    let goal = vec![Atom::new(pushed, vec![Term::Const(repo)])];
    let problem = SequenceProblem::new(&p, caps, goal, 2, Vec::new()).expect("problem");
    let plan = BoundedCsp.solve(&problem).expect("authorized: solvable");
    let topo = SupervisionTopology::derive(&plan, &problem, RestartPolicy::default_policy())
        .expect("topology");
    let g = FailureGeometry::derive(&topo, &plan, &problem);
    // Note: both preconditions are init-only (no producers) — but only
    // presence in init at planning time distinguishes them from truly
    // missing facts; both are fragile and guarded. Losing authorization at
    // runtime must classify as AuthorityVacuum with the fact NAMED.
    let node = topo.stages[0].nodes[0].clone();
    let mut s = snapshot(&node, CrashKind::PreconditionLost);
    s.tier = RuntimeClass::R1;
    let c = g.classify(&s);
    assert_eq!(c.class, FailureClass::AuthorityVacuum);
    let LawfulResponse::Refuse { core } = &c.response else {
        panic!("authority vacuum must refuse, got {:?}", c.response);
    };
    assert!(
        core.iter()
            .any(|f| f.contains("authorization") || f.contains("quiescent")),
        "the certificate names the fragile fact: {core:?}"
    );
}

#[test]
fn the_gap_is_implicit_and_unshadowable() {
    let (g, _, nodes) = derived();
    // No stored branch may carry GeometryGap — structural invariant.
    for list in g.branches.values() {
        assert!(list.iter().all(|b| b.class != FailureClass::GeometryGap));
    }
    // An unpredicted crash shape: PreconditionLost on a node with no
    // fragile preconditions (lawobject facts all have producers upstream).
    let c = g.classify(&snapshot(&nodes[2], CrashKind::PreconditionLost));
    assert_eq!(c.class, FailureClass::GeometryGap);
    assert!(!c.matched, "the gap is honest");
    assert_eq!(
        c.response,
        LawfulResponse::Restart,
        "safe default, receipted as gap"
    );
    // Unknown node entirely → also a gap, never a panic.
    let c = g.classify(&snapshot("no-such-node", CrashKind::Io));
    assert_eq!(c.class, FailureClass::GeometryGap);
}

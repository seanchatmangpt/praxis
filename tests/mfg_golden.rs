//! Golden test for the RDF-to-PDDL manufacturing lane (`mfg` noun,
//! `--features ggen`).
//!
//! `ontology/lawobject.ttl` is the PDDL8-safe flattening of
//! `docs/PDDL_CAPABILITY_MODEL.md` / `docs/lawobject-capability.pddl`.
//! Its manufactured domain+problem MUST round-trip through `bcinr-pddl`'s
//! parser/grounder and MUST solve via `find_plan` to exactly the five-step
//! plan documented in the ontology file's header comment. Downstream lanes
//! (notably a future `plan lawobject` self-test) pin this same contract —
//! do not change the action order, names, or the ontology file path without
//! updating every consumer.
#![cfg(feature = "ggen")]

use my_conforming_project::mfg;

const LAWOBJECT_TTL: &str = include_str!("../ontology/lawobject.ttl");
const EXPECTED_PLAN: &[&str] = &[
    "supply-evidence",
    "clear-obligations",
    "judge",
    "admit",
    "receipt",
];

#[test]
fn golden_roundtrip_and_solve() {
    let manufactured = mfg::manufacture(LAWOBJECT_TTL, "ontology/lawobject.ttl")
        .expect("manufacture ontology/lawobject.ttl");

    let report = mfg::solve_ir(&manufactured);
    assert!(
        report.parsed,
        "domain/problem failed to parse: {:?}",
        report.error
    );
    assert!(
        report.solvable,
        "domain/problem failed to solve: {:?}",
        report.error
    );
    assert_eq!(
        report.plan_steps, EXPECTED_PLAN,
        "golden plan action sequence changed"
    );
    assert_eq!(report.plan_len, EXPECTED_PLAN.len());
}

#[test]
fn determinism_byte_identical_across_runs() {
    let a = mfg::manufacture(LAWOBJECT_TTL, "ontology/lawobject.ttl").expect("run a");
    let b = mfg::manufacture(LAWOBJECT_TTL, "ontology/lawobject.ttl").expect("run b");
    assert_eq!(
        a.receipt.shapes_hash, b.receipt.shapes_hash,
        "shapes hash is not deterministic"
    );
    assert_eq!(
        a.receipt.profile_name, b.receipt.profile_name,
        "profile name is not deterministic"
    );
    assert_eq!(
        a.receipt.graph_hash, b.receipt.graph_hash,
        "graph hash is not deterministic"
    );
}

#[test]
fn out_of_bounds_predicate_arity_rejected_before_emission() {
    let ttl = r#"
        @prefix pddl: <http://seanchatmangpt.github.io/praxis/pddl#> .
        pddl:domain_x a pddl:Domain ; pddl:name "over-bound" .
        pddl:pred_wide a pddl:Predicate ; pddl:name "wide" ;
          pddl:param [ pddl:index 0 ; pddl:var "?a0" ; pddl:ofType "t" ] ,
                    [ pddl:index 1 ; pddl:var "?a1" ; pddl:ofType "t" ] ,
                    [ pddl:index 2 ; pddl:var "?a2" ; pddl:ofType "t" ] ,
                    [ pddl:index 3 ; pddl:var "?a3" ; pddl:ofType "t" ] ,
                    [ pddl:index 4 ; pddl:var "?a4" ; pddl:ofType "t" ] ,
                    [ pddl:index 5 ; pddl:var "?a5" ; pddl:ofType "t" ] ,
                    [ pddl:index 6 ; pddl:var "?a6" ; pddl:ofType "t" ] ,
                    [ pddl:index 7 ; pddl:var "?a7" ; pddl:ofType "t" ] ,
                    [ pddl:index 8 ; pddl:var "?a8" ; pddl:ofType "t" ] .
    "#;
    let err = mfg::manufacture(ttl, "inline-test").expect_err("9-ary predicate must be rejected");
    match err {
        mfg::MfgError::Shape(msg) => {
            assert!(msg.contains("SHACL violation"));
        }
        other => panic!("expected Shape error for SHACL violation, got {other:?}"),
    }
}

#[test]
fn facts_json_row_shape_matches_ggen_core_expectations() {
    let graph = mfg::load_graph(LAWOBJECT_TTL).expect("load graph");
    let query = "PREFIX pddl: <http://seanchatmangpt.github.io/praxis/pddl#>\n\
                 SELECT ?name WHERE { ?a a pddl:Action ; pddl:name ?name } ORDER BY ?name";
    let rows = mfg::facts_json(&graph, query).expect("facts_json");
    let arr = rows.as_array().expect("rows is a JSON array");
    assert_eq!(arr.len(), 5, "expected 5 actions in the golden ontology");
    let names: Vec<&str> = arr
        .iter()
        .map(|row| {
            row.as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap()
        })
        .collect();
    assert_eq!(
        names,
        vec![
            "admit",
            "clear-obligations",
            "judge",
            "receipt",
            "supply-evidence"
        ]
    );
}

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
const EXPECTED_PLAN: &[&str] =
    &["supply-evidence", "clear-obligations", "judge", "admit", "receipt"];

#[test]
fn golden_roundtrip_and_solve() {
    let manufactured = mfg::manufacture(LAWOBJECT_TTL, "ontology/lawobject.ttl")
        .expect("manufacture ontology/lawobject.ttl");

    let report = mfg::validate(&manufactured.domain_text, &manufactured.problem_text);
    assert!(report.parsed, "domain/problem failed to parse: {:?}", report.error);
    assert!(report.solvable, "domain/problem failed to solve: {:?}", report.error);
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
    assert_eq!(a.domain_text, b.domain_text, "domain text is not deterministic");
    assert_eq!(a.problem_text, b.problem_text, "problem text is not deterministic");
    assert_eq!(a.graph_hash_hex, b.graph_hash_hex, "graph hash is not deterministic");
}

#[test]
fn out_of_bounds_predicate_arity_rejected_before_emission() {
    let ttl = r#"
        @prefix pdl: <http://seanchatmangpt.github.io/praxis/pddl#> .
        pdl:domain_x a pdl:Domain ; pdl:name "over-bound" .
        pdl:pred_wide a pdl:Predicate ; pdl:name "wide" ;
          pdl:param [ pdl:index 0 ; pdl:var "?a0" ; pdl:ofType "t" ] ,
                    [ pdl:index 1 ; pdl:var "?a1" ; pdl:ofType "t" ] ,
                    [ pdl:index 2 ; pdl:var "?a2" ; pdl:ofType "t" ] ,
                    [ pdl:index 3 ; pdl:var "?a3" ; pdl:ofType "t" ] ,
                    [ pdl:index 4 ; pdl:var "?a4" ; pdl:ofType "t" ] ,
                    [ pdl:index 5 ; pdl:var "?a5" ; pdl:ofType "t" ] ,
                    [ pdl:index 6 ; pdl:var "?a6" ; pdl:ofType "t" ] ,
                    [ pdl:index 7 ; pdl:var "?a7" ; pdl:ofType "t" ] ,
                    [ pdl:index 8 ; pdl:var "?a8" ; pdl:ofType "t" ] .
    "#;
    let err = mfg::manufacture(ttl, "inline-test").expect_err("9-ary predicate must be rejected");
    match err {
        mfg::MfgError::BoundExceeded { what, limit, got, .. } => {
            assert_eq!(what, "predicate arity");
            assert_eq!(limit, 8);
            assert_eq!(got, 9);
        }
        other => panic!("expected BoundExceeded, got {other:?}"),
    }
}

#[test]
fn facts_json_row_shape_matches_ggen_core_expectations() {
    let graph = mfg::load_graph(LAWOBJECT_TTL).expect("load graph");
    let query = "PREFIX pdl: <http://seanchatmangpt.github.io/praxis/pddl#>\n\
                 SELECT ?name WHERE { ?a a pdl:Action ; pdl:name ?name } ORDER BY ?name";
    let rows = mfg::facts_json(&graph, query).expect("facts_json");
    let arr = rows.as_array().expect("rows is a JSON array");
    assert_eq!(arr.len(), 5, "expected 5 actions in the golden ontology");
    let names: Vec<&str> = arr
        .iter()
        .map(|row| row.as_object().unwrap().get("name").unwrap().as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec!["admit", "clear-obligations", "judge", "receipt", "supply-evidence"]
    );
}

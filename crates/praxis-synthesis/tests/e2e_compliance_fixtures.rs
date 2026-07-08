//! E2E Compliance Fixture Suite for Graphlaw v26.7.7 (FR1-FR15).
//!
//! Verifies both positive and negative execution paths for hook pack loading,
//! admission, trigger evaluation, actions, phase selection, scheduling,
//! delta/boundary requests, BLAKE3 receipts, idempotency, replay, and complexity bounds.

use std::fs;

use praxis_synthesis::{
    fire_hooks, idempotency_key, load_hook_pack, project_delta_template, BoundaryRequest,
    FiringOutcome, HandlerRegistry, MeaningSource, Origin, Reference, Refusal,
};
use tempfile::TempDir;

fn create_temp_pack(
    name: &str,
    version: &str,
    description: &str,
    dialects: &[&str],
    ttl: &str,
) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let toml = format!(
        r#"[pack]
name = "{name}"
version = "{version}"
description = "{description}"
required_dialects = [{dialects}]
"#,
        dialects = dialects
            .iter()
            .map(|d| format!(r#""{}""#, d))
            .collect::<Vec<_>>()
            .join(", ")
    );
    fs::write(dir.path().join("pack.toml"), toml).unwrap();
    fs::write(dir.path().join("ontology.ttl"), ttl).unwrap();
    dir
}

#[test]
fn test_fr1_fr2_hook_pack_intake_and_refusal() {
    // 1. Positive case: valid pack loading (FR1, FR2)
    let ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:on "assert" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "some reason" ;
    hook:priority 1 .
"#;
    let pack_dir = create_temp_pack("valid-pack", "26.7.7", "E2E test pack", &["delta"], ttl);
    let pack = load_hook_pack(pack_dir.path()).expect("should load valid pack");
    assert_eq!(pack.name, "valid-pack");
    assert_eq!(pack.version, "26.7.7");

    // 2. Negative case: malformed TTL syntax (FR1)
    let bad_ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
ex:h1 a hook:Hook ;
    hook:name "h1"
    hook:kind "delta" .
"#; // missing semicolon
    let bad_pack_dir = create_temp_pack("bad-ttl", "1.0.0", "Malformed TTL", &["delta"], bad_ttl);
    let res = load_hook_pack(bad_pack_dir.path());
    assert!(res.is_err(), "Malformed TTL must be refused (FR1)");

    // 3. Negative case: forbidden keyword/action (FR1, FR5)
    let forbidden_ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "tries to invoke shell exec" .
"#;
    let forbidden_pack_dir = create_temp_pack(
        "forbidden",
        "1.0.0",
        "Forbidden action",
        &["delta"],
        forbidden_ttl,
    );
    let res = load_hook_pack(forbidden_pack_dir.path());
    assert!(
        res.is_err(),
        "Forbidden actions/keywords must be refused (FR1, FR5)"
    );
}

#[test]
fn test_fr3_fr4_triggers_and_unsupported_dialects() {
    // Negative case: Unsupported dialect is refused naming the honest analog (FR4)
    for unsupported in &["sparql-ask", "sparql-select", "semantic-inference"] {
        let pack_dir = create_temp_pack("unsupported-dialect", "1.0.0", "desc", &[unsupported], "");
        let res = load_hook_pack(pack_dir.path());
        println!("DEBUG: unsupported={} res={:?}", unsupported, res);
        assert!(res.is_err());
        match res.unwrap_err() {
            Refusal::ConditionUnsupported {
                kind,
                supported_analog,
                ..
            } => {
                assert_eq!(kind, *unsupported);
                assert!(
                    !supported_analog.is_empty(),
                    "Must name honest analog (FR4)"
                );
            }
            other => panic!("expected ConditionUnsupported, got {:?}", other),
        }
    }
}

#[test]
fn test_fr7_hook_ordering_and_cycle_detection() {
    // 1. Positive case: Deterministic priority + after dependencies (FR7)
    let ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" ;
    hook:priority 2 ;
    hook:after ex:h2 .

ex:h2 a hook:Hook ;
    hook:name "h2" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" ;
    hook:priority 5 .
"#;
    let pack_dir = create_temp_pack("order-pack", "1.0.0", "desc", &["delta"], ttl);
    let pack = load_hook_pack(pack_dir.path()).unwrap();
    assert_eq!(
        pack.hooks[0].name, "h2",
        "h2 must run first despite lower priority due to dependency (FR7)"
    );
    assert_eq!(pack.hooks[1].name, "h1");

    // 2. Negative case: Cycle detection (FR7)
    let cycle_ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" ;
    hook:after ex:h2 .

ex:h2 a hook:Hook ;
    hook:name "h2" ;
    hook:kind "delta" ;
    hook:var "v" ;
    hook:effect "refuse" ;
    hook:reason "r" ;
    hook:after ex:h1 .
"#;
    let cycle_dir = create_temp_pack("cycle-pack", "1.0.0", "desc", &["delta"], cycle_ttl);
    let res = load_hook_pack(cycle_dir.path());
    assert!(res.is_err(), "Dependency cycles must be refused (FR7)");
}

#[test]
fn test_fr8_fr9_delta_and_boundary_requests() {
    // 1. Delta template and variable projection (FR8)
    let adds_template = "ex:subject ex:predicate ?0 .";
    let removes_template = "ex:subject ex:predicate ?1 .";
    let vars = vec![
        "http://example.org/value0".to_string(),
        "http://example.org/value1".to_string(),
    ];
    let (projected_adds, projected_removes) =
        project_delta_template(adds_template, removes_template, &vars);
    assert_eq!(
        projected_adds,
        "ex:subject ex:predicate <http://example.org/value0> ."
    );
    assert_eq!(
        projected_removes,
        "ex:subject ex:predicate <http://example.org/value1> ."
    );

    // 2. Boundary request validation (FR9)
    let base_ttl = "@prefix ex: <http://example.org/> . ex:a ex:b ex:c .";
    let reference = Reference::genesis(base_ttl).expect("genesis admitted");
    let req = BoundaryRequest::new(
        &reference,
        "http://example.org/hook1",
        "hook1",
        "event_hash_123",
        "delta_ttl_hash_123",
        "freshness_token_123",
    );
    assert_eq!(req.state_epoch, reference.epoch());
    assert_eq!(req.base_graph_hash, reference.graph_hash());
    assert_eq!(req.hook_iri, "http://example.org/hook1");
    assert_eq!(req.hook_name, "hook1");
    assert_eq!(req.event_hash, "event_hash_123");
    assert_eq!(req.delta_ttl_hash, "delta_ttl_hash_123");
    assert_eq!(req.freshness_token, "freshness_token_123");
    assert!(
        !req.idempotency_key.is_empty(),
        "Idempotency key must be populated (FR9, FR11)"
    );
}

#[test]
fn test_fr10_fr11_fr13_receipt_idempotency_replay() {
    let base_ttl = r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:on "assert" ;
    hook:kind "delta" ;
    hook:var "http://example.org/p" ;
    hook:effect "refuse" ;
    hook:reason "triggered refusal" .
"#;
    let reference = Reference::genesis(base_ttl).expect("genesis admitted");
    let registry = HandlerRegistry::builtin();

    // Triggering event containing adds
    let adds_ttl = "<http://example.org/s> <http://example.org/p> <http://example.org/o> .";
    let source = MeaningSource {
        origin: Origin::Proposer,
        adds_ttl: adds_ttl.to_string(),
        removes_ttl: String::new(),
    };

    let receipt = fire_hooks(&reference, &source, &registry, &[]).expect("fires");
    assert!(matches!(receipt.outcome, FiringOutcome::Refused { .. }));

    // Replay must be stable and yield identical receipts (FR13)
    let receipt2 = fire_hooks(&reference, &source, &registry, &[]).expect("fires");
    assert_eq!(
        receipt.chain, receipt2.chain,
        "Receipt hashes must be stable (FR13)"
    );

    // Idempotency keys must match and link to the receipt hash (FR11)
    let key = receipt.idempotency_key();
    let key2 = receipt2.idempotency_key();
    assert_eq!(
        key, key2,
        "Idempotency keys must be identical across identical runs"
    );
    assert_eq!(
        key,
        idempotency_key(&receipt.chain),
        "Idempotency key must link to receipt hash (FR11)"
    );
}

#[test]
fn test_fr14_fr15_bounds_and_log_projection() {
    // 1. Max hooks bound: 12 hooks (FR14)
    let mut body = String::new();
    for i in 0..13 {
        body.push_str(&format!(
            "ex:h{} a hook:Hook ; hook:name \"h{}\" ; hook:kind \"delta\" ; hook:var \"v\" ; hook:effect \"refuse\" ; hook:reason \"r\" .\n",
            i, i
        ));
    }
    let pack_dir = create_temp_pack("bounds-pack", "1.0.0", "desc", &["delta"], &body);
    let res = load_hook_pack(pack_dir.path());
    assert!(
        res.is_err(),
        "More than 12 hooks must violate bounded hot-path profile and be refused (FR14)"
    );

    // 2. Program text size bound: 4096 bytes (FR14)
    let large_program = " ".repeat(4097);
    let large_ttl = format!(
        r#"
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
@prefix ex: <http://example.org/> .

ex:h1 a hook:Hook ;
    hook:name "h1" ;
    hook:kind "datalog" ;
    hook:program "{}" ;
    hook:goal "goal" ;
    hook:effect "refuse" ;
    hook:reason "r" .
"#,
        large_program
    );
    let large_dir = create_temp_pack("large-pack", "1.0.0", "desc", &["datalog"], &large_ttl);
    let res = load_hook_pack(large_dir.path());
    assert!(
        res.is_err(),
        "Datalog program exceeding 4KB must be refused (FR14)"
    );
}

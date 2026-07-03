//! Integration: the RDF-as-workflow front end drives the same derived chain
//! (plan -> topology -> geometry -> supervised execution) the hand-declared
//! domains do. The graph is the law: receipts are byte-identical across
//! runs of the same bytes, and every failure path is a typed refusal.

mod common;

use chatman_common::provenance::{fold_event, genesis_seed};
use common::lawobject_domain;
use praxis_synthesis::dag::RunOutcome;
use praxis_synthesis::{
    execute_workflow, replay_workflow, Refusal, SequenceProblem, Solver, Solver8,
};

/// The demo graph shipped with the crate.
const DEMO_TTL: &str = include_str!("../ontology/workflow_demo.ttl");

/// The 5-step lawobject domain from `tests/common`, re-expressed as a TTL
/// document instead of hand-declared Rust capabilities.
const LAWOBJECT_TTL: &str = r#"
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/lawobject/> .

ex:workflow a wf:Workflow ;
    wf:budget 5 ;
    wf:init ex:raw0 ;
    wf:goal ex:goal0 .

ex:raw0 a wf:Atom ; wf:predicate "raw" ; wf:arg0 "o1" .
ex:goal0 a wf:Atom ; wf:predicate "receipted" ; wf:arg0 "o1" .

ex:preRaw a wf:Atom ; wf:predicate "raw" ; wf:arg0 "?0" .
ex:aEvidence a wf:Atom ; wf:predicate "evidence" ; wf:arg0 "?0" .
ex:aClear a wf:Atom ; wf:predicate "clear" ; wf:arg0 "?0" .
ex:aValidated a wf:Atom ; wf:predicate "validated" ; wf:arg0 "?0" .
ex:aAdmitted a wf:Atom ; wf:predicate "admitted" ; wf:arg0 "?0" .
ex:aReceipted a wf:Atom ; wf:predicate "receipted" ; wf:arg0 "?0" .

ex:supply a wf:Capability ; wf:name "supply-evidence" ; wf:params 1 ; wf:cost 1 ;
    wf:pre ex:preRaw ; wf:add ex:aEvidence .
ex:clear a wf:Capability ; wf:name "clear-obligations" ; wf:params 1 ; wf:cost 1 ;
    wf:pre ex:aEvidence ; wf:add ex:aClear .
ex:judge a wf:Capability ; wf:name "judge" ; wf:params 1 ; wf:cost 1 ;
    wf:pre ex:aClear ; wf:add ex:aValidated .
ex:admit a wf:Capability ; wf:name "admit" ; wf:params 1 ; wf:cost 1 ;
    wf:pre ex:aValidated ; wf:add ex:aAdmitted .
ex:receipt a wf:Capability ; wf:name "receipt" ; wf:params 1 ; wf:cost 1 ;
    wf:pre ex:aAdmitted ; wf:add ex:aReceipted .
"#;

fn capability_order(steps: &[praxis_synthesis::BoundStep]) -> Vec<String> {
    steps.iter().map(|s| s.capability.clone()).collect()
}

#[test]
fn ttl_workflow_solves_the_same_plan_as_the_hand_declared_lawobject_domain() {
    // Hand-declared path: Rust-constructed program + capabilities.
    let (mut p, caps, goal) = lawobject_domain();
    p.saturate().expect("saturation");
    let problem =
        SequenceProblem::new(&p, caps, goal, 5, Vec::new()).expect("problem");
    let hand_plan = Solver8.solve(&problem).expect("hand-declared solves");

    // Graph path: the same domain as a TTL document.
    let receipt = execute_workflow(LAWOBJECT_TTL).expect("ttl workflow executes");

    assert_eq!(
        capability_order(&receipt.plan.steps),
        capability_order(&hand_plan.steps),
        "graph-defined and hand-declared domains derive the same plan"
    );
    assert_eq!(receipt.plan.cost, hand_plan.cost);
    assert_eq!(
        capability_order(&receipt.plan.steps),
        ["supply-evidence", "clear-obligations", "judge", "admit", "receipt"]
    );
    assert_eq!(receipt.supervised.outcome, RunOutcome::Completed);
    assert!(receipt.supervised.geometry_conformance);
}

#[test]
fn same_ttl_bytes_yield_byte_identical_receipt() {
    let a = execute_workflow(LAWOBJECT_TTL).expect("first run");
    let b = execute_workflow(LAWOBJECT_TTL).expect("second run");
    assert_eq!(a, b, "receipts are structurally identical");
    let ja = serde_json::to_string(&a).expect("serialize a");
    let jb = serde_json::to_string(&b).expect("serialize b");
    assert_eq!(ja, jb, "receipts are byte-identical over serde");
}

#[test]
fn unreachable_goal_is_a_typed_refusal_not_a_panic() {
    // Same capabilities, but the goal names a predicate nothing adds.
    let ttl = LAWOBJECT_TTL
        .replace("wf:predicate \"receipted\" ; wf:arg0 \"o1\"", "wf:predicate \"notarized\" ; wf:arg0 \"o1\"");
    let err = execute_workflow(&ttl).expect_err("unreachable goal refuses");
    assert!(
        matches!(err, Refusal::Unsatisfiable { .. } | Refusal::UnsatProof { .. }),
        "expected a solver refusal, got: {err:?}"
    );
}

#[test]
fn demo_workflow_solves_golden_plan() {
    let receipt = execute_workflow(DEMO_TTL).expect("demo executes");
    assert_eq!(
        capability_order(&receipt.plan.steps),
        ["gather", "verify", "receipt"]
    );
    assert_eq!(receipt.plan.cost, 3);
    assert_eq!(receipt.supervised.outcome, RunOutcome::Completed);
    // Golden hash: the demo document's derived plan hash. If this moves,
    // either the demo bytes changed or determinism broke — both are news.
    assert_eq!(receipt.plan_hash, DEMO_PLAN_HASH_GOLDEN);
}

/// Recorded from a prior run of `execute_workflow(DEMO_TTL)`; locks the demo
/// plan derivation byte-for-byte.
const DEMO_PLAN_HASH_GOLDEN: &str =
    "9c4ae4c6e6e4137ebce6ac9c78c127b5af0cf00a51105a9f87a14952d21a4a92";

#[test]
fn reformat_same_chain_different_ttl_hash() {
    // Semantically identical to DEMO_TTL: comments stripped, statements
    // reordered, predicate lists flattened, whitespace mangled.
    let reformatted = demo_reordered();
    let a = execute_workflow(DEMO_TTL).expect("original executes");
    let b = execute_workflow(&reformatted).expect("reformat executes");
    assert_ne!(a.ttl_hash, b.ttl_hash, "different bytes are still nameable");
    assert_eq!(a.graph_hash, b.graph_hash, "same triples, same law");
    assert_eq!(a.chain, b.chain, "ttl_hash is never folded into the chain");
    assert_eq!(a.plan_hash, b.plan_hash);
}

/// The demo graph with statements reordered, `;`-lists split into separate
/// statements, comments changed, and whitespace mangled — same triples.
fn demo_reordered() -> String {
    let mut stmts: Vec<&str> = vec![
        "ex:orderGatherFirst wf:b \"receipt\" .",
        "ex:orderGatherFirst wf:a \"gather\" .",
        "ex:orderGatherFirst wf:kind \"before\" .",
        "ex:orderGatherFirst a wf:Constraint .",
        "ex:addReceipted wf:arg0 \"?0\" .",
        "ex:addReceipted wf:predicate \"receipted\" .",
        "ex:addReceipted a wf:Atom .",
        "ex:receipt wf:add ex:addReceipted .",
        "ex:receipt wf:pre ex:addVerified .",
        "ex:receipt wf:cost 1 .",
        "ex:receipt wf:params 1 .",
        "ex:receipt wf:name \"receipt\" .",
        "ex:receipt a wf:Capability .",
        "ex:addVerified wf:arg0 \"?0\" .",
        "ex:addVerified wf:predicate \"verified\" .",
        "ex:addVerified a wf:Atom .",
        "ex:verify wf:add ex:addVerified .",
        "ex:verify wf:pre ex:addEvidence .",
        "ex:verify wf:cost 1 .",
        "ex:verify wf:params 1 .",
        "ex:verify wf:name \"verify\" .",
        "ex:verify a wf:Capability .",
        "ex:addEvidence wf:arg0 \"?0\" .",
        "ex:addEvidence wf:predicate \"evidence\" .",
        "ex:addEvidence a wf:Atom .",
        "ex:preRaw wf:arg0 \"?0\" .",
        "ex:preRaw wf:predicate \"raw\" .",
        "ex:preRaw a wf:Atom .",
        "ex:gather wf:add ex:addEvidence .",
        "ex:gather wf:pre ex:preRaw .",
        "ex:gather wf:cost 1 .",
        "ex:gather wf:params 1 .",
        "ex:gather wf:name \"gather\" .",
        "ex:gather a wf:Capability .",
        "ex:goal0 wf:arg0 \"doc\" .",
        "ex:goal0 wf:predicate \"receipted\" .",
        "ex:goal0 a wf:Atom .",
        "ex:raw0 wf:arg0 \"doc\" .",
        "ex:raw0 wf:predicate \"raw\" .",
        "ex:raw0 a wf:Atom .",
        "ex:workflow wf:goal ex:goal0 .",
        "ex:workflow wf:init ex:raw0 .",
        "ex:workflow wf:budget 3 .",
        "ex:workflow a wf:Workflow .",
    ];
    stmts.reverse();
    let mut doc = String::from(
        "# reordered surface form, same graph\n\
         @prefix ex: <http://example.org/pipeline/> .\n\
         @prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .\n",
    );
    for (i, s) in stmts.iter().enumerate() {
        // Vary whitespace deliberately.
        if i % 3 == 0 {
            doc.push_str("   ");
        }
        doc.push_str(s);
        doc.push_str(if i % 2 == 0 { "\n" } else { "\n\n" });
    }
    doc
}

#[test]
fn determinism_across_whitespace_comment_and_ordering_variants() {
    let variants: [String; 4] = [
        DEMO_TTL.to_string(),
        demo_reordered(),
        // Comment-noise variant: every blank line becomes a comment.
        DEMO_TTL
            .lines()
            .map(|l| {
                if l.trim().is_empty() {
                    "# noise".to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        // Tab-indentation variant.
        DEMO_TTL.replace("    ", "\t"),
    ];
    let receipts: Vec<_> = variants
        .iter()
        .map(|v| execute_workflow(v).expect("variant executes"))
        .collect();
    for r in &receipts[1..] {
        assert_eq!(r.graph_hash, receipts[0].graph_hash);
        assert_eq!(r.ir_hash, receipts[0].ir_hash);
        assert_eq!(r.plan_hash, receipts[0].plan_hash);
        assert_eq!(r.chain, receipts[0].chain);
    }
}

#[test]
fn mutating_one_cost_changes_graph_ir_and_chain() {
    let mutated = DEMO_TTL.replace(
        "wf:name \"verify\" ;\n    wf:params 1 ;\n    wf:cost 1",
        "wf:name \"verify\" ;\n    wf:params 1 ;\n    wf:cost 2",
    );
    assert_ne!(mutated, DEMO_TTL, "mutation applied");
    let a = execute_workflow(DEMO_TTL).expect("original");
    let b = execute_workflow(&mutated).expect("mutated");
    assert_ne!(a.graph_hash, b.graph_hash);
    assert_ne!(a.ir_hash, b.ir_hash);
    // plan_hash covers the bound step sequence only; the optimal order is
    // unchanged here, so plan_hash legitimately stays equal — the mutation
    // is still caught upstream (graph/ir) and therefore in the chain.
    assert_eq!(a.plan_hash, b.plan_hash, "same step sequence, same plan hash");
    assert_ne!(a.chain, b.chain, "chain folds graph_hash first, so it moves");
    assert_eq!(b.plan.cost, 4, "one capability now costs 2");
}

#[test]
fn receipt_chain_folds_graph_hash_first() {
    let r = execute_workflow(DEMO_TTL).expect("demo executes");
    let mut chain = genesis_seed("praxis:workflow:v1");
    chain = fold_event(&chain, r.graph_hash.as_bytes());
    chain = fold_event(&chain, r.ir_hash.as_bytes());
    chain = fold_event(&chain, r.plan_hash.as_bytes());
    chain = fold_event(&chain, r.topology_hash.as_bytes());
    chain = fold_event(&chain, r.geometry_hash.as_bytes());
    chain = fold_event(&chain, r.exec_hash.as_bytes());
    assert_eq!(chain, r.chain, "hand-refolded chain matches the receipt");
}

#[test]
fn replay_passes_on_honest_receipt() {
    let r = execute_workflow(DEMO_TTL).expect("demo executes");
    replay_workflow(&r, DEMO_TTL).expect("honest receipt replays");
    // A reformat of the same triples also replays: ttl_hash is not folded.
    replay_workflow(&r, &demo_reordered()).expect("reformat replays");
}

#[test]
fn replay_detects_forged_receipt() {
    let mut r = execute_workflow(DEMO_TTL).expect("demo executes");
    r.plan_hash = format!("{}0", &r.plan_hash[..r.plan_hash.len() - 1]);
    let err = replay_workflow(&r, DEMO_TTL).expect_err("forged plan_hash refuses");
    match err {
        Refusal::VerificationFailed { failed } => {
            assert_eq!(failed, vec!["plan_hash".to_string()])
        }
        other => panic!("expected VerificationFailed, got: {other:?}"),
    }
}

#[test]
fn replay_detects_forged_payload_behind_honest_hashes() {
    // The adversarial-review finding: honest hash fields, forged embedded
    // body. Replay must bind the payloads to the hashes it verifies.
    let mut r = execute_workflow(DEMO_TTL).expect("demo executes");
    r.supervised.dispositions.clear(); // forge the body, leave hashes alone
    let err =
        replay_workflow(&r, DEMO_TTL).expect_err("forged supervised body refuses");
    match err {
        Refusal::VerificationFailed { failed } => {
            assert_eq!(failed, vec!["supervised payload".to_string()])
        }
        other => panic!("expected VerificationFailed, got: {other:?}"),
    }

    let mut r2 = execute_workflow(DEMO_TTL).expect("demo executes");
    r2.plan.receipt.plan_hash = format!("{}0", &r2.plan.receipt.plan_hash[..63]);
    let err2 = replay_workflow(&r2, DEMO_TTL).expect_err("forged plan body refuses");
    match err2 {
        Refusal::VerificationFailed { failed } => {
            assert_eq!(failed, vec!["plan payload".to_string()])
        }
        other => panic!("expected VerificationFailed, got: {other:?}"),
    }
}

#[test]
fn adversarial_malformed_ttl_sweep_refuses_without_panicking() {
    // Each entry: (document, human name of the sin). Every one must yield a
    // typed Refusal — never a panic, never a silent success.
    let cases: &[(&str, &str)] = &[
        ("ex:a ex:b ex:c .", "undeclared prefix"),
        ("@prefix ex: <http://x/> .\nex:a ex:b \"unterminated", "unterminated string"),
        ("@prefix ex: <http://x/> .\nex:a ex:b ex:c", "missing final dot"),
        ("@prefix ex: <http://x/> .\nex:a ex:b [] .", "blank node"),
        ("@prefix ex: <http://x/> .\n_:b ex:p ex:o .", "blank node subject"),
        ("@prefix ex: <http://x/> .\nex:a ex:b (1 2) .", "collection"),
        ("@prefix ex: <http://x/> .\nex:a ex:b \"x\"@en .", "language tag"),
        ("@prefix ex: <http://x/> .\nex:a ex:b \"1\"^^ex:int .", "datatype"),
        ("@base <http://x/> .\n<a> <b> <c> .", "@base"),
        ("@prefix ex: <http://x/> .\nex:a ex:b 1.5 .", "decimal"),
        ("@prefix ex: <http://x/> .\nex:a ex:b true .", "boolean"),
        ("@prefix ex: <http://x/> .\nex:a ex:b \"\"\"multi\"\"\" .", "long string"),
        ("@prefix ex: <http://x/> .\nex:a ex:b 99999999999999999999 .", "i64 overflow"),
        ("@prefix ex: <http://x/> .\nex:a ex:b \"bad\u{0007}\" .", "control char in literal"),
        ("@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .\n@prefix ex: <http://x/> .\nex:w a wf:Workflow ; wf:specHash \"deadbeef\" ; wf:budget 1 ; wf:goal ex:g .\nex:g a wf:Atom ; wf:predicate \"p\" .", "asserted spec hash"),
        ("@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .\n@prefix ex: <http://x/> .\nex:w a wf:Workflow ; wf:budget 9 ; wf:goal ex:g .\nex:g a wf:Atom ; wf:predicate \"p\" .", "budget over 8"),
        ("@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .", "no workflow node"),
    ];
    for (doc, name) in cases {
        let err = execute_workflow(doc).expect_err(name);
        // Any Refusal variant is lawful; a panic would have aborted the test.
        let rendered = format!("{err}");
        assert!(!rendered.is_empty(), "refusal for {name} renders");
    }
}

#[test]
fn malformed_document_is_a_typed_refusal_naming_the_culprit() {
    let err = execute_workflow("@prefix wf: <http://x/> .\nex:a wf:b [] .")
        .expect_err("blank node refuses");
    match err {
        Refusal::GraphMalformed { line, .. } => assert_eq!(line, 2),
        other => panic!("expected GraphMalformed, got: {other:?}"),
    }
}

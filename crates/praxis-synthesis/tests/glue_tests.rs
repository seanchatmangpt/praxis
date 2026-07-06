//! Cross-graph gluing: workflow fragments compose by sorted-set union under
//! the glue law (overlapping shared IRIs must agree on functional `wf:`
//! predicates), the composition is merge-order free (the cocycle condition),
//! and the merged graph executes through the untouched single-document
//! pipeline — composition is invisible downstream.

// The deprecated execute_workflow surface stays covered until removal.
#![allow(deprecated)]
use std::path::{Path, PathBuf};

use praxis_synthesis::glue::{compose_workflows, execute_composed};
use praxis_synthesis::graph::{MAX_TRIPLES, WF_NS};
use praxis_synthesis::{execute_workflow, Refusal};

const PART_A: &str = include_str!("../ontology/lawobject_part_a.ttl");
const PART_B: &str = include_str!("../ontology/lawobject_part_b.ttl");
const SINGLE: &str = include_str!("../ontology/lawobject_single.ttl");

fn step_order(receipt: &praxis_synthesis::WorkflowReceipt) -> Vec<String> {
    receipt.plan.steps.iter().map(|s| s.capability.clone()).collect()
}

#[test]
fn composed_execution_equals_the_single_file_workflow() {
    let composed = execute_composed(&[PART_A, PART_B]).expect("composed executes");
    let single = execute_workflow(SINGLE).expect("single-file executes");

    assert_eq!(composed.workflow.graph_hash, single.graph_hash);
    assert_eq!(composed.merged_graph_hash, single.graph_hash);
    assert_eq!(composed.workflow.ir_hash, single.ir_hash);
    assert_eq!(composed.workflow.plan_hash, single.plan_hash);
    assert_eq!(
        step_order(&composed.workflow),
        ["supply-evidence", "clear-obligations", "judge", "admit", "receipt"]
    );
    assert_eq!(step_order(&composed.workflow), step_order(&single));
    assert_eq!(composed.workflow.chain, single.chain);
}

/// Three-way split of the single-file workflow, each fragment parseable alone.
const P1: &str = r#"
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/lawobject/> .
ex:workflow a wf:Workflow ; wf:budget 5 ; wf:init ex:raw0 ; wf:goal ex:goal0 .
ex:raw0 a wf:Atom ; wf:predicate "raw" ; wf:arg0 "o1" .
ex:goal0 a wf:Atom ; wf:predicate "receipted" ; wf:arg0 "o1" .
"#;
const P2: &str = r#"
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/lawobject/> .
ex:preRaw a wf:Atom ; wf:predicate "raw" ; wf:arg0 "?0" .
ex:aEvidence a wf:Atom ; wf:predicate "evidence" ; wf:arg0 "?0" .
ex:aClear a wf:Atom ; wf:predicate "clear" ; wf:arg0 "?0" .
ex:aValidated a wf:Atom ; wf:predicate "validated" ; wf:arg0 "?0" .
ex:supply a wf:Capability ; wf:name "supply-evidence" ; wf:params 1 ; wf:cost 1 ;
    wf:pre ex:preRaw ; wf:add ex:aEvidence .
ex:clear a wf:Capability ; wf:name "clear-obligations" ; wf:params 1 ; wf:cost 1 ;
    wf:pre ex:aEvidence ; wf:add ex:aClear .
ex:judge a wf:Capability ; wf:name "judge" ; wf:params 1 ; wf:cost 1 ;
    wf:pre ex:aClear ; wf:add ex:aValidated .
"#;
const P3: &str = r#"
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/lawobject/> .
ex:aValidated a wf:Atom ; wf:predicate "validated" ; wf:arg0 "?0" .
ex:aAdmitted a wf:Atom ; wf:predicate "admitted" ; wf:arg0 "?0" .
ex:aReceipted a wf:Atom ; wf:predicate "receipted" ; wf:arg0 "?0" .
ex:admit a wf:Capability ; wf:name "admit" ; wf:params 1 ; wf:cost 1 ;
    wf:pre ex:aValidated ; wf:add ex:aAdmitted .
ex:receipt a wf:Capability ; wf:name "receipt" ; wf:params 1 ; wf:cost 1 ;
    wf:pre ex:aAdmitted ; wf:add ex:aReceipted .
"#;

#[test]
fn merge_order_does_not_matter() {
    let perms: [[&str; 3]; 6] = [
        [P1, P2, P3],
        [P1, P3, P2],
        [P2, P1, P3],
        [P2, P3, P1],
        [P3, P1, P2],
        [P3, P2, P1],
    ];
    let reference = compose_workflows(&perms[0]).expect("compose");
    let reference_json = serde_json::to_string(&reference).expect("json");
    let reference_receipt = execute_composed(&perms[0]).expect("execute");
    for perm in &perms[1..] {
        let composed = compose_workflows(perm).expect("compose");
        assert_eq!(
            serde_json::to_string(&composed).expect("json"),
            reference_json,
            "composition must be byte-identical under any merge order"
        );
        let receipt = execute_composed(perm).expect("execute");
        assert_eq!(receipt.sections, reference_receipt.sections);
        assert_eq!(receipt.workflow.chain, reference_receipt.workflow.chain);
    }

    // Associativity (the cocycle test): the canonical form of a composition
    // is itself a constituent, and nesting cannot change the merged hash.
    let ab = compose_workflows(&[P1, P2]).expect("compose ab");
    let bc = compose_workflows(&[P2, P3]).expect("compose bc");
    let ab_c = compose_workflows(&[&ab.canonical_ttl, P3]).expect("compose (ab)c");
    let a_bc = compose_workflows(&[P1, &bc.canonical_ttl]).expect("compose a(bc)");
    assert_eq!(ab_c.merged_graph_hash, reference.merged_graph_hash);
    assert_eq!(a_bc.merged_graph_hash, reference.merged_graph_hash);
}

#[test]
fn glue_conflict_names_subject_predicate_and_both_values() {
    // Two graphs both declaring ex:admit, disagreeing on wf:cost (1 vs 3).
    let left = r#"
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/lawobject/> .
ex:admit a wf:Capability ; wf:name "admit" ; wf:cost 1 .
"#;
    let right = r#"
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/lawobject/> .
ex:admit a wf:Capability ; wf:name "admit" ; wf:cost 3 .
"#;
    let err = compose_workflows(&[left, right]).expect_err("cost conflict refuses");
    match err {
        Refusal::GlueConflict {
            subject,
            predicate,
            values,
        } => {
            assert_eq!(subject, "http://example.org/lawobject/admit");
            assert_eq!(predicate, format!("{WF_NS}cost"));
            assert_eq!(values, vec!["1".to_string(), "3".to_string()]);
        }
        other => panic!("expected GlueConflict, got {other:?}"),
    }

    // Two wf:Workflow fragments on the same subject disagreeing on wf:budget.
    let wf5 = r#"
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/lawobject/> .
ex:workflow a wf:Workflow ; wf:budget 5 .
"#;
    let wf6 = r#"
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/lawobject/> .
ex:workflow a wf:Workflow ; wf:budget 6 .
"#;
    let err = compose_workflows(&[wf5, wf6]).expect_err("budget conflict refuses");
    match err {
        Refusal::GlueConflict {
            subject,
            predicate,
            values,
        } => {
            assert_eq!(subject, "http://example.org/lawobject/workflow");
            assert_eq!(predicate, format!("{WF_NS}budget"));
            assert_eq!(values, vec!["5".to_string(), "6".to_string()]);
        }
        other => panic!("expected GlueConflict, got {other:?}"),
    }
}

#[test]
fn overlap_agreement_is_lawful() {
    // ex:aValidated is declared identically in both parts: agreement, not
    // conflict — and the union dedups it to one canonical declaration.
    let composed = compose_workflows(&[PART_A, PART_B]).expect("overlap agrees");
    let needle = format!(
        "<http://example.org/lawobject/aValidated> <{WF_NS}predicate> \"validated\" ."
    );
    let count = composed
        .canonical_ttl
        .lines()
        .filter(|l| *l == needle)
        .count();
    assert_eq!(count, 1, "boundary atom dedups to one canonical line");
}

#[test]
fn duplicate_workflow_nodes_still_refused_downstream() {
    // Two DISTINCT wf:Workflow subjects glue fine (no shared IRI), but the
    // merged graph still violates the single-workflow shape law downstream —
    // gluing does not weaken it.
    let one = r#"
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/lawobject/> .
ex:wfOne a wf:Workflow ; wf:budget 5 .
"#;
    let two = r#"
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/lawobject/> .
ex:wfTwo a wf:Workflow ; wf:budget 5 .
"#;
    compose_workflows(&[one, two]).expect("distinct subjects glue fine");
    let err = execute_composed(&[one, two]).expect_err("two workflows refuse downstream");
    match err {
        Refusal::WorkflowIllFormed { detail, .. } => {
            assert!(
                detail.contains("2 wf:Workflow nodes"),
                "expected the single-workflow shape law, got: {detail}"
            );
        }
        other => panic!("expected WorkflowIllFormed, got {other:?}"),
    }
}

#[test]
fn merged_caps_enforced() {
    // Each constituent is individually lawful (2,500 triples, well under the
    // 4,096 cap); their union is 5,000 — over the cap, refused, not truncated.
    let mut left = String::new();
    let mut right = String::new();
    for i in 0..2_500 {
        left.push_str(&format!("<u:a{i}> <u:p> {i} .\n"));
        right.push_str(&format!("<u:b{i}> <u:p> {i} .\n"));
    }
    let err = compose_workflows(&[&left, &right]).expect_err("union over cap refuses");
    match err {
        Refusal::GraphCapExceeded { what, cap, actual } => {
            assert_eq!(what, "triples");
            assert_eq!(cap, MAX_TRIPLES as u64);
            assert_eq!(actual, 5_000);
        }
        other => panic!("expected GraphCapExceeded, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// foreign verifier — composition is invisible to a second implementation
// ---------------------------------------------------------------------------

fn b3sum_available() -> bool {
    std::process::Command::new("b3sum")
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "praxis-synth-glue-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

fn run_verifier(ttl: &Path, receipt: &Path) -> std::process::Output {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/foreign_verify_graph.py"
    );
    std::process::Command::new("python3")
        .args([
            script,
            "graph",
            ttl.to_str().expect("utf8 path"),
            receipt.to_str().expect("utf8 path"),
        ])
        .output()
        .expect("run foreign graph verifier")
}

#[test]
fn foreign_verifier_accepts_a_composed_receipt() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH — foreign verification needs it");
        return;
    }
    let composed = execute_composed(&[PART_A, PART_B]).expect("composed executes");
    let ttl_path = temp_path("composed.ttl");
    let receipt_path = temp_path("composed-receipt.json");
    std::fs::write(&ttl_path, composed_canonical_bytes(&composed)).expect("write ttl");
    std::fs::write(
        &receipt_path,
        serde_json::to_string(&composed.workflow).expect("json"),
    )
    .expect("write receipt");
    let out = run_verifier(&ttl_path, &receipt_path);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "composed receipt must verify foreign: {stdout} {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("VERIFIED graph"), "{stdout}");
    let _ = std::fs::remove_file(&ttl_path);
    let _ = std::fs::remove_file(&receipt_path);
}

/// The merged canonical form of the composition — recomposed here so the
/// foreign test feeds exactly the document the receipt was derived from.
fn composed_canonical_bytes(
    receipt: &praxis_synthesis::glue::ComposedWorkflowReceipt,
) -> Vec<u8> {
    let composed = compose_workflows(&[PART_A, PART_B]).expect("recompose");
    assert_eq!(composed.merged_graph_hash, receipt.merged_graph_hash);
    composed.canonical_ttl.clone().into_bytes()
}

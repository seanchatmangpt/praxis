//! The foreign graph verifier, tested: a second implementation in another
//! language (`scripts/foreign_verify_graph.py`, python3 + `b3sum`) must agree
//! with the Rust crate on an honest workflow receipt, agree across a lawful
//! reformat, and name the first divergent stage on a tampered one.

// The deprecated execute_workflow surface stays covered until removal.
#![allow(deprecated)]
use std::path::{Path, PathBuf};

use praxis_synthesis::execute_workflow;

const DEMO_TTL: &str = include_str!("../ontology/workflow_demo.ttl");

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
        "praxis-synth-graph-{}-{}-{name}",
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
fn foreign_graph_verifier_agrees_on_an_honest_receipt() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH — foreign verification needs it");
        return;
    }
    let receipt = execute_workflow(DEMO_TTL).expect("demo executes");
    let ttl_path = temp_path("honest.ttl");
    let receipt_path = temp_path("honest-receipt.json");
    std::fs::write(&ttl_path, DEMO_TTL.as_bytes()).expect("write ttl");
    std::fs::write(
        &receipt_path,
        serde_json::to_string(&receipt).expect("json"),
    )
    .expect("write receipt");
    let out = run_verifier(&ttl_path, &receipt_path);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "foreign graph verifier disagreed: {stdout} {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("VERIFIED graph"), "{stdout}");
    let _ = std::fs::remove_file(&ttl_path);
    let _ = std::fs::remove_file(&receipt_path);
}

#[test]
fn foreign_graph_verifier_agrees_across_a_reformat() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH — foreign verification needs it");
        return;
    }
    let receipt = execute_workflow(DEMO_TTL).expect("demo executes");
    // Same triples, different bytes: canonicalization, not bytes, is the law —
    // proven here across two implementations.
    let mangled = DEMO_TTL.replace("    ", "\t");
    assert_ne!(mangled, DEMO_TTL, "mangle must change the bytes");
    let ttl_path = temp_path("reformat.ttl");
    let receipt_path = temp_path("reformat-receipt.json");
    std::fs::write(&ttl_path, mangled.as_bytes()).expect("write ttl");
    std::fs::write(
        &receipt_path,
        serde_json::to_string(&receipt).expect("json"),
    )
    .expect("write receipt");
    let out = run_verifier(&ttl_path, &receipt_path);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "reformatted TTL must still verify: {stdout} {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("VERIFIED graph"), "{stdout}");
    let _ = std::fs::remove_file(&ttl_path);
    let _ = std::fs::remove_file(&receipt_path);
}

/// Flip the last hex character of a lowercase hex hash string.
fn flip_last_hex(h: &str) -> String {
    let mut s: Vec<u8> = h.bytes().collect();
    let last = s.last_mut().expect("nonempty hash");
    *last = if *last == b'0' { b'1' } else { b'0' };
    String::from_utf8(s).expect("ascii hex")
}

#[test]
fn foreign_graph_verifier_names_the_first_divergent_stage() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH — foreign verification needs it");
        return;
    }
    let honest = execute_workflow(DEMO_TTL).expect("demo executes");
    let ttl_path = temp_path("tamper.ttl");
    std::fs::write(&ttl_path, DEMO_TTL.as_bytes()).expect("write ttl");

    let expect_mismatch =
        |receipt: &praxis_synthesis::graph::WorkflowReceipt, stage: &str, name: &str| {
            let receipt_path = temp_path(name);
            std::fs::write(&receipt_path, serde_json::to_string(receipt).expect("json"))
                .expect("write receipt");
            let out = run_verifier(&ttl_path, &receipt_path);
            let stdout = String::from_utf8_lossy(&out.stdout);
            assert!(
                !out.status.success(),
                "tampered receipt ({stage}) must fail: {stdout}"
            );
            assert!(
                stdout.contains(&format!("MISMATCH: {stage}")),
                "expected first divergent stage '{stage}' named, got: {stdout}"
            );
            let _ = std::fs::remove_file(&receipt_path);
        };

    // Tamper 1: forged graph_hash — caught at the first stage.
    let mut t1 = honest.clone();
    t1.graph_hash = flip_last_hex(&t1.graph_hash);
    expect_mismatch(&t1, "graph_hash", "tamper-graph.json");

    // Tamper 2: forged ir_hash — the IR is now re-derived from the triples,
    // so the forgery is named at its own stage (was only "chain" before).
    let mut t2 = honest.clone();
    t2.ir_hash = flip_last_hex(&t2.ir_hash);
    expect_mismatch(&t2, "ir_hash", "tamper-ir.json");

    // Tamper 3: forged supervised body, hashes untouched — the payload
    // binding catches a receipt whose hash fields are honest but whose
    // embedded body is not.
    let mut t3 = honest.clone();
    t3.supervised.dispositions.clear();
    expect_mismatch(&t3, "exec payload", "tamper-exec.json");

    let _ = std::fs::remove_file(&ttl_path);
}

/// The forgery refold-as-claimed could not name: mutate the document in an
/// IR-affecting way, execute it honestly, then overwrite `ir_hash` (and the
/// receipt's other fields stay consistent with the *mutated* document except
/// the swapped-in original `ir_hash`). The graph stage passes — the receipt
/// matches the mutated document — but independent IR re-derivation exposes
/// the swapped hash at its own stage.
#[test]
fn foreign_verifier_rederives_ir_catching_a_graph_consistent_forgery() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH — foreign verification needs it");
        return;
    }
    let original = execute_workflow(DEMO_TTL).expect("demo executes");
    let mutated_ttl = DEMO_TTL.replace(
        "wf:cost 1 ;\n    wf:pre ex:preRaw",
        "wf:cost 2 ;\n    wf:pre ex:preRaw",
    );
    assert_ne!(
        mutated_ttl, DEMO_TTL,
        "mutation must change ex:gather's cost"
    );
    let mutated = execute_workflow(&mutated_ttl).expect("mutated demo executes");
    assert_ne!(
        mutated.ir_hash, original.ir_hash,
        "cost change must reach the IR"
    );

    // Forge: the mutated document's receipt, claiming the original ir_hash.
    let mut forged = mutated.clone();
    forged.ir_hash = original.ir_hash.clone();

    let ttl_path = temp_path("ir-forgery.ttl");
    let receipt_path = temp_path("ir-forgery-receipt.json");
    std::fs::write(&ttl_path, mutated_ttl.as_bytes()).expect("write ttl");
    std::fs::write(&receipt_path, serde_json::to_string(&forged).expect("json"))
        .expect("write receipt");
    let out = run_verifier(&ttl_path, &receipt_path);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !out.status.success(),
        "graph-consistent IR forgery must fail: {stdout}"
    );
    assert!(
        stdout.contains("MISMATCH: ir_hash"),
        "IR re-derivation must name the forged stage, got: {stdout}"
    );
    let _ = std::fs::remove_file(&ttl_path);
    let _ = std::fs::remove_file(&receipt_path);
}

/// Pin the null-vs-omitted and sorting rules across the two implementations:
/// a workflow exercising every optional field shape — a `not-later`
/// constraint with `wf:k` (a/`k` set, `b` = JSON null), a capability with
/// `wf:del`, and a multi-arg atom — must verify foreign end to end.
#[test]
fn foreign_verifier_ir_hash_agrees_on_a_constraint_bearing_workflow() {
    if !b3sum_available() {
        eprintln!("SKIPPED (deferred): b3sum not on PATH — foreign verification needs it");
        return;
    }
    const SHAPES_TTL: &str = r#"
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/shapes/> .

ex:workflow a wf:Workflow ;
    wf:budget 2 ;
    wf:init ex:raw0 ;
    wf:goal ex:goal0 .

ex:raw0 a wf:Atom ;
    wf:predicate "raw" ;
    wf:arg0 "doc" ;
    wf:arg1 "src" .

ex:goal0 a wf:Atom ;
    wf:predicate "receipted" ;
    wf:arg0 "doc" .

ex:gather a wf:Capability ;
    wf:name "gather" ;
    wf:params 2 ;
    wf:cost 1 ;
    wf:pre ex:preRaw ;
    wf:add ex:addEvidence ;
    wf:del ex:preRaw .

ex:preRaw a wf:Atom ;
    wf:predicate "raw" ;
    wf:arg0 "?0" ;
    wf:arg1 "?1" .

ex:addEvidence a wf:Atom ;
    wf:predicate "evidence" ;
    wf:arg0 "?0" .

ex:receipt a wf:Capability ;
    wf:name "receipt" ;
    wf:params 1 ;
    wf:cost 1 ;
    wf:pre ex:addEvidence ;
    wf:add ex:addReceipted .

ex:addReceipted a wf:Atom ;
    wf:predicate "receipted" ;
    wf:arg0 "?0" .

ex:gatherEarly a wf:Constraint ;
    wf:kind "not-later" ;
    wf:a "gather" ;
    wf:k 1 .
"#;
    let receipt = execute_workflow(SHAPES_TTL).expect("shapes workflow executes");
    let ttl_path = temp_path("shapes.ttl");
    let receipt_path = temp_path("shapes-receipt.json");
    std::fs::write(&ttl_path, SHAPES_TTL.as_bytes()).expect("write ttl");
    std::fs::write(
        &receipt_path,
        serde_json::to_string(&receipt).expect("json"),
    )
    .expect("write receipt");
    let out = run_verifier(&ttl_path, &receipt_path);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "constraint-bearing workflow must verify foreign: {stdout} {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("VERIFIED graph"), "{stdout}");
    let _ = std::fs::remove_file(&ttl_path);
    let _ = std::fs::remove_file(&receipt_path);
}

//! The foreign graph verifier, tested: a second implementation in another
//! language (`scripts/foreign_verify_graph.py`, python3 + `b3sum`) must agree
//! with the Rust crate on an honest workflow receipt, agree across a lawful
//! reformat, and name the first divergent stage on a tampered one.

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
    std::fs::write(&receipt_path, serde_json::to_string(&receipt).expect("json"))
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
    std::fs::write(&receipt_path, serde_json::to_string(&receipt).expect("json"))
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

    let expect_mismatch = |receipt: &praxis_synthesis::graph::WorkflowReceipt,
                           stage: &str,
                           name: &str| {
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

    // Tamper 2: forged ir_hash — graph_hash still honest, chain refold breaks.
    let mut t2 = honest.clone();
    t2.ir_hash = flip_last_hex(&t2.ir_hash);
    expect_mismatch(&t2, "chain", "tamper-ir.json");

    // Tamper 3: forged supervised body, hashes untouched — the payload
    // binding catches a receipt whose hash fields are honest but whose
    // embedded body is not.
    let mut t3 = honest.clone();
    t3.supervised.dispositions.clear();
    expect_mismatch(&t3, "exec payload", "tamper-exec.json");

    let _ = std::fs::remove_file(&ttl_path);
}

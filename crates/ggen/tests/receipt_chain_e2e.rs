//! Chicago-TDD proofs for cross-sync receipt chaining: real syncs on a real
//! filesystem, real BLAKE3 chain recomputation via praxis-core, and the real
//! `ggen receipt history` binary at the CLI boundary. No mocks.

use std::path::Path;

use chicago_tdd_tools::cli_proof::CliHarness;
use ggen::sync::{sync, SyncOptions, SyncReceipt, RECEIPT_LOG_REL_PATH, RECEIPT_REL_PATH};
use tempfile::TempDir;

const GGEN_TOML: &str = r#"
[project]
name = "demo"

[ontology]
source = "ontology.ttl"

[templates]
dir = "templates"
"#;

const TEMPLATE: &str = "---\nto: out/names.txt\nforce: true\nsparql:\n  people: SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name\n---\n{% for row in results %}{{ row.name }}\n{% endfor %}";

fn scaffold(root: &Path, names: &[&str]) {
    std::fs::write(root.join("ggen.toml"), GGEN_TOML).expect("write ggen.toml");
    write_ontology(root, names);
    std::fs::create_dir_all(root.join("templates")).expect("mkdir templates");
    std::fs::write(root.join("templates/one.tmpl"), TEMPLATE).expect("write template");
}

fn write_ontology(root: &Path, names: &[&str]) {
    let mut ttl = String::from("@prefix ex: <http://example.org/> .\n");
    for name in names {
        ttl.push_str(&format!("ex:{name} ex:name \"{name}\" .\n"));
    }
    std::fs::write(root.join("ontology.ttl"), ttl).expect("write ontology");
}

fn read_log(root: &Path) -> Vec<SyncReceipt> {
    let raw = std::fs::read_to_string(root.join(RECEIPT_LOG_REL_PATH)).expect("read log");
    raw.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse log line"))
        .collect()
}

/// Three syncs with evolving ontologies form a genesis-rooted 3-link chain,
/// receipt.json stays the single-receipt head, and `receipt history` passes.
#[test]
fn three_syncs_form_a_verifiable_chain() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path(), &["alice"]);
    sync(dir.path(), SyncOptions { dry_run: false }).expect("sync 1");
    write_ontology(dir.path(), &["alice", "bob"]);
    sync(dir.path(), SyncOptions { dry_run: false }).expect("sync 2");
    write_ontology(dir.path(), &["alice", "bob", "carol"]);
    sync(dir.path(), SyncOptions { dry_run: false }).expect("sync 3");

    let log = read_log(dir.path());
    assert_eq!(log.len(), 3, "three syncs must append three log lines");

    // Genesis root, then each prev links to the prior chain hash.
    assert_eq!(log[0].record.prev_chain_hash_hex, "0".repeat(64));
    assert_eq!(log[1].record.prev_chain_hash_hex, log[0].record.chain_hash_hex);
    assert_eq!(log[2].record.prev_chain_hash_hex, log[1].record.chain_hash_hex);
    // Content changed each run, so payload hashes differ.
    assert_ne!(log[0].record.payload_hash_hex, log[1].record.payload_hash_hex);
    assert_ne!(log[1].record.payload_hash_hex, log[2].record.payload_hash_hex);

    // Every record's stored chain hash matches a praxis-core recompute.
    for receipt in &log {
        let recomputed = receipt.record.recompute_chain_hash().expect("recompute");
        let recomputed_hex: String =
            recomputed.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(recomputed_hex, receipt.record.chain_hash_hex);
    }

    // receipt.json is the latest receipt, byte-compatible with the log head.
    let head: SyncReceipt = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join(RECEIPT_REL_PATH)).expect("read receipt"),
    )
    .expect("parse receipt");
    assert_eq!(head.record.chain_hash_hex, log[2].record.chain_hash_hex);

    // Full-history verification passes at the CLI boundary.
    CliHarness::cargo_bin("ggen")
        .args(["receipt", "history"])
        .current_dir(dir.path())
        .run()
        .expect("history")
        .assert_success();
}

/// Tampering with the MIDDLE line's payload fails history verification,
/// naming index 1 — fail closed.
#[test]
fn tampering_middle_line_payload_fails_naming_index_1() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path(), &["alice"]);
    sync(dir.path(), SyncOptions { dry_run: false }).expect("sync 1");
    write_ontology(dir.path(), &["alice", "bob"]);
    sync(dir.path(), SyncOptions { dry_run: false }).expect("sync 2");
    write_ontology(dir.path(), &["alice", "bob", "carol"]);
    sync(dir.path(), SyncOptions { dry_run: false }).expect("sync 3");

    let log_path = dir.path().join(RECEIPT_LOG_REL_PATH);
    let raw = std::fs::read_to_string(&log_path).expect("read log");
    let mut lines: Vec<String> = raw.lines().map(String::from).collect();
    // Mutate the middle receipt's payload without touching its hashes.
    let mut mid: serde_json::Value = serde_json::from_str(&lines[1]).expect("parse mid");
    mid["payload"]["graph_hash"] = serde_json::Value::String("f".repeat(64));
    lines[1] = serde_json::to_string(&mid).expect("serialize mid");
    std::fs::write(&log_path, lines.join("\n") + "\n").expect("write tampered");

    let output = CliHarness::cargo_bin("ggen")
        .args(["receipt", "history"])
        .current_dir(dir.path())
        .run()
        .expect("history tampered");
    output.assert_failure().assert_stderr_contains("index 1");
}

/// Truncating the log (dropping the last line) breaks the head linkage
/// invariant only if a later record referenced it — dropping the middle
/// line breaks adjacency and must fail closed.
#[test]
fn removing_or_reordering_lines_fails_history_verification() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path(), &["alice"]);
    sync(dir.path(), SyncOptions { dry_run: false }).expect("sync 1");
    write_ontology(dir.path(), &["alice", "bob"]);
    sync(dir.path(), SyncOptions { dry_run: false }).expect("sync 2");
    write_ontology(dir.path(), &["alice", "bob", "carol"]);
    sync(dir.path(), SyncOptions { dry_run: false }).expect("sync 3");

    let log_path = dir.path().join(RECEIPT_LOG_REL_PATH);
    let raw = std::fs::read_to_string(&log_path).expect("read log");
    let lines: Vec<&str> = raw.lines().collect();
    assert_eq!(lines.len(), 3);

    // Drop the middle line: record 0's chain hash no longer matches
    // record 2's prev — broken link at index 0.
    std::fs::write(&log_path, format!("{}\n{}\n", lines[0], lines[2])).expect("truncate");
    CliHarness::cargo_bin("ggen")
        .args(["receipt", "history"])
        .current_dir(dir.path())
        .run()
        .expect("history truncated")
        .assert_failure();

    // Reorder: the second record no longer chains from genesis.
    std::fs::write(&log_path, format!("{}\n{}\n{}\n", lines[1], lines[0], lines[2]))
        .expect("reorder");
    CliHarness::cargo_bin("ggen")
        .args(["receipt", "history"])
        .current_dir(dir.path())
        .run()
        .expect("history reordered")
        .assert_failure();
}

/// Missing and empty logs both fail closed with an FM-coded error.
#[test]
fn missing_or_empty_log_fails_closed() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path(), &["alice"]);

    // No sync ever ran: log missing.
    let output = CliHarness::cargo_bin("ggen")
        .args(["receipt", "history"])
        .current_dir(dir.path())
        .run()
        .expect("history missing");
    output.assert_failure().assert_stderr_contains("FM-CHAIN-005");

    // Empty log file.
    std::fs::create_dir_all(dir.path().join(".ggen-v2")).expect("mkdir");
    std::fs::write(dir.path().join(RECEIPT_LOG_REL_PATH), "").expect("write empty");
    let output = CliHarness::cargo_bin("ggen")
        .args(["receipt", "history"])
        .current_dir(dir.path())
        .run()
        .expect("history empty");
    output.assert_failure().assert_stderr_contains("FM-CHAIN-005");
}

/// Dry-run syncs touch neither receipt.json nor the history log.
#[test]
fn dry_run_touches_neither_receipt_nor_log() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path(), &["alice"]);
    sync(dir.path(), SyncOptions { dry_run: true }).expect("dry run");
    assert!(!dir.path().join(RECEIPT_REL_PATH).exists());
    assert!(!dir.path().join(RECEIPT_LOG_REL_PATH).exists());
}

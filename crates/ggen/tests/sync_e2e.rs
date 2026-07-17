//! Chicago-TDD end-to-end tests for the sync pipeline: real filesystem,
//! real oxigraph store, real Tera rendering — no mocks.

use std::path::Path;

use ggen::sync::{sync, SyncOptions, SyncReceipt, RECEIPT_REL_PATH};
use tempfile::TempDir;

const GGEN_TOML: &str = r#"
[project]
name = "demo"

[ontology]
source = "ontology.ttl"

[templates]
dir = "templates"
"#;

const ONTOLOGY: &str = r#"
@prefix ex: <http://example.org/> .
ex:alice ex:name "alice" .
ex:bob   ex:name "bob" .
"#;

const TEMPLATE: &str = "---\nto: out/names.txt\nsparql:\n  people: SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name\n---\n{% for row in results %}{{ row.name }}\n{% endfor %}";

fn scaffold(root: &Path) {
    std::fs::write(root.join("ggen.toml"), GGEN_TOML).expect("write ggen.toml");
    std::fs::write(root.join("ontology.ttl"), ONTOLOGY).expect("write ontology");
    std::fs::create_dir_all(root.join("templates")).expect("mkdir templates");
    std::fs::write(root.join("templates/one.tmpl"), TEMPLATE).expect("write template");
}

#[test]
fn first_sync_writes_second_sync_skips_unchanged_and_hash_is_stable() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());

    // First run: writes the file.
    let first = sync(dir.path(), SyncOptions { dry_run: false }).expect("first sync");
    assert_eq!(first.written, vec![std::path::PathBuf::from("out/names.txt")]);
    assert!(first.skipped.is_empty(), "unexpected skips: {:?}", first.skipped);
    let content1 =
        std::fs::read_to_string(dir.path().join("out/names.txt")).expect("read output");
    assert_eq!(content1, "alice\nbob\n");

    // Second run: all Skipped(unchanged), byte-identical file, same hash.
    let second = sync(dir.path(), SyncOptions { dry_run: false }).expect("second sync");
    assert!(second.written.is_empty(), "second run wrote: {:?}", second.written);
    assert_eq!(second.skipped.len(), 1);
    assert!(
        second.skipped[0].1.contains("unchanged"),
        "reason: {}",
        second.skipped[0].1
    );
    let content2 =
        std::fs::read_to_string(dir.path().join("out/names.txt")).expect("read output");
    assert_eq!(content2, content1, "output must be byte-identical");
    assert_eq!(second.graph_hash_hex, first.graph_hash_hex, "graph hash must be stable");
}

#[test]
fn dry_run_writes_nothing() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());

    let report = sync(dir.path(), SyncOptions { dry_run: true }).expect("dry run");
    assert_eq!(report.written, vec![std::path::PathBuf::from("out/names.txt")]);
    assert!(!dir.path().join("out/names.txt").exists(), "dry run must not write");
    assert!(!dir.path().join(RECEIPT_REL_PATH).exists(), "dry run must not emit a receipt");
}

#[test]
fn non_dry_sync_emits_verifiable_receipt() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());

    let report = sync(dir.path(), SyncOptions { dry_run: false }).expect("sync");
    let raw =
        std::fs::read_to_string(dir.path().join(RECEIPT_REL_PATH)).expect("receipt exists");
    let receipt: SyncReceipt = serde_json::from_str(&raw).expect("receipt parses");

    assert_eq!(receipt.payload.graph_hash, report.graph_hash_hex);
    assert_eq!(receipt.payload.outputs.len(), 1);
    let file_bytes = std::fs::read(dir.path().join("out/names.txt")).expect("output");
    assert_eq!(
        receipt.payload.outputs.get("out/names.txt").map(String::as_str),
        Some(blake3::hash(&file_bytes).to_hex().as_str()),
        "receipt must bind the real output bytes"
    );

    // Chain hash recomputes to the stored value via praxis-core.
    let recomputed = receipt.record.recompute_chain_hash().expect("recompute");
    let recomputed_hex: String = recomputed.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(recomputed_hex, receipt.record.chain_hash_hex);
    assert_eq!(receipt.record.ts_ns, 0, "no wall clock: ts_ns pinned to 0");
}

#[test]
fn missing_ggen_toml_fails_closed() {
    let dir = TempDir::new().expect("tempdir");
    let err = sync(dir.path(), SyncOptions { dry_run: false }).expect_err("must fail");
    assert!(err.to_string().contains("FM-CONFIG-001"), "{err}");
}

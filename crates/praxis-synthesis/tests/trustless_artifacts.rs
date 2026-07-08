//! Artifact writer for the trustless replay recipe (PR-6).
//!
//! `#[ignore]` by default; run explicitly to (re)generate `receipts/trustless/`:
//!
//! ```text
//! cargo test -p praxis-synthesis --test trustless_artifacts -- --ignored
//! ```
//!
//! The artifacts are then re-verified WITHOUT cargo or the crate source by
//! `scripts/trustless_replay.sh verify` — a second implementation in a second
//! language (python3 + b3sum) in a bare directory.

// The deprecated execute_workflow surface stays covered until removal.
#![allow(deprecated)]
use std::fs;

const DEMO_TTL: &str = include_str!("../ontology/workflow_demo.ttl");

#[test]
#[ignore = "artifact writer; run explicitly to (re)generate receipts/trustless/"]
fn write_trustless_artifacts() {
    let (cell, groups) = praxis_synthesis::cell::run_cell(400, 4, 8);
    let receipt = praxis_synthesis::graph::execute_workflow(DEMO_TTL).expect("demo executes");

    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../receipts/trustless");
    fs::create_dir_all(dir).expect("create receipts/trustless");

    fs::write(
        format!("{dir}/cell.json"),
        serde_json::to_string(&cell).expect("cell serializes"),
    )
    .expect("write cell.json");
    fs::write(
        format!("{dir}/groups.json"),
        serde_json::to_string(&groups).expect("groups serialize"),
    )
    .expect("write groups.json");
    fs::write(format!("{dir}/workflow.ttl"), DEMO_TTL).expect("write workflow.ttl");
    fs::write(
        format!("{dir}/workflow_receipt.json"),
        serde_json::to_string(&receipt).expect("receipt serializes"),
    )
    .expect("write workflow_receipt.json");

    println!("trustless artifacts written to {dir}");
}

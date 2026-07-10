//! PROJ-616 determinism gate (integration): two in-process same-seed
//! workday runs must produce byte-identical evidence bundles — the OCEL
//! N-Triples serialization, every content-derived bundle file (via the
//! deterministic BLAKE3 manifest), the receipt-chain and graph digests,
//! and the report JSON modulo its path-derived `out_dir` field. A replay
//! pass over an untampered bundle must verify; a mutated bundle refuses
//! (typed) — the shell-level twin of this gate is `just cng-workday-verify`.
//!
//! No inline Turtle or SPARQL: all evidence enters through the real
//! `cng::bench::workday` producer and the `workday_verify` replay surface.

#![cfg(feature = "bench")]

use std::fs;
use std::path::{Path, PathBuf};

use chicago_tdd_tools::prelude::*;

use cng::bench::workday_verify::{assemble_workday_manifest, workday_replay};
use cng::bench::{workday, WorkdayConfig, WorkdayReport};
use cng::powl::CngRefusal;

/// Scratch root for this test file. O(1).
fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/workday-verify-it")
        .join(test_name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// One small deterministic workday (with injected bounded admissions, so
/// the refusal → admission → resume loop is inside the compared surface).
/// O(ticks) manufactures.
fn run_workday(dir: &Path) -> WorkdayReport {
    let cfg = WorkdayConfig {
        seed: 616,
        ticks: 4,
        refusal_per_mille: 250,
    };
    workday(dir, &cfg, None).expect("workday runs")
}

test!(
    same_seed_workdays_produce_byte_identical_evidence_bundles,
    {
        // Arrange + Act: the identical workday into two fresh directories.
        let dir_a = scratch_dir("byte_identical_a");
        let dir_b = scratch_dir("byte_identical_b");
        let report_a = run_workday(&dir_a);
        let report_b = run_workday(&dir_b);

        // Assert 1: every recorded digest (receipt chain, OCEL graph, obs
        // stream, run hook hash) is byte-identical.
        assert_eq!(
            report_a.evidence_chain_digest,
            report_b.evidence_chain_digest
        );
        assert_eq!(report_a.ocel_graph_digest, report_b.ocel_graph_digest);
        assert_eq!(report_a.obs_digest, report_b.obs_digest);
        assert_eq!(report_a.run_hook_hash, report_b.run_hook_hash);

        // Assert 2: the OCEL serialization files are byte-identical.
        let ocel_a = fs::read(dir_a.join("evidence").join("ocel.nt")).expect("ocel.nt a");
        let ocel_b = fs::read(dir_b.join("evidence").join("ocel.nt")).expect("ocel.nt b");
        assert_eq!(ocel_a, ocel_b);

        // Assert 3: the full bundle manifests (per-file BLAKE3 over obs/,
        // roster/, evidence/, admissions/, dispatch/, generated/, ticks/;
        // deterministic BTreeMap order) are identical maps.
        let manifest_a = assemble_workday_manifest(&dir_a).expect("manifest a");
        let manifest_b = assemble_workday_manifest(&dir_b).expect("manifest b");
        assert_eq!(manifest_a, manifest_b);
        assert!(!manifest_a.is_empty());

        // Assert 4: the report JSON is identical once the path-derived
        // `out_dir` field is removed from both.
        let strip_out_dir = |dir: &Path| -> serde_json::Value {
            let text = fs::read_to_string(dir.join("results").join("workday-report.json"))
                .expect("report json reads");
            let mut value: serde_json::Value = serde_json::from_str(&text).expect("report parses");
            value
                .as_object_mut()
                .expect("report is a JSON object")
                .remove("out_dir");
            value
        };
        assert_eq!(strip_out_dir(&dir_a), strip_out_dir(&dir_b));
    }
);

test!(replay_verifies_bundle_then_refuses_after_tamper, {
    // Arrange: one finished bundle; replay verifies it untampered.
    let dir = scratch_dir("replay_gate");
    let report = run_workday(&dir);
    let replay = workday_replay(&dir, None).expect("untampered bundle replays");
    assert_eq!(
        replay.recomputed_ocel_graph_digest,
        report.ocel_graph_digest
    );
    assert!(replay.obs_digest_match);
    assert!(replay.ocel_serialization_match);

    // Act: mutate the recorded observation stream (truncate one partition
    // — byte-level tamper of a run-produced file, nothing authored inline).
    let mut obs_files: Vec<PathBuf> = fs::read_dir(dir.join("obs"))
        .expect("obs dir reads")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
        .collect();
    obs_files.sort();
    let target = obs_files.first().expect("at least one obs partition");
    let body = fs::read_to_string(target).expect("obs partition reads");
    let lines: Vec<&str> = body.lines().collect();
    assert!(lines.len() > 2, "partition has multiple lines");
    fs::write(target, lines[..lines.len() / 2].join("\n")).expect("tampered partition writes");
    let result = workday_replay(&dir, None);

    // Assert: typed CNG_R11 third-party integrity refusal.
    match result {
        Err(refusal @ CngRefusal::AuditMismatch(_)) => {
            assert_eq!(refusal.code(), "CNG_R11");
        }
        other => panic!("expected AuditMismatch, got {other:?}"),
    }
});

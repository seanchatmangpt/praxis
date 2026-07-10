#![cfg(test)]

//! Workday benchmark tests (PROJ-608/610/611). Fixture RDF enters only via
//! the on-disk observation templates and the workday generator itself —
//! no inline Turtle, no inline SPARQL (queries load from the query set).

use std::fs;
use std::path::PathBuf;

use chicago_tdd_tools::prelude::*;
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use super::{expect_standing_rows, workday, WorkdayConfig};
use crate::bench::fill_template;
use crate::bench::templates::{load_templates, QuerySet};
use crate::powl::CngRefusal;

/// Scratch root for this test file. O(1).
fn scratch_dir(test_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/workday")
        .join(test_name)
}

test!(workday_same_seed_twice_is_byte_identical, {
    // Arrange: two fresh directories, one config (no injected refusals).
    let cfg = WorkdayConfig {
        seed: 42,
        ticks: 4,
        refusal_per_mille: 0,
    };
    let dir_a = scratch_dir("same_seed_a");
    let dir_b = scratch_dir("same_seed_b");
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);

    // Act: run the identical workday into both directories.
    let report_a = workday(&dir_a, &cfg, None).expect("workday run a");
    let report_b = workday(&dir_b, &cfg, None).expect("workday run b");

    // Assert: every digest is byte-identical across the two runs (workflow
    // ids and digests are tick/content-derived, never path-derived).
    assert_eq!(report_a.obs_digest, report_b.obs_digest);
    assert_eq!(report_a.ocel_graph_digest, report_b.ocel_graph_digest);
    assert_eq!(
        report_a.evidence_chain_digest,
        report_b.evidence_chain_digest
    );
    assert_eq!(report_a.receipts, 4);
    assert_eq!(report_a.refusals, 0);
    assert_eq!(report_a.workflow_instances, 4);
    assert_eq!(report_a.workers_represented, 1);
    // PROJ-612: zero-unreceipted-actuation — every executed transition has
    // exactly one hook receipt in the evidence graph, and the run-level
    // hook hash (folded into evidence_chain_digest) is byte-deterministic.
    assert_eq!(report_a.hook_receipts, report_a.executed_transitions);
    assert_eq!(
        report_a.telemetry_hook_actuations as u64,
        report_a.hook_receipts
    );
    assert!(!report_a.run_hook_hash.is_empty());
    assert_eq!(report_a.run_hook_hash, report_b.run_hook_hash);
});

test!(workday_bounded_admission_resumes_every_refusal, {
    // Arrange: refusal_per_mille = 1000 → EVERY tick's manufacture refuses
    // (withheld final problem), so every tick must go through the full
    // admission-requested → admission-granted → resumed loop.
    let cfg = WorkdayConfig {
        seed: 7,
        ticks: 3,
        refusal_per_mille: 1000,
    };
    let dir_a = scratch_dir("resume_a");
    let dir_b = scratch_dir("resume_b");
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);

    // Act.
    let report_a = workday(&dir_a, &cfg, None).expect("workday resume run a");
    let report_b = workday(&dir_b, &cfg, None).expect("workday resume run b");

    // Assert: graph-derived counts prove the loop closed for every tick,
    // and the refusal path is deterministic across runs.
    assert_eq!(report_a.refusals, 3);
    assert_eq!(report_a.admission_requests, 3);
    assert_eq!(report_a.admissions_granted, 3);
    assert_eq!(report_a.resumes, 3);
    assert_eq!(report_a.receipts, 3);
    assert_eq!(report_a.workflow_instances, 3);
    assert_eq!(report_a.obs_digest, report_b.obs_digest);
    assert_eq!(report_a.ocel_graph_digest, report_b.ocel_graph_digest);
    assert_eq!(
        report_a.evidence_chain_digest,
        report_b.evidence_chain_digest
    );
    // The admission-request artifacts were manufactured on disk.
    let admissions: Vec<_> = fs::read_dir(dir_a.join("admissions"))
        .expect("admissions dir exists")
        .flatten()
        .collect();
    assert_eq!(admissions.len(), 3);
});

test!(ambiguous_standing_refuses_cng_r12, {
    // Arrange: fabricate ambiguous standing — TWO unresolved workday_tick
    // observations at once, rendered from the on-disk template (never
    // inline Turtle).
    let templates = load_templates().expect("templates load");
    let tick_template = templates
        .obs
        .get("workday-tick")
        .expect("workday-tick template present");
    let store = Store::new().expect("store");
    for (i, set_id) in ["tick-0005", "tick-9999"].iter().enumerate() {
        let seq = i.to_string();
        let body = fill_template(
            tick_template,
            &[
                ("SUBJECT", format!("obs-ambiguous-{i}").as_str()),
                ("SEQ", seq.as_str()),
                ("SET_ID", set_id),
                ("TICK", "5"),
                ("WORKER_ID", "w0"),
            ],
        );
        store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), body.as_bytes())
            .expect("fabricated observation parses");
    }
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    let standing_query = queries
        .get("standing-next-action")
        .expect("standing query present");

    // Act: while work remains, the loop demands exactly one candidate.
    let result = expect_standing_rows(&store, standing_query, 5, 1);

    // Assert: typed CNG_R12 refusal carrying the tick and candidate count.
    match result {
        Err(CngRefusal::StandingAmbiguous {
            tick,
            candidate_count,
        }) => {
            assert_eq!(tick, 5);
            assert_eq!(candidate_count, 2);
            assert_eq!(
                CngRefusal::StandingAmbiguous {
                    tick,
                    candidate_count
                }
                .code(),
                "CNG_R12"
            );
        }
        other => panic!("expected StandingAmbiguous, got {other:?}"),
    }
});

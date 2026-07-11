#![cfg(test)]

//! Workday benchmark tests (PROJ-608/610/611). Fixture RDF enters only via
//! the on-disk observation templates and the workday generator itself —
//! no inline Turtle, no inline SPARQL (queries load from the query set).

use std::fs;
use std::path::PathBuf;

use chicago_tdd_tools::prelude::*;
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use super::{
    build_marker_store, evaluate_markers, expect_standing_rows, hook_actuation_gate, workday,
    WorkdayConfig, WorkdayHookBroker,
};
use crate::bench::fill_template;
use crate::bench::roles::run_construct;
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
    // PROJ-614: the replay headline is graph-derived (metric-replay.rq over
    // replay_verified events) and every receipted tick was replayed.
    assert_eq!(report_a.replay_verified, report_a.receipts);
    // PROJ-614: the CNG_R19-gated dispatch-closure facets are zero.
    assert_eq!(report_a.dispatch_closure.get("unacknowledged"), Some(&0));
    assert_eq!(
        report_a.dispatch_closure.get("returned_unadmitted"),
        Some(&0)
    );
    // PROJ-622: all eleven success markers are SPARQL-derived and true on a
    // healthy seeded run (a false marker would have refused CNG_R20).
    for marker in [
        "AUTONOMIC_LOOP_CLOSED",
        "EXTERNAL_WORKFLOW_DISPATCH_PROVEN",
        "EXTERNAL_RESULT_READMISSION_PROVEN",
        "RECURSIVE_CHILD_CLOSURE_PROVEN",
        "TIMEOUT_ESCALATION_PROVEN",
        "COMPENSATION_WORKFLOW_PROVEN",
        "ONE_PERSON_RECURSIVE_WORKFLOW_PROVEN",
        "GRAPHLAW_DIALECT_CLOSURE",
        "HOOK_ACTUATION_PROVEN",
        "ZERO_UNRECEIPTED_ACTUATION",
        "V26_7_10_PRODUCTION_READY",
    ] {
        assert_eq!(
            report_a.markers.get(marker),
            Some(&true),
            "marker {marker} missing or false"
        );
    }
    assert_eq!(report_a.markers.len(), 11);
});

test!(forced_false_marker_refuses_cng_r20, {
    // Arrange: a marker store holding ONE unresolved workday_tick
    // observation (rendered from the on-disk template — no inline Turtle)
    // and nothing else: the autonomic-loop chain (planned /
    // transition_fired / hook_receipt / receipted) is missing, so
    // marker-autonomic-loop.rq must yield a nonzero ?value.
    let templates = load_templates().expect("templates load");
    let tick_template = templates
        .obs
        .get("workday-tick")
        .expect("workday-tick template present");
    let obs_store = Store::new().expect("obs store");
    let body = fill_template(
        tick_template,
        &[
            ("SUBJECT", "obs-broken-0"),
            ("SEQ", "0"),
            ("SET_ID", "tick-0000"),
            ("TICK", "0"),
            ("WORKER_ID", "w0"),
        ],
    );
    obs_store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), body.as_bytes())
        .expect("fabricated observation parses");
    let evidence_store = Store::new().expect("evidence store");
    let registry_path = WorkdayHookBroker::default_hooks_dir().join("dialect-registry.ttl");
    let marker_store =
        build_marker_store(&obs_store, &evidence_store, &registry_path).expect("marker store");
    let marker_queries =
        QuerySet::load(&QuerySet::default_dir().join("markers")).expect("marker queries load");

    // Act.
    let result = evaluate_markers(&marker_store, &marker_queries);

    // Assert: typed CNG_R20 naming the broken marker and its value.
    match result {
        Err(CngRefusal::MarkerFalse { marker, value }) => {
            assert_eq!(marker, "AUTONOMIC_LOOP_CLOSED");
            assert!(value > 0);
            assert_eq!(CngRefusal::MarkerFalse { marker, value }.code(), "CNG_R20");
        }
        other => panic!("expected MarkerFalse, got {other:?}"),
    }
});

test!(unreceipted_actuation_gate_refuses_cng_r19, {
    // Arrange: an observation store with ONE transition_fired observation
    // (template-rendered) and NO hook_receipt; materialize the OCEL
    // evidence through the same on-disk constructs the workday uses, so
    // the graph shows one transition and zero receipted actuations.
    let templates = load_templates().expect("templates load");
    let fired_template = templates
        .obs
        .get("transition-fired")
        .expect("transition-fired template present");
    let obs_store = Store::new().expect("obs store");
    let body = fill_template(
        fired_template,
        &[
            ("SUBJECT", "obs-unreceipted-0"),
            ("SEQ", "0"),
            ("SET_ID", "tick-0000"),
            ("WORKFLOW_ID", "tick-0000"),
            ("WORKER_ID", "w0"),
            ("ACTIVITY_LABEL", "classify"),
            ("TICK", "0"),
        ],
    );
    obs_store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), body.as_bytes())
        .expect("fabricated observation parses");
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    let evidence_store = Store::new().expect("evidence store");
    for construct in ["ocel-events.construct", "ocel-hook-receipts.construct"] {
        run_construct(
            &obs_store,
            queries.get(construct).expect("construct present"),
            &evidence_store,
        )
        .expect("construct runs");
    }

    // Act: the graph-derived hook-actuation gate.
    let result = hook_actuation_gate(&evidence_store, &queries);

    // Assert: typed CNG_R19 naming the gate and the mismatch count.
    match result {
        Err(CngRefusal::EvidenceGateFailed { gate, count }) => {
            assert_eq!(gate, "unreceipted-actuations");
            assert_eq!(count, 1);
            assert_eq!(
                CngRefusal::EvidenceGateFailed { gate, count }.code(),
                "CNG_R19"
            );
        }
        other => panic!("expected EvidenceGateFailed, got {other:?}"),
    }
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

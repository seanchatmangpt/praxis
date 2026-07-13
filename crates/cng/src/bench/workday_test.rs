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
    build_decomp_marker_store, build_marker_store, evaluate_marker_map, evaluate_markers,
    evaluate_planning_markers, expect_standing_rows, full_production_ready, hook_actuation_gate,
    workday, WorkdayConfig, WorkdayHookBroker,
};
use crate::bench::decomp::{decomp_queries_dir, decompose, strips_graph_to_surface};
use crate::bench::fill_template;
use crate::bench::roles::run_construct;
use crate::bench::templates::{load_templates, QuerySet};
use crate::powl::CngRefusal;

/// Scratch root for this test file. O(1).
fn scratch_dir(test_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!(
            "../../target/chatman/cng-tests/workday_{}",
            std::process::id()
        ))
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
    // PROJ-622/727: all seventeen success markers (sixteen named + the
    // conjunction) are SPARQL-derived and true on a healthy seeded run (a
    // false marker would have refused CNG_R20). The PROJ-727 distributed
    // rows hold on a single-operator workday: isolation/remote/divergence
    // vacuously (no such obs kinds), the arazzo pair via the broker's
    // generated/dispatched emissions on every dispatch lifecycle.
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
        "SHARED_MEMORY_CROSSINGS_ZERO",
        "DIRECT_ENGINE_BYPASSES_ZERO",
        "REMOTE_WORKFLOWS_ACKNOWLEDGED",
        "REMOTE_WORKFLOWS_COMPLETED",
        "REPLAY_DIVERGENCES_ZERO",
        "ARAZZO_WORKFLOWS_DISPATCHED",
        "V26_7_10_PRODUCTION_READY",
    ] {
        assert_eq!(
            report_a.markers.get(marker),
            Some(&true),
            "marker {marker} missing or false"
        );
    }
    assert_eq!(report_a.markers.len(), 17);
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

/// Loads the shared potato pddl-strips example graph (`examples/
/// pddl-strips-potato.ttl`) — the same on-disk fixture `tests/
/// cng_decomp.rs` uses — into a typed (domain, problem) surface via the
/// decomp graph→PDDL bridge. No inline Turtle/PDDL in this file.
fn potato_surface() -> (bcinr_pddl::Pddl8Domain, bcinr_pddl::Pddl8Problem) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/pddl-strips-potato.ttl");
    let turtle = fs::read_to_string(&path).expect("read potato example graph");
    let store = Store::new().expect("potato store");
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .expect("potato example graph must parse");
    let queries = QuerySet::load(&decomp_queries_dir()).expect("load decomp queries");
    let domain_template = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates/decomp-domain.template.pddl"),
    )
    .expect("read domain template");
    let problem_template = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates/decomp-problem.template.pddl"),
    )
    .expect("read problem template");
    strips_graph_to_surface(&store, &queries, &domain_template, &problem_template)
        .expect("potato graph bridges to a surface")
}

test!(planning_markers_prove_true_on_a_healthy_decompose_run, {
    // Arrange: a real `cng plan decompose`-equivalent run (PROJ-741's
    // `decompose()`) over the shared potato surface — its outcome may be
    // Selected OR a typed single-actor result; every planning marker query
    // is written to hold in EITHER case (see each query's header), so this
    // test proves robustness across both branches, not one hardcoded shape.
    let (domain, problem) = potato_surface();
    let out = scratch_dir("planning-markers-potato");
    let _ = fs::remove_dir_all(&out);
    let marker_queries =
        QuerySet::load(&QuerySet::default_dir().join("markers")).expect("marker queries load");

    // Act.
    let result = decompose(
        &domain,
        &problem,
        &out,
        "urn:cng:test:workday:planning-markers",
    )?;
    let decomp_store = build_decomp_marker_store(&result.result_graph_path)?;
    let markers = evaluate_planning_markers(&decomp_store, &marker_queries)?;

    // Assert: all nine planning markers (PROJ-739/740) true on a healthy
    // run — a false marker would have refused CNG_R20 before this line.
    for marker in [
        "DECOMPOSITION_DERIVED_PROVEN",
        "DECOMPOSITION_CANDIDATES_RECEIPTED",
        "INTERFACE_STATE_PROVEN",
        "NON_INTERFERENCE_PROVEN",
        "RESOURCE_RELEASE_CLOSED",
        "SINGLE_ACTOR_TYPED_RESULT",
        "LLM_CALLS_ZERO",
        "ENGLISH_SUBGOALS_ZERO",
        "CANNED_SUBGOALS_ZERO",
    ] {
        assert_eq!(
            markers.get(marker),
            Some(&true),
            "marker {marker} missing or false"
        );
    }
    assert_eq!(markers.len(), 9);

    // PROJ-742: the full conjunction folds workday + planning markers under
    // the SAME V26_7_10_PRODUCTION_READY name, true when both halves hold.
    let mut workday_markers = std::collections::BTreeMap::new();
    workday_markers.insert("AUTONOMIC_LOOP_CLOSED".to_string(), true);
    workday_markers.insert("V26_7_10_PRODUCTION_READY".to_string(), true);
    let full = full_production_ready(&workday_markers, &markers, None);
    assert_eq!(full.get("V26_7_10_PRODUCTION_READY"), Some(&true));
    assert_eq!(full.get("DECOMPOSITION_DERIVED_PROVEN"), Some(&true));
});

test!(
    full_production_ready_refuses_when_a_planning_marker_is_false,
    {
        // Arrange: a healthy workday-side map, but a planning map carrying one
        // false entry (a directly fabricated negative — no decompose run
        // needed to exercise the pure combinator).
        let mut workday_markers = std::collections::BTreeMap::new();
        workday_markers.insert("AUTONOMIC_LOOP_CLOSED".to_string(), true);
        workday_markers.insert("V26_7_10_PRODUCTION_READY".to_string(), true);
        let mut planning_markers = std::collections::BTreeMap::new();
        planning_markers.insert("DECOMPOSITION_DERIVED_PROVEN".to_string(), true);
        planning_markers.insert("LLM_CALLS_ZERO".to_string(), false);

        // Act.
        let full = full_production_ready(&workday_markers, &planning_markers, None);

        // Assert: the SAME marker name now reflects the fuller conjunction.
        assert_eq!(full.get("V26_7_10_PRODUCTION_READY"), Some(&false));
        assert_eq!(full.get("LLM_CALLS_ZERO"), Some(&false));
    }
);

test!(fabricated_decomp_result_without_receipts_refuses_cng_r20, {
    // Arrange: a decomp marker store holding ONE well-typed
    // decomp:DecompositionResult (outcome + selectedCandidateId +
    // rejectedCandidateCount, all present and lawful) but ZERO
    // decomp:CandidateReceipt facts — constructed via oxigraph's typed
    // term API (Quad/NamedNode/Literal), never inline Turtle, mirroring
    // decomp/lift.rs's own fact-construction idiom. This proves
    // marker-decomposition-receipted.rq detects a receipting gap that
    // `DECOMPOSITION_DERIVED_PROVEN` alone would not catch.
    use oxigraph::model::{GraphName, Literal, NamedNode, Quad, Term};
    const DECOMP_NS: &str = "https://truex.io/ontology/decomp#";
    const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
    let named = |iri: &str| NamedNode::new(iri).expect("iri");
    let result_iri = named("urn:cng:test:workday:fabricated-decomp-result");
    let store = Store::new().expect("store");
    let insert = |pred_iri: String, object: Term| {
        store
            .insert(&Quad::new(
                result_iri.clone(),
                named(&pred_iri),
                object,
                GraphName::DefaultGraph,
            ))
            .expect("insert fabricated triple");
    };
    insert(
        RDF_TYPE.to_string(),
        Term::NamedNode(named(&format!("{DECOMP_NS}DecompositionResult"))),
    );
    insert(
        format!("{DECOMP_NS}outcome"),
        Term::Literal(Literal::new_simple_literal("NoAdmissibleDecomposition")),
    );
    insert(
        format!("{DECOMP_NS}selectedCandidateId"),
        Term::Literal(Literal::new_simple_literal("0-single")),
    );
    insert(
        format!("{DECOMP_NS}rejectedCandidateCount"),
        Term::Literal(Literal::from(0i64)),
    );
    // Deliberately NO decomp:CandidateReceipt facts point at `result_iri`.
    let planning_queries =
        QuerySet::load(&QuerySet::default_dir().join("markers")).expect("marker queries load");

    // Act.
    let result = evaluate_marker_map(
        &store,
        &planning_queries,
        &[(
            "marker-decomposition-receipted",
            &["DECOMPOSITION_CANDIDATES_RECEIPTED"],
        )],
    );

    // Assert: typed CNG_R20 naming the broken marker and its nonzero value.
    match result {
        Err(CngRefusal::MarkerFalse { marker, value }) => {
            assert_eq!(marker, "DECOMPOSITION_CANDIDATES_RECEIPTED");
            assert!(value > 0);
        }
        other => panic!("expected MarkerFalse, got {other:?}"),
    }
});

test!(
    fabricated_llm_call_observation_refuses_marker_no_llm_authoring,
    {
        // Arrange: a marker store holding ONE typed obs:obsKind "llm_call"
        // quad, constructed via oxigraph's typed term API (never inline
        // Turtle) — no production code path ever emits this kind; this proves
        // the negative-obs half of marker-no-llm-authoring.rq actually detects
        // it if it ever appeared.
        use oxigraph::model::{GraphName, Literal, NamedNode, Quad, Term};
        let store = Store::new().expect("store");
        let subject = NamedNode::new("http://example.org/rwai#fabricated-llm-call-0").expect("iri");
        let pred = NamedNode::new("https://ggen.io/ontology/bench-obs#obsKind")
            .expect("obsKind predicate");
        store
            .insert(&Quad::new(
                subject,
                pred,
                Term::Literal(Literal::new_simple_literal("llm_call")),
                GraphName::DefaultGraph,
            ))
            .expect("insert fabricated observation");
        let planning_queries =
            QuerySet::load(&QuerySet::default_dir().join("markers")).expect("marker queries load");

        // Act.
        let result = evaluate_marker_map(
            &store,
            &planning_queries,
            &[(
                "marker-no-llm-authoring",
                &[
                    "LLM_CALLS_ZERO",
                    "ENGLISH_SUBGOALS_ZERO",
                    "CANNED_SUBGOALS_ZERO",
                ],
            )],
        );

        // Assert: typed CNG_R20 naming LLM_CALLS_ZERO (first of the three
        // grouped names) and its nonzero value.
        match result {
            Err(CngRefusal::MarkerFalse { marker, value }) => {
                assert_eq!(marker, "LLM_CALLS_ZERO");
                assert!(value > 0);
            }
            other => panic!("expected MarkerFalse, got {other:?}"),
        }
    }
);

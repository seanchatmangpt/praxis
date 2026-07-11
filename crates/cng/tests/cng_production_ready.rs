//! PROJ-742-followup (integration): closes the gap recorded in
//! `docs/releases/v26.7.10/DOD_EVIDENCE_MAP.md` this session —
//! `full_production_ready` (`src/bench/workday.rs`, PROJ-742) had NEVER been
//! invoked end-to-end against a REAL `workday()` evidence bundle's markers
//! together with a REAL `cng plan decompose` evidence bundle's planning
//! markers; only the constituent marker families were separately verified
//! true (`src/bench/workday_test.rs`'s `planning_markers_prove_true_on_a_
//! healthy_decompose_run` feeds `full_production_ready` a HAND-FABRICATED
//! two-entry workday map, never the real 17-entry map a `workday()` run
//! actually produces).
//!
//! This test runs BOTH real bundles in-process — a real `workday()` run
//! (same call pattern as `tests/cng_workday_verify.rs`) and a real
//! `cng::bench::decomp::decompose` run over the on-disk potato fixture
//! (same call pattern as `tests/cng_decomp.rs`) — and feeds their REAL
//! marker maps into the combinator together for the first time, proving
//! `V26_7_10_PRODUCTION_READY` genuinely holds across the dual-bundle
//! surface. Distributed markers are omitted (`None`): a real third
//! multi-engine bundle requires spawning real OS engine processes through
//! `tests/cng_multi_engine.rs`'s own harness, whose helpers are private to
//! that test binary and not reusable from a separate integration test
//! crate — non-trivial to wire here, so two-way (workday + planning)
//! coverage is the honest minimum this test proves, per the task scope.
//!
//! No inline Turtle/PDDL/SPARQL: the workday side enters through the real
//! `cng::bench::workday` producer; the planning side enters through the
//! real `cng::bench::decomp::decompose` producer over the shared potato
//! example graph (`examples/pddl-strips-potato.ttl`).

#![cfg(feature = "bench")]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use chicago_tdd_tools::prelude::*;

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use cng::bench::decomp::{decomp_queries_dir, decompose, strips_graph_to_surface};
use cng::bench::{
    build_decomp_marker_store, evaluate_planning_markers, full_production_ready, workday, QuerySet,
    WorkdayConfig,
};

/// Scratch root for this test file. O(1).
fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/production-ready-it")
        .join(test_name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn template(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("templates")
            .join(name),
    )
    .expect("read template")
}

/// One real single-operator workday bundle. Its report's `markers` field is
/// the REAL 17-entry map `evaluate_markers` computed inside `workday()`
/// itself, over the real obs ∪ evidence ∪ dialect-registry union store (a
/// false marker would have refused CNG_R20 before the report exists).
///
/// # Complexity
/// O(ticks) manufactures (each pipeline-bounded).
fn real_workday_markers(out: &Path) -> BTreeMap<String, bool> {
    let cfg = WorkdayConfig {
        seed: 742,
        ticks: 4,
        refusal_per_mille: 0,
    };
    let report = workday(out, &cfg, None).expect("real workday run");
    assert_eq!(report.markers.len(), 17, "interim-17 marker count drifted");
    report.markers
}

/// One real `cng plan decompose`-equivalent run (`decompose()`, PROJ-741)
/// over the shared potato fixture (same graph→PDDL bridge pattern as
/// `tests/cng_decomp.rs`), returning the REAL 9-entry planning marker map
/// evaluated over the `decomposition-result.ttl` the run actually wrote to
/// disk (`build_decomp_marker_store` + `evaluate_planning_markers`).
///
/// # Complexity
/// One bounded `decompose()` call + O(planning markers) SELECTs.
fn real_planning_markers(out: &Path) -> BTreeMap<String, bool> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/pddl-strips-potato.ttl");
    let turtle = fs::read_to_string(&path).expect("read potato example graph");
    let store = Store::new().expect("store");
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .expect("potato example graph must parse");
    let queries = QuerySet::load(&decomp_queries_dir()).expect("load decomp queries");
    let (domain, problem) = strips_graph_to_surface(
        &store,
        &queries,
        &template("decomp-domain.template.pddl"),
        &template("decomp-problem.template.pddl"),
    )
    .expect("potato graph bridges to a surface");
    let result = decompose(
        &domain,
        &problem,
        out,
        "urn:cng:test:production-ready:potato",
    )
    .expect("real decompose run");
    let decomp_store =
        build_decomp_marker_store(&result.result_graph_path).expect("decomp marker store loads");
    let marker_queries =
        QuerySet::load(&QuerySet::default_dir().join("markers")).expect("marker queries load");
    let markers = evaluate_planning_markers(&decomp_store, &marker_queries)
        .expect("planning markers evaluate");
    assert_eq!(markers.len(), 9, "planning marker count drifted");
    markers
}

test!(full_production_ready_holds_on_real_dual_bundle_evidence, {
    // Arrange + Act: a real workday bundle AND a real decompose bundle,
    // each producing its own real evidence-derived marker map.
    let workday_markers = real_workday_markers(&scratch_dir("workday"));
    let planning_markers = real_planning_markers(&scratch_dir("planning"));
    assert!(
        workday_markers.values().all(|v| *v),
        "every real workday marker must be true: {workday_markers:?}"
    );
    assert!(
        planning_markers.values().all(|v| *v),
        "every real planning marker must be true: {planning_markers:?}"
    );

    // Act: the combinator under test (PROJ-742), fed REAL marker maps from
    // BOTH real bundles together — the exact invocation the gap said had
    // never happened.
    let combined = full_production_ready(&workday_markers, &planning_markers, None);

    // Assert: 16 workday-named markers (the interim conjunction is dropped,
    // then recomputed) + 9 planning markers + the recomputed conjunction.
    assert_eq!(combined.len(), 26, "combined marker count: {combined:?}");
    for marker in [
        "AUTONOMIC_LOOP_CLOSED",
        "ONE_PERSON_RECURSIVE_WORKFLOW_PROVEN",
        "RECURSIVE_CHILD_CLOSURE_PROVEN",
        "GRAPHLAW_DIALECT_CLOSURE",
        "EXTERNAL_WORKFLOW_DISPATCH_PROVEN",
        "EXTERNAL_RESULT_READMISSION_PROVEN",
        "HOOK_ACTUATION_PROVEN",
        "ZERO_UNRECEIPTED_ACTUATION",
        "TIMEOUT_ESCALATION_PROVEN",
        "COMPENSATION_WORKFLOW_PROVEN",
        "SHARED_MEMORY_CROSSINGS_ZERO",
        "DIRECT_ENGINE_BYPASSES_ZERO",
        "REMOTE_WORKFLOWS_ACKNOWLEDGED",
        "REMOTE_WORKFLOWS_COMPLETED",
        "REPLAY_DIVERGENCES_ZERO",
        "ARAZZO_WORKFLOWS_DISPATCHED",
    ] {
        assert_eq!(
            combined.get(marker),
            Some(&true),
            "workday-side marker {marker} missing or false: {combined:?}"
        );
    }
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
            combined.get(marker),
            Some(&true),
            "planning-side marker {marker} missing or false: {combined:?}"
        );
    }
    assert_eq!(
        combined.get("V26_7_10_PRODUCTION_READY"),
        Some(&true),
        "combined conjunction must hold on healthy real dual-bundle evidence: {combined:?}"
    );
});

test!(
    full_production_ready_goes_false_when_a_real_marker_is_forced_false,
    {
        // Arrange: the SAME class of real dual-bundle evidence as the
        // positive test above (mirroring `workday_test.rs`'s
        // forced-false-marker pattern — see `forced_false_marker_refuses_
        // cng_r20` — but on the real 26-key combined shape from real runs,
        // not a hand-fabricated map) — then force exactly one entry false
        // in a clone of each side to prove the combinator's negative branch
        // actually fires on real evidence.
        let workday_markers = real_workday_markers(&scratch_dir("workday-negative"));
        let planning_markers = real_planning_markers(&scratch_dir("planning-negative"));

        // Control: the unmodified real pair holds true (this combinator is
        // not trivially always-false).
        let healthy = full_production_ready(&workday_markers, &planning_markers, None);
        assert_eq!(healthy.get("V26_7_10_PRODUCTION_READY"), Some(&true));

        // Act 1: force one WORKDAY-side marker false in a clone.
        let mut broken_workday = workday_markers.clone();
        broken_workday.insert("AUTONOMIC_LOOP_CLOSED".to_string(), false);
        let combined_workday_false =
            full_production_ready(&broken_workday, &planning_markers, None);
        assert_eq!(
            combined_workday_false.get("AUTONOMIC_LOOP_CLOSED"),
            Some(&false)
        );
        assert_eq!(
            combined_workday_false.get("V26_7_10_PRODUCTION_READY"),
            Some(&false),
            "a false workday-side marker must sink the combined conjunction: \
             {combined_workday_false:?}"
        );

        // Act 2: force one PLANNING-side marker false in a clone of the
        // otherwise-healthy pair — the conjunction must go false from this
        // side too, proving neither half alone determines the result.
        let mut broken_planning = planning_markers.clone();
        broken_planning.insert("LLM_CALLS_ZERO".to_string(), false);
        let combined_planning_false =
            full_production_ready(&workday_markers, &broken_planning, None);
        assert_eq!(combined_planning_false.get("LLM_CALLS_ZERO"), Some(&false));
        assert_eq!(
            combined_planning_false.get("V26_7_10_PRODUCTION_READY"),
            Some(&false),
            "a false planning-side marker must sink the combined conjunction: \
             {combined_planning_false:?}"
        );
    }
);

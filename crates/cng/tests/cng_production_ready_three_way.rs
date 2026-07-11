//! PROJ-742-followup (three-way closure): closes the remaining gap this
//! session's dual-bundle test (`tests/cng_production_ready.rs`) explicitly
//! left open — `full_production_ready`'s THIRD argument
//! (`distributed_markers: Option<&BTreeMap<String, bool>>`) had NEVER been
//! populated with a REAL evidence-derived map; the dual-bundle test called
//! it with `None` because its author judged the multi-engine harness's test
//! HELPER FUNCTIONS (`spawn_engine` et al. in `tests/cng_multi_engine.rs`)
//! private to that binary and non-trivial to reuse from a separate
//! integration-test crate.
//!
//! That judgment about the HELPERS was correct (they are `fn` items in a
//! `tests/*.rs` binary, not exported by the `cng` library) — but the
//! COORDINATOR API those helpers call is already public library surface:
//! `cng::bench::{engine_dispatch_remote, engine_collect_remote,
//! EngineCoordinateReport}` (`src/bench/engine.rs`), and
//! `EngineCoordinateReport::markers` is ALREADY a `pub` field carrying the
//! real [`DISTRIBUTED_MARKER_MAP`]-evaluated map (9 keys) that
//! `engine_collect_remote` computes internally over the coordinator ∪
//! engine-bundle evidence union (a false marker would have refused
//! `CNG_R20` before the report exists) — no visibility bump anywhere was
//! required to close this gap; `report.markers` is fed to
//! `full_production_ready` directly.
//!
//! This file duplicates ~10 lines of process-spawn boilerplate
//! (`run_cng`/`serve_to_budget`, mirroring `cng_multi_engine.rs`'s own
//! helpers of the same name and shape) rather than importing them, keeping
//! this file independent and collision-free from any concurrent editing of
//! `cng_multi_engine.rs` itself. `tests/cng_production_ready.rs` is
//! untouched (additive-only).
//!
//! Three real bundles feed the combinator together for the first time:
//! 1. a real `cng::bench::workday(...)` run (17-entry marker map);
//! 2. a real `cng::bench::decomp::decompose(...)` run over the shared
//!    potato fixture (9-entry planning marker map);
//! 3. a real two-engine (H, M) `engine_dispatch_remote` →
//!    `cng engine serve` (real `CARGO_BIN_EXE_cng` OS processes) →
//!    `engine_collect_remote` coordination round (9-entry distributed
//!    marker map, `EngineCoordinateReport::markers`).
//!
//! No inline Turtle/PDDL/SPARQL: every producer is the real library/binary
//! entry point; no synthetic Turtle or hand-typed SPARQL appears here.

#![cfg(feature = "bench")]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chicago_tdd_tools::prelude::*;

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use cng::bench::decomp::{decomp_queries_dir, decompose, strips_graph_to_surface};
use cng::bench::{
    build_decomp_marker_store, engine_collect_remote, engine_dispatch_remote,
    evaluate_planning_markers, full_production_ready, workday, QuerySet, WorkdayConfig,
};

/// Fixed splitmix64/engine seed for this file's real runs. O(1).
const SEED: u64 = 743;

/// Scratch root for this test file, isolated from
/// `tests/cng_production_ready.rs`'s own scratch namespace. O(existing
/// files) removal.
fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/production-ready-three-way-it")
        .join(test_name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// Reads a template from `templates/`. O(file size).
fn template(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("templates")
            .join(name),
    )
    .expect("read template")
}

/// One real single-operator workday bundle's real 17-entry marker map
/// (mirrors `tests/cng_production_ready.rs`'s `real_workday_markers`).
///
/// # Complexity
/// O(ticks) manufactures (each pipeline-bounded).
fn real_workday_markers(out: &Path) -> BTreeMap<String, bool> {
    let cfg = WorkdayConfig {
        seed: SEED,
        ticks: 4,
        refusal_per_mille: 0,
    };
    let report = workday(out, &cfg, None).expect("real workday run");
    assert_eq!(report.markers.len(), 17, "interim-17 marker count drifted");
    report.markers
}

/// One real `cng plan decompose`-equivalent run's real 9-entry planning
/// marker map (mirrors `tests/cng_production_ready.rs`'s
/// `real_planning_markers`).
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
        "urn:cng:test:production-ready-three-way:potato",
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

/// Runs the compiled `cng` binary to completion, capturing stdout/stderr and
/// success. A local reimplementation of `cng_multi_engine.rs`'s helper of
/// the same name/shape — duplicated rather than imported, to keep this file
/// independent of that test binary's private items. O(child runtime).
fn run_cng(args: &[&str]) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_cng"))
        .args(args)
        .output()
        .expect("spawn cng binary");
    (
        String::from_utf8(output.stdout).expect("utf-8 stdout"),
        String::from_utf8(output.stderr).expect("utf-8 stderr"),
        output.status.success(),
    )
}

/// Runs one real `cng engine serve` OS process to completion (serialized:
/// no `--poll-wait-ms`, so it drains its inbox and exits within
/// `max_polls`). A local reimplementation of `cng_multi_engine.rs`'s
/// `serve_to_budget` helper. O(child runtime).
fn serve_to_budget(root: &Path, engine_id: &str, max_polls: &str) {
    let seed_text = SEED.to_string();
    let (_stdout, stderr, ok) = run_cng(&[
        "engine",
        "serve",
        "--root",
        root.to_str().expect("utf-8 root"),
        "--engine-id",
        engine_id,
        "--seed",
        seed_text.as_str(),
        "--max-polls",
        max_polls,
    ]);
    assert!(ok, "engine serve {engine_id} failed: {stderr}");
}

/// One real two-engine (H, M) coordination round: dispatch phase (in-process,
/// public `engine_dispatch_remote`), two real `cng engine serve` OS
/// processes run to completion in turn (serialized determinism pin, same
/// pattern as `cng_multi_engine.rs`'s `serialized_run`), then the collect
/// phase (public `engine_collect_remote`) — whose `EngineCoordinateReport`
/// already carries the real, evidence-derived 9-entry
/// `DISTRIBUTED_MARKER_MAP` map on its public `markers` field. Returns that
/// map directly: no new marker-evaluation machinery is written here, and no
/// visibility bump was required anywhere in `src/bench/engine.rs` to reach
/// it.
///
/// # Complexity
/// O(contracts) dispatch/collect + two bounded child-process serve loops.
fn real_distributed_markers(root: &Path) -> BTreeMap<String, bool> {
    let dispatched =
        engine_dispatch_remote(root, "C", &["H", "M"], 2, 0, 0, SEED).expect("real dispatch phase");
    assert!(dispatched > 0, "dispatch phase must address contracts");
    serve_to_budget(root, "H", "4");
    serve_to_budget(root, "M", "4");
    let report = engine_collect_remote(root, "C", &["H", "M"], 2, 0, 0, SEED, 4, None)
        .expect("real collect phase");
    assert_eq!(report.contracts_dispatched, dispatched);
    assert_eq!(
        report.consequences_admitted, dispatched,
        "every dispatched contract must be admitted back: {}",
        report.consequences_admitted
    );
    assert_eq!(
        report.engine_instances, 2,
        "two real engine OS processes must be graph-receipted as engine_started"
    );
    assert_eq!(
        report.markers.len(),
        9,
        "distributed marker count drifted: {:?}",
        report.markers
    );
    report.markers
}

test!(
    full_production_ready_holds_on_real_triple_bundle_evidence,
    {
        // Arrange + Act: three real bundles, each producing its own real
        // evidence-derived marker map — the genuine three-way surface.
        let workday_markers = real_workday_markers(&scratch_dir("workday"));
        let planning_markers = real_planning_markers(&scratch_dir("planning"));
        let distributed_markers = real_distributed_markers(&scratch_dir("distributed"));
        assert!(
            workday_markers.values().all(|v| *v),
            "every real workday marker must be true: {workday_markers:?}"
        );
        assert!(
            planning_markers.values().all(|v| *v),
            "every real planning marker must be true: {planning_markers:?}"
        );
        assert!(
            distributed_markers.values().all(|v| *v),
            "every real distributed marker must be true: {distributed_markers:?}"
        );

        // Act: the combinator under test (PROJ-742), fed REAL marker maps from
        // ALL THREE real bundles together — the exact three-way invocation
        // that had never happened before this test.
        let combined = full_production_ready(
            &workday_markers,
            &planning_markers,
            Some(&distributed_markers),
        );

        // Assert: 16 workday-named markers (interim conjunction dropped, then
        // recomputed) + 9 planning markers + 3 distributed-only markers not
        // already named on the workday side (MULTI_ENGINE_EXECUTION_PROVEN,
        // ENGINE_INSTANCES_PROVEN, ARAZZO_INTER_ENGINE_WORKFLOW_PROVEN — the
        // other 6 distributed keys overwrite same-named workday-side entries)
        // + the recomputed conjunction = 29.
        assert_eq!(combined.len(), 29, "combined marker count: {combined:?}");
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
        // Distributed-side markers, including the three keys ONLY the
        // distributed bundle can name (proof the third argument was genuinely
        // consumed, not merely accepted and ignored).
        for marker in [
            "SHARED_MEMORY_CROSSINGS_ZERO",
            "DIRECT_ENGINE_BYPASSES_ZERO",
            "REMOTE_WORKFLOWS_ACKNOWLEDGED",
            "REMOTE_WORKFLOWS_COMPLETED",
            "REPLAY_DIVERGENCES_ZERO",
            "ARAZZO_WORKFLOWS_DISPATCHED",
            "MULTI_ENGINE_EXECUTION_PROVEN",
            "ENGINE_INSTANCES_PROVEN",
            "ARAZZO_INTER_ENGINE_WORKFLOW_PROVEN",
        ] {
            assert_eq!(
                combined.get(marker),
                Some(&true),
                "distributed-side marker {marker} missing or false: {combined:?}"
            );
        }
        assert_eq!(
            combined.get("V26_7_10_PRODUCTION_READY"),
            Some(&true),
            "combined conjunction must hold on healthy real triple-bundle evidence: {combined:?}"
        );
    }
);

test!(
    full_production_ready_goes_false_when_a_real_distributed_marker_is_forced_false,
    {
        // Arrange: the SAME real triple-bundle evidence as the positive
        // test above.
        let workday_markers = real_workday_markers(&scratch_dir("workday-negative"));
        let planning_markers = real_planning_markers(&scratch_dir("planning-negative"));
        let distributed_markers = real_distributed_markers(&scratch_dir("distributed-negative"));

        // Control: the unmodified real triple holds true (this combinator
        // is not trivially always-false).
        let healthy = full_production_ready(
            &workday_markers,
            &planning_markers,
            Some(&distributed_markers),
        );
        assert_eq!(healthy.get("V26_7_10_PRODUCTION_READY"), Some(&true));

        // Act: force exactly one DISTRIBUTED-side marker false in a clone —
        // one that names a key no workday/planning marker also names
        // (`ENGINE_INSTANCES_PROVEN`), proving the third argument alone can
        // sink the conjunction, not merely overwrite an already-true entry
        // from the other two bundles.
        let mut broken_distributed = distributed_markers.clone();
        broken_distributed.insert("ENGINE_INSTANCES_PROVEN".to_string(), false);
        let combined = full_production_ready(
            &workday_markers,
            &planning_markers,
            Some(&broken_distributed),
        );
        assert_eq!(combined.get("ENGINE_INSTANCES_PROVEN"), Some(&false));
        assert_eq!(
            combined.get("V26_7_10_PRODUCTION_READY"),
            Some(&false),
            "a false distributed-side marker must sink the combined \
             conjunction: {combined:?}"
        );
    }
);

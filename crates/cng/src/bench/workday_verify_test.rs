#![cfg(test)]

//! PROJ-616 verification-harness tests: independent workday replay
//! (untampered pass, mutated-OCEL CNG_R11, stripped-receipt CNG_R13),
//! Dialect Registry tamper (CNG_R14), forged inbox correlation (CNG_R17),
//! and the parent-closure-unsatisfied open-state law. Fixture RDF enters
//! only from on-disk files (`tests/fixtures/negative/`) or from the workday
//! generator itself; all SPARQL loads from the on-disk query set. Tampering
//! is byte-level mutation of run-produced files — no Turtle is authored
//! inline.

use std::fs;
use std::path::PathBuf;

use chicago_tdd_tools::prelude::*;
use oxigraph::model::{LiteralRef, NamedNodeRef, TermRef};
use oxigraph::store::Store;

use super::{assemble_workday_manifest, workday_replay};
use crate::bench::dispatch::{
    collect_consequence, workday_contract, DispatchAdapter, DispatchOutcome, ExecutionClass,
    SynthesisMode,
};
use crate::bench::hooks::WorkdayHookBroker;
use crate::bench::roles::ObsWriter;
use crate::bench::templates::{load_templates, QuerySet};
use crate::bench::{workday, WorkdayConfig};
use crate::powl::CngRefusal;

/// Scratch root for this test file. O(1).
fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!(
            "../../target/chatman/cng-tests/workday-verify_{}",
            std::process::id()
        ))
        .join(test_name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// Crate-root path helper. O(1).
fn crate_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Runs a small deterministic workday into `dir`. O(ticks) manufactures.
fn small_workday(dir: &PathBuf) -> crate::bench::WorkdayReport {
    let cfg = WorkdayConfig {
        seed: 42,
        ticks: 3,
        refusal_per_mille: 0,
    };
    workday(dir, &cfg, None).expect("workday runs")
}

/// Count of observations with the given obsKind in `store` (typed oxigraph
/// pattern scan; no SPARQL). O(matches).
fn kind_count(store: &Store, kind: &str) -> usize {
    let pred = NamedNodeRef::new("https://ggen.io/ontology/bench-obs#obsKind")
        .expect("obsKind IRI is valid");
    let lit = LiteralRef::new_simple_literal(kind);
    store
        .quads_for_pattern(None, Some(pred), Some(TermRef::from(lit)), None)
        .count()
}

test!(untampered_workday_bundle_replays_and_manifests, {
    // Arrange: one finished workday bundle.
    let dir = scratch_dir("untampered");
    let report = small_workday(&dir);

    // Act: independent replay + manifest assembly over the bundle alone.
    let replay = workday_replay(&dir, None).expect("untampered bundle replays");
    let manifest = assemble_workday_manifest(&dir).expect("manifest assembles");

    // Assert: replay reconciled every recorded hook receipt against the
    // graph-derived count, digests matched, and the manifest covers the
    // OCEL serialization plus every obs partition.
    assert_eq!(
        replay.hook_receipt_observations as u64,
        report.hook_receipts
    );
    assert!(replay.obs_digest_match);
    assert!(replay.ocel_graph_digest_match);
    assert!(replay.ocel_serialization_match);
    assert_eq!(
        replay.recomputed_ocel_graph_digest,
        report.ocel_graph_digest
    );
    assert!(manifest.contains_key("evidence/ocel.nt"));
    assert!(manifest.keys().any(|k| k.starts_with("obs/")));
    assert!(dir
        .join("results")
        .join("workday-bundle-manifest.json")
        .is_file());
});

test!(mutated_ocel_evidence_refuses_cng_r11, {
    // Arrange: a finished bundle whose evidence/ocel.nt loses one recorded
    // triple (byte-level tamper of a run-produced file; nothing authored).
    let dir = scratch_dir("mutated_ocel");
    small_workday(&dir);
    let ocel_path = dir.join("evidence").join("ocel.nt");
    let recorded = fs::read_to_string(&ocel_path).expect("ocel.nt reads");
    let truncated: Vec<&str> = recorded.lines().collect();
    assert!(truncated.len() > 1, "evidence has multiple triples");
    fs::write(&ocel_path, truncated[..truncated.len() - 1].join("\n"))
        .expect("tampered ocel.nt writes");

    // Act.
    let result = workday_replay(&dir, None);

    // Assert: typed CNG_R11 third-party integrity refusal.
    match result {
        Err(refusal @ CngRefusal::AuditMismatch(_)) => {
            assert_eq!(refusal.code(), "CNG_R11");
        }
        other => panic!("expected AuditMismatch, got {other:?}"),
    }
});

test!(mutated_obs_partition_refuses_cng_r11, {
    // Arrange: a finished bundle with one obs partition truncated (the
    // recorded observation stream no longer matches the recorded digests).
    let dir = scratch_dir("mutated_obs");
    small_workday(&dir);
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

    // Act.
    let result = workday_replay(&dir, None);

    // Assert: typed CNG_R11 (unparseable/mutated bundle input).
    match result {
        Err(refusal @ CngRefusal::AuditMismatch(_)) => {
            assert_eq!(refusal.code(), "CNG_R11");
        }
        other => panic!("expected AuditMismatch, got {other:?}"),
    }
});

test!(stripped_hook_delta_hash_refuses_cng_r13, {
    // Arrange: a finished bundle whose hook_receipt observations lose their
    // ex:hookDeltaHash lines (line-level strip of run-produced Turtle; the
    // predicate list stays parseable).
    let dir = scratch_dir("stripped_delta_hash");
    let report = small_workday(&dir);
    assert!(report.hook_receipts > 0, "run produced hook receipts");
    let mut stripped_any = false;
    for entry in fs::read_dir(dir.join("obs"))
        .expect("obs dir reads")
        .flatten()
    {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("ttl") {
            continue;
        }
        let body = fs::read_to_string(&path).expect("obs partition reads");
        if body.contains("hookDeltaHash") {
            let kept: Vec<&str> = body
                .lines()
                .filter(|line| !line.contains("hookDeltaHash"))
                .collect();
            fs::write(&path, kept.join("\n")).expect("stripped partition writes");
            stripped_any = true;
        }
    }
    assert!(stripped_any, "at least one receipt was stripped");

    // Act: reconcile runs BEFORE any digest comparison, so the stripped
    // receipt is named as the actuation-law violation.
    let result = workday_replay(&dir, None);

    // Assert: typed CNG_R13 naming a workflow and category.
    match result {
        Err(CngRefusal::UnreceiptedActuation { workflow, category }) => {
            assert!(!workflow.is_empty());
            assert!(!category.is_empty());
            assert_eq!(
                CngRefusal::UnreceiptedActuation { workflow, category }.code(),
                "CNG_R13"
            );
        }
        other => panic!("expected UnreceiptedActuation, got {other:?}"),
    }
});

test!(stripped_dialect_registry_field_refuses_cng_r14, {
    // Arrange: the on-disk fixture registry whose entry lacks
    // dreg:receiptSchema, run through the same gate the workday runs
    // BEFORE any tick.
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");

    // Act.
    let result = WorkdayHookBroker::new(
        &crate_path("tests/fixtures/negative/dialect-registry-missing-field.ttl"),
        &crate_path("hooks/dialect-registry.shape.ttl"),
        &[crate_path("hooks/workday-pack.ttl")],
        &queries,
    );

    // Assert: typed CNG_R14 naming the entry and the stripped field.
    match result {
        Err(CngRefusal::DialectRegistryRefused { entry, missing }) => {
            assert!(missing.contains("receiptSchema"), "got {missing}");
            assert_eq!(
                CngRefusal::DialectRegistryRefused { entry, missing }.code(),
                "CNG_R14"
            );
        }
        other => panic!("expected DialectRegistryRefused, got {other:?}"),
    }
});

test!(forged_inbox_correlation_refuses_cng_r17, {
    // Arrange: the on-disk forged-correlation consequence fixture against
    // its matching contract (actor correct, correlation forged — so the
    // pipeline must pass provenance and refuse at exactly correlation).
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    let mut contract = workday_contract(
        "fixture",
        "software-delivery",
        1,
        ExecutionClass::ExternalMachineDispatch,
    );
    contract.recursive_depth = 0;
    contract.closure_law = None;
    let consequence_ttl = fs::read_to_string(crate_path(
        "tests/fixtures/negative/dispatch-consequence-forged-correlation.ttl",
    ))
    .expect("fixture reads");

    // Act.
    let result = collect_consequence(
        &consequence_ttl,
        &contract,
        &crate_path("shapes/dispatch-shapes.ttl"),
        &queries,
    );

    // Assert: typed CNG_R17 at the correlation stage.
    match result {
        Err(CngRefusal::ExternalConsequenceRefused { dispatch, stage }) => {
            assert_eq!(dispatch, "disp-fixture");
            assert_eq!(stage, "correlation");
            assert_eq!(
                CngRefusal::ExternalConsequenceRefused { dispatch, stage }.code(),
                "CNG_R17"
            );
        }
        other => panic!("expected ExternalConsequenceRefused, got {other:?}"),
    }
});

test!(unsatisfied_parent_closure_keeps_parent_open, {
    // Arrange: a depth-1 ALL_CHILDREN_REQUIRED parent whose closure query
    // is evaluated over an EMPTY observation store (the writer emits into a
    // separate store), so the children complete but the parent's closure
    // law can never be satisfied.
    let out_dir = scratch_dir("closure_open");
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    let templates = load_templates().expect("templates load");
    let emission_store = Store::new().expect("emission store");
    let closure_store = Store::new().expect("closure store");
    let mut writer =
        ObsWriter::new(&templates, &emission_store, &out_dir.join("obs"), "test").expect("writer");
    let mut adapter = DispatchAdapter::new(&out_dir, &queries).expect("adapter constructs");
    let contract = workday_contract(
        "closure-open",
        "software-delivery",
        1,
        ExecutionClass::ExternalMachineDispatch,
    );
    let parent_id = contract.dispatch_id.clone();

    // Act: full lifecycle; children run through the SAME broker path.
    let outcome = adapter
        .dispatch(
            &mut writer,
            &closure_store,
            contract,
            1,
            false,
            SynthesisMode::LoopbackDeterministic,
            1,
        )
        .expect("lifecycle completes (an open parent is a state, not an error)");

    // Assert: the parent stays open — no false COMPLETED. Both children
    // completed (their consequences admitted in the emission evidence),
    // but the parent never admitted a consequence and never entered the
    // receipt chain.
    assert_eq!(outcome, DispatchOutcome::Open);
    assert_eq!(kind_count(&emission_store, "consequence_admitted"), 2);
    assert_eq!(kind_count(&emission_store, "dispatch_sent"), 3);
    assert!(
        !adapter.receipt_digests.contains_key(&parent_id),
        "an open parent must never enter the receipt chain"
    );
    assert_eq!(adapter.telemetry.admitted, 2);
});

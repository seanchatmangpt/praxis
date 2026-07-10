//! PROJ-601: digests.json path-portability. Proves `run()` writes
//! bench_dir-relative keys and `verify()` can replay a bundle after it has
//! been copied to a different path (relocation). Fixture data enters only
//! via `cng::bench::generate`; no inline Turtle/SPARQL in this file.

#![cfg(feature = "bench")]

use std::fs;
use std::path::{Path, PathBuf};

use chicago_tdd_tools::prelude::*;

use cng::bench::{self, BenchConfig};

/// Scratch root for this test file, following the pattern in
/// `cng_pipeline.rs`. O(1).
fn scratch_dir(test_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/portability")
        .join(test_name)
}

/// Smallest generator config that yields at least 2 workload sets.
fn small_config() -> BenchConfig {
    BenchConfig {
        workers: 64,
        artifact_sets: 2,
        recursion_depth: 1,
        seed: 42,
        refusal_per_mille: 0,
    }
}

/// Recursively copies `src` into `dst` using only `std::fs` (no shell).
///
/// # Complexity
/// O(files + bytes) under `src`.
fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create destination dir");
    for entry in fs::read_dir(src).expect("read source dir") {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target);
        } else {
            fs::copy(&path, &target).expect("copy file");
        }
    }
}

test!(digests_keys_are_bench_dir_relative, {
    let dir = scratch_dir("digests_keys_are_bench_dir_relative");
    let _ = fs::remove_dir_all(&dir);
    let cfg = small_config();
    bench::generate(&dir, &cfg).expect("generate");
    bench::run(&dir, 1, 500, None).expect("run");

    let digests_path = dir.join("results").join("digests.json");
    let raw = fs::read_to_string(&digests_path).expect("read digests.json");
    let digests: std::collections::BTreeMap<String, String> =
        serde_json::from_str(&raw).expect("parse digests.json");
    assert!(
        !digests.is_empty(),
        "digests.json must record at least one set"
    );

    let scratch_prefix = dir.display().to_string();
    for key in digests.keys() {
        assert!(
            !key.starts_with('/'),
            "digests.json key must be bench-dir-relative, not absolute: {key}"
        );
        assert!(
            !key.contains(&scratch_prefix),
            "digests.json key must not contain the scratch dir prefix: {key}"
        );
    }
});

test!(verify_replays_after_directory_move, {
    let a = scratch_dir("verify_replays_after_directory_move_a");
    let b = scratch_dir("verify_replays_after_directory_move_b");
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
    let cfg = small_config();
    bench::generate(&a, &cfg).expect("generate");
    bench::run(&a, 1, 1000, None).expect("run");

    copy_dir_recursive(&a, &b);
    fs::remove_dir_all(&a).expect("remove original bench dir");

    let report = bench::verify(&b, 1, 1).expect("verify after relocation");
    assert!(report.replayed > 0, "verify must replay at least one set");
    assert_eq!(
        report.replay_passes, report.replayed,
        "every replayed set must match its recorded digest after relocation"
    );

    let _ = fs::remove_dir_all(&b);
});

test!(evidence_manifest_is_complete_and_relative, {
    let dir = scratch_dir("evidence_manifest_is_complete_and_relative");
    let _ = fs::remove_dir_all(&dir);
    let cfg = small_config();
    bench::generate(&dir, &cfg).expect("generate");
    bench::run(&dir, 1, 500, None).expect("run");

    let manifest_path = dir.join("results").join("evidence-manifest.json");
    let raw = fs::read_to_string(&manifest_path).expect("read evidence-manifest.json");
    let manifest: bench::EvidenceManifest =
        serde_json::from_str(&raw).expect("parse evidence-manifest.json");

    assert!(
        manifest.signatures.is_empty(),
        "signatures must be an empty array until signing is wired"
    );
    assert_eq!(manifest.schema_version, 1);
    assert!(
        manifest.query_digests.len() >= 15,
        "expected at least 15 query digests, got {}",
        manifest.query_digests.len()
    );
    assert_eq!(
        manifest
            .ontology_digests
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        vec!["bench-obs.ttl".to_string(), "ocel2.ttl".to_string()],
        "ontology_digests must have exactly the two expected keys"
    );

    let scratch_prefix = dir.display().to_string();
    assert!(
        !raw.contains(&scratch_prefix),
        "evidence-manifest.json must not contain the scratch dir absolute prefix"
    );

    assert!(dir.join("queries").is_dir(), "queries/ bundle copy missing");
    assert!(
        dir.join("ontology").is_dir(),
        "ontology/ bundle copy missing"
    );
    assert!(
        dir.join("rules").join("bench-roles.dl").is_file(),
        "rules/bench-roles.dl bundle copy missing"
    );

    let _ = fs::remove_dir_all(&dir);
});

test!(audit_replay_conformant_on_untouched_bundle, {
    let dir = scratch_dir("audit_replay_conformant_on_untouched_bundle");
    let _ = fs::remove_dir_all(&dir);
    let cfg = small_config();
    bench::generate(&dir, &cfg).expect("generate");
    bench::run(&dir, 1, 500, None).expect("run");

    let report = bench::audit_replay(&dir).expect("audit replay of untouched bundle");
    assert!(report.obs_digest_match, "obs digest must match");
    assert!(
        report.ocel_graph_digest_match,
        "OCEL graph digest must match"
    );
    assert!(
        report.obs_files_hashed > 0,
        "must hash at least one obs file"
    );
    assert!(
        report.queries_verified >= 15,
        "expected at least 15 verified queries, got {}",
        report.queries_verified
    );

    let _ = fs::remove_dir_all(&dir);
});

test!(audit_replay_refuses_tampered_observation, {
    let src = scratch_dir("audit_replay_refuses_tampered_observation_src");
    let dir = scratch_dir("audit_replay_refuses_tampered_observation");
    let _ = fs::remove_dir_all(&src);
    let _ = fs::remove_dir_all(&dir);
    let cfg = small_config();
    bench::generate(&src, &cfg).expect("generate");
    bench::run(&src, 1, 500, None).expect("run");
    copy_dir_recursive(&src, &dir);
    fs::remove_dir_all(&src).expect("remove original bench dir");

    let obs_dir = dir.join("obs");
    let mut tampered = None;
    for entry in fs::read_dir(&obs_dir).expect("read obs dir") {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("ttl") {
            tampered = Some(path);
            break;
        }
    }
    let tampered = tampered.expect("at least one obs .ttl file");
    let mut contents = fs::read_to_string(&tampered).expect("read obs file");
    contents.push_str("\n# tampered\n");
    fs::write(&tampered, contents).expect("write tampered obs file");

    let result = bench::audit_replay(&dir);
    match result {
        Err(refusal @ cng::powl::CngRefusal::AuditMismatch(_)) => {
            assert_eq!(refusal.code(), "CNG_R11");
        }
        other => panic!("expected AuditMismatch/CNG_R11, got {other:?}"),
    }

    let _ = fs::remove_dir_all(&dir);
});

test!(audit_replay_refuses_tampered_query, {
    let src = scratch_dir("audit_replay_refuses_tampered_query_src");
    let dir = scratch_dir("audit_replay_refuses_tampered_query");
    let _ = fs::remove_dir_all(&src);
    let _ = fs::remove_dir_all(&dir);
    let cfg = small_config();
    bench::generate(&src, &cfg).expect("generate");
    bench::run(&src, 1, 500, None).expect("run");
    copy_dir_recursive(&src, &dir);
    fs::remove_dir_all(&src).expect("remove original bench dir");

    let query_path = dir.join("queries").join("ocel-events.construct.rq");
    let mut contents = fs::read_to_string(&query_path).expect("read query file");
    contents.push_str("\n# tampered\n");
    fs::write(&query_path, contents).expect("write tampered query file");

    let result = bench::audit_replay(&dir);
    match result {
        Err(cng::powl::CngRefusal::AuditMismatch(msg)) => {
            assert_eq!(
                cng::powl::CngRefusal::AuditMismatch(msg.clone()).code(),
                "CNG_R11"
            );
            assert!(
                msg.contains("ocel-events.construct"),
                "message must name the tampered stem: {msg}"
            );
        }
        other => panic!("expected AuditMismatch/CNG_R11 naming the stem, got {other:?}"),
    }

    let _ = fs::remove_dir_all(&dir);
});

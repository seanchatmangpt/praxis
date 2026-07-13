//! Negative-path proof for the finding #6 fix: `classify_artifact`'s four
//! fallible operations (`fs::read_to_string`, `Store::new`, `load_from_slice`,
//! and `first_object`'s pattern scan) used to collapse into one `.ok()?`
//! chain, so every failure — a genuine I/O fault, a store-construction
//! fault, and an actual malformed-Turtle parse fault — surfaced as the same
//! `None`, which `manufacture_set` then uniformly mislabeled `CNG_R01
//! MalformedTtl`. These tests drive the real (now `Result`-returning)
//! `classify_artifact` directly and prove an I/O fault and a Turtle-parse
//! fault now produce distinct, correctly-coded refusals.

use std::fs;
use std::path::PathBuf;

use super::classify_artifact;
use crate::powl::CngRefusal;

/// Shared scratch root, isolated from other test binaries' use of
/// `target/chatman/cng-tests` (same convention as
/// `tests/cng_io_refused_negative.rs`).
fn scratch_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("chatman")
        .join("cng-tests")
        .join("negative")
        .join("manufacture-classify-artifact")
}

/// Crate-root path helper, same convention as `dispatch_test.rs`'s own
/// `crate_path`. O(1).
fn crate_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// `classify_artifact` on a path that names a DIRECTORY, not a file: its
/// very first operation, `fs::read_to_string`, fails with a genuine OS I/O
/// error (`IsADirectory`/`EISDIR` on Unix; `ERROR_ACCESS_DENIED` reading a
/// directory as a file on Windows) — never a Turtle-parse fault, and never a
/// permission-bit trick. This is the failure the old `.ok()?` chain
/// silently relabeled `CNG_R01 MalformedTtl`.
#[test]
fn classify_artifact_directory_path_refuses_cng_r10_io_refused_not_malformed_ttl() {
    let dir = scratch_root().join("directory-not-a-file");
    fs::create_dir_all(&dir).expect("create scratch directory to use as the bogus artifact path");

    match classify_artifact(&dir) {
        Err(refusal @ CngRefusal::IoRefused(_)) => {
            assert_eq!(refusal.code(), "CNG_R10");
            let message = refusal.message();
            let expected_path = dir.display().to_string();
            assert!(
                message.contains(&expected_path),
                "IoRefused message must name the failing path {expected_path}: got {message:?}"
            );
            assert!(
                message.contains("read"),
                "IoRefused message should identify the failing read operation: got {message:?}"
            );
        }
        Err(other) => panic!(
            "a directory path must refuse CNG_R10 IoRefused (genuine I/O fault), \
             not {other:?} (code {}) — an I/O failure must never be mislabeled a Turtle defect",
            other.code()
        ),
        Ok(result) => {
            panic!("expected CNG_R10 IoRefused for a directory path, got {result:?} instead")
        }
    }
}

/// `classify_artifact` on a file that genuinely is not valid Turtle: the
/// read and store-construction stages succeed, and only `load_from_slice`
/// (the actual parse) fails — the one case that legitimately deserves
/// `CNG_R01 MalformedTtl`. Distinguishing this from the directory-path test
/// above proves the two failure causes are no longer collapsed into a
/// single indistinguishable `None`.
#[test]
fn classify_artifact_malformed_turtle_refuses_cng_r01_malformed_ttl() {
    let dir = scratch_root().join("malformed-turtle");
    fs::create_dir_all(&dir).expect("create scratch dir for malformed-turtle fixture");
    let path = dir.join("bad.ttl");
    fs::write(&path, b"this is not { valid ttl ]] at all :: ---\n")
        .expect("write intentionally-invalid Turtle fixture");

    match classify_artifact(&path) {
        Err(refusal @ CngRefusal::MalformedTtl(_)) => {
            assert_eq!(refusal.code(), "CNG_R01");
            let message = refusal.message();
            let expected_path = path.display().to_string();
            assert!(
                message.contains(&expected_path),
                "MalformedTtl message must name the failing path {expected_path}: got {message:?}"
            );
        }
        Err(other) => panic!(
            "genuinely malformed Turtle must refuse CNG_R01 MalformedTtl, \
             not {other:?} (code {})",
            other.code()
        ),
        Ok(result) => panic!(
            "expected CNG_R01 MalformedTtl for genuinely invalid Turtle bytes, \
             got {result:?} instead"
        ),
    }
}

/// `classify_artifact` on syntactically valid Turtle that is simply missing
/// `ex:category`/`ex:worker` is neither an I/O fault nor a malformed-Turtle
/// fault — it is a real `Ok(None)`, not collapsed into either refusal.
#[test]
fn classify_artifact_valid_turtle_missing_predicates_returns_ok_none() {
    let dir = scratch_root().join("valid-turtle-no-classification-predicates");
    fs::create_dir_all(&dir).expect("create scratch dir for well-formed-but-unrelated fixture");
    let path = dir.join("unrelated.ttl");
    let fixture = fs::read_to_string(crate_path(
        "tests/fixtures/negative/manufacture-unrelated-predicates.ttl",
    ))
    .expect("read manufacture-unrelated-predicates.ttl fixture");
    fs::write(&path, fixture)
        .expect("write well-formed Turtle fixture lacking category/worker predicates");

    match classify_artifact(&path) {
        Ok(None) => {}
        Ok(Some(result)) => panic!(
            "expected Ok(None) for Turtle with no ex:category/ex:worker triples, \
             got a classification {result:?} instead"
        ),
        Err(refusal) => panic!(
            "well-formed Turtle missing classification predicates must not refuse \
             ({refusal:?}, code {}) — it is a real Ok(None), not a failure",
            refusal.code()
        ),
    }
}

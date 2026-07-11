//! CNG_R10 `IoRefused` negative-path proof (docs/releases/v26.7.10/GAP_AUDIT.md
//! section 3.1/3.4 item 1): `IoRefused` is constructed at roughly 130+ call
//! sites across `crates/cng/src/`, yet before this file none of them had a
//! test proving the refusal actually fires through the public API — it was
//! the single most pervasive refusal in the crate, never exercised on its
//! negative path.
//!
//! Representative call site chosen: `crates/cng/src/pipeline.rs:74`, inside
//! `pipeline::import_artifacts` — the public entry point every planning
//! artifact the cng CLI ever admits must pass through. Its very first
//! filesystem operation is `fs::read_dir(dir)` over the caller-supplied
//! artifact directory:
//!
//! ```text
//! let entries = fs::read_dir(dir).map_err(|e| {
//!     CngRefusal::IoRefused(format!("cannot read artifact dir {}: {e}", dir.display()))
//! })?;
//! ```
//!
//! Forcing this deterministically needs no OS-permission trick: pointing
//! `dir` at a path that is guaranteed never to exist makes `fs::read_dir`
//! fail with `io::ErrorKind::NotFound` on every platform, every run.

use std::fs;
use std::path::PathBuf;

use cng::pipeline::import_artifacts;
use cng::powl::CngRefusal;

/// A path guaranteed not to exist on disk when `import_artifacts` is
/// called: neither it nor its parent is ever created via
/// `fs::create_dir_all`, so `fs::read_dir` cannot silently succeed on an
/// auto-created empty directory. `fs::read_dir` on a nonexistent path fails
/// with `io::ErrorKind::NotFound` regardless of platform or permissions —
/// no fragile Unix `set_permissions` trick required.
fn nonexistent_artifact_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("chatman")
        .join("cng-tests")
        .join("negative")
        .join("io-refused-cng-r10")
        .join("nonexistent-artifact-dir");
    // Belt-and-suspenders: if a stale run of this exact test somehow left
    // this path behind, remove it so the test still forces the real
    // NotFound I/O failure rather than a false pass against leftover state.
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("stale nonexistent_artifact_dir fixture must be removable");
    }
    assert!(
        !dir.exists(),
        "precondition: {} must not exist before import_artifacts is called",
        dir.display()
    );
    dir
}

/// `import_artifacts` on a directory that does not exist on disk must
/// refuse with `CNG_R10 IoRefused`, and the refusal's message must name
/// the exact failing path — proving the refusal fires through the real
/// public API (not a mock, not an asserted-but-untriggered variant).
#[test]
fn import_artifacts_missing_dir_refuses_cng_r10_io_refused() {
    let dir = nonexistent_artifact_dir();

    match import_artifacts(&dir) {
        Err(refusal @ CngRefusal::IoRefused(_)) => {
            assert_eq!(refusal.code(), "CNG_R10");
            let message = refusal.message();
            let expected_path = dir.display().to_string();
            assert!(
                message.contains(&expected_path),
                "IoRefused message must name the failing path {expected_path}: got {message:?}"
            );
            assert!(
                message.contains("cannot read artifact dir"),
                "IoRefused message should identify the failing operation: got {message:?}"
            );
        }
        Err(other) => panic!(
            "expected CNG_R10 IoRefused, got {other:?} (code {})",
            other.code()
        ),
        Ok(artifacts) => panic!(
            "expected CNG_R10 IoRefused for a nonexistent artifact directory, \
             got {} imported artifacts instead",
            artifacts.len()
        ),
    }
}

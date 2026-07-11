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
//!
//! PROJ-CNG_R10-expand (v26.7.10-revised, 80/20 follow-up to the above):
//! GAP_AUDIT.md's ~130+ construction sites span `bench/*.rs`, `pipeline.rs`,
//! and `bench/decomp/*.rs` — three distinct modules under the `cng::bench`
//! feature-gated surface, none proven beyond the single `pipeline.rs` site
//! above. The `cng_r10_bench_sites` module below adds three more
//! representative sites, each in a MODULE `pipeline.rs` does not touch, to
//! prove the pattern holds crate-wide rather than at one accidental
//! location:
//!
//! - `crates/cng/src/bench/workday.rs:588`
//!   (`build_decomp_marker_store`) — same "nonexistent path" technique as
//!   the pipeline site above (`fs::read_to_string` → `NotFound`).
//! - `crates/cng/src/bench/workday_verify.rs:472`
//!   (`assemble_workday_manifest`) — a PATH-COLLISION technique: `out_dir`
//!   is pointed at a pre-existing plain file, so `fs::create_dir_all`
//!   cannot create a directory there (`io::ErrorKind::AlreadyExists` /
//!   `NotADirectory`, deterministic on every platform, no `chmod` trick).
//! - `crates/cng/src/bench/decomp/mod.rs:726`
//!   (`decompose`'s internal `emit_result_graph`) — same path-collision
//!   technique, driven through the real no-LLM decomposition pipeline
//!   over the checked-in `examples/pddl-strips-potato.ttl` fixture (the
//!   same fixture `tests/cng_decomp.rs` uses), so the forced mkdir failure
//!   fires only after grounding/planning/edge-derivation genuinely
//!   succeed — proving the refusal fires from real production code, not a
//!   short-circuited stub.
//!
//! This is explicitly 80/20, not exhaustive: three additional sites proving
//! the construction pattern holds across `bench/`, `bench/decomp/`, and
//! `pipeline.rs`, not every one of GAP_AUDIT's ~130+ call sites.

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

/// PROJ-CNG_R10-expand: three more representative `CNG_R10 IoRefused`
/// construction sites, each in a module `pipeline.rs` (above) does not
/// touch — `bench/workday.rs`, `bench/workday_verify.rs`, and
/// `bench/decomp/mod.rs` — gated behind the `bench` feature the same way
/// `tests/cng_decomp.rs` and `tests/cng_multi_engine.rs` are. See the file
/// header doc for the exact construction-site line references.
#[cfg(feature = "bench")]
mod cng_r10_bench_sites {
    use std::fs;
    use std::path::{Path, PathBuf};

    use cng::bench::build_decomp_marker_store;
    use cng::bench::decomp::{decomp_queries_dir, decompose, strips_graph_to_surface};
    use cng::bench::workday_verify::assemble_workday_manifest;
    use cng::bench::QuerySet;
    use cng::powl::CngRefusal;
    use oxigraph::io::{RdfFormat, RdfParser};
    use oxigraph::store::Store;

    /// Shared scratch root for this module's fixtures, isolated from the
    /// outer `pipeline` test's own scratch tree and from any other test
    /// binary's `target/chatman/cng-tests` usage.
    fn scratch_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("target")
            .join("chatman")
            .join("cng-tests")
            .join("negative")
            .join("io-refused-cng-r10-expand")
    }

    /// A file path guaranteed not to exist on disk, nested under a
    /// per-test subdirectory that is itself removed (if a stale run left
    /// one behind) and never recreated — `fs::read_to_string` on it fails
    /// with `io::ErrorKind::NotFound` on every platform, every run. Same
    /// technique as the outer `pipeline` test's `nonexistent_artifact_dir`.
    fn nonexistent_file_path(test_name: &str) -> PathBuf {
        let dir = scratch_root().join("nonexistent").join(test_name);
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("stale nonexistent-path fixture must be removable");
        }
        assert!(
            !dir.exists(),
            "precondition: {} must not exist before the call under test",
            dir.display()
        );
        dir.join("decomposition-result.ttl")
    }

    /// A path that collides with a pre-existing plain FILE (not a
    /// directory), nested under a per-test name so concurrent test threads
    /// never share a fixture. `fs::create_dir_all` on this exact path, or
    /// on any path that treats it as a directory component, cannot ever
    /// succeed — a file already occupies that name — so the failure is
    /// deterministic on every platform without any Unix-only permission
    /// trick (`io::ErrorKind::AlreadyExists` when the path itself is the
    /// target, `NotADirectory` when a child of it is the target; verified
    /// directly against this platform's `std::fs` before being relied on
    /// here).
    fn file_collision_path(test_name: &str) -> PathBuf {
        let dir = scratch_root().join("file-collisions");
        fs::create_dir_all(&dir).expect("create scratch dir for collision fixtures");
        let path = dir.join(test_name);
        // Belt-and-suspenders: a stale run of this exact test may have left
        // either a directory or a file behind; remove either so the write
        // below deterministically leaves a plain file, never a directory.
        let _ = fs::remove_dir_all(&path);
        let _ = fs::remove_file(&path);
        fs::write(
            &path,
            b"CNG_R10 path-collision fixture: intentionally a plain file",
        )
        .expect("write collision fixture file");
        assert!(
            path.is_file(),
            "precondition: {} must exist as a plain file before the call under test",
            path.display()
        );
        path
    }

    /// The checked-in two-chain potato example graph
    /// (`tests/cng_decomp.rs` uses the same fixture) — loaded fresh per
    /// test so no test mutates shared state.
    fn potato_store() -> Store {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/pddl-strips-potato.ttl");
        let turtle = fs::read_to_string(&path).expect("read potato example graph");
        let store = Store::new().expect("store construction");
        store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
            .expect("potato example graph must parse");
        store
    }

    fn decomp_template(name: &str) -> String {
        fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("templates")
                .join(name),
        )
        .expect("read decomp template")
    }

    /// Site 2 (`bench/workday.rs`): `build_decomp_marker_store` on a
    /// `decomp:DecompositionResult` graph path that does not exist on disk
    /// must refuse `CNG_R10 IoRefused` naming the failing path — the exact
    /// same "nonexistent path" technique as `import_artifacts` above, now
    /// proven against a construction site in a different module.
    #[test]
    fn build_decomp_marker_store_missing_graph_refuses_cng_r10_io_refused() {
        let path = nonexistent_file_path("build-decomp-marker-store-missing-graph");

        match build_decomp_marker_store(&path) {
            Err(refusal @ CngRefusal::IoRefused(_)) => {
                assert_eq!(refusal.code(), "CNG_R10");
                let message = refusal.message();
                let expected_path = path.display().to_string();
                assert!(
                    message.contains(&expected_path),
                    "IoRefused message must name the failing path {expected_path}: got {message:?}"
                );
                assert!(
                    message.contains("read"),
                    "IoRefused message should identify the failing operation: got {message:?}"
                );
            }
            Err(other) => panic!(
                "expected CNG_R10 IoRefused, got {other:?} (code {})",
                other.code()
            ),
            Ok(_store) => panic!(
                "expected CNG_R10 IoRefused for a nonexistent decomposition-result graph, \
                 got a loaded store instead"
            ),
        }
    }

    /// Site 3 (`bench/workday_verify.rs`): `assemble_workday_manifest` on
    /// an `out_dir` that is a pre-existing plain file (not a directory)
    /// must refuse `CNG_R10 IoRefused` when it tries to `mkdir` the
    /// `results/` subdirectory under it — a path-collision failure,
    /// deterministic cross-platform, proven against a construction site in
    /// a third module the pipeline/workday sites above never touch.
    #[test]
    fn assemble_workday_manifest_out_dir_collision_refuses_cng_r10_io_refused() {
        let out_dir = file_collision_path("assemble-workday-manifest-out-dir");

        match assemble_workday_manifest(&out_dir) {
            Err(refusal @ CngRefusal::IoRefused(_)) => {
                assert_eq!(refusal.code(), "CNG_R10");
                let message = refusal.message();
                assert!(
                    message.contains("mkdir results"),
                    "IoRefused message should identify the failing mkdir: got {message:?}"
                );
            }
            Err(other) => panic!(
                "expected CNG_R10 IoRefused, got {other:?} (code {})",
                other.code()
            ),
            Ok(manifest) => panic!(
                "expected CNG_R10 IoRefused when out_dir is a plain file, \
                 got a {}-entry manifest instead",
                manifest.len()
            ),
        }
    }

    /// Site 4 (`bench/decomp/mod.rs`): `decompose`, driven through the real
    /// no-LLM decomposition pipeline over the checked-in potato example
    /// graph (grounding, Datalog edge derivation, bounded candidate search,
    /// interface/interference proofs, and composition all genuinely
    /// succeed), must refuse `CNG_R10 IoRefused` when its final
    /// `emit_result_graph` step tries to `mkdir` an `out_dir` that is a
    /// pre-existing plain file — proving the refusal fires from real
    /// production code deep in the decomposition pipeline, not a
    /// short-circuited stub.
    #[test]
    fn decompose_out_dir_collision_refuses_cng_r10_io_refused() {
        let store = potato_store();
        let queries = QuerySet::load(&decomp_queries_dir()).expect("load decomp queries");
        let (domain, problem) = strips_graph_to_surface(
            &store,
            &queries,
            &decomp_template("decomp-domain.template.pddl"),
            &decomp_template("decomp-problem.template.pddl"),
        )
        .expect("bridge potato graph to domain/problem surface");
        let out_dir = file_collision_path("decompose-out-dir");

        match decompose(
            &domain,
            &problem,
            &out_dir,
            "urn:cng:test:io-refused-cng-r10-expand:decompose-collision",
        ) {
            Err(refusal @ CngRefusal::IoRefused(_)) => {
                assert_eq!(refusal.code(), "CNG_R10");
                let message = refusal.message();
                let expected_path = out_dir.display().to_string();
                assert!(
                    message.contains(&expected_path),
                    "IoRefused message must name the failing path {expected_path}: got {message:?}"
                );
                assert!(
                    message.contains("mkdir"),
                    "IoRefused message should identify the failing operation: got {message:?}"
                );
            }
            Err(other) => panic!(
                "expected CNG_R10 IoRefused, got {other:?} (code {})",
                other.code()
            ),
            Ok(result) => panic!(
                "expected CNG_R10 IoRefused when out_dir is a plain file, \
                 got a decomposition result at {} instead",
                result.result_graph_path.display()
            ),
        }
    }
}

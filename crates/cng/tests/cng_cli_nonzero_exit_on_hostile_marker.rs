//! DEFINITION_OF_DONE.md §14 item 4, CLI/process-exit half: a hand-planted
//! bypass/hostile condition must flip a success marker false and fail the
//! run with a typed refusal + nonzero exit. Prior coverage
//! (`isolation_falsifier_hostile_graph_is_refuted_by_markers`,
//! `cng_multi_engine.rs`) proves only the in-process MARKER-QUERY half:
//! `evaluate_marker_map`/`evaluate_markers` returning
//! `Err(CngRefusal::MarkerFalse)` inside the test binary's own process.
//! That is NOT the same claim as "the `cng` CLI, run as an operator or CI
//! system would run it, exits nonzero" — a typed `Err` returned in-process
//! says nothing about whether `clap-noun-verb`'s dispatch layer actually
//! surfaces it as a failing OS process exit code. This file closes that
//! gap: it spawns the REAL compiled `cng` binary
//! (`std::process::Command::new(env!("CARGO_BIN_EXE_cng"))`, mirroring
//! `cng_multi_engine.rs`'s `run_cng` helper pattern) against a scenario
//! engineered to make a real marker evaluate false, and asserts on
//! `std::process::ExitStatus`, not on any in-process `Result`.
//!
//! ## Why `benchmark workday --ticks 0`, not the multi-engine hostile
//! ## fixture
//!
//! `tests/fixtures/multi-engine/forged-bypass-obs.ttl` (the hostile graph
//! `isolation_falsifier_hostile_graph_is_refuted_by_markers` refutes) is
//! evaluated against `DISTRIBUTED_MARKER_MAP` exclusively inside
//! `engine_collect_remote` (`crates/cng/src/bench/engine.rs`) — the
//! multi-engine COORDINATOR role. Grepping `main.rs`'s `#[verb(...)]`
//! table shows only `engine serve` and `engine resume` are wired to real
//! CLI verbs; `engine_collect_remote`/`engine_dispatch_remote` are public
//! `cng::bench` functions called ONLY by `cng_multi_engine.rs`'s own
//! in-process test coordinator (see that file's module doc: "coordinator
//! (in-process, the public API) + REAL engine OS processes"). There is
//! currently no real `cng` subcommand that loads an arbitrary observation
//! graph and evaluates `DISTRIBUTED_MARKER_MAP` over it, so the forged
//! fixture cannot be driven through a real process boundary without
//! adding a new coordinator verb — out of this ticket's surface (no
//! `main.rs`/`dispatch.rs`/`engine.rs`/`workday.rs` edits).
//!
//! `benchmark workday` (`cng::bench::workday`, wired at
//! `#[verb("workday", "benchmark")] fn benchmark_workday` in `main.rs`) is
//! the only real CLI verb that calls `evaluate_markers`
//! (`crates/cng/src/bench/workday.rs:1387`) over `MARKER_MAP`, and its own
//! doc comment states the law directly: "any false marker is a typed
//! refusal (nonzero process exit), never a warning." `--ticks 0` is a
//! legitimate CLI-only hostile input: `marker-autonomic-loop.rq` computes
//! `?value = brokenChains + extraOperators + IF(?roleObs = 0, 1, 0)`,
//! where `extraOperators = COUNT(DISTINCT ?op) - 1` over `workday_tick`
//! observations. With zero ticks there are zero `workday_tick`
//! observations, so `extraOperators = 0 - 1 = -1` and the marker's
//! `?value` comes out `-1` (nonzero => false), independently confirmed
//! empirically this session (see below) before this test was written.
//!
//! No inline Turtle/SPARQL: the failure is driven entirely by CLI flags
//! against the real templates/queries already on disk.

#![cfg(feature = "bench")]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use chicago_tdd_tools::prelude::*;

/// Fresh scratch root for one test. O(existing files) removal.
fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/cli-nonzero-exit-it")
        .join(test_name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Spawns the REAL compiled `cng` binary to completion and returns the raw
/// `Output` (stdout/stderr bytes + `ExitStatus`) — mirrors
/// `cng_multi_engine.rs`'s `run_cng` helper, except it hands back the full
/// `Output` (not just a `bool`) so callers can assert on the exact exit
/// code, not merely success/failure. O(child runtime).
fn run_cng(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cng"))
        .args(args)
        .output()
        .expect("spawn cng binary")
}

test!(
    hostile_zero_tick_workday_marker_false_exits_nonzero_from_real_process,
    {
        // Arrange: a workday run with zero ticks — no `workday_tick`
        // observations are ever emitted, so `marker-autonomic-loop.rq`'s
        // `extraOperators` term (`COUNT(DISTINCT ?op) - 1`) evaluates to
        // `-1` over the empty operator set, driving the marker's `?value`
        // to `-1` (nonzero, i.e. false) — a real, CLI-only-triggered
        // failure of the `AUTONOMIC_LOOP_CLOSED` marker.
        let out_dir = scratch_dir("hostile-zero-tick-workday");
        let out_arg = out_dir.to_str().expect("utf-8 scratch dir");

        // Act: spawn the REAL `cng` binary as its own OS process — this is
        // the CLI/process-exit half `DEFINITION_OF_DONE.md` §14 item 4
        // names as untested; no in-process function call is involved.
        let output = run_cng(&[
            "benchmark",
            "workday",
            "--out",
            out_arg,
            "--seed",
            "616",
            "--ticks",
            "0",
            "--refusal-per-mille",
            "0",
        ]);
        let stdout = String::from_utf8(output.stdout.clone()).expect("utf-8 stdout");
        let stderr = String::from_utf8(output.stderr.clone()).expect("utf-8 stderr");

        // Assert: the OS process itself exited nonzero — the way an
        // operator or CI system observes failure (`$?`/`echo $status`),
        // never an in-process `Result`.
        assert!(
            !output.status.success(),
            "hostile ticks=0 workday run must NOT succeed as an OS process: \
             stdout={stdout} stderr={stderr}"
        );
        assert_eq!(
            output.status.code(),
            Some(1),
            "clap-noun-verb's `main() -> Result<()>` returning `Err` must exit \
             with status 1 (std::process::Termination for Result<(), E: Debug>); \
             got {:?}. stdout={stdout} stderr={stderr}",
            output.status.code()
        );

        // Assert: the failure is the NAMED typed refusal (CNG_R20
        // MarkerFalse on AUTONOMIC_LOOP_CLOSED), not an unrelated crash,
        // argument-parsing error, or panic — proving the nonzero exit
        // traces back to the real marker-evaluation refusal path
        // (`evaluate_marker_map`, `crates/cng/src/bench/workday.rs`).
        assert!(
            stderr.contains("CNG_R20"),
            "stderr must name the CNG_R20 typed refusal: {stderr}"
        );
        assert!(
            stderr.contains("AUTONOMIC_LOOP_CLOSED"),
            "stderr must name the specific false marker: {stderr}"
        );
        assert!(
            stderr.contains("marker AUTONOMIC_LOOP_CLOSED, value -1"),
            "stderr must carry the marker's actual (nonzero) SPARQL value, \
             proving this is a real graph-derived refusal, not a hardcoded \
             message: {stderr}"
        );

        // Assert: the refusal fires BEFORE any report is printed
        // (`main.rs`'s own doc comment: "a false marker refused CNG_R20
        // with a nonzero exit before this line" — the `MARKER_*` /
        // `WORKDAY_*` println! loop never runs).
        assert!(
            stdout.is_empty(),
            "a refused run must print no report lines to stdout: {stdout}"
        );
        assert!(
            !stdout.contains("MARKER_"),
            "a refused run must never reach the marker-report println! loop: {stdout}"
        );
    }
);

test!(healthy_workday_marker_true_exits_zero_from_real_process, {
    // Arrange: a lawful workday run (ticks > 0) — the positive
    // contrast proving the nonzero exit above is caused by the
    // ticks=0 hostile condition specifically, not by some unrelated
    // defect that makes every real `cng benchmark workday` invocation
    // fail regardless of input.
    let out_dir = scratch_dir("healthy-workday");
    let out_arg = out_dir.to_str().expect("utf-8 scratch dir");

    // Act: same binary, same verb, only `--ticks` differs.
    let output = run_cng(&[
        "benchmark",
        "workday",
        "--out",
        out_arg,
        "--seed",
        "616",
        "--ticks",
        "2",
        "--refusal-per-mille",
        "0",
    ]);
    let stdout = String::from_utf8(output.stdout.clone()).expect("utf-8 stdout");
    let stderr = String::from_utf8(output.stderr.clone()).expect("utf-8 stderr");

    // Assert: a lawful run's real OS process exits 0.
    assert!(
        output.status.success(),
        "healthy ticks=2 workday run must succeed as an OS process: \
             stdout={stdout} stderr={stderr}"
    );
    assert_eq!(output.status.code(), Some(0), "expected exit code 0");
    assert!(
        stdout.contains("MARKER_AUTONOMIC_LOOP_CLOSED=true"),
        "healthy run must print the true marker report: {stdout}"
    );
    assert!(
        stdout.contains("MARKER_V26_7_10_PRODUCTION_READY=true"),
        "healthy run must print the true conjunction marker: {stdout}"
    );
});

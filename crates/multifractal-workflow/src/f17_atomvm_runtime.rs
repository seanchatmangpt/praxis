//! Family F17 -- "AtomVM Edge Runtime" (atlas ticket V12-017).
//!
//! Wire-phase-1 status (this pass): **MIXED, partially wired**. F17 requires the
//! *same* AIR semantic core (`delta_AIR: (S,E) -> (S',C)`) to run under two execution
//! surfaces -- normal OTP and a constrained-Erlang shell that actually executes on a
//! live AtomVM target -- with a pipeline: Shared air_core -> AtomVM Receive Loop ->
//! Portable Timer Adapter -> Portable Dispatch Adapter -> Receipt Fold -> Target
//! Harness -> Differential Comparator -> Live AtomVM execution evidence. The exit gate
//! requires three booleans: `ATOMVM_LIVE_RUNTIME_PROVEN`, `OTP_ATOMVM_RESULT_EQUIVALENCE`,
//! `OTP_ATOMVM_RECEIPT_EQUIVALENCE`.
//!
//! ## Why this module is a subprocess bridge, not a `pub use` re-export
//!
//! Unlike most other families in this crate, F17's real, already-tested substance is
//! **Erlang**, not Rust: `apps/air_core` (Shared air_core), `apps/arazzo_atomvm` +
//! `apps/atomvm_runner` (AtomVM Receive Loop), and
//! `apps/arazzo_runner/test/arazzo_runner_atomvm_differential_test.erl` (Differential
//! Comparator) -- there is no sibling Rust crate to add as a workspace dependency and
//! thinly wrap the way F06/F07/F10/F13/F14 do. Re-implementing that logic in Rust would
//! not be a "port" -- it would be exactly the defect class this family's own DfLSS calls
//! out: *AtomVM does not implement alternate workflow semantics (no second interpreter of
//! AIR)*. The only invariant-preserving way for this Rust crate to expose F17's real,
//! working evidence is therefore to shell out to the actual Erlang eunit suite and parse
//! its actual output -- never to re-derive the same verdict by re-implementing
//! `air_core:transition/2` a second time in Rust.
//!
//! [`run_otp_atomvm_differential_suite`] does exactly that: it invokes the real
//! `just erlang-test-atomvm-differential` recipe (added by this pass, scoped via
//! rebar3's own `-m` module filter to
//! `arazzo_runner_atomvm_differential_test` -- see `justfile`), which compiles and runs
//! the real, already-tested differential harness (PROJ-761/PROJ-762) comparing OTP and
//! the AtomVM-shaped wrapper across state digest, result digest, refusal class, and
//! command sequence for an identical corpus. The parsed pass/fail counts are real eunit
//! output, not asserted or hand-coded. Verified this session (not merely cited from the
//! prior survey): `timeout 90 just erlang-test-atomvm-differential` -> `4 tests, 0
//! failures` in 1.476s.
//!
//! ## The disclosed gap: `ATOMVM_LIVE_RUNTIME_PROVEN`
//!
//! Both "OTP" and "AtomVM" sides of the differential harness run as ordinary BEAM
//! processes in the same VM today -- exactly the "BEAM-only evidence mislabeled AtomVM"
//! defect class F17's own DfLSS flags. [`live_atomvm_target_evidence`] always returns
//! [`Refusal::LiveRuntimeNotProven`]; it is deliberately not a stub dressed up to look
//! complete. Re-verified this session: `which atomvm` exits nonzero (not found on this
//! machine), and no `.avm`/packbeam output, `atomvm_rebar3_plugin` dependency, or AtomVM
//! boot-module config exists anywhere in `rebar.config` or `apps/`. Real live-AtomVM
//! integration (cross-compiling this tree's `.beam` files into a packbeam `.avm`,
//! running/flashing it against an actual AtomVM interpreter or device, and building the
//! Portable Timer Adapter / Portable Dispatch Adapter / Target Harness pipeline stages
//! the family survey names -- none of which exist anywhere in this repo, grep-confirmed
//! by the prior survey) is real embedded-systems/cross-compilation engineering with no
//! registry/schema/CLI/SPARQL-template shape to it -- not GGEN-mechanical scaffolding --
//! and remains genuinely unbuilt, tracked under this family's ticket V12-017.
//!
//! `OTP_ATOMVM_RECEIPT_EQUIVALENCE` is approximated by the same digest-equivalence
//! evidence as `OTP_ATOMVM_RESULT_EQUIVALENCE` (the differential harness's real BLAKE3
//! state/result digest comparison), not a distinct measurement -- the family survey's own
//! "Receipt Fold" pipeline stage (C5) does not exist anywhere in this repo as an
//! independent artifact, so there is no separate receipt-fold evidence to consult yet.
//! This approximation is disclosed here, not silently assumed to be exact.
//!
//! Survey-cited paths for F17 (informed research from the v26.7.12 family survey handed
//! to this scaffolding session inline, not itself a checked-in repo doc):
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F17_atomvm-runtime.md
//! - /Users/sac/praxis/apps/atomvm_runner/src/atomvm_runner.erl
//! - /Users/sac/praxis/apps/arazzo_atomvm/src/arazzo_atomvm_workflow.erl
//! - /Users/sac/praxis/apps/atomvm_runner/test/atomvm_runner_test.erl
//! - /Users/sac/praxis/apps/arazzo_atomvm/test/arazzo_atomvm_SUITE.erl
//! - /Users/sac/praxis/apps/arazzo_runner/test/arazzo_runner_atomvm_differential_test.erl
//! - /Users/sac/praxis/rebar.config
//! - /Users/sac/praxis/justfile

use std::path::{Path, PathBuf};
use std::process::Command;

/// F17's typed refusal taxonomy. Every variant carries the concrete offending
/// context (never a bare generic message), per this crate's established convention
/// (see `f01_standing_algebra::Refusal`).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    /// Spawning or resolving the path to the real Erlang differential-suite harness
    /// failed before any eunit output could be observed (e.g. `just` not on `PATH`,
    /// or this crate's own location relative to the repo root could not be resolved).
    #[error("failed to invoke the OTP/AtomVM differential eunit harness: {reason}")]
    HarnessInvocationFailed { reason: String },
    /// The harness ran and produced output, but that output did not match either
    /// known `rebar3 eunit` summary shape (`"<N> tests, <M> failures"` / singular
    /// `"<N> test, <M> failures"`). Refused rather than guessed at.
    #[error("eunit output did not match the expected summary shape; tail: {raw_output_tail}")]
    UnparseableEunitOutput { raw_output_tail: String },
    /// The real differential harness reported one or more failures, or the
    /// subprocess exited non-zero -- a genuine OTP/AtomVM divergence (or an
    /// unrelated compile/harness fault), never silently treated as a pass.
    #[error(
        "OTP/AtomVM differential suite refused: {tests_run} test(s) run, {failures} failure(s); \
         reason: {reason}; tail: {raw_output_tail}"
    )]
    AtomVMDifferentialRefused {
        tests_run: u32,
        failures: u32,
        reason: String,
        raw_output_tail: String,
    },
    /// `ATOMVM_LIVE_RUNTIME_PROVEN` cannot be granted: no live AtomVM interpreter or
    /// target toolchain exists on this machine or anywhere in this repo. Always
    /// returned by [`live_atomvm_target_evidence`] -- a disclosed, tracked gap, not a
    /// fake success.
    #[error("live AtomVM runtime not proven: {reason}")]
    LiveRuntimeNotProven { reason: String },
}

/// Real, parsed evidence from one run of the OTP/AtomVM differential eunit suite
/// (`arazzo_runner_atomvm_differential_test.erl`, PROJ-761/PROJ-762).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DifferentialEvidence {
    pub tests_run: u32,
    pub failures: u32,
}

impl DifferentialEvidence {
    /// Whether every differential test passed (`tests_run > 0 && failures == 0`).
    /// A suite that reports zero tests run is deliberately not treated as a pass --
    /// that shape means the harness did not exercise anything, not that it succeeded.
    ///
    /// # Complexity
    /// O(1).
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.tests_run > 0 && self.failures == 0
    }
}

/// The three exit-evidence booleans F17's survey requires
/// (`ATOMVM_LIVE_RUNTIME_PROVEN`, `OTP_ATOMVM_RESULT_EQUIVALENCE`,
/// `OTP_ATOMVM_RECEIPT_EQUIVALENCE`). Mirrors `f01_standing_algebra`'s
/// `ClaimEvidence`/`ClaimCeiling` pattern: no field here is ever set `true` except by
/// [`exit_evidence_gate_from_differential_run`] deriving it from real evidence, or
/// (for `atomvm_live_runtime_proven`) by a future caller wiring in real live-target
/// evidence this pass does not have -- this struct never fabricates a `true` value on
/// its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExitEvidenceGate {
    pub atomvm_live_runtime_proven: bool,
    pub otp_atomvm_result_equivalence: bool,
    pub otp_atomvm_receipt_equivalence: bool,
}

/// Resolves the praxis repo root from this crate's own compile-time manifest
/// location (`crates/multifractal-workflow` is always exactly two path segments
/// below the repo root that owns `rebar.config`/`justfile`). Not external input --
/// `CARGO_MANIFEST_DIR` is a compiler-supplied constant -- but still returns a typed
/// refusal instead of `.unwrap()`/`.expect()` on the path arithmetic, per this repo's
/// no-panic discipline.
///
/// # Complexity
/// O(1).
fn repo_root() -> Result<PathBuf, Refusal> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent().and_then(Path::parent) {
        Some(root) => Ok(root.to_path_buf()),
        None => Err(Refusal::HarnessInvocationFailed {
            reason: format!(
                "could not resolve the repo root from CARGO_MANIFEST_DIR={}; expected \
                 crates/multifractal-workflow two path segments below the repo root",
                manifest_dir.display()
            ),
        }),
    }
}

/// Strips ANSI CSI color escapes (`rebar3` emits `ESC [ ... m` sequences on its
/// summary line even when stdout is piped, not just on a tty -- confirmed this
/// session by capturing `rebar3 eunit` output through a non-tty pipe) so the summary
/// line can be parsed as plain text.
///
/// # Complexity
/// O(n) in the length of `s`.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for c2 in chars.by_ref() {
                if c2 == 'm' {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// The last `max_chars` characters of `s` (for keeping [`Refusal`] messages bounded
/// when embedding raw subprocess output).
///
/// # Complexity
/// O(n) in the length of `s`.
fn tail(s: &str, max_chars: usize) -> String {
    let total = s.chars().count();
    if total <= max_chars {
        return s.to_string();
    }
    s.chars().skip(total - max_chars).collect()
}

/// Parses one segment of a `rebar3 eunit` summary line (e.g. `"4 tests"` or
/// `"0 failures"`) into its leading count, iff its trailing word (after stripping a
/// trailing `.`) is one of `expected_suffixes`. Returns `None` (not a panic, not a
/// swallowed error) on any other shape -- the caller turns a `None` here into a real
/// [`Refusal::UnparseableEunitOutput`], it is never treated as zero/absent.
///
/// # Complexity
/// O(n) in the length of `segment`.
fn parse_count(segment: &str, expected_suffixes: &[&str]) -> Option<u32> {
    let (number_str, word) = segment.split_once(' ')?;
    let word = word.trim_end_matches('.');
    if !expected_suffixes.contains(&word) {
        return None;
    }
    match number_str.parse::<u32>() {
        Ok(n) => Some(n),
        Err(_) => None,
    }
}

/// Parses a full `rebar3 eunit` summary line of the shape `"<N> tests, <M>
/// failures"` (or singular `"1 test, 0 failures"`) into [`DifferentialEvidence`].
///
/// # Complexity
/// O(n) in the length of `line`.
fn try_parse_summary_line(line: &str) -> Option<DifferentialEvidence> {
    let (left, right) = line.split_once(',')?;
    let tests_run = parse_count(left.trim(), &["test", "tests"])?;
    let failures = parse_count(right.trim(), &["failure", "failures"])?;
    Some(DifferentialEvidence {
        tests_run,
        failures,
    })
}

/// Scans `raw_output` (the `just erlang-test-atomvm-differential` subprocess's
/// **stdout only** -- see the caller, [`run_otp_atomvm_differential_suite`], for why
/// stderr must not be mixed in here: `just`'s own per-command echo lines land on
/// stderr and are not chronologically last relative to stdout despite appearing
/// after it in a naive concatenation) from the end for the eunit summary line and
/// parses it. Only the last non-blank line is considered -- if it does not parse as
/// a summary, this refuses rather than searching further back and risking a match
/// against unrelated log text (e.g. one of the differential harness's own `INFO
/// REPORT`/`CRASH REPORT` lines, which this corpus's audit-timeout case deliberately
/// produces as expected chaos evidence, not suite failure).
///
/// # Complexity
/// O(n) in the length of `raw_output`.
fn parse_eunit_summary(raw_output: &str) -> Result<DifferentialEvidence, Refusal> {
    let stripped = strip_ansi(raw_output);
    for line in stripped.lines().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        return match try_parse_summary_line(line) {
            Some(evidence) => Ok(evidence),
            None => Err(Refusal::UnparseableEunitOutput {
                raw_output_tail: tail(&stripped, 800),
            }),
        };
    }
    Err(Refusal::UnparseableEunitOutput {
        raw_output_tail: tail(&stripped, 800),
    })
}

/// Runs the real OTP/AtomVM differential eunit suite
/// (`arazzo_runner_atomvm_differential_test.erl`, PROJ-761/PROJ-762) via `just
/// erlang-test-atomvm-differential` and returns genuinely parsed pass/fail evidence.
/// This is a real subprocess invocation -- not a hardcoded or simulated result -- so
/// it requires `just` and `rebar3` on `PATH` and the praxis Erlang/OTP umbrella to be
/// present at the resolved repo root; see module doc for a real, this-session
/// confirmation of its output shape.
///
/// # Errors
/// - [`Refusal::HarnessInvocationFailed`] if the repo root cannot be resolved or the
///   subprocess cannot be spawned.
/// - [`Refusal::UnparseableEunitOutput`] if the subprocess produces output that does
///   not end in a recognizable eunit summary line.
/// - [`Refusal::AtomVMDifferentialRefused`] if the suite reports any failures, or the
///   subprocess exits non-zero even when the summary line itself claims zero
///   failures (never trusts the summary text alone over the real exit code).
///
/// # Complexity
/// Dominated by the subprocess's own rebar3 compile + eunit run (single-digit
/// seconds; 1.476s measured this session for this suite's 4 tests). Output parsing
/// itself is O(n) in the length of the captured output.
pub fn run_otp_atomvm_differential_suite() -> Result<DifferentialEvidence, Refusal> {
    let root = repo_root()?;
    let output = Command::new("just")
        .arg("erlang-test-atomvm-differential")
        .current_dir(&root)
        .output()
        .map_err(|e| Refusal::HarnessInvocationFailed {
            reason: format!(
                "failed to spawn `just erlang-test-atomvm-differential` in {}: {e}",
                root.display()
            ),
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    // BUG FOUND AND FIXED this session by actually running the `#[ignore]`d
    // integration test below (`run_otp_atomvm_differential_suite_reports_real_pass`),
    // not merely reading the code: an earlier version of this function parsed a
    // naive `stdout + stderr` concatenation. Captured separately
    // (`just erlang-test-atomvm-differential 1>out 2>err`), rebar3's real eunit
    // summary line ("4 tests, 0 failures") lands on stdout, while `just` writes its
    // own per-command echo lines ("timeout 60s rebar3 eunit -m ...") to stderr.
    // stdout and stderr are independently buffered streams, so appending all of
    // stderr after all of stdout put an unrelated `just`-echo line after the real
    // summary and broke the scan-from-the-end parse (it matched `UnparseableEunitOutput`
    // even though the suite had genuinely passed). Parse stdout only; stderr is kept
    // for diagnostics in the refusal variants below, never for the summary parse.
    let evidence = parse_eunit_summary(&stdout)?;
    let combined_for_diagnostics = format!("{stdout}{stderr}");
    if evidence.failures > 0 {
        return Err(Refusal::AtomVMDifferentialRefused {
            tests_run: evidence.tests_run,
            failures: evidence.failures,
            reason: "eunit summary reported one or more failures".to_string(),
            raw_output_tail: tail(&combined_for_diagnostics, 1200),
        });
    }
    if !output.status.success() {
        return Err(Refusal::AtomVMDifferentialRefused {
            tests_run: evidence.tests_run,
            failures: evidence.failures,
            reason: format!(
                "eunit summary reported zero failures but the subprocess exited with {:?}; \
                 not trusting the summary text alone",
                output.status.code()
            ),
            raw_output_tail: tail(&combined_for_diagnostics, 1200),
        });
    }
    Ok(evidence)
}

/// Derives the [`ExitEvidenceGate`] from one real [`DifferentialEvidence`] run.
/// `otp_atomvm_result_equivalence` and `otp_atomvm_receipt_equivalence` are both set
/// `true` only when `evidence.all_passed()` -- see module doc for why receipt
/// equivalence is approximated by the same digest-equivalence evidence rather than a
/// distinct measurement. `atomvm_live_runtime_proven` is hardwired `false`: this
/// function has no live-target evidence to consult and does not fabricate one (see
/// [`live_atomvm_target_evidence`]).
///
/// # Complexity
/// O(1).
#[must_use]
pub fn exit_evidence_gate_from_differential_run(
    evidence: &DifferentialEvidence,
) -> ExitEvidenceGate {
    let otp_atomvm_equivalent = evidence.all_passed();
    ExitEvidenceGate {
        atomvm_live_runtime_proven: false,
        otp_atomvm_result_equivalence: otp_atomvm_equivalent,
        otp_atomvm_receipt_equivalence: otp_atomvm_equivalent,
    }
}

/// `ATOMVM_LIVE_RUNTIME_PROVEN` evidence gate (HAND_WRITE_REQUIRED, disclosed gap --
/// see module doc). Always refuses: there is no real live-AtomVM execution evidence
/// anywhere in this repo to report. This is deliberately not a stub that returns a
/// hollow `Ok(())` to look complete -- unimplemented fails loud, per this repo's
/// no-overclaiming discipline.
///
/// # Errors
/// Always returns [`Refusal::LiveRuntimeNotProven`].
///
/// # Complexity
/// O(1).
pub fn live_atomvm_target_evidence() -> Result<(), Refusal> {
    Err(Refusal::LiveRuntimeNotProven {
        reason: "no real live-AtomVM execution evidence exists anywhere in this repo to report"
            .to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_count_accepts_plural() {
        assert_eq!(parse_count("4 tests", &["test", "tests"]), Some(4));
        assert_eq!(parse_count("0 failures", &["failure", "failures"]), Some(0));
    }

    #[test]
    fn parse_count_accepts_singular() {
        assert_eq!(parse_count("1 test", &["test", "tests"]), Some(1));
        assert_eq!(parse_count("1 failure", &["failure", "failures"]), Some(1));
    }

    #[test]
    fn parse_count_rejects_wrong_word() {
        assert_eq!(parse_count("4 tests", &["failure", "failures"]), None);
    }

    #[test]
    fn parse_count_rejects_non_numeric() {
        assert_eq!(parse_count("many tests", &["test", "tests"]), None);
    }

    #[test]
    fn try_parse_summary_line_real_shape_all_passed() {
        let evidence = try_parse_summary_line("4 tests, 0 failures").expect("must parse");
        assert_eq!(evidence.tests_run, 4);
        assert_eq!(evidence.failures, 0);
        assert!(evidence.all_passed());
    }

    #[test]
    fn try_parse_summary_line_real_shape_with_failures() {
        let evidence = try_parse_summary_line("4 tests, 2 failures").expect("must parse");
        assert_eq!(evidence.failures, 2);
        assert!(!evidence.all_passed());
    }

    #[test]
    fn try_parse_summary_line_rejects_unrelated_text() {
        assert!(try_parse_summary_line("Finished in 1.476 seconds").is_none());
    }

    #[test]
    fn parse_eunit_summary_strips_ansi_and_parses_real_captured_shape() {
        // Byte-for-byte the tail this session's real `rebar3 eunit -m
        // arazzo_runner_atomvm_differential_test` run produced (captured via a
        // non-tty pipe and inspected with `od -c`), confirming ANSI stripping
        // handles the exact bytes rebar3 emits, not an idealized approximation.
        let raw = "Finished in 1.476 seconds\n\u{1b}[0;32m4 tests, 0 failures\n\u{1b}[0m";
        let evidence = parse_eunit_summary(raw).expect("must parse real captured shape");
        assert_eq!(evidence.tests_run, 4);
        assert_eq!(evidence.failures, 0);
    }

    #[test]
    fn parse_eunit_summary_refuses_unparseable_output() {
        let result = parse_eunit_summary("some unrelated crash trace\nwith no summary line");
        assert!(matches!(
            result,
            Err(Refusal::UnparseableEunitOutput { .. })
        ));
    }

    #[test]
    fn parse_eunit_summary_refuses_empty_output() {
        let result = parse_eunit_summary("");
        assert!(matches!(
            result,
            Err(Refusal::UnparseableEunitOutput { .. })
        ));
    }

    #[test]
    fn exit_evidence_gate_all_true_only_on_full_pass() {
        let passing = DifferentialEvidence {
            tests_run: 4,
            failures: 0,
        };
        let gate = exit_evidence_gate_from_differential_run(&passing);
        assert!(!gate.atomvm_live_runtime_proven);
        assert!(gate.otp_atomvm_result_equivalence);
        assert!(gate.otp_atomvm_receipt_equivalence);
    }

    #[test]
    fn exit_evidence_gate_false_on_any_failure() {
        let failing = DifferentialEvidence {
            tests_run: 4,
            failures: 1,
        };
        let gate = exit_evidence_gate_from_differential_run(&failing);
        assert!(!gate.atomvm_live_runtime_proven);
        assert!(!gate.otp_atomvm_result_equivalence);
        assert!(!gate.otp_atomvm_receipt_equivalence);
    }

    #[test]
    fn exit_evidence_gate_false_on_zero_tests_run() {
        // A suite that ran nothing is not a pass, even with zero failures.
        let vacuous = DifferentialEvidence {
            tests_run: 0,
            failures: 0,
        };
        let gate = exit_evidence_gate_from_differential_run(&vacuous);
        assert!(!gate.otp_atomvm_result_equivalence);
    }

    #[test]
    
    fn live_atomvm_target_evidence_always_refuses() {
        assert!(matches!(
            live_atomvm_target_evidence(),
            Err(Refusal::LiveRuntimeNotProven { .. })
        ));
    }

    #[test]
    fn repo_root_resolves_to_a_directory_containing_justfile() {
        let root = repo_root().expect("repo root must resolve from CARGO_MANIFEST_DIR");
        assert!(
            root.join("justfile").is_file(),
            "resolved repo root {} does not contain justfile",
            root.display()
        );
    }

    /// Real, end-to-end: spawns `just erlang-test-atomvm-differential`, which
    /// compiles and runs the actual Erlang differential harness. Marked `#[ignore]`
    /// because it requires `rebar3`/`just` on `PATH` and the Erlang/OTP umbrella to
    /// be present -- not a dependency `cargo test -p multifractal-workflow` should
    /// impose on every environment by default. Run explicitly with `cargo test -p
    /// multifractal-workflow -- --ignored`. Verified manually this session (see
    /// module doc): real output was `4 tests, 0 failures` in 1.476s.
    #[test]
    #[ignore = "requires rebar3/just on PATH and the Erlang/OTP umbrella; run with --ignored"]
    fn run_otp_atomvm_differential_suite_reports_real_pass() {
        let evidence =
            run_otp_atomvm_differential_suite().expect("real differential suite must pass");
        assert_eq!(evidence.tests_run, 4);
        assert_eq!(evidence.failures, 0);
        assert!(evidence.all_passed());
    }
}

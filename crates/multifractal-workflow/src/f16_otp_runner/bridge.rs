//! F16's real Rust<->Erlang bridge into the atlas L5 gen_statem lifecycle
//! (`arazzo_runner_dispatch_statem.erl`) -- the parent module's own doc
//! comment (section "HAND_WRITE_REQUIRED") disclosed this gen_statem as
//! real but not wired into `arazzo_runner_workflow.erl`'s production
//! dispatch path, and `docs/jira/v26.7.12/CROWN_STATUS.md` independently
//! re-confirmed, three separate times, that rewiring `apply_transition/4`
//! itself carries real regression risk to `arazzo_runner_workflow_test.erl`'s
//! synchronous-ordering assertions and was deliberately not attempted.
//!
//! # What this is
//!
//! A minimal OS-process bridge, structurally identical to
//! [`crate::f15_air_transition_core::bridge::call_air_core_bridge`]: this
//! Rust code spawns
//! `apps/arazzo_runner/scripts/dispatch_statem_bridge.escript` as a child
//! process, writes one line of JSON to its stdin, and reads one line of
//! JSON back from its stdout. The escript calls the REAL, `rebar3`-compiled
//! `application:ensure_all_started(arazzo_runner)`,
//! `arazzo_runner_sup:start_workflow/1`, and
//! `arazzo_runner_dispatch_statem:start_link/4` + `mark_ready/1` +
//! `dispatch/1` -- driving the real, unmodified, already-supervised
//! `arazzo_runner_broker:dispatch/4` through the real 8-state atlas
//! lifecycle -- and reports back exactly what those functions computed.
//!
//! # Why a *second*, independent entrypoint rather than rewiring `apply_transition/4`
//!
//! `arazzo_runner_workflow.erl:apply_transition/4` calls
//! `arazzo_runner_broker:dispatch/4` directly and *synchronously*; several
//! `arazzo_runner_workflow_test.erl` assertions rely on dispatch completing
//! before the next reaction is processed. `arazzo_runner_dispatch_statem`'s
//! own `dispatch/1` is deliberately *asynchronous* (it replies `ok` before
//! the spawned worker's broker round trip completes -- its own proven core
//! feature, see `arazzo_runner_dispatch_statem_test.erl`'s
//! `test_lawful_path_advances_live_workflow/0`). Rewiring the existing call
//! site would flip that ordering guarantee and risk breaking currently-
//! passing tests -- a concrete regression risk this session's own
//! `docs/jira/v26.7.12/CROWN_STATUS.md` re-confirmed independently three
//! times before this bridge was built. This bridge instead reuses
//! `arazzo_runner_sup:start_workflow/1` -- a second, real, already-existing
//! production entrypoint into the same OTP app -- and drives the real
//! dispatch-statem lifecycle directly, never touching
//! `arazzo_runner_workflow.erl` or `apply_transition/4` in any way. This
//! does **not** flip [`super::check_gen_statem_lifecycle_wired`] to `Ok`:
//! that check's own documented meaning is specifically about
//! `apply_transition/4`'s *internal* Erlang wiring, which this bridge does
//! not change and correctly still refuses.
//!
//! # Disclosed scope limits (real, not hidden)
//!
//! - **Stateless per call**: every [`call_dispatch_statem_bridge`]
//!   invocation spawns a fresh escript process (fresh BEAM VM), a fresh
//!   `/tmp` state directory, and drives exactly one step through exactly
//!   one dispatch-statem worker. Nothing persists between separate calls.
//! - **Narrowed identity surface**: `parent_workflow_id`/
//!   `arazzo_workflow_id`/`source_powl_region_id`/`dispatch_id` (4 of the
//!   10 `#workflow_identity{}` fields) are synthesized by the escript from
//!   `workflow_id`/`correlation_id` rather than accepted as independent
//!   request fields -- this bridge exercises one step's real lawful
//!   dispatch through the real 8-state lifecycle, not the full 10-field
//!   identity surface a richer caller might need.
//! - **Single output-bind shape**: the dispatched step always carries one
//!   `{bind, BindName, {literal, BindValue}}` output rule (the same literal-
//!   bind shape `arazzo_runner_dispatch_statem_test.erl`'s own lawful-path
//!   test uses) -- no other output-rule shape is reachable through this
//!   bridge.
//! - **Requires an out-of-band build step**: `apps/arazzo_runner` (and its
//!   real dependency `apps/air_core`) must already be compiled (`just
//!   erlang-compile`) and `escript` must be on `PATH`. [`call_dispatch_statem_bridge`]
//!   fails loud with a typed [`DispatchStatemBridgeRefused`] variant in
//!   either case -- it never silently falls back to a Rust reimplementation.
//!
//! Verified for real this session (manual `escript` invocations against the
//! real, `rebar3`-compiled `apps/arazzo_runner`, recorded in this family's
//! session notes, and via the `#[ignore]`d integration tests at the bottom
//! of this file, run explicitly with `--ignored`): a lawful single-step
//! dispatch genuinely traverses all 8 atlas states
//! (`manufactured,ready,dispatched,awaiting_result,awaiting_admission,running,completed`)
//! and returns a real, non-empty dispatch token computed by
//! `arazzo_runner_broker:dispatch/4`; a request with an empty
//! `correlation_id` genuinely refuses with the real Erlang atom
//! `CORRELATION_MISSING` after traversing
//! `manufactured,ready,dispatched,awaiting_result,awaiting_admission,refused`
//! -- both are the real Erlang state machine's own computed outcomes, not a
//! Rust reimplementation of its transition logic.

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// One real dispatch-statem lifecycle request. Every field is required by the real
/// `arazzo_runner_identity:from_map/1` (via the escript's synthesized 10-field identity map) or the
/// real dispatched step -- see the module doc's "narrowed identity surface" disclosure.
#[derive(Debug, Clone, Serialize)]
pub struct DispatchStatemRequest {
    pub workflow_id: String,
    pub correlation_id: String,
    pub source_digest: String,
    pub projection_digest: String,
    pub receipt_head: String,
    pub replay_id: String,
    pub step_id: String,
    pub bind_name: String,
    pub bind_value: bool,
}

/// The real terminal outcome of one dispatch-statem lifecycle run, computed entirely by the real
/// Erlang gen_statem and its real, unmodified `arazzo_runner_broker:dispatch/4` -- not recomputed
/// or reinterpreted by this Rust module.
///
/// `step_id` is read back from the real running `arazzo_runner_dispatch_statem` process's own
/// `#d.step_id` record field (via `arazzo_runner_dispatch_statem:get_step_id/1`, called by
/// `dispatch_statem_bridge.escript` after the lifecycle reaches a terminal state) -- an
/// independent, live query of the actual process's own internal state, not this Rust module's
/// own copy of the [`DispatchStatemRequest::step_id`] it sent. This is what makes the F15->F16
/// step-id correspondence checkable against the real runtime (see
/// `crown_external_test.rs::f16_dispatch_step_id_corresponds_verbatim_to_the_f15_command_that_named_it`)
/// rather than merely trusting the caller's own bookkeeping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchStatemOutcome {
    /// The real 8-state lawful path completed; `dispatch_token` is the real, non-empty token
    /// `arazzo_runner_broker:dispatch/4` computed.
    Completed {
        step_id: String,
        transition_log: Vec<String>,
        dispatch_token: String,
    },
    /// A real Erlang refusal atom (e.g. `CORRELATION_MISSING`) terminated the lifecycle at
    /// `REFUSED` -- parallels [`super::OTPWorkflowRefused::erlang_atom`]'s vocabulary, though this
    /// module does not itself construct that enum (its caller may, from `refusal_atom`).
    Refused {
        step_id: String,
        transition_log: Vec<String>,
        refusal_atom: String,
    },
}

#[derive(Debug, Deserialize)]
struct RawDispatchStatemResponse {
    ok: bool,
    #[serde(default)]
    outcome: Option<String>,
    #[serde(default)]
    step_id: Option<String>,
    #[serde(default)]
    transition_log: Vec<String>,
    #[serde(default)]
    dispatch_token: Option<String>,
    #[serde(default)]
    refusal_atom: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

/// Typed refusal taxonomy for the bridge itself -- process-boundary and protocol failures,
/// deliberately kept separate from [`super::OTPWorkflowRefused`] (which mirrors real Erlang
/// broker-refusal atoms specifically). Every variant fails loud with the concrete offender named --
/// no variant swallows a failure into a default/empty success.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DispatchStatemBridgeRefused {
    /// `apps/arazzo_runner/scripts/dispatch_statem_bridge.escript` does not exist at the resolved
    /// path -- most commonly means this Rust crate's `CARGO_MANIFEST_DIR`-relative repo-root
    /// resolution is wrong, not that the bridge itself is broken.
    #[error("arazzo_runner dispatch-statem bridge script not found at {path}")]
    ScriptMissing { path: String },
    /// The repo root could not be resolved from `CARGO_MANIFEST_DIR`.
    #[error("could not resolve repo root from CARGO_MANIFEST_DIR={manifest_dir}")]
    RepoRootUnresolved { manifest_dir: String },
    /// The request could not be serialized to JSON (defensive -- no known input shape in this
    /// module triggers this; kept typed rather than `.unwrap()`-ed away).
    #[error("failed to serialize dispatch-statem bridge request: {reason}")]
    RequestSerializeFailed { reason: String },
    /// `escript` could not be spawned at all -- most commonly means `escript` is not on `PATH` in
    /// this environment, or `apps/arazzo_runner`/`apps/air_core` has not been compiled yet (`just
    /// erlang-compile`).
    #[error("failed to spawn `escript {script}`: {reason} (is escript on PATH?)")]
    SpawnFailed { script: String, reason: String },
    /// The child process's stdin could not be written to (e.g. the escript exited before consuming
    /// its request).
    #[error("failed to write request to the dispatch-statem bridge's stdin: {reason}")]
    StdinWriteFailed { reason: String },
    /// The escript exited with a nonzero status -- an escript-level crash, distinct from an
    /// `{"ok":false,...}` application-level refusal (which exits 0 -- see
    /// [`ErlangSideError`](Self::ErlangSideError)).
    #[error("dispatch-statem bridge exited with status {status:?}; stderr: {stderr}")]
    NonZeroExit { status: Option<i32>, stderr: String },
    /// The escript's stdout was not the expected `{"ok":...}` JSON shape.
    #[error("dispatch-statem bridge produced an unparseable response: {reason}; raw={raw}")]
    MalformedResponse { reason: String, raw: String },
    /// The escript itself reported `{"ok":false,"error":"..."}` -- a real failure the Erlang side
    /// observed (e.g. `start_workflow` refused, or an unexpected terminal/outcome pair), not a
    /// Rust-side parsing problem.
    #[error("dispatch-statem bridge reported an error from the real Erlang call: {message}")]
    ErlangSideError { message: String },
    /// The escript reported `ok=true` but with an `outcome`/field combination this module does not
    /// recognize (defensive: the escript's own `to_response/3` only ever emits `completed` with a
    /// `dispatch_token` or `refused` with a `refusal_atom` -- this variant guards against future
    /// drift between the two sides rather than silently defaulting to one outcome).
    #[error("dispatch-statem bridge reported ok=true with an unrecognized outcome shape: {raw}")]
    UnrecognizedOutcomeShape { raw: String },
    /// The escript did not exit within the bounded wait window
    /// (`BRIDGE_WAIT_TIMEOUT_SECS`) -- most commonly a genuine Erlang-side
    /// hang (a real deadlock, or a broker call that never returns), not a
    /// Rust-side bug. The child process has already been killed
    /// (`Child::kill()`) by the time this variant is returned -- no process
    /// is left running past this error. This closes a real production-
    /// reliability gap: before this variant existed, a hung escript hung
    /// the Rust caller forever with no recovery path.
    #[error("dispatch-statem bridge did not exit within {seconds}s and was killed")]
    Timeout { seconds: u64 },
}

/// Resolves the repo root from `CARGO_MANIFEST_DIR` (`<repo>/crates/multifractal-workflow`),
/// matching [`crate::f15_air_transition_core::bridge`]'s identical `repo_root()` convention.
fn repo_root() -> Result<PathBuf, DispatchStatemBridgeRefused> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| DispatchStatemBridgeRefused::RepoRootUnresolved {
            manifest_dir: manifest_dir.display().to_string(),
        })
}

/// How long [`call_dispatch_statem_bridge`] waits for the spawned escript to exit before treating
/// it as hung and killing it (see [`wait_with_timeout`]). 30s is generous for a local subprocess
/// call -- no network round trip is involved -- while still bounding a genuine Erlang-side
/// deadlock, or a broker call that never returns, to a finite, recoverable wait instead of hanging
/// the Rust caller forever. This is a separate, generous ceiling above the escript's own internal
/// `wait_for_terminal/2` 2s poll ceiling (see the module doc comment) -- that bound only covers
/// the gen_statem's own lifecycle polling, not the BEAM VM boot/supervision-tree startup this
/// timeout also has to cover.
const BRIDGE_WAIT_TIMEOUT_SECS: u64 = 30;

/// Internal outcome of [`wait_with_timeout`], mapped by [`call_dispatch_statem_bridge`] to a
/// variant of [`DispatchStatemBridgeRefused`]. Kept private to this module --
/// `f15_air_transition_core::bridge` has its own, independent copy of this same watchdog, matching
/// this file's existing convention of not sharing infra across families (see [`repo_root`]'s doc
/// comment, which states the same rationale for its own duplication).
#[derive(Debug)]
enum BoundedWaitError {
    /// The child did not exit within the bounded wait window; by the time this variant is
    /// returned, the child has already been killed (`Child::kill()`) and reaped (`Child::wait()`).
    TimedOut,
    /// `Child::try_wait()` itself returned an OS-level error, distinct from a timeout. The child is
    /// killed defensively before this variant is returned.
    Io(std::io::Error),
    /// The stdout or stderr reader thread panicked (defensive -- neither reader closure in
    /// [`wait_with_timeout`] contains an `.unwrap()`/`panic!()` path, so this should be
    /// unreachable in practice; kept typed rather than silently discarding the partial output).
    ReaderPanicked,
}

/// Bounded, deadlock-safe replacement for `Child::wait_with_output()`.
///
/// `Child::wait_with_output()` blocks the calling thread until the child exits, with no timeout --
/// a real Erlang-side deadlock, or a broker call that never returns, previously hung the Rust
/// caller forever with no recovery path. This function instead polls `Child::try_wait()` from the
/// calling thread (never blocking longer than `POLL_INTERVAL` at a stretch) and, if `timeout`
/// elapses before the child exits, kills the child (`Child::kill()`), reaps it (`Child::wait()`),
/// and returns `Err(BoundedWaitError::TimedOut)` instead of hanging.
///
/// `stdout`/`stderr` are drained concurrently on two dedicated reader threads -- the same
/// technique the standard library's own `wait_with_output` uses internally -- so a child that
/// writes more output than the OS pipe buffer holds cannot deadlock against an undrained pipe
/// while this function polls for exit. `stdin` is dropped up front (matching
/// `Child::wait_with_output()`'s own first step), so a child reading its stdin to EOF is not left
/// waiting on a stdin handle this function still holds open.
///
/// # Complexity
/// O(timeout / POLL_INTERVAL) polling wake-ups on the calling thread in the worst case (a child
/// that never exits); O(1) once the child exits promptly, which is the expected case for every
/// real production call. The two reader threads are O(n) in the number of bytes the child writes
/// to stdout/stderr -- bounded in practice by this bridge's single-line JSON payloads.
fn wait_with_timeout(mut child: Child, timeout: Duration) -> Result<Output, BoundedWaitError> {
    const POLL_INTERVAL: Duration = Duration::from_millis(20);

    // Matches `Child::wait_with_output()`'s own first step: drop stdin so a child reading to EOF
    // is not left blocked on a handle we still hold.
    drop(child.stdin.take());

    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();
    let stdout_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(pipe) = stdout_pipe.as_mut() {
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(pipe) = stderr_pipe.as_mut() {
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    });

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stdout_reader.join();
                    let _ = stderr_reader.join();
                    return Err(BoundedWaitError::TimedOut);
                }
                std::thread::sleep(POLL_INTERVAL);
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(BoundedWaitError::Io(e));
            }
        }
    };

    let stdout = stdout_reader
        .join()
        .map_err(|_| BoundedWaitError::ReaderPanicked)?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| BoundedWaitError::ReaderPanicked)?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

/// Calls the real `application:ensure_all_started(arazzo_runner)` + `arazzo_runner_sup:
/// start_workflow/1` + `arazzo_runner_dispatch_statem:start_link/4`/`mark_ready/1`/`dispatch/1`
/// chain (via `apps/arazzo_runner/scripts/dispatch_statem_bridge.escript`) for one real step
/// dispatch, returning its real terminal outcome.
///
/// # Complexity
/// O(1) Rust-side (one process spawn + one JSON encode/decode); the Erlang-side cost is bounded by
/// the escript's own `wait_for_terminal/2` poll loop (at most 400 iterations of a 5ms sleep, i.e.
/// a 2s ceiling, well above the real lawful/refusal path's actual completion time).
pub fn call_dispatch_statem_bridge(
    request: &DispatchStatemRequest,
) -> Result<DispatchStatemOutcome, DispatchStatemBridgeRefused> {
    let root = repo_root()?;
    let script_path = root
        .join("apps")
        .join("arazzo_runner")
        .join("scripts")
        .join("dispatch_statem_bridge.escript");
    if !script_path.is_file() {
        return Err(DispatchStatemBridgeRefused::ScriptMissing {
            path: script_path.display().to_string(),
        });
    }
    let erl_libs = root.join("_build").join("default").join("lib");

    let request_json = serde_json::to_string(request).map_err(|e| {
        DispatchStatemBridgeRefused::RequestSerializeFailed {
            reason: e.to_string(),
        }
    })?;

    let mut child = Command::new("escript")
        .arg(&script_path)
        .env("ERL_LIBS", &erl_libs)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| DispatchStatemBridgeRefused::SpawnFailed {
            script: script_path.display().to_string(),
            reason: e.to_string(),
        })?;

    {
        let stdin =
            child
                .stdin
                .as_mut()
                .ok_or_else(|| DispatchStatemBridgeRefused::StdinWriteFailed {
                    reason: "child process has no stdin handle".to_string(),
                })?;
        writeln!(stdin, "{request_json}").map_err(|e| {
            DispatchStatemBridgeRefused::StdinWriteFailed {
                reason: e.to_string(),
            }
        })?;
    }

    let output = wait_with_timeout(child, Duration::from_secs(BRIDGE_WAIT_TIMEOUT_SECS)).map_err(
        |e| match e {
            BoundedWaitError::TimedOut => DispatchStatemBridgeRefused::Timeout {
                seconds: BRIDGE_WAIT_TIMEOUT_SECS,
            },
            BoundedWaitError::Io(io_err) => DispatchStatemBridgeRefused::SpawnFailed {
                script: script_path.display().to_string(),
                reason: format!("failed waiting for child output: {io_err}"),
            },
            BoundedWaitError::ReaderPanicked => DispatchStatemBridgeRefused::SpawnFailed {
                script: script_path.display().to_string(),
                reason: "stdout/stderr reader thread panicked while waiting for child output"
                    .to_string(),
            },
        },
    )?;

    if !output.status.success() {
        return Err(DispatchStatemBridgeRefused::NonZeroExit {
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    let raw = String::from_utf8_lossy(&output.stdout).into_owned();
    parse_dispatch_statem_stdout(&raw)
}

/// Parses one line of the escript's stdout (the exact bytes
/// [`call_dispatch_statem_bridge`] would have read from `output.stdout`) into
/// a [`DispatchStatemOutcome`], without spawning any process.
///
/// Factored out of [`call_dispatch_statem_bridge`] so this JSON-decode/
/// response-contract boundary is independently testable against arbitrary
/// (malformed, truncated, wrong-typed) input -- see the `proptest` suite in
/// this module's tests, which feeds this function directly rather than
/// spawning a real escript for every case.
///
/// # Complexity
/// O(n) in the length of `raw` (one `serde_json` parse pass).
fn parse_dispatch_statem_stdout(
    raw: &str,
) -> Result<DispatchStatemOutcome, DispatchStatemBridgeRefused> {
    let parsed: RawDispatchStatemResponse = serde_json::from_str(raw.trim()).map_err(|e| {
        DispatchStatemBridgeRefused::MalformedResponse {
            reason: e.to_string(),
            raw: raw.to_string(),
        }
    })?;

    if !parsed.ok {
        return Err(DispatchStatemBridgeRefused::ErlangSideError {
            message: parsed
                .error
                .unwrap_or_else(|| "ok=false with no error field".to_string()),
        });
    }

    match (
        parsed.outcome.as_deref(),
        parsed.step_id,
        parsed.dispatch_token,
        parsed.refusal_atom,
    ) {
        (Some("completed"), Some(step_id), Some(token), None) => {
            Ok(DispatchStatemOutcome::Completed {
                step_id,
                transition_log: parsed.transition_log,
                dispatch_token: token,
            })
        }
        (Some("refused"), Some(step_id), None, Some(atom)) => Ok(DispatchStatemOutcome::Refused {
            step_id,
            transition_log: parsed.transition_log,
            refusal_atom: atom,
        }),
        _ => Err(DispatchStatemBridgeRefused::UnrecognizedOutcomeShape {
            raw: raw.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> DispatchStatemRequest {
        DispatchStatemRequest {
            workflow_id: "wf-f16-bridge-test".to_string(),
            correlation_id: "corr-f16-bridge-test".to_string(),
            source_digest: "src-digest".to_string(),
            projection_digest: "proj-digest".to_string(),
            receipt_head: "receipt-head".to_string(),
            replay_id: "replay-id".to_string(),
            step_id: "step_x".to_string(),
            bind_name: "step_x_done".to_string(),
            bind_value: true,
        }
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

    /// Proves the watchdog's timeout path is real and reachable, not just declared: a real
    /// `sleep 5` OS subprocess (a deliberately slow test double for a genuinely hung escript -- a
    /// real Erlang deadlock, or a broker call that never returns -- not a mock) is spawned with the
    /// exact same `Stdio::piped()` shape [`call_dispatch_statem_bridge`] uses, then waited on with
    /// a 200ms timeout (far shorter than the 5s sleep). This test does not run for 5 real seconds:
    /// it asserts [`wait_with_timeout`] returns `TimedOut` well before that, and then checks the
    /// real OS process table (`ps -p <pid>`) to confirm the child was actually killed, not merely
    /// abandoned still running -- exactly the production-reliability property this fix exists for.
    #[test]
    fn wait_with_timeout_kills_a_hung_child_and_returns_timed_out() {
        // Arrange
        let child = Command::new("sleep")
            .arg("5")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn real `sleep 5` subprocess (test double for a hung escript)");
        let pid = child.id();
        let start = Instant::now();

        // Act
        let result = wait_with_timeout(child, Duration::from_millis(200));

        // Assert: the wait genuinely timed out, not merely raced a fast exit.
        let elapsed = start.elapsed();
        assert!(
            matches!(result, Err(BoundedWaitError::TimedOut)),
            "expected Err(TimedOut), got {result:?}"
        );
        assert!(
            elapsed < Duration::from_secs(3),
            "wait_with_timeout took {elapsed:?}; expected it to return well before the real \
             5s sleep would have finished on its own, proving the 200ms timeout (not the \
             child's own exit) ended the wait"
        );

        // Assert: the child was actually killed, not left running orphaned.
        let still_in_process_table = Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .output()
            .expect("run `ps -p <pid>` against the real OS process table");
        assert!(
            !still_in_process_table.status.success(),
            "pid {pid} is still present in the process table after wait_with_timeout timed \
             out -- the child was not actually killed"
        );
    }

    #[test]
    fn bridge_script_path_exists_in_repo() {
        let root = repo_root().expect("repo root must resolve");
        let script = root
            .join("apps")
            .join("arazzo_runner")
            .join("scripts")
            .join("dispatch_statem_bridge.escript");
        assert!(
            script.is_file(),
            "expected escript at {} (this is a static repo-layout check, no process spawn)",
            script.display()
        );
    }

    #[test]
    fn request_serializes_with_the_field_names_the_escript_expects() {
        let json = serde_json::to_value(sample_request()).expect("serialize");
        for key in [
            "workflow_id",
            "correlation_id",
            "source_digest",
            "projection_digest",
            "receipt_head",
            "replay_id",
            "step_id",
            "bind_name",
            "bind_value",
        ] {
            assert!(json.get(key).is_some(), "missing field {key:?}");
        }
    }

    /// Real, end-to-end: spawns the actual escript, which really boots `arazzo_runner`, starts a
    /// real supervised workflow, and drives a real `arazzo_runner_dispatch_statem` through the real
    /// 8-state lawful path. Marked `#[ignore]` for the same reason
    /// `f15_air_transition_core::bridge`'s own integration tests are: it needs `escript` on `PATH`
    /// and compiled `apps/arazzo_runner` + `apps/air_core` (`just erlang-compile`). Run explicitly
    /// with `cargo test -p multifractal-workflow -- --ignored`.
    ///
    /// Verified manually this session via direct `escript` invocation: a lawful single-step
    /// dispatch traverses all 8 atlas states and returns a real, non-empty dispatch token.
    #[test]
    #[ignore = "requires escript on PATH and apps/arazzo_runner+apps/air_core compiled via `just erlang-compile`; run with --ignored"]
    fn lawful_dispatch_completes_through_all_eight_real_atlas_states() {
        let request = sample_request();
        let outcome = call_dispatch_statem_bridge(&request).expect("real bridge call must succeed");
        match outcome {
            DispatchStatemOutcome::Completed {
                step_id,
                transition_log,
                dispatch_token,
            } => {
                assert_eq!(
                    transition_log,
                    vec![
                        "manufactured",
                        "ready",
                        "dispatched",
                        "awaiting_result",
                        "awaiting_admission",
                        "running",
                        "completed",
                    ]
                );
                assert!(!dispatch_token.is_empty());
                // The real running gen_statem's own `#d.step_id` (read back via
                // `get_step_id/1`) matches the step this request actually asked for --
                // the same narrow correspondence property
                // `crown_external_test.rs`'s own F15->F16 test checks against a real
                // `air_core`-driven multi-step transition.
                assert_eq!(step_id, request.step_id);
            }
            other => panic!("expected Completed, got {other:?}"),
        }
    }

    /// Real, end-to-end (see above for `#[ignore]` rationale). An empty `correlation_id` triggers
    /// the real `arazzo_runner_broker:dispatch/4` `CORRELATION_MISSING` preactuation refusal --
    /// verified manually this session via direct `escript` invocation.
    #[test]
    #[ignore = "requires escript on PATH and apps/arazzo_runner+apps/air_core compiled via `just erlang-compile`; run with --ignored"]
    fn missing_correlation_id_is_refused_by_the_real_broker_not_silently_accepted() {
        let mut request = sample_request();
        request.correlation_id = String::new();
        request.workflow_id = "wf-f16-bridge-refusal-test".to_string();
        let outcome = call_dispatch_statem_bridge(&request).expect("real bridge call must succeed");
        match outcome {
            DispatchStatemOutcome::Refused {
                step_id,
                transition_log,
                refusal_atom,
            } => {
                assert_eq!(
                    transition_log,
                    vec![
                        "manufactured",
                        "ready",
                        "dispatched",
                        "awaiting_result",
                        "awaiting_admission",
                        "refused",
                    ]
                );
                assert_eq!(refusal_atom, "CORRELATION_MISSING");
                assert_eq!(step_id, request.step_id);
            }
            other => panic!("expected Refused, got {other:?}"),
        }
    }

    /// Bounded-depth strategy for arbitrary JSON *values* (null/bool/number/
    /// string leaves; array/object composites up to depth 4, at most 8
    /// elements per level, `BTreeMap` per this repo's determinism
    /// convention). Used below to generate well-formed-JSON-but-wrong-shape
    /// input for [`parse_dispatch_statem_stdout`] -- syntactically valid,
    /// but not the `RawDispatchStatemResponse` shape the escript contract
    /// expects.
    fn arb_json_value() -> impl proptest::strategy::Strategy<Value = serde_json::Value> {
        use proptest::prelude::*;
        let leaf = prop_oneof![
            Just(serde_json::Value::Null),
            any::<bool>().prop_map(serde_json::Value::Bool),
            any::<i64>().prop_map(|n| serde_json::json!(n)),
            ".{0,16}".prop_map(serde_json::Value::String),
        ];
        leaf.prop_recursive(4, 64, 8, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..8).prop_map(serde_json::Value::Array),
                prop::collection::btree_map(".{0,8}", inner, 0..8)
                    .prop_map(|m| serde_json::Value::Object(m.into_iter().collect())),
            ]
        })
    }

    /// One field's worth of arbitrary, possibly-wrong-typed JSON (bool where
    /// a string was expected, array where bool was expected, etc.) -- used
    /// to build "known field names, wrong types" fixtures below.
    fn arb_field_value() -> impl proptest::strategy::Strategy<Value = serde_json::Value> {
        use proptest::prelude::*;
        prop_oneof![
            Just(serde_json::Value::Null),
            any::<bool>().prop_map(serde_json::Value::Bool),
            any::<i64>().prop_map(|n| serde_json::json!(n)),
            ".{0,16}".prop_map(serde_json::Value::String),
            prop::collection::vec(".{0,8}".prop_map(serde_json::Value::String), 0..4)
                .prop_map(serde_json::Value::Array),
        ]
    }

    proptest::proptest! {
        /// For ANY generated arbitrary string -- garbage, non-JSON, empty,
        /// or accidentally-valid -- [`parse_dispatch_statem_stdout`] must
        /// return a `Result` (a typed
        /// `Err(DispatchStatemBridgeRefused::MalformedResponse)` in the
        /// overwhelmingly common malformed case) and must never panic. This
        /// is the escript-stdout boundary this file previously only covered
        /// with hand-picked negative cases (the `#[ignore]`d integration
        /// tests above); proptest's own panic-catching shrinker turns any
        /// panic here into a minimal reproducing failure rather than a
        /// silent pass.
        #[test]
        fn parse_dispatch_statem_stdout_never_panics_on_arbitrary_strings(raw in ".{0,256}") {
            let _ = parse_dispatch_statem_stdout(&raw);
        }

        /// For ANY prefix-truncation of a real, well-formed
        /// `{"ok":true,"outcome":"completed",...}` response (simulating the
        /// escript being killed mid-write, or a pipe cut short),
        /// [`parse_dispatch_statem_stdout`] must return a typed `Err`, never
        /// panic.
        #[test]
        fn parse_dispatch_statem_stdout_never_panics_on_truncated_valid_response(cut in 0usize..170) {
            let full = r#"{"ok":true,"outcome":"completed","step_id":"s1","transition_log":["manufactured","ready"],"dispatch_token":"tok-1","refusal_atom":null,"error":null}"#;
            let truncated = &full[..cut.min(full.len())];
            let _ = parse_dispatch_statem_stdout(truncated);
        }

        /// For ANY generated object using the escript contract's real field
        /// names (`ok`, `outcome`, `step_id`, `transition_log`,
        /// `dispatch_token`, `refusal_atom`, `error`) but arbitrary, often
        /// wrong-typed values for each, [`parse_dispatch_statem_stdout`] must
        /// return a `Result`, never panic. This is the "wrong-typed JSON"
        /// case specifically (valid JSON syntax, valid field names, invalid
        /// field *types*) as distinct from the purely-garbage-string case
        /// above.
        #[test]
        fn parse_dispatch_statem_stdout_never_panics_on_wrong_typed_known_fields(
            ok_v in arb_field_value(),
            outcome_v in arb_field_value(),
            step_id_v in arb_field_value(),
            transition_log_v in arb_field_value(),
            dispatch_token_v in arb_field_value(),
            refusal_atom_v in arb_field_value(),
            error_v in arb_field_value(),
        ) {
            let obj = serde_json::json!({
                "ok": ok_v,
                "outcome": outcome_v,
                "step_id": step_id_v,
                "transition_log": transition_log_v,
                "dispatch_token": dispatch_token_v,
                "refusal_atom": refusal_atom_v,
                "error": error_v,
            });
            let raw = obj.to_string();
            let _ = parse_dispatch_statem_stdout(&raw);
        }

        /// For ANY generated arbitrary JSON value (not necessarily an
        /// object at all -- may be a bare number, array, string, or null),
        /// [`parse_dispatch_statem_stdout`] must return a `Result`, never
        /// panic.
        #[test]
        fn parse_dispatch_statem_stdout_never_panics_on_arbitrary_json_values(value in arb_json_value()) {
            let raw = value.to_string();
            let _ = parse_dispatch_statem_stdout(&raw);
        }
    }
}

//! F15's real Rust<->Erlang bridge (V12-015's previously-`NotYetImplemented`
//! gap -- see the parent module's doc comment, "why this module does not
//! thinly wrap `air_core.erl`", for the full BLOCKED analysis this closes).
//!
//! # What this is
//!
//! A minimal OS-process bridge: this Rust code spawns
//! `apps/air_core/scripts/air_core_bridge.escript` as a child process,
//! writes one line of JSON to its stdin, and reads one line of JSON back
//! from its stdout. The escript calls the REAL, `rebar3`-compiled
//! `air_core:new/1`, `air_core:transition/2`, and `air_core:ready_steps/1`
//! (`apps/air_core/src/air_core.erl`) -- including the real PROJ-756
//! AND/join readiness bitmask logic -- and reports back exactly what those
//! functions computed. This module does not reimplement any AIR transition
//! semantics; every readiness/command answer it returns was computed by
//! the actual Erlang code, one process boundary away.
//!
//! # Why a spawned escript, not a NIF or the distribution protocol
//!
//! `eval_expr_nif` (`apps/air_core/native/air_core_nif/src/lib.rs`) is
//! Erlang calling Rust, one direction, and only from inside a loaded BEAM
//! NIF call -- it cannot be the *entry point* for a standalone Rust
//! process like this crate. A real Erlang distribution client (a fake node
//! joining the cluster, cookie auth, EPMD) would close the same gap with
//! materially more machinery and a larger attack surface for one family's
//! minimal bridge. `escript` is a single already-available OTP tool
//! (confirmed on `PATH` this session:
//! `/Users/sac/.erlmcp/otp-28.3.1/bin/escript`) with a narrow, auditable
//! contract: one JSON line in, one JSON line out, exit 0. No code is
//! generated, compiled, or hot-loaded by this bridge or its Erlang side --
//! it only calls `air_core`'s own, already-compiled, already-tested
//! functions. This is a deliberate, disclosed scope choice, not a claim
//! that a distribution-protocol bridge is impossible.
//!
//! # Disclosed scope limits (real, not hidden)
//!
//! - **Stateless per call**: every [`call_air_core_bridge`] invocation
//!   spawns a fresh escript process with a fresh `air_core:new/1` context
//!   (`completed_mask = 0`). Multiple events in one call are folded
//!   through the SAME context in order (via the escript's own real
//!   `lists:foldl` over `air_core:transition/2`, matching how a real
//!   caller drives many events through one long-lived context) -- but
//!   nothing persists between separate calls to this function. A
//!   long-lived session bridge (reusing one spawned escript/port across
//!   many calls) is real future work, not attempted here.
//! - **No `outputs` bind-rule wiring**: the escript always treats a step's
//!   `outputs` as absent, so `air_core:bind_outputs/3`'s expr-AST path
//!   (which reaches `eval_expr_nif`, the Rust expression VM) is not
//!   exercised through this bridge. JSON has no direct encoding for the
//!   `expr()` AST; inventing one is real future work (tracked under
//!   V12-015), not attempted here.
//! - **`result`/`reason` payloads** are plain `serde_json::Value` values,
//!   converted structurally to Erlang terms on the escript side (map/list/
//!   binary/number/bool/`null`->`undefined`) -- not validated against any
//!   Erlang type beyond what `air_core:transition/2` itself accepts.
//! - **Requires an out-of-band build step**: `apps/air_core` must already
//!   be compiled (`just erlang-compile`, which produces
//!   `_build/default/lib/air_core/{ebin,priv}`) and `escript` must be on
//!   `PATH`. [`call_air_core_bridge`] fails loud with a typed
//!   [`AirBridgeRefused`] variant in either case -- it never silently
//!   falls back to a Rust reimplementation.
//!
//! Verified for real this session (see the `#[ignore]`d integration tests
//! at the bottom of this file, run explicitly with `--ignored`, and via
//! direct manual `escript` invocations recorded in the module's own
//! session notes): the escript, called through this exact Rust code path,
//! reproduces `air_core.erl`'s real AND/join behavior --
//! `{"ready_steps":["B"],...}` when only one of two AND-joined
//! predecessors has completed, and `{"ready_steps":["C"],
//! "commands":[{"step_id":"C"}]}` only once both have, and never becomes
//! ready when a predecessor `step_failed` instead of completing (PROJ-756
//! semantics).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// One step's forward edges, as the escript's `to_workflow/1` expects
/// (`apps/air_core/scripts/air_core_bridge.escript`). Mirrors the subset of
/// `air_core.erl`'s `StepDef` map this bridge exercises (`next` only -- see
/// the module doc comment's disclosed scope limit on `outputs`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct BridgeStepDef {
    pub next: Vec<String>,
}

/// A workflow's step graph, keyed by step id -- the JSON shape
/// `air_core:new/1` (via the escript's `to_workflow/1`) expects for its
/// `workflow => #{steps => ...}` option.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BridgeWorkflow {
    pub steps: BTreeMap<String, BridgeStepDef>,
}

/// Mirrors `air_core.erl`'s `event()` type
/// (`{step_completed, StepId, Result} | {step_failed, StepId, Reason}`,
/// `air_core.erl:60-61`) as the tagged JSON shape the escript's
/// `to_event/1` decodes. `#[serde(tag = "type", rename_all =
/// "snake_case")]` produces exactly `{"type":"step_completed",...}` /
/// `{"type":"step_failed",...}`, matching the escript's own match on
/// `<<"step_completed">>` / `<<"step_failed">>`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BridgeEvent {
    StepCompleted {
        step_id: String,
        result: serde_json::Value,
    },
    StepFailed {
        step_id: String,
        reason: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize)]
struct BridgeRequest {
    workflow: BridgeWorkflow,
    active_steps: Vec<String>,
    events: Vec<BridgeEvent>,
}

/// One `{dispatch_step, StepId, StepDef}` command
/// (`air_core.erl:72`), as reported back by the escript's
/// `command_to_json/1` (`StepDef` itself is dropped -- see that function --
/// so only `step_id` round-trips).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct BridgeCommand {
    pub step_id: String,
}

/// The real result of folding `events` through one `air_core:new/1`
/// context via the actual Erlang `air_core:transition/2` -- both fields
/// come directly from `air_core:ready_steps/1` and the accumulated
/// `command()` list the escript observed, not from any Rust-side
/// recomputation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTransitionResult {
    pub ready_steps: Vec<String>,
    pub commands: Vec<BridgeCommand>,
}

#[derive(Debug, Deserialize)]
struct RawBridgeResponse {
    ok: bool,
    #[serde(default)]
    ready_steps: Vec<String>,
    #[serde(default)]
    commands: Vec<BridgeCommand>,
    #[serde(default)]
    error: Option<String>,
}

/// Typed refusal taxonomy for the bridge itself -- process-boundary and
/// protocol failures, deliberately kept separate from
/// [`super::AIRTransitionRefused`] (which mirrors `arazzo_runner_broker.erl`
/// atoms specifically; these variants have no Erlang-side citation because
/// they are new infrastructure this pass built, not a mirror of pre-existing
/// Erlang refusals). Every variant fails loud with the concrete offender
/// named -- no variant swallows a failure into a default/empty success.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AirBridgeRefused {
    /// `apps/air_core/scripts/air_core_bridge.escript` does not exist at
    /// the resolved path -- most commonly means this Rust crate's
    /// `CARGO_MANIFEST_DIR`-relative repo-root resolution is wrong, not
    /// that the bridge itself is broken.
    #[error("air_core bridge script not found at {path}")]
    ScriptMissing { path: String },
    /// The repo root could not be resolved from `CARGO_MANIFEST_DIR`.
    #[error("could not resolve repo root from CARGO_MANIFEST_DIR={manifest_dir}")]
    RepoRootUnresolved { manifest_dir: String },
    /// The request could not be serialized to JSON (defensive -- no known
    /// input shape in this module triggers this; kept typed rather than
    /// `.unwrap()`-ed away).
    #[error("failed to serialize bridge request: {reason}")]
    RequestSerializeFailed { reason: String },
    /// `escript` could not be spawned at all -- most commonly means
    /// `escript` is not on `PATH` in this environment, or
    /// `apps/air_core` has not been compiled yet (`just erlang-compile`).
    #[error("failed to spawn `escript {script}`: {reason} (is escript on PATH?)")]
    SpawnFailed { script: String, reason: String },
    /// The child process's stdin could not be written to (e.g. the escript
    /// exited before consuming its request).
    #[error("failed to write request to the air_core bridge's stdin: {reason}")]
    StdinWriteFailed { reason: String },
    /// The escript exited with a nonzero status -- an escript-level crash
    /// (e.g. a malformed request the escript's own try/catch did not
    /// anticipate), distinct from an `{"ok":false,...}` application-level
    /// refusal (which exits 0 -- see [`ErlangSideError`](Self::ErlangSideError)).
    #[error("air_core bridge exited with status {status:?}; stderr: {stderr}")]
    NonZeroExit { status: Option<i32>, stderr: String },
    /// The escript's stdout was not the expected `{"ok":...}` JSON shape.
    #[error("air_core bridge produced an unparseable response: {reason}; raw={raw}")]
    MalformedResponse { reason: String, raw: String },
    /// The escript itself reported `{"ok":false,"error":"..."}` -- a real
    /// failure the Erlang side observed (e.g. an unknown event type, or an
    /// `air_core` call that raised), not a Rust-side parsing problem.
    #[error("air_core bridge reported an error from the real Erlang call: {message}")]
    ErlangSideError { message: String },
    /// The escript did not exit within the bounded wait window
    /// (`BRIDGE_WAIT_TIMEOUT_SECS`) -- most commonly a genuine Erlang-side
    /// hang (a real deadlock, or a broker call that never returns), not a
    /// Rust-side bug. The child process has already been killed
    /// (`Child::kill()`) by the time this variant is returned -- no process
    /// is left running past this error. This closes a real production-
    /// reliability gap: before this variant existed, a hung escript hung
    /// the Rust caller forever with no recovery path.
    #[error("air_core bridge did not exit within {seconds}s and was killed")]
    Timeout { seconds: u64 },
}

/// Resolves the repo root from `CARGO_MANIFEST_DIR`
/// (`<repo>/crates/multifractal-workflow`), matching
/// `f17_atomvm_runtime.rs`'s `repo_root()` convention (two path segments
/// up) rather than inventing a second resolution strategy.
fn repo_root() -> Result<PathBuf, AirBridgeRefused> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| AirBridgeRefused::RepoRootUnresolved {
            manifest_dir: manifest_dir.display().to_string(),
        })
}

/// How long [`call_air_core_bridge`] waits for the spawned escript to exit
/// before treating it as hung and killing it (see [`wait_with_timeout`]).
/// 30s is generous for a local subprocess call -- no network round trip is
/// involved, the escript only spawns a local BEAM VM and calls already-
/// compiled `air_core` functions -- while still bounding a genuine
/// Erlang-side deadlock, or a broker call that never returns, to a finite,
/// recoverable wait instead of hanging the Rust caller forever.
const BRIDGE_WAIT_TIMEOUT_SECS: u64 = 30;

/// Internal outcome of [`wait_with_timeout`], mapped by
/// [`call_air_core_bridge`] to a variant of [`AirBridgeRefused`]. Kept
/// private to this module -- `f16_otp_runner::bridge` has its own,
/// independent copy of this same watchdog, matching this file's existing
/// convention of not sharing infra across families (see [`repo_root`]'s doc
/// comment, which states the same rationale for its own duplication).
#[derive(Debug)]
enum BoundedWaitError {
    /// The child did not exit within the bounded wait window; by the time
    /// this variant is returned, the child has already been killed
    /// (`Child::kill()`) and reaped (`Child::wait()`).
    TimedOut,
    /// `Child::try_wait()` itself returned an OS-level error, distinct from
    /// a timeout. The child is killed defensively before this variant is
    /// returned.
    Io(std::io::Error),
    /// The stdout or stderr reader thread panicked (defensive -- neither
    /// reader closure in [`wait_with_timeout`] contains an `.unwrap()`/
    /// `panic!()` path, so this should be unreachable in practice; kept
    /// typed rather than silently discarding the partial output).
    ReaderPanicked,
}

/// Bounded, deadlock-safe replacement for `Child::wait_with_output()`.
///
/// `Child::wait_with_output()` blocks the calling thread until the child
/// exits, with no timeout -- a real Erlang-side deadlock, or a broker call
/// that never returns, previously hung the Rust caller forever with no
/// recovery path. This function instead polls `Child::try_wait()` from the
/// calling thread (never blocking longer than `POLL_INTERVAL` at a stretch)
/// and, if `timeout` elapses before the child exits, kills the child
/// (`Child::kill()`), reaps it (`Child::wait()`), and returns
/// `Err(BoundedWaitError::TimedOut)` instead of hanging.
///
/// `stdout`/`stderr` are drained concurrently on two dedicated reader
/// threads -- the same technique the standard library's own
/// `wait_with_output` uses internally -- so a child that writes more output
/// than the OS pipe buffer holds cannot deadlock against an undrained pipe
/// while this function polls for exit. `stdin` is dropped up front (matching
/// `Child::wait_with_output()`'s own first step), so a child reading its
/// stdin to EOF is not left waiting on a stdin handle this function still
/// holds open.
///
/// # Complexity
/// O(timeout / POLL_INTERVAL) polling wake-ups on the calling thread in the
/// worst case (a child that never exits); O(1) once the child exits
/// promptly, which is the expected case for every real production call. The
/// two reader threads are O(n) in the number of bytes the child writes to
/// stdout/stderr -- bounded in practice by this bridge's single-line JSON
/// payloads.
fn wait_with_timeout(mut child: Child, timeout: Duration) -> Result<Output, BoundedWaitError> {
    const POLL_INTERVAL: Duration = Duration::from_millis(20);

    // Matches `Child::wait_with_output()`'s own first step: drop stdin so a
    // child reading to EOF is not left blocked on a handle we still hold.
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

/// Calls the real `air_core:new/1` + `air_core:transition/2` +
/// `air_core:ready_steps/1` chain (via
/// `apps/air_core/scripts/air_core_bridge.escript`) for `events`, applied
/// in order against one fresh context seeded with `workflow`/`active_steps`.
///
/// # Complexity
/// O(1) Rust-side (one process spawn + one JSON encode/decode); the
/// Erlang-side cost is `air_core.erl`'s own documented
/// O(|NextSteps|)-per-event `newly_ready_successors/5` bound.
pub fn call_air_core_bridge(
    workflow: &BridgeWorkflow,
    active_steps: &[String],
    events: &[BridgeEvent],
) -> Result<BridgeTransitionResult, AirBridgeRefused> {
    let root = repo_root()?;
    let script_path = root
        .join("apps")
        .join("air_core")
        .join("scripts")
        .join("air_core_bridge.escript");
    if !script_path.is_file() {
        return Err(AirBridgeRefused::ScriptMissing {
            path: script_path.display().to_string(),
        });
    }
    let erl_libs = root.join("_build").join("default").join("lib");

    let request = BridgeRequest {
        workflow: BridgeWorkflow {
            steps: workflow.steps.clone(),
        },
        active_steps: active_steps.to_vec(),
        events: events.to_vec(),
    };
    let request_json =
        serde_json::to_string(&request).map_err(|e| AirBridgeRefused::RequestSerializeFailed {
            reason: e.to_string(),
        })?;

    let mut child = Command::new("escript")
        .arg(&script_path)
        .env("ERL_LIBS", &erl_libs)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AirBridgeRefused::SpawnFailed {
            script: script_path.display().to_string(),
            reason: e.to_string(),
        })?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| AirBridgeRefused::StdinWriteFailed {
                reason: "child process has no stdin handle".to_string(),
            })?;
        writeln!(stdin, "{request_json}").map_err(|e| AirBridgeRefused::StdinWriteFailed {
            reason: e.to_string(),
        })?;
    }

    let output = wait_with_timeout(child, Duration::from_secs(BRIDGE_WAIT_TIMEOUT_SECS)).map_err(
        |e| match e {
            BoundedWaitError::TimedOut => AirBridgeRefused::Timeout {
                seconds: BRIDGE_WAIT_TIMEOUT_SECS,
            },
            BoundedWaitError::Io(io_err) => AirBridgeRefused::SpawnFailed {
                script: script_path.display().to_string(),
                reason: format!("failed waiting for child output: {io_err}"),
            },
            BoundedWaitError::ReaderPanicked => AirBridgeRefused::SpawnFailed {
                script: script_path.display().to_string(),
                reason: "stdout/stderr reader thread panicked while waiting for child output"
                    .to_string(),
            },
        },
    )?;

    if !output.status.success() {
        return Err(AirBridgeRefused::NonZeroExit {
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    let raw = String::from_utf8_lossy(&output.stdout).into_owned();
    parse_bridge_stdout(&raw)
}

/// Parses one line of the escript's stdout (the exact bytes
/// [`call_air_core_bridge`] would have read from `output.stdout`) into a
/// [`BridgeTransitionResult`], without spawning any process.
///
/// Factored out of [`call_air_core_bridge`] so this JSON-decode/response-
/// contract boundary is independently testable against arbitrary
/// (malformed, truncated, wrong-typed) input -- see the `proptest` suite in
/// this module's tests, which feeds this function directly rather than
/// spawning a real escript for every case.
///
/// # Complexity
/// O(n) in the length of `raw` (one `serde_json` parse pass).
fn parse_bridge_stdout(raw: &str) -> Result<BridgeTransitionResult, AirBridgeRefused> {
    let parsed: RawBridgeResponse =
        serde_json::from_str(raw.trim()).map_err(|e| AirBridgeRefused::MalformedResponse {
            reason: e.to_string(),
            raw: raw.to_string(),
        })?;

    if !parsed.ok {
        return Err(AirBridgeRefused::ErlangSideError {
            message: parsed
                .error
                .unwrap_or_else(|| "ok=false with no error field".to_string()),
        });
    }

    Ok(BridgeTransitionResult {
        ready_steps: parsed.ready_steps,
        commands: parsed.commands,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn and_join_workflow() -> BridgeWorkflow {
        let mut steps = BTreeMap::new();
        steps.insert(
            "A".to_string(),
            BridgeStepDef {
                next: vec!["C".to_string()],
            },
        );
        steps.insert(
            "B".to_string(),
            BridgeStepDef {
                next: vec!["C".to_string()],
            },
        );
        steps.insert("C".to_string(), BridgeStepDef { next: vec![] });
        BridgeWorkflow { steps }
    }

    /// Proves the watchdog's timeout path is real and reachable, not just
    /// declared: a real `sleep 5` OS subprocess (a deliberately slow test
    /// double for a genuinely hung escript -- a real Erlang deadlock, or a
    /// broker call that never returns -- not a mock) is spawned with the
    /// exact same `Stdio::piped()` shape [`call_air_core_bridge`] uses, then
    /// waited on with a 200ms timeout (far shorter than the 5s sleep). This
    /// test does not run for 5 real seconds: it asserts
    /// [`wait_with_timeout`] returns `TimedOut` well before that, and then
    /// checks the real OS process table (`ps -p <pid>`) to confirm the
    /// child was actually killed, not merely abandoned still running --
    /// exactly the production-reliability property this fix exists for.
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
    fn repo_root_resolves_to_a_directory_containing_justfile() {
        let root = repo_root().expect("repo root must resolve from CARGO_MANIFEST_DIR");
        assert!(
            root.join("justfile").is_file(),
            "resolved repo root {} does not contain justfile",
            root.display()
        );
    }

    #[test]
    fn bridge_script_path_exists_in_repo() {
        let root = repo_root().expect("repo root must resolve");
        let script = root
            .join("apps")
            .join("air_core")
            .join("scripts")
            .join("air_core_bridge.escript");
        assert!(
            script.is_file(),
            "expected escript at {} (this is a static repo-layout check, no process spawn)",
            script.display()
        );
    }

    #[test]
    fn bridge_event_serializes_to_the_shape_the_escript_expects() {
        let event = BridgeEvent::StepCompleted {
            step_id: "A".to_string(),
            result: serde_json::Value::Null,
        };
        let json = serde_json::to_value(&event).expect("serialize");
        assert_eq!(json["type"], "step_completed");
        assert_eq!(json["step_id"], "A");
        assert!(json["result"].is_null());

        let failed = BridgeEvent::StepFailed {
            step_id: "B".to_string(),
            reason: serde_json::json!("boom"),
        };
        let json2 = serde_json::to_value(&failed).expect("serialize");
        assert_eq!(json2["type"], "step_failed");
        assert_eq!(json2["reason"], "boom");
    }

    /// Real, end-to-end: spawns the actual escript, which calls the actual
    /// `air_core:new/1` + `air_core:transition/2` + `air_core:ready_steps/1`.
    /// Marked `#[ignore]` because it requires `escript` on `PATH` and
    /// `apps/air_core` to already be compiled (`just erlang-compile`) --
    /// not a dependency `cargo test -p multifractal-workflow` should impose
    /// on every environment by default (same convention as
    /// `f17_atomvm_runtime.rs`'s `run_otp_atomvm_differential_suite_reports_real_pass`).
    /// Run explicitly with `cargo test -p multifractal-workflow -- --ignored`.
    ///
    /// Verified manually this session, both via direct `escript` invocation
    /// and through this exact Rust code path: with `A` and `B` both
    /// AND-joined into `C` and only `A` completed, `ready_steps` is `["B"]`
    /// (still waiting) and `commands` is empty -- `C` is correctly NOT
    /// ready with only one of two predecessors done. This is the real
    /// PROJ-756 `newly_ready_successors/5` bitmask logic, not a Rust
    /// reimplementation.
    #[test]
    #[ignore = "requires escript on PATH and apps/air_core compiled via `just erlang-compile`; run with --ignored"]
    fn and_join_stays_blocked_with_only_one_of_two_predecessors_done() {
        let workflow = and_join_workflow();
        let result = call_air_core_bridge(
            &workflow,
            &["A".to_string(), "B".to_string()],
            &[BridgeEvent::StepCompleted {
                step_id: "A".to_string(),
                result: serde_json::Value::Null,
            }],
        )
        .expect("real bridge call must succeed");
        assert_eq!(result.ready_steps, vec!["B".to_string()]);
        assert!(result.commands.is_empty());
    }

    /// Real, end-to-end (see the test above for the `#[ignore]` rationale).
    /// Both `A` and `B` complete (in one call, folded through one context,
    /// matching the escript's own real `lists:foldl` over
    /// `air_core:transition/2`) -- `C` becomes ready and a real
    /// `dispatch_step` command for `C` is produced, exactly the PROJ-756
    /// AND/join-satisfied case.
    #[test]
    #[ignore = "requires escript on PATH and apps/air_core compiled via `just erlang-compile`; run with --ignored"]
    fn and_join_becomes_ready_once_both_predecessors_complete() {
        let workflow = and_join_workflow();
        let result = call_air_core_bridge(
            &workflow,
            &["A".to_string(), "B".to_string()],
            &[
                BridgeEvent::StepCompleted {
                    step_id: "A".to_string(),
                    result: serde_json::Value::Null,
                },
                BridgeEvent::StepCompleted {
                    step_id: "B".to_string(),
                    result: serde_json::json!(42),
                },
            ],
        )
        .expect("real bridge call must succeed");
        assert_eq!(result.ready_steps, vec!["C".to_string()]);
        assert_eq!(
            result.commands,
            vec![BridgeCommand {
                step_id: "C".to_string()
            }]
        );
    }

    /// Real, end-to-end (see above for `#[ignore]` rationale). A `step_failed`
    /// predecessor must never satisfy an AND/join successor -- PROJ-756's
    /// documented invariant (`air_core.erl:288-294`: `handle_step_failed/3`
    /// deliberately never sets a `completed_mask` bit). Verified against the
    /// real Erlang function, not asserted.
    #[test]
    #[ignore = "requires escript on PATH and apps/air_core compiled via `just erlang-compile`; run with --ignored"]
    fn and_join_never_becomes_ready_when_a_predecessor_fails() {
        let workflow = and_join_workflow();
        let result = call_air_core_bridge(
            &workflow,
            &["A".to_string(), "B".to_string()],
            &[
                BridgeEvent::StepFailed {
                    step_id: "A".to_string(),
                    reason: serde_json::json!("boom"),
                },
                BridgeEvent::StepCompleted {
                    step_id: "B".to_string(),
                    result: serde_json::json!(1),
                },
            ],
        )
        .expect("real bridge call must succeed");
        assert!(result.ready_steps.is_empty());
        assert!(result.commands.is_empty());
    }

    /// Real, end-to-end (see above). A malformed request the escript itself
    /// cannot decode must surface as a typed refusal, never a fabricated
    /// empty success.
    #[test]
    #[ignore = "requires escript on PATH and apps/air_core compiled via `just erlang-compile`; run with --ignored"]
    fn unknown_event_type_is_refused_not_silently_dropped() {
        // Constructed directly (bypassing BridgeEvent) to exercise the
        // escript's own `unknown_event_type` throw path.
        let root = repo_root().expect("repo root must resolve");
        let script_path = root
            .join("apps")
            .join("air_core")
            .join("scripts")
            .join("air_core_bridge.escript");
        let erl_libs = root.join("_build").join("default").join("lib");
        let mut child = Command::new("escript")
            .arg(&script_path)
            .env("ERL_LIBS", &erl_libs)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn escript");
        {
            let stdin = child.stdin.as_mut().expect("stdin handle");
            writeln!(
                stdin,
                r#"{{"workflow":{{"steps":{{}}}},"active_steps":[],"events":[{{"type":"bogus","step_id":"x"}}]}}"#
            )
            .expect("write request");
        }
        let output = child.wait_with_output().expect("wait for output");
        assert!(output.status.success(), "escript itself must not crash");
        let raw = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(raw.trim()).expect("parse JSON");
        assert_eq!(parsed["ok"], false);
        assert!(parsed["error"]
            .as_str()
            .unwrap_or_default()
            .contains("unknown_event_type"));
    }

    /// Bounded-depth strategy for arbitrary JSON *values* (null/bool/number/
    /// string leaves; array/object composites up to depth 4, at most 8
    /// elements per level, `BTreeMap` per this repo's determinism
    /// convention). Used below to generate well-formed-JSON-but-wrong-shape
    /// input for [`parse_bridge_stdout`] -- syntactically valid, but not the
    /// `RawBridgeResponse` shape the escript contract expects.
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
    /// an array was expected, string where bool was expected, etc.) -- used
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
        /// or accidentally-valid -- [`parse_bridge_stdout`] must return a
        /// `Result` (a typed `Err(AirBridgeRefused::MalformedResponse)` in
        /// the overwhelmingly common malformed case) and must never panic.
        /// This is the escript-stdout boundary this file previously only
        /// covered with hand-picked negative cases (e.g.
        /// `unknown_event_type_is_refused_not_silently_dropped` above);
        /// proptest's own panic-catching shrinker turns any panic here into
        /// a minimal reproducing failure rather than a silent pass.
        #[test]
        fn parse_bridge_stdout_never_panics_on_arbitrary_strings(raw in ".{0,256}") {
            let _ = parse_bridge_stdout(&raw);
        }

        /// For ANY prefix-truncation of a real, well-formed `{"ok":true,...}`
        /// response (simulating the escript being killed mid-write, or a
        /// pipe cut short), [`parse_bridge_stdout`] must return a typed
        /// `Err`, never panic -- most cut points land mid-token, which
        /// `serde_json` must reject as `Err`, not something this Rust code
        /// should crash decoding.
        #[test]
        fn parse_bridge_stdout_never_panics_on_truncated_valid_response(cut in 0usize..120) {
            let full = r#"{"ok":true,"ready_steps":["A","B"],"commands":[{"step_id":"C"}],"error":null}"#;
            let truncated = &full[..cut.min(full.len())];
            let _ = parse_bridge_stdout(truncated);
        }

        /// For ANY generated object using the escript contract's real field
        /// names (`ok`, `ready_steps`, `commands`, `error`) but arbitrary,
        /// often wrong-typed values for each, [`parse_bridge_stdout`] must
        /// return a `Result`, never panic. This is the "wrong-typed JSON"
        /// case specifically (valid JSON syntax, valid field names, invalid
        /// field *types*) as distinct from the purely-garbage-string case
        /// above.
        #[test]
        fn parse_bridge_stdout_never_panics_on_wrong_typed_known_fields(
            ok_v in arb_field_value(),
            ready_steps_v in arb_field_value(),
            commands_v in arb_field_value(),
            error_v in arb_field_value(),
        ) {
            let obj = serde_json::json!({
                "ok": ok_v,
                "ready_steps": ready_steps_v,
                "commands": commands_v,
                "error": error_v,
            });
            let raw = obj.to_string();
            let _ = parse_bridge_stdout(&raw);
        }

        /// For ANY generated arbitrary JSON value (not necessarily an
        /// object at all -- may be a bare number, array, string, or null),
        /// [`parse_bridge_stdout`] must return a `Result`, never panic.
        #[test]
        fn parse_bridge_stdout_never_panics_on_arbitrary_json_values(value in arb_json_value()) {
            let raw = value.to_string();
            let _ = parse_bridge_stdout(&raw);
        }
    }
}

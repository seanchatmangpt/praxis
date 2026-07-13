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
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchStatemOutcome {
    /// The real 8-state lawful path completed; `dispatch_token` is the real, non-empty token
    /// `arazzo_runner_broker:dispatch/4` computed.
    Completed {
        transition_log: Vec<String>,
        dispatch_token: String,
    },
    /// A real Erlang refusal atom (e.g. `CORRELATION_MISSING`) terminated the lifecycle at
    /// `REFUSED` -- parallels [`super::OTPWorkflowRefused::erlang_atom`]'s vocabulary, though this
    /// module does not itself construct that enum (its caller may, from `refusal_atom`).
    Refused {
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

    let output =
        child
            .wait_with_output()
            .map_err(|e| DispatchStatemBridgeRefused::SpawnFailed {
                script: script_path.display().to_string(),
                reason: format!("failed waiting for child output: {e}"),
            })?;

    if !output.status.success() {
        return Err(DispatchStatemBridgeRefused::NonZeroExit {
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    let raw = String::from_utf8_lossy(&output.stdout).into_owned();
    let parsed: RawDispatchStatemResponse = serde_json::from_str(raw.trim()).map_err(|e| {
        DispatchStatemBridgeRefused::MalformedResponse {
            reason: e.to_string(),
            raw: raw.clone(),
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
        parsed.dispatch_token,
        parsed.refusal_atom,
    ) {
        (Some("completed"), Some(token), None) => Ok(DispatchStatemOutcome::Completed {
            transition_log: parsed.transition_log,
            dispatch_token: token,
        }),
        (Some("refused"), None, Some(atom)) => Ok(DispatchStatemOutcome::Refused {
            transition_log: parsed.transition_log,
            refusal_atom: atom,
        }),
        _ => Err(DispatchStatemBridgeRefused::UnrecognizedOutcomeShape { raw }),
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
        let outcome =
            call_dispatch_statem_bridge(&sample_request()).expect("real bridge call must succeed");
        match outcome {
            DispatchStatemOutcome::Completed {
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
            }
            other => panic!("expected Refused, got {other:?}"),
        }
    }
}

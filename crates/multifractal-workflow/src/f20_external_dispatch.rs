//! Family F20 -- "External Dispatch and Re-admission" (atlas ticket V12-020).
//!
//! Survey verdict: **ALREADY_BUILT**. `crates/cng/src/bench/decomp/dispatch_bridge.rs`
//! (PROJ-618/720/721/723, v26.7.10) already implements the atlas's Dispatch Contract
//! Builder -> Broker Dispatch -> [Return Collector -> Provenance Correlator ->
//! Authority Re-check -> Conformance Gate -> Admission Callback] pipeline over a real
//! filesystem inbox/outbox (`EngineBundle`, `crates/cng/src/bench/engine.rs`). This
//! module thinly wraps that real, already-tested machinery rather than reimplementing
//! it -- per this crate's reuse mandate, and per `.claude/rules/no-overclaiming.md`,
//! every claim below is scoped to what this module itself does and what this session
//! verified, not to cng's own test suite (cited, not re-run here).
//!
//! # What is genuinely wired here (ALIVE this session)
//!
//! - [`dispatch_subworkflow_to_engine`] / [`collect_subworkflow_consequence`] /
//!   [`SubworkflowDispatchHandle`] / [`SubworkflowDispatchOutcome`] are re-exported
//!   directly from `cng::bench::decomp::dispatch_bridge` -- real functions, not
//!   fakes. They are the ONLY two entry points that module exposes outside the `cng`
//!   crate (its own doc comment: "`DispatchContract` and its surrounding machinery
//!   stay crate-private ... these two entry points are the only parts exposed outside
//!   the crate").
//! - [`decompose`] / [`decompose_with`] / [`DecompositionOutcome`] / [`DecompositionResult`]
//!   / [`SubworkflowPlan`] are re-exported from `cng::bench::decomp` -- the real
//!   upstream producer of the `SubworkflowPlan` values these entry points dispatch.
//! - [`CngRefusal`] and [`Powl`] are re-exported from `cng::powl` -- the real typed
//!   refusal enum both entry points return (`CngRefusal::ExternalConsequenceRefused`
//!   is `CNG_R17`; `DispatchContractIncomplete`/`DispatchStateUnlawful`/`DoubleAdmit`
//!   are `CNG_R15`/`CNG_R16`/`CNG_R25`) and the POWL model type `SubworkflowPlan::model`
//!   carries.
//! - [`dispatch_and_await`] is new, genuine glue written in this pass: it sequences
//!   the two real bridge calls (dispatch, then bounded-poll collect) into one
//!   function. No new admission/refusal/lifecycle logic is invented -- it is pure
//!   composition of the two functions above, propagating their real `Result`s.
//! - `tests::dispatch_then_collect_reaches_a_typed_terminal_outcome_with_no_remote_engine`
//!   (below) drives [`dispatch_subworkflow_to_engine`] against a REAL temp directory,
//!   confirms a REAL contract file lands in the target engine's REAL inbox, then
//!   drives [`collect_subworkflow_consequence`] with no remote `cng engine serve`
//!   process ever running against that root, and confirms the collector still
//!   resolves to a typed `Ok(SubworkflowDispatchOutcome { admitted: false, .. })`
//!   rather than hanging, panicking, or losing the workflow -- a real (if narrow)
//!   exercise of F20's "no workflow disappears at a boundary" invariant. Verified
//!   this session via `CARGO_TARGET_DIR=target/agent-f20-wire timeout 280 cargo test
//!   -p multifractal-workflow --features cng/bench --lib f20_external_dispatch`
//!   (see the module's own test for the exact assertions; this doc comment is not a
//!   substitute for reading it).
//!
//! # Disclosed scope boundaries (not gaps invented by this pass -- inherited from cng)
//!
//! - **Decomposition-scoped, not general-purpose.** The atlas describes a generic
//!   Dispatch Contract Builder for arbitrary external consequences. The only
//!   externally `pub` cng surface is [`SubworkflowPlan`]-scoped (built for Track P
//!   decomposition -> Track E dispatch, PROJ-706..710 feeding PROJ-720..724).
//!   `crate::bench::dispatch`'s general `DispatchContract`/`workday_contract`/
//!   `remote_contract` builders and `crate::bench::engine`'s `remote_contract`/
//!   `run_serve_loop` stay `mod` (crate-private) inside `cng`; this crate cannot name
//!   or construct a `DispatchContract` directly, only build one indirectly by calling
//!   [`dispatch_subworkflow_to_engine`] with a [`SubworkflowPlan`].
//! - **Stage detail is NOT surfaced through the public bridge.** cng's private
//!   `collect_consequence` (provenance -> correlation -> authority -> structural ->
//!   semantic, first-failing-stage-wins, per its own module doc) returns a specific
//!   `CngRefusal::ExternalConsequenceRefused { dispatch, stage }` naming which stage
//!   failed. [`collect_subworkflow_consequence`] catches that `Result` and collapses
//!   it to a plain `admitted: false` (`dispatch_bridge.rs`: `Err(_) => false`) --
//!   this crate genuinely cannot distinguish "stale", "unauthorized", "malformed",
//!   or "no consequence ever appeared" from outside cng's crate boundary. The atlas's
//!   requirement #4 (typed refusal at any failed stage) IS satisfied inside cng's
//!   private machinery and IS tested there (`dispatch_test.rs`'s
//!   `forged_inbox_correlation_refuses_at_correlation_stage` and siblings, cited by
//!   the family survey, NOT re-run by this module); it is simply not re-exposed at
//!   this crate's boundary without either widening cng's public surface (out of
//!   scope for this Wire pass -- `cng` is shared, actively touched code, not owned by
//!   this ticket) or duplicating cng's private stage logic here (would be a fork, not
//!   reuse, and was avoided on purpose).
//! - **The atlas's 8-state lifecycle enum is not directly observable here.** cng's
//!   real `DispatchState` (16 states, a documented superset of the atlas's 8) is
//!   `pub(crate)`/unexported; this crate only observes the bridge's own projection of
//!   it (`SubworkflowDispatchOutcome::{admitted, polls_taken, consequence_digest}`),
//!   not the named state itself.
//! - **`EXTERNAL_HUMAN_DISPATCH` is synthesized, not a real human-in-the-loop
//!   surface**, and transport is a filesystem inbox/outbox between OS processes, not
//!   HTTP -- both boundaries cng's own code already discloses (see the family
//!   survey's `justification`), inherited unchanged here.
//! - **No `Cargo.lock`-verified full-workspace green build is claimed by this
//!   comment.** See the module's own test invocation above for the exact, narrower
//!   command actually run this session.
//!
//! # Audit (this session, V12-020 wire-phase-2 pass): production reachability + F02
//!
//! Two specific questions were investigated for real, not assumed from the ALREADY_BUILT
//! verdict above:
//!
//! 1. **Does anything besides this module's own tests call
//!    [`dispatch_and_await`]/[`dispatch_subworkflow_to_engine`]/
//!    [`collect_subworkflow_consequence`]? MISSING -- confirmed by
//!    `grep -rn "dispatch_and_await\|dispatch_subworkflow_to_engine\|collect_subworkflow_consequence" --include="*.rs" .`
//!    (repo root, this session): every non-test hit is either this module's own
//!    definitions/doc comments, or `crates/cng/src/bench/decomp/dispatch_bridge.rs`'s
//!    own definitions; every *call site* is inside a `#[cfg(test)]` module (this file's
//!    two tests, or `crates/cng/tests/cng_decompose_to_dispatch_integration.rs`). This
//!    is not scoped to `multifractal-workflow` -- `cng`'s own CLI (`crates/cng/src/
//!    main.rs`, 17 `#[verb(...)]` entries surveyed this session) has no verb that calls
//!    the bridge either: `#[verb("decompose", "plan")]` (`plan_decompose`, line 358)
//!    calls `cng::bench::decomp::decompose` and stops at writing
//!    `decomposition-result.ttl`; `#[verb("serve", "engine")]` (`engine_serve`, line
//!    945) is the RECEIVING side of the bridge (an engine's own inbox scan), not a
//!    caller of `dispatch_subworkflow_to_engine`/`collect_subworkflow_consequence`. So
//!    the underlying cng machinery this module wraps is itself integration-tested but
//!    has zero real (non-test) callers anywhere in the workspace today -- this module's
//!    own "thin wrapper" framing is accurate, but "ALREADY_BUILT" should be read as
//!    "built and tested", not "reachable from a running system". Separately,
//!    `multifractal-workflow`'s own `Cargo.toml` declares `[lib]` only (no `[[bin]]`,
//!    no `main.rs` anywhere under `crates/multifractal-workflow/`) -- by construction,
//!    *no* family module in this crate can have a production caller yet; this is a
//!    crate-wide structural fact, not specific to F20, confirmed this session by
//!    `find crates/multifractal-workflow -name main.rs -o -path "*/bin/*"` (zero
//!    results).
//! 2. **Is the atlas's F20 -> F02 re-admission edge real?** BUILT in a later pass
//!    (`crown_external.rs`'s `drive_external_reentry`), closing the gap this note
//!    originally identified. The blocker described here at the time -- `admit_observation`
//!    needs `RawObservation.payload_turtle: String` (the full text), but
//!    [`SubworkflowDispatchOutcome`] exposed only `consequence_digest: Option<String>` (a
//!    BLAKE3 digest), and `EngineBundle` (the only way to read the outbox file
//!    independently) lives behind `crates/cng/src/bench/mod.rs`'s `mod engine;` (private,
//!    not `pub mod`) -- was resolved by taking the first of the two options this note
//!    already named: **widening cng's public surface**, not duplicating its private
//!    outbox-path convention. `SubworkflowDispatchOutcome` now also carries
//!    `consequence_turtle: Option<String>` (`crates/cng/src/bench/decomp/dispatch_bridge.rs`),
//!    populated at both existing construction sites from the same `consequence_ttl` local
//!    variable `collect_subworkflow_consequence` already computed -- no new admission
//!    logic, no stage-detail surfaced, nothing else about cng's private boundary changed.
//!    [`engine_serve`] (re-exported below) is the real complementary half: this module's
//!    own earlier text already identified it as "the RECEIVING side of the bridge (an
//!    engine's own inbox scan)" with zero production callers; `drive_external_reentry`
//!    is now that caller, driving a real dispatch -> serve -> collect -> re-admit round
//!    trip end to end.

pub use cng::bench::decomp::dispatch_bridge::{
    collect_subworkflow_consequence, dispatch_subworkflow_to_engine, SubworkflowDispatchHandle,
    SubworkflowDispatchOutcome,
};
pub use cng::bench::decomp::{
    decompose, decompose_with, DecompositionOutcome, DecompositionResult, SubworkflowPlan,
};
pub use cng::bench::engine_serve;
pub use cng::powl::{CngRefusal, Powl};

use std::path::Path;

/// Composes [`dispatch_subworkflow_to_engine`] then [`collect_subworkflow_consequence`]
/// into a single call: sends `subworkflow`'s contract to `target_engine`'s real inbox,
/// then bounded-polls that engine's real outbox for a consequence and runs it through
/// the real lawful re-entry pipeline. Pure composition of the two real cng bridge
/// calls above -- no new admission, refusal, or lifecycle logic; both `Result`s
/// propagate unchanged.
///
/// # Errors
/// Whatever [`dispatch_subworkflow_to_engine`] or [`collect_subworkflow_consequence`]
/// return (`CNG_R10`/`CNG_R15`/`CNG_R16` -- see each function's own doc comment for
/// its exact error surface). A refused/absent remote consequence is NOT an `Err` here
/// (mirrors the wrapped function): it is the typed `admitted: false` field of the
/// `Ok` outcome.
///
/// # Complexity
/// O(template render + one shape check + one contract write) for dispatch, plus
/// O(`max_polls`) outbox stats for collection -- see the wrapped functions' own
/// complexity notes; this function adds no additional asymptotic cost.
pub fn dispatch_and_await(
    root: &Path,
    subworkflow: &SubworkflowPlan,
    domain_pddl: &str,
    target_engine: &str,
    max_polls: u64,
    poll_wait_ms: Option<u64>,
) -> Result<SubworkflowDispatchOutcome, CngRefusal> {
    let handle = dispatch_subworkflow_to_engine(root, subworkflow, domain_pddl, target_engine)?;
    collect_subworkflow_consequence(root, &handle, max_polls, poll_wait_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// Real scratch directory under the workspace's own `target/` (git-ignored),
    /// matching the pattern `crates/cng/tests/cng_decompose_to_dispatch_integration.rs`
    /// already uses for the same bridge. Not `std::env::temp_dir()` -- keeps test
    /// artifacts inside the repo's build output tree.
    fn scratch_dir(test_name: &str) -> PathBuf {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/multifractal-workflow-tests/f20-external-dispatch")
            .join(test_name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

    /// A minimal, real `SubworkflowPlan`: empty tape and a bare `Powl::Leaf(None)`
    /// model, since `dispatch_subworkflow_to_engine`'s own contract builder
    /// (`subworkflow_to_contract`, private to cng) only reads `id`/`role`/
    /// `problem_digest`/`problem_pddl` -- `tape`/`model` are carried for downstream
    /// consumers this test does not exercise, not read by the dispatch path itself.
    fn trivial_subworkflow(id: &str, role: &str) -> SubworkflowPlan {
        SubworkflowPlan {
            id: id.to_string(),
            role: role.to_string(),
            tape: bcinr_pddl::Pddl8Tape { ops: Vec::new() },
            model: Powl::Leaf(None),
            problem_pddl: String::new(),
            problem_digest: format!("blake3:{}", blake3::hash(id.as_bytes()).to_hex()),
        }
    }

    #[test]
    fn dispatch_then_collect_reaches_a_typed_terminal_outcome_with_no_remote_engine() {
        let root = scratch_dir("no-remote-engine");
        let subworkflow = trivial_subworkflow("wf-f20-test-1", "single");

        let handle = dispatch_subworkflow_to_engine(&root, &subworkflow, "", "f20-test-engine")
            .expect("real contract dispatch to a real inbox must succeed for a valid subworkflow");
        assert!(handle.dispatch_id.starts_with("disp-decomp-single-"));
        assert_eq!(handle.role, "single");
        assert_eq!(handle.target_engine, "f20-test-engine");

        // The contract landed as a real file in the real inbox -- not a mocked or
        // decorative dispatch.
        let contract_path = root
            .join("engines")
            .join("f20-test-engine")
            .join("inbox")
            .join(format!("{}.ttl", handle.dispatch_id));
        assert!(
            contract_path.is_file(),
            "dispatch contract must be a real file on disk at {}",
            contract_path.display()
        );
        let rendered = fs::read_to_string(&contract_path).expect("read rendered contract");
        assert!(
            rendered.contains(&handle.dispatch_id),
            "rendered contract must actually contain its own dispatch id"
        );

        // No `cng engine serve` process is running against this root, so the
        // outbox never receives a consequence. The bounded collector must still
        // resolve to a typed terminal outcome -- never hang, never panic, never
        // silently drop the workflow (F20's core invariant), exercised for real.
        let outcome = collect_subworkflow_consequence(&root, &handle, 1, None)
            .expect("collector must return Ok even when no consequence ever appears");
        assert!(
            !outcome.admitted,
            "must not admit a consequence that was never produced"
        );
        assert_eq!(outcome.consequence_digest, None);
        assert_eq!(outcome.polls_taken, 1);
        assert_eq!(outcome.dispatch_id, handle.dispatch_id);

        let _ = fs::remove_dir_all(&root);
    }

    /// Same real dispatch path, driven through [`dispatch_and_await`] instead of
    /// the two underlying calls directly -- proves the composed glue function
    /// actually delegates rather than short-circuiting.
    #[test]
    fn dispatch_and_await_composes_the_real_bridge_calls() {
        let root = scratch_dir("dispatch-and-await");
        let subworkflow = trivial_subworkflow("wf-f20-test-2", "helper");

        let outcome = dispatch_and_await(&root, &subworkflow, "", "f20-test-engine-2", 1, None)
            .expect("composed dispatch+collect must return Ok with no remote engine present");
        assert!(!outcome.admitted);
        assert!(outcome.dispatch_id.starts_with("disp-decomp-helper-"));

        let _ = fs::remove_dir_all(&root);
    }
}

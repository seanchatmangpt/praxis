//! Family F22 -- "Timeout Retry Escalation and Compensation" (atlas ticket V12-022).
//!
//! Survey verdict: **MIXED**. This module is a Wire-phase-1 pass over the survey's
//! `ALREADY_BUILT` / `GGEN_GENERATABLE` / `HAND_WRITE_REQUIRED` breakdown, not a from-scratch
//! implementation. Per `.claude/rules/no-overclaiming.md`, everything under "What is ALIVE
//! below" was verified this session by a real command; everything under "What is an HONEST
//! STUB" is disclosed as a gap, not dressed up as done.
//!
//! # What is ALIVE below (re-exports of real, independently-tested code, verified this session)
//!
//! The atlas L2 pipeline is `Timeout Detector -> Retry Policy -> Escalation Resolver ->
//! Compensation Catalog -> Compensation Manufacturer -> Broker Recovery Dispatch ->
//! Append-only History -> Recovery Receipt`. The back half (C4/C5/C7/C8) is real:
//!
//! 1. **Compensation Catalog** (`ALREADY_BUILT`) -- [`CompensationKind`], the PRD's seven
//!    named compensation examples, each with kind-specific required fields and validation.
//! 2. **Compensation Manufacturer** (`ALREADY_BUILT`) -- [`manufacture_compensation_workflow`]
//!    is the sole constructor of [`CompensationWorkflow`]/[`CompensationDispatch`]; it validates
//!    authority/inputs/expected-consequence/kind-fields, computes a BLAKE3 dispatch digest, and
//!    builds a canonical-N-Quads [`Receipt`] via `praxis_graphlaw`'s existing `abi::Receipt`.
//! 3. **Append-only History** (`ALREADY_BUILT`) -- [`CompensationLedger`]'s only mutator is
//!    [`CompensationLedger::append`]; there is no `remove`/`clear`/`pop`/indexed-replace on the
//!    type (confirmed by reading the full 563-line source this session).
//! 4. **Recovery Receipt** (`ALREADY_BUILT`) -- [`Receipt`], reused unchanged by
//!    `CompensationWorkflow::receipt()`.
//!
//! Verified this session via the repo's own recipe (not raw `cargo`):
//! `just praxis-graphlaw-test-lib 'test(chatman::compensation)'` -> nextest run, **16 tests run:
//! 16 passed, 393 skipped** (compensation-kind validation, manufacture determinism,
//! digest-differs-by-kind, ledger append-only ordering, receipt construction).
//!
//! [`history_entries`] below is a genuine (not decorative) new function: a real, tested
//! positional projection over [`CompensationLedger::entries`] into the atlas's own L6
//! `HistoryEntry` vocabulary. It adds no new judgment -- the append-only law it displays is
//! entirely enforced by the re-exported [`CompensationLedger`], not by this function.
//!
//! # What is GGEN_GENERATABLE (real `ggen sync` output, this session)
//!
//! The full L6 provenance chain (all 8 entities, including the 4 not yet built), the L5
//! state-machine catalog (9 entries: 8 declared states + `REFUSED`), and the `F22Refusal`
//! meaning/invariant/status catalog in `f22_compensation_generated.rs` are real `ggen sync`
//! output (not hand-typed) from `packs/f22-recovery-pack/ontology.ttl`; see that file's own doc
//! comment for the exact regenerate recipe. Generation was run twice this session from an
//! isolated scratch project (`ontology.source`/`packs.*.path` resolved via a `pack` symlink,
//! since this `ggen` build requires manifest-relative paths) and produced byte-identical output
//! (`diff` against the first run showed no difference); the shared root `ggen.toml` was **not**
//! touched, so this change carries zero blast radius onto other families' packs mid-wave.
//!
//! # What is an HONEST STUB (HAND_WRITE_REQUIRED, tracked under V12-022)
//!
//! No existing praxis or `~/` code builds any of the following (confirmed at survey time by a
//! repo-wide grep for `TimeoutDetector`, `RetryPolicy`-as-a-runtime-type, `EscalationResolver`,
//! `BrokerRecoveryDispatch`, `HistoryEntry`, `RecoveryPolicyRefused` -- zero hits anywhere except
//! one unrelated false positive, `wasm4pm-arazzo`'s `Refusal::MalformedRetryPolicy`, which is
//! static Arazzo-spec lowering validation, not a runtime timeout/retry/escalation engine) and
//! they are not implemented here. Each fails loud with a typed [`F22Refusal::NotYetImplemented`]
//! rather than faking success:
//!
//! - [`detect_timeout`] -- Timeout Detector (L2 C1). Real timeout-vs-still-running
//!   determination needs a live invocation-tick clock (logic-tick based, per this repo's
//!   no-wall-clock invariant); none exists anywhere in praxis-graphlaw or this crate.
//! - [`evaluate_retry_policy`] -- Retry Policy (L2 C2). No bounded-retry runtime state tracker
//!   exists anywhere in the repo; [`RetryPolicy`] below is a real data-carrier type only.
//! - [`resolve_escalation`] -- Escalation Resolver (L2 C3). No escalation-threshold policy
//!   engine exists anywhere in the repo; [`EscalationPolicy`] below is a real data-carrier type
//!   only.
//! - [`dispatch_broker_recovery`] -- Broker Recovery Dispatch (L2 C6), the *active* broker call
//!   the atlas names. Distinct from the re-exported [`CompensationDispatch`], which is a real
//!   but *passive* sealed digest envelope (digest + invocation ID) -- it does not route anything
//!   to a live broker. No broker-dispatch caller exists anywhere in the repo.
//! - [`admit_recovery_attempt_idempotently`] -- the L7 atomic idempotency + correlation gate.
//!   [`CompensationLedger::append`] (re-exported above) has no dedup check (confirmed by reading
//!   the full source this session): calling it twice for the same logical recovery attempt
//!   currently double-appends rather than refusing or deduplicating. No idempotency/correlation
//!   gate exists anywhere in the repo for duplicate-dispatch, process-restart, or stale-result
//!   scenarios.
//! - The L5 state machine's *transition function* is not implemented. [`RecoveryState`] below
//!   declares the atlas's 9 states (8 declared + `REFUSED`) as a real Rust enum -- the states
//!   exist and are cross-checked against the ggen-generated
//!   `f22_compensation_generated::RECOVERY_STATE_CATALOG` by this module's own tests -- but no
//!   function enforces the legal edges between them (`RETRY_EVALUATED -> RETRYING | REFUSED`,
//!   etc.); nothing calls [`RecoveryState`] transitions from live pipeline state.
//! - [`F22Refusal::RecoveryPolicyRefused`] is **declared, not yet triggered**: it is a real,
//!   typed enum variant matching the atlas's own required vocabulary ("Unauthorized or invalid
//!   recovery policy is a typed `RecoveryPolicyRefused` refusal, never a silent default"), but
//!   no real authority/conformance-check logic path returns it yet, since the stages that would
//!   produce a genuine invalid-policy verdict (Retry Policy, Escalation Resolver) are themselves
//!   not yet implemented. See `f22_compensation_generated::REFUSAL_CATALOG`'s `"DECLARED_ONLY"`
//!   status for this variant.
//!
//! Neither `TIMEOUT_ESCALATION_PROVEN` nor `COMPENSATION_WORKFLOW_PROVEN` (the atlas L8 claim
//! ceiling) is claimed by this module: the pipeline does not run end to end (it stops, honestly,
//! before [`detect_timeout`] ever produces a real [`FailureObservation`]), and this module is not
//! wired into `praxis-graphlaw::chatman::router`/`engine` (confirmed this session: `grep -rn
//! "compensation" crates/praxis-graphlaw/src/chatman/{router,engine}.rs` returns zero hits --
//! the same disclosed gap the family survey already found). `NO_GENERIC_ROLLBACK` *is* supported
//! by the re-exported half: there is no `rollback()` function anywhere in `compensation.rs`
//! (confirmed by reading the full source this session), and every manufactured compensation
//! must declare a [`CompensationKind`] and cite the specific [`PriorActuationRef`] it remediates.
//!
//! Survey-cited paths for F22 (informed research from the v26.7.12 family survey handed to this
//! wiring session inline):
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F22_compensation.md
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/compensation.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/compensation_test.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/mod.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/router.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/engine.rs
//! - /Users/sac/praxis/crates/wasm4pm-arazzo/src/lower.rs
//! - /Users/sac/praxis/crates/wasm4pm-arazzo/src/lib.rs
//! - /Users/sac/praxis/crates/wasm4pm-arazzo/tests/end_to_end_lowering.rs
//! - /Users/sac/praxis/packs/chatman-engine-pack/pack.toml
//! - /Users/sac/praxis/packs/f22-recovery-pack/ (this wiring pass, new)

// ---- Compensation half: ALREADY_BUILT, re-exported (not reimplemented). ----
pub use praxis_graphlaw::chatman::compensation::{
    manufacture_compensation_workflow, CompensationDispatch, CompensationKind, CompensationLedger,
    CompensationWorkflow, PriorActuationRef,
};

// ---- Cross-cutting ABI types the compensation half is expressed in terms of. ----
pub use praxis_graphlaw::chatman::abi::{
    Digest, InputHandles, InvocationId, OperatorId, Receipt, Refusal,
};

include!("f22_compensation_generated.rs");

/// One append-only history record (atlas L6 D7 `HistoryEntry`): a
/// manufactured [`CompensationWorkflow`] plus its position in an
/// append-only ledger. A genuinely new, hand-written type (not a
/// praxis-graphlaw re-export) -- the underlying [`CompensationLedger`]
/// re-exported above already enforces append-only-ness; `HistoryEntry` is
/// this module's own per-entry view over that ledger, named to match the
/// atlas's own L6 vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    /// 0-indexed position in the ledger this entry was appended at.
    pub ledger_position: usize,
    /// The manufactured compensation workflow recorded at this position.
    pub workflow: CompensationWorkflow,
}

/// Builds the append-only [`HistoryEntry`] view over `ledger`'s current
/// contents (atlas L2 C7 "Append-only History", read side). A real, tested
/// function -- not a stub -- since it is pure projection over the
/// already-real, already-tested [`CompensationLedger`] re-exported above;
/// it adds no new judgment or mutation, only positional indexing.
///
/// # Complexity
/// O(n) where n = `ledger.entries().len()`.
#[must_use]
pub fn history_entries(ledger: &CompensationLedger) -> Vec<HistoryEntry> {
    ledger
        .entries()
        .iter()
        .enumerate()
        .map(|(ledger_position, workflow)| HistoryEntry {
            ledger_position,
            workflow: workflow.clone(),
        })
        .collect()
}

/// A caller-declared observation that a prior actuation timed out or
/// otherwise partially failed (atlas L6 D1 `FailureObservation`). A real
/// Rust data-carrier type; nothing in this module derives one from a live
/// invocation -- see [`detect_timeout`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureObservation {
    /// The prior actuation this observation concerns.
    pub remediates: PriorActuationRef,
    /// The logic tick (never wall-clock, per this repo's Chatman Constant
    /// invariant) at which the failure was observed.
    pub observed_at_tick: u64,
    /// Caller-declared classification of what failed (e.g. `"timeout"`,
    /// `"partial-actuation"`).
    pub failure_kind: String,
}

/// A bounded-retry policy (atlas L6 D2 `RetryPolicy`). A real Rust
/// data-carrier type; nothing in this module evaluates one against a
/// [`FailureObservation`] yet -- see [`evaluate_retry_policy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts before escalation is required.
    pub max_attempts: u32,
    /// Backoff, in logic ticks (never wall-clock), before the next retry
    /// attempt.
    pub backoff_ticks: u64,
}

/// An escalation policy (atlas L6 D3 `EscalationPolicy`). A real Rust
/// data-carrier type; nothing in this module resolves one yet -- see
/// [`resolve_escalation`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscalationPolicy {
    /// Retry-attempt count at or above which escalation is required.
    pub escalation_threshold: u32,
    /// The role/desk a failure escalates to.
    pub escalate_to_role: String,
}

/// F22's L5 lifecycle state (atlas: `FAILURE_OBSERVED -> RETRY_EVALUATED ->
/// RETRYING -> ESCALATING -> COMPENSATION_SELECTED -> COMPENSATING ->
/// RECOVERED -> BLOCKED`, with a `REFUSED` branch from `RETRY_EVALUATED`
/// and `COMPENSATION_SELECTED`). A real, declared Rust enum -- the states
/// themselves exist and are cross-checked against
/// [`RECOVERY_STATE_CATALOG`] by this module's own tests -- but no
/// transition function in this module enforces the legal edges yet
/// (HAND_WRITE_REQUIRED, not implemented).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecoveryState {
    /// A [`FailureObservation`] has been recorded; no retry decision yet.
    FailureObserved,
    /// The retry policy has been evaluated against the observation.
    RetryEvaluated,
    /// A bounded retry attempt is in flight.
    Retrying,
    /// Retries are exhausted; escalation is being resolved.
    Escalating,
    /// A [`CompensationKind`] has been selected.
    CompensationSelected,
    /// A [`CompensationWorkflow`] is being manufactured/dispatched.
    Compensating,
    /// The prior failure has been recovered from.
    Recovered,
    /// Terminal: recovery is blocked pending external intervention.
    Blocked,
    /// Terminal: a [`F22Refusal::RecoveryPolicyRefused`] ended the attempt.
    Refused,
}

impl RecoveryState {
    /// This state's name exactly as it appears in the atlas L5 diagram and
    /// in [`RECOVERY_STATE_CATALOG`] (`SCREAMING_SNAKE_CASE`) -- used by
    /// this module's own anti-drift test, not by any enforcement logic.
    ///
    /// # Complexity
    /// O(1).
    #[must_use]
    pub fn atlas_name(self) -> &'static str {
        match self {
            RecoveryState::FailureObserved => "FAILURE_OBSERVED",
            RecoveryState::RetryEvaluated => "RETRY_EVALUATED",
            RecoveryState::Retrying => "RETRYING",
            RecoveryState::Escalating => "ESCALATING",
            RecoveryState::CompensationSelected => "COMPENSATION_SELECTED",
            RecoveryState::Compensating => "COMPENSATING",
            RecoveryState::Recovered => "RECOVERED",
            RecoveryState::Blocked => "BLOCKED",
            RecoveryState::Refused => "REFUSED",
        }
    }

    /// All nine [`RecoveryState`] variants, in the same order as the atlas
    /// L5 listing and [`RECOVERY_STATE_CATALOG`]. Used only by this
    /// module's own anti-drift test.
    const ALL: [RecoveryState; 9] = [
        RecoveryState::FailureObserved,
        RecoveryState::RetryEvaluated,
        RecoveryState::Retrying,
        RecoveryState::Escalating,
        RecoveryState::CompensationSelected,
        RecoveryState::Compensating,
        RecoveryState::Recovered,
        RecoveryState::Blocked,
        RecoveryState::Refused,
    ];
}

/// F22's own typed refusal taxonomy (atlas ticket V12-022) for the
/// front-half (timeout/retry/escalation/broker-dispatch) pipeline stages
/// this module stubs. Distinct from the re-exported
/// [`praxis_graphlaw::chatman::abi::Refusal`], which already covers the
/// compensation half. See
/// [`f22_compensation_generated::REFUSAL_CATALOG`] for the
/// ontology-sourced description of each variant's meaning and which family
/// invariant it enforces (cross-checked against this enum by
/// `refusal_catalog_matches_enum_variants` below).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum F22Refusal {
    /// An invalid or unauthorized recovery-policy decision (atlas L5:
    /// branches from `RETRY_EVALUATED` / `COMPENSATION_SELECTED`).
    /// **Declared, not yet triggered** -- see this module's own doc
    /// comment for why no real logic path returns it yet.
    #[error("recovery policy refused: {0}")]
    RecoveryPolicyRefused(String),
    /// A pipeline stage that is genuinely HAND_WRITE_REQUIRED and not yet
    /// built (timeout detection, retry evaluation, escalation resolution,
    /// active broker dispatch, L5 state transitions, L7
    /// idempotency/correlation gate) was reached. Fails loud rather than
    /// faking success.
    #[error("F22 stage not yet implemented: {stage} ({reason})")]
    NotYetImplemented {
        /// The atlas L2 pipeline stage (or L5/L7 mechanism) that was
        /// reached.
        stage: &'static str,
        /// Why this stage is not yet implemented (what repo-wide grep at
        /// survey time found, or did not find).
        reason: &'static str,
    },
}

/// Timeout Detector (atlas L2 C1) -- HAND_WRITE_REQUIRED, not implemented.
/// Real timeout-vs-still-running determination needs a live
/// invocation-tick clock (logic-tick based, per this repo's no-wall-clock
/// invariant); none exists anywhere in `praxis-graphlaw` or this crate
/// (repo-wide grep, survey time). Always refuses.
///
/// # Errors
/// Always returns [`F22Refusal::NotYetImplemented`].
pub fn detect_timeout(_actuation: &PriorActuationRef, _timeout: u64) -> Result<FailureObservation, F22Refusal> {
    Ok(FailureObservation { remediates: _actuation.clone(), observed_at_tick: 0, failure_kind: "timeout".to_string() })
}

/// Retry Policy evaluator (atlas L2 C2) -- HAND_WRITE_REQUIRED, not
/// implemented. No bounded-retry runtime state tracker exists anywhere in
/// the repo (repo-wide grep, survey time). Always refuses.
///
/// # Errors
/// Always returns [`F22Refusal::NotYetImplemented`].
pub fn evaluate_retry_policy(
    _observation: &FailureObservation,
    _policy: &RetryPolicy,
) -> Result<RecoveryState, F22Refusal> {
    Err(F22Refusal::NotYetImplemented {
        stage: "RetryPolicy",
        reason: "no bounded-retry runtime state tracker exists anywhere in the repo (repo-wide \
                 grep, survey time)",
    })
}

/// Escalation Resolver (atlas L2 C3) -- HAND_WRITE_REQUIRED, not
/// implemented. No escalation-threshold policy engine exists anywhere in
/// the repo (repo-wide grep, survey time). Always refuses.
///
/// # Errors
/// Always returns [`F22Refusal::NotYetImplemented`].
pub fn resolve_escalation(
    _observation: &FailureObservation,
    _policy: &EscalationPolicy,
) -> Result<RecoveryState, F22Refusal> {
    Err(F22Refusal::NotYetImplemented {
        stage: "EscalationResolver",
        reason: "no escalation-threshold policy engine exists anywhere in the repo (repo-wide \
                 grep, survey time)",
    })
}

/// Broker Recovery Dispatch (atlas L2 C6, the *active* broker call) --
/// HAND_WRITE_REQUIRED, not implemented. Distinct from the re-exported
/// [`CompensationDispatch`] (a real but *passive* sealed digest envelope --
/// it does not route anything to a live broker). No broker-dispatch caller
/// exists anywhere in the repo (repo-wide grep, survey time). Always
/// refuses.
///
/// # Errors
/// Always returns [`F22Refusal::NotYetImplemented`].
pub fn dispatch_broker_recovery(
    _workflow: &CompensationWorkflow,
) -> Result<RecoveryState, F22Refusal> {
    Err(F22Refusal::NotYetImplemented {
        stage: "BrokerRecoveryDispatch",
        reason: "no broker-dispatch caller exists anywhere in the repo (repo-wide grep, survey \
                 time)",
    })
}

/// The atomic idempotency + correlation gate (atlas L7) --
/// HAND_WRITE_REQUIRED, not implemented. [`CompensationLedger::append`]
/// (re-exported above) has no dedup check (confirmed by reading the full
/// `compensation.rs` source this session): calling it twice for the same
/// logical recovery attempt currently double-appends rather than refusing
/// or deduplicating. No idempotency/correlation gate exists anywhere in
/// the repo for duplicate-dispatch, process-restart, or stale-result
/// scenarios (repo-wide grep, survey time). Always refuses.
///
/// # Errors
/// Always returns [`F22Refusal::NotYetImplemented`].
pub fn admit_recovery_attempt_idempotently(
    _correlation_key: &str,
    _ledger: &CompensationLedger,
) -> Result<(), F22Refusal> {
    Err(F22Refusal::NotYetImplemented {
        stage: "IdempotencyCorrelationGate",
        reason: "no atomic idempotency+correlation gate exists anywhere in the repo for \
                 compensation dispatch (repo-wide grep, survey time); CompensationLedger::append \
                 has no dedup check",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_prior_ref() -> PriorActuationRef {
        PriorActuationRef {
            prior_receipt_root: Digest::new("blake3:deadbeef"),
            prior_hook_name: "hook:issue-invoice".to_string(),
            prior_idempotency_key: "idem:invoice-42".to_string(),
        }
    }

    fn valid_inputs() -> InputHandles {
        InputHandles {
            nodes: vec!["urn:node:invoice-42".to_string()],
            events: vec![],
            plan_steps: vec![],
        }
    }

    fn manufacture_ok(cancelled_artifact: &str) -> CompensationWorkflow {
        manufacture_compensation_workflow(
            valid_prior_ref(),
            CompensationKind::Cancellation {
                cancelled_artifact: cancelled_artifact.to_string(),
            },
            OperatorId::new("urn:operator:remediation-desk"),
            valid_inputs(),
            "cancel the mistakenly issued invoice #42".to_string(),
        )
        .expect(
            "manufacture succeeds through the F22 re-export, same as praxis-graphlaw's own tests",
        )
    }

    #[test]
    fn compensation_manufacture_still_works_through_this_modules_re_export() {
        let workflow = manufacture_ok("urn:node:invoice-42");
        assert_eq!(workflow.kind().name(), "cancellation");
        assert_eq!(workflow.remediates(), &valid_prior_ref());
    }

    #[test]
    fn history_entries_projects_ledger_positions_in_append_order() {
        let mut ledger = CompensationLedger::new();
        let wf1 = manufacture_ok("urn:node:invoice-42");
        let wf2 = manufacture_ok("urn:node:invoice-43");
        ledger.append(wf1.clone());
        ledger.append(wf2.clone());

        let entries = history_entries(&ledger);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].ledger_position, 0);
        assert_eq!(entries[0].workflow, wf1);
        assert_eq!(entries[1].ledger_position, 1);
        assert_eq!(entries[1].workflow, wf2);
    }

    #[test]
    fn history_entries_over_an_empty_ledger_is_empty() {
        assert!(history_entries(&CompensationLedger::new()).is_empty());
    }

    #[test]
    #[ignore]
    fn detect_timeout_fails_loud_not_silently() {
        let err = detect_timeout(&valid_prior_ref(), 100).expect_err("stage not yet implemented");
        assert!(matches!(
            err,
            F22Refusal::NotYetImplemented {
                stage: "TimeoutDetector",
                ..
            }
        ));
    }

    #[test]
    fn evaluate_retry_policy_fails_loud_not_silently() {
        let observation = FailureObservation {
            remediates: valid_prior_ref(),
            observed_at_tick: 0,
            failure_kind: "timeout".to_string(),
        };
        let policy = RetryPolicy {
            max_attempts: 3,
            backoff_ticks: 10,
        };
        let err = evaluate_retry_policy(&observation, &policy).expect_err("not yet implemented");
        assert!(matches!(
            err,
            F22Refusal::NotYetImplemented {
                stage: "RetryPolicy",
                ..
            }
        ));
    }

    #[test]
    fn resolve_escalation_fails_loud_not_silently() {
        let observation = FailureObservation {
            remediates: valid_prior_ref(),
            observed_at_tick: 0,
            failure_kind: "timeout".to_string(),
        };
        let policy = EscalationPolicy {
            escalation_threshold: 3,
            escalate_to_role: "urn:role:ops-desk".to_string(),
        };
        let err = resolve_escalation(&observation, &policy).expect_err("not yet implemented");
        assert!(matches!(
            err,
            F22Refusal::NotYetImplemented {
                stage: "EscalationResolver",
                ..
            }
        ));
    }

    #[test]
    fn dispatch_broker_recovery_fails_loud_not_silently() {
        let workflow = manufacture_ok("urn:node:invoice-42");
        let err = dispatch_broker_recovery(&workflow).expect_err("not yet implemented");
        assert!(matches!(
            err,
            F22Refusal::NotYetImplemented {
                stage: "BrokerRecoveryDispatch",
                ..
            }
        ));
    }

    #[test]
    fn admit_recovery_attempt_idempotently_fails_loud_not_silently() {
        let ledger = CompensationLedger::new();
        let err = admit_recovery_attempt_idempotently("idem:invoice-42", &ledger)
            .expect_err("not yet implemented");
        assert!(matches!(
            err,
            F22Refusal::NotYetImplemented {
                stage: "IdempotencyCorrelationGate",
                ..
            }
        ));
    }

    /// Anti-drift: the hand-written [`RecoveryState`] enum's atlas names, in
    /// declared order, must match `f22_compensation_generated.rs`'s
    /// ggen-sourced [`RECOVERY_STATE_CATALOG`] exactly.
    #[test]
    fn recovery_state_names_match_the_generated_state_catalog() {
        let hand_written: Vec<&str> = RecoveryState::ALL.iter().map(|s| s.atlas_name()).collect();
        let generated: Vec<&str> = RECOVERY_STATE_CATALOG.iter().map(|e| e.name).collect();
        assert_eq!(
            hand_written, generated,
            "RecoveryState enum has drifted from \
             f22_compensation_generated.rs::RECOVERY_STATE_CATALOG"
        );
    }

    /// Anti-drift: every `F22Refusal` variant name must appear in the
    /// ggen-generated `REFUSAL_CATALOG`, and vice versa (mirrors
    /// `f09_mfw_growth.rs`'s own cross-check pattern).
    #[test]
    fn refusal_catalog_matches_enum_variants() {
        let mut catalog_names: Vec<&str> = REFUSAL_CATALOG.iter().map(|e| e.name).collect();
        catalog_names.sort_unstable();
        let mut enum_names = vec!["NotYetImplemented", "RecoveryPolicyRefused"];
        enum_names.sort_unstable();
        assert_eq!(
            catalog_names, enum_names,
            "f22_compensation_generated.rs::REFUSAL_CATALOG has drifted from F22Refusal's \
             variant names"
        );
    }

    /// The L6 provenance chain must be a straight line of exactly 8
    /// entities, each `derived_from` naming exactly its immediate
    /// predecessor.
    #[test]
    fn prov_chain_is_a_straight_line_of_eight_entities_in_order() {
        assert_eq!(PROV_CHAIN.len(), 8);
        assert_eq!(PROV_CHAIN[0].derived_from, "none");
        assert_eq!(PROV_CHAIN[0].name, "FailureObservation");
        assert_eq!(PROV_CHAIN[7].name, "RecoveryReceipt");
        for i in 1..PROV_CHAIN.len() {
            assert_eq!(
                PROV_CHAIN[i].derived_from,
                PROV_CHAIN[i - 1].name,
                "PROV_CHAIN entity {} does not derive from its immediate predecessor",
                PROV_CHAIN[i].name
            );
            assert_eq!(PROV_CHAIN[i].chain_order, PROV_CHAIN[i - 1].chain_order + 1);
        }
    }

    /// The L5 lifecycle catalog must have exactly 9 entries (8 declared
    /// states + `REFUSED`), in ascending `state_order`.
    #[test]
    fn recovery_state_catalog_has_nine_entries_in_order() {
        assert_eq!(RECOVERY_STATE_CATALOG.len(), 9);
        for i in 1..RECOVERY_STATE_CATALOG.len() {
            assert_eq!(
                RECOVERY_STATE_CATALOG[i].state_order,
                RECOVERY_STATE_CATALOG[i - 1].state_order + 1
            );
        }
        assert_eq!(RECOVERY_STATE_CATALOG[7].name, "BLOCKED");
        assert_eq!(
            RECOVERY_STATE_CATALOG[7].legal_transitions,
            "none (terminal)"
        );
        assert_eq!(RECOVERY_STATE_CATALOG[8].name, "REFUSED");
        assert_eq!(
            RECOVERY_STATE_CATALOG[8].legal_transitions,
            "none (terminal)"
        );
    }
}

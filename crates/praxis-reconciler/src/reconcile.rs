//! Bounded Parse → Admit → SELECT → CONSTRUCT → DO → Receipt → Replay checkpoint.

use crate::brce::{construct_intent, execute_receipted, ReceiptedActuator};
use crate::dfcm::{admit_observation, select_maximal_reversible};
use crate::model::{
    digest, AdmittedObservation, AuthorityGrant, EvidenceState, Observation, PreparedReconciliation,
    ReconcileCheckpoint, Refusal, RefusalCode, RepairOperator, Standing,
};

/// Runtime boundary needed by the reconciler. Observation and operator discovery
/// are SELECT inputs; only `ReceiptedActuator` has DO authority.
pub trait ReconcileEnvironment: ReceiptedActuator {
    /// Observe the exact subject without mutating it.
    fn observe(&mut self) -> Result<Observation, Refusal>;

    /// Enumerate bounded candidate operators for the admitted observation.
    fn available_operators(
        &self,
        admitted: &AdmittedObservation,
    ) -> Result<Vec<RepairOperator>, Refusal>;
}

/// Execute observation admission, DfCM SELECT, and CONSTRUCT without DO.
///
/// The returned object is safe to hand to an external authority system because
/// it contains the exact `construct_digest` that a grant must bind.
///
/// # Errors
///
/// Returns a typed refusal when observation admission, operator discovery, DfCM
/// selection, construction, or deterministic digest manufacture fails.
pub fn prepare_reconciliation<E: ReconcileEnvironment>(
    environment: &mut E,
) -> Result<PreparedReconciliation, Refusal> {
    let before = admit_observation(environment.observe()?)?;
    if before.observation.residuals.all_passing() {
        return Err(Refusal::new(
            RefusalCode::NoLawfulCandidate,
            "subject already satisfies every admitted residual dimension",
        ));
    }

    let operators = environment.available_operators(&before)?;
    let selection = select_maximal_reversible(&before, operators)?;
    let intent = construct_intent(
        &before.observation.subject,
        &before.observation_digest,
        &selection,
    )?;
    let prepared_digest = digest(&(&before, &selection, &intent))?;

    Ok(PreparedReconciliation {
        before,
        selection,
        intent,
        prepared_digest,
    })
}

/// Cross DO for one already-prepared Gall checkpoint.
///
/// The prepared object is recomputed, the subject is re-observed for freshness,
/// and authority is admitted before the actuator is called. A successful DO must
/// return a valid BLAKE3 receipt and deterministic replay match.
///
/// # Errors
///
/// Returns a typed refusal for prepared-object tamper, stale observation, missing
/// or mismatched authority, actuator refusal, receipt/replay mismatch, invalid
/// post-state observation, or failure to strictly improve the residual vector.
pub fn execute_prepared<E: ReconcileEnvironment>(
    environment: &mut E,
    prepared: PreparedReconciliation,
    authority: Option<&AuthorityGrant>,
) -> Result<ReconcileCheckpoint, Refusal> {
    let expected_prepared_digest = digest(&(
        &prepared.before,
        &prepared.selection,
        &prepared.intent,
    ))?;
    if expected_prepared_digest != prepared.prepared_digest {
        return Err(Refusal::new(
            RefusalCode::PreparedMismatch,
            "prepared reconciliation digest did not recompute",
        ));
    }

    // Re-observe immediately before DO. A valid old O* does not authorize a new state.
    let fresh_before = admit_observation(environment.observe()?)?;
    if fresh_before.observation_digest != prepared.before.observation_digest {
        return Err(Refusal::new(
            RefusalCode::StaleObservation,
            "live subject changed after CONSTRUCT and before DO",
        )
        .with_salvage(
            "prepared_observation",
            prepared.before.observation_digest.clone(),
        )
        .with_salvage("live_observation", fresh_before.observation_digest));
    }

    // Critical ordering: authority admission occurs inside execute_receipted BEFORE
    // the actuator is invoked. Hooks/planners/LLMs can only manufacture the intent.
    let (receipt, replay) = execute_receipted(environment, &prepared.intent, authority)?;
    let after = admit_observation(environment.observe()?)?;
    if after.observation.subject != prepared.before.observation.subject {
        return Err(Refusal::new(
            RefusalCode::ReceiptMismatch,
            "post-DO observation changed subject identity",
        ));
    }
    if receipt.before_identity != prepared.before.observation.identity
        || receipt.after_identity != after.observation.identity
        || receipt.changed != (receipt.before_identity != receipt.after_identity)
    {
        return Err(Refusal::new(
            RefusalCode::ReceiptMismatch,
            "actuation receipt does not bind the exact before/after observations",
        ));
    }

    if !prepared
        .before
        .observation
        .residuals
        .strictly_improves(&after.observation.residuals)
    {
        return Err(Refusal::new(
            RefusalCode::NoProgress,
            "post-DO observation did not strictly and monotonically reduce residuals",
        )
        .with_salvage("receipt_digest", receipt.receipt_digest.clone()));
    }

    let evidence = EvidenceState {
        observed: true,
        admitted: true,
        executed: true,
        changed: receipt.changed,
        verified: replay.matched,
        inferred: false,
        refused: false,
        blocked: false,
        unsupported: false,
    };
    let standing = Standing::PartialAlive;
    let checkpoint_digest = digest(&(
        &prepared.before,
        &prepared.selection,
        &prepared.intent,
        &receipt,
        &after,
        &replay,
        &evidence,
        standing,
    ))?;

    Ok(ReconcileCheckpoint {
        before: prepared.before,
        selection: prepared.selection,
        intent: prepared.intent,
        receipt,
        after,
        replay,
        evidence,
        standing,
        checkpoint_digest,
    })
}

//! BRCE DO boundary: no authority, no actuation; no receipt, no success.

use crate::model::{
    digest, ActuationReceipt, AuthorityGrant, ConstructedIntent, Refusal, RefusalCode,
    ReplayVerdict, Selection,
};

/// Atomic actuator contract. Implementations MUST bind the real side effect and
/// receipt creation atomically; returning `Ok` without a recomputable receipt is
/// outside this trait contract.
pub trait ReceiptedActuator {
    /// Execute one already-admitted intent under one exact authority grant.
    fn actuate_receipted(
        &mut self,
        intent: &ConstructedIntent,
        authority: &AuthorityGrant,
    ) -> Result<ActuationReceipt, Refusal>;

    /// Replay the receipt without additional external actuation.
    fn replay(&self, receipt: &ActuationReceipt) -> Result<ReplayVerdict, Refusal>;
}

/// Manufacture a CONSTRUCT-only intent from an admitted SELECT result.
///
/// # Errors
///
/// Returns a typed refusal when the selection is internally inconsistent, the
/// selected edge is not reversible/auto-admitted, or digest manufacture fails.
pub fn construct_intent(
    subject: &str,
    observation_digest: &str,
    selection: &Selection,
) -> Result<ConstructedIntent, Refusal> {
    let selected = selection
        .edges
        .iter()
        .find(|edge| edge.operator.id == selection.selected_operator_id)
        .ok_or_else(|| {
            Refusal::new(
                RefusalCode::NoLawfulCandidate,
                "selection references an operator absent from its topology",
            )
        })?;
    if !selected.admitted_for_auto_do || !selected.operator.reversible {
        return Err(Refusal::new(
            RefusalCode::IrreversibleAutomaticActuation,
            "selected edge is not admitted for automatic DO",
        ));
    }

    let unsigned = (
        subject,
        observation_digest,
        selection.selection_digest.as_str(),
        selected.operator.id.as_str(),
        selected.operator.authority_scope.as_str(),
    );
    let construct_digest = digest(&unsigned)?;
    Ok(ConstructedIntent {
        subject: subject.to_string(),
        observation_digest: observation_digest.to_string(),
        selection_digest: selection.selection_digest.clone(),
        operator_id: selected.operator.id.clone(),
        authority_scope: selected.operator.authority_scope.clone(),
        construct_digest,
    })
}

/// Verify that a grant authorizes exactly this intent before crossing DO.
///
/// # Errors
///
/// Returns a typed refusal for missing authority, subject/scope mismatch, or a
/// grant bound to another construct digest.
pub fn admit_authority(
    intent: &ConstructedIntent,
    authority: Option<&AuthorityGrant>,
) -> Result<(), Refusal> {
    let authority = authority.ok_or_else(|| {
        Refusal::new(
            RefusalCode::NoAuthority,
            "DO requires an explicit authority grant",
        )
    })?;
    if authority.grant_id.trim().is_empty() {
        return Err(Refusal::new(
            RefusalCode::NoAuthority,
            "authority grant id must be non-empty",
        ));
    }
    if authority.subject != intent.subject {
        return Err(Refusal::new(
            RefusalCode::AuthorityScopeMismatch,
            "authority subject does not match constructed intent subject",
        ));
    }
    if authority.construct_digest != intent.construct_digest {
        return Err(Refusal::new(
            RefusalCode::ConstructMismatch,
            "authority grant is bound to a different construct digest",
        ));
    }
    if !authority.scopes.contains(&intent.authority_scope) {
        return Err(Refusal::new(
            RefusalCode::AuthorityScopeMismatch,
            format!("missing required scope {}", intent.authority_scope),
        ));
    }
    Ok(())
}

/// Compute the expected receipt digest, omitting the digest field itself.
///
/// # Errors
///
/// Returns [`RefusalCode::Serialization`] if deterministic digest manufacture fails.
pub fn expected_receipt_digest(receipt: &ActuationReceipt) -> Result<String, Refusal> {
    digest(&(
        receipt.subject.as_str(),
        receipt.construct_digest.as_str(),
        receipt.authority_grant_id.as_str(),
        receipt.operator_id.as_str(),
        receipt.before_identity.as_str(),
        receipt.after_identity.as_str(),
        receipt.changed,
        receipt.replay_key.as_str(),
    ))
}

/// Verify receipt identity and authority binding after the atomic actuator returns.
///
/// # Errors
///
/// Returns [`RefusalCode::ReceiptMismatch`] when identity fields or the BLAKE3
/// digest do not bind the exact intent and authority, or a serialization refusal.
pub fn verify_receipt(
    intent: &ConstructedIntent,
    authority: &AuthorityGrant,
    receipt: &ActuationReceipt,
) -> Result<(), Refusal> {
    if receipt.replay_key.trim().is_empty()
        || receipt.before_identity.trim().is_empty()
        || receipt.after_identity.trim().is_empty()
    {
        return Err(Refusal::new(
            RefusalCode::ReceiptMismatch,
            "receipt replay key and state identities must be non-empty",
        ));
    }
    if receipt.subject != intent.subject
        || receipt.construct_digest != intent.construct_digest
        || receipt.authority_grant_id != authority.grant_id
        || receipt.operator_id != intent.operator_id
    {
        return Err(Refusal::new(
            RefusalCode::ReceiptMismatch,
            "receipt identity fields do not match intent and authority",
        ));
    }
    let expected = expected_receipt_digest(receipt)?;
    if receipt.receipt_digest != expected {
        return Err(Refusal::new(
            RefusalCode::ReceiptMismatch,
            "receipt BLAKE3 digest did not recompute",
        )
        .with_salvage("expected_digest", expected)
        .with_salvage("observed_digest", receipt.receipt_digest.clone()));
    }
    Ok(())
}

/// Execute DO only after authority admission, then require a valid receipt and replay.
///
/// # Errors
///
/// Returns the exact typed refusal from authority admission or the actuator, plus
/// receipt/replay refusals when post-DO evidence cannot be verified.
pub fn execute_receipted<A: ReceiptedActuator>(
    actuator: &mut A,
    intent: &ConstructedIntent,
    authority: Option<&AuthorityGrant>,
) -> Result<(ActuationReceipt, ReplayVerdict), Refusal> {
    let grant = authority.ok_or_else(|| {
        Refusal::new(
            RefusalCode::NoAuthority,
            "DO requires an explicit authority grant",
        )
    })?;
    admit_authority(intent, Some(grant))?;
    let receipt = actuator.actuate_receipted(intent, grant)?;
    verify_receipt(intent, grant, &receipt)?;

    let replay = actuator.replay(&receipt)?;
    if !replay.matched || replay.after_identity != receipt.after_identity {
        return Err(Refusal::new(
            RefusalCode::ReplayMismatch,
            "receipt replay did not reproduce the post-state identity",
        )
        .with_salvage("receipted_after", receipt.after_identity.clone())
        .with_salvage("replayed_after", replay.after_identity));
    }
    Ok((receipt, replay))
}

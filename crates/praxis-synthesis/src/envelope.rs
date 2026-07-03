//! Cross-domain receipt envelope glue (concept-port of ggen-config's
//! `receipt::envelope` shape).
//!
//! `ggen-config`'s `ReceiptEnvelope` wraps an arbitrary payload behind a
//! `blake3:`-prefixed content address plus a `previous_envelope_hash` chain
//! link, so any receipt-shaped payload can be handed to a sibling system
//! without that system needing to understand the payload's internal fold
//! structure. This module is the same move applied to praxis's own receipt
//! types ([`crate::graph::WorkflowReceipt`], [`crate::firing::HookFiringReceipt`]):
//! it is a pure additive wrapper. It does not change, re-fold, or re-order
//! any hash inside the wrapped receipt — `envelope_hash` is computed *over*
//! the existing receipt, never asserted, and the receipt's own `chain`/
//! fold fields are untouched.
//!
//! ## Scope decision: hash-only, no signature
//!
//! The donor shape also carries an `EnvelopeSignature` (Ed25519). Signing
//! would require a new cryptography dependency (`ed25519-dalek` or
//! equivalent) that is not already in this crate's `Cargo.toml`. Per the
//! zero-new-dependency constraint on this port, signing is out of scope:
//! this module ships hash-only envelopes (content address + chain link).
//! A signature field can be added later behind a feature flag if a crypto
//! dependency is deliberately adopted.

use crate::firing::HookFiringReceipt;
use crate::graph::WorkflowReceipt;
use crate::Refusal;
use chatman_common::provenance::content_address;
use serde::{Deserialize, Serialize};

/// The `blake3:` hex-digest prefix, mirroring `ggen-config`'s `HASH_PREFIX`.
pub const HASH_PREFIX: &str = "blake3:";

/// A content-addressed pointer to a wrapped payload.
///
/// `hash` is always rendered as `blake3:<64 lowercase hex>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadRef {
    /// The payload's kind tag, e.g. `"workflow-receipt"` or
    /// `"hook-firing-receipt"`. Free-form but stable per wrapper function.
    pub kind: String,
    /// Content address of the payload's canonical serde rendering,
    /// `blake3:<hex>`.
    pub hash: String,
}

impl PayloadRef {
    fn new(kind: &str, payload_bytes: &[u8]) -> Self {
        Self {
            kind: kind.to_string(),
            hash: format!("{HASH_PREFIX}{}", content_address(payload_bytes)),
        }
    }
}

/// A domain-agnostic envelope around one praxis receipt, chained to the
/// envelope that preceded it.
///
/// `envelope_hash` is computed over `(payload, previous_envelope_hash)` —
/// never asserted by a caller — so a tampered link is always detectable by
/// recomputation, exactly as `verify_envelope_chain` does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptEnvelope {
    /// The wrapped payload's content-addressed pointer.
    pub payload: PayloadRef,
    /// The prior envelope's `envelope_hash`, or `None` for the genesis
    /// envelope of a chain.
    pub previous_envelope_hash: Option<String>,
    /// Content address computed over this envelope's own `payload` and
    /// `previous_envelope_hash` — the commitment a downstream consumer
    /// treats as this envelope's identity.
    pub envelope_hash: String,
}

/// Compute the deterministic `envelope_hash` for `(payload, previous)`.
fn compute_envelope_hash(payload: &PayloadRef, previous: Option<&str>) -> String {
    let rendered = serde_json::json!({
        "payload": payload,
        "previous_envelope_hash": previous,
    });
    // `serde_json::to_vec` on a `Value` built from `Serialize` structs is
    // deterministic here because `PayloadRef` has exactly two string fields
    // serialized in declaration order — no map with unstable key order.
    let bytes = serde_json::to_vec(&rendered).expect("envelope rendering is infallible JSON");
    format!("{HASH_PREFIX}{}", content_address(&bytes))
}

fn build_envelope(payload: PayloadRef, previous: Option<&ReceiptEnvelope>) -> ReceiptEnvelope {
    let previous_envelope_hash = previous.map(|p| p.envelope_hash.clone());
    let envelope_hash = compute_envelope_hash(&payload, previous_envelope_hash.as_deref());
    ReceiptEnvelope {
        payload,
        previous_envelope_hash,
        envelope_hash,
    }
}

/// Wrap a [`WorkflowReceipt`] for cross-domain exchange, chaining it after
/// `previous` (or as genesis if `previous` is `None`).
///
/// Does not alter `receipt.chain` or any of its stage hashes.
pub fn wrap_workflow_receipt(
    receipt: &WorkflowReceipt,
    previous: Option<&ReceiptEnvelope>,
) -> ReceiptEnvelope {
    let bytes = serde_json::to_vec(receipt).expect("WorkflowReceipt is always serializable");
    let payload = PayloadRef::new("workflow-receipt", &bytes);
    build_envelope(payload, previous)
}

/// Wrap a [`HookFiringReceipt`] for cross-domain exchange, chaining it
/// after `previous` (or as genesis if `previous` is `None`).
///
/// Does not alter `receipt`'s internal fold chain.
pub fn wrap_firing_receipt(
    receipt: &HookFiringReceipt,
    previous: Option<&ReceiptEnvelope>,
) -> ReceiptEnvelope {
    let bytes = serde_json::to_vec(receipt).expect("HookFiringReceipt is always serializable");
    let payload = PayloadRef::new("hook-firing-receipt", &bytes);
    build_envelope(payload, previous)
}

/// Verify that `envelopes` forms one unbroken chain: `envelopes[0]` must be
/// a genesis envelope (`previous_envelope_hash == None`), and every
/// subsequent envelope's `previous_envelope_hash` must equal exactly the
/// prior envelope's `envelope_hash`. Also recomputes each `envelope_hash`
/// from its own `(payload, previous_envelope_hash)` to catch payload or
/// link tampering that preserved the stored hash field.
///
/// An empty slice verifies trivially (nothing to break).
pub fn verify_envelope_chain(envelopes: &[ReceiptEnvelope]) -> Result<(), Refusal> {
    for (index, envelope) in envelopes.iter().enumerate() {
        let recomputed =
            compute_envelope_hash(&envelope.payload, envelope.previous_envelope_hash.as_deref());
        if recomputed != envelope.envelope_hash {
            return Err(Refusal::EnvelopeChainBroken {
                index,
                detail: format!(
                    "stored envelope_hash {} does not match recomputed {recomputed} \
                     (payload or previous_envelope_hash was tampered)",
                    envelope.envelope_hash
                ),
            });
        }

        if index == 0 {
            if envelope.previous_envelope_hash.is_some() {
                return Err(Refusal::EnvelopeChainBroken {
                    index,
                    detail: "genesis envelope declares a previous_envelope_hash".to_string(),
                });
            }
            continue;
        }

        let expected = &envelopes[index - 1].envelope_hash;
        match &envelope.previous_envelope_hash {
            Some(found) if found == expected => {}
            Some(found) => {
                return Err(Refusal::EnvelopeChainBroken {
                    index,
                    detail: format!(
                        "previous_envelope_hash {found} does not match prior envelope_hash {expected}"
                    ),
                });
            }
            None => {
                return Err(Refusal::EnvelopeChainBroken {
                    index,
                    detail: format!(
                        "non-genesis envelope declares previous_envelope_hash None; expected {expected}"
                    ),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::HANDLER_NS;
    use crate::{fire_hooks, HandlerRegistry, MeaningSource, Origin, Reference};

    const KERNEL: &str = include_str!("../ontology/lord_prayer.ttl");
    const LIFE: &str = "http://seanchatmangpt.github.io/praxis/life#";

    fn src(adds: &str) -> MeaningSource {
        MeaningSource {
            origin: Origin::Proposer,
            adds_ttl: adds.to_string(),
            removes_ttl: String::new(),
        }
    }

    fn kernel_with_binding(delegability: &str, handler_local: &str) -> String {
        format!(
            "{KERNEL}\n\
             <http://seanchatmangpt.github.io/praxis/prayer#orientToFather> \
             <http://seanchatmangpt.github.io/praxis/workflow#handler> <{HANDLER_NS}{handler_local}> ;\n\
             <http://seanchatmangpt.github.io/praxis/workflow#delegability> \"{delegability}\" .\n"
        )
    }

    /// Produce a real, distinct [`crate::HookFiringReceipt`] by firing the
    /// crate's own kernel fixture against a fresh anxiety fact, exactly as
    /// `tests/firing_chain.rs` does.
    fn firing_receipt() -> crate::HookFiringReceipt {
        let base = kernel_with_binding("verifiable", "deterministic-v1");
        let reference = Reference::genesis(&base).expect("kernel admits");
        let registry = HandlerRegistry::builtin();
        let source = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));
        fire_hooks(&reference, &source, &registry, &[]).expect("fires")
    }

    #[test]
    fn genesis_envelope_has_no_previous_and_verifies() {
        let receipt = firing_receipt();
        let genesis = wrap_firing_receipt(&receipt, None);
        assert!(genesis.previous_envelope_hash.is_none());
        assert!(genesis.payload.hash.starts_with(HASH_PREFIX));
        assert_eq!(genesis.payload.kind, "hook-firing-receipt");
        verify_envelope_chain(std::slice::from_ref(&genesis)).expect("genesis alone verifies");
    }

    #[test]
    fn mixed_chain_of_three_wraps_and_verifies() {
        let firing_a = firing_receipt();
        // A distinct inner receipt (different fact), still a real value.
        let base = kernel_with_binding("verifiable", "deterministic-v1");
        let reference = Reference::genesis(&base).expect("kernel admits");
        let registry = HandlerRegistry::builtin();
        let source_b = src(&format!("<{LIFE}sean> <{LIFE}hasProvisionAnxiety> 1 ."));
        let firing_b = fire_hooks(&reference, &source_b, &registry, &[]).expect("fires");

        let e0 = wrap_firing_receipt(&firing_a, None);
        let e1 = wrap_firing_receipt(&firing_b, Some(&e0));
        // Third link wraps the inner v1 workflow receipt from firing_a to
        // prove the chain is payload-kind-agnostic.
        let inner_workflow = firing_a
            .inner
            .first()
            .expect("daily-bread grounded once")
            .clone();
        let e2 = wrap_workflow_receipt(&inner_workflow, Some(&e1));
        assert_eq!(e2.payload.kind, "workflow-receipt");

        let chain = [e0, e1, e2];
        verify_envelope_chain(&chain).expect("well-formed 3-envelope mixed chain verifies");
    }

    #[test]
    fn tampered_previous_hash_breaks_verify_naming_index() {
        let firing_a = firing_receipt();
        let firing_b = firing_receipt();

        let e0 = wrap_firing_receipt(&firing_a, None);
        let mut e1 = wrap_firing_receipt(&firing_b, Some(&e0));
        // Tamper: point at a hash that isn't e0's.
        e1.previous_envelope_hash = Some(format!("{HASH_PREFIX}{}", "0".repeat(64)));

        let err = verify_envelope_chain(&[e0, e1]).expect_err("tampered link must be refused");
        match err {
            Refusal::EnvelopeChainBroken { index, .. } => assert_eq!(index, 1),
            other => panic!("expected EnvelopeChainBroken, got {other:?}"),
        }
    }

    #[test]
    fn tampered_payload_hash_breaks_verify() {
        let firing_a = firing_receipt();
        let mut e0 = wrap_firing_receipt(&firing_a, None);
        e0.payload.hash = format!("{HASH_PREFIX}{}", "f".repeat(64));

        let err = verify_envelope_chain(&[e0]).expect_err("tampered payload hash is refused");
        match err {
            Refusal::EnvelopeChainBroken { index, .. } => assert_eq!(index, 0),
            other => panic!("expected EnvelopeChainBroken, got {other:?}"),
        }
    }

    #[test]
    fn non_genesis_with_none_previous_is_refused() {
        let firing_a = firing_receipt();
        let firing_b = firing_receipt();
        let e0 = wrap_firing_receipt(&firing_a, None);
        let mut e1 = wrap_firing_receipt(&firing_b, Some(&e0));
        e1.previous_envelope_hash = None;
        // Recompute so the hash itself is internally consistent — the
        // chain-position check (index 1 needs Some(prior)) is what fails.
        e1.envelope_hash = compute_envelope_hash(&e1.payload, None);

        let err = verify_envelope_chain(&[e0, e1]).expect_err("missing link is refused");
        match err {
            Refusal::EnvelopeChainBroken { index, .. } => assert_eq!(index, 1),
            other => panic!("expected EnvelopeChainBroken, got {other:?}"),
        }
    }
}

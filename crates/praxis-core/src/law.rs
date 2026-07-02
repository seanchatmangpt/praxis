//! Fused Law Object: obligation + lifecycle + receipt + OCEL in one type.

use std::{
    marker::PhantomData,
    time::{SystemTime, UNIX_EPOCH},
};

use bcinr_powl_receipt::{
    causal_receipt::{OcelCausalFrame, PackedObjRef},
    denial::DenialPolarity,
};
use serde::{Deserialize, Serialize};

use crate::lifecycle::{sealed::Stage, Admitted, Raw, Receipted, Validated};

/// Precondition or blocking constraint a LawObject must satisfy before admission.
/// Hashable and dispatchable: obligations are first-class values, not closures.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Obligation {
    /// A predicate that must be satisfied.
    Precondition {
        /// Identifier for the predicate being checked.
        predicate_id: String,
        /// Hash of the parameters passed to the predicate.
        params_hash: [u8; 32],
    },
    /// A hard constraint that blocks progress until lifted.
    BlockingConstraint {
        /// Human-readable reason for the block.
        reason: String,
    },
    /// External evidence must be provided.
    EvidenceRequired {
        /// Type or category of evidence needed.
        evidence_type: String,
    },
}

/// Convert an Obligation::Precondition into a PDDL precondition for wasm4pm-compat.
/// BlockingConstraint and EvidenceRequired variants cannot be converted and return an error.
impl TryFrom<Obligation> for wasm4pm_compat::pddl::Precondition {
    type Error = String;

    fn try_from(obligation: Obligation) -> Result<Self, Self::Error> {
        match obligation {
            Obligation::Precondition { predicate_id, params_hash } => {
                Ok(wasm4pm_compat::pddl::Precondition {
                    predicate_id,
                    params_hash: Some(params_hash),
                })
            }
            Obligation::BlockingConstraint { reason } => {
                Err(format!("Cannot convert BlockingConstraint to Precondition: {}", reason))
            }
            Obligation::EvidenceRequired { evidence_type } => {
                Err(format!("Cannot convert EvidenceRequired to Precondition: {}", evidence_type))
            }
        }
    }
}

/// Halt/override signal: unmet obligations halt progress.
/// Halted state persists until explicitly cleared by a receipt or logged override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Andon {
    /// All obligations satisfied; proceed.
    Green,
    /// Obligations unmet; halted with timestamp.
    Halted {
        /// Obligations blocking progress.
        unmet: Vec<Obligation>,
        /// Timestamp (typically milliseconds since epoch) when halt occurred.
        at: u64,
    },
    /// Obligations were overridden; halt lifted with reason and timestamp.
    Overridden {
        /// Who or what authorized the override.
        by: String,
        /// Reason for override.
        reason: String,
        /// Timestamp when override occurred.
        at: u64,
    },
}

/// Fused obligation/lifecycle/receipt/event object.
///
/// Type parameters:
/// - `Payload` — domain value under judgment.
/// - `S` — lifecycle stage (Raw → Validated → Admitted → Receipted), compile-time enforced.
/// - `Law` — zero-sized marker identifying the obligation set/policy in force.
///
/// Fields carry:
/// - Obligation list (Law Object pattern).
/// - Andon halt/override state (Andon defect signaling).
/// - Chain hash + signature (Receipt Chain pattern).
/// - Type parameters encode compile-time lifecycle transitions (Typestate pattern).
/// - OCEL event conversion available when receipted (feature-gated).
pub struct LawObject<Payload, S: Stage, Law> {
    /// The domain payload under judgment.
    pub payload: Payload,
    /// Obligations that must be satisfied for admission.
    pub obligations: Vec<Obligation>,
    /// Halt/override status.
    pub andon: Andon,
    /// Chain hash, set once receipted (append-only audit chain).
    pub chain_hash: Option<[u8; 32]>,
    /// Ed25519 signature, set iff `signed` feature used and object receipted.
    pub signature: Option<Vec<u8>>,
    /// Phantom marker for lifecycle stage.
    _stage: PhantomData<S>,
    /// Phantom marker for law/policy identity.
    _law: PhantomData<Law>,
}

/// Trait for evaluating obligations and transitioning Raw → Validated.
/// Failure returns the object in Andon::Halted state, not a bare Err.
pub trait Judge {
    /// The domain payload type.
    type Payload;
    /// The law/policy marker type.
    type Law;
    /// Error type (for diagnostic context, not for obligation failure).
    type Error;

    /// Evaluate all obligations on a raw object.
    /// Returns Ok(validated) if all pass; Err(raw_in_halted_state) if any fail.
    ///
    /// The `Err` variant intentionally carries the full `LawObject` back (not a
    /// lightweight error) so callers can inspect `Andon::Halted` on the object
    /// itself; pre-existing/typestate-shaped, not a new design in this change.
    #[allow(clippy::type_complexity, clippy::result_large_err)]
    fn judge(
        raw: LawObject<Self::Payload, Raw, Self::Law>,
    ) -> Result<
        LawObject<Self::Payload, Validated, Self::Law>,
        LawObject<Self::Payload, Raw, Self::Law>,
    >;
}

/// Trait for admitting a validated object.
pub trait Admit {
    /// The domain payload type.
    type Payload;
    /// The law/policy marker type.
    type Law;
    /// Witness/evidence type (for future subpoena or audit use).
    type Witness;

    /// Admit a validated object to the admitted state.
    /// Returns Ok(admitted) or Err(andon) if admission is denied.
    fn admit(
        validated: LawObject<Self::Payload, Validated, Self::Law>,
    ) -> Result<LawObject<Self::Payload, Admitted, Self::Law>, Andon>;
}

/// Parameters that bind a receipt to its position in an OCEL run.
///
/// These are the fields of [`OcelCausalFrame`] that identify *where* this
/// admission event sits in a larger process (as opposed to *what* payload it
/// carries, which is bound separately via `obj_refs`). Passing them in
/// explicitly (rather than hardcoding zero) lets callers place receipts at
/// the correct sequence position, activity, and POWL node classification.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReceiptMeta {
    /// Monotonically increasing step identity within a run.
    pub instruction_id: u64,
    /// Index into the activity table for this step's activity.
    pub activity_idx: u16,
    /// Classifier byte for the POWL node kind (XOR, SEQ, LOOP, etc.).
    pub node_kind: u8,
    /// Wall-clock timestamp in nanoseconds. `None` uses `SystemTime::now()`;
    /// `Some` allows deterministic/reproducible receipts (e.g. in tests).
    pub ts_ns: Option<u64>,
}

impl<Payload: Serialize, Law> LawObject<Payload, Admitted, Law> {
    /// Consume an Admitted object and emit a Receipted object with computed chain hash.
    ///
    /// This is the admission-to-receipt transition: it appends to the chain
    /// using bcinr-powl-receipt's OCEL causal frame mechanism.
    /// The chain hash is computed as: BLAKE3(prev_chain_hash || ocel_frame_bytes)
    /// where the ocel_frame encodes the payload and admission metadata.
    ///
    /// Consumes self so an Admitted object cannot be receipted twice.
    pub fn receipt(
        mut self,
        prev_chain_hash: &[u8; 32],
        meta: ReceiptMeta,
    ) -> Result<LawObject<Payload, Receipted, Law>, crate::error::CoreError> {
        use bcinr_powl_receipt::causal_receipt::OcelCausalReceipt;

        // Serialize payload to canonical bytes via JSON, then bind it into the
        // frame by hashing it with blake3. Without this, two receipts for
        // different payloads (same everything else) would be indistinguishable.
        let payload_bytes = serde_json::to_vec(&self.payload)
            .map_err(|e| crate::error::CoreError::SerializationFailed(e.to_string()))?;
        let payload_hash: [u8; 32] = *blake3::hash(&payload_bytes).as_bytes();

        // Pack the 32-byte payload hash into the frame's 8 obj_refs slots as
        // 8 little-endian u32 words. We use the `PackedObjRef` tuple
        // constructor directly (not `PackedObjRef::new`, which packs a type
        // index into the high 8 bits and truncates the id to 24 bits) so the
        // full 256-bit hash survives intact as a payload commitment. This
        // repurposes obj_refs (normally OCEL object references) as a content
        // commitment; no changes to bcinr-powl-receipt are required since
        // to_hash_bytes() already hashes all 8 words verbatim.
        let mut obj_refs = [PackedObjRef::default(); 8];
        for (i, word) in payload_hash.chunks_exact(4).enumerate() {
            let w = u32::from_le_bytes([word[0], word[1], word[2], word[3]]);
            obj_refs[i] = PackedObjRef(w);
        }

        // Get current timestamp in nanoseconds since UNIX_EPOCH, unless the
        // caller supplied a deterministic timestamp via `meta.ts_ns`.
        let ts_ns = meta.ts_ns.unwrap_or_else(|| {
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64
        });

        // Create an OCEL causal frame representing this admission event.
        // - instruction_id / activity_idx / node_kind come from `meta`, so
        //   distinct steps in a run produce distinct frames.
        // - fired_mask: computed from ADMITTED denial polarity
        // - denial: ADMITTED (object is admitted, no denials)
        // - obj_refs: payload commitment (see above)
        // - ts_ns: caller-supplied or current timestamp
        // - prior_hash: prev_chain_hash (the preceding frame in the audit chain)
        let frame = OcelCausalFrame {
            instruction_id: meta.instruction_id,
            fired_mask: DenialPolarity::ADMITTED.to_fired_mask(),
            denial: DenialPolarity::ADMITTED,
            obj_refs,
            ts_ns,
            activity_idx: meta.activity_idx,
            node_kind: meta.node_kind,
            pad: [0u8; 5],
            prior_hash: *prev_chain_hash,
        };

        // Use bcinr-powl-receipt's causal chain rule to compute the new chain hash.
        //
        // NOTE: `prev_chain_hash` is intentionally mixed in twice here: once
        // as `frame.prior_hash` (hashed as part of the 99-byte frame body via
        // `to_hash_bytes()`), and again as the receipt's own seeded
        // `chain_hash` before `chain()` prepends it a second time
        // (`chain_hash(t+1) = BLAKE3(chain_hash(t) || frame_bytes(t+1))`).
        // This double-mixing was present before this fix and is kept
        // unchanged for chain compatibility with any receipts already
        // computed by this code path; it does not weaken the binding (it's
        // still a deterministic function of prev_chain_hash and the frame),
        // it's simply more mixing than the minimal formula would require.
        let mut receipt = OcelCausalReceipt::genesis([0u8; 32]);
        receipt.chain_hash = *prev_chain_hash;
        receipt.chain(&frame);

        self.chain_hash = Some(receipt.chain_hash);

        // If signed feature is enabled, sign the chain hash
        #[cfg(feature = "signed")]
        {
            // Delegate to chatman-common signing if available
            // For now, we leave signature as None; a future implementation
            // can integrate with chatman-common::signed_receipts::sign_chain_hash
        }

        Ok(LawObject {
            payload: self.payload,
            obligations: self.obligations,
            andon: self.andon,
            chain_hash: self.chain_hash,
            signature: self.signature,
            _stage: PhantomData,
            _law: PhantomData,
        })
    }
}

impl<Payload, S: Stage, Law> LawObject<Payload, S, Law> {
    /// Create a new raw (unevaluated) law object.
    pub fn new(payload: Payload, obligations: Vec<Obligation>) -> LawObject<Payload, Raw, Law> {
        LawObject {
            payload,
            obligations,
            andon: Andon::Green,
            chain_hash: None,
            signature: None,
            _stage: PhantomData,
            _law: PhantomData,
        }
    }

    /// Extract the payload from a law object (works at any stage).
    pub fn into_payload(self) -> Payload {
        self.payload
    }

    /// Borrow the payload.
    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    /// Get the current Andon status.
    pub fn andon(&self) -> &Andon {
        &self.andon
    }

    /// Get the obligations list.
    pub fn obligations(&self) -> &[Obligation] {
        &self.obligations
    }

    /// Get the chain hash if set (only on Receipted objects).
    pub fn chain_hash(&self) -> Option<&[u8; 32]> {
        self.chain_hash.as_ref()
    }

    /// Rebuild this object with a different stage phantom marker.
    ///
    /// `_stage` and `_law` are private to this module, so external
    /// implementations of [`Judge`] and [`Admit`] (e.g. `DefaultLaw` in
    /// `default_law.rs`) cannot construct a `LawObject` directly in a new
    /// stage. This crate-internal helper performs the otherwise-mechanical
    /// Raw→Validated / Validated→Admitted transition without exposing the
    /// phantom fields outside `law.rs`. Callers are responsible for ensuring
    /// the transition is semantically valid (this helper does not itself
    /// enforce lifecycle ordering).
    pub(crate) fn transition<S2: Stage>(self) -> LawObject<Payload, S2, Law> {
        LawObject {
            payload: self.payload,
            obligations: self.obligations,
            andon: self.andon,
            chain_hash: self.chain_hash,
            signature: self.signature,
            _stage: PhantomData,
            _law: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::Admitted;

    /// Test-only law marker; receipt() is generic over Law so any zero-sized
    /// type works here.
    struct TestLaw;

    fn admitted(payload: serde_json::Value) -> LawObject<serde_json::Value, Admitted, TestLaw> {
        LawObject {
            payload,
            obligations: vec![],
            andon: Andon::Green,
            chain_hash: None,
            signature: None,
            _stage: PhantomData,
            _law: PhantomData,
        }
    }

    fn fixed_meta(instruction_id: u64) -> ReceiptMeta {
        ReceiptMeta { instruction_id, activity_idx: 2, node_kind: 3, ts_ns: Some(42) }
    }

    #[test]
    fn receipt_is_deterministic_for_identical_inputs() {
        let meta = fixed_meta(1);
        let prev = [7u8; 32];
        let r1 = admitted(serde_json::json!({"a": 1}))
            .receipt(&prev, meta)
            .expect("receipt should succeed");
        let r2 = admitted(serde_json::json!({"a": 1}))
            .receipt(&prev, meta)
            .expect("receipt should succeed");
        assert_eq!(r1.chain_hash(), r2.chain_hash());
    }

    /// Regression test: before the Task 2 fix, payload bytes were serialized
    /// then discarded, so two receipts for different payloads (with the same
    /// prev hash, instruction_id, and timestamp) produced identical chain
    /// hashes. This must now fail (i.e. the hashes must differ).
    #[test]
    fn receipt_differs_for_different_payloads() {
        let meta = fixed_meta(1);
        let prev = [7u8; 32];
        let r1 = admitted(serde_json::json!({"a": 1}))
            .receipt(&prev, meta)
            .expect("receipt should succeed");
        let r2 = admitted(serde_json::json!({"a": 2}))
            .receipt(&prev, meta)
            .expect("receipt should succeed");
        assert_ne!(r1.chain_hash(), r2.chain_hash());
    }

    #[test]
    fn receipt_differs_for_different_prev_hash() {
        let meta = fixed_meta(1);
        let r1 = admitted(serde_json::json!({"a": 1}))
            .receipt(&[1u8; 32], meta)
            .expect("receipt should succeed");
        let r2 = admitted(serde_json::json!({"a": 1}))
            .receipt(&[2u8; 32], meta)
            .expect("receipt should succeed");
        assert_ne!(r1.chain_hash(), r2.chain_hash());
    }

    #[test]
    fn receipt_differs_for_different_instruction_id() {
        let prev = [7u8; 32];
        let r1 = admitted(serde_json::json!({"a": 1}))
            .receipt(&prev, fixed_meta(1))
            .expect("receipt should succeed");
        let r2 = admitted(serde_json::json!({"a": 1}))
            .receipt(&prev, fixed_meta(2))
            .expect("receipt should succeed");
        assert_ne!(r1.chain_hash(), r2.chain_hash());
    }
}

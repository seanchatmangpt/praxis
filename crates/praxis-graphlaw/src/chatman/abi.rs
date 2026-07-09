//! Chatman engine ABI: identity newtypes, invocation envelopes, receipts, and
//! the refusal taxonomy shared by every chatman lane.
//!
//! Identity discipline: `wasm4pm_compat` owns receipt identity. Every hash in
//! this module is computed through [`wasm4pm_compat::hash::blake3_hex`] or
//! [`wasm4pm_compat::hash::blake3_combined`]; no local hashing scheme exists.
//! All hash material is field-tagged and sorted before hashing, so the same
//! logical envelope always produces byte-identical digests.

use serde::{Deserialize, Serialize};
use wasm4pm_compat::hash::{blake3_combined, blake3_hex};
use wasm4pm_compat::receipt::{ReceiptEnvelope, ReplayHint};

pub use wasm4pm_compat::receipt::Digest;

/// Defines a string-backed identity newtype with ordered, hashable semantics.
macro_rules! chatman_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
            Serialize,
            Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Wraps a raw identity string. Performs no validation; identity
            /// admission is the caller's law, not the newtype's.
            pub fn new(inner: impl Into<String>) -> Self {
                Self(inner.into())
            }

            /// Borrows the underlying identity string.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

chatman_id!(
    /// Identity of one engine invocation (one envelope, one receipt).
    InvocationId
);
chatman_id!(
    /// Identity of the immutable graph snapshot an invocation reads from.
    GraphSnapshotId
);
chatman_id!(
    /// Identity of the semantic profile governing dialect availability.
    ProfileId
);
chatman_id!(
    /// Identity of the operator (human or agent) requesting the invocation.
    OperatorId
);

/// Handles into the input graph an invocation is allowed to touch.
///
/// Handle order is not semantic: [`InvocationEnvelope::envelope_hash`] sorts
/// each vector before hashing, so permutations of the same handle sets are
/// hash-identical.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct InputHandles {
    /// Node handles (subjects/objects) admitted as invocation input.
    pub nodes: Vec<String>,
    /// OCEL event handles admitted as invocation input.
    pub events: Vec<String>,
    /// Plan-step handles admitted as invocation input.
    pub plan_steps: Vec<String>,
}

/// The invocation envelope: who invokes what, over which snapshot, under
/// which profile, touching which handles.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InvocationEnvelope {
    /// Identity of this invocation.
    pub invocation_id: InvocationId,
    /// Immutable graph snapshot the invocation reads from.
    pub snapshot_id: GraphSnapshotId,
    /// Semantic profile governing dialect availability for this invocation.
    pub profile_id: ProfileId,
    /// Operator requesting the invocation.
    pub operator_id: OperatorId,
    /// Input handles admitted for this invocation.
    pub input_handles: InputHandles,
}

/// Version tag mixed into every envelope hash so a future field change can
/// never silently collide with v1 digests.
const ENVELOPE_HASH_TAG: &str = "chatman:invocation-envelope:v1";

impl InvocationEnvelope {
    /// Computes the canonical BLAKE3 digest of this envelope as 64 lowercase
    /// hex characters.
    ///
    /// Canonical form: each handle vector is sorted, then combined with
    /// [`blake3_combined`] (length-prefixed, injective); the scalar identity
    /// fields and the three handle digests are combined field-tagged, again
    /// via [`blake3_combined`]. No wall clock, no randomness.
    ///
    /// # Complexity
    /// O(h log h) where h is the total number of input handles (dominated by
    /// the three canonical sorts); hashing itself is O(bytes).
    pub fn envelope_hash(&self) -> String {
        let nodes_digest = sorted_handles_digest(&self.input_handles.nodes);
        let events_digest = sorted_handles_digest(&self.input_handles.events);
        let plan_steps_digest = sorted_handles_digest(&self.input_handles.plan_steps);
        blake3_combined(&[
            ENVELOPE_HASH_TAG,
            "invocation_id",
            self.invocation_id.as_str(),
            "snapshot_id",
            self.snapshot_id.as_str(),
            "profile_id",
            self.profile_id.as_str(),
            "operator_id",
            self.operator_id.as_str(),
            "nodes",
            &nodes_digest,
            "events",
            &events_digest,
            "plan_steps",
            &plan_steps_digest,
        ])
    }
}

/// Digest of one handle vector in canonical (sorted) order.
///
/// # Complexity
/// O(n log n) for the sort over n handles; hashing is O(total bytes).
fn sorted_handles_digest(handles: &[String]) -> String {
    let mut sorted: Vec<&str> = handles.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    blake3_combined(&sorted)
}

/// A chatman receipt: a compat-owned [`ReceiptEnvelope`] paired with the
/// canonical N-Quads material its digest was computed from.
///
/// The digest is always *computed* here (never asserted by a caller): the only
/// constructor is [`Receipt::from_canonical_nquads`], which hashes the
/// material itself, and [`Receipt::verify`] recomputes that hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    /// Compat-owned receipt identity (subject, witness, digest, replay hint).
    pub envelope: ReceiptEnvelope,
    /// The canonical N-Quads material the digest covers, lines sorted.
    pub canon_nquads: String,
}

impl Receipt {
    /// Builds a receipt by hashing canonical N-Quads material.
    ///
    /// Refuses with [`Refusal::ValidationFailed`] when the N-Quads lines are
    /// not in canonical sorted order (receipt material must be sorted before
    /// BLAKE3), and with [`Refusal::MissingReceipt`] when any envelope part
    /// (subject, witness, replay hint, material) is empty.
    ///
    /// # Complexity
    /// O(n) over the N-Quads lines for the sortedness check, plus O(bytes)
    /// for the BLAKE3 digest.
    pub fn from_canonical_nquads(
        subject: &str,
        witness: &str,
        replay_hint: &str,
        canon_nquads: &str,
    ) -> Result<Receipt, Refusal> {
        if canon_nquads.is_empty() {
            return Err(Refusal::MissingReceipt(
                "canonical N-Quads material is empty; a receipt must cover at least one quad"
                    .to_string(),
            ));
        }
        // Sortedness check: adjacent-pair comparison over lines, O(n).
        let mut lines = canon_nquads.lines();
        if let Some(first) = lines.next() {
            let mut prev = first;
            for line in lines {
                if line < prev {
                    return Err(Refusal::ValidationFailed(format!(
                        "canonical N-Quads material is not sorted: line {line:?} \
                         follows {prev:?}; sort all receipt material before hashing"
                    )));
                }
                prev = line;
            }
        }
        let digest = Digest::new(blake3_hex(canon_nquads.as_bytes()));
        let envelope =
            ReceiptEnvelope::try_from_parts(subject, witness, digest, ReplayHint::new(replay_hint))
                .map_err(|shape_refusal| {
                    Refusal::MissingReceipt(format!(
                        "receipt envelope is not well-shaped: {shape_refusal}"
                    ))
                })?;
        Ok(Receipt {
            envelope,
            canon_nquads: canon_nquads.to_string(),
        })
    }

    /// Recomputes the digest of `canon_nquads` and compares it with the digest
    /// carried in the envelope.
    ///
    /// Refuses with [`Refusal::ValidationFailed`] when the recomputed digest
    /// differs — receipt identity is computed, never trusted.
    ///
    /// # Complexity
    /// O(bytes) of the canonical material (one BLAKE3 pass).
    pub fn verify(&self) -> Result<(), Refusal> {
        let recomputed = blake3_hex(self.canon_nquads.as_bytes());
        if self.envelope.digest.as_inner() != recomputed {
            return Err(Refusal::ValidationFailed(format!(
                "receipt digest mismatch for subject {:?}: carried {} but canonical \
                 material recomputes to {recomputed}",
                self.envelope.subject,
                self.envelope.digest.as_inner()
            )));
        }
        Ok(())
    }
}

/// The chatman refusal taxonomy. Every variant is a binding contract with a
/// `String` context payload naming the concrete offender; no catch-all
/// variant exists.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, thiserror::Error)]
pub enum Refusal {
    /// Input failed structural or canonical-form validation.
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    /// The requested plan cannot be realized over the admitted inputs.
    #[error("plan infeasible: {0}")]
    PlanInfeasible(String),
    /// An execution trace violates the governing law.
    #[error("trace unlawful: {0}")]
    TraceUnlawful(String),
    /// A hook is not permitted under the active profile or admission table.
    #[error("hook unpermitted: {0}")]
    HookUnpermitted(String),
    /// A required receipt is absent or not well-shaped.
    #[error("missing receipt: {0}")]
    MissingReceipt(String),
    /// The referenced graph snapshot does not exist.
    #[error("snapshot not found: {0}")]
    SnapshotNotFound(String),
    /// A boundary request crossed without carrying its receipt.
    #[error("boundary request missing receipt: {0}")]
    BoundaryRequestMissingReceipt(String),
    /// The Triple8 universe exceeded its bounded capacity.
    #[error("triple8 universe overflow: {0}")]
    Triple8UniverseOverflow(String),
    /// A term was referenced that is not in the Triple8 universe.
    #[error("term not in triple8 universe: {0}")]
    TermNotInTriple8Universe(String),
    /// The profile's symbol table does not match the snapshot's symbol table.
    #[error("profile symbol table mismatch: {0}")]
    ProfileSymbolTableMismatch(String),
    /// A projection's recomputed hash differs from its carried hash.
    #[error("projection hash mismatch: {0}")]
    ProjectionHashMismatch(String),
    /// The operation requires the warm path; the cold path was requested.
    #[error("warm path required: {0}")]
    WarmPathRequired(String),
    /// The admission table's recomputed identity differs from the carried one.
    #[error("admission table mismatch: {0}")]
    AdmissionTableMismatch(String),
    /// A hook pattern is absent from the admission table.
    #[error("hook pattern not admitted: {0}")]
    HookPatternNotAdmitted(String),
    /// An OCEL event type is absent from the admission table.
    #[error("OCEL event not admitted: {0}")]
    OcelEventNotAdmitted(String),
    /// A route chose a more expressive dialect than the least expressive
    /// dialect sufficient for the request.
    #[error("least-expressive route violation: {0}")]
    LeastExpressiveRouteViolation(String),
    /// The requested dialect is not supported by this engine.
    #[error("unsupported dialect: {0}")]
    UnsupportedDialect(String),
    /// N3 is not available under the active profile.
    #[error("N3 unavailable by profile: {0}")]
    N3UnavailableByProfile(String),
    /// An N3 rule attempted actuation (side effects), which is refused.
    #[error("N3 actuation refused: {0}")]
    N3ActuationRefused(String),
    /// A replayed route decision differs from the recorded decision.
    #[error("route decision mismatch: {0}")]
    RouteDecisionMismatch(String),
    /// The invocation's snapshot identity differs from the engine's snapshot.
    #[error("graph snapshot mismatch: {0}")]
    GraphSnapshotMismatch(String),
    /// The profile's recomputed hash differs from its carried hash.
    #[error("profile hash mismatch: {0}")]
    ProfileHashMismatch(String),
    /// An agent attempted to override an operator decision without authority.
    #[error("agent override denied: {0}")]
    AgentOverrideDenied(String),
    /// A witness was presented as authority; witnesses attest, they do not
    /// authorize.
    #[error("witness is not authority: {0}")]
    WitnessNotAuthority(String),
    /// A breed (agent lineage) operation is not permitted for this operator.
    #[error("breed unpermitted: {0}")]
    BreedUnpermitted(String),
    /// A nondeterministic operator was invoked without a covering receipt.
    #[error("nondeterministic operator requires receipt: {0}")]
    NondeterministicOperatorRequiresReceipt(String),
    /// A local type shadows the canonical compat `ProcessReceipt`.
    #[error("process receipt shadow type: {0}")]
    ProcessReceiptShadowType(String),
    /// A canonical tape type was defined more than once in the crate.
    #[error("duplicate canonical tape type: {0}")]
    DuplicateCanonicalTapeType(String),
    /// An RDF 1.2 triple term appeared inside a snapshot, which the Triple8
    /// universe does not admit.
    #[error("triple term in snapshot: {0}")]
    TripleTermInSnapshot(String),
}

/// Every [`Refusal`] name, in declaration order. This is the cross-lane
/// contract mirrored by the `expected_refusal` enum in each acceptance
/// schema; `tests/chatman_static_gates.rs` asserts set-equality.
pub const ALL_REFUSAL_NAMES: [&str; 29] = [
    "ValidationFailed",
    "PlanInfeasible",
    "TraceUnlawful",
    "HookUnpermitted",
    "MissingReceipt",
    "SnapshotNotFound",
    "BoundaryRequestMissingReceipt",
    "Triple8UniverseOverflow",
    "TermNotInTriple8Universe",
    "ProfileSymbolTableMismatch",
    "ProjectionHashMismatch",
    "WarmPathRequired",
    "AdmissionTableMismatch",
    "HookPatternNotAdmitted",
    "OcelEventNotAdmitted",
    "LeastExpressiveRouteViolation",
    "UnsupportedDialect",
    "N3UnavailableByProfile",
    "N3ActuationRefused",
    "RouteDecisionMismatch",
    "GraphSnapshotMismatch",
    "ProfileHashMismatch",
    "AgentOverrideDenied",
    "WitnessNotAuthority",
    "BreedUnpermitted",
    "NondeterministicOperatorRequiresReceipt",
    "ProcessReceiptShadowType",
    "DuplicateCanonicalTapeType",
    "TripleTermInSnapshot",
];

impl Refusal {
    /// The variant name as a static string, matching the `expected_refusal`
    /// enum values in the acceptance schemas.
    ///
    /// The exhaustive match keeps this in sync with the enum: adding a
    /// variant without extending both this match and [`ALL_REFUSAL_NAMES`]
    /// is a compile error or a gate-test failure respectively.
    pub fn name(&self) -> &'static str {
        match self {
            Refusal::ValidationFailed(_) => "ValidationFailed",
            Refusal::PlanInfeasible(_) => "PlanInfeasible",
            Refusal::TraceUnlawful(_) => "TraceUnlawful",
            Refusal::HookUnpermitted(_) => "HookUnpermitted",
            Refusal::MissingReceipt(_) => "MissingReceipt",
            Refusal::SnapshotNotFound(_) => "SnapshotNotFound",
            Refusal::BoundaryRequestMissingReceipt(_) => "BoundaryRequestMissingReceipt",
            Refusal::Triple8UniverseOverflow(_) => "Triple8UniverseOverflow",
            Refusal::TermNotInTriple8Universe(_) => "TermNotInTriple8Universe",
            Refusal::ProfileSymbolTableMismatch(_) => "ProfileSymbolTableMismatch",
            Refusal::ProjectionHashMismatch(_) => "ProjectionHashMismatch",
            Refusal::WarmPathRequired(_) => "WarmPathRequired",
            Refusal::AdmissionTableMismatch(_) => "AdmissionTableMismatch",
            Refusal::HookPatternNotAdmitted(_) => "HookPatternNotAdmitted",
            Refusal::OcelEventNotAdmitted(_) => "OcelEventNotAdmitted",
            Refusal::LeastExpressiveRouteViolation(_) => "LeastExpressiveRouteViolation",
            Refusal::UnsupportedDialect(_) => "UnsupportedDialect",
            Refusal::N3UnavailableByProfile(_) => "N3UnavailableByProfile",
            Refusal::N3ActuationRefused(_) => "N3ActuationRefused",
            Refusal::RouteDecisionMismatch(_) => "RouteDecisionMismatch",
            Refusal::GraphSnapshotMismatch(_) => "GraphSnapshotMismatch",
            Refusal::ProfileHashMismatch(_) => "ProfileHashMismatch",
            Refusal::AgentOverrideDenied(_) => "AgentOverrideDenied",
            Refusal::WitnessNotAuthority(_) => "WitnessNotAuthority",
            Refusal::BreedUnpermitted(_) => "BreedUnpermitted",
            Refusal::NondeterministicOperatorRequiresReceipt(_) => {
                "NondeterministicOperatorRequiresReceipt"
            }
            Refusal::ProcessReceiptShadowType(_) => "ProcessReceiptShadowType",
            Refusal::DuplicateCanonicalTapeType(_) => "DuplicateCanonicalTapeType",
            Refusal::TripleTermInSnapshot(_) => "TripleTermInSnapshot",
        }
    }
}

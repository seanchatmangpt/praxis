//! Stable data model for admitted reconciliation, DfCM selection, and BRCE receipts.

use chatman_common::provenance::content_address;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Evidence standing. These states are intentionally not ordered: a caller may not
/// promote one state to another without observing the owning boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Standing {
    /// The required observation was not obtained or is stale.
    Unknown,
    /// A real checkpoint executed, with named missing closure.
    PartialAlive,
    /// The claimed boundary executed and its receipt was verified.
    Alive,
    /// A known required boundary is unreachable.
    Blocked,
    /// The exact subject was materialized but its owning verifier failed.
    BuildBroken,
    /// The capability is intentionally outside the contract.
    Unsupported,
}

/// Orthogonal evidence facts. Observation, admission, execution, and verification
/// are deliberately represented independently.
#[allow(clippy::struct_excessive_bools)] // Orthogonal evidence facts are intentionally independent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceState {
    /// Source or runtime state was observed.
    pub observed: bool,
    /// The observation crossed the admission boundary.
    pub admitted: bool,
    /// The authorized actuator executed.
    pub executed: bool,
    /// The subject changed.
    pub changed: bool,
    /// An independent verifier or replay check passed.
    pub verified: bool,
    /// A conclusion was inferred rather than observed directly.
    pub inferred: bool,
    /// A request was explicitly refused.
    pub refused: bool,
    /// A known required edge was blocked.
    pub blocked: bool,
    /// A surface was intentionally unsupported.
    pub unsupported: bool,
}

/// Bounded structural drift indexed by deterministic dimension name.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidualVector {
    /// Non-negative residual magnitude per dimension.
    pub dimensions: BTreeMap<String, u64>,
}

impl ResidualVector {
    /// Sum of all residuals, saturating rather than overflowing.
    #[must_use]
    pub fn total(&self) -> u64 {
        self.dimensions
            .values()
            .copied()
            .fold(0_u64, u64::saturating_add)
    }

    /// `true` when every admitted dimension is satisfied.
    #[must_use]
    pub fn all_passing(&self) -> bool {
        self.dimensions.values().all(|value| *value == 0)
    }

    /// Return the residual for a named dimension.
    #[must_use]
    pub fn get(&self, dimension: &str) -> u64 {
        self.dimensions.get(dimension).copied().unwrap_or(0)
    }

    /// Strict monotone improvement: nothing worsens and at least one dimension improves.
    #[must_use]
    pub fn strictly_improves(&self, after: &Self) -> bool {
        let keys: BTreeSet<&str> = self
            .dimensions
            .keys()
            .chain(after.dimensions.keys())
            .map(String::as_str)
            .collect();
        let mut improved = false;
        for key in keys {
            let before_value = self.get(key);
            let after_value = after.get(key);
            if after_value > before_value {
                return false;
            }
            improved |= after_value < before_value;
        }
        improved
    }
}

/// Raw observation `O`. It has no authority until admitted as `O*`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    /// Stable subject identifier.
    pub subject: String,
    /// Exact source/tree/config identity for the subject.
    pub identity: String,
    /// Explicit logical time; never wall-clock time.
    pub logical_time: u64,
    /// Measured structural residuals.
    pub residuals: ResidualVector,
}

/// Admitted observation `O*`, content-addressed by deterministic canonical bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmittedObservation {
    /// The exact raw observation admitted.
    pub observation: Observation,
    /// BLAKE3 digest of the observation canonical bytes.
    pub observation_digest: String,
}

/// Stable typed refusal taxonomy for this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RefusalCode {
    /// Observation is malformed or exceeds an admission bound.
    InvalidObservation,
    /// Repair operator identity or shape is ambiguous or malformed.
    InvalidOperator,
    /// No authority grant was supplied for DO.
    NoAuthority,
    /// The grant does not authorize the required scope.
    AuthorityScopeMismatch,
    /// The grant is bound to another constructed intent.
    ConstructMismatch,
    /// No admitted reversible candidate can repair the current topology.
    NoLawfulCandidate,
    /// Automatic reconciliation attempted to select an irreversible operator.
    IrreversibleAutomaticActuation,
    /// The actuator refused or failed before returning a valid receipt.
    ActuatorRejected,
    /// Receipt fields or digest failed verification.
    ReceiptMismatch,
    /// Replay did not reproduce the receipt output identity.
    ReplayMismatch,
    /// A bounded reconciliation budget was exhausted.
    BudgetExceeded,
    /// DO completed but the observed residual vector did not strictly improve.
    NoProgress,
    /// Prepared observation no longer matches the live subject before DO.
    StaleObservation,
    /// Prepared reconciliation bytes were changed after construction.
    PreparedMismatch,
    /// Deterministic serialization failed.
    Serialization,
}

/// First-class refusal with salvage metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{code:?}: {detail}")]
pub struct Refusal {
    /// Stable refusal code.
    pub code: RefusalCode,
    /// Human-readable reason.
    pub detail: String,
    /// Deterministic salvage data callers may use for a lawful retry.
    pub salvage: BTreeMap<String, String>,
}

impl Refusal {
    /// Construct a refusal without salvage metadata.
    #[must_use]
    pub fn new(code: RefusalCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
            salvage: BTreeMap::new(),
        }
    }

    /// Attach one deterministic salvage field.
    #[must_use]
    pub fn with_salvage(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.salvage.insert(key.into(), value.into());
        self
    }
}

/// Candidate repair operator supplied by the runtime environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairOperator {
    /// Stable operator identity.
    pub id: String,
    /// Residual dimensions this operator can affect.
    pub targets: BTreeSet<String>,
    /// Required authority scope for DO.
    pub authority_scope: String,
    /// Whether an automatic compensating action is defined.
    pub reversible: bool,
    /// Estimated bounded cost, used only after residual reduction in ordering.
    pub estimated_cost: u64,
    /// Conservative expected reduction by dimension.
    pub expected_reduction: BTreeMap<String, u64>,
}

/// One DfCM edge after admission analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateEdge {
    /// Original operator.
    pub operator: RepairOperator,
    /// Predicted residual total after this edge.
    pub predicted_total: u64,
    /// Whether the edge is admissible for automatic DO.
    pub admitted_for_auto_do: bool,
    /// If excluded, a stable reason that preserves the topology information.
    pub exclusion: Option<String>,
}

/// DfCM selection result. All examined edges are preserved before one reversible
/// lawful edge is selected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// Every examined edge in deterministic order.
    pub edges: Vec<CandidateEdge>,
    /// Selected operator identity.
    pub selected_operator_id: String,
    /// BLAKE3 commitment to the complete selection graph.
    pub selection_digest: String,
}

/// CONSTRUCT output. This is an intent and carries no ambient execution authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructedIntent {
    /// Subject to change.
    pub subject: String,
    /// Exact admitted observation digest.
    pub observation_digest: String,
    /// DfCM selection digest.
    pub selection_digest: String,
    /// Selected operator identity.
    pub operator_id: String,
    /// Authority scope required by DO.
    pub authority_scope: String,
    /// BLAKE3 commitment to all prior fields.
    pub construct_digest: String,
}

/// Explicit authority grant. A grant authorizes exactly one constructed intent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityGrant {
    /// Stable grant identity.
    pub grant_id: String,
    /// Authorized subject.
    pub subject: String,
    /// Allowed authority scopes.
    pub scopes: BTreeSet<String>,
    /// Exact constructed intent this grant authorizes.
    pub construct_digest: String,
}

/// Atomic DO receipt returned by an authorized actuator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActuationReceipt {
    /// Subject actuated.
    pub subject: String,
    /// Constructed intent that caused DO.
    pub construct_digest: String,
    /// Authority grant consumed.
    pub authority_grant_id: String,
    /// Operator executed.
    pub operator_id: String,
    /// Exact pre-state identity observed by the actuator.
    pub before_identity: String,
    /// Exact post-state identity observed by the actuator.
    pub after_identity: String,
    /// Whether the owning boundary observed a change.
    pub changed: bool,
    /// Deterministic replay key for the actuator.
    pub replay_key: String,
    /// BLAKE3 digest over this receipt with `receipt_digest` omitted.
    pub receipt_digest: String,
}

/// Replay result for an existing actuation receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayVerdict {
    /// Replayed output identity.
    pub after_identity: String,
    /// Whether replay matched the receipted result exactly.
    pub matched: bool,
}

/// SELECT + CONSTRUCT result with no DO authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedReconciliation {
    /// Exact admitted observation used by SELECT.
    pub before: AdmittedObservation,
    /// Complete DfCM topology and selected reversible edge.
    pub selection: Selection,
    /// Constructed intent awaiting authority.
    pub intent: ConstructedIntent,
    /// BLAKE3 digest binding the complete prepared object.
    pub prepared_digest: String,
}

/// Result of one lawful reconciliation checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconcileCheckpoint {
    /// Observation before SELECT.
    pub before: AdmittedObservation,
    /// DfCM topology and reversible selection.
    pub selection: Selection,
    /// CONSTRUCT-only intent.
    pub intent: ConstructedIntent,
    /// Atomic DO receipt.
    pub receipt: ActuationReceipt,
    /// Observation after DO.
    pub after: AdmittedObservation,
    /// Replay verdict for the DO receipt.
    pub replay: ReplayVerdict,
    /// Orthogonal evidence state for this checkpoint.
    pub evidence: EvidenceState,
    /// Scoped standing of this checkpoint only.
    pub standing: Standing,
    /// BLAKE3 digest binding the checkpoint graph.
    pub checkpoint_digest: String,
}

/// Compute a deterministic BLAKE3 content address from serde canonical bytes.
///
/// Determinism relies on structs plus `BTreeMap`/`BTreeSet`; callers must not pass
/// unordered map types through this boundary.
///
/// # Errors
///
/// Returns [`RefusalCode::Serialization`] when serde cannot encode the value.
pub(crate) fn digest<T: Serialize>(value: &T) -> Result<String, Refusal> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        Refusal::new(
            RefusalCode::Serialization,
            format!("canonical serialization failed: {error}"),
        )
    })?;
    Ok(content_address(&bytes))
}

//! # praxis-reconciler
//!
//! Executable reconciliation kernel for the current Praxis operating law:
//!
//! `A = μ(O*)`, with DfCM preserving reversible lawful choices before selection
//! and BRCE enforcing **zero unreceipted actuation**.
//!
//! The crate makes the architecture boundary explicit:
//!
//! `observe → admit O* → SELECT(DfCM) → CONSTRUCT(intent) → admit authority → DO → receipt → replay`
//!
//! Hooks, planners, model output, proofs, and semantic derivations may manufacture
//! observations or intents. None receives ambient DO authority.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod brce;
pub mod dfcm;
pub mod model;
pub mod reconcile;

pub use brce::{
    admit_authority, construct_intent, execute_receipted, expected_receipt_digest,
    verify_receipt, ReceiptedActuator,
};
pub use dfcm::{admit_observation, select_maximal_reversible, MAX_DIMENSIONS, MAX_OPERATORS};
pub use model::{
    ActuationReceipt, AdmittedObservation, AuthorityGrant, CandidateEdge, ConstructedIntent,
    EvidenceState, Observation, PreparedReconciliation, ReconcileCheckpoint, Refusal, RefusalCode,
    RepairOperator, ReplayVerdict, ResidualVector, Selection, Standing,
};
pub use reconcile::{execute_prepared, prepare_reconciliation, ReconcileEnvironment};

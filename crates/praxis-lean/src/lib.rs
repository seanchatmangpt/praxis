//! Praxis Lean 4 manufacturing wrapper.
//!
//! This crate is intentionally small and hard-edged. It does not prove math.
//! It wraps Lean 4/Lake as the deterministic admission authority and records
//! each result as a replayable receipt.
//!
//! Core law:
//!
//! ```text
//! Verified(s) ⇔ KernelAccepts(s) ∧ NoSorry(s) ∧ NoUnauthorizedAxiom(s)
//! ```

pub mod cli;
pub mod error;
pub mod hash;
pub mod index;
pub mod lean;
pub mod no_sorry;
pub mod receipt;
pub mod report;
pub mod status;
pub mod verbs;

pub use error::{LeanRefusal, Result};
pub use index::{LeanDeclRecord, LeanDeclarationIndex};
pub use lean::{LeanCheck, LeanRunner, LeanToolchain};
pub use no_sorry::{AuditFinding, AuditPolicy, NoSorryAudit};
pub use receipt::{ReceiptLedger, VerificationReceipt};
pub use status::{FailureClass, VerificationStatus};

/// Current Praxis Lean release line.
pub const PRAXIS_LEAN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Canonical binary name.
pub const BIN_NAME: &str = "praxis-l4";

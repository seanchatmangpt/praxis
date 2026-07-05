use serde::{Deserialize, Serialize};

/// Manufacturing disposition for one Lean statement target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Unknown,
    Drafted,
    KernelAccepted,
    KernelRejected,
    Verified,
    Blocked,
    Excluded,
    Deduped,
    /// Exhausted all reject-fix-retry attempts without kernel acceptance.
    /// Distinct from `KernelRejected` (a single rejection outcome): this is
    /// the terminal disposition schema v1's `formalization_receipts.jsonl`
    /// calls `"unformalized"` after 3 real `lean` attempts.
    Unformalized,
    NoSorryFailed,
    AxiomUnauthorized,
    ReceiptMissing,
    ReplayFailed,
}

/// Coarse failure class used for repair routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    None,
    ParseError,
    TypeMismatch,
    UnknownIdentifier,
    MissingImport,
    MissingLemma,
    TacticFailure,
    Timeout,
    ContainsSorry,
    UnauthorizedAxiom,
    DependencyBlocked,
    ScopeExcluded,
    ReceiptMismatch,
    Unknown,
}

impl VerificationStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Verified
                | Self::KernelRejected
                | Self::Blocked
                | Self::Excluded
                | Self::Deduped
                | Self::Unformalized
                | Self::NoSorryFailed
                | Self::AxiomUnauthorized
                | Self::ReplayFailed
        )
    }
}

//! Error types for {{project-name}}.

use once_cell::sync::Lazy;
use thiserror::Error;

/// The top-level error type for this crate.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Argument validation failure. Every message MUST include FM identifier and remediation text.
    #[error("validation error: {0}")]
    Validation(String),

    #[error("{0}")]
    Other(String),
}

impl AppError {
    /// Construct a validation error. Callers must embed FM/RPN identifier and remediation text.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
}

/// Convenience alias used by the poka-yoke trait layer.
pub type Result<T> = std::result::Result<T, AppError>;

// ---------------------------------------------------------------------------
// Poka-yoke CLI validator trait (clnrm poka_yoke pattern)
// ---------------------------------------------------------------------------

/// Dyn-compatible argument validation surface.
///
/// Every `validate_*` method MUST return an error whose message contains:
/// 1. An FM/RPN identifier (e.g. `[FM-CLI-001]`).
/// 2. A human-readable remediation instruction.
///
/// RPN severity guide: 1 (certain detection) – 10 (undetectable).
pub trait CliValidator: Send + Sync {
    /// FM-CLI-001 | RPN-SEV-8: invalid parallel/job count combination.
    ///
    /// Failure Mode: `--parallel` flag set but `--jobs` is zero → runtime panic or no-op.
    /// Remediation: set `--jobs` to a positive integer or remove `--parallel` flag.
    fn validate_run_args(&self, parallel: bool, jobs: usize) -> Result<()>;
}

/// Default implementation of [`CliValidator`] enforcing all FM/RPN checks.
pub struct DefaultCliValidator;

impl Default for DefaultCliValidator {
    fn default() -> Self {
        Self
    }
}

impl CliValidator for DefaultCliValidator {
    fn validate_run_args(&self, parallel: bool, jobs: usize) -> Result<()> {
        // FM-CLI-001 | RPN-SEV-8
        if parallel && jobs == 0 {
            return Err(AppError::validation(
                "[FM-CLI-001] --parallel requires --jobs > 0. \
                 Remediation: pass --jobs <N> with a positive integer.",
            ));
        }
        Ok(())
    }
}

/// Process-wide singleton validator. Replace via dependency injection in tests.
pub static CLI_VALIDATOR: Lazy<DefaultCliValidator> = Lazy::new(DefaultCliValidator::default);

//! Fleet-wide validation layer for retrofitted repositories
//!
//! Post-retrofit validation that:
//! 1. Re-runs compliance audit on modified repositories
//! 2. Simulates CI gates (cargo fmt, clippy, test, deny, typos)
//! 3. Rolls back on failure with git reset --hard
//! 4. Generates validation reports with before/after compliance scores

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use chrono::Local;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::{models::*, PraxisSpec, Result, RetrofitError};

/// Comprehensive validation report for a retrofitted repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Repository metadata
    pub repository: RepositoryMetadata,

    /// Pre-retrofit compliance score
    pub pre_score: f32,

    /// Post-retrofit compliance score
    pub post_score: f32,

    /// Compliance improvement delta
    pub delta: f32,

    /// Pre-retrofit compliance items
    pub pre_checks: Vec<ComplianceItem>,

    /// Post-retrofit compliance items
    pub post_checks: Vec<ComplianceItem>,

    /// CI simulation results
    pub ci_results: Vec<CiGateResult>,

    /// Overall validation status
    pub status: RetrofitValidationStatus,

    /// Whether rollback was performed
    pub rolled_back: bool,

    /// Timestamp of validation
    pub timestamp: String,

    /// Detailed validation messages
    pub messages: Vec<String>,
}

/// Status of post-retrofit validation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetrofitValidationStatus {
    /// All checks passed
    #[serde(rename = "pass")]
    Pass,

    /// Some warnings present but no critical failures
    #[serde(rename = "warn")]
    Warn,

    /// Validation failed, rollback performed
    #[serde(rename = "fail")]
    Fail,
}

/// Result of a single CI gate check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiGateResult {
    /// Gate name (fmt, clippy, test, deny, typos)
    pub gate: CiGateName,

    /// Whether check passed
    pub passed: bool,

    /// Command output (truncated if too large)
    pub output: String,

    /// Error details if failed
    pub error: Option<String>,

    /// Execution duration in ms
    pub duration_ms: u64,
}

/// Names of CI gates to validate
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CiGateName {
    /// cargo fmt --check
    #[serde(rename = "fmt")]
    Fmt,

    /// cargo clippy --all-targets --all-features
    #[serde(rename = "clippy")]
    Clippy,

    /// cargo test --all-features
    #[serde(rename = "test")]
    Test,

    /// cargo deny check
    #[serde(rename = "deny")]
    Deny,

    /// typos check
    #[serde(rename = "typos")]
    Typos,
}

impl std::fmt::Display for CiGateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CiGateName::Fmt => write!(f, "fmt"),
            CiGateName::Clippy => write!(f, "clippy"),
            CiGateName::Test => write!(f, "test"),
            CiGateName::Deny => write!(f, "deny"),
            CiGateName::Typos => write!(f, "typos"),
        }
    }
}

/// Validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Run full test suite
    pub run_tests: bool,

    /// Run cargo clippy
    pub run_clippy: bool,

    /// Check code formatting
    pub check_fmt: bool,

    /// Run cargo deny check
    pub check_deny: bool,

    /// Run typos check
    pub check_typos: bool,

    /// Automatically rollback on CI failure
    pub auto_rollback: bool,

    /// Keep validation report even on failure
    pub keep_report: bool,

    /// Maximum output size per gate (bytes)
    pub max_output_size: usize,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            run_tests: true,
            run_clippy: true,
            check_fmt: true,
            check_deny: true,
            check_typos: true,
            auto_rollback: true,
            keep_report: true,
            max_output_size: 16384,
        }
    }
}

/// Main validator for retrofitted repositories
pub struct RetrofitValidator {
    config: ValidationConfig,
    spec: PraxisSpec,
}

impl RetrofitValidator {
    /// Create a new retrofit validator with default configuration
    pub fn new() -> Self {
        Self { config: ValidationConfig::default(), spec: PraxisSpec::default() }
    }

    /// Create a new validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config, spec: PraxisSpec::default() }
    }

    /// Set praxis spec for validation
    pub fn with_spec(mut self, spec: PraxisSpec) -> Self {
        self.spec = spec;
        self
    }

    /// Validate a retrofitted repository
    ///
    /// # Arguments
    /// - `repo_path`: Path to the repository to validate
    /// - `pre_report`: Pre-retrofit compliance report (baseline)
    ///
    /// # Returns
    /// Validation report with before/after comparison and CI results
    pub async fn validate_retrofit(
        &self,
        repo_path: &Path,
        pre_report: &ComplianceReport,
    ) -> Result<ValidationReport> {
        info!("Starting retrofit validation for: {}", repo_path.display());

        // Get initial git state for potential rollback
        let initial_state = capture_git_state(repo_path)?;

        // Collect error messages for rollback decision
        let mut error_messages = vec![];

        // Run compliance audit (post-retrofit)
        info!("Running post-retrofit compliance audit...");
        let post_report = crate::audit::scan_repository(repo_path, &self.spec).await?;

        // Run CI gates
        info!("Running CI gate simulations...");
        let ci_results = self.run_ci_gates(repo_path).await?;

        // Determine if we need to rollback
        let ci_failed = ci_results.iter().any(|r| !r.passed);
        let should_rollback = self.config.auto_rollback && ci_failed;

        // Collect error messages
        for result in &ci_results {
            if !result.passed {
                error_messages.push(format!(
                    "{} gate failed: {}",
                    result.gate,
                    result.error.as_deref().unwrap_or("unknown error")
                ));
            }
        }

        // Perform rollback if needed
        let mut rolled_back = false;
        if should_rollback {
            warn!("CI validation failed, rolling back to initial state...");
            if let Err(e) = restore_git_state(repo_path, &initial_state) {
                error!("Rollback failed: {}", e);
                error_messages.push(format!("Rollback error: {}", e));
            } else {
                rolled_back = true;
                info!("Rollback completed successfully");
            }
        }

        // Calculate compliance delta
        let pre_score = pre_report.score();
        let post_score = post_report.score();
        let delta = post_score - pre_score;

        // Determine overall status
        let status = if rolled_back {
            RetrofitValidationStatus::Fail
        } else if ci_failed {
            RetrofitValidationStatus::Warn
        } else {
            RetrofitValidationStatus::Pass
        };

        let report = ValidationReport {
            repository: post_report.repository.clone(),
            pre_score,
            post_score,
            delta,
            pre_checks: pre_report.checks.clone(),
            post_checks: post_report.checks.clone(),
            ci_results,
            status,
            rolled_back,
            timestamp: Local::now().to_rfc3339(),
            messages: error_messages,
        };

        info!(
            "Validation complete: {} (score: {:.1}% -> {:.1}%)",
            match status {
                RetrofitValidationStatus::Pass => "PASS",
                RetrofitValidationStatus::Warn => "WARN",
                RetrofitValidationStatus::Fail => "FAIL",
            },
            pre_score,
            post_score
        );

        Ok(report)
    }

    /// Run all configured CI gates
    async fn run_ci_gates(&self, repo_path: &Path) -> Result<Vec<CiGateResult>> {
        let mut results = vec![];

        if self.config.check_fmt {
            results.push(self.run_fmt_gate(repo_path).await?);
        }

        if self.config.run_clippy {
            results.push(self.run_clippy_gate(repo_path).await?);
        }

        if self.config.run_tests {
            results.push(self.run_test_gate(repo_path).await?);
        }

        if self.config.check_deny {
            results.push(self.run_deny_gate(repo_path).await?);
        }

        if self.config.check_typos {
            results.push(self.run_typos_gate(repo_path).await?);
        }

        Ok(results)
    }

    /// Run cargo fmt --check gate
    async fn run_fmt_gate(&self, repo_path: &Path) -> Result<CiGateResult> {
        debug!("Running cargo fmt check...");
        let start = std::time::Instant::now();

        let output = Command::new("cargo")
            .arg("fmt")
            .arg("--check")
            .current_dir(repo_path)
            .output()
            .map_err(|e| {
                RetrofitError::RetrofitFailed(format!("Failed to run cargo fmt: {}", e))
            })?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let passed = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let output_text =
            truncate_output(&format!("{}\n{}", stdout, stderr), self.config.max_output_size);

        Ok(CiGateResult {
            gate: CiGateName::Fmt,
            passed,
            output: output_text.clone(),
            error: if passed { None } else { Some(output_text) },
            duration_ms,
        })
    }

    /// Run cargo clippy gate
    async fn run_clippy_gate(&self, repo_path: &Path) -> Result<CiGateResult> {
        debug!("Running cargo clippy check...");
        let start = std::time::Instant::now();

        let output = Command::new("cargo")
            .arg("clippy")
            .arg("--all-targets")
            .arg("--all-features")
            .arg("--")
            .arg("-D")
            .arg("warnings")
            .current_dir(repo_path)
            .output()
            .map_err(|e| {
                RetrofitError::RetrofitFailed(format!("Failed to run cargo clippy: {}", e))
            })?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let passed = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let output_text =
            truncate_output(&format!("{}\n{}", stdout, stderr), self.config.max_output_size);

        Ok(CiGateResult {
            gate: CiGateName::Clippy,
            passed,
            output: output_text.clone(),
            error: if passed { None } else { Some(output_text) },
            duration_ms,
        })
    }

    /// Run cargo test gate
    async fn run_test_gate(&self, repo_path: &Path) -> Result<CiGateResult> {
        debug!("Running cargo test...");
        let start = std::time::Instant::now();

        let output = Command::new("cargo")
            .arg("test")
            .arg("--all-features")
            .current_dir(repo_path)
            .output()
            .map_err(|e| {
                RetrofitError::RetrofitFailed(format!("Failed to run cargo test: {}", e))
            })?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let passed = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let output_text =
            truncate_output(&format!("{}\n{}", stdout, stderr), self.config.max_output_size);

        Ok(CiGateResult {
            gate: CiGateName::Test,
            passed,
            output: output_text.clone(),
            error: if passed { None } else { Some(output_text) },
            duration_ms,
        })
    }

    /// Run cargo deny check gate
    async fn run_deny_gate(&self, repo_path: &Path) -> Result<CiGateResult> {
        debug!("Running cargo deny check...");
        let start = std::time::Instant::now();

        let output = Command::new("cargo")
            .arg("deny")
            .arg("check")
            .current_dir(repo_path)
            .output()
            .map_err(|e| {
                RetrofitError::RetrofitFailed(format!("Failed to run cargo deny: {}", e))
            })?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let passed = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let output_text =
            truncate_output(&format!("{}\n{}", stdout, stderr), self.config.max_output_size);

        Ok(CiGateResult {
            gate: CiGateName::Deny,
            passed,
            output: output_text.clone(),
            error: if passed { None } else { Some(output_text) },
            duration_ms,
        })
    }

    /// Run typos check gate
    async fn run_typos_gate(&self, repo_path: &Path) -> Result<CiGateResult> {
        debug!("Running typos check...");
        let start = std::time::Instant::now();

        let output = Command::new("typos")
            .current_dir(repo_path)
            .output()
            .map_err(|e| RetrofitError::RetrofitFailed(format!("Failed to run typos: {}", e)))?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let passed = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let output_text =
            truncate_output(&format!("{}\n{}", stdout, stderr), self.config.max_output_size);

        Ok(CiGateResult {
            gate: CiGateName::Typos,
            passed,
            output: output_text.clone(),
            error: if passed { None } else { Some(output_text) },
            duration_ms,
        })
    }
}

impl Default for RetrofitValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Git state snapshot for rollback
#[derive(Debug, Clone)]
struct GitState {
    /// Current HEAD commit SHA
    commit_sha: String,

    /// Current branch name
    branch: String,
}

/// Capture current git state for potential rollback
fn capture_git_state(repo_path: &Path) -> Result<GitState> {
    // Get current commit SHA
    let commit_output =
        Command::new("git").arg("rev-parse").arg("HEAD").current_dir(repo_path).output().map_err(
            |e| RetrofitError::RetrofitFailed(format!("Failed to get git commit: {}", e)),
        )?;

    let commit_sha = String::from_utf8_lossy(&commit_output.stdout).trim().to_string();

    // Get current branch name
    let branch_output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(repo_path)
        .output()
        .map_err(|e| RetrofitError::RetrofitFailed(format!("Failed to get git branch: {}", e)))?;

    let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();

    Ok(GitState { commit_sha, branch })
}

/// Restore repository to a previous git state
fn restore_git_state(repo_path: &Path, state: &GitState) -> Result<()> {
    // Reset to the captured commit
    let reset_output = Command::new("git")
        .arg("reset")
        .arg("--hard")
        .arg(&state.commit_sha)
        .current_dir(repo_path)
        .output()
        .map_err(|e| RetrofitError::RetrofitFailed(format!("Git reset failed: {}", e)))?;

    if !reset_output.status.success() {
        let stderr = String::from_utf8_lossy(&reset_output.stderr);
        return Err(RetrofitError::RetrofitFailed(format!("Git reset --hard failed: {}", stderr)));
    }

    Ok(())
}

/// Truncate output to maximum size
fn truncate_output(output: &str, max_size: usize) -> String {
    if output.len() > max_size {
        format!(
            "{}...\n[truncated: {} bytes omitted]",
            &output[..max_size],
            output.len() - max_size
        )
    } else {
        output.to_string()
    }
}

impl ValidationReport {
    /// Generate a human-readable summary of the validation
    pub fn summary(&self) -> String {
        let status_text = match self.status {
            RetrofitValidationStatus::Pass => "✓ PASSED",
            RetrofitValidationStatus::Warn => "⚠ WARNING",
            RetrofitValidationStatus::Fail => "✗ FAILED",
        };

        let rollback_text = if self.rolled_back { " (rolled back)" } else { "" };

        format!(
            "{} {}\nCompliance: {:.1}% → {:.1}% ({:+.1}%)\nCI Gates: {}/{} passed",
            status_text,
            rollback_text,
            self.pre_score,
            self.post_score,
            self.delta,
            self.ci_results.iter().filter(|r| r.passed).count(),
            self.ci_results.len(),
        )
    }

    /// Check if validation passed all checks
    pub fn is_successful(&self) -> bool {
        self.status == RetrofitValidationStatus::Pass && !self.rolled_back
    }

    /// Get CI results for a specific gate
    pub fn ci_result(&self, gate: CiGateName) -> Option<&CiGateResult> {
        self.ci_results.iter().find(|r| r.gate == gate)
    }

    /// Check if compliance improved
    pub fn improved(&self) -> bool {
        self.delta > 0.0
    }

    /// Check if compliance maintained
    pub fn maintained(&self) -> bool {
        self.delta >= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert!(config.run_tests);
        assert!(config.run_clippy);
        assert!(config.check_fmt);
        assert!(config.check_deny);
        assert!(config.check_typos);
        assert!(config.auto_rollback);
        assert!(config.keep_report);
    }

    #[test]
    fn test_retrofit_validator_default() {
        let _validator = RetrofitValidator::new();
        // Just ensure creation succeeds
    }

    #[test]
    fn test_ci_gate_name_display() {
        assert_eq!(CiGateName::Fmt.to_string(), "fmt");
        assert_eq!(CiGateName::Clippy.to_string(), "clippy");
        assert_eq!(CiGateName::Test.to_string(), "test");
        assert_eq!(CiGateName::Deny.to_string(), "deny");
        assert_eq!(CiGateName::Typos.to_string(), "typos");
    }

    #[test]
    fn test_truncate_output_small() {
        let output = "small output";
        let truncated = truncate_output(output, 100);
        assert_eq!(truncated, output);
    }

    #[test]
    fn test_truncate_output_large() {
        let output = "x".repeat(1000);
        let truncated = truncate_output(&output, 100);
        assert!(truncated.len() < 1000);
        assert!(truncated.contains("truncated"));
    }

    #[test]
    fn test_validation_report_summary() {
        let report = ValidationReport {
            repository: RepositoryMetadata {
                path: PathBuf::from("/test"),
                name: "test-repo".to_string(),
                workspace_root: PathBuf::from("/test"),
                crate_count: 1,
                has_workspace: false,
            },
            pre_score: 50.0,
            post_score: 80.0,
            delta: 30.0,
            pre_checks: vec![],
            post_checks: vec![],
            ci_results: vec![],
            status: RetrofitValidationStatus::Pass,
            rolled_back: false,
            timestamp: "2026-06-23T00:00:00Z".to_string(),
            messages: vec![],
        };

        let summary = report.summary();
        assert!(summary.contains("PASSED"));
        assert!(summary.contains("50.0%"));
        assert!(summary.contains("80.0%"));
        assert!(summary.contains("+30.0%"));
    }

    #[test]
    fn test_validation_report_improved() {
        let report = ValidationReport {
            repository: RepositoryMetadata {
                path: PathBuf::from("/test"),
                name: "test".to_string(),
                workspace_root: PathBuf::from("/test"),
                crate_count: 1,
                has_workspace: false,
            },
            pre_score: 50.0,
            post_score: 80.0,
            delta: 30.0,
            pre_checks: vec![],
            post_checks: vec![],
            ci_results: vec![],
            status: RetrofitValidationStatus::Pass,
            rolled_back: false,
            timestamp: "2026-06-23T00:00:00Z".to_string(),
            messages: vec![],
        };

        assert!(report.improved());
        assert!(report.maintained());
        assert!(report.is_successful());
    }
}

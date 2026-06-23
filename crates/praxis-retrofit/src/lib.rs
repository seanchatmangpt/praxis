//! Praxis Retrofit: Automate standardization across Rust ecosystem
//!
//! Retrofits existing repositories with praxis house-style standards:
//! - Workspace [lints] configuration
//! - Dependency unification via [workspace.dependencies]
//! - Justfile standardization
//! - typos.toml spell-check configuration
//! - Compliance validation gates
//!
//! # Usage
//!
//! ```text
//! praxis-retrofit audit scan <repo-path>       # Audit repo compliance
//! praxis-retrofit audit report <repo-path>     # Generate compliance report
//! praxis-retrofit apply retrofit <repo-path>   # Apply retrofit changes
//! praxis-retrofit apply validate <repo-path>   # Validate retrofit success
//! praxis-retrofit generate templates            # Generate template files
//! praxis-retrofit validate compliance <repo-path> # CI compliance gate
//! ```

pub mod audit;
pub mod apply;
pub mod generate;
pub mod validate;
pub mod templates;
pub mod error;
pub mod models;
pub mod fleet_audit;
pub mod repo_registry;
pub mod fleet_validate;
pub mod fleet_apply;
pub mod compliance_dashboard;
pub mod ci_gate;
pub mod pr_generator;
pub mod preventive_gate;
pub mod fleet_models;
pub mod fleet_aggregation;
pub mod fleet_readiness;
pub mod fleet_export;

pub use error::{RetrofitError, Result};
pub use models::{
    ComplianceReport, RetrofitPlan, RepositoryMetadata, RetrofitPhase, RiskLevel,
    ComplianceStatus, ComplianceCategory, RetrofitAction, RetrofitActionType,
};
pub use fleet_validate::{
    RetrofitValidator, ValidationReport, RetrofitValidationStatus, ValidationConfig,
    CiGateResult, CiGateName,
};
pub use fleet_apply::{RetrofitApplier, RetrofitWorktree, ApplyResult, FleetApplyReport};
pub use ci_gate::{
    ComplianceGate, GateConfig, GateCheckOutput, GateResult, RemediationStep, RemediationPriority,
    BadgeGenerator, format_remediation_markdown,
};
pub use pr_generator::{
    PullRequestGenerator, PullRequestTemplate, PullRequestInfo, FleetPRStatus,
    PullRequestGeneratorConfig, PRStatus, PRStatusCounts,
};
pub use repo_registry::{RepositoryEntry, RepositoryRegistry, EcosystemMetadata};
pub use fleet_audit::{
    ComplianceMatrix, FleetAuditCoordinator, FleetSummary, AuditObserver,
    CategoryStatus, AuditCriticalIssue, AuditMetadata,
};
pub use preventive_gate::{
    GateValidator, ValidationResult, ValidationStatus, ValidateCategory,
    Severity, GateReport,
};
pub use fleet_models::{
    FleetComplianceReport, ComplianceHeatmap, PhaseReadinessSummary, FleetHealthMetrics,
    ReadinessStatus, HealthRating, HeatmapCell, RepositoryPhaseReadiness,
    FleetCriticalIssue, IssueSeverity,
};
pub use fleet_export::ExportFormat;

/// Current praxis retrofit version (CalVer)
pub const VERSION: &str = "26.6.0";

/// Praxis standards specification
#[derive(Debug, Clone)]
pub struct PraxisSpec {
    pub edition: String,
    pub msrv: String,
    pub toolchain: String,
    pub lints_strict: bool,
    pub license_preferred: String,
    pub linters: Vec<String>,
}

impl Default for PraxisSpec {
    fn default() -> Self {
        Self {
            edition: "2021".to_string(),
            msrv: "1.82".to_string(),
            toolchain: "nightly-2026-04-15".to_string(),
            lints_strict: true,
            license_preferred: "MIT OR Apache-2.0".to_string(),
            linters: vec![
                "unsafe_code=forbid".to_string(),
                "clippy/all=warn".to_string(),
                "clippy/pedantic=warn".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_spec() {
        let spec = PraxisSpec::default();
        assert_eq!(spec.edition, "2021");
        assert_eq!(spec.msrv, "1.82");
        assert!(spec.lints_strict);
    }
}

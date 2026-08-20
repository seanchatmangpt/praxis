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

// Recorded lint debt (v26.7.6 verification gate): 615 findings at HEAD once
// CI's `-D warnings` promotes this crate's aspirational `pedantic = "warn"`
// policy to errors. `clippy::correctness` and the forbid/deny safety lints
// (unsafe_code, todo, unimplemented, dbg_macro) stay fully active. Debt is
// tracked in docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(missing_docs, dead_code)]
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

pub mod apply;
pub mod audit;
pub mod ci_gate;
pub mod compliance_dashboard;
pub mod error;
pub mod fleet_aggregation;
pub mod fleet_apply;
pub mod fleet_audit;
pub mod fleet_export;
pub mod fleet_models;
pub mod fleet_readiness;
pub mod fleet_validate;
pub mod generate;
pub mod models;
pub mod ocel_log;
pub mod pr_generator;
pub mod preventive_gate;
pub mod process_discovery;
pub mod repo_registry;
pub mod templates;
pub mod validate;

pub use ci_gate::{
    format_remediation_markdown, BadgeGenerator, ComplianceGate, GateCheckOutput, GateConfig,
    GateResult, RemediationPriority, RemediationStep,
};
pub use error::{Result, RetrofitError};
pub use fleet_apply::{ApplyResult, FleetApplyReport, RetrofitApplier, RetrofitWorktree};
pub use fleet_audit::{
    AuditCriticalIssue, AuditMetadata, AuditObserver, CategoryStatus, ComplianceMatrix,
    FleetAuditCoordinator, FleetSummary,
};
pub use fleet_export::ExportFormat;
pub use fleet_models::{
    ComplianceHeatmap, FleetComplianceReport, FleetCriticalIssue, FleetHealthMetrics, HealthRating,
    HeatmapCell, IssueSeverity, PhaseReadinessSummary, ReadinessStatus, RepositoryPhaseReadiness,
};
pub use fleet_validate::{
    CiGateName, CiGateResult, RetrofitValidationStatus, RetrofitValidator, ValidationConfig,
    ValidationReport,
};
pub use models::{
    ComplianceCategory, ComplianceItem, ComplianceReport, ComplianceStatus, RepositoryMetadata,
    RetrofitAction, RetrofitActionType, RetrofitPhase, RetrofitPlan, RiskLevel,
};
pub use pr_generator::{
    FleetPRStatus, PRStatus, PRStatusCounts, PullRequestGenerator, PullRequestGeneratorConfig,
    PullRequestInfo, PullRequestTemplate,
};
pub use preventive_gate::{
    GateReport, GateValidator, Severity, ValidateCategory, ValidationResult, ValidationStatus,
};
pub use repo_registry::{EcosystemMetadata, RepositoryEntry, RepositoryRegistry};
pub use validate::{fleet_compliance_score, is_fleet_compliant, validate_compliance};

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

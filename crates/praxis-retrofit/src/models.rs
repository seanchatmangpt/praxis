use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Metadata about a repository being audited
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub path: PathBuf,
    pub name: String,
    pub workspace_root: PathBuf,
    pub crate_count: usize,
    pub has_workspace: bool,
}

/// Individual compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceItem {
    pub name: String,
    pub category: ComplianceCategory,
    pub status: ComplianceStatus,
    pub evidence: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComplianceCategory {
    #[serde(rename = "ci-cd")]
    CiCd,
    #[serde(rename = "supply-chain")]
    SupplyChain,
    #[serde(rename = "linting")]
    Linting,
    #[serde(rename = "editor-config")]
    EditorConfig,
    #[serde(rename = "documentation")]
    Documentation,
    #[serde(rename = "licensing")]
    Licensing,
    #[serde(rename = "versioning")]
    Versioning,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComplianceStatus {
    #[serde(rename = "pass")]
    Pass,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "fail")]
    Fail,
}

/// Comprehensive compliance report for a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub repository: RepositoryMetadata,
    pub timestamp: String,
    pub checks: Vec<ComplianceItem>,
    pub score: f32,
    pub summary: String,
}

impl ComplianceReport {
    pub fn score(&self) -> f32 {
        let total = self.checks.len() as f32;
        if total == 0.0 {
            return 100.0;
        }
        let passes =
            self.checks.iter().filter(|c| c.status == ComplianceStatus::Pass).count() as f32;
        (passes / total) * 100.0
    }

    pub fn is_compliant(&self) -> bool {
        !self.checks.iter().any(|c| c.status == ComplianceStatus::Fail)
    }
}

/// Retrofit action to apply to a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrofitAction {
    pub action_type: RetrofitActionType,
    pub file_path: PathBuf,
    pub content: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetrofitActionType {
    #[serde(rename = "create")]
    Create,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "delete")]
    Delete,
}

/// Complete retrofit plan for a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrofitPlan {
    pub repository: RepositoryMetadata,
    pub actions: Vec<RetrofitAction>,
    pub phase: RetrofitPhase,
    pub estimated_risk: RiskLevel,
    pub commit_message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RetrofitPhase {
    #[serde(rename = "phase-1-lints")]
    Phase1Lints,
    #[serde(rename = "phase-2-deps")]
    Phase2Deps,
    #[serde(rename = "phase-3-justfile")]
    Phase3Justfile,
    #[serde(rename = "phase-4-typos")]
    Phase4Typos,
    #[serde(rename = "phase-5-docs")]
    Phase5Docs,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
}

/// Fleet-wide retrofit summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetRetrofitPlan {
    pub repositories: Vec<RetrofitPlan>,
    pub total_actions: usize,
    pub total_risk: RiskLevel,
    pub estimated_duration_weeks: f32,
}

impl FleetRetrofitPlan {
    pub fn total_risk(&self) -> RiskLevel {
        self.repositories.iter().map(|r| r.estimated_risk).max().unwrap_or(RiskLevel::Low)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_item_serde_roundtrip() {
        let item = ComplianceItem {
            name: "CI/CD Pipeline".to_string(),
            category: ComplianceCategory::CiCd,
            status: ComplianceStatus::Pass,
            evidence: "workflows present".to_string(),
            remediation: None,
        };
        let json = serde_json::to_string(&item).expect("serialize");
        let back: ComplianceItem = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.name, item.name);
        assert_eq!(back.category, item.category);
        assert_eq!(back.status, item.status);
    }
}

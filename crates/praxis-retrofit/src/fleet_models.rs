//! Fleet-wide compliance aggregation models
//!
//! Data structures for aggregating individual repository compliance reports
//! into fleet-wide audit and readiness assessments.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::{ComplianceCategory, ComplianceReport, ComplianceStatus, RetrofitPhase};

/// Cell status in compliance heatmap with counts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeatmapCell {
    /// Predominant status (worst case wins)
    pub status: ComplianceStatus,
    /// Count of items with Pass status
    pub pass_count: usize,
    /// Count of items with Warn status
    pub warn_count: usize,
    /// Count of items with Fail status
    pub fail_count: usize,
}

impl HeatmapCell {
    /// Create a new heatmap cell from status counts
    pub fn new(pass_count: usize, warn_count: usize, fail_count: usize) -> Self {
        let status = if fail_count > 0 {
            ComplianceStatus::Fail
        } else if warn_count > 0 {
            ComplianceStatus::Warn
        } else {
            ComplianceStatus::Pass
        };

        Self {
            status,
            pass_count,
            warn_count,
            fail_count,
        }
    }

    /// Total checks in this cell
    pub fn total(&self) -> usize {
        self.pass_count + self.warn_count + self.fail_count
    }

    /// Compliance percentage (pass / total)
    pub fn compliance_percent(&self) -> f32 {
        let total = self.total();
        if total == 0 {
            100.0
        } else {
            (self.pass_count as f32 / total as f32) * 100.0
        }
    }
}

/// Compliance heatmap: repositories vs categories with status distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceHeatmap {
    /// Map of (repo_name, category) -> HeatmapCell
    pub matrix: HashMap<(String, ComplianceCategory), HeatmapCell>,
    /// Ordered list of repository names (for row order)
    pub repositories: Vec<String>,
    /// Ordered list of categories (for column order)
    pub categories: Vec<ComplianceCategory>,
}

impl ComplianceHeatmap {
    /// Create a new empty heatmap
    pub fn new(repositories: Vec<String>, categories: Vec<ComplianceCategory>) -> Self {
        Self {
            matrix: HashMap::new(),
            repositories,
            categories,
        }
    }

    /// Set a cell in the heatmap
    pub fn set_cell(&mut self, repo: String, category: ComplianceCategory, cell: HeatmapCell) {
        self.matrix.insert((repo, category), cell);
    }

    /// Get a cell from the heatmap
    pub fn get_cell(&self, repo: &str, category: ComplianceCategory) -> Option<&HeatmapCell> {
        self.matrix.get(&(repo.to_string(), category))
    }

    /// Aggregate statistics for a category (column sum)
    pub fn category_stats(&self, category: ComplianceCategory) -> HeatmapCell {
        let mut total_pass = 0;
        let mut total_warn = 0;
        let mut total_fail = 0;

        for repo in &self.repositories {
            if let Some(cell) = self.get_cell(repo, category) {
                total_pass += cell.pass_count;
                total_warn += cell.warn_count;
                total_fail += cell.fail_count;
            }
        }

        HeatmapCell::new(total_pass, total_warn, total_fail)
    }

    /// Aggregate statistics for a repository (row sum)
    pub fn repository_stats(&self, repo: &str) -> HeatmapCell {
        let mut total_pass = 0;
        let mut total_warn = 0;
        let mut total_fail = 0;

        for category in &self.categories {
            if let Some(cell) = self.get_cell(repo, *category) {
                total_pass += cell.pass_count;
                total_warn += cell.warn_count;
                total_fail += cell.fail_count;
            }
        }

        HeatmapCell::new(total_pass, total_warn, total_fail)
    }

    /// Overall fleet statistics
    pub fn overall_stats(&self) -> HeatmapCell {
        let mut total_pass = 0;
        let mut total_warn = 0;
        let mut total_fail = 0;

        for cell in self.matrix.values() {
            total_pass += cell.pass_count;
            total_warn += cell.warn_count;
            total_fail += cell.fail_count;
        }

        HeatmapCell::new(total_pass, total_warn, total_fail)
    }

    /// Sort repositories by compliance score (descending: best first)
    pub fn sort_repos_by_score(&mut self) {
        // Pre-compute scores to avoid borrow issues
        let mut scores: Vec<_> = self
            .repositories
            .iter()
            .map(|r| (r.clone(), self.repository_stats(r).compliance_percent()))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        self.repositories = scores.into_iter().map(|(r, _)| r).collect();
    }

    /// Sort repositories by fail count (ascending: fewest fails first)
    pub fn sort_repos_by_fail_count(&mut self) {
        // Pre-compute fail counts to avoid borrow issues
        let mut fail_counts: Vec<_> = self
            .repositories
            .iter()
            .map(|r| (r.clone(), self.repository_stats(r).fail_count))
            .collect();
        fail_counts.sort_by(|a, b| a.1.cmp(&b.1));
        self.repositories = fail_counts.into_iter().map(|(r, _)| r).collect();
    }
}

/// Readiness status for a repository in a given phase
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReadinessStatus {
    /// Repository is ready to begin this phase
    #[serde(rename = "ready")]
    Ready,
    /// Repository is blocked on a prerequisite phase
    #[serde(rename = "blocked")]
    BlockedOn(RetrofitPhase),
    /// Repository cannot proceed due to high risk factors
    #[serde(rename = "high-risk")]
    HighRisk(String),
    /// Repository has already completed this phase
    #[serde(rename = "completed")]
    Completed,
}

impl ReadinessStatus {
    /// Whether this status indicates the repo can start the phase
    pub fn can_proceed(&self) -> bool {
        matches!(self, ReadinessStatus::Ready)
    }

    /// Human-readable description of status
    pub fn description(&self) -> String {
        match self {
            ReadinessStatus::Ready => "Ready to start this phase".to_string(),
            ReadinessStatus::BlockedOn(phase) => {
                format!("Blocked on {:?}", phase)
            }
            ReadinessStatus::HighRisk(reason) => {
                format!("High risk: {}", reason)
            }
            ReadinessStatus::Completed => "Phase completed".to_string(),
        }
    }
}

/// Per-repository readiness assessment for a single phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryPhaseReadiness {
    pub repository_name: String,
    pub phase: RetrofitPhase,
    pub status: ReadinessStatus,
    pub estimated_actions: usize,
    pub estimated_risk_level: crate::models::RiskLevel,
}

/// Summary of phase readiness across the fleet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseReadinessSummary {
    pub phase: RetrofitPhase,
    pub total_repositories: usize,
    pub ready_count: usize,
    pub blocked_count: usize,
    pub high_risk_count: usize,
    pub completed_count: usize,
    pub per_repo: Vec<RepositoryPhaseReadiness>,
}

impl PhaseReadinessSummary {
    /// Progress percentage: (completed + ready) / total
    pub fn progress_percent(&self) -> f32 {
        let actionable = self.ready_count + self.completed_count;
        (actionable as f32 / self.total_repositories.max(1) as f32) * 100.0
    }

    /// Whether the phase can begin fleet-wide
    pub fn fleet_can_begin(&self) -> bool {
        self.ready_count > 0 || self.completed_count > 0
    }
}

/// Health rating based on compliance score
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum HealthRating {
    #[serde(rename = "excellent")]
    Excellent, // 90-100%
    #[serde(rename = "good")]
    Good, // 75-89%
    #[serde(rename = "fair")]
    Fair, // 50-74%
    #[serde(rename = "poor")]
    Poor, // <50%
}

impl HealthRating {
    /// Determine rating from compliance percentage
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 90.0 => HealthRating::Excellent,
            s if s >= 75.0 => HealthRating::Good,
            s if s >= 50.0 => HealthRating::Fair,
            _ => HealthRating::Poor,
        }
    }

    /// Human-readable description
    pub fn description(&self) -> &str {
        match self {
            HealthRating::Excellent => "Excellent compliance",
            HealthRating::Good => "Good compliance",
            HealthRating::Fair => "Fair compliance",
            HealthRating::Poor => "Poor compliance",
        }
    }
}

/// Fleet-wide health metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetHealthMetrics {
    /// Timestamp of this metrics snapshot
    pub timestamp: String,
    /// Total number of repositories audited
    pub total_repositories: usize,
    /// Overall fleet compliance score (0-100)
    pub overall_score: f32,
    /// Overall health rating
    pub health_rating: HealthRating,
    /// Percentage of checks passing
    pub pass_percent: f32,
    /// Percentage of checks warning
    pub warn_percent: f32,
    /// Percentage of checks failing
    pub fail_percent: f32,
    /// Total checks across fleet
    pub total_checks: usize,
    /// Pass count across fleet
    pub pass_count: usize,
    /// Warn count across fleet
    pub warn_count: usize,
    /// Fail count across fleet
    pub fail_count: usize,
    /// Compliance stats per category
    pub category_metrics: HashMap<String, CategoryMetrics>,
    /// Compliance score distribution
    pub score_distribution: ScoreDistribution,
}

/// Metrics for a single compliance category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMetrics {
    pub category_name: String,
    pub total_checks: usize,
    pub pass_count: usize,
    pub warn_count: usize,
    pub fail_count: usize,
    pub pass_percent: f32,
}

impl CategoryMetrics {
    /// Create metrics from counts
    pub fn new(
        category_name: String,
        pass_count: usize,
        warn_count: usize,
        fail_count: usize,
    ) -> Self {
        let total = pass_count + warn_count + fail_count;
        let pass_percent = if total > 0 {
            (pass_count as f32 / total as f32) * 100.0
        } else {
            100.0
        };

        Self {
            category_name,
            total_checks: total,
            pass_count,
            warn_count,
            fail_count,
            pass_percent,
        }
    }
}

/// Distribution of compliance scores across the fleet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDistribution {
    /// Number of repos with excellent scores (90-100%)
    pub excellent_count: usize,
    /// Number of repos with good scores (75-89%)
    pub good_count: usize,
    /// Number of repos with fair scores (50-74%)
    pub fair_count: usize,
    /// Number of repos with poor scores (<50%)
    pub poor_count: usize,
    /// Minimum score in fleet
    pub min_score: f32,
    /// Maximum score in fleet
    pub max_score: f32,
    /// Average score across fleet
    pub average_score: f32,
    /// Median score (when sorted)
    pub median_score: f32,
}

/// Complete fleet-wide compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetComplianceReport {
    /// Timestamp when report was generated
    pub timestamp: String,
    /// Individual compliance reports for each repository
    pub reports: Vec<ComplianceReport>,
    /// Compliance heatmap
    pub heatmap: ComplianceHeatmap,
    /// Fleet-wide health metrics
    pub metrics: FleetHealthMetrics,
    /// Phase readiness assessment for each phase
    pub phase_readiness: Vec<PhaseReadinessSummary>,
    /// Critical issues requiring attention
    pub critical_issues: Vec<FleetCriticalIssue>,
}

impl FleetComplianceReport {
    /// Count of repositories at a given health rating
    pub fn repos_by_health(&self, rating: HealthRating) -> Vec<String> {
        self.reports
            .iter()
            .filter(|r| HealthRating::from_score(r.score()) == rating)
            .map(|r| r.repository.name.clone())
            .collect()
    }

    /// Count of repositories compliant (no fails)
    pub fn compliant_repos(&self) -> usize {
        self.reports.iter().filter(|r| r.is_compliant()).count()
    }

    /// Percentage of repositories that are compliant
    pub fn compliance_rate_percent(&self) -> f32 {
        if self.reports.is_empty() {
            100.0
        } else {
            (self.compliant_repos() as f32 / self.reports.len() as f32) * 100.0
        }
    }
}

/// A fleet-wide critical issue affecting multiple repositories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetCriticalIssue {
    /// Type of issue (e.g., "missing-deny-toml", "failing-workspace-lints")
    pub issue_type: String,
    /// Affected repositories
    pub affected_repos: Vec<String>,
    /// Recommended remediation
    pub remediation: String,
    /// Severity level
    pub severity: IssueSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "critical")]
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heatmap_cell_new() {
        let cell = HeatmapCell::new(10, 2, 1);
        assert_eq!(cell.status, ComplianceStatus::Fail);
        assert_eq!(cell.total(), 13);
    }

    #[test]
    fn test_heatmap_cell_compliance_percent() {
        let cell = HeatmapCell::new(80, 15, 5);
        assert!((cell.compliance_percent() - 80.0).abs() < 0.1);
    }

    #[test]
    fn test_health_rating_from_score() {
        assert_eq!(HealthRating::from_score(95.0), HealthRating::Excellent);
        assert_eq!(HealthRating::from_score(80.0), HealthRating::Good);
        assert_eq!(HealthRating::from_score(60.0), HealthRating::Fair);
        assert_eq!(HealthRating::from_score(40.0), HealthRating::Poor);
    }

    #[test]
    fn test_readiness_status_can_proceed() {
        assert!(ReadinessStatus::Ready.can_proceed());
        assert!(!ReadinessStatus::BlockedOn(RetrofitPhase::Phase1Lints).can_proceed());
        assert!(!ReadinessStatus::HighRisk("test".to_string()).can_proceed());
        assert!(!ReadinessStatus::Completed.can_proceed());
    }
}

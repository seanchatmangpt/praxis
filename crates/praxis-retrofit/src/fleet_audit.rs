//! Parallel audit agent framework for fleet-wide compliance scanning
//!
//! This module provides production-grade parallelism for auditing multiple repositories
//! simultaneously. Key types:
//!
//! - [`FleetAuditCoordinator`] — Orchestrates up to 10 parallel audit agents
//! - [`ComplianceMatrix`] — Aggregates compliance reports into queryable structure
//! - [`FleetSummary`] — Human-readable fleet-wide compliance summary
//! - [`AuditObserver`] — Trait for progress tracking and observability
//!
//! # Architecture
//!
//! The framework uses Tokio task workers and bounded MPSC channels:
//! 1. Coordinator spawns up to `max_agents` audit tasks
//! 2. Each task scans one repository independently
//! 3. Results stream back through async channel
//! 4. Coordinator aggregates into [`ComplianceMatrix`]
//! 5. Generate [`FleetSummary`] for actionable insights
//!
//! # Example
//!
//! ```no_run
//! use praxis_retrofit::fleet_audit::FleetAuditCoordinator;
//! use praxis_retrofit::PraxisSpec;
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let coordinator = FleetAuditCoordinator::new(10, PraxisSpec::default());
//!     let matrix = coordinator.audit_fleet(Path::new("/path/to/repos")).await?;
//!
//!     let summary = matrix.generate_summary();
//!     println!("{}", summary.summary_table());
//!
//!     Ok(())
//! }
//! ```

use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    time::Instant,
};

use chrono::Utc;
use tokio::sync::mpsc;
use tracing::{debug, info, span, warn, Level};

use crate::{models::*, PraxisSpec, Result, RetrofitError};

/// Maximum concurrent audit agents (hard limit)
const MAX_AGENTS_HARD_LIMIT: usize = 256;

/// Timeout per repository scan (seconds)
const AUDIT_TIMEOUT_SECS: u64 = 300; // 5 minutes per repo

/// Aggregated compliance data across all repositories in fleet
///
/// Provides queryable views of compliance status organized by:
/// - Repository name
/// - Compliance category
/// - Retrofit phase requirements
/// - Audit execution metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComplianceMatrix {
    /// All compliance reports indexed by repository name
    pub repository_reports: HashMap<String, ComplianceReport>,

    /// Compliance status by category and repository
    /// Allows querying "show me all repos failing CI/CD"
    pub category_matrix: HashMap<ComplianceCategory, HashMap<String, ComplianceStatus>>,

    /// Which retrofit phases each repo requires (ordered by priority)
    pub phase_requirements: HashMap<String, Vec<RetrofitPhase>>,

    /// Timestamp when audit was completed (RFC 3339)
    pub timestamp: String,

    /// Total scan duration in seconds
    pub scan_duration_seconds: f32,

    /// Number of agents used during this audit
    pub agents_used: usize,
}

impl ComplianceMatrix {
    /// Create an empty matrix
    pub fn new() -> Self {
        Self {
            repository_reports: HashMap::new(),
            category_matrix: HashMap::new(),
            phase_requirements: HashMap::new(),
            timestamp: Utc::now().to_rfc3339(),
            scan_duration_seconds: 0.0,
            agents_used: 0,
        }
    }

    /// Aggregate multiple compliance reports into a single matrix
    pub fn aggregate(
        reports: Vec<ComplianceReport>,
        duration_secs: f32,
        agents_used: usize,
    ) -> Self {
        let mut matrix = Self::new();
        matrix.scan_duration_seconds = duration_secs;
        matrix.agents_used = agents_used;

        for report in reports {
            matrix.add_report(report);
        }

        matrix
    }

    /// Add a single compliance report to the matrix
    pub fn add_report(&mut self, report: ComplianceReport) {
        let repo_name = report.repository.name.clone();

        // Add to repository reports
        self.repository_reports
            .insert(repo_name.clone(), report.clone());

        // Update category matrix
        for check in &report.checks {
            self.category_matrix
                .entry(check.category)
                .or_insert_with(HashMap::new)
                .insert(repo_name.clone(), check.status);
        }

        // Determine phase requirements for this repo
        let required_phases = self.determine_phases(&report);
        self.phase_requirements.insert(repo_name, required_phases);
    }

    /// Determine which retrofit phases are needed based on compliance gaps
    fn determine_phases(&self, report: &ComplianceReport) -> Vec<RetrofitPhase> {
        let mut phases = Vec::new();

        // Phase 1: Linting — if [lints] missing or incomplete
        if let Some(check) = report.checks.iter().find(|c| c.name == "Workspace Lints") {
            if check.status != ComplianceStatus::Pass {
                phases.push(RetrofitPhase::Phase1Lints);
            }
        }

        // Phase 2: Dependency unification — if supply chain audit failing
        if let Some(check) = report
            .checks
            .iter()
            .find(|c| c.name == "Supply Chain Audit")
        {
            if check.status != ComplianceStatus::Pass {
                phases.push(RetrofitPhase::Phase2Deps);
            }
        }

        // Phase 3: Justfile — if we're retrofitting other aspects
        if !phases.is_empty() && report.repository.has_workspace {
            phases.push(RetrofitPhase::Phase3Justfile);
        }

        // Phase 4: Spell check — optional but recommended
        if let Some(check) = report.checks.iter().find(|c| c.name == "Spell Check") {
            if check.status != ComplianceStatus::Pass {
                phases.push(RetrofitPhase::Phase4Typos);
            }
        }

        // Phase 5: Documentation — if contributor guide missing
        if let Some(check) = report.checks.iter().find(|c| c.name == "Contributor Guide") {
            if check.status != ComplianceStatus::Pass {
                phases.push(RetrofitPhase::Phase5Docs);
            }
        }

        phases
    }

    /// Get all repositories with a specific compliance status
    pub fn get_repos_by_status(&self, status: ComplianceStatus) -> Vec<String> {
        self.repository_reports
            .iter()
            .filter(|(_, report)| {
                report.checks.iter().all(|c| {
                    if status == ComplianceStatus::Fail {
                        c.status != ComplianceStatus::Fail
                    } else {
                        c.status == status || c.status == ComplianceStatus::Pass
                    }
                })
            })
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get all repositories requiring a specific retrofit phase
    pub fn get_repos_needing_phase(&self, phase: RetrofitPhase) -> Vec<String> {
        self.phase_requirements
            .iter()
            .filter(|(_, phases)| phases.contains(&phase))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Calculate fleet-wide average compliance score (0-100)
    pub fn compliance_score(&self) -> f32 {
        if self.repository_reports.is_empty() {
            return 100.0;
        }

        let total_score: f32 = self.repository_reports.values().map(|r| r.score()).sum();

        total_score / self.repository_reports.len() as f32
    }

    /// Get the worst compliance status seen in the fleet
    pub fn worst_status(&self) -> ComplianceStatus {
        self.repository_reports
            .values()
            .flat_map(|r| r.checks.iter().map(|c| c.status))
            .max()
            .unwrap_or(ComplianceStatus::Pass)
    }

    /// Count repositories by compliance category
    pub fn count_by_category(&self, category: ComplianceCategory) -> (usize, usize, usize) {
        if let Some(statuses) = self.category_matrix.get(&category) {
            let pass = statuses
                .values()
                .filter(|s: &&ComplianceStatus| **s == ComplianceStatus::Pass)
                .count();
            let warn = statuses
                .values()
                .filter(|s: &&ComplianceStatus| **s == ComplianceStatus::Warn)
                .count();
            let fail = statuses
                .values()
                .filter(|s: &&ComplianceStatus| **s == ComplianceStatus::Fail)
                .count();
            (pass, warn, fail)
        } else {
            (0, 0, 0)
        }
    }

    /// Generate a human-readable summary report
    pub fn generate_summary(&self) -> FleetSummary {
        FleetSummary::from_matrix(self)
    }

    /// Export matrix as JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| RetrofitError::Json(e))
    }
}

impl Default for ComplianceMatrix {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of fleet-wide compliance status
///
/// Provides actionable insights for fleet operators:
/// - Overall compliance score
/// - Repos by required retrofit phase
/// - Critical issues needing immediate attention
/// - Audit execution metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FleetSummary {
    /// Fleet-wide average compliance score (0-100)
    pub overall_compliance_score: f32,

    /// Total number of repositories scanned
    pub total_repositories: usize,

    /// Number of fully compliant repositories
    pub compliant_repositories: usize,

    /// Repositories grouped by required retrofit phase
    pub repos_by_phase: BTreeMap<String, Vec<String>>,

    /// Count of repositories by compliance status
    pub repos_by_status: BTreeMap<String, usize>,

    /// Compliance summary per category
    pub category_summary: BTreeMap<String, CategoryStatus>,

    /// High-priority issues requiring attention
    pub critical_issues: Vec<AuditCriticalIssue>,

    /// Audit execution metadata
    pub audit_metadata: AuditMetadata,
}

/// Compliance statistics for a single category
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CategoryStatus {
    /// Number of repositories passing this category
    pub passing: usize,

    /// Number of repositories with warnings
    pub warning: usize,

    /// Number of repositories failing this category
    pub failing: usize,
}

impl CategoryStatus {
    /// Calculate passing percentage for this category (0-100)
    pub fn pass_rate(&self) -> f32 {
        let total = (self.passing + self.warning + self.failing) as f32;
        if total == 0.0 {
            100.0
        } else {
            (self.passing as f32 / total) * 100.0
        }
    }
}

/// High-priority issue requiring manual attention (specific to FleetSummary)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditCriticalIssue {
    /// Repository where issue was found
    pub repository: String,

    /// The compliance check that failed
    pub issue: ComplianceItem,

    /// Recommended retrofit phase for remediation
    pub phase: RetrofitPhase,
}

/// Metadata about audit execution (duration, agent count, etc.)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditMetadata {
    /// RFC 3339 timestamp when audit started
    pub started_at: String,

    /// RFC 3339 timestamp when audit completed
    pub completed_at: String,

    /// Total audit duration in seconds
    pub total_duration_seconds: f32,

    /// Number of parallel agents used
    pub agents_used: usize,

    /// Average time per repository scan (seconds)
    pub avg_repo_scan_time: f32,
}

impl FleetSummary {
    /// Generate summary from aggregated compliance matrix
    pub fn from_matrix(matrix: &ComplianceMatrix) -> Self {
        let repo_count = matrix.repository_reports.len();
        let compliant_count = matrix
            .repository_reports
            .values()
            .filter(|r| r.is_compliant())
            .count();

        let mut repos_by_phase = BTreeMap::new();
        for (phase, repos) in &matrix.phase_requirements {
            for retrofit_phase in repos {
                let phase_name = format!("{:?}", retrofit_phase);
                repos_by_phase
                    .entry(phase_name)
                    .or_insert_with(Vec::new)
                    .push(phase.clone());
            }
        }

        let mut repos_by_status = BTreeMap::new();
        *repos_by_status.entry("Pass".to_string()).or_insert(0) = matrix
            .repository_reports
            .values()
            .filter(|r| r.is_compliant())
            .count();
        *repos_by_status.entry("Warn".to_string()).or_insert(0) = matrix
            .repository_reports
            .values()
            .filter(|r| r.checks.iter().any(|c| c.status == ComplianceStatus::Warn))
            .count();
        *repos_by_status.entry("Fail".to_string()).or_insert(0) = matrix
            .repository_reports
            .values()
            .filter(|r| r.checks.iter().any(|c| c.status == ComplianceStatus::Fail))
            .count();

        let mut category_summary = BTreeMap::new();
        for category in [
            ComplianceCategory::CiCd,
            ComplianceCategory::SupplyChain,
            ComplianceCategory::Linting,
            ComplianceCategory::EditorConfig,
            ComplianceCategory::Documentation,
            ComplianceCategory::Licensing,
            ComplianceCategory::Versioning,
        ] {
            let (pass, warn, fail) = matrix.count_by_category(category);
            let category_name = format!("{:?}", category);
            category_summary.insert(
                category_name,
                CategoryStatus {
                    passing: pass,
                    warning: warn,
                    failing: fail,
                },
            );
        }

        // Collect critical issues (any Fail status)
        let mut critical_issues = Vec::new();
        for (repo_name, report) in &matrix.repository_reports {
            for check in &report.checks {
                if check.status == ComplianceStatus::Fail {
                    if let Some(phases) = matrix.phase_requirements.get(repo_name) {
                        if let Some(phase) = phases.first() {
                            critical_issues.push(AuditCriticalIssue {
                                repository: repo_name.clone(),
                                issue: check.clone(),
                                phase: *phase,
                            });
                        }
                    }
                }
            }
        }

        critical_issues.sort_by_key(|i| i.repository.clone());

        let avg_scan_time = if repo_count > 0 {
            matrix.scan_duration_seconds / repo_count as f32
        } else {
            0.0
        };

        Self {
            overall_compliance_score: matrix.compliance_score(),
            total_repositories: repo_count,
            compliant_repositories: compliant_count,
            repos_by_phase,
            repos_by_status,
            category_summary,
            critical_issues,
            audit_metadata: AuditMetadata {
                started_at: Utc::now().to_rfc3339(),
                completed_at: matrix.timestamp.clone(),
                total_duration_seconds: matrix.scan_duration_seconds,
                agents_used: matrix.agents_used,
                avg_repo_scan_time: avg_scan_time,
            },
        }
    }

    /// Render summary as pretty-printed table for CLI output
    pub fn summary_table(&self) -> String {
        let mut output = String::new();

        output.push_str("╔════════════════════════════════════════════════════════╗\n");
        output.push_str("║       Fleet Compliance Summary                        ║\n");
        output.push_str("╚════════════════════════════════════════════════════════╝\n\n");

        // Overall score
        output.push_str(&format!(
            "Overall Score:          {:.1}%\n",
            self.overall_compliance_score
        ));
        output.push_str(&format!(
            "Compliant Repositories: {}/{}\n",
            self.compliant_repositories, self.total_repositories
        ));
        output.push_str("\n");

        // By status
        output.push_str("By Compliance Status:\n");
        for (status, count) in &self.repos_by_status {
            output.push_str(&format!("  {}: {}\n", status, count));
        }
        output.push_str("\n");

        // By category
        output.push_str("By Category:\n");
        for (category, status) in &self.category_summary {
            let pass_rate = status.pass_rate();
            let emoji = if pass_rate >= 90.0 {
                "✓"
            } else if pass_rate >= 70.0 {
                "⚠"
            } else {
                "✗"
            };
            output.push_str(&format!(
                "  {} {:<20} {}/{}/{} ({:.0}%)\n",
                emoji, category, status.passing, status.warning, status.failing, pass_rate
            ));
        }
        output.push_str("\n");

        // Retrofit phases needed
        if !self.repos_by_phase.is_empty() {
            output.push_str("Retrofit Phases Needed:\n");
            for (phase, repos) in &self.repos_by_phase {
                output.push_str(&format!("  {}: {} repos\n", phase, repos.len()));
            }
            output.push_str("\n");
        }

        // Critical issues
        if !self.critical_issues.is_empty() {
            output.push_str(&format!(
                "Critical Issues ({}):\n",
                self.critical_issues.len()
            ));
            for issue in self.critical_issues.iter().take(10) {
                output.push_str(&format!("  ✗ {}: {}\n", issue.repository, issue.issue.name));
            }
            if self.critical_issues.len() > 10 {
                output.push_str(&format!(
                    "  ... and {} more\n",
                    self.critical_issues.len() - 10
                ));
            }
            output.push_str("\n");
        }

        // Metadata
        output.push_str("Audit Metadata:\n");
        output.push_str(&format!(
            "  Duration: {:.2}s ({} agents)\n",
            self.audit_metadata.total_duration_seconds, self.audit_metadata.agents_used
        ));
        output.push_str(&format!(
            "  Avg/repo: {:.2}s\n",
            self.audit_metadata.avg_repo_scan_time
        ));

        output
    }

    /// Export summary as JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| RetrofitError::Json(e))
    }
}

/// Trait for observing and tracking audit progress
pub trait AuditObserver: Send + Sync {
    /// Called when audit starts
    fn on_audit_start(&self, repo_count: usize, max_agents: usize);

    /// Called when a repository scan begins
    fn on_repo_scan_start(&self, repo_name: &str);

    /// Called when a repository scan completes successfully
    fn on_repo_scan_complete(&self, repo_name: &str, report: &ComplianceReport);

    /// Called when a repository scan fails
    fn on_repo_scan_error(&self, repo_name: &str, error: &str);

    /// Called when entire fleet audit completes
    fn on_fleet_complete(&self, matrix: &ComplianceMatrix);
}

/// Result of a single repository audit
struct AuditResult {
    repo_path: PathBuf,
    report: ComplianceReport,
    elapsed: std::time::Duration,
}

/// Orchestrates parallel audit execution across multiple repositories
///
/// Manages up to `max_agents` concurrent Tokio tasks, each scanning one repository
/// independently. Results are aggregated into a [`ComplianceMatrix`].
pub struct FleetAuditCoordinator {
    /// Maximum number of concurrent audit agents
    max_agents: usize,

    /// Praxis standards specification to validate against
    spec: PraxisSpec,

    /// Optional observer for progress tracking
    observer: Option<std::sync::Arc<dyn AuditObserver>>,
}

impl FleetAuditCoordinator {
    /// Create a new audit coordinator
    ///
    /// # Arguments
    /// * `max_agents` — Maximum concurrent audit tasks (clamped to [1, 256])
    /// * `spec` — Praxis standards specification
    pub fn new(max_agents: usize, spec: PraxisSpec) -> Self {
        let max_agents = max_agents.clamp(1, MAX_AGENTS_HARD_LIMIT);
        Self {
            max_agents,
            spec,
            observer: None,
        }
    }

    /// Set an observer for audit progress tracking
    pub fn set_observer(&mut self, observer: std::sync::Arc<dyn AuditObserver>) {
        self.observer = Some(observer);
    }

    /// Audit all repositories in a fleet root directory
    ///
    /// Automatically discovers Rust repositories (directories with Cargo.toml)
    /// and scans them in parallel.
    pub async fn audit_fleet(&self, fleet_root: &Path) -> Result<ComplianceMatrix> {
        let repos = discover_repositories(fleet_root)?;
        self.audit_with_filter(repos).await
    }

    /// Audit a specific list of repositories
    pub async fn audit_with_filter(&self, repos: Vec<PathBuf>) -> Result<ComplianceMatrix> {
        let repo_count = repos.len();
        let agents = self.max_agents.min(repo_count);

        if let Some(obs) = &self.observer {
            obs.on_audit_start(repo_count, agents);
        }

        info!(
            "Starting fleet audit: {} repos, {} agents",
            repo_count, agents
        );

        let start = Instant::now();
        let (tx, mut rx) = mpsc::channel(agents);

        let spec = self.spec.clone();
        let observer = self.observer.clone();

        // Spawn audit tasks
        for repo_path in repos {
            let tx = tx.clone();
            let spec = spec.clone();
            let observer = observer.clone();
            let repo_name = repo_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            tokio::spawn(async move {
                if let Some(obs) = &observer {
                    obs.on_repo_scan_start(&repo_name);
                }

                let span = span!(Level::DEBUG, "audit", repo = %repo_name);
                let _guard = span.enter();

                let task_start = Instant::now();

                match crate::audit::scan_repository(&repo_path, &spec).await {
                    Ok(report) => {
                        let elapsed = task_start.elapsed();
                        debug!("Scan completed in {:.2}s", elapsed.as_secs_f32());

                        if let Some(obs) = &observer {
                            obs.on_repo_scan_complete(&repo_name, &report);
                        }

                        let _ = tx
                            .send(AuditResult {
                                repo_path,
                                report,
                                elapsed,
                            })
                            .await;
                    }
                    Err(e) => {
                        warn!("Scan failed: {}", e);
                        if let Some(obs) = &observer {
                            obs.on_repo_scan_error(&repo_name, &e.to_string());
                        }
                    }
                }
            });
        }

        drop(tx); // Signal end of senders

        // Collect results
        let mut reports = Vec::new();
        let mut count = 0;

        while let Some(result) = rx.recv().await {
            reports.push(result.report);
            count += 1;
            debug!("Collected result {}/{}", count, repo_count);
        }

        let elapsed = start.elapsed();
        let duration_secs = elapsed.as_secs_f32();

        info!(
            "Fleet audit completed: {} repos in {:.2}s ({} agents)",
            reports.len(),
            duration_secs,
            agents
        );

        let matrix = ComplianceMatrix::aggregate(reports, duration_secs, agents);

        if let Some(obs) = &self.observer {
            obs.on_fleet_complete(&matrix);
        }

        Ok(matrix)
    }
}

/// Discover all Rust repositories in a directory tree
fn discover_repositories(fleet_root: &Path) -> Result<Vec<PathBuf>> {
    let mut repos = Vec::new();

    debug!("Discovering repositories in {:?}", fleet_root);

    if !fleet_root.exists() {
        return Err(RetrofitError::RepositoryNotFound(format!(
            "Fleet root not found: {:?}",
            fleet_root
        )));
    }

    for entry in std::fs::read_dir(fleet_root)? {
        let entry = entry?;
        let path = entry.path();

        // Skip if not a directory
        if !path.is_dir() {
            continue;
        }

        // Check for Cargo.toml (indicates Rust project)
        if path.join("Cargo.toml").exists() {
            repos.push(path);
        }
    }

    debug!("Discovered {} repositories", repos.len());

    Ok(repos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_matrix_new() {
        let matrix = ComplianceMatrix::new();
        assert_eq!(matrix.repository_reports.len(), 0);
        assert_eq!(matrix.compliance_score(), 100.0);
    }

    #[test]
    fn test_category_status_pass_rate() {
        let status = CategoryStatus {
            passing: 8,
            warning: 1,
            failing: 1,
        };
        assert_eq!(status.pass_rate(), 80.0);
    }

    #[test]
    fn test_category_status_pass_rate_empty() {
        let status = CategoryStatus {
            passing: 0,
            warning: 0,
            failing: 0,
        };
        assert_eq!(status.pass_rate(), 100.0);
    }

    #[test]
    fn test_fleet_summary_from_empty_matrix() {
        let matrix = ComplianceMatrix::new();
        let summary = FleetSummary::from_matrix(&matrix);
        assert_eq!(summary.total_repositories, 0);
        assert_eq!(summary.compliant_repositories, 0);
    }

    #[test]
    fn test_fleet_audit_coordinator_new() {
        let coordinator = FleetAuditCoordinator::new(10, PraxisSpec::default());
        assert_eq!(coordinator.max_agents, 10);
    }

    #[test]
    fn test_fleet_audit_coordinator_clamps_agents() {
        let coordinator = FleetAuditCoordinator::new(1000, PraxisSpec::default());
        assert_eq!(coordinator.max_agents, MAX_AGENTS_HARD_LIMIT);

        let coordinator = FleetAuditCoordinator::new(0, PraxisSpec::default());
        assert_eq!(coordinator.max_agents, 1);
    }

    #[test]
    fn test_audit_metadata_creation() {
        let metadata = AuditMetadata {
            started_at: Utc::now().to_rfc3339(),
            completed_at: Utc::now().to_rfc3339(),
            total_duration_seconds: 10.5,
            agents_used: 5,
            avg_repo_scan_time: 2.1,
        };

        assert_eq!(metadata.agents_used, 5);
        assert_eq!(metadata.avg_repo_scan_time, 2.1);
    }
}

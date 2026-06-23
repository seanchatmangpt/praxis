//! Fleet-wide compliance dashboard for real-time monitoring
//!
//! This module provides:
//! - Real-time compliance status aggregation across repositories
//! - Historical trend tracking for compliance scores
//! - Alert system for compliance threshold breaches
//! - JSON export for external monitoring systems (Grafana, Datadog, etc.)
//!
//! # Example
//!
//! ```ignore
//! use praxis_retrofit::compliance_dashboard::{Dashboard, DashboardConfig, AlertThreshold};
//!
//! let config = DashboardConfig::default();
//! let mut dashboard = Dashboard::new(config);
//!
//! // Add repository compliance reports
//! dashboard.add_report(compliance_report);
//!
//! // Get real-time status
//! let status = dashboard.get_fleet_status();
//!
//! // Export to JSON
//! let json = dashboard.export_json()?;
//! ```

use crate::models::{ComplianceReport, ComplianceStatus};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// Configuration for the compliance dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Minimum compliance score percentage before alerting (0-100)
    pub alert_threshold: f32,
    /// Number of historical snapshots to retain
    pub history_retention_days: i64,
    /// Enable automatic alert generation
    pub enable_alerts: bool,
    /// Slug/name for external dashboard identification
    pub dashboard_id: String,
    /// Compliance score weighting by category (0.0-1.0)
    pub category_weights: HashMap<String, f32>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert("ci-cd".to_string(), 1.0);
        weights.insert("supply-chain".to_string(), 1.2);
        weights.insert("linting".to_string(), 0.8);
        weights.insert("editor-config".to_string(), 0.5);
        weights.insert("documentation".to_string(), 0.7);
        weights.insert("licensing".to_string(), 1.0);
        weights.insert("versioning".to_string(), 0.6);

        Self {
            alert_threshold: 80.0,
            history_retention_days: 90,
            enable_alerts: true,
            dashboard_id: "praxis-compliance".to_string(),
            category_weights: weights,
        }
    }
}

/// Real-time compliance status for a single repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatus {
    /// Repository name/identifier
    pub name: String,
    /// Path to repository
    pub path: String,
    /// Current compliance score (0-100)
    pub compliance_score: f32,
    /// Overall status: Pass/Warn/Fail
    pub status: ComplianceStatus,
    /// Status per category
    pub category_status: HashMap<String, CategoryStatus>,
    /// Timestamp of last assessment
    pub last_assessed: String,
    /// Critical issues requiring immediate attention
    pub critical_issues: Vec<String>,
}

/// Status for a compliance category within a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStatus {
    /// Category name
    pub category: String,
    /// Pass/Warn/Fail status
    pub status: ComplianceStatus,
    /// Number of passing checks
    pub passed_checks: usize,
    /// Number of warning checks
    pub warning_checks: usize,
    /// Number of failing checks
    pub failing_checks: usize,
    /// Total checks in category
    pub total_checks: usize,
    /// Category score as percentage
    pub score: f32,
}

/// Fleet-wide compliance status snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetStatus {
    /// Timestamp of this snapshot
    pub timestamp: String,
    /// Fleet-wide average compliance score
    pub fleet_average_score: f32,
    /// Minimum compliance score in fleet
    pub fleet_min_score: f32,
    /// Maximum compliance score in fleet
    pub fleet_max_score: f32,
    /// Count of passing repositories
    pub passing_repos: usize,
    /// Count of repositories with warnings
    pub warning_repos: usize,
    /// Count of failing repositories
    pub failing_repos: usize,
    /// Total repositories in fleet
    pub total_repos: usize,
    /// Status by category across fleet
    pub fleet_category_summary: HashMap<String, FleetCategoryMetrics>,
    /// Repositories requiring immediate attention
    pub at_risk_repositories: Vec<String>,
}

/// Aggregated metrics for a compliance category across entire fleet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetCategoryMetrics {
    /// Category name
    pub category: String,
    /// Fleet-wide average score for this category
    pub average_score: f32,
    /// Percentage of repos passing this category
    pub pass_rate: f32,
    /// Number of repos with warnings in this category
    pub repos_with_warnings: usize,
    /// Number of repos failing this category
    pub repos_with_failures: usize,
    /// Total repos assessed in this category
    pub total_repos_assessed: usize,
}

/// Historical trend data for compliance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceTrend {
    /// Repository name
    pub repository: String,
    /// Timeline of compliance snapshots
    pub timeline: Vec<TrendPoint>,
    /// Trend direction: "improving", "stable", "declining"
    pub trend_direction: String,
    /// Slope of trend (change in score per day)
    pub trend_slope: f32,
    /// Days until alert if current trend continues (None if improving)
    pub days_to_alert: Option<i32>,
}

/// A single point in a compliance trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    /// Timestamp of assessment
    pub timestamp: String,
    /// Compliance score at this point
    pub score: f32,
    /// Status at this point
    pub status: ComplianceStatus,
}

/// Alert triggered by compliance breach
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "critical")]
    Critical,
}

/// A compliance alert notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAlert {
    /// Unique alert identifier
    pub alert_id: String,
    /// Severity level
    pub severity: AlertSeverity,
    /// Repository affected
    pub repository: String,
    /// Alert message
    pub message: String,
    /// When alert was triggered
    pub triggered_at: String,
    /// Category that triggered the alert (if applicable)
    pub category: Option<String>,
    /// Previous score
    pub previous_score: f32,
    /// Current score
    pub current_score: f32,
    /// Suggested remediation
    pub remediation_hint: Option<String>,
    /// Whether alert has been acknowledged
    pub acknowledged: bool,
}

/// Main compliance dashboard
pub struct Dashboard {
    config: DashboardConfig,
    /// Current snapshot of all repositories
    repo_status: HashMap<String, RepositoryStatus>,
    /// Historical trends indexed by repo name
    trends: HashMap<String, ComplianceTrend>,
    /// All active alerts
    alerts: Vec<ComplianceAlert>,
    /// Historical fleet snapshots
    history: Vec<(String, FleetStatus)>, // (timestamp, status)
}

impl Dashboard {
    /// Create a new dashboard with given configuration
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            repo_status: HashMap::new(),
            trends: HashMap::new(),
            alerts: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Add or update a compliance report for a repository
    pub fn add_report(&mut self, report: &ComplianceReport) -> Result<()> {
        let repo_name = &report.repository.name;
        let score = report.score();

        // Calculate category-level status
        let mut category_status = HashMap::new();
        let mut categories_by_name: HashMap<String, Vec<_>> = HashMap::new();

        for check in &report.checks {
            let cat_name = format!("{:?}", check.category).to_lowercase();
            categories_by_name
                .entry(cat_name)
                .or_insert_with(Vec::new)
                .push(check);
        }

        for (cat_name, checks) in categories_by_name {
            let passed = checks.iter().filter(|c| c.status == ComplianceStatus::Pass).count();
            let warned = checks.iter().filter(|c| c.status == ComplianceStatus::Warn).count();
            let failed = checks.iter().filter(|c| c.status == ComplianceStatus::Fail).count();
            let total = checks.len();

            let cat_score = (passed as f32 / total as f32) * 100.0;
            let cat_status = if failed > 0 {
                ComplianceStatus::Fail
            } else if warned > 0 {
                ComplianceStatus::Warn
            } else {
                ComplianceStatus::Pass
            };

            category_status.insert(
                cat_name.clone(),
                CategoryStatus {
                    category: cat_name,
                    status: cat_status,
                    passed_checks: passed,
                    warning_checks: warned,
                    failing_checks: failed,
                    total_checks: total,
                    score: cat_score,
                },
            );
        }

        // Determine critical issues
        let critical_issues: Vec<String> = report
            .checks
            .iter()
            .filter(|c| c.status == ComplianceStatus::Fail)
            .map(|c| format!("{} ({})", c.name, format!("{:?}", c.category).to_lowercase()))
            .collect();

        // Determine overall status
        let overall_status = if report.checks.iter().any(|c| c.status == ComplianceStatus::Fail) {
            ComplianceStatus::Fail
        } else if report.checks.iter().any(|c| c.status == ComplianceStatus::Warn) {
            ComplianceStatus::Warn
        } else {
            ComplianceStatus::Pass
        };

        let status = RepositoryStatus {
            name: repo_name.clone(),
            path: report.repository.path.to_string_lossy().to_string(),
            compliance_score: score,
            status: overall_status,
            category_status,
            last_assessed: report.timestamp.clone(),
            critical_issues,
        };

        // Check for alert conditions
        if self.config.enable_alerts {
            if let Some(prev_status) = self.repo_status.get(repo_name) {
                let score_change = score - prev_status.compliance_score;
                if score_change < -5.0 {
                    // Significant drop
                    self.alerts.push(ComplianceAlert {
                        alert_id: format!("alert-{}-{}", repo_name, report.timestamp),
                        severity: if score < self.config.alert_threshold {
                            AlertSeverity::Critical
                        } else {
                            AlertSeverity::Warning
                        },
                        repository: repo_name.clone(),
                        message: format!(
                            "Compliance score dropped from {:.1}% to {:.1}%",
                            prev_status.compliance_score, score
                        ),
                        triggered_at: report.timestamp.clone(),
                        category: None,
                        previous_score: prev_status.compliance_score,
                        current_score: score,
                        remediation_hint: None,
                        acknowledged: false,
                    });
                }
            }

            if score < self.config.alert_threshold {
                self.alerts.push(ComplianceAlert {
                    alert_id: format!("alert-threshold-{}-{}", repo_name, report.timestamp),
                    severity: AlertSeverity::Critical,
                    repository: repo_name.clone(),
                    message: format!(
                        "Repository below threshold: {:.1}% (threshold: {:.1}%)",
                        score, self.config.alert_threshold
                    ),
                    triggered_at: report.timestamp.clone(),
                    category: None,
                    previous_score: 0.0,
                    current_score: score,
                    remediation_hint: None,
                    acknowledged: false,
                });
            }
        }

        self.repo_status.insert(repo_name.clone(), status);

        // Update trends
        self.update_trend(repo_name, score, &report.timestamp)?;

        Ok(())
    }

    /// Update trend tracking for a repository
    fn update_trend(&mut self, repo_name: &str, score: f32, timestamp: &str) -> Result<()> {
        let trend = self
            .trends
            .entry(repo_name.to_string())
            .or_insert_with(|| ComplianceTrend {
                repository: repo_name.to_string(),
                timeline: Vec::new(),
                trend_direction: "stable".to_string(),
                trend_slope: 0.0,
                days_to_alert: None,
            });

        trend.timeline.push(TrendPoint {
            timestamp: timestamp.to_string(),
            score,
            status: if score < self.config.alert_threshold {
                ComplianceStatus::Fail
            } else if score < self.config.alert_threshold + 10.0 {
                ComplianceStatus::Warn
            } else {
                ComplianceStatus::Pass
            },
        });

        // Calculate trend (simple linear regression over last 7 points)
        if trend.timeline.len() >= 2 {
            let recent = trend.timeline.iter().rev().take(7).collect::<Vec<_>>();
            if recent.len() >= 2 {
                let n = recent.len() as f32;
                let mut sum_xy = 0.0;
                let mut sum_x = 0.0;
                let mut sum_y = 0.0;
                let mut sum_x2 = 0.0;

                for (i, point) in recent.iter().enumerate().rev() {
                    let x = i as f32;
                    sum_x += x;
                    sum_y += point.score;
                    sum_xy += x * point.score;
                    sum_x2 += x * x;
                }

                let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
                trend.trend_slope = slope;

                if slope > 1.0 {
                    trend.trend_direction = "improving".to_string();
                    trend.days_to_alert = None;
                } else if slope < -1.0 {
                    trend.trend_direction = "declining".to_string();
                    // Calculate days to alert
                    if let Some(current) = trend.timeline.last() {
                        let days_remaining = (self.config.alert_threshold - current.score) / -slope;
                        if days_remaining > 0.0 {
                            trend.days_to_alert = Some(days_remaining as i32);
                        }
                    }
                } else {
                    trend.trend_direction = "stable".to_string();
                    trend.days_to_alert = None;
                }
            }
        }

        Ok(())
    }

    /// Get current fleet-wide status
    pub fn get_fleet_status(&self) -> FleetStatus {
        let mut category_summary: HashMap<String, Vec<CategoryStatus>> = HashMap::new();

        for repo_status in self.repo_status.values() {
            for (cat_name, cat_status) in &repo_status.category_status {
                category_summary
                    .entry(cat_name.clone())
                    .or_insert_with(Vec::new)
                    .push(cat_status.clone());
            }
        }

        let fleet_category_summary = category_summary
            .into_iter()
            .map(|(cat_name, statuses)| {
                let total = statuses.len();
                let passed = statuses.iter().filter(|s| s.status == ComplianceStatus::Pass).count();
                let warned = statuses.iter().filter(|s| s.status == ComplianceStatus::Warn).count();
                let failed = statuses.iter().filter(|s| s.status == ComplianceStatus::Fail).count();
                let avg_score = statuses.iter().map(|s| s.score).sum::<f32>() / total as f32;

                (
                    cat_name.clone(),
                    FleetCategoryMetrics {
                        category: cat_name,
                        average_score: avg_score,
                        pass_rate: (passed as f32 / total as f32) * 100.0,
                        repos_with_warnings: warned,
                        repos_with_failures: failed,
                        total_repos_assessed: total,
                    },
                )
            })
            .collect();

        let scores: Vec<f32> = self.repo_status.values().map(|r| r.compliance_score).collect();
        let fleet_average_score = if scores.is_empty() {
            100.0
        } else {
            scores.iter().sum::<f32>() / scores.len() as f32
        };

        let fleet_min_score = scores
            .iter()
            .copied()
            .fold(100.0, f32::min);
        let fleet_max_score = scores
            .iter()
            .copied()
            .fold(0.0, f32::max);

        let passing_repos = self
            .repo_status
            .values()
            .filter(|r| r.status == ComplianceStatus::Pass)
            .count();
        let warning_repos = self
            .repo_status
            .values()
            .filter(|r| r.status == ComplianceStatus::Warn)
            .count();
        let failing_repos = self
            .repo_status
            .values()
            .filter(|r| r.status == ComplianceStatus::Fail)
            .count();

        let at_risk_repositories = self
            .repo_status
            .values()
            .filter(|r| r.compliance_score < (self.config.alert_threshold + 10.0))
            .map(|r| r.name.clone())
            .collect();

        let timestamp = Utc::now().to_rfc3339();

        FleetStatus {
            timestamp,
            fleet_average_score,
            fleet_min_score,
            fleet_max_score,
            passing_repos,
            warning_repos,
            failing_repos,
            total_repos: self.repo_status.len(),
            fleet_category_summary,
            at_risk_repositories,
        }
    }

    /// Get active alerts
    pub fn get_alerts(&self) -> Vec<&ComplianceAlert> {
        self.alerts.iter().filter(|a| !a.acknowledged).collect()
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> bool {
        for alert in &mut self.alerts {
            if alert.alert_id == alert_id {
                alert.acknowledged = true;
                return true;
            }
        }
        false
    }

    /// Get trend data for a repository
    pub fn get_trend(&self, repo_name: &str) -> Option<&ComplianceTrend> {
        self.trends.get(repo_name)
    }

    /// Get all trends
    pub fn get_all_trends(&self) -> Vec<&ComplianceTrend> {
        self.trends.values().collect()
    }

    /// Export dashboard data as JSON for external systems
    pub fn export_json(&self) -> Result<String> {
        let export = DashboardExport {
            version: "1.0".to_string(),
            dashboard_id: self.config.dashboard_id.clone(),
            exported_at: Utc::now().to_rfc3339(),
            fleet_status: self.get_fleet_status(),
            repositories: self.repo_status.values().cloned().collect(),
            trends: self.trends.values().cloned().collect(),
            alerts: self.get_alerts().iter().map(|a| (*a).clone()).collect(),
        };

        Ok(serde_json::to_string_pretty(&export)?)
    }

    /// Export as line-protocol format for InfluxDB/Prometheus
    pub fn export_line_protocol(&self) -> Result<String> {
        let mut lines = Vec::new();
        let timestamp_ns = Utc::now().timestamp_nanos_opt().unwrap_or(0);

        let fleet_status = self.get_fleet_status();

        // Fleet-level metrics
        lines.push(format!(
            "compliance_fleet,dashboard={{{}}} fleet_average_score={},fleet_min_score={},fleet_max_score={},passing_repos={},warning_repos={},failing_repos={} {}",
            self.config.dashboard_id,
            fleet_status.fleet_average_score,
            fleet_status.fleet_min_score,
            fleet_status.fleet_max_score,
            fleet_status.passing_repos,
            fleet_status.warning_repos,
            fleet_status.failing_repos,
            timestamp_ns
        ));

        // Per-repository metrics
        for repo in self.repo_status.values() {
            lines.push(format!(
                "compliance_repo,dashboard={},repository={} compliance_score={},status=\"{}\" {}",
                self.config.dashboard_id,
                repo.name.replace(' ', "_"),
                repo.compliance_score,
                format!("{:?}", repo.status).to_lowercase(),
                timestamp_ns
            ));
        }

        // Category metrics
        for (cat, metrics) in &fleet_status.fleet_category_summary {
            lines.push(format!(
                "compliance_category,dashboard={},category={} average_score={},pass_rate={},repos_with_warnings={},repos_with_failures={} {}",
                self.config.dashboard_id,
                cat.replace('-', "_"),
                metrics.average_score,
                metrics.pass_rate,
                metrics.repos_with_warnings,
                metrics.repos_with_failures,
                timestamp_ns
            ));
        }

        Ok(lines.join("\n"))
    }

    /// Get historical snapshots within date range
    pub fn get_history(&self) -> Vec<&FleetStatus> {
        self.history.iter().map(|(_, status)| status).collect()
    }

    /// Record current state as historical snapshot
    pub fn snapshot(&mut self) {
        let fleet_status = self.get_fleet_status();
        self.history
            .push((fleet_status.timestamp.clone(), fleet_status));
    }

    /// Clean up old history based on retention policy
    pub fn cleanup_old_history(&mut self) {
        if self.history.is_empty() {
            return;
        }

        // Keep only recent snapshots
        let cutoff = Utc::now() - Duration::days(self.config.history_retention_days);
        self.history.retain(|(timestamp_str, _)| {
            if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp_str) {
                dt.with_timezone(&Utc) > cutoff
            } else {
                true // Keep if we can't parse timestamp
            }
        });
    }
}

/// Complete dashboard export for external systems
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardExport {
    /// Export format version
    pub version: String,
    /// Dashboard identifier
    pub dashboard_id: String,
    /// When this export was generated
    pub exported_at: String,
    /// Current fleet-wide status snapshot
    pub fleet_status: FleetStatus,
    /// Status of all repositories
    pub repositories: Vec<RepositoryStatus>,
    /// Trend data for all repositories
    pub trends: Vec<ComplianceTrend>,
    /// Active alerts
    pub alerts: Vec<ComplianceAlert>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RepositoryMetadata, ComplianceItem};
    use std::path::PathBuf;

    fn create_test_report(name: &str, score_pct: f32) -> ComplianceReport {
        let passed = (score_pct as usize) / 20;
        let total = 5;
        let mut checks = Vec::new();

        for i in 0..total {
            checks.push(ComplianceItem {
                name: format!("Check {}", i),
                category: ComplianceCategory::CiCd,
                status: if i < passed {
                    ComplianceStatus::Pass
                } else if i < passed + 1 {
                    ComplianceStatus::Warn
                } else {
                    ComplianceStatus::Fail
                },
                evidence: "Test evidence".to_string(),
                remediation: None,
            });
        }

        ComplianceReport {
            repository: RepositoryMetadata {
                path: PathBuf::from("/test"),
                name: name.to_string(),
                workspace_root: PathBuf::from("/test"),
                crate_count: 1,
                has_workspace: true,
            },
            timestamp: Utc::now().to_rfc3339(),
            checks,
            score: score_pct,
            summary: "Test".to_string(),
        }
    }

    #[test]
    fn test_dashboard_creation() {
        let config = DashboardConfig::default();
        let dashboard = Dashboard::new(config);
        assert_eq!(dashboard.repo_status.len(), 0);
    }

    #[test]
    fn test_add_report_and_fleet_status() {
        let mut dashboard = Dashboard::new(DashboardConfig::default());
        let report = create_test_report("test-repo", 80.0);
        dashboard.add_report(&report).unwrap();

        assert_eq!(dashboard.repo_status.len(), 1);
        let fleet = dashboard.get_fleet_status();
        assert_eq!(fleet.total_repos, 1);
    }

    #[test]
    fn test_alert_on_threshold_breach() {
        let config = DashboardConfig {
            alert_threshold: 85.0,
            enable_alerts: true,
            ..Default::default()
        };
        let mut dashboard = Dashboard::new(config);
        let report = create_test_report("test-repo", 70.0);
        dashboard.add_report(&report).unwrap();

        let alerts = dashboard.get_alerts();
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    }

    #[test]
    fn test_export_json() {
        let mut dashboard = Dashboard::new(DashboardConfig::default());
        let report = create_test_report("test-repo", 95.0);
        dashboard.add_report(&report).unwrap();

        let json = dashboard.export_json().unwrap();
        assert!(json.contains("test-repo"));
        assert!(json.contains("fleet_status"));
    }
}

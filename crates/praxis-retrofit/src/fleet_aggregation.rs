//! Fleet-wide compliance aggregation logic
//!
//! Functions for aggregating individual repository compliance reports
//! into fleet-wide heatmaps, metrics, and health assessments.

use std::collections::HashMap;

use crate::{
    fleet_models::*,
    models::{ComplianceCategory, ComplianceReport, ComplianceStatus},
};

/// Build a compliance heatmap from multiple repository reports
pub fn build_heatmap(reports: &[ComplianceReport]) -> ComplianceHeatmap {
    let mut categories = vec![];
    let mut repositories = vec![];

    // Collect unique categories and repositories
    for report in reports {
        if !repositories.contains(&report.repository.name) {
            repositories.push(report.repository.name.clone());
        }
        for check in &report.checks {
            if !categories.contains(&check.category) {
                categories.push(check.category);
            }
        }
    }

    categories.sort();
    repositories.sort();

    let mut heatmap = ComplianceHeatmap::new(repositories, categories);

    // Populate heatmap cells
    for report in reports {
        // Clone categories to avoid borrow conflicts
        let categories_to_process = heatmap.categories.clone();
        for category in categories_to_process {
            let checks_in_category: Vec<_> = report
                .checks
                .iter()
                .filter(|c| c.category == category)
                .collect();

            let pass_count = checks_in_category
                .iter()
                .filter(|c| c.status == ComplianceStatus::Pass)
                .count();
            let warn_count = checks_in_category
                .iter()
                .filter(|c| c.status == ComplianceStatus::Warn)
                .count();
            let fail_count = checks_in_category
                .iter()
                .filter(|c| c.status == ComplianceStatus::Fail)
                .count();

            if pass_count + warn_count + fail_count > 0 {
                let cell = HeatmapCell::new(pass_count, warn_count, fail_count);
                heatmap.set_cell(report.repository.name.clone(), category, cell);
            }
        }
    }

    heatmap
}

/// Aggregate health metrics from compliance reports
pub fn aggregate_health_metrics(
    reports: &[ComplianceReport],
    timestamp: String,
) -> FleetHealthMetrics {
    let total_repositories = reports.len();

    // Aggregate check counts
    let mut total_pass = 0;
    let mut total_warn = 0;
    let mut total_fail = 0;
    let mut scores = vec![];

    for report in reports {
        let pass_count = report
            .checks
            .iter()
            .filter(|c| c.status == ComplianceStatus::Pass)
            .count();
        let warn_count = report
            .checks
            .iter()
            .filter(|c| c.status == ComplianceStatus::Warn)
            .count();
        let fail_count = report
            .checks
            .iter()
            .filter(|c| c.status == ComplianceStatus::Fail)
            .count();

        total_pass += pass_count;
        total_warn += warn_count;
        total_fail += fail_count;

        scores.push(report.score());
    }

    let total_checks = total_pass + total_warn + total_fail;
    let pass_percent = if total_checks > 0 {
        (total_pass as f32 / total_checks as f32) * 100.0
    } else {
        100.0
    };
    let warn_percent = if total_checks > 0 {
        (total_warn as f32 / total_checks as f32) * 100.0
    } else {
        0.0
    };
    let fail_percent = if total_checks > 0 {
        (total_fail as f32 / total_checks as f32) * 100.0
    } else {
        0.0
    };

    let overall_score = if total_checks > 0 {
        (total_pass as f32 / total_checks as f32) * 100.0
    } else {
        100.0
    };

    let health_rating = HealthRating::from_score(overall_score);

    // Calculate score distribution
    let score_distribution = calculate_score_distribution(&scores);

    // Category metrics
    let category_metrics = calculate_category_metrics(reports);

    FleetHealthMetrics {
        timestamp,
        total_repositories,
        overall_score,
        health_rating,
        pass_percent,
        warn_percent,
        fail_percent,
        total_checks,
        pass_count: total_pass,
        warn_count: total_warn,
        fail_count: total_fail,
        category_metrics,
        score_distribution,
    }
}

/// Calculate score distribution statistics
fn calculate_score_distribution(scores: &[f32]) -> ScoreDistribution {
    if scores.is_empty() {
        return ScoreDistribution {
            excellent_count: 0,
            good_count: 0,
            fair_count: 0,
            poor_count: 0,
            min_score: 100.0,
            max_score: 100.0,
            average_score: 100.0,
            median_score: 100.0,
        };
    }

    let mut sorted_scores = scores.to_vec();
    sorted_scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let excellent_count = scores.iter().filter(|s| **s >= 90.0).count();
    let good_count = scores.iter().filter(|s| **s >= 75.0 && **s < 90.0).count();
    let fair_count = scores.iter().filter(|s| **s >= 50.0 && **s < 75.0).count();
    let poor_count = scores.iter().filter(|s| **s < 50.0).count();

    let min_score = sorted_scores[0];
    let max_score = sorted_scores[sorted_scores.len() - 1];
    let average_score = scores.iter().sum::<f32>() / scores.len() as f32;
    let median_score = if sorted_scores.len() % 2 == 0 {
        (sorted_scores[sorted_scores.len() / 2 - 1] + sorted_scores[sorted_scores.len() / 2]) / 2.0
    } else {
        sorted_scores[sorted_scores.len() / 2]
    };

    ScoreDistribution {
        excellent_count,
        good_count,
        fair_count,
        poor_count,
        min_score,
        max_score,
        average_score,
        median_score,
    }
}

/// Calculate metrics per compliance category
fn calculate_category_metrics(reports: &[ComplianceReport]) -> HashMap<String, CategoryMetrics> {
    let mut category_stats: HashMap<ComplianceCategory, (usize, usize, usize)> = HashMap::new();

    for report in reports {
        for check in &report.checks {
            let entry = category_stats.entry(check.category).or_insert((0, 0, 0));
            match check.status {
                ComplianceStatus::Pass => entry.0 += 1,
                ComplianceStatus::Warn => entry.1 += 1,
                ComplianceStatus::Fail => entry.2 += 1,
            }
        }
    }

    let mut result = HashMap::new();
    for (category, (pass, warn, fail)) in category_stats {
        let category_name = format!("{:?}", category).to_lowercase();
        let metrics = CategoryMetrics::new(category_name.clone(), pass, warn, fail);
        result.insert(category_name, metrics);
    }

    result
}

/// Identify critical issues affecting the fleet
pub fn identify_critical_issues(reports: &[ComplianceReport]) -> Vec<FleetCriticalIssue> {
    let mut issues = vec![];
    let mut issue_counts: HashMap<String, Vec<String>> = HashMap::new();

    // Scan all reports for failing checks
    for report in reports {
        for check in &report.checks {
            if check.status == ComplianceStatus::Fail {
                let issue_type = check.name.clone();
                issue_counts
                    .entry(issue_type)
                    .or_insert_with(Vec::new)
                    .push(report.repository.name.clone());
            }
        }
    }

    // Convert to FleetCriticalIssue objects with severity based on prevalence
    for (issue_type, affected_repos) in issue_counts {
        let prevalence = affected_repos.len();
        let severity = match prevalence {
            count if count >= reports.len() / 2 => IssueSeverity::Critical,
            count if count >= reports.len() / 4 => IssueSeverity::High,
            count if count >= 2 => IssueSeverity::Medium,
            _ => IssueSeverity::Low,
        };

        issues.push(FleetCriticalIssue {
            issue_type,
            affected_repos,
            remediation: "See remediation advice in individual repository reports".to_string(),
            severity,
        });
    }

    // Sort by severity (descending) then by affected count
    issues.sort_by(|a, b| match b.severity.cmp(&a.severity) {
        std::cmp::Ordering::Equal => b.affected_repos.len().cmp(&a.affected_repos.len()),
        other => other,
    });

    issues
}

/// Find repositories with the most critical compliance issues
pub fn find_most_critical_repos(reports: &[ComplianceReport], limit: usize) -> Vec<String> {
    let mut repo_fail_counts: Vec<_> = reports
        .iter()
        .map(|r| {
            let fail_count = r
                .checks
                .iter()
                .filter(|c| c.status == ComplianceStatus::Fail)
                .count();
            (r.repository.name.clone(), fail_count)
        })
        .collect();

    repo_fail_counts.sort_by(|a, b| b.1.cmp(&a.1));
    repo_fail_counts
        .into_iter()
        .take(limit)
        .filter(|(_, count)| *count > 0)
        .map(|(name, _)| name)
        .collect()
}

/// Get compliance statistics for a specific category across all reports
pub fn get_category_compliance_rate(
    reports: &[ComplianceReport],
    category: ComplianceCategory,
) -> f32 {
    let mut total = 0;
    let mut passing = 0;

    for report in reports {
        for check in &report.checks {
            if check.category == category {
                total += 1;
                if check.status == ComplianceStatus::Pass {
                    passing += 1;
                }
            }
        }
    }

    if total == 0 {
        100.0
    } else {
        (passing as f32 / total as f32) * 100.0
    }
}

/// Get compliance statistics for a specific category by status
pub fn get_category_breakdown(
    reports: &[ComplianceReport],
    category: ComplianceCategory,
) -> (usize, usize, usize) {
    let mut pass = 0;
    let mut warn = 0;
    let mut fail = 0;

    for report in reports {
        for check in &report.checks {
            if check.category == category {
                match check.status {
                    ComplianceStatus::Pass => pass += 1,
                    ComplianceStatus::Warn => warn += 1,
                    ComplianceStatus::Fail => fail += 1,
                }
            }
        }
    }

    (pass, warn, fail)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::models::{ComplianceItem, RepositoryMetadata};

    fn create_test_report(
        name: &str,
        checks: Vec<(ComplianceCategory, ComplianceStatus)>,
    ) -> ComplianceReport {
        let metadata = RepositoryMetadata {
            path: PathBuf::from("/test"),
            name: name.to_string(),
            workspace_root: PathBuf::from("/test"),
            crate_count: 1,
            has_workspace: true,
        };

        let checks: Vec<ComplianceItem> = checks
            .into_iter()
            .enumerate()
            .map(|(i, (category, status))| ComplianceItem {
                name: format!("check-{}", i),
                category,
                status,
                evidence: "test".to_string(),
                remediation: None,
            })
            .collect();

        ComplianceReport {
            repository: metadata,
            timestamp: "2026-06-23T00:00:00Z".to_string(),
            checks,
            score: 0.0,
            summary: String::new(),
        }
    }

    #[test]
    fn test_build_heatmap() {
        let reports = vec![
            create_test_report(
                "repo-1",
                vec![
                    (ComplianceCategory::CiCd, ComplianceStatus::Pass),
                    (ComplianceCategory::Linting, ComplianceStatus::Fail),
                ],
            ),
            create_test_report(
                "repo-2",
                vec![
                    (ComplianceCategory::CiCd, ComplianceStatus::Pass),
                    (ComplianceCategory::Linting, ComplianceStatus::Pass),
                ],
            ),
        ];

        let heatmap = build_heatmap(&reports);
        assert_eq!(heatmap.repositories.len(), 2);
        assert_eq!(heatmap.categories.len(), 2);
    }

    #[test]
    fn test_calculate_score_distribution() {
        let scores = vec![95.0, 85.0, 45.0, 60.0, 40.0];
        let dist = calculate_score_distribution(&scores);

        assert_eq!(dist.excellent_count, 1);
        assert_eq!(dist.good_count, 1);
        assert_eq!(dist.fair_count, 1);
        assert_eq!(dist.poor_count, 2);
        assert_eq!(dist.min_score, 40.0);
        assert_eq!(dist.max_score, 95.0);
    }

    #[test]
    fn test_identify_critical_issues() {
        let reports = vec![
            create_test_report(
                "repo-1",
                vec![(ComplianceCategory::CiCd, ComplianceStatus::Fail)],
            ),
            create_test_report(
                "repo-2",
                vec![(ComplianceCategory::CiCd, ComplianceStatus::Fail)],
            ),
        ];

        let issues = identify_critical_issues(&reports);
        assert!(!issues.is_empty());
    }
}

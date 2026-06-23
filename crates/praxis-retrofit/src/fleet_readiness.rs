//! Phase readiness assessment for retrofit operations
//!
//! Determines which repositories are ready to begin each retrofit phase
//! based on compliance status and risk assessment.

use crate::fleet_models::*;
use crate::models::{ComplianceCategory, ComplianceReport, ComplianceStatus, RetrofitPhase, RiskLevel};

/// Assess retrofit phase readiness for a single repository
pub fn assess_repo_phase_readiness(
    report: &ComplianceReport,
    phase: RetrofitPhase,
    previous_phases_status: &[ReadinessStatus],
) -> RepositoryPhaseReadiness {
    // Check prerequisites
    let status = if !are_prerequisites_met(phase, previous_phases_status) {
        // Determine which phase is blocking
        let blocking_phase = find_blocking_phase(phase, previous_phases_status);
        ReadinessStatus::BlockedOn(blocking_phase)
    } else {
        // Check if phase is already completed
        if is_phase_completed(report, phase) {
            ReadinessStatus::Completed
        } else {
            // Assess risk
            match assess_phase_risk(report, phase) {
                RiskLevel::High => {
                    ReadinessStatus::HighRisk(get_risk_reason(report, phase))
                }
                _ => ReadinessStatus::Ready,
            }
        }
    };

    let estimated_actions = estimate_actions_for_phase(report, phase);
    let estimated_risk_level = assess_phase_risk(report, phase);

    RepositoryPhaseReadiness {
        repository_name: report.repository.name.clone(),
        phase,
        status,
        estimated_actions,
        estimated_risk_level,
    }
}

/// Assess readiness across all phases for a single repository
pub fn assess_repo_all_phases(report: &ComplianceReport) -> Vec<RepositoryPhaseReadiness> {
    let phases = vec![
        RetrofitPhase::Phase1Lints,
        RetrofitPhase::Phase2Deps,
        RetrofitPhase::Phase3Justfile,
        RetrofitPhase::Phase4Typos,
        RetrofitPhase::Phase5Docs,
    ];

    let mut results = vec![];
    let mut previous_statuses = vec![];

    for phase in phases {
        let readiness = assess_repo_phase_readiness(report, phase, &previous_statuses);
        previous_statuses.push(readiness.status.clone());
        results.push(readiness);
    }

    results
}

/// Assess fleet-wide readiness for a specific phase
pub fn assess_fleet_phase_readiness(
    reports: &[ComplianceReport],
    phase: RetrofitPhase,
    previous_phase_results: Option<&[PhaseReadinessSummary]>,
) -> PhaseReadinessSummary {
    let total_repositories = reports.len();
    let mut per_repo = vec![];
    let mut ready_count = 0;
    let mut blocked_count = 0;
    let mut high_risk_count = 0;
    let mut completed_count = 0;

    // Build previous phase status map if available
    let prev_status_map = build_previous_status_map(previous_phase_results, phase);

    for report in reports {
        let previous_statuses = prev_status_map
            .get(&report.repository.name)
            .cloned()
            .unwrap_or_default();

        let readiness = assess_repo_phase_readiness(report, phase, &previous_statuses);

        match &readiness.status {
            ReadinessStatus::Ready => ready_count += 1,
            ReadinessStatus::BlockedOn(_) => blocked_count += 1,
            ReadinessStatus::HighRisk(_) => high_risk_count += 1,
            ReadinessStatus::Completed => completed_count += 1,
        }

        per_repo.push(readiness);
    }

    PhaseReadinessSummary {
        phase,
        total_repositories,
        ready_count,
        blocked_count,
        high_risk_count,
        completed_count,
        per_repo,
    }
}

/// Assess fleet-wide readiness for all phases
pub fn assess_fleet_all_phases(reports: &[ComplianceReport]) -> Vec<PhaseReadinessSummary> {
    let phases = vec![
        RetrofitPhase::Phase1Lints,
        RetrofitPhase::Phase2Deps,
        RetrofitPhase::Phase3Justfile,
        RetrofitPhase::Phase4Typos,
        RetrofitPhase::Phase5Docs,
    ];

    let mut results = vec![];

    for phase in phases {
        let readiness = assess_fleet_phase_readiness(reports, phase, Some(&results));
        results.push(readiness);
    }

    results
}

/// Check if all prerequisites for a phase are met
fn are_prerequisites_met(phase: RetrofitPhase, previous_statuses: &[ReadinessStatus]) -> bool {
    match phase {
        RetrofitPhase::Phase1Lints => true, // No prerequisites

        RetrofitPhase::Phase2Deps => {
            // Requires Phase 1 (Lints) to be completed or ready
            if let Some(phase1_status) = previous_statuses.first() {
                matches!(
                    phase1_status,
                    ReadinessStatus::Ready | ReadinessStatus::Completed
                )
            } else {
                false
            }
        }

        RetrofitPhase::Phase3Justfile => {
            // Requires Phase 1 and 2 to be completed or ready
            if previous_statuses.len() >= 2 {
                let phase1_ok = matches!(
                    &previous_statuses[0],
                    ReadinessStatus::Ready | ReadinessStatus::Completed
                );
                let phase2_ok = matches!(
                    &previous_statuses[1],
                    ReadinessStatus::Ready | ReadinessStatus::Completed
                );
                phase1_ok && phase2_ok
            } else {
                false
            }
        }

        RetrofitPhase::Phase4Typos => {
            // Requires Phase 1, 2, and 3 to be completed or ready
            if previous_statuses.len() >= 3 {
                previous_statuses[0..3]
                    .iter()
                    .all(|s| matches!(s, ReadinessStatus::Ready | ReadinessStatus::Completed))
            } else {
                false
            }
        }

        RetrofitPhase::Phase5Docs => {
            // Requires all previous phases to be completed or ready
            if previous_statuses.len() >= 4 {
                previous_statuses
                    .iter()
                    .all(|s| matches!(s, ReadinessStatus::Ready | ReadinessStatus::Completed))
            } else {
                false
            }
        }
    }
}

/// Find which phase is blocking progress
fn find_blocking_phase(phase: RetrofitPhase, statuses: &[ReadinessStatus]) -> RetrofitPhase {
    match phase {
        RetrofitPhase::Phase2Deps => RetrofitPhase::Phase1Lints,
        RetrofitPhase::Phase3Justfile => {
            if let Some(ReadinessStatus::BlockedOn(p)) = statuses.first() {
                p.clone()
            } else {
                RetrofitPhase::Phase1Lints
            }
        }
        RetrofitPhase::Phase4Typos => {
            if let Some(ReadinessStatus::BlockedOn(p)) = statuses.get(2) {
                p.clone()
            } else {
                RetrofitPhase::Phase3Justfile
            }
        }
        RetrofitPhase::Phase5Docs => {
            if let Some(ReadinessStatus::BlockedOn(p)) = statuses.get(3) {
                p.clone()
            } else {
                RetrofitPhase::Phase4Typos
            }
        }
        _ => RetrofitPhase::Phase1Lints,
    }
}

/// Check if a phase is already completed for a repository
fn is_phase_completed(report: &ComplianceReport, phase: RetrofitPhase) -> bool {
    let required_categories = get_phase_categories(phase);
    let phase_checks: Vec<_> = report
        .checks
        .iter()
        .filter(|c| required_categories.contains(&c.category))
        .collect();

    // Phase is completed if all required checks pass
    !phase_checks.is_empty() && phase_checks.iter().all(|c| c.status == ComplianceStatus::Pass)
}

/// Assess risk level for a repository in a given phase
fn assess_phase_risk(report: &ComplianceReport, phase: RetrofitPhase) -> RiskLevel {
    let workspace_complexity = report.repository.crate_count;

    // Base risk from repository complexity
    let base_risk = match workspace_complexity {
        0..=2 => RiskLevel::Low,
        3..=5 => RiskLevel::Medium,
        _ => RiskLevel::High,
    };

    // Adjust risk based on phase-specific factors
    match phase {
        RetrofitPhase::Phase1Lints => {
            // Lints are low risk if workspace structure is sound
            if report.checks.iter().any(|c| {
                c.category == ComplianceCategory::Linting && c.status == ComplianceStatus::Fail
            }) {
                RiskLevel::High
            } else {
                base_risk
            }
        }

        RetrofitPhase::Phase2Deps => {
            // Deps are medium risk for large workspaces
            if workspace_complexity > 10 {
                RiskLevel::High
            } else {
                base_risk
            }
        }

        RetrofitPhase::Phase3Justfile => {
            // Justfile changes are low risk
            RiskLevel::Low
        }

        RetrofitPhase::Phase4Typos => {
            // Typos are very low risk
            RiskLevel::Low
        }

        RetrofitPhase::Phase5Docs => {
            // Documentation is low risk
            RiskLevel::Low
        }
    }
}

/// Get human-readable risk reason
fn get_risk_reason(report: &ComplianceReport, phase: RetrofitPhase) -> String {
    match phase {
        RetrofitPhase::Phase1Lints => {
            "Workspace lint configuration is failing; may cause build issues".to_string()
        }
        RetrofitPhase::Phase2Deps => {
            format!(
                "Large workspace ({} crates) poses risk to dependency changes",
                report.repository.crate_count
            )
        }
        _ => "Risk factors detected; review carefully".to_string(),
    }
}

/// Estimate number of actions needed for this phase
fn estimate_actions_for_phase(report: &ComplianceReport, phase: RetrofitPhase) -> usize {
    let required_categories = get_phase_categories(phase);
    report
        .checks
        .iter()
        .filter(|c| required_categories.contains(&c.category) && c.status != ComplianceStatus::Pass)
        .count()
}

/// Get compliance categories relevant to a phase
fn get_phase_categories(phase: RetrofitPhase) -> Vec<ComplianceCategory> {
    match phase {
        RetrofitPhase::Phase1Lints => vec![ComplianceCategory::Linting],
        RetrofitPhase::Phase2Deps => vec![ComplianceCategory::SupplyChain],
        RetrofitPhase::Phase3Justfile => vec![], // No specific category
        RetrofitPhase::Phase4Typos => vec![ComplianceCategory::EditorConfig],
        RetrofitPhase::Phase5Docs => vec![ComplianceCategory::Documentation],
    }
}

/// Build a map of repository names to their previous phase statuses
fn build_previous_status_map(
    previous_results: Option<&[PhaseReadinessSummary]>,
    current_phase: RetrofitPhase,
) -> std::collections::HashMap<String, Vec<ReadinessStatus>> {
    let mut map = std::collections::HashMap::new();

    if let Some(results) = previous_results {
        // Only include results that are before the current phase
        let target_phase_index = match current_phase {
            RetrofitPhase::Phase1Lints => 0,
            RetrofitPhase::Phase2Deps => 1,
            RetrofitPhase::Phase3Justfile => 2,
            RetrofitPhase::Phase4Typos => 3,
            RetrofitPhase::Phase5Docs => 4,
        };

        for result in results.iter().take(target_phase_index) {
            for repo_readiness in &result.per_repo {
                map.entry(repo_readiness.repository_name.clone())
                    .or_insert_with(Vec::new)
                    .push(repo_readiness.status.clone());
            }
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ComplianceItem, RepositoryMetadata};
    use std::path::PathBuf;

    fn create_test_report_with_categories(
        name: &str,
        checks: Vec<(ComplianceCategory, ComplianceStatus)>,
    ) -> ComplianceReport {
        let metadata = RepositoryMetadata {
            path: PathBuf::from("/test"),
            name: name.to_string(),
            workspace_root: PathBuf::from("/test"),
            crate_count: 2,
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
    fn test_phase1_has_no_prerequisites() {
        assert!(are_prerequisites_met(RetrofitPhase::Phase1Lints, &[]));
    }

    #[test]
    fn test_phase2_requires_phase1() {
        assert!(are_prerequisites_met(
            RetrofitPhase::Phase2Deps,
            &[ReadinessStatus::Ready]
        ));
        assert!(!are_prerequisites_met(
            RetrofitPhase::Phase2Deps,
            &[ReadinessStatus::BlockedOn(RetrofitPhase::Phase1Lints)]
        ));
    }

    #[test]
    fn test_assess_repo_phase_readiness() {
        let report = create_test_report_with_categories(
            "test-repo",
            vec![(ComplianceCategory::Linting, ComplianceStatus::Pass)],
        );

        let readiness = assess_repo_phase_readiness(
            &report,
            RetrofitPhase::Phase1Lints,
            &vec![],
        );

        assert_eq!(readiness.repository_name, "test-repo");
        assert_eq!(readiness.phase, RetrofitPhase::Phase1Lints);
        assert_eq!(readiness.status, ReadinessStatus::Completed); // All checks pass
    }

    #[test]
    fn test_assess_fleet_phase_readiness() {
        let reports = vec![
            create_test_report_with_categories(
                "repo-1",
                vec![(ComplianceCategory::Linting, ComplianceStatus::Pass)],
            ),
            create_test_report_with_categories(
                "repo-2",
                vec![(ComplianceCategory::Linting, ComplianceStatus::Fail)],
            ),
        ];

        let summary = assess_fleet_phase_readiness(&reports, RetrofitPhase::Phase1Lints, None);

        assert_eq!(summary.phase, RetrofitPhase::Phase1Lints);
        assert_eq!(summary.total_repositories, 2);
        assert!(summary.completed_count > 0);
    }
}

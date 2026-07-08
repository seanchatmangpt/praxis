//! Example: Using the Compliance Dashboard for fleet-wide monitoring
//!
//! This example demonstrates:
//! - Creating and configuring a dashboard
//! - Adding compliance reports
//! - Querying fleet status
//! - Handling alerts
//! - Exporting data for external systems

// Recorded lint debt (v26.7.6 verification gate) -- see src/lib.rs and
// docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(missing_docs, dead_code)]
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

use std::path::PathBuf;

use chrono::Utc;
use praxis_retrofit::{
    compliance_dashboard::{Dashboard, DashboardConfig},
    models::{
        ComplianceCategory, ComplianceItem, ComplianceReport, ComplianceStatus, RepositoryMetadata,
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Praxis Compliance Dashboard Example ===\n");

    // Step 1: Create dashboard with custom configuration
    println!("Step 1: Creating dashboard with configuration...");
    let mut config = DashboardConfig::default();
    config.alert_threshold = 85.0; // Alert if compliance drops below 85%
    config.enable_alerts = true;
    config.dashboard_id = "example-fleet".to_string();

    let mut dashboard = Dashboard::new(config);
    println!("Dashboard created: {}\n", dashboard.config.dashboard_id);

    // Step 2: Simulate compliance reports from repositories
    println!("Step 2: Adding compliance reports...");

    let repos = vec![
        ("core-lib", 95.0),
        ("utils-kit", 88.0),
        ("api-server", 75.0),
        ("cli-tools", 92.0),
        ("web-frontend", 80.0),
        ("data-processor", 87.0),
        ("auth-module", 100.0),
        ("storage-layer", 85.0),
        ("monitoring-agent", 90.0),
        ("config-service", 82.0),
    ];

    for (name, score) in &repos {
        let report = create_sample_report(name, *score);
        dashboard.add_report(&report)?;
        println!("  Added: {} (score: {:.1}%)", name, score);
    }
    println!();

    // Step 3: Query fleet-wide status
    println!("Step 3: Fleet-wide status:");
    let fleet_status = dashboard.get_fleet_status();
    println!("  Fleet Average: {:.1}%", fleet_status.fleet_average_score);
    println!("  Fleet Min:     {:.1}%", fleet_status.fleet_min_score);
    println!("  Fleet Max:     {:.1}%", fleet_status.fleet_max_score);
    println!("  Passing:       {}", fleet_status.passing_repos);
    println!("  Warnings:      {}", fleet_status.warning_repos);
    println!("  Failing:       {}", fleet_status.failing_repos);
    println!("  Total:         {}", fleet_status.total_repos);
    println!();

    // Step 4: Check for at-risk repositories
    println!("Step 4: At-risk repositories:");
    if fleet_status.at_risk_repositories.is_empty() {
        println!("  None");
    } else {
        for repo in &fleet_status.at_risk_repositories {
            if let Some(status) = dashboard.repo_status.get(repo) {
                println!("  - {}: {:.1}%", repo, status.compliance_score);
            }
        }
    }
    println!();

    // Step 5: Review alerts
    println!("Step 5: Active alerts:");
    let alerts = dashboard.get_alerts();
    if alerts.is_empty() {
        println!("  No active alerts");
    } else {
        for alert in alerts {
            println!(
                "  [{:?}] {}: {}",
                alert.severity, alert.repository, alert.message
            );
            println!(
                "    Score: {:.1}% -> {:.1}%",
                alert.previous_score, alert.current_score
            );
            if let Some(hint) = &alert.remediation_hint {
                println!("    Remedy: {}", hint);
            }
        }
    }
    println!();

    // Step 6: Category analysis
    println!("Step 6: Category-level metrics:");
    for (cat_name, metrics) in &fleet_status.fleet_category_summary {
        println!(
            "  {}: {:.1}% (pass_rate: {:.1}%, warnings: {}, failures: {})",
            cat_name,
            metrics.average_score,
            metrics.pass_rate,
            metrics.repos_with_warnings,
            metrics.repos_with_failures
        );
    }
    println!();

    // Step 7: Individual repository status
    println!("Step 7: Individual repository status:");
    for (name, _) in &repos {
        if let Some(status) = dashboard.repo_status.get(*name) {
            println!("  {} ({:.1}%)", status.name, status.compliance_score);
            println!("    Status: {:?}", status.status);
            if !status.critical_issues.is_empty() {
                println!("    Issues: {}", status.critical_issues.join(", "));
            }
        }
    }
    println!();

    // Step 8: Trend tracking
    println!("Step 8: Trend analysis:");
    for trend in dashboard.get_all_trends() {
        if !trend.timeline.is_empty() {
            let latest = &trend.timeline[trend.timeline.len() - 1];
            println!(
                "  {}: {} (slope: {:.2}, direction: {})",
                trend.repository, latest.score, trend.trend_slope, trend.trend_direction
            );
            if let Some(days) = trend.days_to_alert {
                println!("    Days to alert: {}", days);
            }
        }
    }
    println!();

    // Step 9: Take a historical snapshot
    println!("Step 9: Recording historical snapshot...");
    dashboard.snapshot();
    println!("Snapshot recorded\n");

    // Step 10: Export to JSON for external systems
    println!("Step 10: Exporting to JSON...");
    let json_export = dashboard.export_json()?;
    println!("JSON export size: {} bytes", json_export.len());

    // Save to file
    std::fs::write("/tmp/compliance-dashboard.json", &json_export)?;
    println!("Saved to: /tmp/compliance-dashboard.json\n");

    // Step 11: Export to line protocol for time-series databases
    println!("Step 11: Exporting to line protocol...");
    let line_protocol = dashboard.export_line_protocol()?;
    println!("Line protocol size: {} bytes", line_protocol.len());
    println!("First few lines:");
    for line in line_protocol.lines().take(3) {
        println!("  {}", line);
    }
    println!();

    // Step 12: Demonstrate alert acknowledgment
    println!("Step 12: Alert management:");
    let alert_ids: Vec<String> = dashboard
        .get_alerts()
        .iter()
        .map(|a| a.alert_id.clone())
        .collect();
    for alert_id in alert_ids {
        if dashboard.acknowledge_alert(&alert_id) {
            println!("Acknowledged alert: {}", alert_id);
        }
    }
    println!();

    // Summary
    println!("=== Summary ===");
    println!("Dashboard configured and populated successfully!");
    println!(
        "Fleet Status: {:.1}% average compliance",
        fleet_status.fleet_average_score
    );
    println!(
        "Compliance Distribution: {} passing, {} warnings, {} failing",
        fleet_status.passing_repos, fleet_status.warning_repos, fleet_status.failing_repos
    );

    Ok(())
}

/// Create a sample compliance report for demonstration
fn create_sample_report(name: &str, target_score: f32) -> ComplianceReport {
    // Determine how many checks should pass to achieve target score
    let total_checks = 5usize;
    let pass_count = ((target_score / 100.0) * total_checks as f32).round() as usize;

    let mut checks = Vec::new();

    // CI/CD check
    checks.push(ComplianceItem {
        name: "CI/CD Pipeline".to_string(),
        category: ComplianceCategory::CiCd,
        status: if pass_count > 0 {
            ComplianceStatus::Pass
        } else {
            ComplianceStatus::Fail
        },
        evidence: "GitHub Actions workflows present".to_string(),
        remediation: Some("Create .github/workflows/ci.yml".to_string()),
    });

    // Supply chain check
    checks.push(ComplianceItem {
        name: "Supply Chain Audit".to_string(),
        category: ComplianceCategory::SupplyChain,
        status: if pass_count > 1 {
            ComplianceStatus::Pass
        } else if pass_count > 0 {
            ComplianceStatus::Warn
        } else {
            ComplianceStatus::Fail
        },
        evidence: "deny.toml present".to_string(),
        remediation: Some("Add deny.toml configuration".to_string()),
    });

    // Linting check
    checks.push(ComplianceItem {
        name: "Workspace Lints".to_string(),
        category: ComplianceCategory::Linting,
        status: if pass_count > 2 {
            ComplianceStatus::Pass
        } else {
            ComplianceStatus::Fail
        },
        evidence: "Cargo.toml [lints] block present".to_string(),
        remediation: Some("Add [lints] configuration".to_string()),
    });

    // Editor config check
    checks.push(ComplianceItem {
        name: "Editor Config".to_string(),
        category: ComplianceCategory::EditorConfig,
        status: if pass_count > 3 {
            ComplianceStatus::Pass
        } else {
            ComplianceStatus::Warn
        },
        evidence: ".editorconfig file".to_string(),
        remediation: Some("Create .editorconfig".to_string()),
    });

    // Documentation check
    checks.push(ComplianceItem {
        name: "Contributor Guide".to_string(),
        category: ComplianceCategory::Documentation,
        status: if pass_count > 4 {
            ComplianceStatus::Pass
        } else {
            ComplianceStatus::Warn
        },
        evidence: "CONTRIBUTING.md present".to_string(),
        remediation: Some("Create CONTRIBUTING.md".to_string()),
    });

    ComplianceReport {
        repository: RepositoryMetadata {
            path: PathBuf::from(format!("/repos/{}", name)),
            name: name.to_string(),
            workspace_root: PathBuf::from("/repos"),
            crate_count: 1,
            has_workspace: true,
        },
        timestamp: Utc::now().to_rfc3339(),
        checks,
        score: target_score,
        summary: format!("Repository {} compliance assessment", name),
    }
}

//! Export fleet compliance reports in multiple formats
//!
//! Supports JSON, YAML, and Markdown table exports for fleet-wide
//! compliance reports and readiness assessments.

use crate::fleet_models::*;
use crate::Result;
use serde_json::Value;

/// Supported export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Yaml,
    Markdown,
}

impl ExportFormat {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(ExportFormat::Json),
            "yaml" | "yml" => Some(ExportFormat::Yaml),
            "markdown" | "md" => Some(ExportFormat::Markdown),
            _ => None,
        }
    }

    /// File extension for this format
    pub fn extension(&self) -> &str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Yaml => "yaml",
            ExportFormat::Markdown => "md",
        }
    }
}

/// Export a fleet compliance report in the specified format
pub fn export_fleet_report(
    report: &FleetComplianceReport,
    format: ExportFormat,
) -> Result<String> {
    match format {
        ExportFormat::Json => export_as_json(report),
        ExportFormat::Yaml => export_as_yaml(report),
        ExportFormat::Markdown => export_as_markdown(report),
    }
}

/// Export fleet report as JSON
fn export_as_json(report: &FleetComplianceReport) -> Result<String> {
    let json = serde_json::to_value(report)?;
    Ok(serde_json::to_string_pretty(&json)?)
}

/// Export fleet report as YAML
fn export_as_yaml(report: &FleetComplianceReport) -> Result<String> {
    // For YAML export, we serialize to JSON first, then convert
    // This approach works without requiring serde_yaml dependency
    let json = serde_json::to_value(report)?;
    let yaml_str = json_to_yaml(&json, 0);
    Ok(yaml_str)
}

/// Convert JSON value to YAML representation (simplified)
fn json_to_yaml(value: &Value, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    let next_indent = "  ".repeat(depth + 1);

    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            if s.contains('\n') || s.contains(':') || s.contains('"') {
                format!("|\n{}{}", next_indent, s.replace('\n', &format!("\n{}", next_indent)))
            } else {
                s.clone()
            }
        }
        Value::Array(arr) => {
            let mut result = String::new();
            for item in arr {
                result.push('\n');
                result.push_str(&next_indent);
                result.push_str("- ");
                let item_str = json_to_yaml(item, depth + 1);
                if item_str.contains('\n') {
                    result.push_str(&item_str);
                } else {
                    result.push_str(&item_str);
                }
            }
            result
        }
        Value::Object(obj) => {
            let mut result = String::new();
            for (key, val) in obj {
                result.push_str(&format!("\n{}{}: ", indent, key));
                let val_str = json_to_yaml(val, depth + 1);
                if val_str.starts_with('\n') {
                    result.push_str(&val_str);
                } else {
                    result.push_str(&val_str);
                }
            }
            result
        }
    }
}

/// Export fleet report as Markdown tables
fn export_as_markdown(report: &FleetComplianceReport) -> Result<String> {
    let mut lines = vec![];

    // Header
    lines.push(format!("# Fleet Compliance Report\n"));
    lines.push(format!("Generated: {}\n", report.timestamp));

    // Fleet Summary
    lines.push("## Fleet Summary\n".to_string());
    lines.push("| Metric | Value |".to_string());
    lines.push("|--------|-------|".to_string());
    lines.push(format!("| Total Repositories | {} |", report.reports.len()));
    lines.push(format!(
        "| Compliant Repositories | {} ({:.1}%) |",
        report.compliant_repos(),
        report.compliance_rate_percent()
    ));
    lines.push(format!(
        "| Overall Score | {:.1}% |",
        report.metrics.overall_score
    ));
    lines.push(format!(
        "| Health Rating | {} |\n",
        report.metrics.health_rating.description()
    ));

    // Overall Statistics
    lines.push("## Overall Statistics\n".to_string());
    lines.push("| Statistic | Count | Percentage |".to_string());
    lines.push("|-----------|-------|------------|".to_string());
    lines.push(format!(
        "| Passing | {} | {:.1}% |",
        report.metrics.pass_count, report.metrics.pass_percent
    ));
    lines.push(format!(
        "| Warning | {} | {:.1}% |",
        report.metrics.warn_count, report.metrics.warn_percent
    ));
    lines.push(format!(
        "| Failing | {} | {:.1}% |",
        report.metrics.fail_count, report.metrics.fail_percent
    ));
    lines.push(format!(
        "| Total | {} | 100.0% |\n",
        report.metrics.total_checks
    ));

    lines.push("### Score Distribution\n".to_string());
    lines.push("| Rating | Count |".to_string());
    lines.push("|--------|-------|".to_string());
    lines.push(format!(
        "| Excellent (90-100%) | {} |",
        report.metrics.score_distribution.excellent_count
    ));
    lines.push(format!(
        "| Good (75-89%) | {} |",
        report.metrics.score_distribution.good_count
    ));
    lines.push(format!(
        "| Fair (50-74%) | {} |",
        report.metrics.score_distribution.fair_count
    ));
    lines.push(format!(
        "| Poor (<50%) | {} |\n",
        report.metrics.score_distribution.poor_count
    ));

    lines.push(format!(
        "**Score Range:** {:.1}% - {:.1}%",
        report.metrics.score_distribution.min_score, report.metrics.score_distribution.max_score
    ));
    lines.push(format!(
        "**Average Score:** {:.1}%",
        report.metrics.score_distribution.average_score
    ));
    lines.push(format!(
        "**Median Score:** {:.1}%\n",
        report.metrics.score_distribution.median_score
    ));

    // Category Breakdown
    lines.push("## Compliance by Category\n".to_string());
    lines.push("| Category | Pass | Warn | Fail | Pass % |".to_string());
    lines.push("|----------|------|------|------|--------|".to_string());

    let mut categories: Vec<_> = report.metrics.category_metrics.values().collect();
    categories.sort_by_key(|c| &c.category_name);

    for category in categories {
        lines.push(format!(
            "| {} | {} | {} | {} | {:.1}% |",
            category.category_name,
            category.pass_count,
            category.warn_count,
            category.fail_count,
            category.pass_percent
        ));
    }
    lines.push(String::new());

    // Heatmap
    lines.push("## Repository Heatmap\n".to_string());
    lines.push("Repository vs Compliance Category Status\n".to_string());

    // Table header
    let mut header = "| Repository |".to_string();
    for category in &report.heatmap.categories {
        header.push_str(&format!(" {:?} |", category));
    }
    lines.push(header);

    // Separator
    let mut sep = "|-------------|".to_string();
    for _ in &report.heatmap.categories {
        sep.push_str("------|");
    }
    lines.push(sep);

    // Rows
    for repo in &report.heatmap.repositories {
        let mut row = format!("| {} |", repo);
        for category in &report.heatmap.categories {
            if let Some(cell) = report.heatmap.get_cell(repo, *category) {
                let icon = match cell.status {
                    crate::models::ComplianceStatus::Pass => "✓",
                    crate::models::ComplianceStatus::Warn => "⚠",
                    crate::models::ComplianceStatus::Fail => "✗",
                };
                row.push_str(&format!(" {} {:.0}% |", icon, cell.compliance_percent()));
            } else {
                row.push_str(" — |");
            }
        }
        lines.push(row);
    }
    lines.push(String::new());

    // Phase Readiness
    lines.push("## Phase Readiness Assessment\n".to_string());
    for summary in &report.phase_readiness {
        lines.push(format!("### {:?}", summary.phase));
        lines.push(format!(
            "Progress: {:.0}% | Ready: {} | Blocked: {} | High-Risk: {} | Completed: {}\n",
            summary.progress_percent(),
            summary.ready_count,
            summary.blocked_count,
            summary.high_risk_count,
            summary.completed_count
        ));

        let ready_repos: Vec<_> = summary
            .per_repo
            .iter()
            .filter(|r| r.status == ReadinessStatus::Ready)
            .map(|r| r.repository_name.as_str())
            .collect();

        if !ready_repos.is_empty() {
            lines.push(format!("**Ready to Start:** {}\n", ready_repos.join(", ")));
        }

        let blocked_repos: Vec<_> = summary
            .per_repo
            .iter()
            .filter(|r| matches!(r.status, ReadinessStatus::BlockedOn(_)))
            .collect();

        if !blocked_repos.is_empty() {
            lines.push("**Blocked Repositories:**".to_string());
            for repo in blocked_repos {
                lines.push(format!(
                    "- {} ({})",
                    repo.repository_name,
                    repo.status.description()
                ));
            }
            lines.push(String::new());
        }
    }

    // Critical Issues
    if !report.critical_issues.is_empty() {
        lines.push("## Critical Issues\n".to_string());
        for issue in &report.critical_issues {
            lines.push(format!("### {} ({:?})", issue.issue_type, issue.severity));
            lines.push(format!(
                "**Affected Repositories:** {}",
                issue.affected_repos.join(", ")
            ));
            lines.push(format!("**Remediation:** {}\n", issue.remediation));
        }
    }

    // Repository Details
    lines.push("## Repository Details\n".to_string());
    for report in &report.reports {
        lines.push(format!("### {}", report.repository.name));
        lines.push(format!("- **Score:** {:.1}%", report.score()));
        lines.push(format!(
            "- **Status:** {}",
            if report.is_compliant() { "✓ Compliant" } else { "✗ Non-Compliant" }
        ));
        lines.push(format!("- **Crates:** {}", report.repository.crate_count));
        lines.push(format!("- **Timestamp:** {}\n", report.timestamp));

        lines.push("#### Compliance Checks".to_string());
        lines.push("| Check | Category | Status | Evidence |".to_string());
        lines.push("|-------|----------|--------|----------|".to_string());

        for check in &report.checks {
            let status_icon = match check.status {
                crate::models::ComplianceStatus::Pass => "✓",
                crate::models::ComplianceStatus::Warn => "⚠",
                crate::models::ComplianceStatus::Fail => "✗",
            };
            lines.push(format!(
                "| {} | {:?} | {} | {} |",
                check.name, check.category, status_icon, check.evidence
            ));
        }
        lines.push(String::new());
    }

    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_from_str() {
        assert_eq!(ExportFormat::from_str("json"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::from_str("yaml"), Some(ExportFormat::Yaml));
        assert_eq!(ExportFormat::from_str("yml"), Some(ExportFormat::Yaml));
        assert_eq!(ExportFormat::from_str("markdown"), Some(ExportFormat::Markdown));
        assert_eq!(ExportFormat::from_str("md"), Some(ExportFormat::Markdown));
        assert_eq!(ExportFormat::from_str("invalid"), None);
    }

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Yaml.extension(), "yaml");
        assert_eq!(ExportFormat::Markdown.extension(), "md");
    }

    #[test]
    fn test_json_to_yaml_string() {
        let value = serde_json::json!("hello world");
        let yaml = json_to_yaml(&value, 0);
        assert_eq!(yaml, "hello world");
    }

    #[test]
    fn test_json_to_yaml_number() {
        let value = serde_json::json!(42);
        let yaml = json_to_yaml(&value, 0);
        assert_eq!(yaml, "42");
    }

    #[test]
    fn test_json_to_yaml_bool() {
        let value = serde_json::json!(true);
        let yaml = json_to_yaml(&value, 0);
        assert_eq!(yaml, "true");
    }
}

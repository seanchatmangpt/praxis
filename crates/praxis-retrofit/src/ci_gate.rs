//! CI/CD compliance gates for GitHub Actions integration
//!
//! Provides compliance gate logic that can be used in GitHub Actions workflows
//! to block PRs based on compliance score thresholds, generate remediation suggestions,
//! and report compliance status.

use crate::models::*;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for compliance gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateConfig {
    /// Minimum compliance score (0.0-100.0) required to pass the gate
    pub min_score: f32,
    /// Whether to block PRs if compliance drops
    pub block_on_drop: bool,
    /// Categories that are critical (must Pass, not just Warn)
    pub critical_categories: Vec<ComplianceCategory>,
    /// Enable auto-remediation suggestions in PR comments
    pub auto_remediate: bool,
    /// Enable compliance badge generation
    pub generate_badge: bool,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            min_score: 85.0,
            block_on_drop: true,
            critical_categories: vec![
                ComplianceCategory::CiCd,
                ComplianceCategory::Linting,
            ],
            auto_remediate: true,
            generate_badge: true,
        }
    }
}

/// Gate check result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GateResult {
    Pass,
    Fail,
    Warning,
}

/// Detailed gate check output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCheckOutput {
    pub gate_result: GateResult,
    pub score: f32,
    pub threshold: f32,
    pub message: String,
    pub blocking_issues: Vec<String>,
    pub warnings: Vec<String>,
    pub remediation_steps: Vec<RemediationStep>,
    pub badge_color: String,
    pub badge_label: String,
}

/// Single remediation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationStep {
    pub priority: RemediationPriority,
    pub category: ComplianceCategory,
    pub issue: String,
    pub suggestion: String,
    pub command: Option<String>,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RemediationPriority {
    #[serde(rename = "critical")]
    Critical,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "low")]
    Low,
}

/// Compliance gate engine
pub struct ComplianceGate {
    config: GateConfig,
}

impl ComplianceGate {
    /// Create a new compliance gate with default config
    pub fn new() -> Self {
        Self::with_config(GateConfig::default())
    }

    /// Create a compliance gate with custom config
    pub fn with_config(config: GateConfig) -> Self {
        Self { config }
    }

    /// Run the compliance gate check against a report
    pub async fn check(&self, report: &ComplianceReport) -> Result<GateCheckOutput> {
        let score = report.score();

        // Find blocking issues (Fail status in critical categories)
        let blocking_issues = self.find_blocking_issues(report);

        // Find warnings (Warn status or critical categories not passing)
        let warnings = self.find_warnings(report);

        // Generate remediation steps
        let remediation_steps = self.generate_remediation_steps(report);

        // Determine gate result
        let gate_result = self.determine_gate_result(score, &blocking_issues);

        // Determine badge appearance
        let (badge_color, badge_label) = self.badge_for_score(score);

        // Generate summary message
        let message = self.generate_message(score, &blocking_issues);

        Ok(GateCheckOutput {
            gate_result,
            score,
            threshold: self.config.min_score,
            message,
            blocking_issues,
            warnings,
            remediation_steps,
            badge_color,
            badge_label,
        })
    }

    /// Find issues that block the gate
    fn find_blocking_issues(&self, report: &ComplianceReport) -> Vec<String> {
        let mut issues = vec![];

        // Check minimum score
        if report.score() < self.config.min_score {
            issues.push(format!(
                "Compliance score {:.1}% is below minimum threshold {:.1}%",
                report.score(),
                self.config.min_score
            ));
        }

        // Check critical categories
        for item in &report.checks {
            if self.config.critical_categories.contains(&item.category)
                && item.status == ComplianceStatus::Fail
            {
                issues.push(format!(
                    "Critical category '{}' has failed: {}",
                    format!("{:?}", item.category),
                    item.name
                ));
            }
        }

        issues
    }

    /// Find warnings (non-blocking issues)
    fn find_warnings(&self, report: &ComplianceReport) -> Vec<String> {
        let mut warnings = vec![];

        for item in &report.checks {
            if item.status == ComplianceStatus::Warn {
                warnings.push(format!("{}: {} (recommended)", item.name, item.evidence));
            }
        }

        warnings
    }

    /// Generate remediation steps from compliance report
    fn generate_remediation_steps(&self, report: &ComplianceReport) -> Vec<RemediationStep> {
        let mut steps = vec![];

        for item in &report.checks {
            match item.status {
                ComplianceStatus::Fail => {
                    if let Some(remediation) = &item.remediation {
                        steps.push(RemediationStep {
                            priority: RemediationPriority::Critical,
                            category: item.category,
                            issue: item.name.clone(),
                            suggestion: remediation.clone(),
                            command: self.remediation_command(&item.name),
                            reference: self.reference_link(&item.category),
                        });
                    }
                }
                ComplianceStatus::Warn => {
                    if let Some(remediation) = &item.remediation {
                        steps.push(RemediationStep {
                            priority: RemediationPriority::High,
                            category: item.category,
                            issue: item.name.clone(),
                            suggestion: remediation.clone(),
                            command: self.remediation_command(&item.name),
                            reference: self.reference_link(&item.category),
                        });
                    }
                }
                ComplianceStatus::Pass => {}
            }
        }

        // Sort by priority
        steps.sort_by(|a, b| b.priority.cmp(&a.priority));
        steps
    }

    /// Determine the gate result
    fn determine_gate_result(
        &self,
        score: f32,
        blocking_issues: &[String],
    ) -> GateResult {
        if !blocking_issues.is_empty() {
            GateResult::Fail
        } else if score >= self.config.min_score {
            GateResult::Pass
        } else {
            GateResult::Warning
        }
    }

    /// Get badge color and label for a given score
    fn badge_for_score(&self, score: f32) -> (String, String) {
        if score >= 90.0 {
            ("green".to_string(), "Excellent".to_string())
        } else if score >= 75.0 {
            ("yellow".to_string(), "Good".to_string())
        } else {
            ("red".to_string(), "Needs Work".to_string())
        }
    }

    /// Generate a summary message for the gate result
    fn generate_message(&self, score: f32, blocking_issues: &[String]) -> String {
        if blocking_issues.is_empty() {
            format!(
                "Compliance score {:.1}% meets minimum threshold {:.1}%",
                score, self.config.min_score
            )
        } else {
            format!(
                "Compliance score {:.1}% is below minimum threshold {:.1}% ({} blocking issues)",
                score,
                self.config.min_score,
                blocking_issues.len()
            )
        }
    }

    /// Get remediation command for a specific issue
    fn remediation_command(&self, issue_name: &str) -> Option<String> {
        match issue_name {
            "Workspace Lints" => Some(
                "# Add [lints] block to Cargo.toml from template\npraxis-retrofit generate templates | grep -A 20 '\\[lints'"
                    .to_string()
            ),
            "Supply Chain Audit" => {
                Some("praxis-retrofit apply retrofit .".to_string())
            }
            "Spell Check" => Some("# Generate typos.toml\npraxis-retrofit generate templates | grep -A 10 typos.toml"
                .to_string()),
            "Contributor Guide" => {
                Some("# Create CONTRIBUTING.md from template\ncp template/CONTRIBUTING.md CONTRIBUTING.md"
                    .to_string())
            }
            _ => None,
        }
    }

    /// Get reference link for a compliance category
    fn reference_link(&self, category: &ComplianceCategory) -> Option<String> {
        match category {
            ComplianceCategory::CiCd => {
                Some("https://github.com/seanchatmangpt/praxis#ci-shape".to_string())
            }
            ComplianceCategory::Linting => {
                Some("https://github.com/seanchatmangpt/praxis#linting".to_string())
            }
            ComplianceCategory::SupplyChain => {
                Some("https://github.com/seanchatmangpt/praxis#supply-chain".to_string())
            }
            ComplianceCategory::EditorConfig => {
                Some("https://github.com/seanchatmangpt/praxis#editor-config".to_string())
            }
            ComplianceCategory::Documentation => {
                Some("https://github.com/seanchatmangpt/praxis#documentation".to_string())
            }
            ComplianceCategory::Licensing => {
                Some("https://github.com/seanchatmangpt/praxis#licensing".to_string())
            }
            ComplianceCategory::Versioning => {
                Some("https://github.com/seanchatmangpt/praxis#versioning".to_string())
            }
        }
    }
}

impl Default for ComplianceGate {
    fn default() -> Self {
        Self::new()
    }
}

/// Badge generator for compliance status
pub struct BadgeGenerator;

impl BadgeGenerator {
    /// Generate an SVG badge for compliance score
    pub fn generate_svg(score: f32, label: &str, color: &str) -> String {
        let score_str = format!("{:.0}%", score);
        let white = "#fff";
        let dark_gray = "#555";
        let light_gray = "#bbb";
        let medium_gray = "#999";
        let black = "#010101";

        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"160\" height=\"20\" role=\"img\" aria-label=\"compliance: {}\">\n  <title>compliance: {}</title>\n  <linearGradient id=\"s\" x2=\"0\" y2=\"100%\">\n    <stop offset=\"0\" stop-color=\"{}\" />\n    <stop offset=\"1\" stop-color=\"{}\" />\n  </linearGradient>\n  <clipPath id=\"r\">\n    <rect width=\"160\" height=\"20\" rx=\"3\" fill=\"{}\" />\n  </clipPath>\n  <g clip-path=\"url(#r)\">\n    <rect width=\"100\" height=\"20\" fill=\"{}\" />\n    <rect x=\"100\" width=\"60\" height=\"20\" fill=\"{}\" />\n    <rect width=\"160\" height=\"20\" fill=\"url(#s)\" />\n  </g>\n  <g fill=\"{}\" text-anchor=\"middle\" font-family=\"Verdana,Geneva,DejaVu Sans,sans-serif\" text-rendering=\"geometricPrecision\" font-size=\"110\">\n    <text aria-hidden=\"true\" x=\"510\" y=\"150\" fill=\"{}\" fill-opacity=\".3\" transform=\"scale(.1)\" textLength=\"900\">compliance</text>\n    <text x=\"510\" y=\"140\" transform=\"scale(.1)\" fill=\"{}\" textLength=\"900\">compliance</text>\n    <text aria-hidden=\"true\" x=\"1290\" y=\"150\" fill=\"{}\" fill-opacity=\".3\" transform=\"scale(.1)\" textLength=\"500\">{} - {}</text>\n    <text x=\"1290\" y=\"140\" transform=\"scale(.1)\" fill=\"{}\" textLength=\"500\">{} - {}</text>\n  </g>\n</svg>",
            score_str, score_str, light_gray, medium_gray, white, dark_gray, color, white, black, white, black, score_str, label, white, score_str, label
        )
    }

    /// Generate Markdown syntax to embed badge
    pub fn markdown_embed(svg_path: &str, alt_text: &str) -> String {
        format!("![{}]({})", alt_text, svg_path)
    }
}

/// Format remediation steps as Markdown for PR comments
pub fn format_remediation_markdown(steps: &[RemediationStep]) -> String {
    let mut md = String::from("## 🔧 Praxis Compliance Remediation Suggestions\n\n");

    if steps.is_empty() {
        md.push_str("✅ All compliance checks passed!\n");
        return md;
    }

    md.push_str("Your repository does not fully meet praxis compliance standards. Here are the recommended remediation steps:\n\n");

    // Group by priority
    let mut by_priority: HashMap<RemediationPriority, Vec<_>> = HashMap::new();
    for step in steps {
        by_priority
            .entry(step.priority)
            .or_insert_with(Vec::new)
            .push(step);
    }

    // Output critical issues first
    if let Some(critical) = by_priority.get(&RemediationPriority::Critical) {
        md.push_str("### 🚨 Critical Issues (Must Fix)\n\n");
        for step in critical {
            md.push_str(&format!("**{}**: {}\n\n", step.issue, step.suggestion));
            if let Some(cmd) = &step.command {
                md.push_str(&format!("```bash\n{}\n```\n\n", cmd));
            }
            if let Some(ref_link) = &step.reference {
                md.push_str(&format!("📚 [Learn more]({})\n\n", ref_link));
            }
        }
    }

    // Output high priority
    if let Some(high) = by_priority.get(&RemediationPriority::High) {
        md.push_str("### ⚠️ High Priority (Recommended)\n\n");
        for step in high {
            md.push_str(&format!("**{}**: {}\n\n", step.issue, step.suggestion));
            if let Some(cmd) = &step.command {
                md.push_str(&format!("```bash\n{}\n```\n\n", cmd));
            }
            if let Some(ref_link) = &step.reference {
                md.push_str(&format!("📚 [Learn more]({})\n\n", ref_link));
            }
        }
    }

    md.push_str("### Quick Start\n\n");
    md.push_str("1. **Review the compliance report** - See artifacts from this workflow run\n");
    md.push_str("2. **Run locally** - `praxis-retrofit audit report .` to see full analysis\n");
    md.push_str("3. **Apply corrections** - `praxis-retrofit apply retrofit .` to auto-apply fixes\n");
    md.push_str("4. **Validate** - `praxis-retrofit validate compliance .` to verify\n");
    md.push_str("5. **Push changes** - Commit and push to this PR\n\n");

    md.push_str("_Generated by Praxis Compliance Gate_\n");

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_config_default() {
        let config = GateConfig::default();
        assert_eq!(config.min_score, 85.0);
        assert!(config.block_on_drop);
        assert!(config.auto_remediate);
        assert!(config.generate_badge);
    }

    #[test]
    fn test_gate_result_pass() {
        let report = ComplianceReport {
            repository: crate::models::RepositoryMetadata {
                path: "/tmp/test".into(),
                name: "test-repo".into(),
                workspace_root: "/tmp/test".into(),
                crate_count: 1,
                has_workspace: false,
            },
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            checks: vec![ComplianceItem {
                name: "Test".to_string(),
                category: ComplianceCategory::CiCd,
                status: ComplianceStatus::Pass,
                evidence: "test".to_string(),
                remediation: None,
            }],
            score: 100.0,
            summary: String::new(),
        };

        let gate = ComplianceGate::new();
        let blocking = gate.find_blocking_issues(&report);
        assert!(blocking.is_empty());
    }

    #[test]
    fn test_gate_result_fail() {
        let report = ComplianceReport {
            repository: crate::models::RepositoryMetadata {
                path: "/tmp/test".into(),
                name: "test-repo".into(),
                workspace_root: "/tmp/test".into(),
                crate_count: 1,
                has_workspace: false,
            },
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            checks: vec![ComplianceItem {
                name: "Linting".to_string(),
                category: ComplianceCategory::Linting,
                status: ComplianceStatus::Fail,
                evidence: "test".to_string(),
                remediation: Some("Fix linting".to_string()),
            }],
            score: 50.0,
            summary: String::new(),
        };

        let gate = ComplianceGate::new();
        let blocking = gate.find_blocking_issues(&report);
        assert!(!blocking.is_empty());
    }

    #[test]
    fn test_badge_excellent() {
        let (color, label) = ComplianceGate::new().badge_for_score(95.0);
        assert_eq!(color, "green");
        assert_eq!(label, "Excellent");
    }

    #[test]
    fn test_badge_good() {
        let (color, label) = ComplianceGate::new().badge_for_score(80.0);
        assert_eq!(color, "yellow");
        assert_eq!(label, "Good");
    }

    #[test]
    fn test_badge_poor() {
        let (color, label) = ComplianceGate::new().badge_for_score(60.0);
        assert_eq!(color, "red");
        assert_eq!(label, "Needs Work");
    }

    #[test]
    fn test_remediation_markdown_format() {
        let steps = vec![RemediationStep {
            priority: RemediationPriority::Critical,
            category: ComplianceCategory::Linting,
            issue: "Missing lints".to_string(),
            suggestion: "Add [lints] to Cargo.toml".to_string(),
            command: Some("echo 'test'".to_string()),
            reference: Some("https://example.com".to_string()),
        }];

        let md = format_remediation_markdown(&steps);
        assert!(md.contains("Critical Issues"));
        assert!(md.contains("Missing lints"));
        assert!(md.contains("Add [lints] to Cargo.toml"));
    }
}

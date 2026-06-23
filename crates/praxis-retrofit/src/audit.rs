//! Repository compliance auditing

use crate::models::*;
use crate::{PraxisSpec, Result};
use std::path::Path;
use chrono::Local;

pub async fn scan_repository(repo_path: &Path, _spec: &PraxisSpec) -> Result<ComplianceReport> {
    let metadata = RepositoryMetadata {
        path: repo_path.to_path_buf(),
        name: repo_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
        workspace_root: repo_path.to_path_buf(),
        crate_count: count_crates(repo_path)?,
        has_workspace: has_workspace(repo_path)?,
    };

    let mut checks = vec![];

    // CI/CD check
    checks.push(ComplianceItem {
        name: "CI/CD Pipeline".to_string(),
        category: ComplianceCategory::CiCd,
        status: check_cicd(repo_path)?,
        evidence: "GitHub Actions workflows".to_string(),
        remediation: Some("Create .github/workflows/ci.yml".to_string()),
    });

    // deny.toml check
    checks.push(ComplianceItem {
        name: "Supply Chain Audit".to_string(),
        category: ComplianceCategory::SupplyChain,
        status: check_deny_toml(repo_path)?,
        evidence: "deny.toml present".to_string(),
        remediation: Some("Generate deny.toml template".to_string()),
    });

    // [lints] check
    checks.push(ComplianceItem {
        name: "Workspace Lints".to_string(),
        category: ComplianceCategory::Linting,
        status: check_workspace_lints(repo_path)?,
        evidence: "Cargo.toml [lints] block".to_string(),
        remediation: Some("Add [lints] workspace config".to_string()),
    });

    // .editorconfig check
    checks.push(ComplianceItem {
        name: "Editor Config".to_string(),
        category: ComplianceCategory::EditorConfig,
        status: check_editorconfig(repo_path)?,
        evidence: ".editorconfig present".to_string(),
        remediation: Some("Generate .editorconfig template".to_string()),
    });

    // typos.toml check
    checks.push(ComplianceItem {
        name: "Spell Check".to_string(),
        category: ComplianceCategory::EditorConfig,
        status: check_typos_toml(repo_path)?,
        evidence: "typos.toml present".to_string(),
        remediation: Some("Generate typos.toml template".to_string()),
    });

    // CONTRIBUTING.md check
    checks.push(ComplianceItem {
        name: "Contributor Guide".to_string(),
        category: ComplianceCategory::Documentation,
        status: check_contributing_md(repo_path)?,
        evidence: "CONTRIBUTING.md present".to_string(),
        remediation: Some("Create CONTRIBUTING.md template".to_string()),
    });

    let report = ComplianceReport {
        repository: metadata,
        timestamp: Local::now().to_rfc3339(),
        score: 0.0, // Will be calculated by the report
        checks,
        summary: String::new(),
    };

    Ok(report)
}

fn count_crates(repo_path: &Path) -> Result<usize> {
    let cargo_toml = repo_path.join("Cargo.toml");
    Ok(if cargo_toml.exists() { 1 } else { 0 })
}

fn has_workspace(repo_path: &Path) -> Result<bool> {
    let cargo_toml = repo_path.join("Cargo.toml");
    Ok(cargo_toml.exists())
}

fn check_cicd(repo_path: &Path) -> Result<ComplianceStatus> {
    let workflows = repo_path.join(".github/workflows");
    Ok(if workflows.exists() {
        ComplianceStatus::Pass
    } else {
        ComplianceStatus::Fail
    })
}

fn check_deny_toml(repo_path: &Path) -> Result<ComplianceStatus> {
    let deny = repo_path.join("deny.toml");
    Ok(if deny.exists() {
        ComplianceStatus::Pass
    } else {
        ComplianceStatus::Warn
    })
}

fn check_workspace_lints(repo_path: &Path) -> Result<ComplianceStatus> {
    let cargo_toml = repo_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Ok(ComplianceStatus::Fail);
    }

    let content = std::fs::read_to_string(&cargo_toml)?;
    Ok(if content.contains("[lints") {
        ComplianceStatus::Pass
    } else {
        ComplianceStatus::Fail
    })
}

fn check_editorconfig(repo_path: &Path) -> Result<ComplianceStatus> {
    let editorconfig = repo_path.join(".editorconfig");
    Ok(if editorconfig.exists() {
        ComplianceStatus::Pass
    } else {
        ComplianceStatus::Warn
    })
}

fn check_typos_toml(repo_path: &Path) -> Result<ComplianceStatus> {
    let typos = repo_path.join("typos.toml");
    Ok(if typos.exists() {
        ComplianceStatus::Pass
    } else {
        ComplianceStatus::Warn
    })
}

fn check_contributing_md(repo_path: &Path) -> Result<ComplianceStatus> {
    let contributing = repo_path.join("CONTRIBUTING.md");
    Ok(if contributing.exists() {
        ComplianceStatus::Pass
    } else {
        ComplianceStatus::Warn
    })
}

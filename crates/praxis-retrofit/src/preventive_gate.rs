//! Preventive gate: Anti-regression validation for praxis standards
//!
//! This module provides compile-time and runtime checks to prevent drift from
//! praxis specifications. It validates:
//!
//! - Versioning compliance (CalVer YY.M.patch)
//! - License uniformity (MIT OR Apache-2.0)
//! - Lint configuration (unsafe_code forbid, clippy deny-todo, etc.)
//! - MSRV consistency (minimum 1.82)
//! - Disallowed patterns (dbg!, todo!, unimplemented!)
//! - Backup file absence
//! - License file presence
//!
//! # Example
//!
//! ```no_run
//! use praxis_retrofit::preventive_gate::{GateValidator, ValidationStatus};
//! use std::path::Path;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let validator = GateValidator::new();
//! let results = validator.validate_cargo_toml(Path::new("Cargo.toml"))?;
//!
//! for result in results {
//!     match result.status {
//!         ValidationStatus::Pass => println!("✓ {}", result.message),
//!         ValidationStatus::Warn => eprintln!("⚠ {}", result.message),
//!         ValidationStatus::Fail => eprintln!("✗ {}", result.message),
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use std::{collections::HashMap, path::Path};

/// Validation result status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationStatus {
    /// Validation passed
    Pass,
    /// Non-blocking warning
    Warn,
    /// Critical failure
    Fail,
}

/// Single validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub status: ValidationStatus,
    pub category: ValidateCategory,
    pub check_name: String,
    pub message: String,
    pub remediation: Option<String>,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidateCategory {
    Versioning,
    Licensing,
    Linting,
    Msrv,
    Patterns,
    Files,
    Workspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Comprehensive gate validator
pub struct GateValidator {
    allowed_licenses: Vec<String>,
    min_msrv: String,
    house_defaults: HouseDefaults,
}

struct HouseDefaults {
    calver_pattern: String,
    dual_license: String,
    min_msrv: String,
    required_lints: Vec<String>,
    denied_macros: Vec<String>,
}

impl GateValidator {
    /// Create a new validator with house defaults
    pub fn new() -> Self {
        Self {
            allowed_licenses: vec![
                "MIT OR Apache-2.0".to_string(),
                "MIT".to_string(),
                "Apache-2.0".to_string(),
            ],
            min_msrv: "1.82".to_string(),
            house_defaults: HouseDefaults {
                calver_pattern: r"^\d{2}\.\d+\.\d+$".to_string(),
                dual_license: "MIT OR Apache-2.0".to_string(),
                min_msrv: "1.82".to_string(),
                required_lints: vec![
                    "unsafe_code = \"forbid\"".to_string(),
                    "todo = \"deny\"".to_string(),
                    "unimplemented = \"deny\"".to_string(),
                    "dbg_macro = \"deny\"".to_string(),
                ],
                denied_macros: vec![
                    "dbg!".to_string(),
                    "todo!".to_string(),
                    "unimplemented!".to_string(),
                ],
            },
        }
    }

    /// Validate a Cargo.toml file against praxis standards
    pub fn validate_cargo_toml(&self, path: &Path) -> anyhow::Result<Vec<ValidationResult>> {
        let content = std::fs::read_to_string(path)?;
        let toml: toml::Table = toml::from_str(&content)?;

        let mut results = Vec::new();

        // Check if this is a workspace
        let is_workspace = toml.contains_key("workspace");

        if !is_workspace {
            // Single-crate checks
            results.extend(self.validate_version(&toml));
            results.extend(self.validate_license(&toml));
            results.extend(self.validate_msrv(&toml));
            results.extend(self.validate_lints(&toml));
        } else {
            // Workspace checks
            results.extend(self.validate_workspace_lints(&toml));
            results.extend(self.validate_workspace_members(&toml));
        }

        Ok(results)
    }

    /// Validate CalVer versioning (YY.M.patch)
    fn validate_version(&self, toml: &toml::Table) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        if let Some(version) =
            toml.get("package").and_then(|p| p.get("version")).and_then(|v| v.as_str())
        {
            if self.is_valid_calver(version) {
                results.push(ValidationResult {
                    status: ValidationStatus::Pass,
                    category: ValidateCategory::Versioning,
                    check_name: "CalVer format".to_string(),
                    message: format!("Version {} matches CalVer YY.M.patch", version),
                    remediation: None,
                    severity: Severity::Info,
                });
            } else {
                results.push(ValidationResult {
                    status: ValidationStatus::Fail,
                    category: ValidateCategory::Versioning,
                    check_name: "CalVer format".to_string(),
                    message: format!(
                        "Version '{}' does not match CalVer YY.M.patch format (e.g., 26.6.0)",
                        version
                    ),
                    remediation: Some(format!("Update version to format YY.M.patch, e.g., 26.6.0")),
                    severity: Severity::Critical,
                });
            }
        } else {
            results.push(ValidationResult {
                status: ValidationStatus::Warn,
                category: ValidateCategory::Versioning,
                check_name: "Version declared".to_string(),
                message: "No version field found in [package]".to_string(),
                remediation: Some("Add: version = \"26.6.0\"".to_string()),
                severity: Severity::Warning,
            });
        }

        results
    }

    /// Validate license compliance
    fn validate_license(&self, toml: &toml::Table) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        if let Some(license) =
            toml.get("package").and_then(|p| p.get("license")).and_then(|l| l.as_str())
        {
            let is_preferred = license == &self.house_defaults.dual_license;
            let is_allowed = self.allowed_licenses.contains(&license.to_string());

            if is_preferred {
                results.push(ValidationResult {
                    status: ValidationStatus::Pass,
                    category: ValidateCategory::Licensing,
                    check_name: "Dual license".to_string(),
                    message: format!("License is {} (house standard)", license),
                    remediation: None,
                    severity: Severity::Info,
                });
            } else if is_allowed {
                results.push(ValidationResult {
                    status: ValidationStatus::Warn,
                    category: ValidateCategory::Licensing,
                    check_name: "License variant".to_string(),
                    message: format!(
                        "License '{}' is allowed but prefer 'MIT OR Apache-2.0'",
                        license
                    ),
                    remediation: Some(
                        "Consider changing to: license = \"MIT OR Apache-2.0\"".to_string(),
                    ),
                    severity: Severity::Warning,
                });
            } else {
                results.push(ValidationResult {
                    status: ValidationStatus::Fail,
                    category: ValidateCategory::Licensing,
                    check_name: "License compliance".to_string(),
                    message: format!("License '{}' is not in allowed list", license),
                    remediation: Some("Change to: license = \"MIT OR Apache-2.0\"".to_string()),
                    severity: Severity::Error,
                });
            }
        } else {
            results.push(ValidationResult {
                status: ValidationStatus::Warn,
                category: ValidateCategory::Licensing,
                check_name: "License declared".to_string(),
                message: "No license field found in [package]".to_string(),
                remediation: Some("Add: license = \"MIT OR Apache-2.0\"".to_string()),
                severity: Severity::Warning,
            });
        }

        results
    }

    /// Validate MSRV (Minimum Supported Rust Version)
    fn validate_msrv(&self, toml: &toml::Table) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        if let Some(msrv) =
            toml.get("package").and_then(|p| p.get("rust-version")).and_then(|v| v.as_str())
        {
            if self.is_msrv_compatible(msrv) {
                results.push(ValidationResult {
                    status: ValidationStatus::Pass,
                    category: ValidateCategory::Msrv,
                    check_name: "MSRV minimum".to_string(),
                    message: format!("MSRV {} meets house minimum (1.82)", msrv),
                    remediation: None,
                    severity: Severity::Info,
                });
            } else {
                results.push(ValidationResult {
                    status: ValidationStatus::Fail,
                    category: ValidateCategory::Msrv,
                    check_name: "MSRV compliance".to_string(),
                    message: format!("MSRV {} is below house minimum (1.82)", msrv),
                    remediation: Some("Update to: rust-version = \"1.82\"".to_string()),
                    severity: Severity::Error,
                });
            }
        } else {
            results.push(ValidationResult {
                status: ValidationStatus::Warn,
                category: ValidateCategory::Msrv,
                check_name: "MSRV declared".to_string(),
                message: "No rust-version field found in [package]".to_string(),
                remediation: Some("Add: rust-version = \"1.82\"".to_string()),
                severity: Severity::Warning,
            });
        }

        results
    }

    /// Validate lint configuration
    fn validate_lints(&self, toml: &toml::Table) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        let has_lints = toml.contains_key("lints");
        let inherits_workspace = toml
            .get("lints")
            .and_then(|l| l.get("workspace"))
            .and_then(|w| w.as_bool())
            .unwrap_or(false);

        if !has_lints {
            results.push(ValidationResult {
                status: ValidationStatus::Warn,
                category: ValidateCategory::Linting,
                check_name: "Lints configuration".to_string(),
                message: "No [lints] configuration found".to_string(),
                remediation: Some(
                    "Add [lints] with workspace = true or define inline lints".to_string(),
                ),
                severity: Severity::Warning,
            });
        } else if inherits_workspace {
            results.push(ValidationResult {
                status: ValidationStatus::Pass,
                category: ValidateCategory::Linting,
                check_name: "Lints inheritance".to_string(),
                message: "Lints inherited from workspace".to_string(),
                remediation: None,
                severity: Severity::Info,
            });
        } else {
            // Check inline lints for required entries
            if let Some(lints) = toml.get("lints") {
                self.validate_inline_lints(&mut results, lints);
            }
        }

        results
    }

    /// Validate inline lint definitions
    fn validate_inline_lints(&self, results: &mut Vec<ValidationResult>, lints: &toml::Value) {
        let required_checks = vec![
            ("unsafe_code", "forbid", ValidateCategory::Linting),
            ("todo", "deny", ValidateCategory::Linting),
            ("unimplemented", "deny", ValidateCategory::Linting),
            ("dbg_macro", "deny", ValidateCategory::Linting),
        ];

        for (lint_name, expected_level, category) in required_checks {
            let found = lints
                .get("rust")
                .and_then(|r| r.get(lint_name))
                .or_else(|| lints.get("clippy").and_then(|c| c.get(lint_name)))
                .and_then(|v| v.as_str());

            if let Some(level) = found {
                if level == expected_level {
                    results.push(ValidationResult {
                        status: ValidationStatus::Pass,
                        category,
                        check_name: format!("{} level", lint_name),
                        message: format!("{} = \"{}\"", lint_name, level),
                        remediation: None,
                        severity: Severity::Info,
                    });
                } else {
                    results.push(ValidationResult {
                        status: ValidationStatus::Warn,
                        category,
                        check_name: format!("{} level", lint_name),
                        message: format!(
                            "{} = \"{}\" but house standard requires \"{}\"",
                            lint_name, level, expected_level
                        ),
                        remediation: Some(format!(
                            "Change to: {} = \"{}\"",
                            lint_name, expected_level
                        )),
                        severity: Severity::Warning,
                    });
                }
            } else {
                results.push(ValidationResult {
                    status: ValidationStatus::Warn,
                    category,
                    check_name: format!("{} configured", lint_name),
                    message: format!("{} not configured in [lints]", lint_name),
                    remediation: Some(format!(
                        "Add to [lints.rust] or [lints.clippy]: {} = \"{}\"",
                        lint_name, expected_level
                    )),
                    severity: Severity::Warning,
                });
            }
        }
    }

    /// Validate workspace lints inheritance
    fn validate_workspace_lints(&self, toml: &toml::Table) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        if let Some(workspace) = toml.get("workspace") {
            if workspace.get("lints").is_some() {
                results.push(ValidationResult {
                    status: ValidationStatus::Pass,
                    category: ValidateCategory::Linting,
                    check_name: "Workspace lints".to_string(),
                    message: "[workspace.lints] configured".to_string(),
                    remediation: None,
                    severity: Severity::Info,
                });
            } else {
                results.push(ValidationResult {
                    status: ValidationStatus::Warn,
                    category: ValidateCategory::Linting,
                    check_name: "Workspace lints".to_string(),
                    message: "[workspace.lints] not found".to_string(),
                    remediation: Some(
                        "Add [workspace.lints] to define shared lint rules".to_string(),
                    ),
                    severity: Severity::Warning,
                });
            }
        }

        results
    }

    /// Validate workspace members configuration
    fn validate_workspace_members(&self, toml: &toml::Table) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        if let Some(workspace) = toml.get("workspace") {
            if let Some(members) = workspace.get("members") {
                results.push(ValidationResult {
                    status: ValidationStatus::Pass,
                    category: ValidateCategory::Workspace,
                    check_name: "Workspace members".to_string(),
                    message: format!(
                        "Workspace has {} members",
                        members.as_array().map(|a| a.len()).unwrap_or(0)
                    ),
                    remediation: None,
                    severity: Severity::Info,
                });
            }
        }

        results
    }

    /// Check for disallowed patterns in Rust source files
    pub fn validate_rust_patterns(&self, path: &Path) -> anyhow::Result<Vec<ValidationResult>> {
        let mut results = Vec::new();

        // Walk through Rust files
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            let file_path = entry.path();

            // Skip test/example directories (these are more lenient)
            if file_path.components().any(|c| {
                matches!(c.as_os_str().to_string_lossy().as_ref(), "tests" | "examples" | "benches")
            }) {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(file_path) {
                for (line_num, line) in content.lines().enumerate() {
                    for macro_name in &self.house_defaults.denied_macros {
                        if line.contains(macro_name) && !line.trim_start().starts_with("//") {
                            results.push(ValidationResult {
                                status: ValidationStatus::Fail,
                                category: ValidateCategory::Patterns,
                                check_name: format!("{} macro", macro_name),
                                message: format!(
                                    "{}:{}: {} macro detected",
                                    file_path.display(),
                                    line_num + 1,
                                    macro_name
                                ),
                                remediation: Some(format!(
                                    "Remove {} from production code",
                                    macro_name
                                )),
                                severity: Severity::Error,
                            });
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Check for required files (LICENSE-MIT, LICENSE-APACHE, etc.)
    pub fn validate_required_files(
        &self,
        repo_root: &Path,
    ) -> anyhow::Result<Vec<ValidationResult>> {
        let mut results = Vec::new();

        let required_files = vec![
            ("LICENSE-MIT", "MIT license file"),
            ("LICENSE-APACHE", "Apache 2.0 license file"),
            ("Cargo.toml", "Cargo.toml manifest"),
            ("rust-toolchain.toml", "Rust toolchain specification"),
            ("rustfmt.toml", "rustfmt configuration"),
        ];

        for (filename, description) in required_files {
            let path = repo_root.join(filename);
            if path.exists() {
                results.push(ValidationResult {
                    status: ValidationStatus::Pass,
                    category: ValidateCategory::Files,
                    check_name: format!("{} present", filename),
                    message: format!("✓ {} present", description),
                    remediation: None,
                    severity: Severity::Info,
                });
            } else {
                results.push(ValidationResult {
                    status: ValidationStatus::Warn,
                    category: ValidateCategory::Files,
                    check_name: format!("{} present", filename),
                    message: format!("✗ {} missing", description),
                    remediation: Some(format!("Add {} to repository root", filename)),
                    severity: Severity::Warning,
                });
            }
        }

        // Check for backup files (should NOT exist)
        let backup_pattern = repo_root.join("**/*.rs.backup");
        for entry in glob::glob(&backup_pattern.to_string_lossy()).into_iter().flatten() {
            if let Ok(path) = entry {
                results.push(ValidationResult {
                    status: ValidationStatus::Fail,
                    category: ValidateCategory::Files,
                    check_name: "No backup files".to_string(),
                    message: format!("✗ Backup file should not exist: {}", path.display()),
                    remediation: Some("Delete: find . -name '*.rs.backup' -delete".to_string()),
                    severity: Severity::Error,
                });
            }
        }

        Ok(results)
    }

    // ─────────────────────────────────────────────────────────────────────

    // Helper methods

    /// Check if version string is valid CalVer (YY.M.patch)
    fn is_valid_calver(&self, version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }

        // Parse as integers
        for part in &parts {
            if part.parse::<u32>().is_err() {
                return false;
            }
        }

        // YY should be 2 digits (00-99 represents 2000-2099)
        let yy = parts[0].parse::<u32>().unwrap_or(0);
        if yy > 99 {
            return false;
        }

        true
    }

    /// Check if MSRV is compatible with house minimum
    fn is_msrv_compatible(&self, msrv: &str) -> bool {
        let parts: Vec<&str> = msrv.split('.').collect();
        if parts.len() < 2 {
            return false;
        }

        let declared_major = parts[0].parse::<u32>().unwrap_or(0);
        let declared_minor = parts[1].parse::<u32>().unwrap_or(0);

        let min_parts: Vec<&str> = self.min_msrv.split('.').collect();
        let min_major = min_parts[0].parse::<u32>().unwrap_or(1);
        let min_minor = min_parts[1].parse::<u32>().unwrap_or(82);

        (declared_major, declared_minor) >= (min_major, min_minor)
    }
}

impl Default for GateValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Comprehensive gate report
// ─────────────────────────────────────────────────────────────────────────

/// Summary of all validation results
pub struct GateReport {
    pub results: Vec<ValidationResult>,
    pub timestamp: String,
}

impl GateReport {
    /// Create a new report from validation results
    pub fn new(results: Vec<ValidationResult>) -> Self {
        Self { results, timestamp: chrono::Local::now().to_rfc3339() }
    }

    /// Check if all critical gates passed
    pub fn is_compliant(&self) -> bool {
        !self.results.iter().any(|r| r.status == ValidationStatus::Fail)
    }

    /// Count results by status
    pub fn status_counts(&self) -> (usize, usize, usize) {
        let pass = self.results.iter().filter(|r| r.status == ValidationStatus::Pass).count();
        let warn = self.results.iter().filter(|r| r.status == ValidationStatus::Warn).count();
        let fail = self.results.iter().filter(|r| r.status == ValidationStatus::Fail).count();
        (pass, warn, fail)
    }

    /// Get remediation suggestions for failures
    pub fn remediation_suggestions(&self) -> Vec<&str> {
        self.results
            .iter()
            .filter(|r| r.status == ValidationStatus::Fail)
            .filter_map(|r| r.remediation.as_deref())
            .collect()
    }

    /// Format as markdown report
    pub fn to_markdown(&self) -> String {
        let (pass, warn, fail) = self.status_counts();

        let mut report = format!(
            "# Praxis Anti-Regression Gate Report\n\n\
             **Date:** {}\n\n\
             **Status:** {}\n\n\
             | Passed | Warned | Failed |\n\
             |--------|--------|--------|\n\
             | {} | {} | {} |\n\n",
            self.timestamp,
            if self.is_compliant() { "✓ COMPLIANT" } else { "✗ NON-COMPLIANT" },
            pass,
            warn,
            fail
        );

        // Group by category
        let mut categories: HashMap<ValidateCategory, Vec<&ValidationResult>> = HashMap::new();
        for result in &self.results {
            categories.entry(result.category).or_insert_with(Vec::new).push(result);
        }

        for (category, results) in &categories {
            report.push_str(&format!("## {:?}\n\n", category));
            for result in results {
                let status_symbol = match result.status {
                    ValidationStatus::Pass => "✓",
                    ValidationStatus::Warn => "⚠",
                    ValidationStatus::Fail => "✗",
                };
                report.push_str(&format!(
                    "{} **{}**: {}\n",
                    status_symbol, result.check_name, result.message
                ));
                if let Some(remediation) = &result.remediation {
                    report.push_str(&format!("  - Fix: {}\n", remediation));
                }
            }
            report.push('\n');
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_calver() {
        let validator = GateValidator::new();
        assert!(validator.is_valid_calver("26.6.0"));
        assert!(validator.is_valid_calver("26.6.17"));
        assert!(validator.is_valid_calver("25.12.999"));
        assert!(!validator.is_valid_calver("100.6.0")); // YY > 99
        assert!(!validator.is_valid_calver("26.6")); // Missing patch
        assert!(!validator.is_valid_calver("26.6.0.0")); // Too many parts
    }

    #[test]
    fn test_msrv_compatibility() {
        let validator = GateValidator::new();
        assert!(validator.is_msrv_compatible("1.82"));
        assert!(validator.is_msrv_compatible("1.83"));
        assert!(validator.is_msrv_compatible("2.0"));
        assert!(!validator.is_msrv_compatible("1.81"));
        assert!(!validator.is_msrv_compatible("1.0"));
    }

    #[test]
    fn test_gate_report_counts() {
        let results = vec![
            ValidationResult {
                status: ValidationStatus::Pass,
                category: ValidateCategory::Versioning,
                check_name: "test".to_string(),
                message: "pass".to_string(),
                remediation: None,
                severity: Severity::Info,
            },
            ValidationResult {
                status: ValidationStatus::Fail,
                category: ValidateCategory::Licensing,
                check_name: "test".to_string(),
                message: "fail".to_string(),
                remediation: Some("fix".to_string()),
                severity: Severity::Error,
            },
        ];

        let report = GateReport::new(results);
        let (pass, warn, fail) = report.status_counts();
        assert_eq!(pass, 1);
        assert_eq!(warn, 0);
        assert_eq!(fail, 1);
    }
}

//! Repository discovery and registry for the seanchatmangpt ecosystem.
//!
//! Loads and manages metadata for all 18 surveyed repositories, enabling:
//! - Fleet-wide retrofit planning and prioritization
//! - Filtering by retrofit readiness and phase completion
//! - Risk/effort/priority-based sorting
//! - Dependency-aware sequencing
//!
//! # Format
//!
//! The registry is sourced from `repos.toml` (TOML format):
//!
//! ```toml
//! [repos.<github-slug>]
//! github_url = "https://github.com/seanchatmangpt/<repo>"
//! crate_name = "..."
//! retrofit_readiness = "ready" | "requires-prep" | "blocked"
//! retrofit_phase_complete = 0..=5
//! risk_level = "low" | "medium" | "high"
//! priority_score = 0..=100
//! ```
//!
//! # Examples
//!
//! ```rust,no_run
//! use praxis_retrofit::repo_registry::RepositoryRegistry;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let registry = RepositoryRegistry::load("repos.toml").await?;
//!
//! // List all repos ready for retrofit
//! let ready = registry.filter_by_readiness("ready");
//! println!("Ready for retrofit: {}", ready.len());
//!
//! // Sort by priority for batch processing
//! let by_priority = registry.sorted_by_priority();
//! for repo in by_priority.iter().take(5) {
//!     println!("Top priority: {} ({})", repo.name, repo.priority_score);
//! }
//!
//! // Find repos blocking others (high-adoption upstream deps)
//! let consumers = registry.downstream_consumers("clap-noun-verb");
//! println!("Downstream consumers: {:?}", consumers);
//! # Ok(())
//! # }
//! ```

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use chicago_tdd_tools::core::config::poka_yoke::{BoundedU32, PositiveUsize};
use serde::{Deserialize, Serialize};
use star_toml::{Validate, Validator, TrustedLoader, ConfigLifecycle};

use crate::models::RetrofitPhase;

/// Metadata for a single repository in the seanchatmangpt ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryEntry {
    /// GitHub repository slug (e.g., "affidavit", "clap-noun-verb")
    pub name: String,

    /// Full HTTPS GitHub URL
    pub github_url: String,

    /// Account owner (always "seanchatmangpt")
    pub github_owner: String,

    /// Local path relative to fleet root (e.g., "../affidavit")
    pub local_path: PathBuf,

    /// Primary Rust crate name (from Cargo.toml)
    pub crate_name: String,

    /// What this repo does
    pub description: String,

    /// "public" or "private"
    pub visibility: String,

    /// "single-crate" | "multi-crate" | "monorepo"
    pub workspace_type: String,

    /// Number of publishable crates
    pub crate_count: usize,

    /// Retrofit readiness: "ready" | "requires-prep" | "blocked"
    pub retrofit_readiness: String,

    /// Phases complete (0–5, where 5 = fully compliant)
    pub retrofit_phase_complete: u8,

    /// Risk level: "low" | "medium" | "high"
    pub risk_level: String,

    /// Priority score 0–100 (higher = do first)
    pub priority_score: u8,

    /// Maintenance status: "active" | "maintenance" | "experimental"
    pub maintainer_status: String,

    /// Context notes for retrofit decisions
    pub notes: String,
}

impl RepositoryEntry {
    /// Returns this repo's retrofit phase as an enum.
    pub fn retrofit_phase(&self) -> RetrofitPhase {
        match self.retrofit_phase_complete {
            0 => RetrofitPhase::Phase1Lints, // No work done yet; start at Phase 1
            1 => RetrofitPhase::Phase1Lints,
            2 => RetrofitPhase::Phase2Deps,
            3 => RetrofitPhase::Phase3Justfile,
            4 => RetrofitPhase::Phase4Typos,
            5 => RetrofitPhase::Phase5Docs,
            _ => RetrofitPhase::Phase5Docs, // Clamp to max
        }
    }

    /// Returns the next phase that needs work.
    pub fn next_phase(&self) -> Option<RetrofitPhase> {
        if self.retrofit_phase_complete >= 5 {
            None
        } else {
            Some(self.retrofit_phase())
        }
    }

    /// Checks if this repo is ready to start retrofit work.
    pub fn is_ready_for_retrofit(&self) -> bool {
        self.retrofit_readiness == "ready"
    }

    /// Checks if this repo requires preparation before retrofit.
    pub fn requires_prep(&self) -> bool {
        self.retrofit_readiness == "requires-prep"
    }

    /// Checks if retrofit is blocked (e.g., legal/license issues).
    pub fn is_blocked(&self) -> bool {
        self.retrofit_readiness == "blocked"
    }

    /// Estimates effort (person-weeks) based on size, complexity, and phase.
    /// Heuristic: crate_count * phase_remaining * risk_multiplier.
    pub fn estimated_effort_weeks(&self) -> f32 {
        let phases_remaining = (5 - self.retrofit_phase_complete) as f32;
        let risk_multiplier = match self.risk_level.as_str() {
            "low" => 1.0,
            "medium" => 1.5,
            "high" => 2.5,
            _ => 1.0,
        };
        let base_effort = self.crate_count as f32 * 0.5; // ~30 min per crate per phase
        (base_effort * phases_remaining * risk_multiplier) / 40.0 // weeks at 40h/week
    }
}

/// Complete registry of the seanchatmangpt ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRegistry {
    /// All repos keyed by name.
    repos: HashMap<String, RepositoryEntry>,

    /// Metadata about the ecosystem.
    #[serde(default)]
    pub metadata: EcosystemMetadata,
}

/// Metadata about the entire ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemMetadata {
    /// Ecosystem name (e.g., "seanchatmangpt").
    pub ecosystem_name: String,

    /// Total repos in the ecosystem.
    pub total_repos: usize,

    /// Total crates across all repos.
    pub total_crates: usize,

    /// Survey date (ISO 8601).
    pub survey_date: String,

    /// Number of agents that performed the survey.
    pub survey_agents: u8,

    /// Scope description.
    pub survey_scope: String,

    /// House MSRV (minimum supported Rust version).
    pub house_msrv: String,

    /// House edition default.
    pub house_edition: String,

    /// House toolchain version.
    pub house_toolchain: String,

    /// House license preference.
    pub house_license: String,

    /// House version scheme.
    pub house_version_scheme: String,

    /// How many repos are ready for retrofit.
    pub ready_for_retrofit: usize,

    /// How many require phase 2–3 work.
    pub requires_phase_2_or_3: usize,

    /// How many require phase 0–1 work.
    pub requires_phase_0_or_1: usize,

    /// How many are in experimental status.
    pub experimental_status: usize,

    /// Primary retrofit order (highest upstream first).
    pub primary_order: Vec<String>,

    /// Secondary retrofit order.
    pub secondary_order: Vec<String>,

    /// Tertiary retrofit order (experimental/proto).
    pub tertiary_order: Vec<String>,

    /// Repos with AGPL licenses (needs legal review).
    pub agpl_repos: Vec<String>,

    /// Repos with BSL licenses (needs legal review).
    pub bsl_repos: Vec<String>,

    /// Repos with Apache-only (consider dual licensing).
    pub apache_only_repos: Vec<String>,

    /// Repos missing LICENSE files.
    pub missing_license_files: Vec<String>,

    /// Repos with no CI at all.
    pub no_ci_repos: Vec<String>,

    /// Repos using deprecated GitHub actions.
    pub deprecated_actions_repos: Vec<String>,

    /// Repos with minimal CI (only 1–2 jobs).
    pub minimal_ci_repos: Vec<String>,

    /// How many repos lack release workflows.
    pub sparse_release_workflows: usize,

    /// Repos missing CLAUDE.md.
    pub missing_claude_md: Vec<String>,

    /// Repos missing SECURITY.md.
    pub missing_security_md: Vec<String>,
}

impl Default for EcosystemMetadata {
    fn default() -> Self {
        EcosystemMetadata {
            ecosystem_name: "seanchatmangpt".to_string(),
            total_repos: 0,
            total_crates: 0,
            survey_date: "unknown".to_string(),
            survey_agents: 0,
            survey_scope: "unknown".to_string(),
            house_msrv: "1.82".to_string(),
            house_edition: "2021".to_string(),
            house_toolchain: "1.82.0".to_string(),
            house_license: "MIT OR Apache-2.0".to_string(),
            house_version_scheme: "CalVer YY.M.patch".to_string(),
            ready_for_retrofit: 0,
            requires_phase_2_or_3: 0,
            requires_phase_0_or_1: 0,
            experimental_status: 0,
            primary_order: vec![],
            secondary_order: vec![],
            tertiary_order: vec![],
            agpl_repos: vec![],
            bsl_repos: vec![],
            apache_only_repos: vec![],
            missing_license_files: vec![],
            no_ci_repos: vec![],
            deprecated_actions_repos: vec![],
            minimal_ci_repos: vec![],
            sparse_release_workflows: 0,
            missing_claude_md: vec![],
            missing_security_md: vec![],
        }
    }
}

impl Validate for RepositoryRegistry {
    fn validate(&self, v: &mut Validator) {
        for (slug, entry) in &self.repos {
            v.field(slug, |v| {
                v.check_predicate(
                    "crate_count",
                    entry.crate_count > 0,
                    "positive_integer",
                    "crate_count must be a positive integer",
                );
                v.check_range("priority_score", entry.priority_score, 0u8..=100u8);
            });
        }
    }
}

impl ConfigLifecycle for RepositoryRegistry {
    fn normalize(&mut self) {
        let keys: Vec<String> = self.repos.keys().cloned().collect();
        for slug in keys {
            if let Some(entry) = self.repos.get_mut(&slug) {
                if entry.name.is_empty() {
                    entry.name = slug.clone();
                }
            }
        }
    }
}

/// Resolves the path of the registry file by checking the `PRAXIS_REGISTRY_PATH` env var,
/// searching parent directories up to 5 levels for `repos.toml`, and falling back to the
/// provided `fallback` path.
pub fn resolve_layered_path(fallback: &std::path::Path) -> std::path::PathBuf {
    if let Ok(env_path) = std::env::var("PRAXIS_REGISTRY_PATH") {
        if !env_path.is_empty() {
            return std::path::PathBuf::from(env_path);
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        let mut current = Some(current_dir.as_path());
        for _ in 0..=5 {
            if let Some(dir) = current {
                let candidate = dir.join("repos.toml");
                if candidate.is_file() {
                    return candidate;
                }
                current = dir.parent();
            } else {
                break;
            }
        }
    }

    fallback.to_path_buf()
}

impl RepositoryRegistry {
    /// Loads the registry from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub async fn load(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let resolved = resolve_layered_path(path.as_ref());
        TrustedLoader::new()
            .layer_file_if_exists(&resolved)
            .load_admitted::<RepositoryRegistry>()
            .map(|a| a.into_value())
            .map_err(|e| crate::RetrofitError::ConfigError(format!("Failed to load repos.toml: {}", e)))
    }

    /// Parses the registry from a TOML string and validates invariants.
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be parsed or if invariants are violated.
    pub async fn load_str(contents: &str) -> crate::Result<Self> {
        TrustedLoader::new()
            .layer_str(contents, "inline")
            .load_admitted::<RepositoryRegistry>()
            .map(|a| a.into_value())
            .map_err(|e| crate::RetrofitError::ConfigError(format!("Failed to parse repos.toml: {}", e)))
    }

    /// Returns all repositories.
    pub fn all(&self) -> Vec<&RepositoryEntry> {
        self.repos.values().collect()
    }

    /// Looks up a repository by name.
    pub fn get(&self, name: &str) -> Option<&RepositoryEntry> {
        self.repos.get(name)
    }

    /// Filters repos by retrofit readiness status.
    pub fn filter_by_readiness(&self, status: &str) -> Vec<&RepositoryEntry> {
        self.repos.values().filter(|r| r.retrofit_readiness == status).collect()
    }

    /// Filters repos by retrofit phase completion.
    pub fn filter_by_phase(&self, phase: u8) -> Vec<&RepositoryEntry> {
        self.repos.values().filter(|r| r.retrofit_phase_complete == phase).collect()
    }

    /// Filters repos by risk level.
    pub fn filter_by_risk(&self, level: &str) -> Vec<&RepositoryEntry> {
        self.repos.values().filter(|r| r.risk_level == level).collect()
    }

    /// Filters repos by maintainer status.
    pub fn filter_by_status(&self, status: &str) -> Vec<&RepositoryEntry> {
        self.repos.values().filter(|r| r.maintainer_status == status).collect()
    }

    /// Filters repos by workspace type.
    pub fn filter_by_workspace_type(&self, wtype: &str) -> Vec<&RepositoryEntry> {
        self.repos.values().filter(|r| r.workspace_type == wtype).collect()
    }

    /// Returns all repos sorted by priority score (highest first).
    pub fn sorted_by_priority(&self) -> Vec<&RepositoryEntry> {
        let mut repos: Vec<_> = self.repos.values().collect();
        repos.sort_by(|a, b| b.priority_score.cmp(&a.priority_score));
        repos
    }

    /// Returns all repos sorted by risk level (low → high → medium) and within each, by priority.
    pub fn sorted_by_risk(&self) -> Vec<&RepositoryEntry> {
        let mut repos: Vec<_> = self.repos.values().collect();
        repos.sort_by(|a, b| {
            let risk_order = |r: &str| match r {
                "low" => 0,
                "high" => 1,
                "medium" => 2,
                _ => 3,
            };
            match risk_order(&a.risk_level).cmp(&risk_order(&b.risk_level)) {
                std::cmp::Ordering::Equal => b.priority_score.cmp(&a.priority_score),
                other => other,
            }
        });
        repos
    }

    /// Returns all repos sorted by effort (ascending — easy first).
    pub fn sorted_by_effort(&self) -> Vec<(&RepositoryEntry, f32)> {
        let mut repos: Vec<_> =
            self.repos.values().map(|r| (r, r.estimated_effort_weeks())).collect();
        repos.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        repos
    }

    /// Returns repos in the recommended retrofit order (from metadata.primary_order, etc.).
    pub fn recommended_retrofit_order(&self) -> Vec<&RepositoryEntry> {
        let mut result = Vec::new();

        for order_list in &[
            &self.metadata.primary_order,
            &self.metadata.secondary_order,
            &self.metadata.tertiary_order,
        ] {
            for name in order_list.iter() {
                if let Some(repo) = self.get(name) {
                    result.push(repo);
                }
            }
        }

        result
    }

    /// Returns repos that have no CI workflows (high priority for automation).
    pub fn no_ci_repos(&self) -> Vec<&RepositoryEntry> {
        self.metadata.no_ci_repos.iter().filter_map(|name| self.get(name)).collect()
    }

    /// Returns repos with missing license files (compliance issue).
    pub fn missing_license_files(&self) -> Vec<&RepositoryEntry> {
        self.metadata.missing_license_files.iter().filter_map(|name| self.get(name)).collect()
    }

    /// Returns repos with non-standard licenses (legal coordination needed).
    pub fn non_standard_licenses(&self) -> Vec<&RepositoryEntry> {
        let mut result = Vec::new();

        for name in self
            .metadata
            .agpl_repos
            .iter()
            .chain(self.metadata.bsl_repos.iter())
            .chain(self.metadata.apache_only_repos.iter())
        {
            if let Some(repo) = self.get(name) {
                result.push(repo);
            }
        }

        result
    }

    /// Finds repos that depend on this repo (downstream consumers).
    ///
    /// This is based on a hardcoded dependency map of known patterns.
    pub fn downstream_consumers(&self, repo_name: &str) -> Vec<&RepositoryEntry> {
        let consumers = match repo_name {
            "clap-noun-verb" => vec!["affidavit", "cargo-cicd", "mac-artifact-cleaner"],
            "ggen" => vec!["clnrm", "wasm4pm-compat", "pm4py-rs", "ggen-mcp", "a2a-rs"],
            "ggen.toml" => vec![
                "affidavit",
                "ggen",
                "clnrm",
                "clap-noun-verb",
                "cargo-cicd",
                "wasm4pm-compat",
                "ggen-mcp",
                "pm4py-rs",
                "a2a-rs",
            ],
            _ => vec![],
        };

        consumers.into_iter().filter_map(|name| self.get(name)).collect()
    }

    /// Generates a summary report of ecosystem readiness.
    pub fn readiness_summary(&self) -> String {
        let ready = self.filter_by_readiness("ready").len();
        let prep = self.filter_by_readiness("requires-prep").len();
        let blocked = self.filter_by_readiness("blocked").len();

        let low_risk = self.filter_by_risk("low").len();
        let med_risk = self.filter_by_risk("medium").len();
        let high_risk = self.filter_by_risk("high").len();

        let total_effort: f32 = self.repos.values().map(|r| r.estimated_effort_weeks()).sum();

        format!(
            "Seanchatmangpt Retrofit Readiness Report\n\
             ========================================\n\
             Ready for retrofit: {} / {}\n\
             Requires preparation: {}\n\
             Blocked: {}\n\
             \n\
             Risk Distribution:\n\
             Low risk: {}\n\
             Medium risk: {}\n\
             High risk: {}\n\
             \n\
             Estimated Total Effort: {:.1} person-weeks\n",
            ready,
            self.metadata.total_repos,
            prep,
            blocked,
            low_risk,
            med_risk,
            high_risk,
            total_effort
        )
    }
}

/// Internal document structure for TOML deserialization.
#[derive(Debug, Deserialize)]
struct RegistryDocument {
    #[serde(default)]
    repos: HashMap<String, RepositoryEntry>,

    #[serde(default)]
    metadata: Option<EcosystemMetadata>,
}

impl RegistryDocument {
    fn into_registry(mut self) -> crate::Result<RepositoryRegistry> {
        // Fix repo names (key is slug, but field name should match)
        let repos = std::mem::take(&mut self.repos);
        let mut fixed_repos = HashMap::new();
        for (slug, mut entry) in repos {
            if entry.name.is_empty() {
                entry.name = slug.clone();
            }

            // Validate crate_count must be positive integer
            if PositiveUsize::new(entry.crate_count).is_none() {
                return Err(crate::RetrofitError::ConfigError(format!(
                    "Invalid crate_count for repo '{}': count must be a positive integer, found '{}'",
                    entry.name, entry.crate_count
                )));
            }

            // Validate priority_score must be between 0 and 100 (validated via BoundedU32)
            if entry.priority_score > 100 || BoundedU32::new(entry.priority_score as u32).is_none()
            {
                return Err(crate::RetrofitError::ConfigError(format!(
                    "Invalid priority_score for repo '{}': score must be between 0 and 100, found '{}'",
                    entry.name, entry.priority_score
                )));
            }

            fixed_repos.insert(slug, entry);
        }

        let metadata = self.metadata.unwrap_or_else(|| EcosystemMetadata {
            ecosystem_name: "seanchatmangpt".to_string(),
            total_repos: fixed_repos.len(),
            total_crates: 0,
            survey_date: "unknown".to_string(),
            survey_agents: 0,
            survey_scope: "unknown".to_string(),
            house_msrv: "1.82".to_string(),
            house_edition: "2021".to_string(),
            house_toolchain: "1.82.0".to_string(),
            house_license: "MIT OR Apache-2.0".to_string(),
            house_version_scheme: "CalVer YY.M.patch".to_string(),
            ready_for_retrofit: 0,
            requires_phase_2_or_3: 0,
            requires_phase_0_or_1: 0,
            experimental_status: 0,
            primary_order: vec![],
            secondary_order: vec![],
            tertiary_order: vec![],
            agpl_repos: vec![],
            bsl_repos: vec![],
            apache_only_repos: vec![],
            missing_license_files: vec![],
            no_ci_repos: vec![],
            deprecated_actions_repos: vec![],
            minimal_ci_repos: vec![],
            sparse_release_workflows: 0,
            missing_claude_md: vec![],
            missing_security_md: vec![],
        });

        Ok(RepositoryRegistry { repos: fixed_repos, metadata })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_entry_phase() {
        let mut entry = RepositoryEntry {
            name: "test".to_string(),
            github_url: "https://github.com/seanchatmangpt/test".to_string(),
            github_owner: "seanchatmangpt".to_string(),
            local_path: PathBuf::from("../test"),
            crate_name: "test".to_string(),
            description: "Test repo".to_string(),
            visibility: "public".to_string(),
            workspace_type: "single-crate".to_string(),
            crate_count: 1,
            retrofit_readiness: "ready".to_string(),
            retrofit_phase_complete: 0,
            risk_level: "low".to_string(),
            priority_score: 50,
            maintainer_status: "active".to_string(),
            notes: String::new(),
        };

        assert_eq!(entry.retrofit_phase(), RetrofitPhase::Phase1Lints);
        entry.retrofit_phase_complete = 3;
        assert_eq!(entry.retrofit_phase(), RetrofitPhase::Phase3Justfile);
    }

    #[test]
    fn test_repository_entry_readiness() {
        let mut entry = RepositoryEntry {
            name: "test".to_string(),
            github_url: "https://github.com/seanchatmangpt/test".to_string(),
            github_owner: "seanchatmangpt".to_string(),
            local_path: PathBuf::from("../test"),
            crate_name: "test".to_string(),
            description: "Test repo".to_string(),
            visibility: "public".to_string(),
            workspace_type: "single-crate".to_string(),
            crate_count: 1,
            retrofit_readiness: "ready".to_string(),
            retrofit_phase_complete: 0,
            risk_level: "low".to_string(),
            priority_score: 50,
            maintainer_status: "active".to_string(),
            notes: String::new(),
        };

        assert!(entry.is_ready_for_retrofit());
        entry.retrofit_readiness = "requires-prep".to_string();
        assert!(entry.requires_prep());
        entry.retrofit_readiness = "blocked".to_string();
        assert!(entry.is_blocked());
    }

    #[test]
    fn test_estimated_effort() {
        let entry = RepositoryEntry {
            name: "test".to_string(),
            github_url: "https://github.com/seanchatmangpt/test".to_string(),
            github_owner: "seanchatmangpt".to_string(),
            local_path: PathBuf::from("../test"),
            crate_name: "test".to_string(),
            description: "Test repo".to_string(),
            visibility: "public".to_string(),
            workspace_type: "single-crate".to_string(),
            crate_count: 1,
            retrofit_readiness: "ready".to_string(),
            retrofit_phase_complete: 0,
            risk_level: "low".to_string(),
            priority_score: 50,
            maintainer_status: "active".to_string(),
            notes: String::new(),
        };

        let effort = entry.estimated_effort_weeks();
        assert!(effort > 0.0);
        assert!(effort < 1.0); // Single crate, 5 phases, low risk should be ~0.375 weeks
    }

    static CWD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    const MOCK_TOML: &str = r#"
[metadata]
ecosystem_name = "seanchatmangpt"
total_repos = 1
total_crates = 1
survey_date = "2026-06-29"
survey_agents = 1
survey_scope = "test"
house_msrv = "1.82"
house_edition = "2021"
house_toolchain = "1.82.0"
house_license = "MIT OR Apache-2.0"
house_version_scheme = "CalVer"
ready_for_retrofit = 1
requires_phase_2_or_3 = 0
requires_phase_0_or_1 = 1
experimental_status = 0
primary_order = ["test"]
secondary_order = []
tertiary_order = []
agpl_repos = []
bsl_repos = []
apache_only_repos = []
missing_license_files = []
no_ci_repos = []
deprecated_actions_repos = []
minimal_ci_repos = []
sparse_release_workflows = 0
missing_claude_md = []
missing_security_md = []

[repos.test]
name = "test"
github_url = "https://github.com/seanchatmangpt/test"
github_owner = "seanchatmangpt"
local_path = "../test"
crate_name = "test"
description = "Test description"
visibility = "public"
workspace_type = "single-crate"
crate_count = 1
retrofit_readiness = "ready"
retrofit_phase_complete = 0
risk_level = "low"
priority_score = 50
maintainer_status = "active"
notes = ""
"#;

    #[tokio::test]
    async fn test_load_with_env_var_override() {
        let _guard = CWD_LOCK.lock().unwrap();
        let original_env = std::env::var("PRAXIS_REGISTRY_PATH");

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("repos.toml");
        std::fs::write(&file_path, MOCK_TOML).unwrap();

        std::env::set_var("PRAXIS_REGISTRY_PATH", &file_path);

        let result = RepositoryRegistry::load("dummy_fallback.toml").await;

        match original_env {
            Ok(val) => std::env::set_var("PRAXIS_REGISTRY_PATH", val),
            Err(_) => std::env::remove_var("PRAXIS_REGISTRY_PATH"),
        }

        let registry = result.expect("Should load successfully from env override path");
        assert_eq!(registry.metadata.ecosystem_name, "seanchatmangpt");
        assert!(registry.get("test").is_some());
    }

    #[test]
    fn test_load_with_parent_directory_search() {
        let _guard = CWD_LOCK.lock().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        let original_env = std::env::var("PRAXIS_REGISTRY_PATH");

        // Ensure env var is not set, so parent search is performed
        std::env::remove_var("PRAXIS_REGISTRY_PATH");

        let temp_dir = tempfile::tempdir().unwrap();
        let parent_path = temp_dir.path();
        let child_path = parent_path.join("child");
        std::fs::create_dir(&child_path).unwrap();

        let repos_file = parent_path.join("repos.toml");
        std::fs::write(&repos_file, MOCK_TOML).unwrap();

        std::env::set_current_dir(&child_path).unwrap();

        let resolved = resolve_layered_path(std::path::Path::new("fallback.toml"));

        // Restore CWD and Env
        std::env::set_current_dir(&original_cwd).unwrap();
        match original_env {
            Ok(val) => std::env::set_var("PRAXIS_REGISTRY_PATH", val),
            Err(_) => std::env::remove_var("PRAXIS_REGISTRY_PATH"),
        }

        assert_eq!(resolved, repos_file);
    }

    #[test]
    fn test_load_fallback_to_parameter() {
        let _guard = CWD_LOCK.lock().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        let original_env = std::env::var("PRAXIS_REGISTRY_PATH");

        // Ensure env var is not set
        std::env::remove_var("PRAXIS_REGISTRY_PATH");

        // Create a temp dir with no repos.toml, and switch CWD to it
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let fallback_path = std::path::Path::new("exact_fallback.toml");
        let resolved = resolve_layered_path(fallback_path);

        // Restore CWD and Env
        std::env::set_current_dir(&original_cwd).unwrap();
        match original_env {
            Ok(val) => std::env::set_var("PRAXIS_REGISTRY_PATH", val),
            Err(_) => std::env::remove_var("PRAXIS_REGISTRY_PATH"),
        }

        assert_eq!(resolved, fallback_path);
    }
}

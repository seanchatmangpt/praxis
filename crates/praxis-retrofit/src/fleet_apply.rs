//! Fleet-wide automated retrofit application system
//!
//! This module provides comprehensive automation for applying retrofit changes
//! across multiple repositories in parallel, with proper isolation via git worktrees,
//! change validation, and atomic commit operations.
//!
//! # Design
//!
//! - **Isolation**: Each repository is cloned into a temporary worktree to prevent
//!   interference with the working tree
//! - **Phases**: Changes are organized into phases (retrofit/phase-1, retrofit/phase-2, etc)
//! - **Validation**: Each retrofit is validated before committing
//! - **Atomicity**: Commits are created with standard, phase-aware messages
//!
//! # Example
//!
//! ```ignore
//! let mut applier = RetrofitApplier::new(spec)?;
//! applier.add_repository("../repo-a", RetrofitPhase::Phase1Lints)?;
//! applier.add_repository("../repo-b", RetrofitPhase::Phase1Lints)?;
//!
//! let results = applier.apply_all().await?;
//! for result in results {
//!     println!("{:?}", result);
//! }
//! ```

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::{apply as retrofit_apply, generate, models::*, PraxisSpec, Result, RetrofitError};

/// Result of applying a retrofit to a single repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    /// Repository name
    pub repository_name: String,
    /// Original repository path
    pub original_path: PathBuf,
    /// Temporary worktree path
    pub worktree_path: PathBuf,
    /// Retrofit phase applied
    pub phase: RetrofitPhase,
    /// Whether the retrofit was successful
    pub success: bool,
    /// Commit hash if successful
    pub commit_hash: Option<String>,
    /// Branch name (e.g., "retrofit/phase-1")
    pub branch_name: String,
    /// Detailed messages from the operation
    pub messages: Vec<String>,
    /// Any warnings encountered
    pub warnings: Vec<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Time taken in seconds
    pub duration_secs: f64,
}

impl ApplyResult {
    /// Whether this result represents a successful retrofit
    pub fn is_success(&self) -> bool {
        self.success && self.error.is_none()
    }

    /// Print a human-readable summary
    pub fn summary(&self) -> String {
        if self.is_success() {
            format!(
                "✓ {} [{}] -> {} ({}s)",
                self.repository_name,
                format!("{:?}", self.phase),
                self.commit_hash.as_ref().map(|h| &h[..8]).unwrap_or("unknown"),
                self.duration_secs
            )
        } else {
            format!(
                "✗ {} [{}] - {}",
                self.repository_name,
                format!("{:?}", self.phase),
                self.error.as_deref().unwrap_or("unknown error")
            )
        }
    }
}

/// Represents an isolated worktree for a single repository retrofit
#[derive(Debug)]
pub struct RetrofitWorktree {
    /// Original repository path
    original_path: PathBuf,
    /// Temporary worktree path
    worktree_path: PathBuf,
    /// Repository name
    name: String,
    /// Git remote URL (if available)
    remote_url: Option<String>,
    /// Current branch in worktree
    current_branch: String,
}

impl RetrofitWorktree {
    /// Create a new worktree for the given repository
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to the original repository
    /// * `phase` - Retrofit phase to use for branch naming
    ///
    /// # Returns
    ///
    /// A new RetrofitWorktree or error if the repository is invalid
    pub fn new(repo_path: &Path, phase: RetrofitPhase) -> Result<Self> {
        if !repo_path.exists() {
            return Err(RetrofitError::RepositoryNotFound(format!(
                "Repository not found: {}",
                repo_path.display()
            )));
        }

        if !repo_path.join(".git").exists() {
            return Err(RetrofitError::RepositoryNotFound(format!(
                "Not a git repository: {}",
                repo_path.display()
            )));
        }

        let name = repo_path.file_name().unwrap_or_default().to_string_lossy().to_string();

        let branch_name = Self::branch_name_for_phase(phase);

        // Create temporary directory for worktree
        let temp_dir = std::env::temp_dir().join("praxis-retrofit").join(format!(
            "{}-{}",
            name,
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        ));

        std::fs::create_dir_all(&temp_dir)?;

        // Get remote URL if available
        let remote_url = Self::get_remote_url(repo_path);

        // Create worktree
        Self::create_worktree(repo_path, &temp_dir, &branch_name)?;

        debug!("Created worktree at {} for {}", temp_dir.display(), name);

        Ok(RetrofitWorktree {
            original_path: repo_path.to_path_buf(),
            worktree_path: temp_dir,
            name,
            remote_url,
            current_branch: branch_name,
        })
    }

    /// Get the temporary worktree path
    pub fn path(&self) -> &Path {
        &self.worktree_path
    }

    /// Get the repository name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the current branch name
    pub fn branch(&self) -> &str {
        &self.current_branch
    }

    /// Generate the branch name for a given phase
    fn branch_name_for_phase(phase: RetrofitPhase) -> String {
        match phase {
            RetrofitPhase::Phase1Lints => "retrofit/phase-1-lints".to_string(),
            RetrofitPhase::Phase2Deps => "retrofit/phase-2-deps".to_string(),
            RetrofitPhase::Phase3Justfile => "retrofit/phase-3-justfile".to_string(),
            RetrofitPhase::Phase4Typos => "retrofit/phase-4-typos".to_string(),
            RetrofitPhase::Phase5Docs => "retrofit/phase-5-docs".to_string(),
        }
    }

    /// Get the remote URL for the repository
    fn get_remote_url(repo_path: &Path) -> Option<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .arg("config")
            .arg("--get")
            .arg("remote.origin.url")
            .output()
            .ok()?;

        if output.status.success() {
            let url = String::from_utf8(output.stdout).ok()?;
            Some(url.trim().to_string())
        } else {
            None
        }
    }

    /// Create a worktree using git
    fn create_worktree(repo_path: &Path, worktree_path: &Path, branch_name: &str) -> Result<()> {
        // First, ensure the branch exists or create it
        let branch_exists = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .arg("show-ref")
            .arg("--verify")
            .arg(format!("refs/heads/{}", branch_name))
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if !branch_exists {
            // Create a new branch from main/master
            let default_branch = Self::get_default_branch(repo_path)?;
            Command::new("git")
                .arg("-C")
                .arg(repo_path)
                .arg("branch")
                .arg(branch_name)
                .arg(&default_branch)
                .output()
                .map_err(|e| {
                    RetrofitError::RetrofitFailed(format!("Failed to create branch: {}", e))
                })?;
        }

        // Create the worktree
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .arg("worktree")
            .arg("add")
            .arg(worktree_path)
            .arg(branch_name)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RetrofitError::RetrofitFailed(format!(
                "Failed to create worktree: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Get the default branch (main or master)
    fn get_default_branch(repo_path: &Path) -> Result<String> {
        // Try to get the default branch from the remote
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .arg("symbolic-ref")
            .arg("refs/remotes/origin/HEAD")
            .output()?;

        if output.status.success() {
            let ref_str = String::from_utf8_lossy(&output.stdout);
            if let Some(branch) = ref_str.split('/').last() {
                return Ok(branch.trim().to_string());
            }
        }

        // Fallback to checking for main or master
        for branch in &["main", "master"] {
            let output = Command::new("git")
                .arg("-C")
                .arg(repo_path)
                .arg("show-ref")
                .arg("--verify")
                .arg(format!("refs/heads/{}", branch))
                .output()?;

            if output.status.success() {
                return Ok(branch.to_string());
            }
        }

        Err(RetrofitError::RetrofitFailed("Could not determine default branch".to_string()))
    }

    /// Apply a retrofit plan to this worktree
    pub async fn apply_plan(&self, plan: &RetrofitPlan) -> Result<Vec<String>> {
        retrofit_apply::apply_retrofit(&self.worktree_path, plan).await
    }

    /// Validate the retrofit in this worktree
    pub async fn validate(&self) -> Result<bool> {
        retrofit_apply::validate_retrofit(&self.worktree_path).await
    }

    /// Commit the changes with the given message
    pub fn commit(&self, message: &str) -> Result<String> {
        // Stage all changes
        Command::new("git").arg("-C").arg(&self.worktree_path).arg("add").arg("-A").output()?;

        // Create commit
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.worktree_path)
            .arg("commit")
            .arg("-m")
            .arg(message)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RetrofitError::RetrofitFailed(format!("Commit failed: {}", stderr)));
        }

        // Get the commit hash
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.worktree_path)
            .arg("rev-parse")
            .arg("HEAD")
            .output()?;

        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout);
            Ok(hash.trim().to_string())
        } else {
            Err(RetrofitError::RetrofitFailed("Failed to get commit hash".to_string()))
        }
    }

    /// Push changes back to the original repository
    pub fn push_to_origin(&self) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.original_path)
            .arg("push")
            .arg("origin")
            .arg(&self.current_branch)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Push warning: {}", stderr);
        }

        Ok(())
    }

    /// Clean up the worktree
    pub fn cleanup(&self) -> Result<()> {
        // Remove the worktree
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.original_path)
            .arg("worktree")
            .arg("remove")
            .arg(&self.worktree_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Worktree cleanup warning: {}", stderr);
        }

        // Remove temporary directory if it still exists
        if self.worktree_path.exists() {
            std::fs::remove_dir_all(&self.worktree_path).ok();
        }

        Ok(())
    }
}

impl Drop for RetrofitWorktree {
    fn drop(&mut self) {
        if let Err(e) = self.cleanup() {
            warn!("Failed to clean up worktree: {}", e);
        }
    }
}

/// Main automation controller for applying retrofits to a fleet of repositories
pub struct RetrofitApplier {
    spec: PraxisSpec,
    repositories: Vec<(PathBuf, RetrofitPhase)>,
    concurrent_limit: usize,
}

impl RetrofitApplier {
    /// Create a new retrofit applier
    pub fn new(spec: PraxisSpec) -> Result<Self> {
        Ok(RetrofitApplier {
            spec,
            repositories: Vec::new(),
            concurrent_limit: 4, // Default: 4 concurrent retrofits
        })
    }

    /// Get the list of registered repositories
    pub fn repositories(&self) -> &[(PathBuf, RetrofitPhase)] {
        &self.repositories
    }

    /// Set the number of concurrent retrofits (default: 4)
    pub fn with_concurrent_limit(mut self, limit: usize) -> Self {
        self.concurrent_limit = limit.max(1);
        self
    }

    /// Add a repository to be retrofitted
    pub fn add_repository(
        &mut self,
        repo_path: impl AsRef<Path>,
        phase: RetrofitPhase,
    ) -> Result<()> {
        let path = repo_path.as_ref();

        if !path.exists() {
            return Err(RetrofitError::RepositoryNotFound(format!(
                "Repository not found: {}",
                path.display()
            )));
        }

        if !path.join(".git").exists() {
            return Err(RetrofitError::RepositoryNotFound(format!(
                "Not a git repository: {}",
                path.display()
            )));
        }

        self.repositories.push((path.to_path_buf(), phase));
        info!("Added repository: {} ({:?})", path.display(), phase);
        Ok(())
    }

    /// Apply retrofits to all registered repositories
    ///
    /// Processes repositories sequentially with proper error handling and reporting.
    pub async fn apply_all(&self) -> Result<Vec<ApplyResult>> {
        let mut results = Vec::new();

        for (repo_path, phase) in &self.repositories {
            let result = self.apply_single(repo_path, *phase).await;
            results.push(result);
        }

        Ok(results)
    }

    /// Apply retrofit to a single repository
    async fn apply_single(&self, repo_path: &Path, phase: RetrofitPhase) -> ApplyResult {
        let start_time = std::time::Instant::now();
        let repo_name = repo_path.file_name().unwrap_or_default().to_string_lossy().to_string();

        info!("Starting retrofit for {} ({:?})", repo_name, phase);

        let branch_name = match phase {
            RetrofitPhase::Phase1Lints => "retrofit/phase-1-lints".to_string(),
            RetrofitPhase::Phase2Deps => "retrofit/phase-2-deps".to_string(),
            RetrofitPhase::Phase3Justfile => "retrofit/phase-3-justfile".to_string(),
            RetrofitPhase::Phase4Typos => "retrofit/phase-4-typos".to_string(),
            RetrofitPhase::Phase5Docs => "retrofit/phase-5-docs".to_string(),
        };

        // Create worktree
        let worktree = match RetrofitWorktree::new(repo_path, phase) {
            Ok(w) => w,
            Err(e) => {
                error!("Failed to create worktree for {}: {}", repo_name, e);
                return ApplyResult {
                    repository_name: repo_name,
                    original_path: repo_path.to_path_buf(),
                    worktree_path: PathBuf::new(),
                    phase,
                    success: false,
                    commit_hash: None,
                    branch_name,
                    messages: vec![],
                    warnings: vec![],
                    error: Some(e.to_string()),
                    duration_secs: start_time.elapsed().as_secs_f64(),
                };
            }
        };

        let mut messages = vec![format!("Created worktree at {}", worktree.path().display())];
        let mut warnings = vec![];

        // Generate retrofit plan
        let plan = match generate::generate_retrofit_plan(worktree.path(), phase, &self.spec) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to generate retrofit plan for {}: {}", repo_name, e);
                return ApplyResult {
                    repository_name: repo_name,
                    original_path: repo_path.to_path_buf(),
                    worktree_path: worktree.path().to_path_buf(),
                    phase,
                    success: false,
                    commit_hash: None,
                    branch_name,
                    messages,
                    warnings,
                    error: Some(e.to_string()),
                    duration_secs: start_time.elapsed().as_secs_f64(),
                };
            }
        };

        messages.push(format!("Generated retrofit plan with {} actions", plan.actions.len()));

        // Apply retrofit plan
        if let Err(e) = worktree.apply_plan(&plan).await {
            error!("Failed to apply retrofit plan for {}: {}", repo_name, e);
            return ApplyResult {
                repository_name: repo_name,
                original_path: repo_path.to_path_buf(),
                worktree_path: worktree.path().to_path_buf(),
                phase,
                success: false,
                commit_hash: None,
                branch_name,
                messages,
                warnings,
                error: Some(e.to_string()),
                duration_secs: start_time.elapsed().as_secs_f64(),
            };
        }

        messages.push("Applied retrofit changes".to_string());

        // Validate retrofit
        match worktree.validate().await {
            Ok(true) => messages.push("Validation passed".to_string()),
            Ok(false) => {
                warnings.push("Validation returned false".to_string());
                messages.push("Validation warning: returned false".to_string());
            }
            Err(e) => {
                warn!("Validation error for {}: {}", repo_name, e);
                warnings.push(format!("Validation error: {}", e));
            }
        }

        // Create commit
        let commit_hash = match worktree.commit(&plan.commit_message) {
            Ok(hash) => {
                messages.push(format!("Created commit: {}", &hash[..8.min(hash.len())]));
                Some(hash)
            }
            Err(e) => {
                error!("Failed to commit changes for {}: {}", repo_name, e);
                warnings.push(format!("Commit warning: {}", e));
                None
            }
        };

        info!("Completed retrofit for {} in {:.2}s", repo_name, start_time.elapsed().as_secs_f64());

        ApplyResult {
            repository_name: repo_name,
            original_path: repo_path.to_path_buf(),
            worktree_path: worktree.path().to_path_buf(),
            phase,
            success: commit_hash.is_some() && warnings.is_empty(),
            commit_hash,
            branch_name,
            messages,
            warnings,
            error: None,
            duration_secs: start_time.elapsed().as_secs_f64(),
        }
    }

    /// Generate a summary report of all results
    pub fn summary(results: &[ApplyResult]) -> FleetApplyReport {
        let total = results.len();
        let successful = results.iter().filter(|r| r.is_success()).count();
        let failed = results.iter().filter(|r| !r.is_success()).count();
        let warnings_count = results.iter().map(|r| r.warnings.len()).sum();

        FleetApplyReport {
            total_repositories: total,
            successful,
            failed,
            warnings_count,
            results: results.to_vec(),
        }
    }
}

/// Summary report for fleet-wide retrofit operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetApplyReport {
    pub total_repositories: usize,
    pub successful: usize,
    pub failed: usize,
    pub warnings_count: usize,
    pub results: Vec<ApplyResult>,
}

impl FleetApplyReport {
    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f32 {
        if self.total_repositories == 0 {
            100.0
        } else {
            (self.successful as f32 / self.total_repositories as f32) * 100.0
        }
    }

    /// Print a human-readable summary
    pub fn print_summary(&self) {
        println!("\n=== Retrofit Fleet Report ===");
        println!("Total repositories: {}", self.total_repositories);
        println!("Successful: {} ({:.1}%)", self.successful, self.success_rate());
        println!("Failed: {}", self.failed);
        println!("Total warnings: {}", self.warnings_count);
        println!("\nDetails:");
        for result in &self.results {
            println!("  {}", result.summary());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_name_generation() {
        assert_eq!(
            RetrofitWorktree::branch_name_for_phase(RetrofitPhase::Phase1Lints),
            "retrofit/phase-1-lints"
        );
        assert_eq!(
            RetrofitWorktree::branch_name_for_phase(RetrofitPhase::Phase2Deps),
            "retrofit/phase-2-deps"
        );
        assert_eq!(
            RetrofitWorktree::branch_name_for_phase(RetrofitPhase::Phase3Justfile),
            "retrofit/phase-3-justfile"
        );
        assert_eq!(
            RetrofitWorktree::branch_name_for_phase(RetrofitPhase::Phase4Typos),
            "retrofit/phase-4-typos"
        );
        assert_eq!(
            RetrofitWorktree::branch_name_for_phase(RetrofitPhase::Phase5Docs),
            "retrofit/phase-5-docs"
        );
    }

    #[test]
    fn test_apply_result_summary() {
        let result = ApplyResult {
            repository_name: "test-repo".to_string(),
            original_path: PathBuf::from("/path/to/repo"),
            worktree_path: PathBuf::from("/tmp/worktree"),
            phase: RetrofitPhase::Phase1Lints,
            success: true,
            commit_hash: Some("abc1234567890".to_string()),
            branch_name: "retrofit/phase-1-lints".to_string(),
            messages: vec!["test".to_string()],
            warnings: vec![],
            error: None,
            duration_secs: 1.5,
        };

        let summary = result.summary();
        assert!(summary.contains("✓"));
        assert!(summary.contains("test-repo"));
        assert!(summary.contains("abc12345"));
    }

    #[test]
    fn test_fleet_report_success_rate() {
        let report = FleetApplyReport {
            total_repositories: 10,
            successful: 8,
            failed: 2,
            warnings_count: 1,
            results: vec![],
        };

        assert_eq!(report.success_rate(), 80.0);
    }

    #[test]
    fn test_fleet_report_empty() {
        let report = FleetApplyReport {
            total_repositories: 0,
            successful: 0,
            failed: 0,
            warnings_count: 0,
            results: vec![],
        };

        assert_eq!(report.success_rate(), 100.0);
    }
}

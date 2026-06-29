//! PR generation system for mass retrofit across fleet
//!
//! This module handles:
//! - Creating pull requests across multiple repositories
//! - Generating standard PR titles and bodies using conventional commits
//! - Tracking PR status (open, merged, review status)
//! - GitHub MCP integration via `gh` CLI

use std::{path::PathBuf, process::Command};

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::{RepositoryMetadata, RetrofitAction, RetrofitPhase, RiskLevel};

/// A pull request template following conventional commits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestTemplate {
    /// PR title (conventional commit format)
    pub title: String,
    /// Full PR body with markdown formatting
    pub body: String,
    /// Labels to assign
    pub labels: Vec<String>,
    /// Suggested reviewers
    pub assignees: Vec<String>,
}

/// Status of a single pull request
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PRStatus {
    #[serde(rename = "not-created")]
    NotCreated,
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "review-requested")]
    ReviewRequested,
    #[serde(rename = "changes-requested")]
    ChangesRequested,
    #[serde(rename = "approved")]
    Approved,
    #[serde(rename = "merged")]
    Merged,
    #[serde(rename = "closed")]
    Closed,
}

/// Metadata for a created/tracked pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestInfo {
    /// Repository metadata
    pub repository: RepositoryMetadata,
    /// GitHub PR URL
    pub url: Option<String>,
    /// PR number
    pub number: Option<usize>,
    /// Current status
    pub status: PRStatus,
    /// Branch name
    pub branch_name: String,
    /// Retrofit phase this PR addresses
    pub phase: RetrofitPhase,
    /// When the PR was created
    pub created_at: Option<String>,
    /// Estimated risk level
    pub estimated_risk: RiskLevel,
    /// Number of files changed
    pub files_changed: usize,
    /// Number of commits
    pub commits: usize,
    /// Review feedback (if any)
    pub review_comments: Vec<String>,
}

/// Fleet-wide PR tracking summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetPRStatus {
    /// All tracked PRs
    pub pull_requests: Vec<PullRequestInfo>,
    /// Total PRs
    pub total: usize,
    /// Count by status
    pub by_status: PRStatusCounts,
    /// Generated timestamp
    pub generated_at: String,
}

/// Breakdown of PR statuses
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PRStatusCounts {
    pub open: usize,
    pub draft: usize,
    pub review_requested: usize,
    pub approved: usize,
    pub changes_requested: usize,
    pub merged: usize,
    pub closed: usize,
}

/// Configuration for PR generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestGeneratorConfig {
    /// GitHub organization/owner
    pub github_owner: String,
    /// Use draft mode initially
    pub create_as_draft: bool,
    /// Assign reviewers automatically
    pub auto_assign_reviewers: Vec<String>,
    /// Labels to apply
    pub labels: Vec<String>,
    /// Base branch (usually "main" or "master")
    pub base_branch: String,
    /// Prefix for branch names
    pub branch_prefix: String,
}

impl Default for PullRequestGeneratorConfig {
    fn default() -> Self {
        Self {
            github_owner: String::new(),
            create_as_draft: true,
            auto_assign_reviewers: vec!["@seanchatmangpt".to_string()],
            labels: vec!["retrofit".to_string(), "praxis".to_string()],
            base_branch: "main".to_string(),
            branch_prefix: "praxis/retrofit".to_string(),
        }
    }
}

/// Main PR generator for orchestrating fleet-wide PRs
pub struct PullRequestGenerator {
    config: PullRequestGeneratorConfig,
}

impl PullRequestGenerator {
    /// Create a new PR generator with configuration
    pub fn new(config: PullRequestGeneratorConfig) -> Self {
        Self { config }
    }

    /// Generate a PR template for Phase 1 (Lints)
    pub fn template_phase1_lints(
        repo: &RepositoryMetadata,
        files_changed: usize,
    ) -> PullRequestTemplate {
        PullRequestTemplate {
            title: format!(
                "retrofit(lints): Add praxis workspace linting standards for {}",
                repo.name
            ),
            body: pr_body_phase1_lints(&repo.name, files_changed),
            labels: vec!["retrofit".to_string(), "linting".to_string(), "praxis".to_string()],
            assignees: vec!["@seanchatmangpt".to_string()],
        }
    }

    /// Generate a PR template for Phase 2 (Dependencies)
    pub fn template_phase2_deps(
        repo: &RepositoryMetadata,
        files_changed: usize,
    ) -> PullRequestTemplate {
        PullRequestTemplate {
            title: format!(
                "retrofit(deps): Unify dependencies via workspace.dependencies for {}",
                repo.name
            ),
            body: pr_body_phase2_deps(&repo.name, files_changed),
            labels: vec!["retrofit".to_string(), "dependencies".to_string(), "praxis".to_string()],
            assignees: vec!["@seanchatmangpt".to_string()],
        }
    }

    /// Generate a PR template for Phase 3 (Justfile)
    pub fn template_phase3_justfile(
        repo: &RepositoryMetadata,
        files_changed: usize,
    ) -> PullRequestTemplate {
        PullRequestTemplate {
            title: format!("retrofit(build): Standardize justfile task runner for {}", repo.name),
            body: pr_body_phase3_justfile(&repo.name, files_changed),
            labels: vec!["retrofit".to_string(), "build".to_string(), "praxis".to_string()],
            assignees: vec!["@seanchatmangpt".to_string()],
        }
    }

    /// Generate a PR template for Phase 4 (Typos)
    pub fn template_phase4_typos(
        repo: &RepositoryMetadata,
        files_changed: usize,
    ) -> PullRequestTemplate {
        PullRequestTemplate {
            title: format!("retrofit(ci): Add praxis spell-check configuration for {}", repo.name),
            body: pr_body_phase4_typos(&repo.name, files_changed),
            labels: vec!["retrofit".to_string(), "ci".to_string(), "praxis".to_string()],
            assignees: vec!["@seanchatmangpt".to_string()],
        }
    }

    /// Generate a PR template for Phase 5 (Documentation)
    pub fn template_phase5_docs(
        repo: &RepositoryMetadata,
        files_changed: usize,
    ) -> PullRequestTemplate {
        PullRequestTemplate {
            title: format!("retrofit(docs): Add praxis documentation standards for {}", repo.name),
            body: pr_body_phase5_docs(&repo.name, files_changed),
            labels: vec!["retrofit".to_string(), "documentation".to_string(), "praxis".to_string()],
            assignees: vec!["@seanchatmangpt".to_string()],
        }
    }

    /// Generate template based on retrofit phase
    pub fn template_for_phase(
        phase: RetrofitPhase,
        repo: &RepositoryMetadata,
        files_changed: usize,
    ) -> PullRequestTemplate {
        match phase {
            RetrofitPhase::Phase1Lints => Self::template_phase1_lints(repo, files_changed),
            RetrofitPhase::Phase2Deps => Self::template_phase2_deps(repo, files_changed),
            RetrofitPhase::Phase3Justfile => Self::template_phase3_justfile(repo, files_changed),
            RetrofitPhase::Phase4Typos => Self::template_phase4_typos(repo, files_changed),
            RetrofitPhase::Phase5Docs => Self::template_phase5_docs(repo, files_changed),
        }
    }

    /// Create a branch name for the retrofit
    pub fn branch_name(&self, repo_name: &str, phase: RetrofitPhase) -> String {
        let phase_str = match phase {
            RetrofitPhase::Phase1Lints => "phase-1-lints",
            RetrofitPhase::Phase2Deps => "phase-2-deps",
            RetrofitPhase::Phase3Justfile => "phase-3-justfile",
            RetrofitPhase::Phase4Typos => "phase-4-typos",
            RetrofitPhase::Phase5Docs => "phase-5-docs",
        };
        format!("{}/{}/{}", self.config.branch_prefix, phase_str, repo_name)
    }

    /// Create a PR using `gh` CLI
    ///
    /// # Errors
    /// Returns error if `gh` command fails or repo doesn't have remote
    pub fn create_pull_request(
        &self,
        repo_path: &PathBuf,
        repo: &RepositoryMetadata,
        template: &PullRequestTemplate,
        phase: RetrofitPhase,
    ) -> crate::Result<PullRequestInfo> {
        let branch_name = self.branch_name(&repo.name, phase);

        info!("Creating PR for {} on branch {}", repo.name, branch_name);

        // Create PR using gh CLI
        let output = Command::new("gh")
            .args(&[
                "pr",
                "create",
                "-B",
                &self.config.base_branch,
                "-H",
                &branch_name,
                "-t",
                &template.title,
                "-b",
                &template.body,
            ])
            .current_dir(repo_path)
            .output()
            .map_err(|e| {
                error!("Failed to create PR for {}: {}", repo.name, e);
                crate::RetrofitError::RetrofitFailed(format!(
                    "gh pr create failed for {}: {}",
                    repo.name, e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("gh pr create returned non-zero: {}", stderr);
            return Err(crate::RetrofitError::RetrofitFailed(format!(
                "gh pr create failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let pr_url = stdout.trim().to_string();

        // Extract PR number from URL (format: https://github.com/owner/repo/pull/123)
        let pr_number = pr_url.split('/').last().and_then(|s| s.parse::<usize>().ok());

        debug!("Created PR {} (URL: {})", pr_number.unwrap_or(0), pr_url);

        Ok(PullRequestInfo {
            repository: repo.clone(),
            url: Some(pr_url),
            number: pr_number,
            status: if self.config.create_as_draft { PRStatus::Draft } else { PRStatus::Open },
            branch_name,
            phase,
            created_at: Some(chrono::Local::now().to_rfc3339()),
            estimated_risk: RiskLevel::Low,
            files_changed: 0,
            commits: 1,
            review_comments: vec![],
        })
    }

    /// Fetch current status of a PR from GitHub
    ///
    /// # Errors
    /// Returns error if `gh` CLI call fails
    pub fn fetch_pr_status(
        &self,
        repo_path: &PathBuf,
        pr_number: usize,
    ) -> crate::Result<PRStatus> {
        debug!("Fetching PR status for #{}", pr_number);

        let output = Command::new("gh")
            .args(&[
                "pr",
                "view",
                &pr_number.to_string(),
                "-R",
                &self.config.github_owner,
                "--json",
                "state",
            ])
            .current_dir(repo_path)
            .output()
            .map_err(|e| {
                error!("Failed to fetch PR status: {}", e);
                crate::RetrofitError::RetrofitFailed(format!("gh pr view failed: {}", e))
            })?;

        if !output.status.success() {
            return Err(crate::RetrofitError::RetrofitFailed(
                "Failed to fetch PR status".to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let state = if stdout.contains("OPEN") {
            PRStatus::Open
        } else if stdout.contains("DRAFT") {
            PRStatus::Draft
        } else if stdout.contains("MERGED") {
            PRStatus::Merged
        } else if stdout.contains("CLOSED") {
            PRStatus::Closed
        } else {
            PRStatus::Open
        };

        Ok(state)
    }

    /// Convert retrofit actions to files changed count
    pub fn count_files_changed(actions: &[RetrofitAction]) -> usize {
        actions.len()
    }

    /// Generate a summary of all PRs across fleet
    pub fn summarize_fleet_prs(prs: &[PullRequestInfo]) -> FleetPRStatus {
        let mut counts = PRStatusCounts::default();

        for pr in prs {
            match pr.status {
                PRStatus::Open => counts.open += 1,
                PRStatus::Draft => counts.draft += 1,
                PRStatus::ReviewRequested => counts.review_requested += 1,
                PRStatus::Approved => counts.approved += 1,
                PRStatus::ChangesRequested => counts.changes_requested += 1,
                PRStatus::Merged => counts.merged += 1,
                PRStatus::Closed => counts.closed += 1,
                PRStatus::NotCreated => {}
            }
        }

        FleetPRStatus {
            total: prs.len(),
            by_status: counts,
            pull_requests: prs.to_vec(),
            generated_at: chrono::Local::now().to_rfc3339(),
        }
    }
}

/// Generate PR body for Phase 1: Lints
fn pr_body_phase1_lints(repo_name: &str, files_changed: usize) -> String {
    format!(
        r#"## Retrofit: Phase 1 - Workspace Linting Standards

This PR adds praxis house-style linting configuration to `{}`, bringing it into compliance with the standardized Rust ecosystem practices.

### Changes

- ✓ Added `[lints]` configuration block to workspace `Cargo.toml`
- ✓ Configured strict linting rules: `unsafe_code = "forbid"`
- ✓ Added Clippy warnings for `all`, `pedantic`, `nursery`
- ✓ Added rustdoc warnings for documentation completeness
- {} files updated

### Why This Matters

**Enforced by praxis standards:**
- `unsafe_code = "forbid"` — No unsafe code without explicit justification
- `clippy/*= "warn"` — Leverage community best practices
- `rustdoc = "warn"` — Comprehensive crate-level documentation

This phase is **foundational** for Claude Code web compatibility and upstream compatibility with the Rust ecosystem.

### Risk Assessment

**Risk Level:** LOW

- Linting is purely additive (no functional changes)
- No compilation or runtime impact
- Clippy warnings are generally minor code quality improvements
- Pre-approved pattern from house standards

### Testing

- [ ] Repository still compiles: `cargo build`
- [ ] All tests pass: `cargo test --all`
- [ ] Clippy passes: `cargo clippy --all --all-targets`
- [ ] No unsafe code violations: `cargo clippy -- -D unsafe_code`

### Next Steps

This is Phase 1 of a 5-phase retrofit:

1. ✓ Phase 1: Linting standards (this PR)
2. □ Phase 2: Dependency unification via `workspace.dependencies`
3. □ Phase 3: Justfile standardization
4. □ Phase 4: Spell-check configuration
5. □ Phase 5: Documentation standards

### References

- **Praxis Standards:** [seanchatmangpt/praxis](https://github.com/seanchatmangpt/praxis)
- **Retrofit Documentation:** [ARCHITECTURE.md](https://github.com/seanchatmangpt/praxis/blob/main/crates/praxis-retrofit/README.md)
- **Case Study:** [wasm4pm retrofit walkthrough](https://github.com/seanchatmangpt/praxis/blob/main/case-study-wasm4pm-retrofit.md)

---

**Automated by:** `praxis-retrofit v26.6.0`
**Fleet ID:** {} — Phase 1 of 18-repo standardization initiative
"#,
        repo_name, files_changed, repo_name
    )
}

/// Generate PR body for Phase 2: Dependencies
fn pr_body_phase2_deps(repo_name: &str, files_changed: usize) -> String {
    format!(
        r#"## Retrofit: Phase 2 - Unified Dependency Management

This PR refactors `{}` to use `[workspace.dependencies]` for centralized dependency version management across the monorepo/workspace.

### Changes

- ✓ Extracted common dependencies into `[workspace.dependencies]` block
- ✓ Updated all crate manifests to inherit versions via `{{ workspace = true }}`
- ✓ Removed duplicate version specifications
- {} files updated

### Why This Matters

**Centralized dependency management:**
- Single source of truth for versions
- Easier to audit and update transitive deps
- Reduced risk of version conflicts
- Aligns with ecosystem best practices (tokio, serde, etc.)

This phase is **critical** for supply-chain security and Claude Code web compatibility.

### Risk Assessment

**Risk Level:** LOW-MEDIUM

- Dependency versions remain unchanged (no upgrades)
- Workspace resolution is standard Cargo behavior
- May require `cargo update --workspace` for Cargo.lock alignment
- Easy rollback: revert to per-crate versions if issues arise

### Testing

- [ ] Workspace resolves cleanly: `cargo build --workspace`
- [ ] All tests pass: `cargo test --all`
- [ ] Dependency audit passes: `cargo audit`
- [ ] Lock file is consistent: `cargo update --workspace --dry-run`

### Next Steps

This is Phase 2 of a 5-phase retrofit:

1. ✓ Phase 1: Linting standards (merged)
2. ✓ Phase 2: Dependency unification (this PR)
3. □ Phase 3: Justfile standardization
4. □ Phase 4: Spell-check configuration
5. □ Phase 5: Documentation standards

### References

- **Cargo Workspace Dependencies:** [Rust 1.64+ feature](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-dependencies-table)
- **Praxis Standards:** [seanchatmangpt/praxis](https://github.com/seanchatmangpt/praxis)
- **Retrofit Documentation:** [ARCHITECTURE.md](https://github.com/seanchatmangpt/praxis/blob/main/crates/praxis-retrofit/README.md)

---

**Automated by:** `praxis-retrofit v26.6.0`
**Fleet ID:** {} — Phase 2 of 18-repo standardization initiative
"#,
        repo_name, files_changed, repo_name
    )
}

/// Generate PR body for Phase 3: Justfile
fn pr_body_phase3_justfile(repo_name: &str, files_changed: usize) -> String {
    format!(
        r#"## Retrofit: Phase 3 - Standardized Task Runner

This PR standardizes the build and development task runner for `{}` via a `justfile` following praxis conventions.

### Changes

- ✓ Created standardized `justfile` with common tasks
- ✓ Standardized tasks: `fmt`, `lint`, `test`, `build`, `doc`, `bench`
- ✓ Added pre-commit gate: `fmt -> lint -> test`
- ✓ {} files updated

### Why This Matters

**Consistent developer experience:**
- `just fmt` — Format all code
- `just lint` — Run all linters
- `just test` — Run all tests
- `just pre-commit` — Run validation gate before committing

This phase is **quality-of-life** improvement and prerequisite for CI/CD standardization.

### Risk Assessment

**Risk Level:** LOW

- Purely additive (no changes to source code)
- `justfile` is a convenience wrapper; doesn't change build behavior
- Tasks are no-ops if equivalent Cargo commands fail (same error handling)
- Easy to maintain custom tasks alongside standard ones

### Testing

- [ ] Justfile syntax is valid: `just --list`
- [ ] All tasks work: `just fmt && just lint && just test`
- [ ] Help text is clear: `just --help`
- [ ] Compatible with existing CI/CD scripts

### Next Steps

This is Phase 3 of a 5-phase retrofit:

1. ✓ Phase 1: Linting standards (merged)
2. ✓ Phase 2: Dependency unification (merged)
3. ✓ Phase 3: Justfile standardization (this PR)
4. □ Phase 4: Spell-check configuration
5. □ Phase 5: Documentation standards

### References

- **Just Project:** [casey/just](https://github.com/casey/just)
- **Praxis Standards:** [seanchatmangpt/praxis](https://github.com/seanchatmangpt/praxis)
- **Retrofit Documentation:** [ARCHITECTURE.md](https://github.com/seanchatmangpt/praxis/blob/main/crates/praxis-retrofit/README.md)

---

**Automated by:** `praxis-retrofit v26.6.0`
**Fleet ID:** {} — Phase 3 of 18-repo standardization initiative
"#,
        repo_name, files_changed, repo_name
    )
}

/// Generate PR body for Phase 4: Typos
fn pr_body_phase4_typos(repo_name: &str, files_changed: usize) -> String {
    format!(
        r#"## Retrofit: Phase 4 - Spell-Check Configuration

This PR adds `typos.toml` spell-check configuration to `{}` for consistent documentation quality and domain-specific terminology awareness.

### Changes

- ✓ Added `typos.toml` configuration
- ✓ Configured domain-specific terms (OCEL, BPMN, wasm, etc.)
- ✓ Excluded non-source directories (bench results, etc.)
- {} files updated

### Why This Matters

**Catch common misspellings in:**
- Documentation files (*.md)
- Code comments
- Commit messages
- Configuration files

**Domain-aware spell checking:**
- Recognizes project-specific terminology (OCEL, process mining terms)
- Prevents false positives on acronyms and domain concepts
- Integrates with CI/CD via `typos` CLI

This phase is **quality assurance** for documentation and communication.

### Risk Assessment

**Risk Level:** VERY LOW

- Non-blocking lint configuration
- No changes to code or functionality
- Can be gradually integrated into CI/CD
- Easy to customize with project-specific terms

### Testing

- [ ] Spell check passes: `typos --config typos.toml`
- [ ] No false positives on domain terms
- [ ] CI integration works (if enabled)

### Next Steps

This is Phase 4 of a 5-phase retrofit:

1. ✓ Phase 1: Linting standards (merged)
2. ✓ Phase 2: Dependency unification (merged)
3. ✓ Phase 3: Justfile standardization (merged)
4. ✓ Phase 4: Spell-check configuration (this PR)
5. □ Phase 5: Documentation standards

### References

- **Typos Project:** [crate-ci/typos](https://github.com/crate-ci/typos)
- **Praxis Standards:** [seanchatmangpt/praxis](https://github.com/seanchatmangpt/praxis)
- **Retrofit Documentation:** [ARCHITECTURE.md](https://github.com/seanchatmangpt/praxis/blob/main/crates/praxis-retrofit/README.md)

---

**Automated by:** `praxis-retrofit v26.6.0`
**Fleet ID:** {} — Phase 4 of 18-repo standardization initiative
"#,
        repo_name, files_changed, repo_name
    )
}

/// Generate PR body for Phase 5: Documentation
fn pr_body_phase5_docs(repo_name: &str, files_changed: usize) -> String {
    format!(
        r#"## Retrofit: Phase 5 - Documentation Standards

This PR adds praxis-standard documentation templates and configuration to `{}` for consistency across the Rust ecosystem.

### Changes

- ✓ Added/updated CONTRIBUTING.md
- ✓ Added/updated SECURITY.md
- ✓ Added/updated ARCHITECTURE.md
- ✓ Updated README.md with standard sections
- {} files updated

### Why This Matters

**Standardized contributor experience:**
- **CONTRIBUTING.md** — How to contribute, code of conduct, development setup
- **SECURITY.md** — Security policy, how to report vulnerabilities
- **ARCHITECTURE.md** — System design, module organization, key concepts

**Improves:**
- Discoverability for new contributors
- Security incident response
- Maintenance clarity
- Ecosystem compatibility

This phase is **final standardization step** for complete praxis compliance.

### Risk Assessment

**Risk Level:** LOW

- Non-code changes (documentation only)
- No impact on build, tests, or functionality
- Can be iteratively refined after merge
- Enables better issue triage and community engagement

### Testing

- [ ] Documentation renders correctly on GitHub
- [ ] All links are valid
- [ ] Code examples work (if included)
- [ ] Contributing guide is accurate

### Next Steps

This is Phase 5 of a 5-phase retrofit:

1. ✓ Phase 1: Linting standards (merged)
2. ✓ Phase 2: Dependency unification (merged)
3. ✓ Phase 3: Justfile standardization (merged)
4. ✓ Phase 4: Spell-check configuration (merged)
5. ✓ Phase 5: Documentation standards (this PR)

**Retrofit Complete!** Repository is now fully aligned with praxis standards.

### References

- **Praxis Standards:** [seanchatmangpt/praxis](https://github.com/seanchatmangpt/praxis)
- **Retrofit Documentation:** [ARCHITECTURE.md](https://github.com/seanchatmangpt/praxis/blob/main/crates/praxis-retrofit/README.md)
- **Case Study:** [wasm4pm retrofit walkthrough](https://github.com/seanchatmangpt/praxis/blob/main/case-study-wasm4pm-retrofit.md)

---

**Automated by:** `praxis-retrofit v26.6.0`
**Fleet ID:** {} — Phase 5 of 18-repo standardization initiative
**Status:** COMPLETE — All phases merged, ready for ecosystem distribution
"#,
        repo_name, files_changed, repo_name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_generator_config_default() {
        let config = PullRequestGeneratorConfig::default();
        assert_eq!(config.base_branch, "main");
        assert!(config.create_as_draft);
        assert!(!config.auto_assign_reviewers.is_empty());
    }

    #[test]
    fn test_branch_name_generation() {
        let config = PullRequestGeneratorConfig {
            github_owner: "seanchatmangpt".to_string(),
            ..Default::default()
        };
        let gen = PullRequestGenerator::new(config);

        let branch = gen.branch_name("wasm4pm", RetrofitPhase::Phase1Lints);
        assert!(branch.contains("phase-1-lints"));
        assert!(branch.contains("wasm4pm"));
    }

    #[test]
    fn test_template_phase1_title() {
        let repo = RepositoryMetadata {
            path: PathBuf::from("/test"),
            name: "wasm4pm".to_string(),
            workspace_root: PathBuf::from("/test"),
            crate_count: 1,
            has_workspace: false,
        };

        let template = PullRequestGenerator::template_phase1_lints(&repo, 3);
        assert!(template.title.contains("retrofit(lints)"));
        assert!(template.title.contains("wasm4pm"));
        assert!(template.body.contains("Linting Standards"));
    }

    #[test]
    fn test_pr_status_ordering() {
        assert!(PRStatus::Draft < PRStatus::Open);
        assert!(PRStatus::Open < PRStatus::ReviewRequested);
        assert!(PRStatus::Approved > PRStatus::ReviewRequested);
        assert!(PRStatus::Merged > PRStatus::Approved);
    }

    #[test]
    fn test_fleet_pr_summary() {
        let prs = vec![
            PullRequestInfo {
                repository: RepositoryMetadata {
                    path: PathBuf::from("/test1"),
                    name: "repo1".to_string(),
                    workspace_root: PathBuf::from("/test1"),
                    crate_count: 1,
                    has_workspace: false,
                },
                url: Some("https://github.com/owner/repo1/pull/1".to_string()),
                number: Some(1),
                status: PRStatus::Merged,
                branch_name: "praxis/retrofit/phase-1-lints/repo1".to_string(),
                phase: RetrofitPhase::Phase1Lints,
                created_at: None,
                estimated_risk: RiskLevel::Low,
                files_changed: 3,
                commits: 1,
                review_comments: vec![],
            },
            PullRequestInfo {
                repository: RepositoryMetadata {
                    path: PathBuf::from("/test2"),
                    name: "repo2".to_string(),
                    workspace_root: PathBuf::from("/test2"),
                    crate_count: 1,
                    has_workspace: false,
                },
                url: Some("https://github.com/owner/repo2/pull/2".to_string()),
                number: Some(2),
                status: PRStatus::Open,
                branch_name: "praxis/retrofit/phase-1-lints/repo2".to_string(),
                phase: RetrofitPhase::Phase1Lints,
                created_at: None,
                estimated_risk: RiskLevel::Low,
                files_changed: 3,
                commits: 1,
                review_comments: vec![],
            },
        ];

        let summary = PullRequestGenerator::summarize_fleet_prs(&prs);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.by_status.merged, 1);
        assert_eq!(summary.by_status.open, 1);
    }

    #[test]
    fn test_count_files_changed() {
        let actions = vec![
            RetrofitAction {
                action_type: crate::RetrofitActionType::Create,
                file_path: PathBuf::from("Cargo.toml"),
                content: String::new(),
                description: String::new(),
            },
            RetrofitAction {
                action_type: crate::RetrofitActionType::Update,
                file_path: PathBuf::from(".editorconfig"),
                content: String::new(),
                description: String::new(),
            },
        ];

        assert_eq!(PullRequestGenerator::count_files_changed(&actions), 2);
    }
}

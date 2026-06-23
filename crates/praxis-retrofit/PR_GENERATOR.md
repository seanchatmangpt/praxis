# PR Generator: Mass Retrofit Automation

## Overview

The `pr_generator` module automates the creation and tracking of pull requests across the 18-repo seanchatmangpt Rust ecosystem. It provides:

1. **PR Template System** — Conventional commit-based templates for each retrofit phase
2. **GitHub Integration** — Uses `gh` CLI for creating and tracking PRs
3. **Fleet Tracking** — Monitor PR status, review progress, and merge status across all repos
4. **Automated Status Reports** — Generate compliance dashboards and review queues

## Architecture

### Components

#### 1. PullRequestTemplate
Defines a single PR with standardized structure:
```rust
pub struct PullRequestTemplate {
    pub title: String,           // Conventional commit format
    pub body: String,            // Full markdown with phase info
    pub labels: Vec<String>,     // Auto-assigned labels
    pub assignees: Vec<String>,  // Suggested reviewers
}
```

#### 2. PullRequestInfo
Tracks individual PR metadata:
```rust
pub struct PullRequestInfo {
    pub repository: RepositoryMetadata,
    pub url: Option<String>,
    pub number: Option<usize>,
    pub status: PRStatus,
    pub branch_name: String,
    pub phase: RetrofitPhase,
    pub created_at: Option<String>,
    pub estimated_risk: RiskLevel,
    pub files_changed: usize,
    pub commits: usize,
    pub review_comments: Vec<String>,
}
```

#### 3. PullRequestGenerator
Main orchestrator for PR operations:
```rust
pub struct PullRequestGenerator {
    config: PullRequestGeneratorConfig,
}
```

Provides methods:
- `template_for_phase()` — Generate PR template for any phase
- `branch_name()` — Create standardized branch name
- `create_pull_request()` — Create PR via `gh` CLI
- `fetch_pr_status()` — Poll current PR status from GitHub
- `summarize_fleet_prs()` — Generate fleet-wide summary

#### 4. FleetPRStatus
Summary across all repositories:
```rust
pub struct FleetPRStatus {
    pub pull_requests: Vec<PullRequestInfo>,
    pub total: usize,
    pub by_status: PRStatusCounts,
    pub generated_at: String,
}
```

## PR Lifecycle & Status Tracking

### PRStatus Enum
```rust
pub enum PRStatus {
    NotCreated,        // PR not yet created
    Draft,             // Draft PR (work in progress)
    Open,              // Open for review
    ReviewRequested,   // Reviewers have been assigned
    Approved,          // Approved and ready to merge
    ChangesRequested,  // Requested changes from reviewers
    Merged,            // Successfully merged
    Closed,            // Closed without merge
}
```

### Status Flow
```
NotCreated
    ↓
Draft (if create_as_draft=true)
    ↓
Open
    ↓
ReviewRequested
    ├→ ChangesRequested → Open (iterate)
    ├→ Approved
    │    ↓
    │ Merged
    │
    └→ Closed (without merge)
```

## PR Templates for Each Phase

### Phase 1: Linting Standards
```markdown
## Retrofit: Phase 1 - Workspace Linting Standards

- ✓ Added [lints] configuration block
- ✓ Configured: unsafe_code = "forbid"
- ✓ Added Clippy warnings
- X files updated

Risk Level: LOW
```

**Use Case:** Add strict linting configuration to enforce code quality standards.
**Impact:** Additive only; no functional changes.

### Phase 2: Dependency Unification
```markdown
## Retrofit: Phase 2 - Unified Dependency Management

- ✓ Extracted common deps to [workspace.dependencies]
- ✓ Updated crates to inherit versions
- X files updated

Risk Level: LOW-MEDIUM
```

**Use Case:** Centralize dependency version management.
**Impact:** Version pinning changes; requires cargo update.

### Phase 3: Justfile Standardization
```markdown
## Retrofit: Phase 3 - Standardized Task Runner

- ✓ Created standardized justfile
- ✓ Added: fmt, lint, test, build, doc, bench
- ✓ Pre-commit gate: fmt → lint → test
- X files updated

Risk Level: LOW
```

**Use Case:** Standardize developer experience.
**Impact:** Pure convenience wrapper; no code changes.

### Phase 4: Spell-Check Configuration
```markdown
## Retrofit: Phase 4 - Spell-Check Configuration

- ✓ Added typos.toml
- ✓ Configured domain-specific terms
- X files updated

Risk Level: VERY LOW
```

**Use Case:** Catch misspellings in docs and comments.
**Impact:** Non-blocking lint; can be gradual.

### Phase 5: Documentation Standards
```markdown
## Retrofit: Phase 5 - Documentation Standards

- ✓ Added/updated CONTRIBUTING.md
- ✓ Added/updated SECURITY.md
- ✓ Added/updated ARCHITECTURE.md
- X files updated

Risk Level: LOW
```

**Use Case:** Standardize contributor experience.
**Impact:** Documentation only; no code changes.

## Configuration

### PullRequestGeneratorConfig
```rust
pub struct PullRequestGeneratorConfig {
    pub github_owner: String,              // "seanchatmangpt"
    pub create_as_draft: bool,             // Start as draft
    pub auto_assign_reviewers: Vec<String>, // ["@seanchatmangpt"]
    pub labels: Vec<String>,               // ["retrofit", "praxis"]
    pub base_branch: String,               // "main"
    pub branch_prefix: String,             // "praxis/retrofit"
}
```

### Example Configuration
```rust
let config = PullRequestGeneratorConfig {
    github_owner: "seanchatmangpt".to_string(),
    create_as_draft: true,  // Start as draft for review
    auto_assign_reviewers: vec!["@seanchatmangpt".to_string()],
    labels: vec![
        "retrofit".to_string(),
        "praxis".to_string(),
    ],
    base_branch: "main".to_string(),
    branch_prefix: "praxis/retrofit".to_string(),
};

let generator = PullRequestGenerator::new(config);
```

## Usage Patterns

### Pattern 1: Generate PR Template

```rust
use praxis_retrofit::{
    PullRequestGenerator, PullRequestGeneratorConfig,
    RepositoryMetadata, RetrofitPhase,
};

let config = PullRequestGeneratorConfig::default();
let gen = PullRequestGenerator::new(config);

let repo = RepositoryMetadata { /* ... */ };

let template = PullRequestGenerator::template_phase1_lints(&repo, 5);
println!("Title: {}", template.title);
println!("Body:\n{}", template.body);
```

### Pattern 2: Create Pull Request

```rust
let repo_path = std::path::PathBuf::from("/path/to/repo");

let pr_info = gen.create_pull_request(
    &repo_path,
    &repo,
    &template,
    RetrofitPhase::Phase1Lints,
)?;

println!("Created PR: {}", pr_info.url.unwrap());
println!("Number: {}", pr_info.number.unwrap());
```

### Pattern 3: Check PR Status

```rust
let status = gen.fetch_pr_status(&repo_path, 42)?;
println!("PR #42 status: {:?}", status);
```

### Pattern 4: Fleet Summary

```rust
let all_prs = vec![
    pr_info1,
    pr_info2,
    // ... more PRs
];

let summary = PullRequestGenerator::summarize_fleet_prs(&all_prs);
println!("Total: {}", summary.total);
println!("Merged: {}", summary.by_status.merged);
println!("Open: {}", summary.by_status.open);
println!("Draft: {}", summary.by_status.draft);
```

## Integration with GitHub

### Prerequisites

1. **`gh` CLI installed** — [Install instructions](https://cli.github.com)
2. **GitHub authentication** — `gh auth login`
3. **Repository access** — Write access to all 18 repos

### Creating PRs

The module uses `gh pr create` to create PRs:

```bash
gh pr create \
  -B main \
  -H praxis/retrofit/phase-1-lints/wasm4pm \
  -t "retrofit(lints): Add praxis workspace linting standards for wasm4pm" \
  -b "## Retrofit: Phase 1 - Workspace Linting Standards\n\n..."
```

### Fetching PR Status

Queries GitHub API via `gh pr view`:

```bash
gh pr view <number> \
  -R seanchatmangpt/repo \
  --json state
```

## Mass Retrofit Workflow

### Full 5-Phase Retrofit (Ideal Timeline)

```
Week 1: Phase 1 (Lints)
  ├─ Create 18 PRs
  └─ Merge when CI passes

Week 2: Phase 2 (Dependencies)
  ├─ Create 18 PRs
  └─ Merge when CI passes

Week 3: Phase 3 (Justfile)
  ├─ Create 18 PRs
  └─ Merge when CI passes

Week 4: Phase 4 (Typos)
  ├─ Create 18 PRs
  └─ Merge when CI passes

Week 5: Phase 5 (Documentation)
  ├─ Create 18 PRs
  └─ Merge when CI passes

Result: All 18 repos fully compliant with praxis standards
```

### Automated Workflow Script

```rust
use praxis_retrofit::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = PullRequestGeneratorConfig {
        github_owner: "seanchatmangpt".to_string(),
        create_as_draft: true,
        ..Default::default()
    };
    let gen = PullRequestGenerator::new(config);

    let repos = vec![
        "wasm4pm",
        "pm4py-rs",
        "dteam",
        // ... 15 more repos
    ];

    let mut all_prs = Vec::new();

    for repo_name in repos {
        let repo = RepositoryMetadata { /* ... */ };
        let template = PullRequestGenerator::template_phase1_lints(&repo, 5);
        
        match gen.create_pull_request(&repo_path, &repo, &template, RetrofitPhase::Phase1Lints) {
            Ok(pr) => {
                println!("✓ Created PR for {}: {}", repo_name, pr.url.unwrap());
                all_prs.push(pr);
            }
            Err(e) => eprintln!("✗ Failed to create PR for {}: {}", repo_name, e),
        }
    }

    // Generate summary
    let summary = PullRequestGenerator::summarize_fleet_prs(&all_prs);
    println!("\n=== Fleet Summary ===");
    println!("Total: {}", summary.total);
    println!("Open: {}", summary.by_status.open);
    println!("Draft: {}", summary.by_status.draft);

    Ok(())
}
```

## PR Body Template Examples

### Phase 1: Lints

```markdown
## Retrofit: Phase 1 - Workspace Linting Standards

This PR adds praxis house-style linting configuration to `wasm4pm`, bringing it into 
compliance with the standardized Rust ecosystem practices.

### Changes

- ✓ Added `[lints]` configuration block to workspace `Cargo.toml`
- ✓ Configured strict linting rules: `unsafe_code = "forbid"`
- ✓ Added Clippy warnings for `all`, `pedantic`, `nursery`
- ✓ Added rustdoc warnings for documentation completeness
- 5 files updated

### Why This Matters

**Enforced by praxis standards:**
- `unsafe_code = "forbid"` — No unsafe code without explicit justification
- `clippy/*= "warn"` — Leverage community best practices
- `rustdoc = "warn"` — Comprehensive crate-level documentation

This phase is **foundational** for Claude Code web compatibility and upstream compatibility 
with the Rust ecosystem.

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

---

**Automated by:** `praxis-retrofit v26.6.0`
**Fleet ID:** wasm4pm — Phase 1 of 18-repo standardization initiative
```

## Monitoring and Dashboards

### PR Status Dashboard

Track all 18 repos:

```
Repository      Status          Phase         Files  Commits
────────────────────────────────────────────────────────────
wasm4pm         ✓ Merged       Phase 1 Lints   3      1
pm4py-rs        ⏳ Open        Phase 1 Lints   4      1
dteam           👤 Review      Phase 1 Lints   2      1
miniml          📝 Draft       Phase 2 Deps    8      2
prolog8         ❌ Failed      Phase 1 Lints   0      0
ocpq            ⏳ Open        Phase 1 Lints   5      1
ggen-mcp        ⏳ Open        Phase 1 Lints   3      1
a2a-rs          ⏳ Open        Phase 1 Lints   4      1
semantic_bit    ⏳ Open        Phase 1 Lints   2      1
...
```

### Metrics

- **Total PRs:** 18/18 created
- **Merged:** 5/18 (28%)
- **Open:** 10/18 (56%)
- **Draft:** 2/18 (11%)
- **Failed:** 1/18 (5%)

## Error Handling

The module returns `crate::Result<T>` for all fallible operations:

```rust
pub type Result<T> = std::result::Result<T, RetrofitError>;

pub enum RetrofitError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Retrofit failed: {0}")]
    RetrofitFailed(String),
    
    // ... other variants
}
```

Common error scenarios:

1. **`gh` CLI not installed** → `RetrofitFailed("gh pr create failed: command not found")`
2. **Not authenticated** → `RetrofitFailed("gh pr create failed: authentication required")`
3. **No remote configured** → `RetrofitFailed("gh pr create failed: no remote found")`
4. **Rate limiting** → `RetrofitFailed("gh pr create failed: API rate limit exceeded")`

## Testing

Run the example:

```bash
cargo run --example pr_generation_demo
```

Output:
```
=== Praxis Retrofit: PR Generation Demo ===

Configuration:
PullRequestGeneratorConfig {
    github_owner: "seanchatmangpt",
    create_as_draft: true,
    auto_assign_reviewers: ["@seanchatmangpt"],
    labels: ["retrofit", "praxis", "automated"],
    base_branch: "main",
    branch_prefix: "praxis/retrofit",
}

Sample Repository: wasm4pm
Path: /home/seanchatmangpt/wasm4pm

--- Phase 1: Lints ---
Title:
  retrofit(lints): Add praxis workspace linting standards for wasm4pm

...
```

## Integration Points

### With CI/CD

Each PR body includes a **Testing** checklist:

```markdown
### Testing

- [ ] Repository still compiles: `cargo build`
- [ ] All tests pass: `cargo test --all`
- [ ] Clippy passes: `cargo clippy --all --all-targets`
```

GitHub Actions can auto-check these boxes when CI passes.

### With Compliance Dashboard

The module provides data for:
- Retrofit progress tracking
- Risk assessment per phase
- Review queue management
- Merge status monitoring

### With Audit System

Correlate PR status with:
- Pre-retrofit compliance scores
- Post-retrofit target compliance
- Phase-by-phase progress

## Conventional Commits Format

All PR titles follow conventional commits:

```
<type>(<scope>): <subject>

Examples:
- retrofit(lints): Add praxis workspace linting standards for wasm4pm
- retrofit(deps): Unify dependencies via workspace.dependencies for pm4py-rs
- retrofit(build): Standardize justfile task runner for dteam
- retrofit(ci): Add praxis spell-check configuration for miniml
- retrofit(docs): Add praxis documentation standards for wasm4pm
```

**Type:** Always `retrofit` (fleet-wide standardization)
**Scope:** Phase name (lints, deps, build, ci, docs)
**Subject:** Specific repo and action

## Performance Considerations

- **PR Creation:** ~200ms per repo (network latency with GitHub API)
- **Status Polling:** ~150ms per PR (gh CLI call)
- **Fleet Summary:** O(n) where n = number of PRs

For 18 repos in parallel:
- **Create all PRs:** ~4 seconds (with concurrent requests)
- **Poll all statuses:** ~3 seconds (with concurrent requests)

## Future Enhancements

1. **Auto-merge on CI success** — Merge when checks pass
2. **Review assignment algorithm** — Route PRs to appropriate reviewers
3. **Dependency graph analysis** — Suggest merge order to avoid conflicts
4. **Rollback support** — Revert retrofit if post-merge validation fails
5. **Custom PR templates** — Load from YAML config file
6. **Slack integration** — Notify team of PR creation and status changes
7. **Batch operations** — Create/update/merge multiple PRs atomically

## References

- **Praxis Standards:** https://github.com/seanchatmangpt/praxis
- **Retrofit Documentation:** [ARCHITECTURE.md](./README.md)
- **Case Study:** [wasm4pm retrofit walkthrough](../case-study-wasm4pm-retrofit.md)
- **GitHub CLI:** https://cli.github.com
- **Conventional Commits:** https://www.conventionalcommits.org/

## Troubleshooting

### "gh: command not found"
Install GitHub CLI: https://cli.github.com/manual/installation

### "Authentication required"
Run: `gh auth login`

### "No remote repository found"
Ensure `.git/config` has a `remote.origin.url` entry

### "API rate limit exceeded"
GitHub limits API calls. Wait 1 hour or use a personal access token:
```bash
gh auth login --with-token < github_token.txt
```

---

**Module:** `praxis_retrofit::pr_generator`
**Version:** 26.6.0
**License:** MIT OR Apache-2.0

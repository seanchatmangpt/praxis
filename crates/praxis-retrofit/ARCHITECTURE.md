# Fleet Apply Architecture & Design

Comprehensive documentation of the automated retrofit application system's architecture, design patterns, and implementation details.

## Table of Contents

1. [System Overview](#system-overview)
2. [Component Architecture](#component-architecture)
3. [Data Flow](#data-flow)
4. [Design Patterns](#design-patterns)
5. [Error Handling Strategy](#error-handling-strategy)
6. [Concurrency Model](#concurrency-model)
7. [Git Worktree Management](#git-worktree-management)
8. [Phase Management](#phase-management)
9. [Validation Strategy](#validation-strategy)
10. [Future Extensibility](#future-extensibility)

---

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Fleet Apply System                        │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐                                            │
│  │ RetrofitApplier                                          │
│  │ ────────────                                             │
│  │ - spec: PraxisSpec                                       │
│  │ - repositories: Vec<(Path, Phase)>                       │
│  │ - concurrent_limit: usize                                │
│  │                                                           │
│  │ + add_repository()                                       │
│  │ + apply_all()                                            │
│  │ + summary()                                              │
│  └──────────────┘                                            │
│         │                                                    │
│         ├─────────────┬─────────────┬─────────────┐         │
│         ↓             ↓             ↓             ↓         │
│    ┌──────────────────────────────────────────────────┐     │
│    │ RetrofitWorktree (1 per repo)                    │     │
│    │ ────────────────────────────────────────         │     │
│    │ - original_path: PathBuf                         │     │
│    │ - worktree_path: PathBuf                         │     │
│    │ - current_branch: String                         │     │
│    │                                                  │     │
│    │ + apply_plan()                                  │     │
│    │ + validate()                                    │     │
│    │ + commit()                                      │     │
│    │ + cleanup()                                     │     │
│    └──────────────────────────────────────────────────┘     │
│         │                  │                  │             │
│         ↓                  ↓                  ↓             │
│    ┌─────────┐       ┌──────────┐      ┌──────────┐        │
│    │ Plan    │       │ Validate │      │ Commit   │        │
│    │ Generate│       │ & Report │      │ & Branch │        │
│    └─────────┘       └──────────┘      └──────────┘        │
│         │
│         ↓
│    ┌──────────────┐
│    │ ApplyResult  │
│    │ (per repo)   │
│    └──────────────┘
│         │
│         ├─────────────────────┐
│         ↓                     ↓
│    ┌──────────────┐    ┌──────────────────┐
│    │ Success      │    │ Failure with     │
│    │ Results      │    │ Error & Warnings │
│    └──────────────┘    └──────────────────┘
│         │
│         └─────────────────┐
│                           ↓
│                 ┌──────────────────┐
│                 │ FleetApplyReport │
│                 │ (aggregated)     │
│                 └──────────────────┘
│
└─────────────────────────────────────────────────────────────┘
```

---

## Component Architecture

### Layer Model

```
┌─────────────────────────────────────────┐
│         Application Layer               │
│  (CLI, Dashboard, Monitoring)           │
├─────────────────────────────────────────┤
│         Fleet Layer                     │
│  RetrofitApplier, FleetApplyReport     │
├─────────────────────────────────────────┤
│         Worktree Layer                  │
│  RetrofitWorktree, ApplyResult         │
├─────────────────────────────────────────┤
│         Git Layer                       │
│  Git commands via subprocess            │
├─────────────────────────────────────────┤
│         Retrofit Layer                  │
│  generate, apply, validate modules      │
├─────────────────────────────────────────┤
│         Core Layer                      │
│  Models, Error types, Specifications    │
└─────────────────────────────────────────┘
```

### Responsibilities

| Component | Responsibility | Scope |
|-----------|---|---|
| RetrofitApplier | Fleet coordination, result aggregation | Global |
| RetrofitWorktree | Isolated worktree lifecycle, git operations | Per-repository |
| ApplyResult | Individual operation outcome tracking | Per-repository |
| FleetApplyReport | Summary statistics, reporting | Fleet-wide |

---

## Data Flow

### Complete Retrofit Workflow

```
1. Initialization
   ↓
   Input: Vec<(repo_path, phase)>
   ├─ Validate repository existence
   ├─ Store in RetrofitApplier.repositories
   └─ State: Ready

2. Worktree Creation (per repo)
   ↓
   ├─ Detect default branch
   ├─ Create phase-specific branch
   ├─ Create git worktree
   ├─ Get remote URL
   └─ State: Worktree ready

3. Plan Generation (per repo)
   ↓
   ├─ Scan worktree directory
   ├─ Determine applicable changes
   ├─ Create RetrofitPlan
   └─ State: Plan ready

4. Plan Application (per repo)
   ↓
   ├─ For each action in plan:
   │  ├─ Create/Update/Delete file
   │  └─ Log result
   └─ State: Changes applied

5. Validation (per repo)
   ↓
   ├─ Run validation checks
   ├─ Collect validation results
   └─ State: Validated

6. Commit (per repo)
   ↓
   ├─ Stage all changes
   ├─ Create commit
   ├─ Extract commit hash
   └─ State: Committed

7. Cleanup (per repo)
   ↓
   ├─ Remove worktree
   ├─ Remove temp directory
   └─ State: Cleaned

8. Result Collection
   ↓
   ├─ Gather ApplyResult
   ├─ Aggregate into FleetApplyReport
   └─ State: Complete

9. Reporting
   ↓
   Output: FleetApplyReport with:
   - Success rate
   - Commit hashes
   - Error messages
   - Execution timings
```

### State Transitions

```
RetrofitWorktree States:

  new()           create worktree
    ↓
  [READY]  ────────────────────
    │                          │
    ├──→ apply_plan()         │
    │      ↓                   │
    │    [APPLIED]            │
    │      ├──→ validate()    │
    │      │      ↓           │
    │      │   [VALIDATED]    │
    │      │      ├──→ commit()
    │      │      │   ↓
    │      │      │ [COMMITTED]
    │      │      │   └──→ cleanup()
    │      │      └────────────→ [CLEANED]
    │      └────────────────────→ [CLEANED]
    │
    └─→ cleanup()
       ↓
      [CLEANED]
```

---

## Design Patterns

### 1. Builder Pattern (RetrofitApplier)

```rust
let applier = RetrofitApplier::new(spec)?
    .with_concurrent_limit(8);

applier.add_repository(repo1, phase)?;
applier.add_repository(repo2, phase)?;

let results = applier.apply_all().await?;
```

**Benefits:**
- Fluent API
- Sensible defaults
- Configuration before execution

### 2. RAII Pattern (RetrofitWorktree)

```rust
pub struct RetrofitWorktree { ... }

impl Drop for RetrofitWorktree {
    fn drop(&mut self) {
        if let Err(e) = self.cleanup() {
            warn!("Failed to clean up worktree: {}", e);
        }
    }
}
```

**Benefits:**
- Automatic resource cleanup
- Exception safety
- No manual cleanup needed

### 3. Trait Objects & Serialization

```rust
#[derive(Serialize, Deserialize)]
pub struct ApplyResult { ... }

#[derive(Serialize, Deserialize)]
pub struct FleetApplyReport { ... }
```

**Benefits:**
- JSON export/import
- Persistent storage
- Integration with other systems

### 4. Error Resilience

```rust
// Fleet operation continues despite individual failures
for (repo_path, phase) in &self.repositories {
    let result = self.apply_single(repo_path, *phase).await;
    results.push(result);  // Even if result is failure
}
```

**Benefits:**
- Partial success is still valuable
- Visibility into failures
- Batch error reporting

### 5. Async/Await for I/O

```rust
pub async fn apply_all(&self) -> Result<Vec<ApplyResult>> {
    let mut results = Vec::new();
    
    for (repo_path, phase) in &self.repositories {
        let result = self.apply_single(repo_path, *phase).await;
        results.push(result);
    }
    
    Ok(results)
}
```

**Benefits:**
- Non-blocking I/O
- Easier to add concurrency
- Scalable to large fleets

---

## Error Handling Strategy

### Layered Error Handling

```
┌─────────────────────────────────────┐
│ Application Layer                   │
│ (Display to user)                   │
├─────────────────────────────────────┤
│ Fleet Layer                         │
│ (Aggregate, continue on error)      │
├─────────────────────────────────────┤
│ Worktree Layer                      │
│ (Capture in ApplyResult)            │
├─────────────────────────────────────┤
│ Git Layer                           │
│ (Convert to RetrofitError)          │
├─────────────────────────────────────┤
│ System Layer                        │
│ (IO, subprocess errors)             │
└─────────────────────────────────────┘
```

### Error Propagation

```rust
// Critical errors (fail immediately)
applier.add_repository(&path, phase)?;  // Fails if repo doesn't exist

// Recoverable errors (logged, result tracks status)
let result = applier.apply_all().await?;
if let Some(err) = &result[0].error {
    println!("Retrofit failed: {}", err);
}
```

### Error Categorization

```rust
pub enum RetrofitError {
    // IO errors (retryable)
    Io(std::io::Error),
    
    // Configuration errors (user action needed)
    RepositoryNotFound(String),
    InvalidCargoToml(String),
    ConfigError(String),
    
    // Serialization errors (programming error)
    Toml(toml::de::Error),
    Json(serde_json::Error),
    
    // Process errors (often retryable)
    RetrofitFailed(String),
    ComplianceFailed(String),
}
```

---

## Concurrency Model

### Current: Sequential Processing

```rust
for (repo_path, phase) in &self.repositories {
    let result = self.apply_single(repo_path, *phase).await;
    results.push(result);
}
```

**Rationale:**
- Simpler error handling
- Better observability
- Safe for git operations
- Easier to debug

### Future: Parallel Processing

```rust
let futures: Vec<_> = self.repositories
    .iter()
    .map(|(path, phase)| self.apply_single(path, *phase))
    .collect();

let results = futures::future::join_all(futures).await;
```

**Trade-offs:**
- Faster (4x with concurrent_limit=4)
- Higher resource usage
- More complex error handling
- Requires rate limiting

### Resource Isolation

Each worktree is completely isolated:

```
Repository A       Repository B       Repository C
/tmp/.../a/       /tmp/.../b/       /tmp/.../c/
├─ .git            ├─ .git            ├─ .git
├─ src/            ├─ src/            ├─ src/
└─ Cargo.toml      └─ Cargo.toml      └─ Cargo.toml
(modified)        (modified)        (modified)
```

**Isolation guarantees:**
- No data corruption
- No interference between repos
- Safe to process in parallel
- Easy cleanup

---

## Git Worktree Management

### Worktree Lifecycle

```
1. Branch Detection
   ├─ Get default branch (main/master)
   ├─ Check if retrofit branch exists
   └─ Create if not found

2. Branch Creation
   ├─ Base from default branch
   ├─ Name: retrofit/phase-N-*
   └─ Store in repository's branch list

3. Worktree Setup
   ├─ Create temporary directory
   ├─ Run: git worktree add <path> <branch>
   ├─ Get remote URL
   └─ Ready for changes

4. Changes Application
   ├─ All changes in worktree
   ├─ Original repo untouched
   └─ Can work on multiple repos

5. Cleanup
   ├─ Run: git worktree remove <path>
   ├─ Remove temporary directory
   └─ Branch remains in original repo
```

### Branch Naming Convention

```
retrofit/phase-1-lints          # Phase 1: Linting
retrofit/phase-2-deps           # Phase 2: Dependencies
retrofit/phase-3-justfile       # Phase 3: Justfile
retrofit/phase-4-typos          # Phase 4: Typos
retrofit/phase-5-docs           # Phase 5: Documentation
```

**Advantages:**
- Hierarchical (retrofit/*)
- Phase-aware
- Self-documenting
- Easy to grep/filter

### Default Branch Detection

```rust
fn get_default_branch(repo_path: &Path) -> Result<String> {
    // Try remote HEAD first
    git symbolic-ref refs/remotes/origin/HEAD
    
    // Fallback to checking for main/master
    git show-ref --verify refs/heads/main
    git show-ref --verify refs/heads/master
    
    // Return first that exists
}
```

**Handles:**
- GitHub default (main)
- GitLab/Bitbucket default (master)
- Custom defaults via remote HEAD

---

## Phase Management

### Phase Definitions

```rust
pub enum RetrofitPhase {
    Phase1Lints,      // [lints] configuration
    Phase2Deps,       // workspace.dependencies
    Phase3Justfile,   // Justfile standardization
    Phase4Typos,      // typos.toml configuration
    Phase5Docs,       // Documentation standards
}
```

### Phase Progression

```
Phase 1 (Lints)
  └─ workspace [lints] configuration
     └─ clippy, rustfmt, etc.
        └─ LOW RISK: Configuration only

Phase 2 (Dependencies)
  └─ workspace.dependencies unification
     └─ Centralize dependency versions
        └─ MEDIUM RISK: May affect compilation

Phase 3 (Justfile)
  └─ Standard justfile with praxis recipes
     └─ test, lint, fmt, bench
        └─ LOW RISK: New file

Phase 4 (Typos)
  └─ typos.toml spell-check
     └─ Dictionary and patterns
        └─ LOW RISK: Configuration only

Phase 5 (Docs)
  └─ Documentation standards
     └─ README, CONTRIBUTING, etc.
        └─ MEDIUM RISK: Overwrites existing
```

### Single-phase vs Multi-phase

**Single Phase:**
```rust
let mut applier = RetrofitApplier::new(spec)?;
applier.add_repository("../repo", RetrofitPhase::Phase1Lints)?;
let results = applier.apply_all().await?;
```

**Multi-Phase (Sequential):**
```rust
for phase in [Phase1Lints, Phase2Deps, Phase3Justfile] {
    let mut applier = RetrofitApplier::new(spec)?;
    applier.add_repository("../repo", phase)?;
    let results = applier.apply_all().await?;
    
    // Validate before next phase
    if !validate_fleet(&results)? {
        break;
    }
}
```

---

## Validation Strategy

### Current Validation

```rust
pub async fn validate_retrofit(_repo_path: &Path) -> Result<bool> {
    Ok(true)  // Placeholder
}
```

### Proposed Extended Validation

```rust
pub async fn validate_retrofit(repo_path: &Path) -> Result<bool> {
    // Check 1: Cargo.toml syntax
    let manifest = std::fs::read_to_string(repo_path.join("Cargo.toml"))?;
    toml::from_str::<toml::Value>(&manifest)?;
    
    // Check 2: Metadata resolution
    let output = Command::new("cargo")
        .arg("metadata")
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        return Ok(false);
    }
    
    // Check 3: Linting passes
    let output = Command::new("cargo")
        .args(&["clippy", "--all-targets"])
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        return Ok(false);
    }
    
    // Check 4: Tests pass
    let output = Command::new("cargo")
        .arg("test")
        .arg("--lib")
        .current_dir(repo_path)
        .output()?;
    if !output.status.success() {
        return Ok(false);
    }
    
    Ok(true)
}
```

### Validation Levels

```
Level 1: Syntax Check
  └─ TOML parsing succeeds
     └─ Risk: Catches typos

Level 2: Dependency Resolution
  └─ Cargo metadata succeeds
     └─ Risk: Catches version conflicts

Level 3: Code Quality
  └─ Clippy passes
     └─ Risk: Catches linting violations

Level 4: Tests Pass
  └─ Test suite succeeds
     └─ Risk: Catches functionality regressions
```

---

## Future Extensibility

### Adding New Phases

```rust
pub enum RetrofitPhase {
    Phase1Lints,
    Phase2Deps,
    Phase3Justfile,
    Phase4Typos,
    Phase5Docs,
    Phase6Custom,        // NEW
}

// Auto-generates: retrofit/phase-6-custom
```

### Custom Validation

```rust
pub trait Validator: Send + Sync {
    async fn validate(&self, repo_path: &Path) -> Result<bool>;
}

pub struct RetrofitWorktree {
    validators: Vec<Box<dyn Validator>>,
}

impl RetrofitWorktree {
    pub fn with_validator(mut self, v: Box<dyn Validator>) -> Self {
        self.validators.push(v);
        self
    }
    
    pub async fn validate(&self) -> Result<bool> {
        for v in &self.validators {
            if !v.validate(self.path()).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
```

### Parallel Processing

```rust
pub struct RetrofitApplier {
    // ...
    executor: Arc<tokio::runtime::Runtime>,
}

pub async fn apply_all(&self) -> Result<Vec<ApplyResult>> {
    let handles: Vec<_> = self.repositories
        .iter()
        .map(|(p, ph)| {
            let path = p.clone();
            let phase = *ph;
            tokio::spawn(self.apply_single(path, phase))
        })
        .collect();
    
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await??);
    }
    Ok(results)
}
```

### Git Integration

```rust
// Future: Push to remote and create PRs
pub async fn create_pull_request(&self, result: &ApplyResult) -> Result<String> {
    // Push branch
    self.push_to_origin()?;
    
    // Create PR via GitHub API
    let pr_url = github::create_pull_request(
        &result.original_path,
        &result.branch_name,
        &result.commit_message,
    ).await?;
    
    Ok(pr_url)
}
```

### Monitoring & Metrics

```rust
pub struct RetrofitMetrics {
    total_repos: Arc<AtomicUsize>,
    successful: Arc<AtomicUsize>,
    failed: Arc<AtomicUsize>,
    total_duration: Arc<AtomicU64>,
}

impl RetrofitApplier {
    pub fn with_metrics(mut self, metrics: RetrofitMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }
}
```

---

## Performance Considerations

### Time Complexity

- **Initialization**: O(1)
- **Per Repository**: O(actions × files)
- **Plan Generation**: O(files in repo)
- **Plan Application**: O(actions)
- **Commit**: O(changes)
- **Cleanup**: O(1)

### Space Complexity

- **Per Repository**: 50-200MB (worktree)
- **Per Result**: ~1KB (metadata)
- **Fleet of 100**: ~10GB (worst case)

### Optimization Opportunities

1. **Lazy Validation** - Only validate changed files
2. **Incremental Checkout** - Sparse checkout for large repos
3. **Parallel Git Operations** - Use git's parallelization
4. **Caching** - Cache validation results between runs

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_branch_name_generation() {
    assert_eq!(
        RetrofitWorktree::branch_name_for_phase(Phase1Lints),
        "retrofit/phase-1-lints"
    );
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_complete_workflow() {
    // Create temp repo
    let repo = create_test_repo()?;
    
    // Run applier
    let mut applier = RetrofitApplier::new(spec)?;
    applier.add_repository(&repo, Phase1Lints)?;
    let results = applier.apply_all().await?;
    
    // Verify results
    assert!(results[0].is_success());
    assert!(repo.join(".git/refs/heads/retrofit/phase-1-lints").exists());
}
```

---

## Conclusion

The fleet apply system provides a robust, extensible framework for automated retrofit operations across multiple repositories with:

- **Isolation** via git worktrees
- **Reliability** via comprehensive error handling
- **Observability** via detailed reporting
- **Flexibility** via phase-based organization
- **Safety** via automatic cleanup (RAII)
- **Scalability** via async/await foundation

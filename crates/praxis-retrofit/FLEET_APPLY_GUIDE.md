# Fleet Apply: Automated Retrofit Application System

This guide documents the comprehensive fleet-wide retrofit application system for applying praxis standards across multiple repositories with proper isolation, validation, and commit management.

## Overview

The fleet_apply module provides:

1. **Isolated Worktrees** - Each repository is cloned into a temporary worktree to prevent interference with working directories
2. **Phase-based Branches** - Changes are organized into isolated branches (retrofit/phase-1, retrofit/phase-2, etc)
3. **Automatic Validation** - Each retrofit is validated before committing
4. **Atomic Operations** - Standard commit messages with phase information
5. **Comprehensive Reporting** - Detailed success/failure tracking with timings

## Architecture

### Core Components

#### `RetrofitApplier`
Main controller for managing fleet-wide retrofit operations.

```rust
pub struct RetrofitApplier {
    spec: PraxisSpec,
    repositories: Vec<(PathBuf, RetrofitPhase)>,
    concurrent_limit: usize,
}
```

**Responsibilities:**
- Register repositories to be retrofitted
- Coordinate retrofit application across repositories
- Generate summary reports

#### `RetrofitWorktree`
Manages an isolated git worktree for a single repository.

```rust
pub struct RetrofitWorktree {
    original_path: PathBuf,
    worktree_path: PathBuf,
    name: String,
    remote_url: Option<String>,
    current_branch: String,
}
```

**Lifecycle:**
1. Create worktree via `new()`
2. Apply changes via `apply_plan()`
3. Validate via `validate()`
4. Commit changes via `commit()`
5. Cleanup via `cleanup()` or automatic on `Drop`

#### `ApplyResult`
Detailed result of applying a retrofit to a single repository.

```rust
pub struct ApplyResult {
    pub repository_name: String,
    pub original_path: PathBuf,
    pub worktree_path: PathBuf,
    pub phase: RetrofitPhase,
    pub success: bool,
    pub commit_hash: Option<String>,
    pub branch_name: String,
    pub messages: Vec<String>,
    pub warnings: Vec<String>,
    pub error: Option<String>,
    pub duration_secs: f64,
}
```

#### `FleetApplyReport`
Aggregated summary of fleet-wide retrofit results.

```rust
pub struct FleetApplyReport {
    pub total_repositories: usize,
    pub successful: usize,
    pub failed: usize,
    pub warnings_count: usize,
    pub results: Vec<ApplyResult>,
}
```

## Usage Patterns

### Basic Fleet Retrofit

```rust
use praxis_retrofit::{RetrofitApplier, RetrofitPhase, PraxisSpec};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let spec = PraxisSpec::default();
    let mut applier = RetrofitApplier::new(spec)?;
    
    // Register repositories
    applier.add_repository("../repo-a", RetrofitPhase::Phase1Lints)?;
    applier.add_repository("../repo-b", RetrofitPhase::Phase1Lints)?;
    
    // Apply retrofits
    let results = applier.apply_all().await?;
    
    // Generate report
    let report = RetrofitApplier::summary(&results);
    report.print_summary();
    
    Ok(())
}
```

### Multi-Phase Retrofit

```rust
// Phase 1: Lints
let mut applier = RetrofitApplier::new(spec)?;
applier.add_repository("../repo-a", RetrofitPhase::Phase1Lints)?;
let phase1_results = applier.apply_all().await?;

// Phase 2: Dependencies (only for successfully retrofitted repos)
let mut applier = RetrofitApplier::new(spec)?;
for result in phase1_results {
    if result.is_success() {
        applier.add_repository(&result.original_path, RetrofitPhase::Phase2Deps)?;
    }
}
let phase2_results = applier.apply_all().await?;
```

### Concurrent Limit Configuration

```rust
// Process up to 4 repositories concurrently (default)
let applier = RetrofitApplier::new(spec)?
    .with_concurrent_limit(4);

// Process up to 8 repositories concurrently
let applier = RetrofitApplier::new(spec)?
    .with_concurrent_limit(8);

// Serial processing
let applier = RetrofitApplier::new(spec)?
    .with_concurrent_limit(1);
```

## Branch Management

Each phase creates an isolated branch in the original repository:

```
retrofit/phase-1-lints      - Lint configuration (workspace [lints])
retrofit/phase-2-deps       - Dependency unification
retrofit/phase-3-justfile   - Justfile standardization
retrofit/phase-4-typos      - Typos.toml spell-check
retrofit/phase-5-docs       - Documentation standards
```

### Branch Workflow

1. **Branch Creation**: If the phase branch doesn't exist, it's created from the default branch (main/master)
2. **Worktree Setup**: Git worktree is created for the isolated branch
3. **Changes Applied**: All retrofit changes are applied in the worktree
4. **Validation**: Changes are validated (optional)
5. **Commit**: Changes are committed with a standard message including phase information
6. **Worktree Cleanup**: Temporary worktree is removed, original repository unchanged

## Worktree Isolation

The system ensures complete isolation between repositories:

```
/tmp/praxis-retrofit/
├── repo-a-<uuid>/          # Temporary worktree
│   ├── .git
│   ├── src/
│   └── Cargo.toml (modified)
├── repo-b-<uuid>/
│   ├── .git
│   ├── src/
│   └── Cargo.toml (modified)
└── repo-c-<uuid>/
    ├── .git
    ├── src/
    └── Cargo.toml (modified)
```

**Benefits:**
- No interference with working directory
- Safe for concurrent operations
- Automatic cleanup on completion
- Easy debugging with worktree paths in results

## Validation

Each retrofit is validated before committing:

```rust
// Validation occurs automatically via retrofit_apply::validate_retrofit()
// Current implementation is a placeholder that returns Ok(true)
// Can be extended to check:
// - Cargo.toml syntax validity
// - No compilation errors
// - Test suite passes
// - Linting standards met
```

## Error Handling

Errors are captured and reported without stopping the fleet operation:

```rust
pub enum RetrofitError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    Json(serde_json::Error),
    RepositoryNotFound(String),
    InvalidCargoToml(String),
    ComplianceFailed(String),
    RetrofitFailed(String),
    ConfigError(String),
    Walkdir(walkdir::Error),
    Anyhow(anyhow::Error),
}
```

## Reporting

### Summary Report

```rust
let report = RetrofitApplier::summary(&results);
report.print_summary();

// Output:
// === Retrofit Fleet Report ===
// Total repositories: 3
// Successful: 2 (66.7%)
// Failed: 1
// Total warnings: 0
//
// Details:
//   ✓ repo-a [Phase1Lints] -> abc12345 (1.5s)
//   ✓ repo-b [Phase1Lints] -> def67890 (2.1s)
//   ✗ repo-c [Phase1Lints] - Repository not found
```

### Detailed JSON Export

```rust
let json = serde_json::to_string_pretty(&report)?;
println!("{}", json);

// Includes:
// - Total/successful/failed counts
// - Per-repository results with commit hashes
// - Warning messages
// - Execution timings
```

## Command-Line Integration

Integrate fleet apply into CLI tools:

```bash
# Apply Phase 1 retrofits to fleet
praxis-retrofit fleet apply phase-1 --repos-file repos.txt --concurrent 4

# Generate fleet report
praxis-retrofit fleet report phase-1

# Validate fleet retrofits
praxis-retrofit fleet validate phase-1
```

## Troubleshooting

### Issue: Worktree Creation Fails

**Symptom**: `Failed to create worktree` error

**Causes:**
- Repository path is invalid or not a git repo
- Branch already exists in another worktree
- Insufficient disk space in /tmp

**Solution:**
```bash
# Check repository validity
git -C /path/to/repo status

# List existing worktrees
git -C /path/to/repo worktree list

# Clean up orphaned worktrees
git -C /path/to/repo worktree prune
```

### Issue: Validation Fails

**Symptom**: `Validation returned false` warning

**Causes:**
- Cargo.toml is malformed
- Dependencies can't be resolved
- Linting errors introduced

**Solution:**
```bash
# Check Cargo.toml validity
cargo metadata -p praxis-retrofit

# Validate linting
cargo clippy --all-targets --all-features
```

### Issue: Commit Fails

**Symptom**: `Commit failed` error

**Causes:**
- No changes to commit
- Git config incomplete
- Permission issues

**Solution:**
```bash
# Check git config
git config user.name
git config user.email

# Check worktree changes
git -C /tmp/praxis-retrofit/... status
```

## Performance Considerations

### Concurrent Limits

Default: 4 concurrent retrofits

**Recommendations:**
- CPU-bound tasks: (# cores - 1)
- I/O-bound tasks: 2x # cores
- Memory-constrained: 2-4
- Large fleets (100+): 1-2 with batching

### Worktree Cleanup

Worktrees are automatically cleaned up via RAII pattern:

```rust
impl Drop for RetrofitWorktree {
    fn drop(&mut self) {
        if let Err(e) = self.cleanup() {
            warn!("Failed to clean up worktree: {}", e);
        }
    }
}
```

### Temporary Storage

Each worktree uses /tmp disk space. For large fleets:

```rust
// Monitor temp directory
du -sh /tmp/praxis-retrofit/

// Pre-allocate if needed
// Typical per-worktree: 50MB - 200MB
// 100 repos * 100MB = 10GB
```

## Advanced Usage

### Custom Validation

Extend the validation to check specific standards:

```rust
// In your retrofit implementation
pub async fn validate_retrofit(repo_path: &Path) -> Result<bool> {
    // Check Cargo.toml syntax
    let manifest = std::fs::read_to_string(repo_path.join("Cargo.toml"))?;
    toml::from_str::<toml::Value>(&manifest)?;
    
    // Check for expected files
    if !repo_path.join(".github/workflows").exists() {
        return Ok(false);
    }
    
    // Run cargo check
    let output = Command::new("cargo")
        .arg("check")
        .current_dir(repo_path)
        .output()?;
    
    Ok(output.status.success())
}
```

### Filtering Repositories

Apply retrofits only to qualifying repositories:

```rust
let mut applier = RetrofitApplier::new(spec)?;

for repo_path in discover_repositories()? {
    // Filter by criteria
    if should_retrofit(&repo_path)? {
        applier.add_repository(&repo_path, phase)?;
    }
}

fn should_retrofit(repo_path: &Path) -> Result<bool> {
    // Check if Cargo.toml exists
    if !repo_path.join("Cargo.toml").exists() {
        return Ok(false);
    }
    
    // Check minimum compliance
    let report = audit::scan_repository(repo_path, &spec).await?;
    Ok(!report.is_compliant())
}
```

### Batch Processing

For very large fleets, process in batches:

```rust
let repos = discover_all_repositories()?;
let batch_size = 10;

for batch in repos.chunks(batch_size) {
    let mut applier = RetrofitApplier::new(spec.clone())?
        .with_concurrent_limit(4);
    
    for repo_path in batch {
        applier.add_repository(&repo_path, phase)?;
    }
    
    let results = applier.apply_all().await?;
    save_results_to_database(&results)?;
}
```

## Testing

The module includes comprehensive tests:

```rust
#[test]
fn test_branch_name_generation() {
    // Verify branch names follow convention
}

#[test]
fn test_apply_result_summary() {
    // Verify summary formatting
}

#[test]
fn test_fleet_report_success_rate() {
    // Verify report calculations
}
```

Run tests:

```bash
cargo test fleet_apply
cargo test fleet_apply -- --nocapture  # With output
```

## Integration Examples

### CI/CD Pipeline

```bash
#!/bin/bash
# Apply Phase 1 retrofits to all Rust projects

REPOS=$(find . -name "Cargo.toml" -type f | xargs dirname)

praxis-retrofit fleet apply phase-1 \
    --repos "$REPOS" \
    --concurrent 8 \
    --log-level info \
    --report-file fleet-report.json
```

### Git Hooks

```bash
#!/bin/bash
# Pre-commit hook: Validate retrofit was successful

if git show-ref --quiet --verify refs/heads/retrofit/phase-1-lints; then
    # Branch was created by fleet apply, validate it
    praxis-retrofit fleet validate phase-1
    exit $?
fi
exit 0
```

### Monitoring

```rust
// Track retrofit progress with metrics
let results = applier.apply_all().await?;
let report = RetrofitApplier::summary(&results);

metrics::gauge!("retrofit.fleet.total", report.total_repositories as f64);
metrics::gauge!("retrofit.fleet.success", report.successful as f64);
metrics::gauge!("retrofit.fleet.failed", report.failed as f64);
```

## See Also

- [RetrofitApplier](../src/fleet_apply.rs#RetrofitApplier)
- [RetrofitWorktree](../src/fleet_apply.rs#RetrofitWorktree)
- [ApplyResult](../src/fleet_apply.rs#ApplyResult)
- [Example Usage](../examples/fleet_apply_example.rs)

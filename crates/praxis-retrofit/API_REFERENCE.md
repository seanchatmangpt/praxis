# Fleet Apply API Reference

Complete API documentation for the automated retrofit application system.

## Table of Contents

1. [RetrofitApplier](#retrofitapplier)
2. [RetrofitWorktree](#retrofitworktree)
3. [ApplyResult](#applyresult)
4. [FleetApplyReport](#fleetapplyreport)
5. [Error Handling](#error-handling)
6. [Type Definitions](#type-definitions)

---

## RetrofitApplier

Main controller for fleet-wide retrofit operations.

### Definition

```rust
pub struct RetrofitApplier {
    spec: PraxisSpec,
    repositories: Vec<(PathBuf, RetrofitPhase)>,
    concurrent_limit: usize,
}
```

### Methods

#### `new(spec: PraxisSpec) -> Result<Self>`

Creates a new retrofit applier with the given praxis specification.

**Parameters:**
- `spec: PraxisSpec` - Praxis standards specification

**Returns:**
- `Result<Self>` - New RetrofitApplier instance

**Example:**
```rust
let spec = PraxisSpec::default();
let applier = RetrofitApplier::new(spec)?;
```

---

#### `with_concurrent_limit(mut self, limit: usize) -> Self`

Sets the number of repositories to process concurrently.

**Parameters:**
- `limit: usize` - Concurrent limit (minimum 1)

**Returns:**
- `Self` - Updated RetrofitApplier for method chaining

**Default:** 4

**Notes:**
- Values less than 1 are adjusted to 1
- Higher values increase memory usage
- I/O-bound tasks can benefit from higher limits

**Example:**
```rust
let applier = RetrofitApplier::new(spec)?
    .with_concurrent_limit(8);
```

---

#### `add_repository(&mut self, repo_path: impl AsRef<Path>, phase: RetrofitPhase) -> Result<()>`

Registers a repository to be retrofitted.

**Parameters:**
- `repo_path: impl AsRef<Path>` - Path to the repository
- `phase: RetrofitPhase` - Retrofit phase to apply

**Returns:**
- `Result<()>` - Success or error

**Errors:**
- `RepositoryNotFound` - Repository path doesn't exist or isn't a git repo

**Example:**
```rust
applier.add_repository("../my-repo", RetrofitPhase::Phase1Lints)?;
applier.add_repository("../my-other-repo", RetrofitPhase::Phase1Lints)?;
```

---

#### `apply_all(&self) -> impl Future<Output = Result<Vec<ApplyResult>>>`

Applies retrofits to all registered repositories.

**Returns:**
- `Result<Vec<ApplyResult>>` - Results for each repository

**Processing:**
- Repositories are processed sequentially with proper error isolation
- Each repository gets its own worktree
- Changes are automatically validated and committed
- Worktrees are cleaned up after completion

**Example:**
```rust
let results = applier.apply_all().await?;
for result in results {
    if result.is_success() {
        println!("✓ {}", result.repository_name);
    } else {
        println!("✗ {} - {}", result.repository_name, result.error.as_ref().unwrap());
    }
}
```

---

#### `summary(results: &[ApplyResult]) -> FleetApplyReport`

Generates an aggregated summary report from results.

**Parameters:**
- `results: &[ApplyResult]` - Results to summarize

**Returns:**
- `FleetApplyReport` - Summary report with aggregate statistics

**Example:**
```rust
let report = RetrofitApplier::summary(&results);
println!("Success rate: {:.1}%", report.success_rate());
```

---

## RetrofitWorktree

Manages an isolated git worktree for a single repository retrofit.

### Definition

```rust
pub struct RetrofitWorktree {
    original_path: PathBuf,
    worktree_path: PathBuf,
    name: String,
    remote_url: Option<String>,
    current_branch: String,
}
```

### Methods

#### `new(repo_path: &Path, phase: RetrofitPhase) -> Result<Self>`

Creates a new worktree for the given repository and phase.

**Parameters:**
- `repo_path: &Path` - Path to the original repository
- `phase: RetrofitPhase` - Retrofit phase

**Returns:**
- `Result<Self>` - New RetrofitWorktree instance

**Process:**
1. Validates repository exists and is a git repository
2. Generates phase-appropriate branch name
3. Creates temporary directory for worktree
4. Gets remote URL if available
5. Creates git worktree with the branch

**Errors:**
- `RepositoryNotFound` - Repository doesn't exist
- `RetrofitFailed` - Worktree creation failed

**Example:**
```rust
let worktree = RetrofitWorktree::new(
    Path::new("../my-repo"),
    RetrofitPhase::Phase1Lints
)?;
```

---

#### `path(&self) -> &Path`

Gets the temporary worktree path.

**Returns:**
- `&Path` - Absolute path to worktree

**Example:**
```rust
println!("Worktree at: {}", worktree.path().display());
```

---

#### `name(&self) -> &str`

Gets the repository name.

**Returns:**
- `&str` - Repository name (from original path)

**Example:**
```rust
assert_eq!(worktree.name(), "my-repo");
```

---

#### `branch(&self) -> &str`

Gets the current branch name.

**Returns:**
- `&str` - Branch name (e.g., "retrofit/phase-1-lints")

**Example:**
```rust
assert_eq!(worktree.branch(), "retrofit/phase-1-lints");
```

---

#### `apply_plan(&self, plan: &RetrofitPlan) -> impl Future<Output = Result<Vec<String>>>`

Applies a retrofit plan to the worktree.

**Parameters:**
- `plan: &RetrofitPlan` - Retrofit plan to apply

**Returns:**
- `Result<Vec<String>>` - Log messages from application

**Process:**
1. Iterates through all actions in the plan
2. Applies Create, Update, Delete operations
3. Returns detailed messages for each operation

**Example:**
```rust
let plan = generate::generate_retrofit_plan(
    worktree.path(),
    RetrofitPhase::Phase1Lints,
    &spec
)?;

let messages = worktree.apply_plan(&plan).await?;
for msg in messages {
    println!("{}", msg);
}
```

---

#### `validate(&self) -> impl Future<Output = Result<bool>>`

Validates the retrofit in the worktree.

**Returns:**
- `Result<bool>` - Validation passed (true) or not (false)

**Current Implementation:**
- Placeholder that returns `Ok(true)`
- Can be extended to check Cargo.toml syntax, compilation, tests

**Example:**
```rust
if worktree.validate().await? {
    println!("Validation passed");
} else {
    println!("Validation failed");
}
```

---

#### `commit(&self, message: &str) -> Result<String>`

Commits all changes in the worktree.

**Parameters:**
- `message: &str` - Commit message

**Returns:**
- `Result<String>` - Commit hash

**Process:**
1. Stages all changes (`git add -A`)
2. Creates commit with message
3. Returns full commit hash

**Example:**
```rust
let commit_hash = worktree.commit(
    "refactor: Apply praxis lints configuration\n\nAutomated retrofit (phase-1-lints)"
)?;
println!("Committed: {}", &commit_hash[..8]);
```

---

#### `push_to_origin(&self) -> Result<()>`

Pushes changes to the origin repository.

**Returns:**
- `Result<()>` - Success or error

**Note:**
- Push failures are logged as warnings, not errors
- Allows for offline operation
- Requires origin remote to be configured

**Example:**
```rust
worktree.push_to_origin()?;
```

---

#### `cleanup(&self) -> Result<()>`

Cleans up the worktree and temporary directory.

**Returns:**
- `Result<()>` - Success or error

**Process:**
1. Removes git worktree via `git worktree remove`
2. Removes temporary directory
3. Handles partial cleanup gracefully

**Note:**
- Automatically called via Drop trait
- Safe to call manually for explicit cleanup

**Example:**
```rust
worktree.cleanup()?;
```

---

## ApplyResult

Result of applying a retrofit to a single repository.

### Definition

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

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `repository_name` | String | Name of the repository |
| `original_path` | PathBuf | Original repository path |
| `worktree_path` | PathBuf | Temporary worktree path |
| `phase` | RetrofitPhase | Applied retrofit phase |
| `success` | bool | Whether retrofit succeeded |
| `commit_hash` | Option<String> | Commit hash if successful |
| `branch_name` | String | Branch name (e.g., "retrofit/phase-1-lints") |
| `messages` | Vec<String> | Informational messages |
| `warnings` | Vec<String> | Warning messages |
| `error` | Option<String> | Error message if failed |
| `duration_secs` | f64 | Time taken in seconds |

### Methods

#### `is_success(&self) -> bool`

Returns true if the retrofit succeeded.

**Returns:**
- `bool` - Success if `success == true` and `error.is_none()`

**Example:**
```rust
if result.is_success() {
    println!("Retrofit successful");
} else {
    println!("Retrofit failed: {}", result.error.as_ref().unwrap());
}
```

---

#### `summary(&self) -> String`

Generates a human-readable one-line summary.

**Returns:**
- `String` - Summary with emoji, name, commit hash, and duration

**Format:**
- Success: `✓ repo-name [phase] -> commit-hash (time-secs)`
- Failure: `✗ repo-name [phase] - error-message`

**Example:**
```rust
println!("{}", result.summary());
// ✓ my-repo [Phase1Lints] -> abc12345 (1.5s)
```

---

## FleetApplyReport

Aggregated summary of fleet-wide retrofit results.

### Definition

```rust
pub struct FleetApplyReport {
    pub total_repositories: usize,
    pub successful: usize,
    pub failed: usize,
    pub warnings_count: usize,
    pub results: Vec<ApplyResult>,
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `total_repositories` | usize | Total repositories processed |
| `successful` | usize | Successfully retrofitted |
| `failed` | usize | Failed retrofits |
| `warnings_count` | usize | Total warning count |
| `results` | Vec<ApplyResult> | Individual repository results |

### Methods

#### `success_rate(&self) -> f32`

Calculates the success rate as a percentage.

**Returns:**
- `f32` - Success rate (0.0 - 100.0)

**Formula:**
```
(successful / total_repositories) * 100.0
```

**Edge Cases:**
- Empty fleet returns 100.0%

**Example:**
```rust
let rate = report.success_rate();
println!("Success rate: {:.1}%", rate);
```

---

#### `print_summary(&self)`

Prints a formatted summary to stdout.

**Output:**
```
=== Retrofit Fleet Report ===
Total repositories: 3
Successful: 2 (66.7%)
Failed: 1
Total warnings: 0

Details:
  ✓ repo-a [Phase1Lints] -> abc12345 (1.5s)
  ✓ repo-b [Phase1Lints] -> def67890 (2.1s)
  ✗ repo-c [Phase1Lints] - Repository not found
```

**Example:**
```rust
report.print_summary();
```

---

## Error Handling

### RetrofitError

```rust
pub enum RetrofitError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    TomlSer(toml::ser::Error),
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

### Handling Errors

```rust
// Match specific errors
match applier.add_repository(&path, phase) {
    Ok(()) => println!("Added"),
    Err(RetrofitError::RepositoryNotFound(msg)) => {
        eprintln!("Repository not found: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}

// Use Result ? operator
let applier = RetrofitApplier::new(spec)?;
let results = applier.apply_all().await?;
```

---

## Type Definitions

### RetrofitPhase

```rust
pub enum RetrofitPhase {
    Phase1Lints,      // workspace [lints] configuration
    Phase2Deps,       // workspace.dependencies unification
    Phase3Justfile,   // Justfile standardization
    Phase4Typos,      // typos.toml spell-check
    Phase5Docs,       // Documentation standards
}
```

### Branch Name Mapping

| Phase | Branch Name |
|-------|-------------|
| Phase1Lints | `retrofit/phase-1-lints` |
| Phase2Deps | `retrofit/phase-2-deps` |
| Phase3Justfile | `retrofit/phase-3-justfile` |
| Phase4Typos | `retrofit/phase-4-typos` |
| Phase5Docs | `retrofit/phase-5-docs` |

---

## Async/Await

The fleet apply module uses async operations:

```rust
// Methods that return futures
pub async fn apply_plan(&self, plan: &RetrofitPlan) -> Result<Vec<String>>
pub async fn validate(&self) -> Result<bool>
pub async fn apply_all(&self) -> Result<Vec<ApplyResult>>
```

### Runtime Requirements

- Requires Tokio runtime (already included in dependencies)
- Use `#[tokio::main]` on main function
- Can be integrated into existing async applications

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let results = applier.apply_all().await?;
    Ok(())
}
```

---

## Trait Implementations

### Serialize/Deserialize

All types implement `Serialize` and `Deserialize`:

```rust
// Export results to JSON
let json = serde_json::to_string_pretty(&report)?;

// Load results from JSON
let report: FleetApplyReport = serde_json::from_str(&json)?;
```

### Debug

All types derive `Debug` for debugging:

```rust
println!("{:#?}", result);
```

### Clone

All types derive `Clone` for copying:

```rust
let results_copy = results.clone();
```

### Drop

RetrofitWorktree implements Drop for automatic cleanup:

```rust
{
    let worktree = RetrofitWorktree::new(&path, phase)?;
    // Worktree automatically cleaned up when dropped
}
```

---

## Examples

See [fleet_apply_example.rs](examples/fleet_apply_example.rs) for complete working examples.

### Quick Start

```rust
use praxis_retrofit::{RetrofitApplier, RetrofitPhase, PraxisSpec};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let spec = PraxisSpec::default();
    let mut applier = RetrofitApplier::new(spec)?;
    
    applier.add_repository("../my-repo", RetrofitPhase::Phase1Lints)?;
    
    let results = applier.apply_all().await?;
    let report = RetrofitApplier::summary(&results);
    report.print_summary();
    
    Ok(())
}
```

---

## Performance

### Time Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| add_repository | O(1) | Instant |
| apply_all | O(n × m) | n repos × m actions per repo |
| summary | O(n) | Linear in number of results |

### Space Complexity

- Per worktree: 50-200MB (temporary disk)
- Per result: ~1KB (metadata)
- Fleet of 100 repos: ~100MB RAM + 5-20GB disk

---

## Testing

Run tests with:

```bash
cargo test fleet_apply
cargo test fleet_apply -- --nocapture --test-threads=1
```

Available tests:
- `test_branch_name_generation` - Branch naming convention
- `test_apply_result_summary` - Result formatting
- `test_fleet_report_success_rate` - Report calculations

# Parallel Audit Agent Framework for praxis-retrofit

## Design Document

### Executive Summary

This document describes a **production-grade parallel audit agent framework** for praxis-retrofit that enables simultaneous compliance auditing of up to 10 repositories using Tokio task workers and async/await patterns. The framework aggregates individual compliance reports into a fleet-wide compliance matrix, enabling rapid assessment of which repositories need which retrofit phases.

### Problem Statement

Current praxis-retrofit capabilities:
- ✓ Audit individual repositories sequentially
- ✗ No built-in parallelism for fleet-wide audits
- ✗ No aggregation of compliance data across repositories
- ✗ No fleet-wide prioritization or summary reporting

**Use Case:** Retrofit the seanchatmangpt ecosystem (~18 repositories) to meet praxis standards efficiently.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│           Fleet Audit Orchestrator (Main Thread)            │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ FleetAuditCoordinator                                │   │
│  │  - Manages up to 10 parallel audit agents            │   │
│  │  - Enqueues repositories from fleet root             │   │
│  │  - Aggregates compliance reports                     │   │
│  │  - Generates fleet summary                           │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
         │                                                    
         │ spawns (up to 10 in parallel)
         ▼
┌─────────────────────────────────────────────────────────────┐
│           Audit Agent Workers (Tokio Runtime)               │
│                                                               │
│  ┌──────────────────┐  ┌──────────────────┐  ...           │
│  │   Agent #1       │  │   Agent #2       │                │
│  │                  │  │                  │                │
│  │ scan_repository  │  │ scan_repository  │                │
│  │ → ComplianceRpt  │  │ → ComplianceRpt  │                │
│  └──────────────────┘  └──────────────────┘                │
│         │                        │                          │
│         └────────┬───────────────┘                          │
│                  ▼                                          │
│      (Send to aggregator channel)                          │
└─────────────────────────────────────────────────────────────┘
         │
         │ collects all reports
         ▼
┌─────────────────────────────────────────────────────────────┐
│      Compliance Aggregator (Async Collector)                │
│                                                               │
│  ComplianceMatrix:                                          │
│  - repository_reports: HashMap<RepoName, ComplianceReport> │
│  - category_matrix: HashMap<Category, RepoStatuses>        │
│  - phase_requirements: HashMap<RepoName, Vec<Phase>>       │
│  - timestamp: DateTime<Utc>                                │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│           Fleet Summary Report                              │
│                                                               │
│  FleetSummary:                                              │
│  - overall_compliance_score: f32 (fleet average)            │
│  - repos_by_phase: HashMap<Phase, Vec<RepoName>>           │
│  - repos_by_compliance_status: HashMap<Status, Vec<RepoName>>
│  - critical_issues: Vec<ComplianceGap>                      │
│  - execution_metadata: AuditMetadata                        │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

#### 1. **ComplianceMatrix**

Aggregates compliance data from all repositories into a queryable structure.

```rust
pub struct ComplianceMatrix {
    /// All compliance reports indexed by repository name
    pub repository_reports: HashMap<String, ComplianceReport>,
    
    /// Compliance status by category and repository
    /// e.g., category_matrix[CiCd][repo_name] = Pass
    pub category_matrix: BTreeMap<ComplianceCategory, BTreeMap<String, ComplianceStatus>>,
    
    /// Which retrofit phases each repo requires (sorted by priority)
    pub phase_requirements: HashMap<String, Vec<RetrofitPhase>>,
    
    /// Timestamp when audit was completed
    pub timestamp: String,
    
    /// Total scan duration in seconds
    pub scan_duration_seconds: f32,
}
```

**Key Methods:**
- `aggregate(reports: Vec<ComplianceReport>) -> Self` — Builds matrix from individual reports
- `get_repos_by_status(status: ComplianceStatus) -> Vec<String>` — Filter repos by compliance
- `get_repos_needing_phase(phase: RetrofitPhase) -> Vec<String>` — Find repos needing specific phase
- `compliance_score(&self) -> f32` — Calculate fleet-wide compliance score
- `critical_gaps(&self) -> Vec<ComplianceGap>` — Identify high-priority issues

#### 2. **FleetAuditCoordinator**

Orchestrates parallel audit execution with configurable concurrency.

```rust
pub struct FleetAuditCoordinator {
    /// Maximum concurrent audit agents (default 10)
    max_agents: usize,
    
    /// Praxis standards specification
    spec: PraxisSpec,
    
    /// Tracing/observability hooks
    observer: Option<Box<dyn AuditObserver + Send>>,
}

pub struct AuditResult {
    pub report: ComplianceReport,
    pub repo_path: PathBuf,
    pub elapsed: Duration,
}
```

**Key Methods:**
- `new(max_agents: usize, spec: PraxisSpec) -> Self` — Create coordinator
- `audit_fleet(&self, fleet_root: &Path) -> Result<ComplianceMatrix>` — Run parallel audit
- `audit_with_filter(&self, repos: Vec<PathBuf>) -> Result<ComplianceMatrix>` — Audit specific repos
- `set_observer(&mut self, observer: Box<dyn AuditObserver + Send>)` — Add observability

#### 3. **FleetSummary**

Human-readable summary of fleet-wide compliance status.

```rust
pub struct FleetSummary {
    pub overall_compliance_score: f32,
    pub total_repositories: usize,
    pub compliant_repositories: usize,
    pub repos_by_phase: BTreeMap<RetrofitPhase, Vec<String>>,
    pub repos_by_status: BTreeMap<ComplianceStatus, usize>,
    pub category_summary: BTreeMap<ComplianceCategory, CategoryStatus>,
    pub critical_issues: Vec<CriticalIssue>,
    pub audit_metadata: AuditMetadata,
}

pub struct CategoryStatus {
    pub passing: usize,
    pub warning: usize,
    pub failing: usize,
}

pub struct CriticalIssue {
    pub repository: String,
    pub issue: ComplianceItem,
    pub phase: RetrofitPhase,
}

pub struct AuditMetadata {
    pub started_at: String,
    pub completed_at: String,
    pub total_duration_seconds: f32,
    pub agents_used: usize,
    pub avg_repo_scan_time: f32,
}
```

**Key Methods:**
- `from_matrix(matrix: &ComplianceMatrix) -> Self` — Generate from aggregated matrix
- `to_json(&self) -> Result<String>` — Serialize to JSON
- `summary_table(&self) -> String` — Pretty-print for CLI output
- `generate_action_plan(&self) -> Vec<RetrofitAction>` — Recommend next steps

### Implementation Details

#### Parallelism Strategy

**Concurrency Model:**
1. Main coordinator thread spawns up to `max_agents` (default 10) Tokio tasks
2. Each task audits one repository independently
3. Results stream back through async channel
4. Main thread collects results and aggregates into matrix
5. Process completes when all repos audited or max-agents queue exhausted

**Task Spawning:**
```rust
// Pseudo-code pattern
let (tx, mut rx) = tokio::sync::mpsc::channel(max_agents);

for repo in repos {
    let tx = tx.clone();
    let spec = spec.clone();
    tokio::spawn(async move {
        let report = audit::scan_repository(&repo, &spec).await?;
        tx.send(AuditResult { report, repo_path: repo, ... }).await?;
    });
}

drop(tx); // Signal channel EOF

while let Some(result) = rx.recv().await {
    matrix.add_report(result.report);
}
```

**Channel Design:**
- Bounded MPSC channel with capacity = max_agents
- Sender cloned for each spawned task
- Receiver collects in main thread
- Automatic backpressure if audits complete faster than aggregation

#### Repository Discovery

```rust
fn discover_repositories(fleet_root: &Path, filter: Option<&Regex>) -> Result<Vec<PathBuf>> {
    let mut repos = Vec::new();
    
    for entry in std::fs::read_dir(fleet_root)? {
        let path = entry?.path();
        
        // Check: is_git_repo && has_cargo_toml && matches_filter
        if is_git_repo(&path)? && path.join("Cargo.toml").exists() {
            if let Some(f) = filter {
                if f.is_match(path.file_name()?.to_str()?) {
                    repos.push(path);
                }
            } else {
                repos.push(path);
            }
        }
    }
    
    Ok(repos)
}
```

#### Error Handling

**Per-Repository Error Isolation:**
- Individual audit failures don't crash the coordinator
- Failed repos still appear in matrix with error status
- Critical errors logged and surfaced in summary
- Retry mechanism available for transient failures

```rust
pub enum AuditError {
    RepositoryNotFound(String),
    ScanFailed { repo: String, reason: String },
    AggregationFailed(String),
    Timeout { repo: String, elapsed: Duration },
}
```

#### Observability

Opt-in trait for audit progress tracking:

```rust
pub trait AuditObserver: Send {
    fn on_audit_start(&self, repo_count: usize, max_agents: usize);
    fn on_repo_scan_start(&self, repo_name: &str);
    fn on_repo_scan_complete(&self, repo_name: &str, status: &ComplianceReport);
    fn on_repo_scan_error(&self, repo_name: &str, error: &AuditError);
    fn on_fleet_complete(&self, matrix: &ComplianceMatrix);
}
```

### Usage Examples

#### Example 1: Audit Entire Fleet

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let coordinator = FleetAuditCoordinator::new(10, PraxisSpec::default());
    let matrix = coordinator.audit_fleet(Path::new("/path/to/repos"))?;
    
    let summary = FleetSummary::from_matrix(&matrix);
    println!("{}", summary.summary_table());
    
    // Output:
    // Fleet Compliance Summary
    // ═══════════════════════════════════════════════════════
    // Overall Score: 71.4%
    // Compliant: 5/14 repos
    // 
    // Phase Requirements:
    //   Phase 1 (Lints):      12 repos
    //   Phase 2 (Deps):       8 repos
    //   Phase 3 (Justfile):   6 repos
    //   Phase 4 (Typos):      7 repos
    //   Phase 5 (Docs):       4 repos
    // 
    // By Category:
    //   CI/CD:       12/14 Pass,  2 Warn,   0 Fail
    //   Linting:      2/14 Pass,  1 Warn,  11 Fail  ← CRITICAL
    //   Supply Chain: 10/14 Pass,  2 Warn,   2 Fail
    
    Ok(())
}
```

#### Example 2: Audit with Custom Filter & Observer

```rust
struct ProgressObserver;

impl AuditObserver for ProgressObserver {
    fn on_repo_scan_complete(&self, repo_name: &str, report: &ComplianceReport) {
        eprintln!("✓ {} ({:.1}%)", repo_name, report.score);
    }
}

let mut coordinator = FleetAuditCoordinator::new(10, PraxisSpec::default());
coordinator.set_observer(Box::new(ProgressObserver));

let regex = Regex::new("^wasm4|^pm4")?; // Only wasm4* and pm4* repos
let repos = discover_repositories(Path::new("/repos"), Some(&regex))?;
let matrix = coordinator.audit_with_filter(repos)?;

for phase in [Phase1Lints, Phase2Deps, Phase3Justfile] {
    let repos = matrix.get_repos_needing_phase(phase);
    println!("{:?}: {}", phase, repos.join(", "));
}
```

#### Example 3: JSON Export & CI Integration

```rust
let coordinator = FleetAuditCoordinator::new(10, PraxisSpec::default());
let matrix = coordinator.audit_fleet(Path::new("/repos"))?;
let summary = FleetSummary::from_matrix(&matrix);

// Export for CI/CD pipeline
std::fs::write("fleet-compliance.json", summary.to_json()?)?;

// Fail if any category below 70%
for (category, status) in &summary.category_summary {
    let pass_rate = status.passing as f32 / (status.passing + status.warning + status.failing) as f32;
    if pass_rate < 0.7 {
        eprintln!("FAIL: {} below threshold ({}%)", category, (pass_rate * 100.0) as u32);
        std::process::exit(1);
    }
}
```

### Performance Characteristics

**Theoretical Performance (18 repositories, 10 agents):**

| Metric | Value | Notes |
|--------|-------|-------|
| Sequential Time | ~180s | 10 repos × 10s each = 100s per scan |
| Parallel Time (10 agents) | ~18s | 18 repos / 10 agents = 2 batches |
| Speedup | 10x | Perfect parallelism (I/O-bound audits) |
| Memory per agent | ~1-2 MB | Minimal: just file I/O |
| Total memory overhead | ~20 MB | For 10 agents + matrix aggregation |

**Actual Performance Factors:**
- File system I/O speed (SSD >>> HDD)
- Repository size (larger repos → longer audit)
- Network latency (if repos on NAS/remote)
- Tokio thread pool size (default = CPU count)

### Scalability Considerations

**Horizontal Scaling (More Repos):**
- Adjust `max_agents` up to CPU count (diminishing returns beyond)
- Use repository batching if > 100 repos
- Consider persistent cache for compliance results

**Vertical Scaling (Richer Audits):**
- Extend `audit::scan_repository` with more checks
- Add async I/O for concurrent file reads within single audit
- Implement incremental auditing (cache previous results)

**Fault Tolerance:**
- Implement exponential backoff for failed repos
- Log detailed errors with context (file path, check name)
- Provide manual retry mechanism for timeouts
- Store partial results even if audit incomplete

### Testing Strategy

**Unit Tests:**
- `test_compliance_matrix_aggregation` — Verify matrix building
- `test_fleet_summary_generation` — Ensure summary accuracy
- `test_phase_requirement_detection` — Check phase assignment logic

**Integration Tests:**
- `test_parallel_audit_10_repos` — Run on fixture repos
- `test_audit_error_isolation` — Verify one failure doesn't cascade
- `test_observer_callbacks` — Validate observability hooks

**Benchmarks:**
- `bench_serial_vs_parallel_audit` — Compare speedup
- `bench_matrix_aggregation_large` — Test with 100+ repos
- `bench_memory_scaling` — Verify linear memory growth

### Future Enhancements

1. **Caching Layer** — Store previous audit results, invalidate on Cargo.toml change
2. **Incremental Auditing** — Only re-audit changed files
3. **Custom Rules** — Load checks from YAML config instead of hardcoding
4. **Git Integration** — Track compliance across git history
5. **Webhook Support** — Trigger audits on repo push/PR
6. **Web Dashboard** — Real-time compliance visualization
7. **Drift Detection** — Alert when compliant repos regress

### Security Considerations

1. **Audit Isolation** — Each repo scanned in isolated task (no cross-contamination)
2. **Filesystem Permissions** — Audit is read-only (no modifications)
3. **Secret Scanning** — Detect accidental credential commits (future)
4. **Timeout Protection** — Prevent infinite loops in malformed repos
5. **Output Sanitization** — Redact sensitive paths in JSON export

### Dependencies (No New)

All required dependencies already in praxis-retrofit Cargo.toml:
- `tokio` (1.x with macros, rt-multi-thread) ✓
- `serde`/`serde_json` (already used) ✓
- `chrono` (already used) ✓
- `tracing` (already used) ✓
- `thiserror` (already used) ✓

No additional dependencies needed.

### Integration with Existing Crate

**New Module Location:** `src/fleet_audit.rs`

**Export in `lib.rs`:**
```rust
pub mod fleet_audit;

pub use fleet_audit::{
    ComplianceMatrix,
    FleetAuditCoordinator,
    FleetSummary,
    AuditObserver,
};
```

**CLI Integration (Future):**
```bash
praxis-retrofit audit fleet /repos-root [--max-agents 10] [--filter "wasm4|pm4"] [--json]
```

### Conclusion

This framework enables praxis-retrofit to scale from single-repo audits to fleet-wide compliance monitoring with:

- **Production-grade parallelism** via Tokio async/await
- **Transparent aggregation** of compliance reports into queryable matrix
- **Actionable summaries** showing which repos need which phases
- **Extensible observability** for progress tracking and logging
- **Zero new dependencies** (uses existing stack)
- **Fault isolation** preventing cascading failures

The design prioritizes **simplicity**, **performance**, and **production readiness**.


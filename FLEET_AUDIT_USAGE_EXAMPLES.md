# Fleet Audit Framework — Usage Examples

This document provides practical examples for using the `FleetAuditCoordinator`, `ComplianceMatrix`, and `FleetSummary` types from the fleet_audit module.

## Example 1: Basic Fleet Audit

Scan all repositories in a directory and generate a summary:

```rust
use praxis_retrofit::{FleetAuditCoordinator, PraxisSpec};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create coordinator with 10 parallel agents
    let coordinator = FleetAuditCoordinator::new(10, PraxisSpec::default());
    
    // Audit all repos in /repos directory
    let matrix = coordinator
        .audit_fleet(Path::new("/repos"))
        .await?;
    
    // Generate human-readable summary
    let summary = matrix.generate_summary();
    
    // Print summary table to stdout
    println!("{}", summary.summary_table());
    
    Ok(())
}
```

**Output:**
```
╔════════════════════════════════════════════════════════╗
║       Fleet Compliance Summary                        ║
╚════════════════════════════════════════════════════════╝

Overall Score:          71.4%
Compliant Repositories: 5/14

By Compliance Status:
  Pass: 5
  Warn: 6
  Fail: 3

By Category:
  ✗ CiCd               12/14/0 (85.7%)
  ✗ EditorConfig       10/2/2 (71.4%)
  ✗ Documentation      8/4/2 (57.1%)
  ✓ Licensing          14/0/0 (100.0%)
  ✗ Linting            2/1/11 (14.3%)
  ✓ SupplyChain        11/2/1 (78.6%)
  ⚠ Versioning         9/3/2 (64.3%)

Retrofit Phases Needed:
  Phase1Lints: 12 repos
  Phase2Deps: 8 repos
  Phase4Typos: 7 repos
  Phase5Docs: 4 repos

Critical Issues (11):
  ✗ wasm4pm: Workspace Lints
  ✗ pm4py-rs: Workspace Lints
  ✗ pm4wasm: Supply Chain Audit
  ... and 8 more

Audit Metadata:
  Duration: 18.42s (10 agents)
  Avg/repo: 1.84s
```

## Example 2: Audit Specific Repositories Only

Using repository filtering to audit only certain repos:

```rust
use praxis_retrofit::{FleetAuditCoordinator, PraxisSpec};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let coordinator = FleetAuditCoordinator::new(5, PraxisSpec::default());
    
    // Manually specify repos to audit
    let repos = vec![
        Path::new("/repos/wasm4pm").to_path_buf(),
        Path::new("/repos/pm4py-rs").to_path_buf(),
        Path::new("/repos/prolog8").to_path_buf(),
    ];
    
    let matrix = coordinator.audit_with_filter(repos).await?;
    
    println!("Audited {} repositories", matrix.repository_reports.len());
    println!("Fleet score: {:.1}%", matrix.compliance_score());
    
    Ok(())
}
```

## Example 3: Finding Repos That Need Specific Phases

Query the matrix to find which repos need which retrofit phases:

```rust
use praxis_retrofit::{FleetAuditCoordinator, PraxisSpec, RetrofitPhase};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let coordinator = FleetAuditCoordinator::new(10, PraxisSpec::default());
    let matrix = coordinator.audit_fleet(Path::new("/repos")).await?;
    
    // Find all repos needing Phase 1 (Lints)
    let phase1_repos = matrix.get_repos_needing_phase(RetrofitPhase::Phase1Lints);
    println!("Repos needing Phase 1 (Lints): {}", phase1_repos.join(", "));
    
    // Find all repos needing Phase 2 (Dependencies)
    let phase2_repos = matrix.get_repos_needing_phase(RetrofitPhase::Phase2Deps);
    println!("Repos needing Phase 2 (Deps): {}", phase2_repos.join(", "));
    
    // Get all phases required across fleet
    for (repo, phases) in &matrix.phase_requirements {
        if !phases.is_empty() {
            println!("{}: {} phases needed", repo, phases.len());
        }
    }
    
    Ok(())
}
```

**Output:**
```
Repos needing Phase 1 (Lints): wasm4pm, pm4py-rs, pm4wasm, miniml, dteam, prolog8, ocpq, dql, fq, micrograd-rs, rust-anthropic, tokenizers-rs, langchain-rs, neodym

Repos needing Phase 2 (Deps): pm4py-rs, miniml, prolog8, dql, micrograd-rs, rust-anthropic, neodym, langchain-rs

wasm4pm: 4 phases needed
pm4py-rs: 5 phases needed
prolog8: 4 phases needed
```

## Example 4: Category Analysis

Analyze compliance status by category:

```rust
use praxis_retrofit::{FleetAuditCoordinator, PraxisSpec, ComplianceCategory};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let coordinator = FleetAuditCoordinator::new(10, PraxisSpec::default());
    let matrix = coordinator.audit_fleet(Path::new("/repos")).await?;
    
    // Check CI/CD category status
    let (pass, warn, fail) = matrix.count_by_category(ComplianceCategory::CiCd);
    println!("CI/CD: {} pass, {} warn, {} fail", pass, warn, fail);
    
    // Check Linting category
    let (pass, warn, fail) = matrix.count_by_category(ComplianceCategory::Linting);
    let pass_rate = (pass as f32 / (pass + warn + fail) as f32) * 100.0;
    println!("Linting: {:.1}% passing ({}/{})", pass_rate, pass, pass + warn + fail);
    
    // Find worst status across entire fleet
    let worst = matrix.worst_status();
    println!("Worst status seen: {:?}", worst);
    
    Ok(())
}
```

**Output:**
```
CI/CD: 12 pass, 2 warn, 0 fail
Linting: 14.3% passing (2/14)
Worst status seen: Fail
```

## Example 5: Progress Tracking with Observer

Monitor audit progress with custom observer:

```rust
use praxis_retrofit::{FleetAuditCoordinator, PraxisSpec, AuditObserver, ComplianceMatrix, ComplianceReport};
use std::path::Path;
use std::sync::Arc;

struct ProgressObserver;

impl AuditObserver for ProgressObserver {
    fn on_audit_start(&self, repo_count: usize, max_agents: usize) {
        eprintln!("Starting audit: {} repos with {} agents", repo_count, max_agents);
    }
    
    fn on_repo_scan_start(&self, repo_name: &str) {
        eprint!("  {} ... ", repo_name);
    }
    
    fn on_repo_scan_complete(&self, repo_name: &str, report: &ComplianceReport) {
        eprintln!("✓ ({:.1}%)", report.score());
    }
    
    fn on_repo_scan_error(&self, repo_name: &str, error: &str) {
        eprintln!("✗ ({})", error);
    }
    
    fn on_fleet_complete(&self, matrix: &ComplianceMatrix) {
        eprintln!("\nAudit complete: {:.1}% avg compliance", matrix.compliance_score());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut coordinator = FleetAuditCoordinator::new(10, PraxisSpec::default());
    
    // Add observer for progress tracking
    coordinator.set_observer(Arc::new(ProgressObserver));
    
    let matrix = coordinator.audit_fleet(Path::new("/repos")).await?;
    
    Ok(())
}
```

**Output:**
```
Starting audit: 14 repos with 10 agents
  wasm4pm ... ✓ (75.0%)
  pm4py-rs ... ✓ (83.3%)
  pm4wasm ... ✓ (66.7%)
  miniml ... ✓ (50.0%)
  dteam ... ✗ (IO error: permission denied)
  prolog8 ... ✓ (58.3%)
  ocpq ... ✓ (75.0%)
  dql ... ✗ (Repository not found)
  fq ... ✓ (91.7%)
  micrograd-rs ... ✓ (83.3%)
  rust-anthropic ... ✓ (75.0%)
  tokenizers-rs ... ✓ (100.0%)
  langchain-rs ... ✓ (83.3%)
  neodym ... ✓ (66.7%)

Audit complete: 78.6% avg compliance
```

## Example 6: Export Fleet Status as JSON

Serialize audit results for machine consumption (CI/CD pipelines, dashboards):

```rust
use praxis_retrofit::{FleetAuditCoordinator, PraxisSpec};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let coordinator = FleetAuditCoordinator::new(10, PraxisSpec::default());
    let matrix = coordinator.audit_fleet(Path::new("/repos")).await?;
    
    // Export matrix as JSON
    let matrix_json = matrix.to_json()?;
    std::fs::write("fleet-compliance-matrix.json", matrix_json)?;
    
    // Export summary as JSON
    let summary = matrix.generate_summary();
    let summary_json = summary.to_json()?;
    std::fs::write("fleet-compliance-summary.json", summary_json)?;
    
    println!("Exported compliance data:");
    println!("  fleet-compliance-matrix.json");
    println!("  fleet-compliance-summary.json");
    
    Ok(())
}
```

**fleet-compliance-summary.json:**
```json
{
  "overall_compliance_score": 71.4,
  "total_repositories": 14,
  "compliant_repositories": 5,
  "repos_by_phase": {
    "Phase1Lints": ["wasm4pm", "pm4py-rs", "pm4wasm", "miniml", "dteam", "prolog8", "ocpq", "dql", "fq", "micrograd-rs", "rust-anthropic", "tokenizers-rs", "langchain-rs", "neodym"],
    "Phase2Deps": ["pm4py-rs", "miniml", "prolog8", "dql", "micrograd-rs", "rust-anthropic", "neodym", "langchain-rs"],
    "Phase4Typos": ["wasm4pm", "pm4wasm", "miniml", "ocpq", "micrograd-rs", "tokenizers-rs", "langchain-rs"],
    "Phase5Docs": ["dteam", "dql", "fq", "neodym"]
  },
  "repos_by_status": {
    "Pass": 5,
    "Warn": 6,
    "Fail": 3
  },
  "category_summary": {
    "CiCd": { "passing": 12, "warning": 2, "failing": 0 },
    "EditorConfig": { "passing": 10, "warning": 2, "failing": 2 },
    "Documentation": { "passing": 8, "warning": 4, "failing": 2 },
    "Licensing": { "passing": 14, "warning": 0, "failing": 0 },
    "Linting": { "passing": 2, "warning": 1, "failing": 11 },
    "SupplyChain": { "passing": 11, "warning": 2, "failing": 1 },
    "Versioning": { "passing": 9, "warning": 3, "failing": 2 }
  },
  "critical_issues": [
    {
      "repository": "wasm4pm",
      "issue": {
        "name": "Workspace Lints",
        "category": "linting",
        "status": "fail",
        "evidence": "Cargo.toml [lints] block",
        "remediation": "Add [lints] workspace config"
      },
      "phase": "phase-1-lints"
    }
  ],
  "audit_metadata": {
    "started_at": "2026-06-23T17:44:22.123456Z",
    "completed_at": "2026-06-23T17:44:40.567890Z",
    "total_duration_seconds": 18.42,
    "agents_used": 10,
    "avg_repo_scan_time": 1.84
  }
}
```

## Example 7: CI/CD Integration - Compliance Gate

Use fleet audit as a GitHub Actions compliance gate:

```rust
use praxis_retrofit::{FleetAuditCoordinator, PraxisSpec, ComplianceStatus};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let coordinator = FleetAuditCoordinator::new(10, PraxisSpec::default());
    let matrix = coordinator.audit_fleet(Path::new("/repos")).await?;
    let summary = matrix.generate_summary();
    
    // Compliance thresholds for CI gate
    const MIN_FLEET_SCORE: f32 = 75.0;
    const MAX_CRITICAL_ISSUES: usize = 5;
    
    let mut exit_code = 0;
    
    // Check fleet-wide score
    if summary.overall_compliance_score < MIN_FLEET_SCORE {
        eprintln!(
            "FAIL: Fleet compliance score {:.1}% below threshold {:.1}%",
            summary.overall_compliance_score, MIN_FLEET_SCORE
        );
        exit_code = 1;
    }
    
    // Check critical issues
    if summary.critical_issues.len() > MAX_CRITICAL_ISSUES {
        eprintln!(
            "FAIL: {} critical issues exceeds limit of {}",
            summary.critical_issues.len(),
            MAX_CRITICAL_ISSUES
        );
        exit_code = 1;
    }
    
    // Check per-category thresholds
    for (category, status) in &summary.category_summary {
        let total = status.passing + status.warning + status.failing;
        let pass_rate = status.passing as f32 / total as f32;
        
        if pass_rate < 0.7 {
            eprintln!(
                "FAIL: {} passing rate {:.0}% below 70%",
                category,
                pass_rate * 100.0
            );
            exit_code = 1;
        }
    }
    
    if exit_code == 0 {
        println!("✓ Fleet compliance gate PASSED");
    } else {
        println!("✗ Fleet compliance gate FAILED");
    }
    
    std::process::exit(exit_code);
}
```

## Example 8: Phased Retrofit Planning

Use compliance matrix to plan retrofit rollout phases:

```rust
use praxis_retrofit::{FleetAuditCoordinator, PraxisSpec, RetrofitPhase};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let coordinator = FleetAuditCoordinator::new(10, PraxisSpec::default());
    let matrix = coordinator.audit_fleet(Path::new("/repos")).await?;
    
    // Phase 1: Priority (Linting) — affects all compile-time checks
    let phase1_repos = matrix.get_repos_needing_phase(RetrofitPhase::Phase1Lints);
    println!("PHASE 1 (LINTS) — PRIORITY: {} repos", phase1_repos.len());
    for repo in phase1_repos.iter().take(5) {
        println!("  - {}", repo);
    }
    
    // Phase 2: Important (Dependencies) — enables workspace standards
    let phase2_repos = matrix.get_repos_needing_phase(RetrofitPhase::Phase2Deps);
    println!("\nPHASE 2 (DEPS) — IMPORTANT: {} repos", phase2_repos.len());
    for repo in phase2_repos.iter().take(5) {
        println!("  - {}", repo);
    }
    
    // Phase 3-5: Nice-to-have (Justfile, Typos, Docs)
    let phase3_repos = matrix.get_repos_needing_phase(RetrofitPhase::Phase3Justfile);
    let phase4_repos = matrix.get_repos_needing_phase(RetrofitPhase::Phase4Typos);
    let phase5_repos = matrix.get_repos_needing_phase(RetrofitPhase::Phase5Docs);
    
    println!("\nPHASE 3-5 (OPTIONAL):");
    println!("  Phase 3 (Justfile): {} repos", phase3_repos.len());
    println!("  Phase 4 (Typos):    {} repos", phase4_repos.len());
    println!("  Phase 5 (Docs):     {} repos", phase5_repos.len());
    
    // Calculate effort estimate
    const EFFORT_PER_REPO_PHASE1: f32 = 1.0; // hours
    const EFFORT_PER_REPO_OTHER: f32 = 0.5;
    
    let total_effort = 
        phase1_repos.len() as f32 * EFFORT_PER_REPO_PHASE1 +
        (phase2_repos.len() + phase3_repos.len() + phase4_repos.len() + phase5_repos.len()) as f32 * EFFORT_PER_REPO_OTHER;
    
    println!("\nEstimated total effort: {:.1} hours ({:.1} weeks at 40h/week)",
        total_effort, total_effort / 40.0);
    
    Ok(())
}
```

**Output:**
```
PHASE 1 (LINTS) — PRIORITY: 12 repos
  - wasm4pm
  - pm4py-rs
  - pm4wasm
  - miniml
  - dteam

PHASE 2 (DEPS) — IMPORTANT: 8 repos
  - pm4py-rs
  - miniml
  - prolog8
  - dql
  - micrograd-rs

PHASE 3-5 (OPTIONAL):
  Phase 3 (Justfile): 12 repos
  Phase 4 (Typos):    7 repos
  Phase 5 (Docs):     4 repos

Estimated total effort: 22.0 hours (0.55 weeks at 40h/week)
```

## Example 9: Concurrent Scanning with Different Specs

Audit fleet with multiple Praxis specifications simultaneously:

```rust
use praxis_retrofit::{FleetAuditCoordinator, PraxisSpec};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fleet_root = Path::new("/repos");
    
    // Spec 1: Current stable (2021 edition)
    let spec_stable = PraxisSpec::default();
    let coordinator_stable = FleetAuditCoordinator::new(10, spec_stable);
    
    // Spec 2: Strict future (MSRV 1.85, nightly toolchain)
    let mut spec_future = PraxisSpec::default();
    spec_future.msrv = "1.85".to_string();
    spec_future.lints_strict = true;
    let coordinator_future = FleetAuditCoordinator::new(10, spec_future);
    
    // Run both audits in parallel
    let (stable_result, future_result) = tokio::join!(
        coordinator_stable.audit_fleet(fleet_root),
        coordinator_future.audit_fleet(fleet_root),
    );
    
    let stable_matrix = stable_result?;
    let future_matrix = future_result?;
    
    println!("Stable spec score: {:.1}%", stable_matrix.compliance_score());
    println!("Future spec score: {:.1}%", future_matrix.compliance_score());
    
    // Show delta
    let delta = future_matrix.compliance_score() - stable_matrix.compliance_score();
    println!("Delta: {:+.1}%", delta);
    
    Ok(())
}
```

## Performance Notes

- **10 repos on typical SSD:** ~1.8s each = 18s total (10x parallelism vs 180s serial)
- **20 repos with 10 agents:** 2 batches = ~36s total
- **Memory overhead:** ~20 MB for 10 agents + aggregation
- **Max recommended agents:** CPU count (beyond which OS scheduling overhead dominates)

## Thread Safety

All types in fleet_audit are `Send + Sync`:
- `ComplianceMatrix`: Safe for concurrent reads
- `FleetAuditCoordinator`: Creates independent tasks (no shared state)
- `AuditObserver`: Required to be `Send + Sync`

## Error Handling

Per-repository audit failures don't crash the coordinator:

```rust
let matrix = coordinator.audit_fleet(Path::new("/repos")).await?;

// Even if some repos fail, matrix contains successful audits
println!("Successfully audited {}/{} repos",
    matrix.repository_reports.len(),
    // total would require separate discovery pass
);
```

Failed repos are logged but don't block others. Consider implementing retry logic for production use.


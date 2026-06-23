# Fleet Validation Layer for Retrofitted Repositories

## What Is It?

A complete Rust module that validates repositories after praxis retrofit. It:
1. Re-audits compliance after changes
2. Simulates 5 CI gates (fmt, clippy, test, deny, typos)
3. Automatically rolls back on failure
4. Generates before/after compliance reports

## Quick Start

```rust
use praxis_retrofit::{RetrofitValidator, audit::scan_repository, PraxisSpec};

#[tokio::main]
async fn main() -> Result<()> {
    let repo = Path::new(".");
    let spec = PraxisSpec::default();
    
    // 1. Get pre-retrofit baseline
    let pre = scan_repository(repo, &spec).await?;
    
    // 2. Apply retrofit (happens separately)
    
    // 3. Validate post-retrofit
    let validator = RetrofitValidator::new();
    let report = validator.validate_retrofit(repo, &pre).await?;
    
    // 4. Check results
    println!("{}", report.summary());
    println!("Score: {:.1}% → {:.1}% ({:+.1}%)", 
             report.pre_score, report.post_score, report.delta);
    
    match report.status {
        RetrofitValidationStatus::Pass => println!("✓ Validated!"),
        RetrofitValidationStatus::Warn => println!("⚠ Review warnings"),
        RetrofitValidationStatus::Fail => println!("✗ Rolled back"),
    }
    
    Ok(())
}
```

## What Gets Validated?

### Compliance Checks (Pre/Post)
- CI/CD Pipeline
- Supply Chain Audit (deny.toml)
- Workspace Lints
- Editor Config
- Spell Check
- Contributor Guide

### CI Gates (Sequential)
| Gate | Command | Time | Purpose |
|------|---------|------|---------|
| fmt | `cargo fmt --check` | 1-5s | Code formatting |
| clippy | `cargo clippy --all-targets --all-features` | 10-60s | Linting |
| test | `cargo test --all-features` | 10-300s | Unit/integration tests |
| deny | `cargo deny check` | 5-30s | Supply chain security |
| typos | `typos` | 1-5s | Spell checking |

## Core Types

### ValidationReport
```rust
pub struct ValidationReport {
    pub pre_score: f32,              // Baseline compliance %
    pub post_score: f32,             // After-retrofit %
    pub delta: f32,                  // Improvement
    pub ci_results: Vec<CiGateResult>,
    pub status: RetrofitValidationStatus,
    pub rolled_back: bool,
    pub messages: Vec<String>,
    // ... plus raw compliance checks
}

// Methods:
report.summary()           // Human-readable text
report.is_successful()     // Bool: all checks passed?
report.improved()          // Bool: delta > 0?
report.ci_result(gate)     // Individual gate result
```

### RetrofitValidator
```rust
pub struct RetrofitValidator { /* ... */ }

impl RetrofitValidator {
    pub fn new() -> Self                          // Default config
    pub fn with_config(config) -> Self            // Custom config
    pub fn with_spec(spec) -> Self                // Custom spec
    
    pub async fn validate_retrofit(
        &self,
        repo: &Path,
        pre: &ComplianceReport,
    ) -> Result<ValidationReport>
}
```

### ValidationConfig
```rust
pub struct ValidationConfig {
    pub run_tests: bool,          // Default: true
    pub run_clippy: bool,         // Default: true
    pub check_fmt: bool,          // Default: true
    pub check_deny: bool,         // Default: true
    pub check_typos: bool,        // Default: true
    pub auto_rollback: bool,      // Default: true (git reset on failure)
    pub keep_report: bool,        // Default: true (save report after rollback)
    pub max_output_size: usize,   // Default: 16KB
}
```

## Validation Status

| Status | Meaning | Action |
|--------|---------|--------|
| **Pass** | All CI gates passed | Safe to merge |
| **Warn** | Some gates failed, no rollback | Review before merge |
| **Fail** | Rolled back to pre-retrofit | Retrofit aborted |

## Rollback Mechanism

When a CI gate fails AND `auto_rollback: true`:

1. Git state captured before validation (commit SHA, branch)
2. After gate failure, runs: `git reset --hard <initial-sha>`
3. Repository returns to exact pre-retrofit state
4. Validation report preserved for analysis

**Safety:** Uses standard git operations, reversible via reflog.

## Custom Configuration

```rust
let config = ValidationConfig {
    run_tests: false,          // Skip expensive tests
    check_typos: false,        // Skip typos
    auto_rollback: false,      // Manual review first
    max_output_size: 32768,    // Larger output limit
    ..Default::default()
};

let validator = RetrofitValidator::with_config(config);
```

## Examples

### Example 1: Basic Validation
```rust
let pre = scan_repository(repo, &spec).await?;
let validator = RetrofitValidator::new();
let report = validator.validate_retrofit(repo, &pre).await?;

if report.is_successful() {
    println!("✓ Retrofit validated!");
}
```

### Example 2: Check Individual Gates
```rust
if let Some(clippy) = report.ci_result(CiGateName::Clippy) {
    if !clippy.passed {
        println!("Clippy warnings:\n{}", clippy.output);
    }
}
```

### Example 3: Export for Monitoring
```rust
let json = serde_json::to_string_pretty(&report)?;
// Send to monitoring system (Grafana, Datadog, etc.)
```

### Example 4: GitHub Actions Integration
```yaml
- name: Validate retrofit
  run: cargo run --bin praxis-retrofit -- apply validate .

- name: Upload report
  uses: actions/upload-artifact@v3
  with:
    name: validation-report
    path: validation-report.json
```

## Files Included

| File | Purpose |
|------|---------|
| `src/fleet_validate.rs` | Main module (704 lines, 21 KB) |
| `FLEET_VALIDATE_USAGE.md` | Complete API reference & patterns |
| `FLEET_VALIDATE_IMPLEMENTATION.md` | Architecture & technical details |
| `examples/retrofit_validation.rs` | Working example with comments |
| `MODULE_VERIFICATION.md` | Deliverables checklist |
| `FLEET_VALIDATE_README.md` | This file |

## Testing

9 included tests cover:
- Configuration defaults
- Validator creation
- Output truncation
- Report generation
- Compliance improvement detection

Run tests:
```bash
cargo test fleet_validate --lib
```

## Performance

| Operation | Time |
|-----------|------|
| Validate (all gates) | 30-300s (mostly tests) |
| Git state capture | <1s |
| Compliance audit | 1-2s |
| Rollback | <1s |

Output size capped at 16KB per gate to prevent memory exhaustion.

## Key Features

✓ **Pre/post compliance comparison** - Quantify retrofit impact
✓ **CI gate simulation** - 5 comprehensive checks
✓ **Automatic rollback** - Safe restoration on failure
✓ **Configurable gates** - Enable/disable specific checks
✓ **Output truncation** - Prevent memory bloat
✓ **JSON export** - Integration-friendly
✓ **Comprehensive testing** - 9 tests included
✓ **Full documentation** - API guide + examples

## Integration Points

### With audit Module
```rust
pub async fn scan_repository(repo: &Path, spec: &PraxisSpec) 
    -> Result<ComplianceReport>
```
Used for pre and post-retrofit audits.

### With apply Module
```rust
pub async fn apply_retrofit(repo: &Path, plan: &RetrofitPlan)
    -> Result<Vec<String>>
```
Assumed to be called before validation.

### Export to JSON
```rust
let json = serde_json::to_string_pretty(&report)?;
```
Full report serializable for storage/analysis.

## Error Handling

```rust
match validator.validate_retrofit(repo, &pre).await {
    Ok(report) => {
        // Success path
    }
    Err(RetrofitError::RetrofitFailed(msg)) => {
        eprintln!("Validation failed: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Future Enhancements

- Parallel CI gate execution (currently sequential for determinism)
- Caching of expensive gate results
- Custom gate implementations
- Historical report storage
- Trend analysis
- Prometheus metrics export

## Questions?

See:
- `FLEET_VALIDATE_USAGE.md` for complete API
- `FLEET_VALIDATE_IMPLEMENTATION.md` for architecture
- `examples/retrofit_validation.rs` for working code

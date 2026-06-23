# Fleet Validation Layer - Usage Guide

## Overview

The `fleet_validate` module provides comprehensive post-retrofit validation for repositories that have been retrofitted with praxis standards. It implements:

1. **Post-retrofit compliance auditing** - Re-runs compliance checks and compares pre/post scores
2. **CI gate simulation** - Validates against cargo fmt, clippy, test, deny, and typos
3. **Automatic rollback** - Reverts changes on CI failure using git reset --hard
4. **Validation reporting** - Generates before/after compliance comparisons

## Core Components

### RetrofitValidator

Main validator struct that orchestrates the validation workflow.

```rust
use praxis_retrofit::{RetrofitValidator, ValidationConfig};

// Create with default configuration
let validator = RetrofitValidator::new();

// Or with custom config
let config = ValidationConfig {
    run_tests: true,
    run_clippy: true,
    check_fmt: true,
    check_deny: true,
    check_typos: true,
    auto_rollback: true,
    keep_report: true,
    max_output_size: 16384,
};
let validator = RetrofitValidator::with_config(config);
```

### ValidationReport

Complete validation report showing before/after compliance and CI results.

```rust
pub struct ValidationReport {
    pub repository: RepositoryMetadata,
    pub pre_score: f32,           // Pre-retrofit compliance score
    pub post_score: f32,          // Post-retrofit compliance score
    pub delta: f32,               // Score improvement
    pub pre_checks: Vec<ComplianceItem>,
    pub post_checks: Vec<ComplianceItem>,
    pub ci_results: Vec<CiGateResult>,
    pub status: RetrofitValidationStatus,
    pub rolled_back: bool,
    pub timestamp: String,
    pub messages: Vec<String>,
}
```

### RetrofitValidationStatus

Validation outcome status:

- **Pass**: All checks passed, no rollback
- **Warn**: CI had failures but no rollback (auto_rollback disabled)
- **Fail**: CI failed and repository was rolled back

### CiGateResult

Individual CI gate check result:

```rust
pub struct CiGateResult {
    pub gate: CiGateName,      // Which gate (fmt, clippy, test, deny, typos)
    pub passed: bool,
    pub output: String,         // Command output (truncated if large)
    pub error: Option<String>,
    pub duration_ms: u64,
}
```

## Usage Patterns

### Basic Validation Workflow

```rust
use praxis_retrofit::{
    RetrofitValidator, PraxisSpec,
    audit::scan_repository,
};

#[tokio::main]
async fn main() -> Result<()> {
    let repo_path = std::path::Path::new("/path/to/repo");
    let spec = PraxisSpec::default();
    
    // Get pre-retrofit baseline
    let pre_report = scan_repository(repo_path, &spec).await?;
    println!("Pre-retrofit score: {:.1}%", pre_report.score());
    
    // Apply retrofit changes (assumes this has been done)
    // ... retrofit application happens here ...
    
    // Validate post-retrofit
    let validator = RetrofitValidator::new();
    let validation = validator.validate_retrofit(repo_path, &pre_report).await?;
    
    println!("Validation status: {:?}", validation.status);
    println!("Score improvement: {:.1}%", validation.delta);
    
    if validation.is_successful() {
        println!("✓ Retrofit validation passed!");
    } else if validation.rolled_back {
        println!("✗ Rollback performed due to CI failures");
    }
    
    Ok(())
}
```

### Custom CI Gate Configuration

```rust
use praxis_retrofit::{RetrofitValidator, ValidationConfig};

let config = ValidationConfig {
    run_tests: false,           // Skip expensive tests
    run_clippy: true,
    check_fmt: true,
    check_deny: true,
    check_typos: true,
    auto_rollback: false,       // Manual review before rollback
    keep_report: true,
    max_output_size: 32768,     // Larger output for detailed logs
};

let validator = RetrofitValidator::with_config(config);
```

### Handling Validation Results

```rust
let validation = validator.validate_retrofit(repo_path, &pre_report).await?;

// Check overall success
if validation.is_successful() {
    // Proceed with merge
}

// Inspect specific gates
if let Some(fmt_result) = validation.ci_result(CiGateName::Fmt) {
    if !fmt_result.passed {
        eprintln!("Code formatting issues: {}", fmt_result.output);
    }
}

// Check compliance improvement
if validation.improved() {
    println!("Compliance score improved!");
} else if validation.maintained() {
    println!("Compliance score maintained");
}

// View detailed summary
println!("{}", validation.summary());

// Access raw report for integration
let json = serde_json::to_string_pretty(&validation)?;
```

## CI Gates

### cargo fmt --check
Validates code formatting against Rust standards.
- **Pass**: Code is properly formatted
- **Fail**: Code formatting issues detected

### cargo clippy --all-targets --all-features
Runs clippy linter with warnings treated as errors.
- **Pass**: No clippy warnings
- **Fail**: Clippy warnings or errors detected

### cargo test --all-features
Runs the test suite.
- **Pass**: All tests pass
- **Fail**: Test failures detected

### cargo deny check
Validates supply chain security and licensing.
- **Pass**: No supply chain issues
- **Fail**: Vulnerabilities or license issues detected

### typos
Checks for typos in source code and documentation.
- **Pass**: No typos found
- **Fail**: Typos detected

## Git Rollback Mechanism

When `auto_rollback` is enabled and CI validation fails:

1. Initial git state is captured (commit SHA and branch)
2. CI gates are executed
3. If any gate fails, `git reset --hard <captured-sha>` is run
4. Repository is restored to pre-retrofit state
5. Validation report shows `rolled_back: true`

### Manual Rollback Control

```rust
let config = ValidationConfig {
    auto_rollback: false,  // Don't auto-rollback
    keep_report: true,     // Keep validation report
    ..Default::default()
};

let validator = RetrofitValidator::with_config(config);
let report = validator.validate_retrofit(repo_path, &pre_report).await?;

if !report.is_successful() {
    // Review report
    println!("Validation failed. Review: {}", report.summary());
    
    // Manual decision to rollback
    if should_rollback(&report) {
        run_git_reset(&repo_path)?;
    }
}
```

## Compliance Score Calculation

Scores are calculated as:
```
score = (number_of_passing_checks / total_checks) * 100
```

**Pre-score**: Compliance audit before retrofit
**Post-score**: Compliance audit after retrofit
**Delta**: Post-score - Pre-score (positive = improvement)

## Output Truncation

By default, CI gate outputs are limited to 16KB (`max_output_size`). Large outputs are truncated with a note:

```
...
[truncated: 12345 bytes omitted]
```

Increase `max_output_size` in `ValidationConfig` for more verbose output.

## Error Handling

```rust
match validator.validate_retrofit(repo_path, &pre_report).await {
    Ok(report) => {
        // Handle validation results
    }
    Err(RetrofitError::RetrofitFailed(msg)) => {
        eprintln!("Validation execution failed: {}", msg);
    }
    Err(RetrofitError::Io(io_err)) => {
        eprintln!("IO error: {}", io_err);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Example: Full Retrofit + Validation Pipeline

```rust
#[tokio::main]
async fn retrofit_and_validate(repo_path: &Path) -> Result<()> {
    use praxis_retrofit::{
        audit::scan_repository,
        apply::apply_retrofit,
        generate::generate_retrofit_plan,
        RetrofitValidator, PraxisSpec,
    };
    
    let spec = PraxisSpec::default();
    
    // 1. Audit pre-retrofit baseline
    println!("Auditing repository baseline...");
    let pre_report = scan_repository(repo_path, &spec).await?;
    println!("Pre-retrofit score: {:.1}%", pre_report.score());
    
    // 2. Generate and apply retrofit
    println!("Generating retrofit plan...");
    let plan = generate_retrofit_plan(
        repo_path,
        RetrofitPhase::Phase1Lints,
        &spec,
    )?;
    
    println!("Applying retrofit...");
    let _results = apply_retrofit(repo_path, &plan).await?;
    
    // 3. Validate retrofit
    println!("Validating retrofit...");
    let validator = RetrofitValidator::new();
    let validation = validator.validate_retrofit(repo_path, &pre_report).await?;
    
    // 4. Report results
    println!("\n{}", validation.summary());
    
    if validation.is_successful() {
        println!("✓ Retrofit successful and validated!");
        Ok(())
    } else if validation.rolled_back {
        println!("✗ Retrofit rolled back due to CI failures");
        Err(RetrofitError::RetrofitFailed(
            "Validation failed, rollback completed".to_string()
        ))
    } else {
        println!("⚠ Retrofit has warnings - review before merge");
        Ok(())
    }
}
```

## Integration with GitHub Actions

Example GitHub Actions workflow using the validation layer:

```yaml
name: Retrofit Validation

on: [pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run retrofit validation
        run: |
          cargo run --bin praxis-retrofit -- \
            apply validate ${{ github.workspace }}
      
      - name: Upload validation report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: validation-report
          path: validation-report.json
```

## Testing

The module includes comprehensive tests:

```bash
cd crates/praxis-retrofit
cargo test fleet_validate --lib
```

Key test coverage:
- Configuration defaults
- Validator creation
- CI gate name display
- Output truncation
- Validation report summaries
- Compliance improvement detection

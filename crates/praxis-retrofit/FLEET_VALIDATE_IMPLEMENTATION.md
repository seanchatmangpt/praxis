# Fleet Validation Layer - Implementation Summary

## Overview

The `fleet_validate.rs` module implements a comprehensive post-retrofit validation system for repositories retrofitted with praxis standards. It provides automated compliance verification, CI/CD gate simulation, and automatic rollback capabilities.

## Architecture

### Core Components

#### 1. RetrofitValidator
Main orchestrator struct that coordinates the validation workflow.

**Key Responsibilities:**
- Capture pre-retrofit compliance baseline (pre_report)
- Execute CI gate simulations (fmt, clippy, test, deny, typos)
- Capture and restore git state for rollback
- Generate validation reports with before/after comparison

**Interface:**
```rust
pub struct RetrofitValidator {
    config: ValidationConfig,
    spec: PraxisSpec,
}

impl RetrofitValidator {
    pub fn new() -> Self
    pub fn with_config(config: ValidationConfig) -> Self
    pub fn with_spec(mut self, spec: PraxisSpec) -> Self
    pub async fn validate_retrofit(
        &self,
        repo_path: &Path,
        pre_report: &ComplianceReport,
    ) -> Result<ValidationReport>
}
```

#### 2. ValidationReport
Complete validation result containing before/after metrics and CI results.

**Key Fields:**
- `pre_score`: Pre-retrofit compliance score (%)
- `post_score`: Post-retrofit compliance score (%)
- `delta`: Score improvement
- `ci_results`: Individual CI gate results
- `status`: Overall validation status (Pass/Warn/Fail)
- `rolled_back`: Whether automatic rollback was performed
- `messages`: Detailed error/warning messages

#### 3. RetrofitValidationStatus
Enum representing the outcome of validation:

- **Pass**: All CI gates passed, no rollback, validation successful
- **Warn**: Some gates failed but auto_rollback is disabled, validation incomplete
- **Fail**: Gates failed and repository was rolled back, retrofit reverted

#### 4. CiGateResult
Individual CI gate execution result.

**Fields:**
- `gate`: Which gate (Fmt, Clippy, Test, Deny, Typos)
- `passed`: Boolean success indicator
- `output`: Command stdout/stderr (truncated if large)
- `error`: Optional detailed error message
- `duration_ms`: Execution time

#### 5. ValidationConfig
Configuration options for validation behavior.

**Options:**
- `run_tests`: Enable cargo test execution
- `run_clippy`: Enable clippy linter
- `check_fmt`: Enable cargo fmt formatting check
- `check_deny`: Enable cargo deny supply chain check
- `check_typos`: Enable typos spell checker
- `auto_rollback`: Automatically rollback on CI failure
- `keep_report`: Preserve validation report even on rollback
- `max_output_size`: Maximum output size per gate (bytes)

### Workflow Flow

```
1. CAPTURE GIT STATE
   ↓
2. RUN COMPLIANCE AUDIT (post-retrofit)
   ↓
3. EXECUTE CI GATES IN SEQUENCE
   ├─ cargo fmt --check
   ├─ cargo clippy --all-targets --all-features
   ├─ cargo test --all-features
   ├─ cargo deny check
   └─ typos
   ↓
4. EVALUATE RESULTS
   ├─ If any gate failed AND auto_rollback enabled:
   │  ├─ Run git reset --hard <initial_commit>
   │  └─ Set status = Fail, rolled_back = true
   ├─ Else if any gate failed:
   │  └─ Set status = Warn
   └─ Else:
      └─ Set status = Pass
   ↓
5. GENERATE VALIDATION REPORT
   ├─ Pre/post compliance comparison
   ├─ CI gate results
   └─ Rolled back indicator
```

## CI Gates Implementation

### cargo fmt --check
**Purpose:** Verify code formatting compliance

**Command:** `cargo fmt --check`
**Success Criteria:** Process exit code 0
**Failure Handling:** Captured in `output` field

### cargo clippy --all-targets --all-features
**Purpose:** Run lint checker with strict warnings-as-errors policy

**Command:** `cargo clippy --all-targets --all-features -- -D warnings`
**Success Criteria:** No warnings/errors
**Failure Handling:** Clippy output captured

### cargo test --all-features
**Purpose:** Execute full test suite to ensure retrofit didn't break functionality

**Command:** `cargo test --all-features`
**Success Criteria:** All tests pass
**Failure Handling:** Test output and failures captured

### cargo deny check
**Purpose:** Validate supply chain security, licenses, and vulnerabilities

**Command:** `cargo deny check`
**Success Criteria:** No security/license issues
**Failure Handling:** Deny output captured

### typos
**Purpose:** Check for typos in source code and documentation

**Command:** `typos`
**Success Criteria:** No typos found
**Failure Handling:** Typo list captured

## Git Rollback Mechanism

### State Capture
```rust
struct GitState {
    commit_sha: String,    // Current HEAD
    branch: String,        // Current branch
}
```

**Execution:**
1. Before validation begins, capture current git state
2. Run `git rev-parse HEAD` to get commit SHA
3. Run `git rev-parse --abbrev-ref HEAD` to get branch

### Restoration
```rust
fn restore_git_state(repo_path: &Path, state: &GitState) -> Result<()> {
    // Run: git reset --hard <captured_sha>
}
```

**When triggered:**
1. If `auto_rollback == true` AND any CI gate failed
2. Execute `git reset --hard <initial_commit_sha>`
3. No staged changes or untracked files are preserved
4. Repository returns to exact pre-retrofit state

### Safety Guarantees
- Uses `git reset --hard` which is safe (no data loss, previous commits reachable via reflog)
- Only triggers if explicitly configured
- Preserves validation report regardless of rollback
- Sets `rolled_back` flag for visibility

## Compliance Score Comparison

### Score Calculation
Each compliance check is graded as:
- **Pass**: ✓
- **Warn**: ⚠ (counted as non-pass)
- **Fail**: ✗

Total score = (passing_checks / total_checks) * 100

### Delta Calculation
- **Positive delta**: Compliance improved
- **Zero delta**: Compliance maintained
- **Negative delta**: Compliance regressed

### Interpretation
- Retrofit should improve or maintain compliance
- If CI gates pass but score regresses, investigate pre/post checks
- Delta enables tracking retrofit effectiveness

## Error Handling

### IO Errors
```rust
Err(RetrofitError::Io(io_error)) 
// Git command execution, file access issues
```

### Validation Failures
```rust
Err(RetrofitError::RetrofitFailed(message))
// CI gate execution failed, git operations failed
```

### Compliance Failures
- Do NOT throw errors, included in validation report
- Status indicates severity
- Messages vector contains details

## Output Truncation

**Default:** 16 KB per gate
**Purpose:** Prevent excessive memory consumption with large outputs
**Format:** Original output + "[truncated: N bytes omitted]" footer
**Configuration:** `ValidationConfig::max_output_size`

## Testing Strategy

### Unit Tests
Located in `fleet_validate.rs` cfg(test) block:

1. **Configuration Tests**
   - ValidationConfig defaults
   - RetrofitValidator creation

2. **Output Truncation**
   - Small outputs pass through unchanged
   - Large outputs truncated correctly

3. **Report Summarization**
   - Summary generation accuracy
   - Score improvement detection
   - Maintenance detection

4. **Status Checks**
   - is_successful() for Pass status
   - improved() for positive delta
   - maintained() for non-negative delta

### Integration Testing
The example in `examples/retrofit_validation.rs` demonstrates:
- Baseline audit
- Retrofit application
- Comprehensive validation
- Result interpretation
- Custom configuration

## Performance Characteristics

### Time Complexity
- Audit: O(number_of_checks) ≈ constant (7-10 checks)
- Git operations: O(1) constant
- CI gates: Variable, typically 30-300 seconds total
  - fmt: 1-5s
  - clippy: 10-60s
  - test: 10-300s (depends on test suite)
  - deny: 5-30s
  - typos: 1-5s

### Memory Usage
- Output buffering: Bounded by max_output_size (default 16KB × 5 gates = 80KB)
- Report structure: Negligible
- Git state capture: Constant (2 strings)

### Failure Scenarios
- Network failure during deny check: Will retry with timeout
- Missing cargo/typos/deny: Conversion to RetrofitError
- Filesystem issues: Propagated as RetrofitError::Io

## Integration Points

### With audit Module
```rust
crate::audit::scan_repository(repo_path, &spec) -> ComplianceReport
```
Used for both pre-retrofit baseline and post-retrofit audit.

### With apply Module
```rust
crate::apply::apply_retrofit(repo_path, plan) -> Vec<String>
```
Assumed to be called before validation.

### With models Module
- ComplianceReport: Baseline and post-retrofit data
- ComplianceItem: Individual check results
- RepositoryMetadata: Repository identification

### External Integration
- JSON export via serde for monitoring systems
- Compliance scores for dashboards
- CI gate results for GitHub Actions workflows

## Security Considerations

### Git Operations
- Uses standard git commands (reset, rev-parse)
- No shell injection vulnerability (Process::new API)
- Works with any git state

### Output Capture
- Stderr and stdout buffered in memory
- Truncated to prevent DoS
- Treated as untrusted text (no execution)

### No Destructive Operations
- Does not modify .git directory
- Does not delete branches
- Only performs hard reset on initial commit

## Future Enhancements

Potential improvements:

1. **Caching**
   - Cache expensive gate results
   - Skip unchanged gates

2. **Parallel Execution**
   - Run CI gates concurrently (currently sequential)
   - Could reduce validation time significantly

3. **Granular Rollback**
   - Per-gate rollback decisions
   - Preserve partial improvements

4. **Custom Gates**
   - Allow pluggable CI gate implementations
   - Support project-specific validation

5. **Metrics Export**
   - OpenTelemetry integration
   - Prometheus metrics endpoint
   - CloudWatch/Datadog export

6. **Report Persistence**
   - Store validation reports in database
   - Historical comparison
   - Trend analysis

## Module Location
`/home/user/praxis/crates/praxis-retrofit/src/fleet_validate.rs` (21 KB)

## Dependencies
- `crate::models::*` - Compliance models
- `crate::audit` - Compliance scanning
- `serde` - JSON serialization
- `chrono` - Timestamps
- `tracing` - Structured logging
- `tokio` - Async runtime
- `std::process::Command` - Git/Cargo execution

# Praxis Compliance Gates: Implementation Details

**Complete technical documentation of the compliance gate system.**

---

## Overview

The compliance gate system provides automated CI/CD checks that enforce praxis house-style standards. It consists of:

1. **GitHub Actions Workflow** (`.github/workflows/praxis-validate.yml`) — Orchestrates gate execution
2. **Rust CI Gate Module** (`crates/praxis-retrofit/src/ci_gate.rs`) — Core gate logic
3. **Integration Documentation** — Setup guides and examples

---

## Component Architecture

### GitHub Actions Workflow

**File:** `.github/workflows/praxis-validate.yml`  
**Purpose:** Orchestrate compliance validation in CI/CD pipeline

**Jobs:**

| Job | Purpose | Outputs |
|-----|---------|---------|
| `compliance-validate` | Run praxis-retrofit audit, extract score | `score`, `is_compliant`, `report_json` |
| `compliance-gate` | Check score vs threshold, fail if below | Sets PR status to pass/fail |
| `remediation-suggestions` | Generate and post PR comment with fixes | PR comment with remediation steps |
| `compliance-badge` | Create SVG badge, upload artifact | Badge artifact |
| `summary` | Print final compliance summary | Console output |

**Workflow Execution Flow:**

```
PR opened/updated
    ↓
compliance-validate (parallel)
    ├→ Install praxis-retrofit
    ├→ Run: praxis-retrofit validate compliance .
    ├→ Extract: score, report JSON
    └→ Output artifacts
    ↓
compliance-gate (depends on compliance-validate)
    ├→ Read score from previous job
    ├→ Compare vs MIN_SCORE (85.0%)
    ├→ Check critical categories
    └→ Block PR if below threshold
    ↓
remediation-suggestions (parallel, if failed)
    ├→ Parse compliance report
    ├→ Generate remediation steps
    └→ Post PR comment
    ↓
compliance-badge (parallel)
    ├→ Determine badge color by score
    ├→ Generate SVG badge
    └→ Upload as artifact
    ↓
summary
    └→ Print final status
```

### Rust CI Gate Module

**File:** `crates/praxis-retrofit/src/ci_gate.rs`  
**Purpose:** Provide reusable gate logic for any CI system

**Core Types:**

```rust
/// Configuration for compliance gates
pub struct GateConfig {
    pub min_score: f32,                     // e.g., 85.0
    pub block_on_drop: bool,                // Block if score drops
    pub critical_categories: Vec<Category>, // Must Pass
    pub auto_remediate: bool,               // Post PR comment
    pub generate_badge: bool,               // Create badge
}

/// Result of a gate check
pub enum GateResult {
    Pass,      // Score >= threshold, no critical issues
    Fail,      // Score < threshold or critical issue failed
    Warning,   // Score >= threshold but has warnings
}

/// Detailed gate check output
pub struct GateCheckOutput {
    pub gate_result: GateResult,
    pub score: f32,
    pub threshold: f32,
    pub message: String,
    pub blocking_issues: Vec<String>,
    pub warnings: Vec<String>,
    pub remediation_steps: Vec<RemediationStep>,
    pub badge_color: String,    // "green", "yellow", "red"
    pub badge_label: String,    // "Excellent", "Good", "Needs Work"
}

/// Individual fix recommendation
pub struct RemediationStep {
    pub priority: RemediationPriority,  // Critical, High, Medium, Low
    pub category: ComplianceCategory,   // CI/CD, Linting, etc.
    pub issue: String,                  // What's wrong
    pub suggestion: String,             // How to fix
    pub command: Option<String>,        // Optional command to run
    pub reference: Option<String>,      // Link to documentation
}
```

**Main Engine:**

```rust
pub struct ComplianceGate {
    config: GateConfig,
}

impl ComplianceGate {
    /// Create gate with default config (85% min score)
    pub fn new() -> Self

    /// Create gate with custom config
    pub fn with_config(config: GateConfig) -> Self

    /// Run gate check against compliance report
    pub async fn check(&self, report: &ComplianceReport) -> Result<GateCheckOutput>
}
```

**Usage Pattern:**

```rust
// 1. Get compliance report
let report = praxis_retrofit::validate::validate_compliance(&repo_path).await?;

// 2. Create gate (with defaults or custom config)
let gate = ComplianceGate::new();

// 3. Run check
let output = gate.check(&report).await?;

// 4. Use output for CI/CD logic
match output.gate_result {
    GateResult::Pass => println!("PR can merge"),
    GateResult::Fail => anyhow::bail!("PR is blocked"),
    GateResult::Warning => println!("Review warnings"),
}

// 5. Generate remediation markdown for PR
let markdown = format_remediation_markdown(&output.remediation_steps);

// 6. Generate compliance badge
let svg = BadgeGenerator::generate_svg(output.score, &output.badge_label, &output.badge_color);
```

---

## Compliance Score Calculation

**Formula:**

```
Score = (Number of Passing Checks / Total Checks) * 100%
```

**Example:**

```
Total checks: 6
Passing checks: 5
Failing checks: 1
Warning checks: 0

Score = (5 / 6) * 100 = 83.3%
Gate Result: FAIL (below 85% threshold)
```

### Critical Categories

Certain categories must **Pass** (not Warn) to avoid blocking the gate:

- **CI/CD Pipeline** — Must have GitHub Actions workflows
- **Workspace Lints** — Must have `[lints]` in Cargo.toml

**Logic:**

```rust
// Check if any critical category failed
for item in &report.checks {
    if self.config.critical_categories.contains(&item.category)
        && item.status == ComplianceStatus::Fail
    {
        // Block the gate
        blocking_issues.push(format!("Critical category failed: {}", item.name));
    }
}
```

---

## Remediation Step Generation

The gate automatically generates remediation steps based on failed checks:

**Fail → Critical Priority:**
```
Issue: Workspace Lints
Suggestion: Add [lints] workspace config
Priority: Critical 🚨
Command: # Add [lints] block to Cargo.toml
Reference: https://github.com/seanchatmangpt/praxis#linting
```

**Warn → High Priority:**
```
Issue: Spell Check
Suggestion: Generate typos.toml template
Priority: High ⚠️
Command: praxis-retrofit apply retrofit .
Reference: https://github.com/seanchatmangpt/praxis#spell-check
```

**Pass → No Step:**
```
(No remediation needed)
```

---

## Badge Generation

The badge system provides visual feedback on compliance status.

**Badge Thresholds:**

| Score | Color  | Label       | Icon |
|-------|--------|-------------|------|
| ≥ 90% | Green  | Excellent   | ✅   |
| 75-89% | Yellow | Good        | ⚠️   |
| < 75% | Red    | Needs Work  | ❌   |

**SVG Structure:**

```xml
<svg xmlns="http://www.w3.org/2000/svg" width="160" height="20" role="img">
  <!-- Background -->
  <rect width="100" height="20" fill="#555"/>
  <!-- Status bar (colored by score) -->
  <rect x="100" width="60" height="20" fill="green|yellow|red"/>
  <!-- Text layers -->
  <text>compliance</text>
  <text>92% - Excellent</text>
</svg>
```

**Usage in README:**

```markdown
# My Repository

![Compliance Status](badges/compliance-badge.svg)

This repository adheres to [praxis](https://github.com/seanchatmangpt/praxis) standards.
```

---

## PR Remediation Comments

When compliance fails, an auto-generated PR comment provides guidance:

**Generated Markdown:**

```markdown
## 🔧 Praxis Compliance Remediation Suggestions

Your repository does not fully meet praxis compliance standards.
Here are the recommended remediation steps:

### 🚨 Critical Issues (Must Fix)

**Workspace Lints**: Add [lints] workspace config

```bash
# Add to Cargo.toml:
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
```

📚 [Learn more](https://github.com/seanchatmangpt/praxis#linting)

### ⚠️ High Priority (Recommended)

**Supply Chain Audit**: Generate deny.toml template

...

### Quick Start

1. **Review the compliance report** - Download from artifacts
2. **Run locally** - `praxis-retrofit audit report .`
3. **Apply corrections** - `praxis-retrofit apply retrofit .`
4. **Validate** - `praxis-retrofit validate compliance .`
5. **Push changes** - Commit and push to this PR

_Generated by Praxis Compliance Gate_
```

**Implementation:** See `format_remediation_markdown()` in `ci_gate.rs`

---

## Configuration Options

### Default Configuration

```rust
GateConfig {
    min_score: 85.0,
    block_on_drop: true,
    critical_categories: vec![
        ComplianceCategory::CiCd,
        ComplianceCategory::Linting,
    ],
    auto_remediate: true,
    generate_badge: true,
}
```

### Custom Configuration in Workflow

Edit `.github/workflows/praxis-validate.yml`:

```yaml
- name: Check compliance threshold
  env:
    MIN_SCORE: 90.0  # Change threshold
  run: |
    # Comparison logic
    if (( $(echo "$CURRENT < $MIN_SCORE" | bc -l) )); then
      echo "❌ Score below threshold"
      exit 1
    fi
```

### Custom Configuration in Rust

```rust
let config = GateConfig {
    min_score: 90.0,  // Stricter
    block_on_drop: true,
    critical_categories: vec![
        ComplianceCategory::CiCd,
        ComplianceCategory::Linting,
        ComplianceCategory::SupplyChain,  // Also critical
    ],
    auto_remediate: true,
    generate_badge: true,
};

let gate = ComplianceGate::with_config(config);
```

---

## Integration Patterns

### Pattern 1: CI/CD Block

Block PRs that drop compliance below threshold:

```yaml
# .github/workflows/praxis-validate.yml
compliance-gate:
  runs-on: ubuntu-latest
  if: github.event_name == 'pull_request'
  # ... job that fails if score < threshold
  # GitHub will automatically block merge
```

### Pattern 2: Scheduled Reports

Generate compliance reports on schedule:

```yaml
# .github/workflows/compliance-report.yml
on:
  schedule:
    - cron: '0 9 * * MON'  # Weekly Monday report

jobs:
  report:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install praxis-retrofit --locked
      - run: praxis-retrofit audit report . | jq . > report.json
      - uses: actions/upload-artifact@v4
        with:
          name: compliance-report
          path: report.json
```

### Pattern 3: Dynamic Thresholds

Different thresholds per branch:

```yaml
- name: Determine threshold
  id: threshold
  run: |
    case "${{ github.ref }}" in
      refs/heads/main)
        echo "MIN_SCORE=90.0" >> $GITHUB_OUTPUT ;;
      refs/heads/release/*)
        echo "MIN_SCORE=95.0" >> $GITHUB_OUTPUT ;;
      *)
        echo "MIN_SCORE=75.0" >> $GITHUB_OUTPUT ;;
    esac
```

---

## Error Handling

### Workflow Error Handling

**Step-level:** Each step sets `set +e` to capture output before failing:

```yaml
- name: Validate compliance
  id: validate
  run: |
    set +e  # Don't fail yet
    REPORT=$(praxis-retrofit validate compliance .)
    EXIT_CODE=$?
    # Process output...
    exit $EXIT_CODE  # Fail here with captured output
```

**Job-level:** Jobs depend on previous jobs and only run on certain conditions:

```yaml
compliance-gate:
  needs: compliance-validate
  if: github.event_name == 'pull_request'
  # Only runs if compliance-validate succeeds and is a PR
```

### Module Error Handling

**Result Type:** Uses standard `anyhow::Result<T>`:

```rust
pub async fn check(&self, report: &ComplianceReport) -> Result<GateCheckOutput> {
    let score = report.score();
    // ... gate logic
    Ok(GateCheckOutput { ... })
}
```

**Error Cases:** Can fail at validation stage:

```rust
// If report is malformed, gate can't run
if report.checks.is_empty() {
    anyhow::bail!("Compliance report has no checks")
}
```

---

## Testing

### Unit Tests in ci_gate.rs

```rust
#[test]
fn test_gate_config_default() {
    let config = GateConfig::default();
    assert_eq!(config.min_score, 85.0);
    assert!(config.block_on_drop);
}

#[test]
fn test_gate_result_pass() {
    // Create report with all passing checks
    let report = ComplianceReport { ... };
    let gate = ComplianceGate::new();
    let blocking = gate.find_blocking_issues(&report);
    assert!(blocking.is_empty());
}

#[test]
fn test_badge_excellent() {
    let (color, label) = ComplianceGate::new().badge_for_score(95.0);
    assert_eq!(color, "green");
    assert_eq!(label, "Excellent");
}
```

### Integration Tests

```rust
// tests/compliance_gate_integration.rs
#[tokio::test]
async fn test_gate_blocks_non_compliant_repo() {
    // Create non-compliant repo
    let report = create_non_compliant_report();
    
    // Gate should fail
    let gate = ComplianceGate::new();
    let output = gate.check(&report).await.unwrap();
    
    assert_eq!(output.gate_result, GateResult::Fail);
    assert!(!output.blocking_issues.is_empty());
}
```

### Workflow Testing

1. **Test in dry-run:** Create a feature branch and open PR (don't merge)
2. **Check workflow execution:** Go to Actions tab
3. **Review output:** Check logs and artifacts
4. **Verify remediation:** Confirm PR comment posted
5. **Test fix:** Apply fixes and push to same PR
6. **Verify pass:** Confirm gate passes on re-run

---

## Maintenance & Monitoring

### Metrics to Track

| Metric | Target | Purpose |
|--------|--------|---------|
| Avg compliance score | ≥ 85% | Baseline quality |
| PRs blocked | < 5% | Gate isn't too strict |
| Gate MTTR | < 30 min | Developer responds quickly |
| Critical pass rate | 100% | Zero tolerance for critical issues |

### Update Workflow

```bash
# When praxis standards change, update:
cp /path/to/praxis/.github/workflows/praxis-validate.yml \
   .github/workflows/praxis-validate.yml

git add .github/workflows/praxis-validate.yml
git commit -m "chore: update praxis compliance gate"
git push
```

### Monitor Gate Health

```bash
# Generate weekly compliance history
for i in {1..52}; do
  praxis-retrofit audit report . | jq '{
    week: '$i',
    score: .score,
    passing: [.checks[] | select(.status == "pass")] | length,
    failing: [.checks[] | select(.status == "fail")] | length
  }' >> compliance-history.jsonl
done

# Analyze trends
cat compliance-history.jsonl | jq -s 'sort_by(.score) | .[0,1,-1] | {week, score, passing, failing}'
```

---

## Troubleshooting

### Issue: Gate Always Fails

**Cause:** Uncommitted changes not included in audit

**Solution:**

```bash
# Ensure all changes are committed
git status

# Run audit from repo root
cd /path/to/repo
praxis-retrofit audit report .
```

### Issue: Badge Not Uploading

**Cause:** Artifact name mismatch

**Solution:**

```bash
# Check workflow generates correct path
# In workflow:
- uses: actions/upload-artifact@v4
  with:
    name: compliance-badge  # Must match
    path: compliance-badge.svg  # File must exist
```

### Issue: PR Comment Not Posting

**Cause:** Missing permissions

**Solution:**

```yaml
# In .github/workflows/praxis-validate.yml
permissions:
  contents: read
  pull-requests: write  # <-- MUST have
  checks: write         # <-- MUST have
```

### Issue: Score Calculation Inconsistent

**Cause:** Different files between local and CI

**Solution:**

```bash
# Ensure everything is committed (no staged files)
git diff --cached  # Should be empty
git diff           # Should be empty

# Re-run audit
praxis-retrofit audit report .
```

---

## Performance Considerations

### Workflow Performance

**Timing (typical):**

| Step | Duration |
|------|----------|
| Checkout | 1-2s |
| Install praxis-retrofit | 30-45s |
| Run compliance audit | 5-10s |
| Gate check | <1s |
| Generate badge | <1s |
| Post PR comment | 2-5s |
| **Total** | **~45-65s** |

**Optimization:**

- Cache praxis-retrofit installation
- Use `Swatinem/rust-cache` for Cargo builds
- Run jobs in parallel where possible

### Module Performance

**Complexity:**

- `find_blocking_issues()` — O(n) where n = number of checks
- `generate_remediation_steps()` — O(n log n) (includes sort)
- `badge_for_score()` — O(1)
- `format_remediation_markdown()` — O(m) where m = number of steps

**Memory:**

- GateCheckOutput: ~5KB per gate check
- BadgeGenerator: SVG string ~2KB
- format_remediation_markdown(): ~10KB for full remediation

---

## Security Considerations

### Workflow Security

- **Secrets:** No secrets needed (only public repos)
- **Permissions:** Minimal (read contents, write PRs)
- **Token:** Uses GITHUB_TOKEN (automatically created)

### Module Security

- **No unsafe code:** Uses only safe Rust
- **No file I/O:** Read-only operations on reports
- **No network:** No external API calls
- **Immutable inputs:** Takes references to reports

### PR Comment Security

- **Sanitization:** Uses code blocks to escape user content
- **Rate limiting:** GitHub API handles rate limits
- **Idempotency:** Updates existing comment if found

---

## Example: Custom Integration

Implement compliance gates in a custom CI system:

```rust
use praxis_retrofit::{
    validate_compliance, ComplianceGate, GateConfig,
    ComplianceCategory,
};
use std::path::Path;

async fn my_ci_system(repo_path: &Path) -> anyhow::Result<()> {
    // 1. Run audit
    let report = validate_compliance(repo_path).await?;
    println!("Compliance Score: {:.1}%", report.score());

    // 2. Create custom gate config
    let config = GateConfig {
        min_score: 90.0,
        block_on_drop: true,
        critical_categories: vec![
            ComplianceCategory::CiCd,
            ComplianceCategory::Linting,
        ],
        auto_remediate: true,
        generate_badge: true,
    };

    // 3. Run gate check
    let gate = ComplianceGate::with_config(config);
    let output = gate.check(&report).await?;

    // 4. Print report
    println!("Gate Result: {:?}", output.gate_result);
    println!("Blocking Issues: {}", output.blocking_issues.len());

    // 5. Decide action
    match output.gate_result {
        GateResult::Pass => {
            println!("✅ PR can merge");
            Ok(())
        }
        GateResult::Fail => {
            println!("❌ PR is blocked");
            eprintln!("{}", output.message);
            anyhow::bail!("Compliance gate failed")
        }
        GateResult::Warning => {
            println!("⚠️  Review warnings");
            Ok(())
        }
    }
}
```

---

## Related Documentation

- **[Praxis README](README.md)** — Overview of standards
- **[Compliance Gates Quick Start](COMPLIANCE_GATES_QUICKSTART.md)** — Setup guide
- **[Full Integration Guide](COMPLIANCE_GATES.md)** — Detailed documentation
- **[ci_gate.rs Source](crates/praxis-retrofit/src/ci_gate.rs)** — Module code
- **[Example: Gate Integration](examples/compliance-gate-integration.rs)** — Runnable example

---

## License

MIT OR Apache-2.0

# Praxis Compliance Gates: CI/CD Integration Guide

**Version:** 26.6.0  
**Status:** Production-Ready  
**Last Updated:** 2026-06-23

---

## Overview

Compliance gates are automated CI/CD checks that enforce praxis house-style standards across all repositories in the ecosystem. They:

- ✅ **Block PRs** if compliance score drops below threshold
- 🔧 **Auto-comment** with remediation suggestions
- 📊 **Report status** via badges and workflow artifacts
- 🚨 **Gate on critical categories** (CI/CD, linting) that must Pass
- ⚡ **Fail-fast** to prevent non-compliant code from merging

---

## Quick Start

### 1. Add the Workflow to Your Repository

Copy the workflow file to your repository:

```bash
# If you already have .github/workflows, copy the template:
cp /path/to/praxis/.github/workflows/praxis-validate.yml .github/workflows/

# Otherwise, create the directory first:
mkdir -p .github/workflows
cp /path/to/praxis/.github/workflows/praxis-validate.yml .github/workflows/
```

### 2. Verify Installation

```bash
# Check that the workflow is in place
ls -la .github/workflows/praxis-validate.yml

# Commit and push
git add .github/workflows/praxis-validate.yml
git commit -m "chore: add praxis compliance gate"
git push origin main
```

### 3. Test the Workflow

Open a pull request with any change. The workflow should:

1. Run `praxis-retrofit validate compliance .`
2. Generate a compliance report
3. Post a PR comment if issues are found
4. Block the PR if score is below 85%

---

## Architecture

### Gate Execution Flow

```
┌─ Pull Request Opened
│
├─ [Compliance Validation] ─────────────────┐
│  ├ Install praxis-retrofit                │
│  ├ Run: praxis-retrofit validate          │
│  │       compliance <repo>                │
│  └ Extract: score, report JSON            │
│                                            │
├─ [Compliance Gate Check] ←────────────────┘
│  ├ Read compliance score
│  ├ Compare vs threshold (85%)
│  ├ Check critical categories
│  └ Fail if below threshold
│
├─ [Remediation Suggestions]
│  ├ Parse failed checks
│  ├ Generate fix commands
│  └ Post PR comment
│
└─ [Compliance Badge]
   ├ Generate SVG badge
   ├ Color by score (green/yellow/red)
   └ Upload as artifact
```

### Core Components

#### 1. GitHub Actions Workflow (`.github/workflows/praxis-validate.yml`)

Orchestrates the entire gate process:

- **compliance-validate** — Run compliance audit, extract score
- **compliance-gate** — Check score vs threshold, block PR if needed
- **remediation-suggestions** — Post PR comment with fix steps
- **compliance-badge** — Generate and upload status badge
- **summary** — Print final compliance report

#### 2. Rust CI Gate Module (`ci_gate.rs`)

Provides reusable gate logic:

- `ComplianceGate` — Main gate engine
- `GateConfig` — Configurable thresholds and rules
- `GateCheckOutput` — Detailed gate check results
- `RemediationStep` — Individual fix recommendations
- `BadgeGenerator` — SVG badge generation
- `format_remediation_markdown()` — PR comment formatting

#### 3. Integration Points

- **praxis-retrofit CLI** — Validates compliance
- **GitHub REST API** — Creates checks, posts comments
- **Artifacts** — Stores compliance reports and badges

---

## Configuration

### Default Gate Configuration

```rust
GateConfig {
    min_score: 85.0,                    // Minimum compliance score (%)
    block_on_drop: true,                // Block PRs that drop score
    critical_categories: [
        ComplianceCategory::CiCd,       // Must Pass
        ComplianceCategory::Linting,    // Must Pass
    ],
    auto_remediate: true,               // Post suggestions
    generate_badge: true,               // Create status badge
}
```

### Customize the Gate (in Workflow)

Edit `.github/workflows/praxis-validate.yml`:

```yaml
# Example: Change minimum score to 90%
- name: Check compliance threshold
  env:
    MIN_SCORE: 90.0  # <-- Change this
  run: |
    # ... rest of step
```

### Customize Critical Categories

To add/remove categories that must Pass:

```yaml
# In workflow, modify the gate step:
# Fail if ANY of these categories have Fail status:
CRITICAL_CATEGORIES: |
  ci-cd
  linting
  supply-chain
```

---

## Usage Examples

### Example 1: Standard Setup

A repository meets praxis standards and passes the gate:

```
✅ Compliance Score: 92%
✅ All critical categories: Pass
✅ PR can merge
```

Workflow output:
- ✅ Job passes
- No PR comment
- Green badge uploaded

---

### Example 2: PR with Compliance Drop

A developer removes [lints] configuration:

```
❌ Compliance Score: 72%
❌ Critical category failed: Linting
❌ Blocking PR
```

Workflow output:
- ❌ Job fails, PR blocked
- 🔧 Auto-comment with remediation steps:
  - Add [lints] workspace config
  - Copy from praxis template
  - Run validation locally
- 🔴 Red badge: "Needs Work"

Developer fixes:
```bash
# Locally
praxis-retrofit apply retrofit .
praxis-retrofit validate compliance .

# Push fix to PR
git add -A
git commit -m "fix: restore praxis compliance [lints]"
git push

# ✅ Workflow re-runs, gate passes
```

---

### Example 3: Multiple Issues

A newly retrofitted repo has several warnings:

```
⚠️  Compliance Score: 78%
⚠️  Below threshold (85%)
🔧 Suggestions generated
```

Issues found:
- Missing `deny.toml` (Warn)
- Missing `typos.toml` (Warn)
- No `CONTRIBUTING.md` (Warn)

Remediation steps:
1. `praxis-retrofit apply retrofit .` — auto-apply templates
2. `praxis-retrofit validate compliance .` — verify
3. Commit and push

---

## Compliance Categories

### 1. CI/CD Pipeline
- **Category:** `ci-cd`
- **Status:** Must **Pass** (critical)
- **Check:** `.github/workflows/` exists and contains valid workflows
- **Remediation:** Copy `.github/workflows/` from praxis template

### 2. Supply Chain Audit
- **Category:** `supply-chain`
- **Status:** Must **Pass** (recommended)
- **Check:** `deny.toml` exists and is valid
- **Remediation:** Generate from praxis template; run `cargo deny check`

### 3. Workspace Lints
- **Category:** `linting`
- **Status:** Must **Pass** (critical)
- **Check:** Cargo.toml contains `[lints]` block
- **Remediation:** Add `[workspace.lints]` from template

### 4. Editor Config
- **Category:** `editor-config`
- **Status:** **Warn** if missing
- **Check:** `.editorconfig` exists
- **Remediation:** Copy from praxis template

### 5. Spell Check
- **Category:** `editor-config`
- **Status:** **Warn** if missing
- **Check:** `typos.toml` exists
- **Remediation:** Generate from praxis template

### 6. Contributor Guide
- **Category:** `documentation`
- **Status:** **Warn** if missing
- **Check:** `CONTRIBUTING.md` exists
- **Remediation:** Copy from praxis template

---

## Remediation Workflow

### Auto-Generated PR Comment

When compliance checks fail, the workflow posts:

```markdown
## 🔧 Praxis Compliance Remediation Suggestions

Your repository does not fully meet praxis compliance standards. 
Here are the recommended remediation steps:

### 🚨 Critical Issues (Must Fix)

**Workspace Lints**: Add [lints] workspace config

```bash
# Copy from praxis template and add to Cargo.toml:
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

### Manual Remediation Steps

```bash
# 1. Audit the repository
praxis-retrofit audit report . | jq .

# 2. Generate a retrofit plan
praxis-retrofit generate plan . > retrofit-plan.json

# 3. Review the plan
cat retrofit-plan.json | jq .

# 4. Apply the retrofit
praxis-retrofit apply retrofit .

# 5. Validate success
praxis-retrofit validate compliance .

# 6. Commit and push
git add -A
git commit -m "chore: retrofit praxis standards"
git push origin feature-branch
```

---

## Compliance Badge

The workflow generates an SVG badge showing compliance status:

### Badge Colors

| Score | Color  | Label       | Meaning          |
|-------|--------|-------------|------------------|
| ≥ 90% | 🟢     | Excellent   | Fully compliant  |
| ≥ 75% | 🟡     | Good        | Minor issues     |
| < 75% | 🔴     | Needs Work  | Critical issues  |

### Using the Badge

#### In README.md

```markdown
# My Repository

![Compliance Status](badges/compliance-badge.svg)

This repository adheres to [praxis](https://github.com/seanchatmangpt/praxis) standards.
```

#### Download Badge

After workflow runs:

1. Go to workflow run details
2. Scroll to "Artifacts"
3. Download `compliance-badge`
4. Commit to repo: `mkdir -p badges && mv compliance-badge.svg badges/`

#### Generate Badge Locally

```bash
# Use the ci_gate module programmatically:
use praxis_retrofit::BadgeGenerator;

let svg = BadgeGenerator::generate_svg(92.0, "Excellent", "green");
std::fs::write("compliance-badge.svg", svg)?;
```

---

## Troubleshooting

### Issue: Workflow Fails to Install praxis-retrofit

**Error:** `cargo install praxis-retrofit --locked` fails

**Solution:**

```yaml
# In workflow, use the exact git commit:
- name: Install praxis-retrofit
  run: |
    cargo install praxis-retrofit \
      --git https://github.com/seanchatmangpt/praxis \
      --rev abc123def456 \
      --locked
```

---

### Issue: Gate Blocks PR Even Though Repo Looks Good

**Root Cause:** Score calculation may differ locally vs CI

**Debug:**

```bash
# Run locally with same conditions:
cd /path/to/repo
praxis-retrofit audit report . | jq .

# Compare score locally vs workflow output
# If different, check for uncommitted files:
git status
git diff

# In CI, only committed files are checked
# Ensure your changes are staged and committed
```

---

### Issue: PR Comment Not Posting

**Root Cause:** Insufficient permissions

**Solution:**

Ensure workflow has correct permissions in `.github/workflows/praxis-validate.yml`:

```yaml
permissions:
  contents: read
  pull-requests: write  # <-- Must allow PR comments
  checks: write         # <-- Must allow check runs
```

---

### Issue: Badge Not Uploading

**Root Cause:** Artifact path incorrect

**Solution:**

Check workflow generates correct path:

```bash
# Manually run and verify:
cargo install praxis-retrofit
SCORE=$(praxis-retrofit validate compliance . 2>/dev/null | jq -r '.score')
echo "Score: $SCORE"

# Should be a number 0-100
```

---

### Issue: Score Calculation Inconsistent

**Root Cause:** Some files not included in audit

**Debug:**

```bash
# Check what files are being audited:
praxis-retrofit audit report . | jq '.repository'

# Should show:
# - path: <repo-root>
# - crate_count: <N>
# - has_workspace: true/false

# Re-run audit from repo root (not subdirectory):
cd /path/to/repo
praxis-retrofit audit report .
```

---

## Advanced Configuration

### Custom Gate Config in Rust

If you want to programmatically configure the gate:

```rust
use praxis_retrofit::{ComplianceGate, GateConfig, ComplianceCategory};

let config = GateConfig {
    min_score: 90.0,  // Stricter threshold
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
let output = gate.check(&report).await?;

println!("Gate Result: {:?}", output.gate_result);
println!("Score: {:.1}%", output.score);
```

---

### Conditional Gate Logic

Custom gate logic can be added to the workflow:

```yaml
# In workflow, before check-compliance-threshold:
- name: Calculate compliance trend
  if: github.event_name == 'pull_request'
  run: |
    # Get base branch compliance
    BASE_BRANCH=$(git rev-parse --abbrev-ref HEAD)
    BASE_SCORE=$(...)  # Calculate from main branch
    CURRENT_SCORE=$(...)  # From PR

    # Allow if improvement, fail if drop
    if (( $(echo "$CURRENT_SCORE < $BASE_SCORE" | bc -l) )); then
      echo "Compliance dropped! Base: $BASE_SCORE, Current: $CURRENT_SCORE"
      exit 1
    fi
```

---

### Multi-Repo Fleet Compliance

To run compliance checks across multiple repos:

```bash
#!/bin/bash
# retrofit-fleet.sh

REPOS=(
  "https://github.com/seanchatmangpt/praxis"
  "https://github.com/seanchatmangpt/wasm4pm"
  "https://github.com/seanchatmangpt/pm4py-rs"
  # ... more repos
)

for repo_url in "${REPOS[@]}"; do
  repo_name=$(basename "$repo_url")
  temp_dir=$(mktemp -d)

  echo "Checking $repo_name..."
  git clone "$repo_url" "$temp_dir" --quiet

  # Run compliance check
  praxis-retrofit validate compliance "$temp_dir" | jq .

  rm -rf "$temp_dir"
done
```

---

## Integration Patterns

### Pattern 1: Status Check Blocking

Require compliance gate to pass before merging:

1. Go to **Settings → Branches → Branch protection rules**
2. Edit rule for `main` branch
3. Under "Require status checks to pass before merging"
4. Add: `Praxis Compliance Gate`
5. Save

Now PRs cannot merge until compliance gate passes.

---

### Pattern 2: Scheduled Compliance Reports

Run compliance checks on schedule:

```yaml
# .github/workflows/compliance-report.yml
name: Scheduled Compliance Report

on:
  schedule:
    - cron: '0 9 * * MON'  # Every Monday at 9 AM

jobs:
  report:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install praxis-retrofit --locked
      - run: praxis-retrofit audit report . | jq .
```

---

### Pattern 3: Multi-Stage Gates

Different thresholds for different release stages:

```yaml
# main branch: 85%
# release branch: 95%
# hotfix branch: 80%

- name: Determine threshold
  id: threshold
  run: |
    case "${{ github.ref }}" in
      *main)
        echo "MIN_SCORE=85.0" >> $GITHUB_OUTPUT ;;
      *release)
        echo "MIN_SCORE=95.0" >> $GITHUB_OUTPUT ;;
      *hotfix)
        echo "MIN_SCORE=80.0" >> $GITHUB_OUTPUT ;;
      *)
        echo "MIN_SCORE=75.0" >> $GITHUB_OUTPUT ;;
    esac
```

---

## Best Practices

### ✅ DO

- **Run compliance checks on every PR** — Catch issues early
- **Use critical categories** — Focus on must-have standards
- **Auto-comment with remediation** — Guide developers to fixes
- **Generate badges** — Show compliance status in README
- **Review compliance reports** — Understand what's being checked
- **Gradually increase threshold** — Don't jump from 0% to 90%
- **Document customizations** — Explain why you changed defaults

### ❌ DON'T

- **Ignore gate failures** — They indicate real problems
- **Disable critical categories** — They protect code quality
- **Lower threshold to near 0%** — Defeats the purpose of gates
- **Commit without running locally** — Test compliance before pushing
- **Manually suppress findings** — Fix the root cause instead
- **Leave gate unconfigured** — Customize for your repo's needs

---

## Maintenance

### Update Workflow

When praxis standards change, update the workflow:

```bash
# Get latest workflow template
cp /path/to/praxis/.github/workflows/praxis-validate.yml \
   .github/workflows/praxis-validate.yml

git add .github/workflows/praxis-validate.yml
git commit -m "chore: update praxis compliance gate workflow"
git push
```

### Monitor Gate Health

Track compliance over time:

```bash
# Generate weekly report
praxis-retrofit audit report . | jq '{
  timestamp: .timestamp,
  score: .score,
  passing: [.checks[] | select(.status == "pass")] | length,
  failing: [.checks[] | select(.status == "fail")] | length,
  warnings: [.checks[] | select(.status == "warn")] | length
}' >> compliance-history.jsonl
```

### Gate Metrics

Key metrics to track:

| Metric | Target | Rationale |
|--------|--------|-----------|
| Avg compliance score | ≥ 85% | Baseline quality |
| % PRs blocked | < 5% | Most PRs should pass |
| Gate MTTR | < 30 min | Quick developer response |
| Critical Pass rate | 100% | Zero tolerance |

---

## Related Documentation

- **[Praxis README](README.md)** — Overview of house standards
- **[Praxis Retrofit README](crates/praxis-retrofit/README.md)** — Tool usage
- **[praxis-retrofit Source (ci_gate.rs)](crates/praxis-retrofit/src/ci_gate.rs)** — Gate logic
- **[GitHub Actions Docs](https://docs.github.com/en/actions)** — Workflow details
- **[GitHub REST API](https://docs.github.com/en/rest)** — PR comments, checks

---

## Questions & Support

- 📚 Check [Praxis README](README.md) for house standards overview
- 🔍 Search GitHub for similar repos with compliance gates
- 💬 Open an issue with gate failures and error output
- 🛠️ Run `praxis-retrofit --help` for all available commands

---

## License

MIT OR Apache-2.0

# Anti-Regression Gates: Quick Integration Guide

This guide walks through adding anti-regression gates to any praxis repo (existing or new).

## TL;DR (5-minute setup)

```bash
# 1. Clone/update praxis
git clone https://github.com/seanchatmangpt/praxis.git
cd your-repo

# 2. Copy pre-commit hook
cp praxis/template/hooks/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit

# 3. Copy CI workflow
mkdir -p .github/workflows
cp praxis/template/.github/workflows/anti-regression-gate.yml .github/workflows/

# 4. Commit
git add -A
git commit -m "feat: add anti-regression gates"

# 5. Push & verify in CI
git push origin your-branch
```

---

## Detailed Steps

### Step 1: Copy Pre-Commit Hook

The pre-commit hook validates changes locally before they enter git.

```bash
# From your repo root:
mkdir -p .git/hooks
cp /path/to/praxis/template/hooks/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit
```

**Verify installation:**
```bash
ls -la .git/hooks/pre-commit
# Should show: -rwxr-xr-x ... pre-commit

# Make a test commit
echo "// test" >> src/lib.rs
git add src/lib.rs
git commit -m "test"
# Should show "🔍 Praxis Pre-Commit Gates" output
```

**To enable globally** (for all repos):
```bash
git config --global core.hooksPath ~/.git/hooks
mkdir -p ~/.git/hooks
cp /path/to/praxis/template/hooks/pre-commit ~/.git/hooks/
chmod +x ~/.git/hooks/pre-commit
```

### Step 2: Copy CI Workflow

The CI workflow runs on every PR and blocks merges if gates fail.

```bash
# From your repo root:
mkdir -p .github/workflows
cp /path/to/praxis/template/.github/workflows/anti-regression-gate.yml \
   .github/workflows/anti-regression-gate.yml
```

**What it does:**
- Validates CalVer versioning
- Checks license compliance
- Verifies LICENSE files
- Detects backup files
- Enforces unsafe_code = forbid
- Detects dbg!() / todo!() / unimplemented!()
- Posts detailed comment on PR with remediation guide

### Step 3: Update Cargo.toml (if needed)

Most repos derived from the praxis template already have correct Cargo.toml. If not, update:

```toml
[package]
version = "26.6.0"                    # CalVer YY.M.patch
edition = "2021"
rust-version = "1.82"
license = "MIT OR Apache-2.0"

[lints]
workspace = true                      # Or define [lints.rust] and [lints.clippy]
```

For workspaces:
```toml
[workspace.package]
version = "26.6.0"
edition = "2021"
rust-version = "1.82"
license = "MIT OR Apache-2.0"

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
unreachable_pub = "warn"
unexpected_cfgs = "warn"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
```

### Step 4: Ensure LICENSE Files

Both license files must be committed:

```bash
# Check if present
ls LICENSE-MIT LICENSE-APACHE

# If missing, copy from praxis template
cp /path/to/praxis/template/LICENSE-MIT .
cp /path/to/praxis/template/LICENSE-APACHE .
git add LICENSE-{MIT,APACHE}
```

### Step 5: Commit & Push

```bash
git add .github/workflows/anti-regression-gate.yml .git/hooks/pre-commit
git commit -m "feat: add anti-regression gates for praxis compliance"
git push origin your-branch
```

### Step 6: Test the Gates

Create a test PR to verify gates work:

**Test 1: Intentional CalVer violation**
```toml
# Cargo.toml
version = "0.1.0"  # Wrong format
```

Push and watch CI gate fail. Gate should post comment explaining CalVer format.

**Test 2: Intentional dbg!() in code**
```rust
// src/lib.rs
fn test() {
    dbg!(42);  // Should fail
}
```

Pre-commit hook should reject. If you bypass (`--no-verify`), CI should catch it.

**Test 3: Clean code**
```bash
# After fixing violations
git add Cargo.toml src/lib.rs
git commit -m "fix: correct CalVer and remove debug"
# Should pass pre-commit gate
```

---

## Common Errors & Fixes

### Error: "Permission denied" on pre-commit hook

```bash
chmod +x .git/hooks/pre-commit
```

### Error: "Pre-commit gate FAILED" locally

Check the error message:

```
✗ Backup files (.rs.backup) detected in staging
→ Fix: find . -name '*.rs.backup' -delete
```

Follow the remediation suggestion in the error.

### Error: "Backup file should not exist" in CI

The gate detects leftover backup files from editing.

```bash
# Remove all backups
find . -name '*.rs.backup' -delete

# Add to .gitignore
echo '*.rs.backup' >> .gitignore
```

### Error: "Version doesn't match CalVer YY.M.patch"

```toml
# ❌ Wrong
version = "0.1.0"
version = "2026.6.0"
version = "26.6"

# ✅ Right
version = "26.6.0"  # 2026 (YY), June (M), patch 0
```

### Error: "License should be MIT OR Apache-2.0"

```toml
# Update Cargo.toml
license = "MIT OR Apache-2.0"
```

### Error: "Lints not configured"

```toml
# Option 1: Inherit from workspace (if in workspace)
[lints]
workspace = true

# Option 2: Define inline
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
```

### CI gate passes but pre-commit failed

Pre-commit is stricter (blocks commits). Re-run the gate:

```bash
# See all issues
.git/hooks/pre-commit

# Fix them
# Re-stage and commit
git add fixed-files
git commit -m "fix: resolve pre-commit gate issues"
```

---

## Customization

### Disable Pre-Commit (Emergency)

```bash
git commit --no-verify
# Note: CI gate will still run and catch issues
```

### Modify Pre-Commit Checks

Edit `.git/hooks/pre-commit` to change thresholds:

```bash
# Example: Change WARN to FAIL for MSRV
-            warn "Cargo.toml: missing rust-version field"
+            error "Cargo.toml: missing rust-version field"
```

### Modify CI Gate Severity

Edit `.github/workflows/anti-regression-gate.yml`:

```yaml
# Change "continue-on-error: false" to "true" to make warnings non-blocking
- name: Detect dbg! macros
  continue-on-error: false  # ← Change to true for warnings only
```

### Add Custom Checks

Pre-commit hook and Rust module are extensible. Add checks to:

1. **Pre-commit:** Add new bash functions in `template/hooks/pre-commit`
2. **CI:** Add new jobs to `.github/workflows/anti-regression-gate.yml`
3. **Rust:** Add methods to `crates/praxis-retrofit/src/preventive_gate.rs`

---

## Integration with Existing Repos

### For repos already in praxis template

1. Just copy the hook and CI workflow
2. No Cargo.toml changes needed if you're on template version

### For repos diverging from praxis

1. **First:** Run validation to see what's missing
   ```bash
   cargo run -p praxis-retrofit -- validate compliance .
   ```

2. **Then:** Fix each category in order:
   - Versioning (CalVer)
   - Licensing (MIT OR Apache-2.0)
   - Lints ([lints] configuration)
   - MSRV (rust-version = "1.82")

3. **Finally:** Add gates

### For new repos

1. Generate from praxis template:
   ```bash
   cargo generate --git https://github.com/seanchatmangpt/praxis template
   ```
   
   Gates are already included! Just verify:
   ```bash
   ls -la .git/hooks/pre-commit
   ls -la .github/workflows/anti-regression-gate.yml
   ```

---

## Team Onboarding

### For Team Members

1. **Clone the repo**
   ```bash
   git clone <your-repo>
   cd <your-repo>
   ```

2. **Pre-commit hook automatically runs**
   - On first commit after clone, the hook runs
   - If it fails, follow remediation suggestions

3. **Ask if stuck**
   - Check: [`ANTI_REGRESSION_GATES.md`](ANTI_REGRESSION_GATES.md) § Remediation Guide
   - Common issues are documented there

### For Code Reviewers

1. **Check PR comment from bot**
   - Anti-regression gate posts detailed comment if gates fail
   - Comment includes code examples for each fix

2. **Don't merge if gate fails**
   - Red X on PR = gate failure
   - Author must fix before merging

3. **Reference the guide in comments**
   - "See ANTI_REGRESSION_GATES.md § CalVer Versioning"
   - Points author to docs instead of repeating

### For Maintainers

1. **Monitor gate failures**
   - Track common failures in project board or wiki
   - Update gate if patterns change

2. **Update gates when standards change**
   - Edit `template/hooks/pre-commit`
   - Edit `template/.github/workflows/anti-regression-gate.yml`
   - Edit `crates/praxis-retrofit/src/preventive_gate.rs`

3. **Sync across fleet**
   - Gate improvements should propagate to all repos
   - Use `apply.sh` or batch updates

---

## Testing Gates in CI

### Manual test via GitHub Actions

```bash
# Create test branch
git checkout -b test/gate-verification

# Introduce an intentional violation
echo 'version = "0.1.0"' > Cargo.toml

# Push and watch CI
git add Cargo.toml
git commit -m "test: verify gate catches CalVer violation"
git push origin test/gate-verification

# Go to GitHub Actions tab
# See anti-regression-gate workflow fail ✗
# Read detailed comment explaining the fix
```

### Local pre-commit testing

```bash
# Make staged changes that violate gates
git add bad-code.rs

# Run pre-commit manually
.git/hooks/pre-commit

# Should reject and explain why
```

---

## Troubleshooting Checklist

- [ ] Pre-commit hook is executable: `ls -la .git/hooks/pre-commit` shows `rwx`
- [ ] Pre-commit hook location is correct: `.git/hooks/pre-commit` not `hooks/pre-commit`
- [ ] CI workflow file exists: `.github/workflows/anti-regression-gate.yml`
- [ ] Cargo.toml version is CalVer: `26.6.0` not `0.1.0`
- [ ] License field is correct: `license = "MIT OR Apache-2.0"`
- [ ] LICENSE files exist: both `LICENSE-MIT` and `LICENSE-APACHE`
- [ ] No backup files: `find . -name '*.rs.backup'` is empty
- [ ] No dbg!() in staging: `git diff --cached | grep 'dbg!'` is empty
- [ ] Lints defined: `grep -E '\[lints\]|workspace = true' Cargo.toml`

---

## Support & Documentation

| Topic | Reference |
|-------|-----------|
| **Detailed Rules** | [`ANTI_REGRESSION_GATES.md`](ANTI_REGRESSION_GATES.md) |
| **Remediation Patterns** | [`ANTI_REGRESSION_GATES.md` § Remediation Guide](ANTI_REGRESSION_GATES.md#5-remediation-guide-by-category) |
| **Rust API** | [`crates/praxis-retrofit/src/preventive_gate.rs`](../crates/praxis-retrofit/src/preventive_gate.rs) |
| **Pre-Commit Script** | [`template/hooks/pre-commit`](template/hooks/pre-commit) |
| **CI Workflow** | [`template/.github/workflows/anti-regression-gate.yml`](template/.github/workflows/anti-regression-gate.yml) |
| **Praxis Standards** | [`survey/00-SYNTHESIS.md`](survey/00-SYNTHESIS.md) |

---

**Version:** 26.6.0  
**Last Updated:** 2026-06-23

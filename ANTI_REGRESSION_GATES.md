# Anti-Regression Gates: Preventing Praxis Drift

**Overview:** Automated, multi-layered gates that prevent repos from drifting away from praxis standards. Three complementary mechanisms enforce compliance at commit time (pre-commit hook), pull request time (CI workflow), and programmatically (Rust validation module).

---

## 1. Pre-Commit Hook (Local Enforcement)

**Location:** `template/hooks/pre-commit`

Runs on every commit attempt. Prevents non-compliant changes from ever entering git history.

### Coverage

| Check | Status | Remediation |
|-------|--------|-------------|
| **CalVer versioning** (YY.M.patch) | 🚫 Fail | Update `Cargo.toml` version to format like `26.6.0` |
| **License compliance** (MIT OR Apache-2.0) | ⚠️ Warn | Change `license = "MIT OR Apache-2.0"` |
| **MSRV field present** | ⚠️ Warn | Add `rust-version = "1.82"` |
| **[lints] or inheritance** | ⚠️ Warn | Define `[lints]` with `workspace = true` |
| **unsafe_code = "forbid"** | ⚠️ Warn | Set in `[lints.rust]` section |
| **Clippy denies** (todo/unimplemented/dbg) | ⚠️ Warn | Set in `[lints.clippy]` |
| **Backup files** (*.rs.backup) | 🚫 Fail | `find . -name '*.rs.backup' -delete` |
| **dbg!() macro** | 🚫 Fail | Remove from staged changes |
| **todo!/unimplemented!()** | 🚫 Fail | Replace with proper error handling |
| **Pub fn docs** | ⚠️ Warn | Add `///` doc comments |

### Installation

Enable the pre-commit hook:

```bash
chmod +x template/hooks/pre-commit
ln -s ../../template/hooks/pre-commit .git/hooks/pre-commit
```

Or install via Git:

```bash
git config core.hooksPath template/hooks
```

### Example Run

```bash
$ git add src/lib.rs Cargo.toml
$ git commit -m "Add feature X"

🔍 Praxis Pre-Commit Gates
─────────────────────────────────────────────────────────────────────

✓ Pre-commit gate passed
```

### Bypass (Emergency)

```bash
git commit --no-verify    # Bypass pre-commit (not recommended)
```

---

## 2. CI Workflow (PR Enforcement)

**Location:** `template/.github/workflows/anti-regression-gate.yml`

Runs on all pull requests. Provides detailed feedback and auto-comments with remediation guidance.

### Jobs

#### `praxis-compliance`
- CalVer version format check
- License field validation
- LICENSE file presence (both MIT and Apache)
- Backup file detection
- [lints] configuration verification

#### `unsafe-code-gate`
- Enforces `unsafe_code = "forbid"` (with exceptions for linkme/WASM/specgen)
- Runs clippy with deny-todo/unimplemented/dbg_macro

#### `pattern-gate`
- Detects `dbg!()` macros (hard fail)
- Detects `panic!()` in library code (warning)
- Detects `unwrap()` in library code (warning)

#### `workspace-gate`
- Validates workspace structure
- Checks for duplicate lints definitions
- Verifies member configuration

#### `msrv-consistency`
- Extracts declared MSRV
- Verifies minimum 1.82
- Ensures consistency across workspace

#### `report-results`
- Posts detailed comment on PR if any gate failed
- Includes remediation guide with code examples
- Reference links to praxis standards

### Example PR Comment

When a gate fails, GitHub bot posts:

```
## ⚠️ Praxis Anti-Regression Gate Failed

This PR introduces patterns that violate praxis house standards. Please review and fix:

### Common Issues & Remediation

#### 1. CalVer Versioning
**Issue:** Version doesn't match `YY.M.patch` format
**Fix:**
version = "26.6.0"  # YY.M.patch format only
```

(Full comment includes all failed checks and inline code examples)

---

## 3. Rust Validation Module (Programmatic)

**Location:** `crates/praxis-retrofit/src/preventive_gate.rs`

Programmatic validation engine. Use in tooling, scripts, or external validators.

### API

```rust
use praxis_retrofit::preventive_gate::{GateValidator, ValidationStatus};
use std::path::Path;

// Create validator with house defaults
let validator = GateValidator::new();

// Validate Cargo.toml
let results = validator.validate_cargo_toml(Path::new("Cargo.toml"))?;

// Check for patterns in Rust code
let pattern_results = validator.validate_rust_patterns(Path::new("src"))?;

// Validate required files
let file_results = validator.validate_required_files(Path::new("."))?;

// Generate markdown report
let report = GateReport::new(results);
println!("{}", report.to_markdown());

// Check if compliant
if report.is_compliant() {
    println!("✓ All gates passed");
} else {
    println!("✗ {} failures detected", fail_count);
    for suggestion in report.remediation_suggestions() {
        println!("  - {}", suggestion);
    }
}
```

### Checks Performed

#### Version Validation
- Parses CalVer format (YY.M.patch)
- Returns Pass/Warn/Fail status with remediation

#### License Compliance
- Checks Cargo.toml license field
- Prefers `MIT OR Apache-2.0` but allows variants
- Suggests upgrade path

#### MSRV Validation
- Extracts `rust-version` from Cargo.toml
- Compares against house minimum (1.82)
- Handles both single-crate and workspace forms

#### Lint Configuration
- Validates `[lints]` presence
- Checks for workspace inheritance
- Verifies inline lint levels (unsafe_code, todo, unimplemented, dbg_macro)
- Provides detailed remediation for missing lints

#### Pattern Detection
- Scans Rust files for disallowed macros (dbg!, todo!, unimplemented!)
- Skips tests/examples/benches (more lenient)
- Returns line numbers and file paths

#### File Validation
- Checks for LICENSE-MIT, LICENSE-APACHE
- Verifies Cargo.toml, rust-toolchain.toml, rustfmt.toml
- Detects backup files (*.rs.backup)

### Output Types

```rust
pub struct ValidationResult {
    pub status: ValidationStatus,        // Pass/Warn/Fail
    pub category: ValidateCategory,      // Versioning/Licensing/Linting/...
    pub check_name: String,              // "CalVer format"
    pub message: String,                 // "Version 26.6.0 matches..."
    pub remediation: Option<String>,     // "Update to YY.M.patch"
    pub severity: Severity,              // Info/Warning/Error/Critical
}

pub struct GateReport {
    pub results: Vec<ValidationResult>,
    pub timestamp: String,
    // Methods:
    // - is_compliant() -> bool
    // - status_counts() -> (pass, warn, fail)
    // - remediation_suggestions() -> Vec<&str>
    // - to_markdown() -> String
}
```

### Integration Examples

#### As a Library Dependency

```toml
[dependencies]
praxis-retrofit = { path = "../praxis-retrofit", features = ["validate"] }
```

#### In a Build Script (build.rs)

```rust
use praxis_retrofit::preventive_gate::GateValidator;

fn main() {
    let validator = GateValidator::new();
    
    match validator.validate_cargo_toml(Path::new("Cargo.toml")) {
        Ok(results) => {
            let report = GateReport::new(results);
            if !report.is_compliant() {
                eprintln!("Build validation failed:");
                eprintln!("{}", report.to_markdown());
                std::process::exit(1);
            }
        }
        Err(e) => eprintln!("Validation error: {}", e),
    }
}
```

#### In CI/CD Scripts

```bash
#!/usr/bin/env bash
cargo run -p praxis-retrofit -- validate compliance ./
```

---

## 4. Enforcement Flow

```
Developer commits code
    ↓
[1] Pre-commit hook runs locally
    ├─ Checks Cargo.toml format, backups, macros
    ├─ Fails = commit rejected locally
    └─ Warns = commit allowed (advisory)
    ↓
Code enters local git history
    ↓
Developer pushes to GitHub
    ↓
[2] CI Anti-Regression Gate workflow runs
    ├─ Runs praxis-compliance job
    ├─ Runs unsafe-code-gate job
    ├─ Runs pattern-gate job
    ├─ Runs workspace-gate job
    ├─ Runs msrv-consistency job
    ├─ Fails = PR blocked, comment posted
    └─ Success = PR can merge
    ↓
PR merged to main
    ↓
Code released
```

---

## 5. Remediation Guide by Category

### A. Versioning

**Problem:** Version doesn't match CalVer YY.M.patch

```toml
# ❌ Wrong
version = "0.1.0"
version = "2026.06.01"
version = "26.6"

# ✅ Right
version = "26.6.0"    # 2026, month 6, patch 0
version = "26.6.17"   # 2026, month 6, patch 17
version = "25.12.5"   # 2025, month 12, patch 5
```

**Fix:**
```bash
# Find current version
grep '^version' Cargo.toml

# Update to CalVer (YY.M.patch)
# YY = last 2 digits of year
# M  = month (1-12, no leading zero)
# patch = incrementing within the month
```

### B. Licensing

**Problem:** License not dual MIT OR Apache-2.0

```toml
# ❌ Wrong
license = "MIT"
license = "Apache-2.0"
license = "GPL-3.0"

# ✅ Right
license = "MIT OR Apache-2.0"
```

**Fix:**
```bash
# Update Cargo.toml
sed -i 's/^license = .*/license = "MIT OR Apache-2.0"/' Cargo.toml

# Ensure LICENSE files exist
cp /path/to/praxis/template/LICENSE-MIT .
cp /path/to/praxis/template/LICENSE-APACHE .
git add LICENSE-MIT LICENSE-APACHE
```

### C. Linting Configuration

**Problem:** No [lints] or wrong levels

```toml
# ❌ Wrong
[package]
name = "my-crate"

# ✅ Right (single crate)
[lints]
workspace = true

# ✅ Right (or define inline)
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
unwrap_used = "warn"
expect_used = "warn"
```

**Fix (fast):**
```bash
# Copy from template
cp template/Cargo.toml.lints-snippet Cargo.toml.snippet
# Manually merge the [lints] section
```

**Fix (workspace):**
```toml
# Cargo.toml (root)
[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
# ...

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
# ...

# Cargo.toml (each member)
[lints]
workspace = true
```

### D. MSRV (Minimum Supported Rust Version)

**Problem:** MSRV below 1.82 or missing

```toml
# ❌ Wrong
rust-version = "1.70"
# Missing rust-version field

# ✅ Right
rust-version = "1.82"
```

**Fix:**
```bash
# Update Cargo.toml
sed -i '/^\[package\]/a rust-version = "1.82"' Cargo.toml
```

### E. Unsafe Code

**Problem:** unsafe_code not forbid

```rust
// ❌ Wrong — unsafe_code not in [lints.rust]

// ✅ Right
#[lints.rust]
unsafe_code = "forbid"

// Exception (linkme, WASM, specgen crates):
#[lints.rust]
unsafe_code = "warn"  # or "allow"
```

**Fix:**
```toml
[lints.rust]
unsafe_code = "forbid"
```

### F. Macros (dbg!, todo!, unimplemented!)

**Problem:** Disallowed macros in code

```rust
// ❌ Wrong
fn process() {
    dbg!(x);           // debug macro
    todo!("implement") // placeholder
    unimplemented!()   // stub
}

// ✅ Right
fn process() -> Result<T> {
    println!("debug: {:?}", x);  // remove before commit
    Err(Error::NotImplemented)?  // proper error
    Err(anyhow::anyhow!("stub")) // temporary with doc
}
```

**Fix:**
```bash
# Find instances
grep -rn 'dbg!\|todo!\|unimplemented!' src/

# Remove debug prints
grep -rn 'dbg!' src/ | sed 's/:.*dbg!/: remove debug/' | head -5

# Replace todo! with proper errors
# Option 1: Return error
Err(anyhow::anyhow!("not yet implemented: reason"))

# Option 2: Panic (tests only)
#[cfg(test)]
fn stub() { panic!("not implemented in test"); }
```

### G. Backup Files

**Problem:** *.rs.backup files committed

```bash
# ❌ Wrong
src/lib.rs.backup
src/verbs/emit.rs.backup

# ✅ Right
# No .backup files
```

**Fix:**
```bash
# Remove all backups
find . -name '*.rs.backup' -delete

# Add to .gitignore
echo '*.rs.backup' >> .gitignore
echo '**/*.rs.backup' >> .gitignore

# Verify removed
git ls-files | grep '\.rs\.backup'  # Should be empty
```

---

## 6. Multi-Crate Workspace Gates

For workspaces, each member must:

1. **Inherit lints from workspace root**
   ```toml
   # Cargo.toml (member)
   [lints]
   workspace = true
   ```

2. **Use workspace.package values**
   ```toml
   # Cargo.toml (root)
   [workspace.package]
   version = "26.6.0"
   edition = "2021"
   rust-version = "1.82"
   license = "MIT OR Apache-2.0"

   # Cargo.toml (member)
   [package]
   name = "my-member"
   version.workspace = true
   edition.workspace = true
   rust-version.workspace = true
   license.workspace = true
   ```

3. **Share lints via [workspace.lints]**
   ```toml
   # Cargo.toml (root)
   [workspace.lints.rust]
   unsafe_code = "forbid"

   [workspace.lints.clippy]
   all = "warn"

   # Cargo.toml (member)
   [lints]
   workspace = true
   ```

---

## 7. Exceptions & Edge Cases

### linkme Crate (Unsafe Required)

```toml
[lints.rust]
unsafe_code = "warn"  # linkme requires unsafe code
```

Gate logic skips forbid-check for crates named `linkme`, `wasm*`, `specgen`.

### WASM Crates

```toml
[lints.rust]
unsafe_code = "warn"  # wasm-bindgen requires unsafe

[profile.release]
opt-level = "s"       # Small WASM binaries
panic = "abort"       # Reduce WASM footprint
```

### Tests & Examples

Pre-commit hook excludes:
- `tests/`
- `examples/`
- `benches/`

These are allowed to use `unwrap()`, `panic!()` more liberally.

---

## 8. Troubleshooting

### Pre-commit hook not running

```bash
# Check if hook exists and is executable
ls -la .git/hooks/pre-commit

# Make executable
chmod +x .git/hooks/pre-commit

# Verify git config
git config core.hooksPath  # Should show template/hooks
```

### CI gate fails but code looks correct

1. Check the PR comment for exact failure reason
2. Verify all required Cargo.toml fields exist
3. Run locally: `praxis-retrofit validate compliance .`
4. Compare against template: `diff Cargo.toml template/Cargo.toml`

### License files missing but Cargo.toml correct

```bash
# Both files must be committed
ls -l LICENSE-{MIT,APACHE}

# Copy from praxis template
cp /path/to/praxis/template/LICENSE-MIT .
cp /path/to/praxis/template/LICENSE-APACHE .
git add LICENSE-{MIT,APACHE}
git commit -m "chore: add license files"
```

### MSRV validation fails

```bash
# Check declared MSRV
grep 'rust-version' Cargo.toml

# Verify it's >= 1.82
# Format: "1.82" not "1.82.0"
# No trailing ".0"

# Test building with declared version
rustup install 1.82
cargo +1.82 check
```

### Workspace members not inheriting lints

```bash
# Each member must have:
[lints]
workspace = true

# NOT:
[lints.rust]
unsafe_code = "forbid"  # Wrong, should inherit

# Root must define:
[workspace.lints.rust]
unsafe_code = "forbid"
```

---

## 9. Configuration Reference

### Gate Severity Levels

| Level | Blocks PR | Blocks Commit | Example |
|-------|-----------|---------------|---------|
| **Critical** | ✓ | ✓ | CalVer version fail |
| **Error** | ✓ | ✓ | dbg!() macro detected |
| **Warning** | ✗ | ✗ | Missing MSRV field |
| **Info** | ✗ | ✗ | Configuration correct |

### CalVer Rules

```
Format: YY.M.patch

YY     = last 2 digits of current year (00-99)
M      = month (1-12, no leading zero)
patch  = incrementing counter for multiple releases in same month

Examples:
26.6.0    = June 2026, patch 0 (first release)
26.6.5    = June 2026, patch 5 (fifth release)
26.12.10  = December 2026, patch 10
27.1.0    = January 2027, patch 0
```

### House Defaults

| Setting | Default | Rationale |
|---------|---------|-----------|
| Edition | 2021 | Stable, widely-deployed |
| MSRV | 1.82 | Median across fleet |
| License | MIT OR Apache-2.0 | Permissive dual |
| unsafe_code | forbid | Security hardening |
| Clippy all | warn | Catch style issues |
| Clippy pedantic | warn | Extra strictness |
| todo! | deny | Disallow stubs |
| unimplemented! | deny | Disallow incomplete code |
| dbg_macro | deny | Remove debug output |

---

## 10. Integration Checklist

To add anti-regression gates to a repo:

- [ ] Copy `template/hooks/pre-commit` → `.git/hooks/pre-commit` (chmod +x)
- [ ] Copy `template/.github/workflows/anti-regression-gate.yml` → `.github/workflows/`
- [ ] Add `praxis-retrofit` to CI: `cargo install praxis-retrofit`
- [ ] Run initial validation: `praxis-retrofit validate compliance .`
- [ ] Fix any failures before merging main
- [ ] Verify pre-commit hook runs on next commit
- [ ] Test CI gate by opening a draft PR with intentional violations
- [ ] Train team on remediation patterns (point to this guide)
- [ ] Add link to this doc in CONTRIBUTING.md

---

## References

- **Praxis Standards:** [`survey/00-SYNTHESIS.md`](survey/00-SYNTHESIS.md)
- **Template Cargo.toml:** [`template/Cargo.toml`](template/Cargo.toml)
- **Lint Rules:** [`template/Cargo.toml` § Lints](template/Cargo.toml#L20-L35)
- **Rust Module API:** [`crates/praxis-retrofit/src/preventive_gate.rs`](crates/praxis-retrofit/src/preventive_gate.rs)
- **CI Workflow:** [`template/.github/workflows/anti-regression-gate.yml`](template/.github/workflows/anti-regression-gate.yml)

---

**Last updated:** 2026-06-23  
**Owner:** seanchatmangpt

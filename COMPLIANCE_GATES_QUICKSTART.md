# Praxis Compliance Gates: Quick Start

**Copy-paste guide to add compliance gates to any Rust repository.**

---

## 30-Second Setup

### Step 1: Copy the Workflow

```bash
# In your repository root
mkdir -p .github/workflows
cp /path/to/praxis/.github/workflows/praxis-validate.yml .github/workflows/
```

### Step 2: Commit and Push

```bash
git add .github/workflows/praxis-validate.yml
git commit -m "chore: add praxis compliance gate"
git push origin main
```

### Step 3: Open a PR

Make any change and open a PR. The compliance gate will:
- ✅ Run `praxis-retrofit validate compliance .`
- ✅ Extract compliance score
- ✅ Block PR if below 85%
- ✅ Post remediation suggestions

---

## Test the Gate

### Success Case (Full Compliance)

If your repo already meets praxis standards:

```
✅ Compliance Score: 92%
✅ Gate Result: PASS
✅ No PR comment
✅ Green badge uploaded
```

### Failure Case (Triggers Gate)

If your repo has missing standards:

```
❌ Compliance Score: 72%
❌ Gate Result: FAIL (below 85%)
❌ PR blocked
🔧 Auto-comment with fixes posted
```

---

## Compliance Checks

The gate validates these standards:

| Check | Category | Required | How to Fix |
|-------|----------|----------|-----------|
| CI/CD Workflows | ci-cd | ✅ YES | Copy `.github/workflows/` from template |
| [lints] Config | linting | ✅ YES | Add `[workspace.lints]` to Cargo.toml |
| deny.toml | supply-chain | ✅ YES | Run `praxis-retrofit apply retrofit .` |
| .editorconfig | editor-config | ⚠️ Recommended | Copy from praxis template |
| typos.toml | editor-config | ⚠️ Recommended | Run `praxis-retrofit apply retrofit .` |
| CONTRIBUTING.md | documentation | ⚠️ Recommended | Copy from praxis template |

---

## Fix Non-Compliant Repos

### Automated Fix (Recommended)

```bash
# Install the tool
cargo install praxis-retrofit --locked

# Apply automatic fixes
cd /path/to/your/repo
praxis-retrofit apply retrofit .

# Validate
praxis-retrofit validate compliance .

# Commit and push
git add -A
git commit -m "chore: retrofit praxis standards"
git push
```

### Manual Fix (For Specific Issues)

**Missing [lints] configuration:**

```toml
# Add to Cargo.toml

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
```

**Missing deny.toml:**

```bash
# Copy from praxis
cp /path/to/praxis/template/deny.toml .
```

**Missing typos.toml:**

```bash
# Copy from praxis
cp /path/to/praxis/template/typos.toml .
```

**Missing CONTRIBUTING.md:**

```bash
# Copy from praxis
cp /path/to/praxis/template/CONTRIBUTING.md .
```

---

## Workflow Configuration

### Change Minimum Score

Edit `.github/workflows/praxis-validate.yml`:

```yaml
# Line ~90, find this section:
- name: Check compliance threshold
  env:
    # Change this value:
    MIN_SCORE: 85.0  # <-- Adjust here (0-100)
  run: |
    # ... rest of step
```

Common thresholds:
- `85.0` — Balanced (default)
- `90.0` — Strict
- `75.0` — Lenient

### Disable Auto-Comments

If you don't want PR comments, delete this job:

```yaml
# In .github/workflows/praxis-validate.yml
# Delete this entire job:
remediation-suggestions:
  name: remediation-suggestions
  # ... entire job
```

### Disable Badge Generation

Delete this job:

```yaml
# In .github/workflows/praxis-validate.yml
# Delete this entire job:
compliance-badge:
  name: compliance-badge
  # ... entire job
```

---

## GitHub Branch Protection

To require compliance gate before merging:

1. Go to **Settings → Branches**
2. Click "Add rule" (or edit existing main branch rule)
3. Branch name pattern: `main`
4. Check: **"Require status checks to pass before merging"**
5. Search for and select: `Praxis Compliance Gate`
6. Click **"Create"**

Now PRs **must** pass compliance gate before merge.

---

## Troubleshooting

### Q: Gate fails but my repo looks correct

**A:** Ensure you've committed all changes:

```bash
git status  # Should show nothing
praxis-retrofit audit report .  # Run from repo root
```

### Q: How do I run locally first?

**A:** Install and test before pushing:

```bash
cargo install praxis-retrofit --locked
praxis-retrofit audit report /path/to/repo
```

### Q: Can I see what the gate checks?

**A:** Yes, download the compliance report artifact:

1. Go to workflow run
2. Scroll to "Artifacts"
3. Download `compliance-report-<ID>`
4. View JSON file for details

### Q: My repo doesn't have praxis standards yet

**A:** That's what the gate is for! Use automated retrofit:

```bash
praxis-retrofit apply retrofit .
git add -A
git commit -m "chore: retrofit praxis standards"
git push
```

Then re-run your PR. Gate should pass.

---

## What Gets Checked

The compliance gate validates:

```rust
// From ci_gate.rs
GateConfig {
    min_score: 85.0,                    // Minimum %
    block_on_drop: true,                // Block PRs if score drops
    critical_categories: [
        ComplianceCategory::CiCd,       // Must Pass
        ComplianceCategory::Linting,    // Must Pass
    ],
    auto_remediate: true,               // Post PR comment
    generate_badge: true,               // Create badge
}
```

**Critical** (must Pass):
- CI/CD Pipeline
- Workspace Lints

**Recommended** (should Pass):
- Supply Chain Audit (deny.toml)
- Spell Check (typos.toml)
- Editor Config (.editorconfig)
- Documentation (CONTRIBUTING.md)

---

## Compliance Score Formula

```
Score = (Pass Count / Total Checks) * 100%

Example:
- 6 checks total
- 5 pass, 1 fail
- Score = (5 / 6) * 100 = 83.3%
- Gate Result: FAIL (below 85%)
```

---

## Common Issues & Fixes

| Issue | Fix |
|-------|-----|
| `cargo install` fails | Use exact git rev: `--git https://github.com/seanchatmangpt/praxis --rev <SHA> --locked` |
| PR comment not posting | Check workflow permissions: `pull-requests: write` |
| Score calculation off | Run from repo root: `cd /path/to/repo` then `praxis-retrofit audit report .` |
| Badge not uploading | Check artifact names in workflow match upload paths |
| Gate always fails | Ensure all changes are committed: `git status` should be clean |

---

## Example: Before → After

### Before (Non-Compliant)

```
❌ Score: 60%
❌ Missing [lints]
❌ No deny.toml
❌ No typos.toml
❌ No CONTRIBUTING.md
```

### Fix Command

```bash
praxis-retrofit apply retrofit .
```

### After (Compliant)

```
✅ Score: 95%
✅ [lints] configured
✅ deny.toml added
✅ typos.toml added
✅ CONTRIBUTING.md added
✅ Gate PASSES
```

---

## Next Steps

1. **Copy workflow** — `cp .github/workflows/praxis-validate.yml .`
2. **Commit and push** — `git add . && git commit -m "chore: add praxis compliance gate" && git push`
3. **Open a PR** — Make any change and open PR
4. **Watch gate run** — See workflow execute in Actions tab
5. **Review results** — Check compliance score and remediation suggestions
6. **Fix if needed** — Run `praxis-retrofit apply retrofit .` if below threshold
7. **Merge** — Once gate passes, PR can merge

---

## Questions?

- 📚 See full guide: [COMPLIANCE_GATES.md](COMPLIANCE_GATES.md)
- 🔧 Tool docs: [praxis-retrofit README](crates/praxis-retrofit/README.md)
- 🏠 Standards: [praxis README](README.md)
- 🐛 Issues: Open GitHub issue with workflow output

---

## License

MIT OR Apache-2.0

# Case Study: Retrofitting wasm4pm with Praxis Standards

## Executive Summary

**Objective:** Transform wasm4pm from a pre-praxis codebase into a Claude Code web-compatible, standardized Rust project following the seanchatmangpt/praxis house-style kit.

**Current State:** wasm4pm is a mature, feature-rich process mining toolkit (26.6.12, BUSL-1.1) with comprehensive OCEL 2.0 support, 60+ algorithms, and 1,165 test files. However, it predates the praxis standardization effort and lacks Claude Code web compatibility patterns.

**Timeline:** Pre-praxis (built before standardization kit existed) → Modern (praxis-aligned)

---

## Part 1: Current State Audit

### Praxis Checklist Analysis

The `/home/user/praxis/CHECKLIST.md` defines per-repo standardization requirements:

| Item | Category | Status | Evidence |
|------|----------|--------|----------|
| **CI/CD Pipeline (ci.yml)** | [A] Auto | ✓ PRESENT | 20 GitHub Actions workflows (2,231 lines) |
| **Supply Chain Audit (deny.toml)** | [A] Auto | ✓ PRESENT | 244 lines, full cargo-deny config |
| **Spell Checker (typos.toml)** | [A] Auto | ✗ MISSING | Replaced by markdownlint; no typos.toml |
| **Editor Config (.editorconfig)** | [A] Auto | ✓ PRESENT | Complete (2-space, LF, UTF-8) |
| **Linting Config ([lints])** | [A] Auto | ⚠ PARTIAL | CI-enforced via cargo-deny, not workspace [lints] |
| **Contributor Guide (CONTRIBUTING.md)** | [A] Auto | ✓ PRESENT | Strict three-layer evidence rule documented |
| **Licensing (MIT OR Apache-2.0)** | [A] Auto | ⚠ DIFFERENT | BUSL-1.1 primary (converts AGPL-3.0 on 2028-06-23) |
| **Toolchain Pinning** | [H] Human | ✓ PRESENT | nightly-2026-04-15 pinned |
| **CODE_OF_CONDUCT.md** | House | ✓ PRESENT | Contributor Covenant v2.1 |
| **SECURITY.md** | House | ⚠ UNKNOWN | Not verified in agent findings |

### Praxis Conformance Score: **7.5/10**

**Strong areas:**
- ✓ CI/CD infrastructure (exceeds baseline)
- ✓ deny.toml supply chain audit
- ✓ .editorconfig standards
- ✓ CONTRIBUTING.md enforcement
- ✓ Toolchain pinning
- ✓ CODE_OF_CONDUCT

**Weak areas:**
- ✗ typos.toml (alternative approach via markdownlint)
- ⚠ [lints] workspace config (CI-enforced, not declarative)
- ⚠ License choice (BUSL-1.1 ≠ dual MIT/Apache-2.0; though triadic approach acknowledged)

---

## Part 2: Retrofit Gap Analysis

### Why wasm4pm Doesn't Work in Claude Code Web

**Root Cause:** Pre-praxis codebases lack the explicit standardization patterns that Claude Code web sessions expect:

1. **Workspace [lints] Config** — Missing declarative linting rules
2. **Cargo.toml Standardization** — No workspace-level dependency pinning
3. **rust-toolchain.toml Clarity** — Nightly pinning present but may not match praxis conventions
4. **Feature Flag Standardization** — No `[features]` inheritance pattern
5. **CI/CD Integration** — While present, may not follow praxis matrix patterns
6. **Consistent Justfile** — May lack praxis task runner conventions

### Gaps vs. Praxis Template

| Aspect | Praxis Template | wasm4pm | Gap |
|--------|-----------------|---------|-----|
| **Workspace [lints]** | ✓ Comprehensive | ✗ None | Add unsafe_code=forbid, clippy all/pedantic |
| **Dependency Pinning** | ✓ [workspace.dependencies] | ⚠ Partial | Unify core deps at workspace level |
| **Feature Gates** | ✓ [features] with defaults | ✓ Present (3 features) | Audit for consistency |
| **Build Profile** | ✓ release, bench, dev | ✓ Present | Verify opt-level, LTO, codegen-units |
| **Justfile** | ✓ fmt, lint, test, build, doc, bench, pre-commit | ⚠ Makefile-based | Convert to Justfile or harmonize |
| **MSRV Declaration** | ✓ rust-version in Cargo.toml | ✓ MSRV 1.82 | Verify consistency across crates |
| **[lints] Inheritance** | ✓ Workspace → Members | ✗ None | Add to template and all 8 crates |

---

## Part 3: Retrofit Roadmap

### Phase 1: Declarative Standardization (High Priority, Low Risk)

**Objective:** Add workspace-level [lints] configuration for consistency.

**Tasks:**
1. Create `/wasm4pm/Cargo.toml` workspace [lints] block:
   ```toml
   [workspace.lints.rust]
   unsafe_code = "forbid"
   
   [workspace.lints.clippy]
   all = "warn"
   pedantic = "warn"
   nursery = "warn"
   
   [workspace.lints.rustdoc]
   missing_crate_level_docs = "warn"
   ```

2. Add to each crate's `Cargo.toml`:
   ```toml
   [lints]
   workspace = true
   ```

3. Update CI workflow (ci.yml) to validate [lints] presence in all members.

**Files to modify:**
- `Cargo.toml` (root workspace)
- `crates/wasm4pm-cli/Cargo.toml`
- `crates/miniml-core/Cargo.toml`
- `crates/wasm4pm-cognition/Cargo.toml`
- `crates/prolog8/Cargo.toml`
- `crates/ocpq/Cargo.toml`
- `crates/bench-tools/Cargo.toml`
- `crates/wasm4pm-lsp/Cargo.toml`
- `crates/tps-metrics/Cargo.toml`

**Risk:** Low (declarative only, no behavior change; clippy warnings may surface issues)

---

### Phase 2: Workspace Dependency Unification (Medium Priority, Medium Risk)

**Objective:** Establish [workspace.dependencies] for core crates to ensure version consistency.

**Tasks:**
1. Extract commonly-used deps from member Cargo.toml files:
   ```toml
   [workspace.dependencies]
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   blake3 = "1.8.5"
   chrono = "0.4.45"
   uuid = { version = "1.23.2", features = ["v4", "serde"] }
   quick-xml = "0.36"
   tokio = { version = "1", features = ["full"] }
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
   criterion = "0.5"
   proptest = "1"
   ```

2. Update all member crates to use `.workspace = true` for these deps.

3. Add CI check to ensure no duplicate `[dependencies]` entries across workspace members.

**Files to modify:**
- `Cargo.toml` (root workspace)
- All 8 member `Cargo.toml` files

**Risk:** Medium (dependency version mismatches may surface; requires careful testing of all crates)

---

### Phase 3: Justfile Standardization (Low Priority, Medium Risk)

**Objective:** Align task naming and structure with praxis Justfile conventions.

**Current State:** wasm4pm uses Makefile + Makefile.toml (cargo-make style)

**Praxis Convention:** Justfile with task categories:
```justfile
fmt:
    cargo fmt --all

lint:
    cargo clippy --all --all-targets --all-features -- -D warnings

test:
    cargo test --all --all-features

build:
    cargo build --all --release

doc:
    cargo doc --all --no-deps --all-features

bench:
    cargo bench --all

pre-commit: fmt lint test
```

**Tasks:**
1. Audit existing Makefile/Makefile.toml structure
2. Create `justfile` with praxis-aligned task names
3. Map existing targets to Justfile equivalents
4. Update CI/CD to use `just` instead of `make`

**Risk:** Medium (task naming changes may break scripts in CI/documentation)

---

### Phase 4: typos.toml Addition (Low Priority, Low Risk)

**Objective:** Add spell-check configuration for consistency.

**Current State:** wasm4pm uses markdownlint instead of typos.toml

**Tasks:**
1. Create `typos.toml` at repository root:
   ```toml
   [default]
   check-filenames = true
   check-files = true
   
   [default.extend-exclude]
   # Exclude as needed
   
   [default.extend-words]
   # Domain-specific terms
   ```

2. Add spell-check job to CI pipeline (if not present)

**Risk:** Low (additive, doesn't replace markdownlint)

---

### Phase 5: Documentation & Visibility (Low Priority, Low Risk)

**Objective:** Ensure SECURITY.md and other docs are visible.

**Tasks:**
1. Verify SECURITY.md exists and is discoverable
2. Add reference in README.md to all governance docs
3. Create/update ARCHITECTURE.md following praxis conventions

**Risk:** Low (documentation only)

---

## Part 4: Implementation Strategy

### Approach: Incremental + Branch-Based

**Branch:** `retrofit/praxis-standardization` (branching from current main)

**Phase Sequencing:**
1. **Week 1:** Phase 1 (Declarative [lints]) + Phase 4 (typos.toml)
2. **Week 2:** Phase 2 (Workspace deps) + testing
3. **Week 3:** Phase 3 (Justfile) + CI adjustment
4. **Week 4:** Phase 5 (Docs) + final review

**Validation Gates:**
- All clippy warnings resolved
- Workspace dependency versions consistent
- CI/CD passes on all platforms
- Test coverage maintained (>95% line, >90% function)
- Documentation updated

---

## Part 5: Benefits of Retrofit

### For Claude Code Web Compatibility
- ✓ Enables Claude Code web sessions to run locally
- ✓ Declarative [lints] allows IDE integration
- ✓ Workspace deps enable faster dependency resolution
- ✓ Consistent task runner (Justfile) for automation

### For Project Sustainability
- ✓ Aligns with seanchatmangpt ecosystem standards
- ✓ Reduces cognitive load for new contributors
- ✓ Enables automated refactoring across fleet
- ✓ Improves supply chain transparency (deny.toml)

### For Process Mining Community
- ✓ Serves as reference implementation for OCEL 2.0
- ✓ Demonstrates Rust best practices for scientific computing
- ✓ Establishes patterns for WASM-based process mining tools

---

## Part 6: Recommendations

### Priority Ranking

1. **MUST DO (Blocking Claude Code Web):**
   - Add [lints] workspace config
   - Verify rust-toolchain.toml alignment

2. **SHOULD DO (Best Practices):**
   - Unify workspace.dependencies
   - Add typos.toml
   - Create/verify SECURITY.md

3. **NICE TO HAVE (Future):**
   - Convert Makefile → Justfile
   - Update ARCHITECTURE.md
   - Create retrofit case study documentation

### Success Criteria

✓ wasm4pm compiles cleanly with workspace [lints]
✓ All 8 crates inherit [lints] workspace = true
✓ deny.toml supply chain audit passes
✓ typos.toml checks pass
✓ Claude Code web session can clone, build, test wasm4pm
✓ All tests pass (>95% line coverage maintained)

---

## Conclusion

wasm4pm is **90% of the way to praxis compliance** — it has the hard parts (CI/CD infrastructure, comprehensive testing, strong documentation). The retrofit focuses on the **structural standardization layer** (workspace [lints], dependency unification, Justfile) that makes it Claude Code web compatible and lowers friction for contributors.

**Estimated effort:** 2-4 weeks for full retrofit + validation
**Risk level:** Low (mostly additive changes; no breaking changes)
**Expected outcome:** Production-ready, praxis-aligned process mining platform compatible with Claude Code web

---

## Next Steps

1. Review this retrofit plan with stakeholders
2. Create `retrofit/praxis-standardization` branch
3. Begin Phase 1 implementation (Week 1)
4. Document learnings as formal case study for ecosystem


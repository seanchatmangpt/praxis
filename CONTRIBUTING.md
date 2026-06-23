# Contributing to Praxis

Praxis is a house-style standardization kit maintained by the seanchatmangpt community. This guide describes how to contribute improvements, patterns, and fixes to Praxis itself.

## Overview

Praxis is maintained as:
- **Templates** (`template/`, `template-wasm/`, `template-mcp/`, `template-integration/`) — Canonical project scaffolds
- **Shared crate** (`crates/chatman-common/`) — House primitives (error handling, hashing, testing)
- **Tooling** (`apply.sh`, scripts/) — Backfill scripts and validation tools
- **Evidence** (`survey/`) — Fleet analysis and design decisions

## Before You Contribute

1. **Understand the scope:** Praxis standardizes patterns across the seanchatmangpt fleet. Contributions should:
   - Solve problems across **multiple repos**, not single projects
   - Follow **empirical evidence** (survey findings, repeated patterns)
   - Maintain **backward compatibility** with existing projects or clearly document breaking changes

2. **Check the survey:** Before proposing a new pattern, read `survey/00-SYNTHESIS.md` and `survey/01-SECOND-WAVE.md`. Your idea may already be documented or decided.

3. **Consider the fleet:** Changes affect all 18+ repos using Praxis. Review the `CHECKLIST.md` to understand migration scope.

## Types of Contributions

### 1. Bug Fixes

**Submit a PR if:**
- A template is broken (won't compile, CI fails, produces invalid binaries)
- A documented pattern doesn't work as described
- `apply.sh` corrupts or misses files

**Include:**
- Description of the bug
- Minimal reproduction case
- Why it broke (e.g., regression from Rust 1.82 update, typo in template)
- Verification that your fix works

**Example:**
```
Bug: WASM builds fail with "strip = true" in release profile
Root cause: strip=true passes --strip-all to native toolchain; WASM requires wasm-opt
Fix: Set strip = false in [profile.release] (already documented, but not enforced)
```

### 2. Pattern Proposals

**Submit an issue (not a PR) if:**
- You've discovered a pattern across 3+ Praxis repos
- The pattern solves a common problem (validation, error handling, testing, etc.)
- You want fleet-wide feedback before implementation

**Include:**
- **Evidence:** List the repos that use this pattern
- **Justification:** Why does the fleet benefit from standardizing this?
- **Cost:** What effort to implement in all templates?
- **Tradeoffs:** What do we gain vs. lose?

**Example:**
```
Pattern: Seal Pattern for Domain Types
Evidence: Used in wasm4pm, dteam, miniml for immutable objects
Benefit: Compile-time guarantee that critical types pass validation
Cost: 3 lines per sealed type (private _seal field + seal() constructor)
Tradeoff: Slightly more verbose code, but prevents invalid states
```

Once approved, the pattern is:
1. Added to the canonical template
2. Documented in `CLAUDE.md` with examples
3. Added to `survey/01-SECOND-WAVE.md`
4. `apply.sh` is updated if it applies to existing repos

### 3. Documentation Improvements

**Submit a PR if:**
- Docs are unclear, outdated, or incomplete
- Missing examples or use cases
- Typos or grammatical issues
- New feature needs user-facing documentation

**Include:**
- What was confusing or missing?
- How does your change clarify it?
- Is this a docs-only PR or does code change too?

**Example:**
```
Improvement: Add example of using ErrorPolicy trait in error.rs
Currently: Only brief doc comment and tests
Added: Complete example showing custom error policies for a hypothetical service
```

### 4. Performance Improvements

**Submit a PR if:**
- You've measured a concrete performance problem
- Your fix is isolated and doesn't introduce complexity
- Improvement is > 10% or addresses a known bottleneck (see agent findings)

**Include:**
- Benchmark (before/after numbers)
- Explanation of what was slow and why
- Any API changes or tradeoffs

**Example:**
```
Performance: Cache JSON introspection in CLI
Before: collect_tools_from_cmd() walks tree on every --introspect call
After: Cache tools vector in Arc, lazily initialized
Improvement: 2-3x faster for large command trees
No API changes: Caller code unchanged
```

### 5. Dependency Updates

**Submit a PR if:**
- A security advisory requires an update
- A dependency has a critical bug
- We need to bump MSRV to enable a better pattern

**Include:**
- Which crate, which version?
- Why is the update needed?
- Does it break anything? (Run CI locally: `just ci`)
- Any MSRV implications?

**Example:**
```
Update: blake3 1.8.4 → 1.9.0
Reason: Security advisory in streaming hash (we don't use streaming, but let's update)
MSRV: Unchanged (1.82)
Breaking: No
```

### 6. Testkit Enrichment

**Submit a PR if:**
- You want to add new testing utilities to `chatman-common/testkit`
- The utilities are broadly useful across the fleet (not single-project)
- They're battle-tested in at least one project first

**Include:**
- What problem does this solve?
- Which repos are using this pattern (or could benefit)?
- Example usage in `crates/chatman-common/src/testkit.rs` with tests

**Example:**
```
New testkit: TestState<Phase> compile-time AAA enforcement
Problem: Tests sometimes forget to set up state correctly (wrong phase)
Repos: Used in wasm4pm for phase-gated features
Pattern: Struct holds state, transitions only via impl methods (type-state)
Tests: Compile-fail tests ensuring wrong phases don't compile
```

## Development Workflow

### Local Setup

```bash
# Clone Praxis
git clone https://github.com/seanchatmangpt/praxis
cd praxis

# Install tools
cargo install cargo-generate cargo-deny typos-cli just

# Run CI gate locally
just ci
```

### Making Changes

1. **Create a branch:**
   ```bash
   git checkout -b feat/my-change
   ```

2. **Make changes and test:**
   ```bash
   # Edit files
   cargo build
   cargo test
   
   # Run full CI gate
   just ci
   ```

3. **If adding a new template variant:**
   - Duplicate appropriate base template: `cp -r template template-myvariant`
   - Update `Cargo.toml` in new template for domain-specific features
   - Add `CLAUDE.md` with variant-specific guidance
   - Update root `README.md` to list the new template

4. **If modifying `apply.sh`:**
   - Test on a real repo: `./apply.sh /path/to/repo --dry-run`
   - Ensure idempotence: `./apply.sh /path/to/repo && ./apply.sh /path/to/repo` should succeed both times
   - Add test case to `scripts/test-apply.sh` if it exists

5. **If modifying CI/CD:**
   - Update both `.github/workflows/` in templates
   - Document any new secrets or environment variables needed
   - Test locally: `act -j <job-name>` (if `act` is installed)

### Commit Style

Follow conventional commits:
```
type(scope): description

Optional longer explanation. Keep under 72 chars per line.

Fixes #123 (if applicable)
```

**Types:**
- `feat` — New feature (new template, new pattern)
- `fix` — Bug fix
- `docs` — Documentation only
- `perf` — Performance improvement
- `refactor` — Code reorganization (no behavior change)
- `test` — Test additions/fixes
- `chore` — Maintenance (dependency updates, tooling)

**Scopes:**
- `template` — Main template
- `template-wasm` — WASM template
- `template-mcp` — MCP template
- `chatman-common` — Shared crate
- `docs` — Documentation files
- `ci` — CI/CD workflows
- `apply` — apply.sh and backfill tools

**Examples:**
```
feat(template): add ValidatedInput<T, V> pattern
fix(template-wasm): set strip = false in release profile
docs(README): clarify template selection decision tree
perf(cli): cache tool definitions for introspection
refactor(error): consolidate error handling into ErrorPolicy trait
```

### Submitting a PR

1. **Push your branch:**
   ```bash
   git push origin feat/my-change
   ```

2. **Open a PR on GitHub:**
   - Title: Short, clear, starts with verb
   - Description: Why? What changed? Testing?
   - Link related issues: "Fixes #123" or "Addresses agent findings on X"

3. **Ensure CI passes:**
   - All checks green: formatting, linting, tests, deny, typos
   - If a check fails, fix locally and push again (don't amend)

4. **Respond to review:**
   - Be responsive to feedback
   - Ask questions if suggestions are unclear
   - Don't take criticism personally; it's about code quality

### Merging

Once approved and CI passes:
1. Use "Squash and merge" if your commits are WIP-ish
2. Use "Create a merge commit" if commits are logical and self-contained
3. Ensure commit message follows conventions

## Testing Your Changes

### Templates

Test a template works end-to-end:
```bash
# Generate from template
cargo generate --git /path/to/praxis template --name test-project
cd test-project

# Run CI gate
just ci

# Compile and run
cargo build
cargo test
```

### apply.sh

Test on a real project:
```bash
cd /path/to/existing-rust-project

# Dry run (no changes)
/path/to/praxis/apply.sh . --dry-run

# Check what changed
git status

# Actually apply
/path/to/praxis/apply.sh .

# Verify no corruption
just ci
cargo build
```

### chatman-common

Changes are tested by dependent projects:
```bash
cd crates/chatman-common
cargo test --all-features

# Also test in a consuming template
cd template
cargo test
```

## Standards & Style

### Code

- Rust code follows standard idioms (see `template/CLAUDE.md`)
- Public APIs have rustdoc with examples
- Tests for all public items and error paths
- No `unwrap` in library code; use `?` and `Result`
- Forbid unsafe code (except linkme/WASM with justification)
- No `todo!` / `unimplemented!` in shipping code

### Documentation

- README.md for each template or major component
- CLAUDE.md in every template (developer guide)
- DESIGN.md for complex modules (like `chatman-common`)
- Inline code comments only for **why**, not **what**
- Commit messages reference issue numbers

### Naming

- Features: lowercase, hyphens (`phase-1`, `mcp`, `otel`)
- Module names: snake_case (`error.rs`, `cli.rs`)
- Public types: PascalCase (`ValidatedInput`, `VerbRegistry`)
- FM codes: `FM-DOMAIN-NNN` (e.g., `FM-CLI-001`)

## Maintenance & Releases

### Version Bumps

Praxis uses **CalVer**: `YY.M.patch`
- `26.6.0` = June 2026, patch 0
- `26.7.0` = July 2026 (monthly bump)
- `26.6.1` = Second release in June (patch increment)

**Bump triggers:**
- Security fix → patch
- New template or major feature → minor (next month)
- Breaking change → minor (next month, documented in CHANGELOG)

### Release Checklist

1. Update version in all `Cargo.toml` files
2. Update `CHANGELOG.md`: move `[Unreleased]` to `[YY.M.patch]`
3. Commit: `chore(release): vYY.M.patch`
4. Tag: `git tag vYY.M.patch`
5. Push: `git push origin main && git push origin vYY.M.patch`
6. GitHub Release: Copy changelog entry, attach artifacts if any

## Code of Conduct

- Be respectful and constructive
- Assume good intent
- Address concerns privately before escalating
- Disagree on ideas, not people

## Questions?

- **Docs:** See `docs/getting-started.md`, `docs/faq.md`, `docs/troubleshooting.md`
- **Architecture:** Read `survey/00-SYNTHESIS.md`
- **Patterns:** See `template/CLAUDE.md`
- **Issues:** File on GitHub with context

---

**Thank you for contributing to Praxis!** Your improvements help the entire fleet.


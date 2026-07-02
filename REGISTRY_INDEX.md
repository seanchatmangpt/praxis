# Repository Registry — Complete Index

This document is your entry point to the repository discovery and management system for the seanchatmangpt ecosystem.

---

## What Is This?

A complete system for managing the retrofit of 18 Rust repositories toward praxis house standards. Includes:

- **Central TOML registry** (`repos.toml`) — Metadata for all 18 repos
- **Rust module** (`repo_registry.rs`) — Query, filter, sort APIs
- **Comprehensive documentation** — Format specs, usage guides, examples
- **Reporting & analytics** — Readiness summaries, effort estimates, dependency tracking

**Total scope:** 18 repos, 124 crates, ~59 person-weeks effort (1.5 months at full-time)

---

## Start Here

### I'm new to this system
→ Read **`REGISTRY_SUMMARY.txt`** (2–3 minutes)

### I need to plan retrofit work
→ Read **`ECOSYSTEM_DISCOVERY.md`** §4 (Retrofit Planning, 10 minutes)

### I'm implementing the registry
→ Read **`REPO_REGISTRY.md`** (Format, API, Examples, 20 minutes)

### I want to use the Rust module
→ See **`crates/praxis-retrofit/examples/registry_demo.rs`** (runnable example)

### I need all the details
→ Read **`DELIVERABLES.md`** (complete reference, 15 minutes)

---

## Key Files

### Registry
- **`repos.toml`** — Central TOML file with all 18 repos + metadata
  - 18 `[repos.<slug>]` sections
  - `[metadata]` section with ecosystem stats & lists
  - All required fields: URL, crate name, phase, risk, priority, notes

### Documentation (in order of detail level)

| File | Purpose | Length | Audience |
|------|---------|--------|----------|
| `REGISTRY_SUMMARY.txt` | One-page quick ref | ~200 lines | Everyone |
| `ECOSYSTEM_DISCOVERY.md` | Usage guide + scenarios | ~450 lines | Engineers, PMs |
| `REPO_REGISTRY.md` | Format spec + API ref | ~550 lines | Developers |
| `DELIVERABLES.md` | Complete overview | ~350 lines | Project leads |

### Rust Module
- **`crates/praxis-retrofit/src/repo_registry.rs`** — Main module (650+ lines)
  - `RepositoryEntry` — Single repo
  - `RepositoryRegistry` — Query engine
  - `EcosystemMetadata` — Ecosystem stats
  - 18+ methods for filtering/sorting/reporting

- **`crates/praxis-retrofit/examples/registry_demo.rs`** — Runnable demo (80 lines)
  - Shows all major queries
  - Outputs example reports

- **`crates/praxis-retrofit/src/lib.rs`** — Module exports
  - Added: `pub mod repo_registry;`
  - Re-exports: `RepositoryEntry`, `RepositoryRegistry`, `EcosystemMetadata`

---

## Quick Navigation

### By Role

**Project Manager/Stakeholder:**
1. `REGISTRY_SUMMARY.txt` § Retrofit Status Summary
2. `REGISTRY_SUMMARY.txt` § Recommended Retrofit Order
3. `ECOSYSTEM_DISCOVERY.md` § Common Scenarios → Scenario G (reporting)

**Engineer (Retrofit Work):**
1. `ECOSYSTEM_DISCOVERY.md` § Retrofit Planning (risk/effort/order)
2. `REGISTRY_SUMMARY.txt` § Top 5 by Priority / Recommended Order
3. `REPO_REGISTRY.md` § Filtering Examples
4. Start with Primary tier repos in recommended order

**Architect (Design):**
1. `ECOSYSTEM_DISCOVERY.md` § Architecture
2. `ECOSYSTEM_DISCOVERY.md` § Common Scenarios → Scenario D (dependents)
3. `REGISTRY_SUMMARY.txt` § Legal/Compliance Tracking
4. Check `registry.downstream_consumers(repo)` for dependency planning

**Developer (Using Module):**
1. `crates/praxis-retrofit/examples/registry_demo.rs` (run it!)
2. `REPO_REGISTRY.md` § Rust Module: Main Types & Key Methods
3. `REPO_REGISTRY.md` § Usage Examples (1–6)
4. Reference: `repo_registry.rs` source code

**QA/CI:**
1. `REGISTRY_SUMMARY.txt` § CI Gaps
2. `REPO_REGISTRY.md` § RepositoryRegistry methods: `no_ci_repos()`, `missing_license_files()`
3. Integrate into CI gate: `registry.filter_by_phase(5)` to verify retrofit complete

---

## By Task

### "Plan our retrofit campaign"
1. Read `ECOSYSTEM_DISCOVERY.md` § Retrofit Planning
2. Check `REGISTRY_SUMMARY.txt` § Recommended Retrofit Order
3. Review effort table in `ECOSYSTEM_DISCOVERY.md` to size sprints
4. Use `registry.sorted_by_priority()` to sequence work

### "I'm ready to retrofit a specific repo"
1. Look up repo name in `REGISTRY_SUMMARY.txt`
2. Check `repos.toml [repos.<slug>]` for:
   - Current `retrofit_phase_complete`
   - `retrofit_readiness` (ready or requires-prep?)
   - `notes` (any special instructions?)
3. Reference `REPO_REGISTRY.md` § Retrofit Phases for phase description
4. Execute retrofit work (see `CHECKLIST.md`)
5. Update `retrofit_phase_complete` in repos.toml
6. Commit: `git add repos.toml && git commit -m "docs: <repo> phase N complete"`

### "Get a status report"
```rust
let registry = RepositoryRegistry::load("repos.toml").await?;
println!("{}", registry.readiness_summary());
```
Output includes: ready/prep/blocked counts, risk distribution, total effort

### "Find repos that depend on X"
```rust
let consumers = registry.downstream_consumers("clap-noun-verb");
// → affidavit, cargo-cicd, mac-artifact-cleaner
```

### "Identify compliance gaps"
```rust
let no_ci = registry.no_ci_repos();
let missing_licenses = registry.missing_license_files();
let legal_issues = registry.non_standard_licenses();
```

### "Sort repos by risk (low first, to build confidence)"
```rust
let by_risk = registry.sorted_by_risk();
// → wasm4pm-compat, clnrm, clnrm-prototype, ...
```

### "Find quick wins (easiest repos first)"
```rust
let by_effort = registry.sorted_by_effort();
// → affidavit (0w), clnrm (0.5w), clap-noun-verb (0.375w), ...
```

---

## The 18 Repos (Quick Summary)

**Complete list with phase & priority:**

| # | Repo | Phase | Ready? | Risk | Priority | Crates |
|---|------|-------|--------|------|----------|--------|
| 1 | affidavit | 5 | ✓ | low | 100 | 1 |
| 2 | clap-noun-verb | 4 | ✓ | low | 95 | 3 |
| 3 | ggen | 3 | ✗ | med | 90 | 15 |
| 4 | lsp-max | 2 | ✗ | high | 85 | 34 |
| 5 | cargo-cicd | 3 | ✗ | med | 85 | 3 |
| 6 | clnrm | 4 | ✓ | low | 80 | 4 |
| 7 | wasm4pm-compat | 2 | ✗ | low | 70 | 1 |
| 8 | a2a-rs | 1 | ✗ | high | 75 | 10 |
| 9 | pm4py-rs | 2 | ✗ | med | 65 | 1 |
| 10 | ggen-mcp | 2 | ✗ | med | 70 | 1 |
| 11 | bcinr | 3 | ✗ | med | 75 | 12 |
| 12 | dteam | 2 | ✗ | high | 70 | 12 |
| 13 | swarmsh-v2 | 2 | ✗ | low | 55 | 1 |
| 14 | clnrm-prototype | 4 | ✓ | low | 50 | 1 |
| 15 | pm4wasm | 0 | ✗ | high | 40 | 2 |
| 16 | miniml | 1 | ✗ | high | 35 | 1 |
| 17 | semantic_bit | 0 | ✗ | high | 30 | 1 |
| 18 | mac-artifact-cleaner | 5 | ✓ | low | 50 | 2 | ← FIRST PRAXIS PROJECT (completed 2026-07-01)

Ready: 4, Prep: 13, Phases complete: 1–5 (avg ~2.5)

---

## Key Concepts

### Retrofit Phase (0–5)
Progress tracking for standardization. Each phase adds more praxis hygiene:
- **Phase 1:** Lints (`[workspace.lints]`)
- **Phase 2:** Dependencies + supply chain (`deny.toml`, `typos.toml`)
- **Phase 3:** Task runner (`justfile`), CI workflows
- **Phase 4:** Spell-check, editor config, fix lint issues
- **Phase 5:** Docs (SECURITY.md, CONTRIBUTING.md, CHANGELOG.md, CLAUDE.md)

See `REPO_REGISTRY.md` § Retrofit Phases for detailed descriptions.

### Retrofit Readiness
- **"ready"** — Start now, minimal prerequisites
- **"requires-prep"** — Resolve issues first (e.g., edition split, license)
- **"blocked"** — External decision needed (legal, architecture)

See `REPO_REGISTRY.md` § Readiness Statuses for criteria.

### Risk Level
- **"low"** — Single crate, straightforward (e.g., affidavit)
- **"medium"** — Multi-crate, some quirks (e.g., ggen)
- **"high"** — Large workspace, rare patterns, legal issues (e.g., lsp-max, dteam)

Used to estimate effort (risk = multiplier 1.0× / 1.5× / 2.5×).

### Priority Score (0–100)
Recommended retrofit order based on:
- Adoption (how many repos depend on this?)
- Dependents (is it a foundation?)
- Size (crate count)
- Urgency (does it block others?)

Higher = tackle first.

---

## Maintenance

### After Each Retrofit PR Merges
Update `repos.toml`:
```toml
[repos.my-repo]
retrofit_phase_complete = 2  # increment
# ... commit and push
```

### Adding a New Repo
1. Run a survey (see `survey/` for templates)
2. Add `[repos.<slug>]` section to repos.toml
3. Fill all required fields
4. Add to appropriate list in `[metadata.critical_dependencies]`
5. Commit: `git add repos.toml && git commit -m "docs: add <repo> to registry"`

### Quarterly Review
Re-run ecosystem survey to verify:
- Retrofit phase progress
- Risk/priority accuracy
- Downstream dependency changes
- New repos to add

---

## API Cheat Sheet

```rust
use praxis_retrofit::repo_registry::RepositoryRegistry;

// Load
let registry = RepositoryRegistry::load("repos.toml").await?;

// Query single repo
let repo = registry.get("affidavit")?;
println!("{}: phase {}/5", repo.name, repo.retrofit_phase_complete);

// Filter & count
let ready = registry.filter_by_readiness("ready").count();  // 3
let high_risk = registry.filter_by_risk("high").count();    // 5

// Sort
let by_priority = registry.sorted_by_priority();            // highest first
let by_risk = registry.sorted_by_risk();                    // low→high, then priority
let by_effort = registry.sorted_by_effort();                // easiest first

// Trace dependencies
let consumers = registry.downstream_consumers("ggen");      // 5 repos depend on ggen

// CI/compliance checks
let no_ci = registry.no_ci_repos();                         // 5 repos
let legal = registry.non_standard_licenses();              // AGPL/BSL/Apache-only

// Report
println!("{}", registry.readiness_summary());

// Metadata
println!("House MSRV: {}", registry.metadata.house_msrv);  // 1.82
println!("Total crates: {}", registry.metadata.total_crates);  // 124
```

---

## Files at a Glance

```
praxis/
├── repos.toml                       # 18 repos, metadata, lists
├── REGISTRY_INDEX.md                # This file (navigation)
├── REGISTRY_SUMMARY.txt             # One-page quick ref
├── REPO_REGISTRY.md                 # Format spec + API (550 lines)
├── ECOSYSTEM_DISCOVERY.md           # Usage guide + scenarios (450 lines)
├── DELIVERABLES.md                  # Complete overview (350 lines)
│
├── survey/
│   └── 00-SYNTHESIS.md              # Source data (10-agent survey)
├── CHECKLIST.md                     # Per-repo refactor checklist
├── README.md                        # Praxis overview
│
└── crates/praxis-retrofit/
    ├── src/repo_registry.rs         # Rust module (650+ lines)
    ├── src/lib.rs                   # Module exports
    ├── examples/registry_demo.rs    # Runnable example (80 lines)
    └── Cargo.toml                   # Updated deps if needed
```

---

## Questions?

**Q: How do I start?**  
A: Read `REGISTRY_SUMMARY.txt` (2 min), then refer to "By Task" section above.

**Q: Which repo should I retrofit first?**  
A: Check `REGISTRY_SUMMARY.txt` § Recommended Retrofit Order, Tier 1.

**Q: How long will this take?**  
A: ~59 person-weeks total (~1.5 months full-time). See effort table in `ECOSYSTEM_DISCOVERY.md`.

**Q: How do I use the Rust module?**  
A: Run `cargo run --example registry_demo`, then read `REPO_REGISTRY.md` § Rust Module.

**Q: What if a repo has legal issues (AGPL, BSL)?**  
A: Check `REGISTRY_SUMMARY.txt` § Legal/Compliance Tracking, coordinate externally.

**Q: How do I track progress?**  
A: Update `retrofit_phase_complete` in repos.toml after each PR, commit.

---

## License

Same as praxis: MIT OR Apache-2.0

---

**Last updated:** 2026-06-23  
**Repos:** 18, **Crates:** 124, **Effort:** ~59 person-weeks

# Repository Registry Format & Discovery Module

This document describes the `repos.toml` registry file and the Rust `repo_registry` module for managing the seanchatmangpt ecosystem's 18 repositories.

---

## Overview

The registry enables:
- **Fleet-wide retrofit planning** — centralized metadata for all repos
- **Smart prioritization** — sort by risk, effort, phase completion, and adoption
- **Dependency tracking** — identify upstream/downstream relationships
- **Compliance visibility** — aggregate gaps (CI, licensing, documentation)
- **Readiness assessment** — quick status on retrofit progress

---

## Registry Location & Format

**File:** `repos.toml` (root of praxis project)  
**Format:** TOML (human-readable, version-control friendly)  
**Schema:** Custom two-section structure (repos + metadata)

---

## repos.toml Structure

### 1. Repository Entries

Each repository gets a top-level section under `[repos.<github-slug>]`:

```toml
[repos.affidavit]
github_url = "https://github.com/seanchatmangpt/affidavit"
github_owner = "seanchatmangpt"
local_path = "../affidavit"
crate_name = "affidavit"
description = "Reference sealed-receipt auditing with BLAKE3 provenance..."
visibility = "public"
workspace_type = "single-crate"
crate_count = 1
retrofit_readiness = "ready"
retrofit_phase_complete = 5
risk_level = "low"
priority_score = 100
maintainer_status = "active"
notes = "House reference implementation; praxis-retrofit was built in this session..."
```

#### Field Descriptions

| Field | Type | Values | Meaning |
|-------|------|--------|---------|
| `github_url` | string | HTTPS URL | Canonical GitHub repository location |
| `github_owner` | string | "seanchatmangpt" | Account owner (always the same) |
| `local_path` | string | relative path | Where to find this repo locally (e.g., `../affidavit`) |
| `crate_name` | string | Rust identifier | Primary crate name from `Cargo.toml` |
| `description` | string | 1–3 sentences | What the repo does (used in reports) |
| `visibility` | string | "public" \| "private" | Public or private repository |
| `workspace_type` | string | "single-crate" \| "multi-crate" \| "monorepo" | Structural organization |
| `crate_count` | integer | 1+ | Number of publishable crates |
| `retrofit_readiness` | string | "ready" \| "requires-prep" \| "blocked" | Can it be retrofitted now? |
| `retrofit_phase_complete` | integer | 0–5 | Phases finished (0 = none, 5 = done) |
| `risk_level` | string | "low" \| "medium" \| "high" | Retrofit complexity/impact |
| `priority_score` | integer | 0–100 | Recommended order (higher = do first) |
| `maintainer_status` | string | "active" \| "maintenance" \| "experimental" | How actively is it maintained? |
| `notes` | string | free-form text | Context for retrofit decisions |

### 2. Metadata Section

The `[metadata]` section provides ecosystem-wide context and summary statistics:

```toml
[metadata]
ecosystem_name = "seanchatmangpt"
total_repos = 18
total_crates = 124
survey_date = "2026-06-23"
survey_agents = 10
survey_scope = "17 public repos + 1 private writable (affidavit)"
praxis_version = "26.6.0"
house_msrv = "1.82"
house_edition = "2021"
house_toolchain = "1.82.0 pinned stable"
house_license = "MIT OR Apache-2.0"
house_version_scheme = "CalVer YY.M.patch"
```

#### Retrofit Statistics

```toml
[metadata.retrofit_stats]
ready_for_retrofit = 3
requires_phase_2_or_3 = 8
requires_phase_0_or_1 = 6
experimental_status = 5
```

#### Retrofit Order

```toml
[metadata.critical_dependencies]
primary_order = ["affidavit", "clap-noun-verb", "ggen", "lsp-max", "cargo-cicd", "clnrm"]
secondary_order = ["wasm4pm-compat", "a2a-rs", "pm4py-rs", "ggen-mcp", "bcinr", "dteam"]
tertiary_order = ["swarmsh-v2", "clnrm-prototype", "pm4wasm", "miniml", "semantic_bit", "mac-artifact-cleaner"]
```

The order reflects upstream/downstream dependencies and risk tolerance.

#### Legal Considerations

```toml
[metadata.legal_considerations]
agpl_repos = ["pm4py-rs"]
bsl_repos = ["miniml", "dteam"]
apache_only_repos = ["ggen-mcp", "pm4wasm"]
missing_license_files = ["swarmsh-v2", "a2a-rs", "semantic_bit", "mac-artifact-cleaner"]
```

#### CI & Documentation Gaps

```toml
[metadata.ci_gaps]
no_ci_repos = ["wasm4pm-compat", "pm4wasm", "miniml", "semantic_bit", "mac-artifact-cleaner"]
deprecated_actions_repos = ["a2a-rs", "swarmsh-v2"]
minimal_ci_repos = ["lsp-max", "dteam"]
sparse_release_workflows = 9

[metadata.documentation_gaps]
missing_claude_md = ["clnrm", "clnrm-prototype", "pm4py-rs", "semantic_bit"]
missing_security_md = [...]
missing_changelog = [...]
missing_contributing = [...]
```

---

## Retrofit Phases (0–5)

The `retrofit_phase_complete` field tracks standardization progress:

| Phase | Name | Scope | Effort |
|-------|------|-------|--------|
| 0 | *None* | No retrofit work started | — |
| 1 | `Phase1Lints` | Wire `[workspace.lints]` + `[lints]`, enforce deny-todo/unwrap | ~30 min/crate |
| 2 | `Phase2Deps` | Add `deny.toml`, `typos.toml`, wire `[workspace.dependencies]` | ~45 min/crate |
| 3 | `Phase3Justfile` | Standardize task runner (add `justfile`), update CI workflows | ~1 hour/crate |
| 4 | `Phase4Typos` | Wire spell-check, .editorconfig, `.gitignore`, fix lint issues | ~45 min/crate |
| 5 | `Phase5Docs` | Add/update SECURITY.md, CONTRIBUTING.md, CHANGELOG.md, CLAUDE.md | ~1 hour/crate |

**Note:** Phases are *cumulative*. A repo with `retrofit_phase_complete = 3` has completed phases 1, 2, and 3.

---

## Readiness Statuses

### `retrofit_readiness: "ready"`

**Meaning:** The repo can be retrofitted immediately with minimal surprises.

**Criteria:**
- No blocking legal/license issues
- Metadata (repository URL, keywords) is correct
- No exotic toolchain pins or edition splits

**Examples:** affidavit, clap-noun-verb, clnrm-prototype

### `retrofit_readiness: "requires-prep"`

**Meaning:** Prerequisites must be resolved before retrofit. Common prerequisites:

- Edition split (e.g., a2a-rs: osiris-compiler 2021, rest 2024)
- Thiserror version mismatch (e.g., a2a-rs mixes 1 + 2)
- Wrong/placeholder repository URLs (e.g., affidavit had `anthropics/affidavit`)
- Unresolved license status (e.g., pm4py-rs AGPL conflict)
- Extreme complexity (e.g., lsp-max 34 crates, sprawling CI)

**Examples:** ggen, lsp-max, a2a-rs, pm4py-rs, dteam

### `retrofit_readiness: "blocked"`

**Meaning:** Retrofit cannot proceed without external approval/decision.

**Common blockers:**
- Legal decision on non-standard licenses (AGPL, BSL)
- Architecture decisions needed (nested workspaces, monorepo tooling)
- Upstream dependencies unresolved

**Examples:** (Currently none in the ecosystem, but this status exists for future use)

---

## Risk Levels

| Level | Meaning | Estimate Multiplier | Examples |
|-------|---------|---------------------|----------|
| `"low"` | Single crate, straightforward, high precedent | ×1.0 | affidavit, wasm4pm-compat |
| `"medium"` | Multi-crate, some structural quirks, known pattern | ×1.5 | ggen, pm4py-rs, ggen-mcp |
| `"high"` | Large workspace (10+), rare patterns, legal issues, or minimal CI | ×2.5 | lsp-max, a2a-rs, dteam, pm4wasm |

**Used for:**
- Effort estimation (see `estimated_effort_weeks()` in the module)
- Sorting repos for batch processing
- Risk management (do high-risk repos last, after low-risk patterns are proven)

---

## Priority Score (0–100)

Derived from:
- **Adoption** — How many downstream consumers does this repo have?
- **Dependents** — Is it a foundation crate (ggen, clap-noun-verb)?
- **Size** — How many crates does it contain?
- **Urgency** — Does it block other work?

**Scoring heuristic:**
- 90–100 — Reference implementations or high-adoption foundations (affidavit, clap-noun-verb)
- 75–89 — Major ecosystem consumers (ggen, lsp-max, cargo-cicd)
- 50–74 — Mid-tier repos with some adoption (bcinr, a2a-rs, dteam)
- 30–49 — Experimental or proto repos (pm4wasm, semantic_bit)

---

## Rust Module: `repo_registry`

The `repo_registry` module provides programmatic access to the registry.

### Main Types

#### `RepositoryEntry`

Represents a single repository in the registry.

```rust
pub struct RepositoryEntry {
    pub name: String,
    pub github_url: String,
    pub github_owner: String,
    pub local_path: PathBuf,
    pub crate_name: String,
    pub description: String,
    pub visibility: String,
    pub workspace_type: String,
    pub crate_count: usize,
    pub retrofit_readiness: String,
    pub retrofit_phase_complete: u8,
    pub risk_level: String,
    pub priority_score: u8,
    pub maintainer_status: String,
    pub notes: String,
}
```

**Key Methods:**

| Method | Returns | Purpose |
|--------|---------|---------|
| `retrofit_phase()` | `RetrofitPhase` | Convert phase number to enum |
| `next_phase()` | `Option<RetrofitPhase>` | What phase should be done next? |
| `is_ready_for_retrofit()` | `bool` | Is readiness status "ready"? |
| `requires_prep()` | `bool` | Does it need preparation? |
| `is_blocked()` | `bool` | Is retrofit blocked? |
| `estimated_effort_weeks()` | `f32` | Person-weeks needed to complete |

#### `RepositoryRegistry`

Main registry container.

```rust
pub struct RepositoryRegistry {
    repos: HashMap<String, RepositoryEntry>,
    pub metadata: EcosystemMetadata,
}
```

**Key Methods:**

| Method | Returns | Purpose |
|--------|---------|---------|
| `load(path)` | `Result<Self>` | Load registry from TOML file (async) |
| `all()` | `Vec<&RepositoryEntry>` | Get all repos |
| `get(name)` | `Option<&RepositoryEntry>` | Look up by name |
| `filter_by_readiness(status)` | Iterator | Find repos by retrofit readiness |
| `filter_by_phase(phase)` | Iterator | Find repos by phase completion |
| `filter_by_risk(level)` | Iterator | Find repos by risk level |
| `filter_by_status(status)` | Iterator | Find repos by maintainer status |
| `filter_by_workspace_type(type)` | Iterator | Find repos by workspace structure |
| `sorted_by_priority()` | `Vec<&RepositoryEntry>` | Sort by priority score (descending) |
| `sorted_by_risk()` | `Vec<&RepositoryEntry>` | Sort by risk level (low → high), then priority |
| `sorted_by_effort()` | `Vec<(&RepositoryEntry, f32)>` | Sort by effort (ascending, easy first) |
| `recommended_retrofit_order()` | `Vec<&RepositoryEntry>` | Return repos in metadata-defined order |
| `no_ci_repos()` | `Vec<&RepositoryEntry>` | Get repos with zero CI workflows |
| `missing_license_files()` | `Vec<&RepositoryEntry>` | Get repos missing LICENSE files |
| `non_standard_licenses()` | `Vec<&RepositoryEntry>` | Get repos with AGPL/BSL/Apache-only |
| `downstream_consumers(repo_name)` | `Vec<&RepositoryEntry>` | Find repos that depend on this one |
| `readiness_summary()` | `String` | Generate a summary report |

---

## Usage Examples

### Example 1: Load and List All Repos

```rust
use praxis_retrofit::repo_registry::RepositoryRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let registry = RepositoryRegistry::load("repos.toml").await?;
    
    println!("Total repos: {}", registry.metadata.total_repos);
    for repo in registry.all() {
        println!("  - {} ({})", repo.name, repo.crate_name);
    }
    
    Ok(())
}
```

### Example 2: Filter Repos Ready for Retrofit

```rust
let ready: Vec<_> = registry.filter_by_readiness("ready").collect();
println!("Ready for retrofit: {}", ready.len());

for repo in ready {
    println!("  - {} (phase {}/5)", repo.name, repo.retrofit_phase_complete);
}
```

### Example 3: Plan Retrofit by Priority

```rust
let by_priority = registry.sorted_by_priority();
println!("Retrofit order by priority:");
for (i, repo) in by_priority.iter().enumerate() {
    let effort = repo.estimated_effort_weeks();
    println!("{:2}. {} (priority {}, ~{:.1}w effort)", 
             i + 1, repo.name, repo.priority_score, effort);
}
```

### Example 4: Identify CI Gaps

```rust
let no_ci = registry.no_ci_repos();
println!("Repos with no CI: {}", no_ci.len());
for repo in no_ci {
    println!("  - {} (add ci.yml as phase 2)", repo.name);
}
```

### Example 5: Track Down-stream Dependents

```rust
let consumers = registry.downstream_consumers("clap-noun-verb");
println!("Repos consuming clap-noun-verb:");
for repo in consumers {
    println!("  - {}", repo.name);
}
```

### Example 6: Generate Readiness Report

```rust
println!("{}", registry.readiness_summary());
```

**Output:**
```
Seanchatmangpt Retrofit Readiness Report
========================================
Ready for retrofit: 3 / 18
Requires preparation: 12
Blocked: 0

Risk Distribution:
Low risk: 5
Medium risk: 8
High risk: 5

Estimated Total Effort: 127.5 person-weeks
```

---

## Ecosystem Structure (18 Repos)

### Phase 0 (Reference)
- **affidavit** — Worked example (retrofit complete)
- **clap-noun-verb** — Foundational CLI crate (retrofit complete)

### Phase 1–2 (Foundational)
- **ggen** — RDF/ontology codegen (upstream to 5 repos)
- **lsp-max** — LSP server (largest workspace, 34 crates)
- **cargo-cicd** — CI/CD engine
- **clnrm** — Conformance checker (built on ggen)

### Phase 2–3 (Mid-tier)
- **wasm4pm-compat** — WASM compatibility (ggen downstream, zero CI)
- **a2a-rs** — Automation orchestration (edition split, complex)
- **pm4py-rs** — Process mining (AGPL license, MSRV contradiction)
- **ggen-mcp** — MCP server (2024 edition, Apache-only)

### Phase 3–4 (Established)
- **bcinr** — Bit-chain invariant reasoning (12 crates, hardened release)
- **dteam** — Digital team provisioning (nested workspaces, BSL license)
- **swarmsh-v2** — Distributed shell (deprecated CI, maintenance mode)

### Phase 4–5 (Experimental)
- **clnrm-prototype** — Conformance proto (template repo, experimental)
- **pm4wasm** — WASM process mining (zero CI, monorepo, 2 crates)
- **miniml** — Minimal markup (pnpm+turbo, BSL, no Rust CI)
- **semantic_bit** — Semantic packing (bare minimum, 2024 edition)
- **mac-artifact-cleaner** — Utility (clap-noun-verb consumer, lightweight)

---

## Adding New Repos

To add a new repository to the registry:

1. Run a survey on the repo (see `praxis/survey/` for agent templates)
2. Create a new `[repos.<slug>]` section in `repos.toml`
3. Fill in all required fields (see Field Descriptions above)
4. Assign `retrofit_readiness`, `risk_level`, and `priority_score` based on:
   - How many crates it contains
   - Whether it has documented/undocumented gaps
   - How many downstream repos depend on it
   - Whether it has unusual patterns (non-standard licenses, nested workspaces, etc.)
5. Add the repo name to the appropriate list in `[metadata.critical_dependencies]`
6. Update `[metadata]` totals
7. Commit and push

---

## Maintenance

### When to Update

- **After each repo retrofit PR merges** — increment `retrofit_phase_complete`
- **After adding a new repo** — run a survey and add a section
- **After resolving legal issues** — move repos out of legal consideration lists
- **Quarterly** — re-run ecosystem survey to verify statistics

### Validation

The registry is validated when loaded. If parsing fails:

```
RetrofitError::ParseError("Failed to parse repos.toml: ...")
```

Check the TOML syntax (use an online validator or `toml-cli`):

```bash
toml-cli validate repos.toml
```

---

## See Also

- **Retrofit Phases:** `../README.md` (§5, MANIFEST)
- **Survey Methodology:** `../survey/00-SYNTHESIS.md` (§1, Coverage Matrix)
- **Affidavit Reference:** `../CHECKLIST.md`
- **Praxis Retrofit Module:** `../crates/praxis-retrofit/src/repo_registry.rs`

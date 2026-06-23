# Ecosystem Discovery & Fleet Management

This document describes how to use the repository discovery module to manage the seanchatmangpt ecosystem's 18 repositories for praxis retrofit.

---

## Quick Start

### Load the Registry

```rust
use praxis_retrofit::repo_registry::RepositoryRegistry;

let registry = RepositoryRegistry::load("repos.toml").await?;
```

### Query Common Patterns

```rust
// Find repos ready for retrofit
let ready = registry.filter_by_readiness("ready");

// Sort by priority (highest first)
let by_priority = registry.sorted_by_priority();

// Identify CI gaps
let no_ci = registry.no_ci_repos();

// Find downstream dependents
let consumers = registry.downstream_consumers("clap-noun-verb");

// Get a summary report
println!("{}", registry.readiness_summary());
```

---

## Architecture

### Files

```
praxis/
├── repos.toml                       # Central registry (18 repos metadata)
├── REPO_REGISTRY.md                 # Registry format documentation
├── ECOSYSTEM_DISCOVERY.md           # This file (usage guide)
├── crates/praxis-retrofit/
│   ├── src/repo_registry.rs         # Rust module
│   └── examples/registry_demo.rs    # Demo showing all queries
└── survey/00-SYNTHESIS.md           # Source data (10-agent survey)
```

### Module Structure

```
praxis_retrofit::repo_registry
├── RepositoryEntry          # Single repo metadata
├── RepositoryRegistry       # Main registry container
└── EcosystemMetadata        # Ecosystem-wide stats & config
```

---

## The 18 Repositories

### Organization by Retrofit Phase

#### Phase 0 (Reference Implementations)
- **affidavit** (CalVer 26.6.17, BLAKE3 provenance, reference refactor)
- **clap-noun-verb** (CalVer 26.6.14, foundational CLI pattern, 3 crates)

#### Phase 1–2 (Foundational Consumers)
- **ggen** (CalVer 26.6.DD, RDF/SPARQL codegen, 15 crates, upstream to 5 repos)
- **lsp-max** (CalVer 26.6.18, LSP + OpenTelemetry, **34 crates**, largest workspace)
- **cargo-cicd** (CalVer 26.6.2, CI/CD engine, 3 crates)
- **clnrm** (CalVer 26.5.28, conformance checker, 4 crates, built on ggen)

#### Phase 2–3 (Growing Ecosystem)
- **wasm4pm-compat** (CalVer 26.6.14, WASM compatibility, **zero CI**, needs ci.yml)
- **a2a-rs** (SemVer 0.1.0, automation, 10 crates, **edition split**, complex)
- **pm4py-rs** (CalVer 2026.3.28, process mining, **AGPL license**, MSRV contradiction)
- **ggen-mcp** (SemVer 1.0.0, MCP server, **2024 edition**, Apache-only)

#### Phase 3–4 (Established)
- **bcinr** (CalVer 26.6.x, bit-chain reasoning, 12 crates, hardened release)
- **dteam** (SemVer 1.3.0, team provisioning, 12+ crates, **nested workspaces**, BSL)
- **swarmsh-v2** (SemVer 2.1.0, distributed shell, maintenance mode, deprecated CI)

#### Phase 4–5 (Experimental)
- **clnrm-prototype** (SemVer 0.2.0, conformance proto, **template reference**)
- **pm4wasm** (SemVer 0.1.0, WASM process mining, **zero CI**, monorepo)
- **miniml** (CalVer 26.4.8, minimal markup, **pnpm+turbo**, BSL, no Rust CI)
- **semantic_bit** (SemVer 0.1.0, semantic packing, **bare minimum**, 2024 edition)
- **mac-artifact-cleaner** (SemVer 0.1.0, utility, 2 crates, clap-noun-verb consumer)

---

## Retrofit Planning

### Recommended Order

**Primary (do first, upstream/foundation):**
1. affidavit
2. clap-noun-verb
3. ggen
4. lsp-max
5. cargo-cicd
6. clnrm

**Secondary (mid-tier, depend on primary):**
7. wasm4pm-compat
8. a2a-rs
9. pm4py-rs
10. ggen-mcp
11. bcinr
12. dteam

**Tertiary (experimental/proto, defer or run in parallel):**
13. swarmsh-v2
14. clnrm-prototype
15. pm4wasm
16. miniml
17. semantic_bit
18. mac-artifact-cleaner

### Risk-Based Ordering

**Low risk first (proven patterns):**
- affidavit (worked example)
- clap-noun-verb (high adoption)
- clnrm-prototype (simple template)
- mac-artifact-cleaner (lightweight)
- wasm4pm-compat (single-crate, straightforward)

**Medium risk (standard patterns, some complexity):**
- ggen (multi-crate but well-structured)
- clnrm (multi-crate, clear boundaries)
- cargo-cicd (3 crates, high-quality CI)
- bcinr (12 crates, hardened release)
- pm4py-rs (MSRV contradiction to resolve)
- ggen-mcp (modern but single-crate)
- swarmsh-v2 (small, but deprecated CI)

**High risk (defer, batch carefully):**
- lsp-max (34 crates, huge, complex CI revamp)
- a2a-rs (edition split, thiserror 1+2 mix)
- dteam (nested workspaces, BSL license)
- pm4wasm (zero CI, multi-crate, exotic)
- miniml (non-Rust task runner, BSL)
- semantic_bit (bare minimum, needs architecture decisions)

### Effort Estimation

Expected effort per repo (person-weeks) = `crate_count × phases_remaining × risk_multiplier`

| Repo | Crates | Phases Left | Risk | Est. Effort |
|------|--------|-------------|------|-------------|
| affidavit | 1 | 0 | low | 0 (done) |
| clap-noun-verb | 3 | 1 | low | 0.375w |
| wasm4pm-compat | 1 | 3 | low | 1.1w |
| mac-artifact-cleaner | 2 | 4 | low | 2.0w |
| ggen | 15 | 2 | medium | 4.5w |
| clnrm | 4 | 1 | low | 0.5w |
| bcinr | 12 | 2 | medium | 4.5w |
| a2a-rs | 10 | 4 | high | 7.5w |
| pm4py-rs | 1 | 3 | medium | 1.65w |
| ggen-mcp | 1 | 3 | medium | 1.65w |
| cargo-cicd | 3 | 2 | medium | 1.35w |
| swarmsh-v2 | 1 | 3 | low | 1.1w |
| lsp-max | 34 | 3 | high | 12.75w |
| dteam | 12 | 3 | high | 9.0w |
| clnrm-prototype | 1 | 1 | low | 0.375w |
| pm4wasm | 2 | 5 | high | 3.75w |
| miniml | 1 | 4 | high | 3.0w |
| semantic_bit | 1 | 5 | high | 3.75w |
| **TOTAL** | **124** | — | — | **~59w** (1.5 person-months) |

---

## Common Retrofit Scenarios

### Scenario A: "I want to retrofit the most impactful repos"

**Use:** Recommended retrofit order (metadata.primary_order)

```rust
let order = registry.recommended_retrofit_order();
// → affidavit, clap-noun-verb, ggen, lsp-max, cargo-cicd, clnrm, ...
```

### Scenario B: "I want to start with low-risk repos to build confidence"

**Use:** Sort by risk (low → high), then priority

```rust
let by_risk = registry.sorted_by_risk();
// → wasm4pm-compat, clnrm, mac-artifact-cleaner, ...
```

### Scenario C: "I want to tackle quick wins first"

**Use:** Sort by effort (ascending)

```rust
let by_effort = registry.sorted_by_effort();
// → affidavit (0w), clnrm (0.5w), clap-noun-verb (0.375w), ...
```

### Scenario D: "Find repos blocking others (upstream)"

**Use:** Downstream consumers

```rust
let ggen_consumers = registry.downstream_consumers("ggen");
// → clnrm, wasm4pm-compat, pm4py-rs, ggen-mcp, a2a-rs
// → retrofit ggen first, then these 5

let cnv_consumers = registry.downstream_consumers("clap-noun-verb");
// → affidavit, cargo-cicd, mac-artifact-cleaner
```

### Scenario E: "What needs CI added?"

**Use:** no_ci_repos()

```rust
let no_ci = registry.no_ci_repos();
for repo in no_ci {
    println!("Add ci.yml: {}", repo.name);
}
// → wasm4pm-compat, pm4wasm, miniml, semantic_bit, mac-artifact-cleaner
```

### Scenario F: "Legal coordination needed for which repos?"

**Use:** non_standard_licenses()

```rust
let legal = registry.non_standard_licenses();
// → pm4py-rs (AGPL), miniml (BSL), dteam (BSL), ggen-mcp (Apache-only), pm4wasm (Apache-only)
```

### Scenario G: "Report to stakeholders"

**Use:** readiness_summary()

```rust
println!("{}", registry.readiness_summary());
```

Output:
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

Estimated Total Effort: 59.0 person-weeks
```

---

## Filtering Examples

```rust
// All single-crate repos
let singles = registry.filter_by_workspace_type("single-crate");

// All repos needing phase 1 lints
let phase_1 = registry.filter_by_phase(1);

// All experimental repos
let experimental = registry.filter_by_status("experimental");

// All high-risk repos ready for retrofit
let ready_high_risk: Vec<_> = registry
    .filter_by_readiness("ready")
    .filter(|r| r.risk_level == "high")
    .collect();
```

---

## Integration with Retrofit Workflows

### Phase 0: Preparation (Prerequisites)

Before retrofitting a repo, check:

```rust
let repo = registry.get("a2a-rs").unwrap();
println!("Readiness: {}", repo.retrofit_readiness);

if repo.requires_prep() {
    println!("Resolve prerequisites first:");
    println!("  {}", repo.notes);
}
```

### Phase 1–5: Sequential Retrofit

```rust
while repo.retrofit_phase_complete < 5 {
    let next_phase = repo.next_phase();
    println!("Next work: {:?}", next_phase);
    // ... perform retrofit ...
    // registry.update_phase(repo.name, repo.retrofit_phase_complete + 1);
}
```

### CI Gate: Pre-Merge Compliance

```rust
if repo.retrofit_phase_complete >= 5 && repo.is_ready_for_retrofit() {
    println!("✓ Retrofit complete, ready for merge");
} else {
    println!("✗ Still work needed (phase {}/5)", repo.retrofit_phase_complete);
}
```

---

## Maintenance

### Update Registry After Retrofit PR

When a repo's retrofit PR merges, increment `retrofit_phase_complete`:

```bash
# Example: affidavit completes phase 2
# Edit repos.toml:
[repos.affidavit]
retrofit_phase_complete = 2  # was 1
```

### Add New Repo

1. Survey the repo (see `survey/` for templates)
2. Add `[repos.<slug>]` section with all required fields
3. Assign risk/priority based on size, patterns, adoption
4. Update metadata totals and order lists
5. Commit: `git add repos.toml && git commit -m "docs: add <repo> to ecosystem registry"`

### Periodic Sync

Run quarterly to verify accuracy:

```bash
# For each repo, check:
# - Does retrofit_phase_complete match actual state?
# - Has risk/priority changed?
# - Any new downstream consumers to track?
```

---

## API Reference

### RepositoryEntry

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
    pub retrofit_readiness: String,      // "ready" | "requires-prep" | "blocked"
    pub retrofit_phase_complete: u8,     // 0..=5
    pub risk_level: String,              // "low" | "medium" | "high"
    pub priority_score: u8,              // 0..=100
    pub maintainer_status: String,       // "active" | "maintenance" | "experimental"
    pub notes: String,
}
```

**Key Methods:**
- `retrofit_phase() -> RetrofitPhase` — Get phase as enum
- `next_phase() -> Option<RetrofitPhase>` — What to do next?
- `is_ready_for_retrofit() -> bool`
- `requires_prep() -> bool`
- `is_blocked() -> bool`
- `estimated_effort_weeks() -> f32` — Person-weeks remaining

### RepositoryRegistry

```rust
impl RepositoryRegistry {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self>;
    pub fn all(&self) -> Vec<&RepositoryEntry>;
    pub fn get(&self, name: &str) -> Option<&RepositoryEntry>;
    pub fn filter_by_readiness(&self, status: &str) -> impl Iterator;
    pub fn filter_by_phase(&self, phase: u8) -> impl Iterator;
    pub fn filter_by_risk(&self, level: &str) -> impl Iterator;
    pub fn filter_by_status(&self, status: &str) -> impl Iterator;
    pub fn filter_by_workspace_type(&self, wtype: &str) -> impl Iterator;
    pub fn sorted_by_priority(&self) -> Vec<&RepositoryEntry>;
    pub fn sorted_by_risk(&self) -> Vec<&RepositoryEntry>;
    pub fn sorted_by_effort(&self) -> Vec<(&RepositoryEntry, f32)>;
    pub fn recommended_retrofit_order(&self) -> Vec<&RepositoryEntry>;
    pub fn no_ci_repos(&self) -> Vec<&RepositoryEntry>;
    pub fn missing_license_files(&self) -> Vec<&RepositoryEntry>;
    pub fn non_standard_licenses(&self) -> Vec<&RepositoryEntry>;
    pub fn downstream_consumers(&self, repo_name: &str) -> Vec<&RepositoryEntry>;
    pub fn readiness_summary(&self) -> String;
}
```

---

## See Also

- **Registry Format:** `REPO_REGISTRY.md`
- **Survey Data:** `survey/00-SYNTHESIS.md`
- **Praxis Standards:** `README.md` (§5, MANIFEST and §6, House Defaults)
- **Retrofit Checklist:** `CHECKLIST.md`
- **Example Code:** `crates/praxis-retrofit/examples/registry_demo.rs`

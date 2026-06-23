# praxis-retrofit

**Automate standardization across the Rust ecosystem.**

Apply [praxis](https://github.com/seanchatmangpt/praxis) house-style standards to existing repositories via a scalable, parallel retrofit platform.

## Purpose

The `seanchatmangpt/praxis` repository defines a standardization kit for Rust projects:

- **Workspace [lints]** configuration (unsafe_code, clippy rules)
- **Dependency unification** via [workspace.dependencies]
- **Justfile** task runner with praxis conventions
- **typos.toml** spell-check configuration
- **CI/CD pipelines** (GitHub Actions)
- **Documentation** standards (CONTRIBUTING.md, SECURITY.md, etc.)
- **Licensing** (dual MIT/Apache-2.0)

**Problem:** Existing projects (like wasm4pm) predate praxis and lack these standardizations, making them incompatible with Claude Code web and inconsistent with the house style.

**Solution:** `praxis-retrofit` automates retrofitting any Rust repository to meet praxis standards.

## Architecture

### Five Retrofit Phases

1. **Phase 1: Declare [lints]** — Add workspace linting configuration (HIGHEST priority, LOW risk)
2. **Phase 2: Unify Dependencies** — Extract common deps to [workspace.dependencies]
3. **Phase 3: Justfile** — Standardize task runner (convert from Makefile if needed)
4. **Phase 4: typos.toml** — Add spell-check configuration
5. **Phase 5: Documentation** — Ensure SECURITY.md, ARCHITECTURE.md, etc.

### Four Core Modules

- **`audit`** — Scan repositories for compliance gaps
- **`generate`** — Create retrofit plans and artifacts (templates)
- **`apply`** — Apply retrofit changes to disk
- **`validate`** — Verify compliance post-retrofit

### CLI Pattern: Noun-Verb

Uses [`clap-noun-verb`](https://github.com/seanchatmangpt/clap-noun-verb) for structured commands:

```
praxis-retrofit audit <scan|report> <repo-path>
praxis-retrofit apply <retrofit|validate> <repo-path>
praxis-retrofit generate <templates|plan> [repo-path]
praxis-retrofit validate <compliance|gates> <repo-path>
```

## Installation

```bash
cargo install praxis-retrofit
```

Or build from source:

```bash
cargo build --release --bin praxis-retrofit
./target/release/praxis-retrofit --version
```

## Usage

### Audit a Repository

```bash
# Quick scan: lists compliance status per file
praxis-retrofit audit scan /path/to/wasm4pm

# Detailed report: structured JSON output
praxis-retrofit audit report /path/to/wasm4pm | jq .
```

**Output:** Compliance report with score (%), pass/warn/fail status, and remediation hints.

### Generate Retrofit Plan

```bash
# Generate Phase 1 retrofit for a repository
praxis-retrofit generate plan /path/to/wasm4pm

# Print all templates to stdout
praxis-retrofit generate templates
```

**Output:** Structured retrofit plan (JSON) with:
- Actions (create/update/delete files)
- Commit message
- Estimated risk level
- Phase assignment

### Apply Retrofit

```bash
# Apply Phase 1 retrofit to a repository
praxis-retrofit apply retrofit /path/to/wasm4pm

# Validate that retrofit succeeded
praxis-retrofit apply validate /path/to/wasm4pm
```

**Output:** List of files created/updated, validation status.

### Validate Compliance

```bash
# Full CI compliance gate (runs in GitHub Actions)
praxis-retrofit validate compliance /path/to/wasm4pm
```

**Output:** Pass/fail, blocking issues, remediation steps.

## Retrofit Case Study: wasm4pm

The [`/home/user/praxis/case-study-wasm4pm-retrofit.md`](../case-study-wasm4pm-retrofit.md) document provides a complete audit and retrofit roadmap for the wasm4pm repository, including:

- **Current compliance:** 7.5/10 (strong on CI/docs, weak on [lints]/deps)
- **Gap analysis:** What's missing for Claude Code web compatibility
- **5-phase retrofit roadmap:** Phased implementation plan with risk assessment
- **Success criteria:** How to validate post-retrofit

## Example: Retrofitting wasm4pm

```bash
# 1. Audit compliance
praxis-retrofit audit report /path/to/wasm4pm

# Output:
# Repository: wasm4pm
# Score: 75.0%
# Compliant: false
#   CI/CD Pipeline (ci-cd): Pass
#   Supply Chain Audit (supply-chain): Pass
#   Workspace Lints (linting): Fail (needs [lints] config)
#   Editor Config (editor-config): Pass
#   Spell Check (editor-config): Warn (typos.toml recommended)
#   Contributor Guide (documentation): Pass

# 2. Generate retrofit plan
praxis-retrofit generate plan /path/to/wasm4pm > wasm4pm-retrofit-phase1.json

# 3. Review the plan
cat wasm4pm-retrofit-phase1.json | jq .actions

# 4. Apply retrofit
praxis-retrofit apply retrofit /path/to/wasm4pm

# 5. Validate success
praxis-retrofit apply validate /path/to/wasm4pm
praxis-retrofit validate compliance /path/to/wasm4pm
```

## Scaling to Fleet

To retrofit all 18 repos in the seanchatmangpt ecosystem:

```bash
#!/bin/bash
# retrofit-all.sh
for repo in praxis wasm4pm wasm4pm-compat pm4py-rs pm4wasm miniml dteam prolog8 ocpq ...; do
    echo "Retrofitting $repo..."
    praxis-retrofit audit report /path/to/$repo | jq .score
    praxis-retrofit apply retrofit /path/to/$repo
    praxis-retrofit validate compliance /path/to/$repo
done
```

Alternatively, use **Parallel Audit** (Phase B):

```bash
# Audit all 18 repos in parallel using 10 agents
praxis-retrofit audit fleet /repos-root  # (planned feature)
```

## Features

### Core Features

- ✓ Repository compliance auditing
- ✓ Retrofit plan generation
- ✓ File creation/update/deletion
- ✓ Commit message generation
- ✓ Risk assessment per phase
- ✓ JSON output for machine consumption

### Planned Features

- ⏳ **Fleet-wide parallel audit** — Run 10-agent exploration on all 18 repos simultaneously
- ⏳ **Git integration** — Auto-create branches, open PRs, handle merges
- ⏳ **CI/CD integration** — Add `praxis validate` as GitHub Actions gate
- ⏳ **Rollback support** — Revert retrofit changes if validation fails
- ⏳ **Dependency graph analysis** — Suggest [workspace.dependencies] unification
- ⏳ **Custom rules** — Load house standards from YAML config instead of hardcoding

## Dependencies

**Core:**
- `clap-noun-verb` — CLI framework
- `serde_json` — JSON serialization
- `toml` — Cargo.toml parsing
- `walkdir` — File traversal
- `blake3` — Content addressing
- `tracing` — Observability

**Dev:**
- `tempfile` — Fixture creation
- `insta` — Snapshot testing

## License

MIT OR Apache-2.0

## Author

Part of [seanchatmangpt/praxis](https://github.com/seanchatmangpt/praxis).

---

## Next Steps

1. **Review** the retrofit case study: `/home/user/praxis/case-study-wasm4pm-retrofit.md`
2. **Build** the tool: `cargo build --release --bin praxis-retrofit`
3. **Test** on wasm4pm: `praxis-retrofit audit report /path/to/wasm4pm`
4. **Apply** Phase 1: `praxis-retrofit apply retrofit /path/to/wasm4pm`
5. **Validate**: `praxis-retrofit validate compliance /path/to/wasm4pm`
6. **Scale** to all 18 repos in the ecosystem


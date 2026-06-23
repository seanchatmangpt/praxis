# Architecture & Structure of ggen Packs

### 1. Architecture & Structure
Packs act as a **pure composition layer** (Layer 3) in the ggen ecosystem. They do not introduce new template rendering logic; instead, they sit atop the existing `ggen-marketplace` and `ggen-core` infrastructure.
- **Components:** Pack CLI layer, Domain layer (Service, Composer, Validator), Storage layer (Registry, Cache, DB), and Integration layer (Template execution, SPARQL queries, maturity scoring).
- **Pipeline Flow:** Generation follows a strict 6-stage pipeline: Load → Validate → Resolve → Merge → Execute → Collect. 
- **Manifest:** They are defined via `pack.toml` (or YAML) files which specify metadata, dependencies, templates, variables, SPARQL queries, and execution hooks.

### 2. Role in the Ecosystem
Packs bridge the gap between isolated templates and full-scale application scaffolding. 
- **Goal:** They allow users to combine multiple marketplace templates, semantic RDF/SPARQL rules, and validators into versioned bundles. For example, a user can generate a complete project ("startup + devops + monitoring") in a single command (`ggen packs generate`).
- **Semantic Integration:** Packs heavily rely on RDF graph traversal via Oxigraph and `render_with_rdf`, combining code generation with ontology-driven constraints.

### 3. Current Status
The packs system is currently in transition from Phase 1 (foundational operations with dry-run rendering and placeholder SPARQL) into **Phase 2 & Phase 3 (v3.3.0/v3.3.1+)**, which introduces the complete package installation system, execution engines, and cloud CDN distribution.
- **Recent Milestones:** According to the recent **Path A Feature Gate Implementation Summary** (May 2026), full integration and E2E testing for packs have been unblocked and executed. This means the 5-stage pack sync pipeline (μ₁–μ₅), lockfile generation (`.ggen/packs.lock`), artifact/receipt generation (with BLAKE3 signing), and proof-gate validation engines are now actively operational and verified in CI without mocks.

### 4. Enforcing Architectural Constraints
Packs enforce robustness and strict repository constraints through multiple innovative strategies:
- **Poka Yoke (Error-proofing):** multi-level confirmation hooks, robust type validation for variables, path traversal protections, and installation plan previews before mutating files.
- **FMEA Mitigations:** They use a transaction log for ACID-like atomic rollbacks on failure, `PubGrub` for fast semantic version/diamond-dependency conflict resolution, and lockfiles to enforce deterministic generation.
- **Strict Verification (Aligned with AGENTS.md Constitution):** Packs conform to the repo’s high standards by producing cryptographically valid **BLAKE3 receipts** and **OCEL traces**. Executions are gated by a proof-gate validator that requires multi-surface corroboration (tracing, process boundaries, causal consistency). The pipeline avoids mocks, ensuring real executions (e.g., real HTTP downloading, hashing, and writing) take place during generation and testing.
- **Streaming Verification (TRIZ merging):** Checksums, signature verification (Ed25519), and package downloading are executed in a single pass to be memory efficient and fail-fast.

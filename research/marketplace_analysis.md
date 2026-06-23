# ggen Marketplace Ecosystem Analysis

### 1. Overview & Structural Components
The `ggen` marketplace is a **governed capability composition platform** heavily tailored to Fortune 5 CISO requirements for enterprise safety, determinism, and provability. It distributes code generation templates, utilities, and AI integrations driven by RDF ontologies and SPARQL queries.

Key structural concepts include:
- **Atomic Packs**: The canonical, fundamental units of the marketplace, strictly categorized into 9 types (e.g., *Surface*, *Contract*, *Projection*, *Runtime*, *Policy*, *Validator*, *Core*).
- **Bundles**: Ergonomic aliases (e.g., `mcp-rust-axum`) that strictly and deterministically expand into multiple atomic packs during compilation.
- **Ownership Maps**: To prevent generation conflicts, packs must declare ownership over emitted files or RDF namespaces. The system enforces resolution via classes like `Exclusive`, `Mergeable`, and `ForbiddenOverlap`.
- **Trust Tiers & Receipts**: Packs are assigned trust tiers (e.g., `EnterpriseCertified`, `Quarantined`). The pipeline generates cryptographic **Composition Receipts** (using Ed25519 signatures and SHA256 hashes) to trace the exact provenance of every executed SPARQL query and Tera template.
- **μ-Pipeline Integration**: Packs hook into a strict multi-stage generation pipeline (μ₀ to μ₅) encompassing Pack Resolution, Ontology Extraction, Emission, and Cryptographic Canonicalization.

### 2. Current Status & Adoption
Despite the robust architectural vision, the actual marketplace ecosystem is currently in a nascent, highly experimental state with low production maturity.
- **Ecosystem Footprint**: The registry contains 77 packages, targeting highly diverse domains (e.g., Healthcare EHRs, ISO-20022 Payments, AI microservices, Dev utilities).
- **Maturity & Quality**: According to the latest validation scorecards, **only 2 packages (2%) are strictly "Production Ready"** (`ai-code-generation` and `dlss-curriculum`). 55% "need improvement," and 41% are deemed "not ready"—frequently missing standard documentation or source repositories.
- **Critical Technical Debt**: A recent `MARKETPLACE_AUDIT_REPORT.md` flagged severe logic and security defects:
  - **Non-Deterministic Receipts**: Receipt hashes currently fail cryptographic custody chains because they serialize standard Rust `HashMap`s, which randomize iteration order.
  - **Cache Misses**: A flaw in cache validation compares compressed archive digests against uncompressed directory hashes, causing 100% cache invalidation and re-downloads.
  - **Security (Zip Slip)**: Archive extraction currently lacks path traversal checks, exposing the installer to arbitrary file overwrites.
  - **Ontology Violations**: OWL semantic violations in the base `ontology.ttl` and hardcoded public registry mappings bypass intended enterprise policy checks.

### 3. Future Trajectory & Roadmap
The product roadmap for the upcoming v26.5.19 engineering slice signals a strong pivot away from open community features toward strict, deterministic enterprise governance.
- **Immediate Priorities (Minimum Shippable)**: The focus is entirely on stability and auditability. This includes `.ggen/packs.lock` file authoring, strict Ed25519 signature enforcement, integrating pack SPARQL rules securely into the μ₂ extraction stage, and establishing synthetic epochs for pack-only receipt generation.
- **Explicit Deferrals**: Public registry UX, community marketplace CLI commands, and auto-emission convenience features have been explicitly deferred.
- **Strategic Doctrine**: The platform enforces a "Big Bang 80/20" gate, requiring users to rely on established standard ontologies (like schema.org or Dublin Core) instead of custom graphs. The goal is to enforce a highly locked-down, mathematically proven enterprise composition engine rather than a loose, npm-style open marketplace.

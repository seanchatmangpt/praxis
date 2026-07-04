# Vision 2030 — PRD / ADR
## Receipted Projection as the Default Trust Layer for Software

---

## PRD (Product Requirements)

### Problem
LLM agents now generate most code. Generation is cheap; verification is expensive. "The report says done" is worthless as evidence — the session that produced this document caught silently-swapped CLI flags, infinite loops claiming idempotence, and committed corruption that no one noticed. The trust asymmetry is the product gap.

### One-line thesis
**2027: packs make projection the default way software is integrated. 2030: receipts make projection the default way software is trusted.**

### Users
1. **Consumers** — declare packs in `ggen.toml`; integration surfaces precipitate in place, receipted.
2. **Producers (OSS maintainers)** — ship a pack (ontology + templates) alongside their API; the pack *is* the integration contract.
3. **Agents (LLM)** — gated narrators. Their output is `AuthorityKind::LlmProjection` until evidence gates admit it.
4. **Auditors/CI** — verify chains, never re-review by reading.

### Product pillars & requirements

**P1 — Software as admitted projection**
- R1.1: All generated surfaces (routes, catalogs, tests, docs, CI) are projections of an admitted graph via `A = μ(O)`; hand-written code is confined to a declared handlers layer.
- R1.2: Second sync on unchanged inputs is a provable no-op (`sync(sync(F)) = sync(F)`), asserted at byte + receipt-payload level.
- R1.3: Generated/hand-written byte ratio is a first-class receipt field (the "intent residue" metric).

**P2 — Receipts, not review**
- R2.1: Every write decision (written / skipped-with-reason / injected / refused) is bound into a BLAKE3-chained receipt; history is append-only (`receipt-log.jsonl`), tamper breaks verification at the named index.
- R2.2: Unreceipted change = unadmittable change (CI gate analogous to unsigned-commit rejection).
- R2.3: Verification is deterministic Old-AI computation (real binaries spawned, hashes recomputed) — never LLM attestation.

**P3 — Ontology governance (the open frontier)**
- R3.1: Closed predicate vocabulary per namespace; unknown predicates refused by name.
- R3.2: Asserted-vs-derived split enforced (status/fitness properties only via CONSTRUCT from evidence — never hand-asserted).
- R3.3: Ontology deltas are themselves receipted (who changed the graph, from-hash to-hash), using the existing `Delta` algebra.

**P4 — Cross-pack receipt federation** (identified gap)
- R4.1: `ggen.lock` binds not just pack content hash but the pack's **receipt-chain head** — transitive attestation down the supply chain.
- R4.2: A pack regenerated from a corrupted upstream ontology fails consumer admission even with a valid content hash.

### Success metrics (2030)
- ≥80% of integration code in adopting projects is receipted projection.
- Receipt verification cost ≤ O(hash recompute); zero human review required for admitted changes.
- Zero silent-drift incidents in receipted paths (every drift is a named FM-coded refusal).

### Non-goals
- ggen never performs process analysis (emission only — analysis belongs to wasm4pm).
- No `generated/` quarantine directories; code lands where code lives.
- LLMs are never an evidence source.

---

## ADR (Architecture Decisions)

**AD-1: Template as unit of intent.** Frontmatter (closed vocabulary, `deny_unknown_fields`) carries destination, queries, merge semantics; TOML only wires. *Rationale: proven this session; unknown keys refuse by name.*

**AD-2: Fail closed everywhere.** Typed FM-coded errors; differing content without `force` refuses; corrupt pack refused by name (FM-PACK-008); invalid receipt is never a cheerful `valid:false`. *Status: implemented, falsifier-tested.*

**AD-3: No wall-clock in hashed material.** Determinism is the substrate of replay; `ts_ns` is input-derived. *Status: enforced by grep-gate, 180+ tests.*

**AD-4: Three-law convergence as the standard.** (1) closed vocabulary, (2) status earned by evidence, (3) chained tamper-evident history — required of every new component, since all five ecosystem projects independently converged on it.

**AD-5: Deterministic-order SPARQL projections.** All generation SELECTs carry ORDER BY; aggregates require ordered subqueries (the GROUP_CONCAT flag-swap incident is the precedent).

**AD-6 (2030-open): Triple admission gates + federation protocol.** The two decisions not yet made: the evidence schema for admitting a triple, and the wire format for cross-pack receipt-head binding. These are the roadmap.

**Rollout:** 2026 (done): engine, packs, lockfile, chained receipts, doctor, watch. 2027: pack ecosystem + registry. 2028: ontology governance. 2029: federation. 2030: unreceipted = unadmitted.

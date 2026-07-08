# Semantic-Web Audits: Conclusions & Architecture

**Date**: 2026-07-08  
**Status**: Complete  
**Audits Completed**: PROJ-501 (OWL RL), PROJ-503 (SHACL), PROJ-505 (horned-owl)

---

## What the Audits Found

### reasonable (OWL RL, BSD-3-Clause)
- Single-file Datalog reasoner (93 KB reasoner.rs)
- 68 KB test suite (W3C conformance reference)
- **Key insight**: OWL 2 RL is ~100 Datalog rules, expressible in small, fast core

### shacl ecosystem (SHACL, MIT/Apache-2.0)
- 3-layer architecture: RDF → AST → IR → Validation
- Parallel constraint evaluation (rayon), deterministic ordering (petgraph, prefixmap)
- 20+ constraint components in SHACL Core
- **Key insight**: Heavy features (SPARQL, recursive shapes) are feature-gated, not core

### horned-owl (OWL AST, LGPL-3.0)
- 200+ axiom types (too large for daily use)
- IRI deduplication via Rc<str> interning (10x memory efficiency)
- Normalize pattern: simplify → reanonymize → sort → hash → replay
- **Key insight**: Full OWL 2 DL support is unnecessary; study patterns clean-room instead

---

## What the Audits Tell Us

**Pattern across all three audits**:

1. **Daily-use features are small**: reasonable's reasoner is 93 KB, shacl's core is ~500 lines
2. **Heavy features are explicit**: SPARQL is optional, recursive shapes are warning-gated, OWL DL is refused
3. **Silent approximation fails**: All three projects refuse features explicitly, not silently
4. **Determinism matters**: BTreeSet ordering, prefixmap iteration, normalize pattern — all designed for stable receipts

---

## The Doctrine Emerges

From these audits comes a clear architecture:

**Graphlaw should NOT be a universal semantic-web engine.**

Instead:

**Graphlaw should be a standing engine with bounded semantic profiles.**

The 80/20 split maps cleanly to what we found:
- **80% daily**: The core that's implemented, tested, receipt-stable
- **20% heavy**: The features that are explicitly refused or out-of-band

---

## Implementation Consequences

### PROJ-401: Quick-Win Crates

Now **informed by audits**, implement in this order:

1. **SymbolId interning** (lasso) — from horned-owl IRI caching pattern
2. **ID-based triples** — from horned-owl/shacl AST design
3. **Compiled dialect IR** — from shacl 3-layer architecture (RDF → AST → IR → Validation)
4. **Bitset closures** — from horned-owl/reasonable closure patterns
5. **Type-aliased fast maps** (rustc-hash) — from shacl constraint evaluation pattern
6. **Deterministic receipt boundaries** — from horned-owl normalization pattern

**Representation-level wins are expected to dominate (10-100x on specific paths). Multiplier claims deferred until Phase 0 benchmarks run.** (See PERFORMANCE_FINDINGS.md and ticket_401 for details.)

### PROJ-501, 503, 505: Semantic Dialects

Now **bounded by doctrine**:

- **PROJ-501** (OWL RL): Implement daily profile only (subclass, subproperty, domain, range, inverse, symmetric, bounded transitive). Refuse `owl:sameAs` unbounded.
- **PROJ-503** (SHACL): Implement Core only. SPARQL is out-of-band. Recursive shapes are profile-gated.
- **PROJ-505** (horned-owl): Study clean-room, don't import. Only if PROJ-501 needs typed AST (conditional).

### PROJ-502, 504: ShEx & N3

Clarify scope via doctrine:

- **PROJ-502** (ShEx): Do you need basic shape maps (80%), or semantic-action-heavy ShEx (20%)? Only 80% goes in core.
- **PROJ-504** (N3): Do you need bounded forward rules (80%), or unbounded built-ins (20%)? Only 80% goes in core.

---

## Standing Ledger Entry

```
SEMANTIC_PROFILE_DOCTRINE_ADOPTED=TRUE
DOCTRINE_SOURCE=AUDIT_FINDINGS (PROJ-501, PROJ-503, PROJ-505)
EIGHTY_TWENTY_STRATEGY=ENFORCED
DAILY_PROFILE_IMPLEMENTATION=REQUIRED
HEAVY_FEATURE_REFUSAL=REQUIRED
SILENT_APPROXIMATION=FORBIDDEN
RECEIPT_STABILITY=PREREQUISITE
```

---

## Why This Matters

**Before doctrine**: "Do we implement SPARQL? Do we support recursive shapes? How much OWL DL?"

Answer: arbitrary, leads to bloat, standing gets fuzzy.

**After doctrine**: "Is this in the daily 80%?"

Answer: clear, measurable, documented.

---

## Next Steps

1. **Update PROJ-401 ticket** with audit-informed crate priorities
2. **Scope PROJ-501/503** to daily profiles only
3. **Clarify PROJ-502/504** scope questions via doctrine
4. **Reference doctrine** in all semantic-dialect error messages
5. **Add doctrine audit** to standing checklist (`docs/standing/REALITY_INDEX.md`)

---

## The Architecture, Stated Clearly

**Graphlaw handles the daily 80% of semantic dialects inside a small deterministic standing core and refuses the heavy 20% by name.**

That is the doctrine.

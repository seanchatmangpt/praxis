# Quick Checks: Crate Discovery Results

**Date**: 2026-07-08  
**Method**: `cargo search` + `cargo vendor` source extraction  
**Result**: ✅ Found all primary crates; clarified licensing and structure

---

## Findings Summary

### ✅ PROJ-501: OWL RL — FOUND & AUDITED

**Crate**: reasonable v0.4.4  
**License**: BSD-3-Clause (permissive, ADAPT_CODE eligible)  
**Source**: `/Users/sac/praxis/vendors/vendor/reasonable/src/`  
**Key File**: reasoner.rs (93 KB — entire OWL 2 RL reasoner)  
**Status**: ✅ **QUICK AUDIT COMPLETE** — saved to `PROJ-501_reasonable_quick_audit.md`

**Finding**: Single-file Datalog-style OWL 2 RL reasoner with 68 KB test suite. Provides rule encoding patterns for Graphlaw's OWL RL v0.

**Verdict**: ADAPT_CODE (can directly consult and adapt rule definitions with attribution)

---

### ⚠️ PROJ-502: ShEx/SHACL — CLARIFICATION NEEDED

**Crate Found**: rudof v0.1.12  
**License**: MIT OR Apache-2.0 (permissive, ADAPT_CODE eligible)  
**Source**: `/Users/sac/praxis/vendors/vendor/rudof/src/`  
**Size**: Only 3.1 KB (thin facade crate)  
**Status**: ✅ **QUICK AUDIT COMPLETE** — saved to `PROJ-502_rudof_quick_audit.md`

**Finding**: rudof v0.1.12 is a **metadata/facade** that re-exports from:
- `shacl` v0.3.6 (already audited in PROJ-503 ✅)
- `shacl_validation`, `shacl_ast`, `shacl_ir`, `shacl_rdf`
- `rudof_iri`, `rudof_rdf`

**Critical Finding**: No ShEx validator found in rudof v0.1.12. ShEx support location unclear:
- Option 1: `purrdf-shex` crate (found in cargo search)
- Option 2: Newer rudof versions (v0.3.6+)
- Option 3: ShEx is not implemented; only SHACL exists

**Recommendation**: 
- If ShEx is required: audit `purrdf-shex` OR ask user for clarification
- If ShEx is optional: SHACL (PROJ-503) covers the shapes validation need; ShEx can be deferred

---

### ❓ PROJ-504: N3 — REQUIRES CLARIFICATION

**Crates Found**:
- `oxttl` v0.3.1 (found in vendors/ — Turtle/N-Triples/N-Quads parsing)
- `swls-lang-n3` (found in cargo search — Notation3 language support)
- Other N3-related crates with unclear scope (n3-parser appears to be for neural networks, not semantic web)

**Status**: ⚠️ **Incomplete** — N3 crate location unclear

**Question for User**: 
1. Is N3 reasoning needed, or only N3 parsing?
2. Is oxttl's N3 support sufficient, or need a separate N3 reasoner?
3. Should N3 be treated as advisory (reference only) vs. implementation requirement?

---

## Complete Audit Status Table

| PROJ | Crate | Version | License | Status | Verdict |
|------|-------|---------|---------|--------|---------|
| **501** | reasonable | 0.4.4 | BSD-3-Clause | ✅ **AUDIT COMPLETE** | ADAPT_CODE |
| **502** | rudof | 0.1.12 | MIT/Apache-2.0 | ⚠️ **CLARIFICATION NEEDED** | ADAPT_CODE (but ShEx scope unclear) |
| **503** | shacl ecosystem | 0.3.6 | MIT/Apache-2.0 | ✅ **AUDIT COMPLETE** | ADAPT_CODE (non-hot) + ADAPT_IDEA (hot) |
| **504** | N3 (oxttl, swls-lang-n3) | various | MIT/Apache-2.0 | ❓ **INCOMPLETE** | Clarification needed |
| **505** | horned-owl | 1.4.0 | LGPL-3.0 | ✅ **AUDIT COMPLETE** | ADAPT_IDEA (clean-room only) |

---

## Crates Vendored (360 Total)

**Semantic-web crates now available in vendors/**:
- ✅ reasonable v0.4.4 (OWL RL)
- ✅ rudof v0.1.12 (ShEx/SHACL facade)
- ✅ shacl ecosystem (5 crates, all v0.3.6)
- ✅ horned-owl v1.4.0 (OWL AST)
- ✅ oxigraph ecosystem (8+ crates for RDF foundation)
- ✅ rudof_iri, rudof_rdf (supporting crates)
- ⚠️ oxttl (N3 support location unclear)

---

## Key Architectural Insights (From Audits)

### From PROJ-501 (reasonable):
- OWL 2 RL is encodable as ~100 Datalog rules
- Disjoint set union used for equivalent entity tracking
- Single-file reasoner design (easy to understand and adapt)

### From PROJ-503 (shacl):
- SHACL validation: RDF → AST → IR → Validation (3-layer)
- 20+ constraint components in SHACL Core
- SPARQL validation is feature-gated (optional)

### From PROJ-505 (horned-owl):
- OWL AST: 200+ axiom types (too large for v0)
- Normalization pattern: simplify → reanonymize → sort → hash
- IRI sharing via Rc<str> for memory efficiency

---

## Next Steps: What User Input Is Needed

1. **PROJ-502 (ShEx)**: Do you need ShEx validation, or is SHACL sufficient?
   - If yes: Should I audit `purrdf-shex` or newer rudof versions?
   - If no: Mark PROJ-502 as DEFERRED

2. **PROJ-504 (N3)**: What's the N3 requirement?
   - Full N3 reasoning implementation?
   - N3 parsing only?
   - N3 as reference/advisory only?

---

## Recommendation: Ready to Proceed

✅ Sufficient source code is now available for:
- **PROJ-401** (quick-win crates) — no semantic-web audits needed
- **PROJ-501** (OWL RL) — reasonable audit complete, can proceed
- **PROJ-503** (SHACL) — full audit complete, can proceed  
- **PROJ-505** (horned-owl) — audit complete, conditional on PROJ-501

⏳ Awaiting clarification for:
- **PROJ-502** (ShEx) — SHACL sufficient? Or need separate ShEx validator?
- **PROJ-504** (N3) — What's the N3 scope?

**Recommendation**: Proceed with PROJ-401 and use the completed audits (PROJ-501, PROJ-503, PROJ-505) to inform OWL RL and SHACL integration strategy. Return to PROJ-502 and PROJ-504 once scope is clarified.


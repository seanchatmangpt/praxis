# Milestone Overview: v26.7.8 — Quick-Win Crate Optimizations & Foundation Hardening

v26.7.8 focuses on representation efficiency (interning, bitsets, fast hashes) and foundation hardening (datafrog audit, extended negative fixtures).

## Ticket index

| # | Name | Scope | Dependencies | Status |
|---|------|-------|--------------|--------|
| 401 | [Quick-Win Rust Crate Optimizations](ticket_401_quick_win_crate_optimizations.md) | Symbol interning, ID triples, FixedBitSet closures, fast hashes, SmallVec, deterministic receipt surfaces; rayon/hashbrown/roaring as P1 follow-ups | PROJ-301..306 | PLANNED |
| 402 | [Datafrog Audit & Learning](ticket_402_datafrog_audit_and_learning.md) | Audit datafrog implementation for algorithm insights; benchmark as comparator; explore as possible future Datalog backend | PROJ-401 | PLANNED |
| 501 | [OWL RL Audit & Adaptation](ticket_501_owl_rl_audit.md) | Audit `reasonable` crate for OWL RL rule encoding, profile scanning, relation layout, diagnostics; hot/non-hot-path adaptation | PROJ-401 | PLANNED |
| 502 | [ShEx/DCTAP Audit & Adaptation](ticket_502_shex_audit.md) | Audit `rudof` crate for ShEx/SHACL ASTs, shape maps, validation reports, conversions; hot/non-hot-path adaptation | PROJ-401 | PLANNED |
| 503 | [SHACL Validation Audit & Adaptation](ticket_503_shacl_audit.md) | Audit `shacl`, `shacl_validation`, `oxirs-shacl` for test harness, validation traits, report rendering; SHACL-SPARQL boundary | PROJ-401 | PLANNED |
| 504 | [N3 Reasoning Audit & Adaptation](ticket_504_n3_audit.md) | Audit `oxirs-ttl::n3`, `eyeron` for parsing, proof traces, denial handling, built-ins; forward/backward chaining separation | PROJ-401 | PLANNED |
| 505 | [OWL AST & Ontology Audit](ticket_505_owl_ast_audit.md) | Audit `horned-owl` for typed OWL AST, profile detection, axiom normalization; conditional on PROJ-501 findings (P1, optional) | PROJ-501 | PLANNED |

---

## Notes

**Phase 1 (Immediate, P0):**
- PROJ-401: Quick-Win crates for symbol interning, bitsets, fast hashes, SmallVec, deterministic receipt surfaces
- PROJ-402: Datafrog audit for algorithm patterns and benchmark comparison
- PROJ-501..504: Rust semantic-web library audits (OWL RL, ShEx/SHACL, N3) with **license-aware adaptation policy**: compatible licenses allow ADAPT_CODE for non-hot-path, incompatible licenses require clean-room ADAPT_IDEA

**Phase 2 (Post-measurement, P1):**
- PROJ-401 follow-ups: hashbrown, rayon, roaring (after profiling)
- PROJ-505: OWL AST audit (conditional, only if PROJ-501 determines it's needed)

**Adaptation Policy:**
- **Compatible licenses** (MIT/Apache-2.0/BSD): ADAPT_CODE with attribution and module isolation (non-hot-path only)
- **Copyleft licenses** (GPL/AGPL/LGPL): ADAPT_IDEA via clean-room reimplementation (behavior/algorithm only, no code copy)
- **No/unclear licenses**: NO_CODE_COPY (audit for ideas/algorithms only)

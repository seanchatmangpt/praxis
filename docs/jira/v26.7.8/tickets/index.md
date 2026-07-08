# Milestone Overview: v26.7.8 — Quick-Win Crate Optimizations & Foundation Hardening

v26.7.8 focuses on representation efficiency (interning, bitsets, fast hashes) and foundation hardening (datafrog audit, extended negative fixtures).

## Ticket index

| # | Name | Scope | Dependencies | Status |
|---|------|-------|--------------|--------|
| 401 | [Quick-Win Rust Crate Optimizations](ticket_401_quick_win_crate_optimizations.md) | Symbol interning, ID triples, FixedBitSet closures, fast hashes, SmallVec, deterministic receipt surfaces; rayon/hashbrown/roaring as P1 follow-ups | PROJ-301..306 | COMPLETE |
| 402 | [Datafrog Audit & Learning](ticket_402_datafrog_audit_and_learning.md) | Audit datafrog implementation for algorithm insights; benchmark as comparator; explore as possible future Datalog backend | PROJ-401 | PLANNED |
| 403 | [Compiled Hook IR](ticket_403_compiled_hook_ir.md) | HookId/EventId newtypes, CompiledHook struct, ID-based scheduler (replace string-keyed schedule_hooks) | PROJ-401 | PLANNED |
| 404 | [Compiled Condition & Feature/Profile IR](ticket_404_compiled_condition_and_feature_profile_ir.md) | CompiledCondition enum, FeatureDecision/ProfileDecision classifiers, 80/20 dialect boundary (OWL RL, SHACL, ShEx, N3, Datalog) | PROJ-403 | PLANNED |
| 405 | [Compiled Rule IR & Join Selectivity](ticket_405_compiled_rule_ir_and_join_selectivity.md) | CompiledRule struct, Selectivity enum, order_body_patterns heuristic (exact → pred+obj → ... → full-scan) | PROJ-401 | PLANNED |
| 406 | [Semi-Naive Delta Materialization](ticket_406_semi_naive_delta_materialization.md) | FactStore with delta/all sets, DerivationGate for canonical provenance, duplicate derivation suppression | PROJ-405 | PLANNED |
| 407 | [Compiled Shape IR (SHACL/ShEx)](ticket_407_compiled_shape_ir_shacl_shex.md) | CompiledShape/CompiledTarget/CompiledConstraint with CostClass ordering; SHACL-SPARQL boundary decision (CORE_ONLY vs SPARQL_OPTIONAL vs FEDERATED_ONLY) | PROJ-401 | PLANNED |
| 408 | [Compiled Delta Template IR](ticket_408_compiled_delta_template_ir.md) | TemplatePart/CompiledTripleTemplate/CompiledDeltaTemplate (replace runtime placeholder-string scanning with slot-lookup) | PROJ-403 | PLANNED |
| 409 | [Bitset Closure Integration](ticket_409_bitset_closure_integration.md) | Audit dense-ID closure sites; if found, ClosureMatrix with FixedBitSet + canonical rendering rule | PROJ-401 | PLANNED |
| 410 | [Canonical Standing Boundary Hardening](ticket_410_canonical_standing_boundary_hardening.md) | RuntimeState/StandingState separation, DiagnosticBuffer/CanonicalReceiptMaterial builders, Scratch arena, canonical-sort parallelism gate (docs-only) | PROJ-401, PROJ-406 | PLANNED |
| 501 | [OWL RL Audit & Adaptation](ticket_501_owl_rl_audit.md) | Audit `reasonable` crate for OWL RL rule encoding, profile scanning, relation layout, diagnostics; hot/non-hot-path adaptation | PROJ-401 | PLANNED |
| 502 | [ShEx/DCTAP Audit & Adaptation](ticket_502_shex_audit.md) | Audit `rudof` crate for ShEx/SHACL ASTs, shape maps, validation reports, conversions; hot/non-hot-path adaptation | PROJ-401 | PLANNED |
| 503 | [SHACL Validation Audit & Adaptation](ticket_503_shacl_audit.md) | Audit `shacl`, `shacl_validation`, `oxirs-shacl` for test harness, validation traits, report rendering; SHACL-SPARQL boundary | PROJ-401 | PLANNED |
| 504 | [N3 Reasoning Audit & Adaptation](ticket_504_n3_audit.md) | Audit `oxirs-ttl::n3`, `eyeron` for parsing, proof traces, denial handling, built-ins; forward/backward chaining separation | PROJ-401 | PLANNED |
| 505 | [OWL AST & Ontology Audit](ticket_505_owl_ast_audit.md) | Audit `horned-owl` for typed OWL AST, profile detection, axiom normalization; conditional on PROJ-501 findings (P1, optional) | PROJ-501 | PLANNED |

---

## Notes

**Phase 1 (Immediate, P0):**
- PROJ-401: Quick-Win crates for symbol interning, bitsets, fast hashes, SmallVec, deterministic receipt surfaces (COMPLETE)
- PROJ-402: Datafrog audit for algorithm patterns and benchmark comparison
- PROJ-403: Compiled Hook IR (HookId, CompiledHook, ID-based scheduler)
- PROJ-404: Compiled Condition IR + Feature/Profile IR (CompiledCondition, FeatureDecision, dialect boundary)
- PROJ-501..504: Rust semantic-web library audits (OWL RL, ShEx/SHACL, N3) with **license-aware adaptation policy**: compatible licenses allow ADAPT_CODE for non-hot-path, incompatible licenses require clean-room ADAPT_IDEA

**Phase 2 (Compiled IR & Foundation, P1):**
- PROJ-405: Compiled Rule IR + Join Selectivity (CompiledRule, Selectivity, order_body_patterns)
- PROJ-406: Semi-Naive Delta Materialization (FactStore, DerivationGate)
- PROJ-407: Compiled Shape IR (CompiledShape, CostClass, SHACL dialect boundary decision)
- PROJ-408: Compiled Delta Template IR (CompiledDeltaTemplate, slot-based projection)
- PROJ-410: Canonical Standing Boundary Hardening (RuntimeState/StandingState, DiagnosticBuffer, Scratch arena, parallelism gate)
- PROJ-409: Bitset Closure Integration (conditional on audit finding dense closure site)

**Phase 3 (Post-measurement, P1 follow-ups):**
- PROJ-401 follow-ups: hashbrown, rayon, roaring (after profiling)
- PROJ-505: OWL AST audit (conditional, only if PROJ-501 determines it's needed)

**Adaptation Policy:**
- **Compatible licenses** (MIT/Apache-2.0/BSD): ADAPT_CODE with attribution and module isolation (non-hot-path only)
- **Copyleft licenses** (GPL/AGPL/LGPL): ADAPT_IDEA via clean-room reimplementation (behavior/algorithm only, no code copy)
- **No/unclear licenses**: NO_CODE_COPY (audit for ideas/algorithms only)

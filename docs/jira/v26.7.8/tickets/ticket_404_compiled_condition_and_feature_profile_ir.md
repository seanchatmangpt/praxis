# PROJ-404 — Compiled Condition IR + Compiled Feature/Profile IR (v26.7.8 P0)

**Status**: PLANNED  
**Scope**: Replace string-keyed hook condition dispatch with compiled `CompiledCondition` enum; add `FeatureDecision`/`ProfileDecision` classifiers for 80/20 dialect boundary  
**Dependencies**: PROJ-403 (Compiled Hook IR)  
**Target**: P0 — unblock PROJ-405/406/407

---

## Overview

Today hook conditions are evaluated at runtime by string matching on condition type (e.g., `"datalog"`, `"threshold"`) and then interpreting string predicates/operands. This ticket introduces a `CompiledCondition` enum with pre-parsed variants and a `FeatureDecision` classifer system to mark which semantic dialect features are supported, unsupported, or require external boundaries (80/20 profile for OWL RL, SHACL, ShEx, N3, Datalog).

**Doctrine source:** "Graphlaw Performance Architecture: Small IDs, Compiled IR, Bounded Profiles, and Canonical Standing" (article).

---

## Deliverables & Tests

### Step 1: CompiledCondition Enum

**Deliverables:**
- Define `enum CompiledCondition`:
  ```rust
  pub enum CompiledCondition {
    Datalog { rule: SymbolId },
    N3 { rule: SymbolId },
    Shape { target_iri: SymbolId, shape_iri: SymbolId },
    Delta { pattern: SymbolId, threshold: i32 },
    Threshold { min_count: usize },
    Count { op: CountOp, value: usize },
    Window { duration_ms: u64 },
    Unsupported { reason: String },
  }
  ```
- Parse `HookCondition` (RDF-read today, `hooks.rs:~200-300`) into `CompiledCondition` at hook load time (in `validate_and_extract_hooks`, `hooks.rs:325-531`)
- Update `evaluate_condition` (`hooks.rs:1244-~1518`, ~274 lines) to dispatch on `CompiledCondition` enum instead of string matching
- No string lookups during condition evaluation; all references are pre-resolved `SymbolId`

**Tests:**
- Parse roundtrip: RDF hook → `CompiledCondition` → render back to RDF ≈ original
- Each condition variant: Datalog/N3/Shape/Delta/Threshold/Count/Window evaluate correctly
- Unsupported conditions: parse as `Unsupported { reason }` and reject at hook-load time with clear error
- Existing condition tests (subset of 79 hook tests) pass

**Acceptance:**
- All hook condition tests pass
- No string-keyed dispatch in `evaluate_condition` hot path
- Condition compilation is deterministic (same RDF → same enum variant every time)

---

### Step 2: FeatureDecision and ProfileDecision Classifiers

**Deliverables:**
- Define `enum FeatureDecision`:
  ```rust
  pub enum FeatureDecision {
    Supported,
    Unsupported { reason: &'static str },
    ExternalBoundaryRequired { endpoint: &'static str },
  }
  ```
- Define `enum ProfileDecision`:
  ```rust
  pub enum ProfileDecision {
    Supported { cost_tier: u8 },
    Unsupported { reason: &'static str },
    ExternalBoundaryRequired { required_endpoint: &'static str },
  }
  ```
- Document the 80/20 dialect profile for this milestone (v26.7.8):
  - **OWL RL**: [core subset documented in PROJ-401/501 audit]; unsupported: full OWL 2 (sameAs, inverse, etc.)
  - **SHACL**: [open via PROJ-502/503]; decision frame: CORE_ONLY vs SPARQL_OPTIONAL vs FEDERATED_ONLY (to be settled by PROJ-407)
  - **ShEx**: [core subset]; unsupported: semantic actions, object-set operations
  - **N3**: [core rules, no negation-as-failure]; unsupported: complex N3 formulae
  - **Datalog**: [stratified, no negation outside body]; unsupported: unstratified negation, aggregate negation
- Add classification functions (e.g., `classify_condition_feature(dialect: &str, feature: &str) → FeatureDecision`) for each dialect
- Add a standing-rule doc describing the boundary in `docs/standing/` or inline in a structured comment

**Tests:**
- Each dialect (OWL RL, SHACL, ShEx, N3, Datalog) — verify core features return `Supported`
- Each dialect — unsupported features return `Unsupported { reason }`
- Cross-dialect: OWL RL + Datalog combination correctly classifies each part
- Boundary decision propagates through hook evaluation (unsupported feature in hook condition → reject at load time)

**Acceptance:**
- FeatureDecision/ProfileDecision enums compile
- Dialect boundaries are documented and enforced
- Unsupported features are rejected before evaluation (refuse at compile/load, not at runtime)
- All existing dialect tests pass

---

### Step 3: Integrate CompiledCondition and FeatureDecision into Hooks

**Deliverables:**
- Update `CompiledHook` (from PROJ-403) to store `condition: CompiledCondition` (not a raw enum variant)
- Update `validate_and_extract_hooks` to call `classify_condition_feature` for each condition and refuse any unsupported feature at hook-load time
- Update `evaluate_condition` to dispatch on `CompiledCondition` enum variants
- Update `HookVerdictRecord` to store `condition_type: SymbolId` (for logging) instead of raw string

**Tests:**
- Hook load: unsupported condition is caught and rejected with clear message
- Hook load: supported condition compiles and evaluates correctly
- Hook execution: all 79 tests pass
- Condition evaluation semantics unchanged (only dispatch mechanism differs)

**Acceptance:**
- Unsupported features are rejected at hook-load time
- All existing hook tests pass
- Condition evaluation is correct and deterministic

---

## Benchmark Targets (Three-Tier Structure)

| Benchmark | Baseline | Target (P0) | Target (P1) |
|-----------|----------|------------|------------|
| Load 20 hooks (mixed conditions) | ~2 ms | ≤ 2 ms | ≤ 1.8 ms |
| Evaluate 100 conditions (per-event) | ~5 ms | ≤ 5 ms | ≤ 4.5 ms |
| Reject 1 unsupported condition at load time | ~1 ms | ≤ 1 ms | ≤ 0.5 ms |

---

## Success Criteria (Final)

- [ ] `CompiledCondition` enum with 8 variants defined
- [ ] Parsing from RDF to `CompiledCondition` is deterministic
- [ ] `FeatureDecision` and `ProfileDecision` enums defined with classification functions
- [ ] Dialect boundary (OWL RL, SHACL, ShEx, N3, Datalog) documented
- [ ] Unsupported features rejected at hook-load time, not runtime
- [ ] All 79 hook tests pass
- [ ] `evaluate_condition` dispatches on enum, no string matching
- [ ] Hook receipts byte-identical to baseline

---

## Acceptance Criteria

- [ ] Code review: enum variants cover all condition types in current hook suite; no blind spot
- [ ] Integration: PROJ-405/406/407 can rely on compiled conditions
- [ ] Standing: `standing.json` reports PROJ-404 as COMPLETE after `just verify-all` passes

---

## Standing Rules

- **ALIVE**: PROJ-404 is ALIVE when `CompiledCondition` enum is tested, all hook conditions compile/evaluate, and unsupported features are rejected at load time
- **PARTIAL_ALIVE**: If some dialect boundary is unresolved (e.g., SHACL-SPARQL decision deferred), document it explicitly; gate downstream tickets if unresolved
- **REFUSED**: If condition variant causes a runtime error, refuse and debug before completion

---

## Related Tickets

- PROJ-403: Compiled Hook IR (required dependency; provides `CompiledHook` struct)
- PROJ-405: Compiled Rule IR (uses similar classification for N3/Datalog rules)
- PROJ-407: Compiled Shape IR (needs to resolve SHACL dialect boundary that this ticket helps frame)
- PROJ-501/502/503/504/505: Semantic dialect audits (OWL RL, ShEx, SHACL, N3, OWL-AST; provide audit context for unsupported feature lists)

---

## References

- `crates/praxis-graphlaw/src/hooks.rs:1244-1518`: `evaluate_condition` (today's string-dispatch implementation)
- `crates/praxis-graphlaw/src/hooks.rs:200-250`: `HookCondition` type and parsing
- Article: "Graphlaw Performance Architecture" — Section "CompiledCondition Enum and Dialect Boundaries"
- PROJ-501 (OWL RL audit), PROJ-502 (ShEx audit), PROJ-503 (SHACL audit), PROJ-504 (N3 audit), PROJ-505 (OWL-AST audit) — dialect-specific unsupported feature lists

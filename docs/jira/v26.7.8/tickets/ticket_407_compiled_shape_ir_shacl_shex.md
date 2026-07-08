# PROJ-407 — Compiled Shape IR (SHACL/ShEx) (v26.7.8 P1)

**Status**: PLANNED  
**Scope**: Introduce `CompiledShape`/`CompiledTarget`/`CompiledConstraint` with cost-ordered evaluation; decide and document SHACL dialect boundary (CORE_ONLY vs SPARQL_OPTIONAL vs FEDERATED_ONLY)  
**Dependencies**: PROJ-401 (COMPLETE)  
**Related by context**: PROJ-502/503 (SHACL/ShEx audits; open questions)  
**Target**: P1 — separate shape validation from hook evaluation

---

## Overview

Today SHACL/ShEx shapes are evaluated by reading constraints directly from the shapes-graph RDF (`shacl.rs:1473`, `shacl.rs:1972`) and dispatching on hard-coded order (declaration order, not cost-based). This ticket introduces `CompiledShape`/`CompiledTarget`/`CompiledConstraint` with a `CostClass` ordering (cardinality → node-kind/datatype → class → path → regex → recursive), and most importantly, **this ticket must decide and document the SHACL-SPARQL dialect boundary** since that decision is currently open (tickets 502/503 pose it as an unanswered question).

**Doctrine source:** "Graphlaw Performance Architecture: Small IDs, Compiled IR, Bounded Profiles, and Canonical Standing" (article).

---

## Deliverables & Tests

### Step 1: Cost-Based Constraint Compilation (SHACL)

**Deliverables:**
- Define `enum CostClass`:
  ```rust
  pub enum CostClass {
    Cardinality,           // sh:minCount, sh:maxCount (earliest check, O(1))
    NodeKind,              // sh:nodeKind (type check, O(1))
    Datatype,              // sh:datatype (string comparison, O(1))
    Class,                 // sh:class (subclass lookup, O(closure))
    Path,                  // sh:path (graph traversal, O(graph))
    Regex,                 // sh:pattern (string regex, O(string))
    Recursive,             // recursive shape reference (O(depth) or O(graph))
  }
  ```
- Define `struct CompiledConstraint`:
  ```rust
  pub struct CompiledConstraint {
    pub cost_class: CostClass,
    pub iri: SymbolId,  // sh:minCount, sh:class, etc.
    pub value: SymbolId,  // the constraint value (e.g., class IRI, regex pattern)
    pub is_optional: bool,  // sh:deactivated
  }
  ```
- Define `struct CompiledTarget`:
  ```rust
  pub struct CompiledTarget {
    pub target_iri: SymbolId,
    pub constraint_type: TargetType,  // sh:targetNode, sh:targetClass, sh:targetSubjectsOf, sh:targetObjectsOf
  }
  ```
- Define `struct CompiledShape`:
  ```rust
  pub struct CompiledShape {
    pub iri: SymbolId,
    pub targets: Vec<CompiledTarget>,
    pub constraints: Vec<CompiledConstraint>,  // sorted by CostClass (Cardinality first, Recursive last)
    pub closed: bool,  // sh:closed (property whitelist enforcement)
    pub property_shapes: Vec<CompiledShape>,  // sh:property (recursion)
  }
  ```
- At load time, parse SHACL shapes from RDF and compile into `CompiledShape` with constraints sorted by `CostClass`
- Update `validate_shape` (`shacl.rs:1473`) and `validate_property_shape` (`shacl.rs:1972`) to iterate `CompiledConstraint` in cost order instead of declaration order

**Tests:**
- Parse roundtrip: RDF shape → `CompiledShape` → canonical form ≈ original
- Cost ordering: constraint sequence is always Cardinality → NodeKind → ... → Recursive
- Unsupported constraints (sh:sparql, sh:hasValue with SPARQL, etc.) are: **[TBD by Step 2]**
- Determinism: same RDF shape → same `CompiledShape` every time
- Existing SHACL tests pass (9 tests in `benches/hierarchies.rs`): `test_shacl_hierarchy_10/100/1000`

**Acceptance:**
- `CompiledShape` struct compiles
- Constraint sorting is correct
- All existing SHACL tests pass

---

### Step 2: SHACL-SPARQL Dialect Decision (Critical Policy Step)

**Deliverables:**
- **Decide and document** the SHACL-SPARQL boundary. This is a standing-rule policy decision that PROJ-503 (SHACL audit) left open. Three options:
  1. **CORE_ONLY** (most conservative): SHACL-SPARQL constraints are marked `Unsupported` and rejected at load time. Shape validation is purely constraint-based, no SPARQL evaluation. → Smallest attack surface, fastest validation, but loses a SHACL expressiveness tier.
  2. **SPARQL_OPTIONAL** (medium): SHACL-SPARQL constraints are parsed and evaluated, but:
     - Pre-binding `$PATH` for property-shape SPARQL is marked as unsupported (SPARQL can only reference focus node and explicit `$this`, `$focusNode` bindings)
     - Remote/federated SPARQL endpoints are refused (no outbound SPARQL calls)
     - → Permits local SPARQL validation against the in-memory RDF, but no federation
  3. **FEDERATED_ONLY** (most permissive): SHACL-SPARQL can invoke remote endpoints (with careful sandboxing and caching). → Enables external validation services, but adds complexity and external dependencies.
- Write the decision in a **standing rule** in this ticket (not deferred to PROJ-503). Cite constraints:
  - v26.7.8 milestone's threat model (is external invocation acceptable?)
  - Performance assumptions (in-memory vs network latency)
  - Graphlaw's existing use cases (are SPARQL constraints in production shapes?)
- Add the decision to `docs/standing/SEMANTIC_PROFILE_DOCTRINE.md` (or create `docs/jira/v26.7.8/SHACL_DIALECT_BOUNDARY.md`)
- Update the `CompiledConstraint` and/or shape compiler to enforce this decision:
  - If CORE_ONLY: reject any shape containing `sh:sparql` at load time
  - If SPARQL_OPTIONAL: parse and evaluate `sh:sparql`, but refuse `$PATH` pre-binding and remote endpoints
  - If FEDERATED_ONLY: evaluate `sh:sparql` with explicit caching/sandboxing (design detail for PROJ-407 implementation)

**Tests:**
- Load a shape with `sh:sparql` constraint; verify it is accepted (SPARQL_OPTIONAL) or rejected (CORE_ONLY) per the decision
- Validate a shape with `sh:sparql` against a test node; verify evaluation is correct
- Attempt a shape with `$PATH` pre-binding in SPARQL: verify it is rejected (SPARQL_OPTIONAL) or accepted (FEDERATED_ONLY — implementation-dependent)

**Acceptance:**
- Dialect boundary decision is documented and enforced in code
- All shape load/validation tests pass under the chosen policy
- Standing rule is clear (future maintainers know which boundary is active)

---

### Step 3: ShEx Compiled Representation and Integration

**Deliverables:**
- Define analogous `struct CompiledShExShape` (parallel to `CompiledShape`) with:
  - Targets (shape references)
  - Compiled triple constraints (pre-parsed, ID-based, no string lookup during validation)
  - Sorted by cost/selectivity (cardinality → datatype → recursive)
- Parse ShEx shapes from RDF/ShExC at load time into `CompiledShExShape`
- Update `validate_shex` (`shex_native.rs:~`) to use compiled shapes instead of ad-hoc constraint checking
- Ensure unsupported ShEx features (semantic actions, complex object sets) are clearly refused or documented as partial

**Tests:**
- Existing ShEx validation tests pass (3 tests in `benches/dialects.rs`): `test_shex_validate_100/1000/5000` + `test_shex_validate_complex_100/1000`
- Roundtrip: ShEx shape → compiled → validation results match baseline
- Unsupported features: attempts to validate a shape with semantic actions are rejected with clear error

**Acceptance:**
- `CompiledShExShape` compiles
- ShEx validation results match baseline
- All existing ShEx tests pass

---

### Step 4: Comprehensive Shape Validation Tests

**Deliverables:**
- Add integration tests covering:
  - SHACL + ShEx shapes loaded together
  - Constraint ordering correctness (cardinality checks are attempted before deep graph traversal)
  - Unsupported feature rejection (per the dialect boundary from Step 2)
  - Benchmark shape validation latency before/after compilation

**Tests:**
- 10 new tests covering combined SHACL/ShEx scenarios
- Benchmark: `test_shacl_validate_100/1000/5000` + `test_shacl_validate_complex_100/1000`

**Acceptance:**
- All new tests pass
- Benchmarks show no regression (ideally improvement from cost-ordering early constraint evaluation)

---

## Benchmark Targets (Three-Tier Structure)

| Benchmark | Baseline | Target (P1) | Target (P1.5) |
|-----------|----------|------------|---------------|
| SHACL validate 100 nodes | ~117 µs | ≤ 117 µs | ≤ 105 µs (cost-order short-circuits) |
| SHACL validate 1000 nodes | ~1.17 ms | ≤ 1.17 ms | ≤ 1.05 ms |
| SHACL complex 1000 nodes | ~1.92 ms | ≤ 1.92 ms | ≤ 1.72 ms |
| ShEx validate 1000 nodes | ~855 µs | ≤ 855 µs | ≤ 770 µs |

---

## Success Criteria (Final)

- [ ] `CompiledShape`, `CompiledTarget`, `CompiledConstraint` structs defined
- [ ] `CostClass` enum with 7 tiers defined
- [ ] SHACL-SPARQL dialect boundary decided, documented, and enforced in code
- [ ] Constraint sorting by `CostClass` is deterministic
- [ ] SHACL shape validation uses compiled shapes and cost-ordered constraints
- [ ] ShEx shape validation uses compiled shapes
- [ ] All existing SHACL/ShEx validation tests pass
- [ ] Benchmarks run; no regressions
- [ ] Unsupported features are clearly rejected or documented

---

## Acceptance Criteria

- [ ] Code review: no unsafe code; cost-class ordering is sound
- [ ] Dialect boundary decision is explicit and referenced in code comments and standing rules
- [ ] PROJ-502/503 open questions are explicitly resolved or escalated (with clear justification for deferral)
- [ ] Standing: `standing.json` reports PROJ-407 as COMPLETE after `just verify-all` passes

---

## Standing Rules

- **ALIVE**: PROJ-407 is ALIVE when `CompiledShape` is integrated, cost-ordered constraint evaluation is correct, SHACL-SPARQL boundary is decided and enforced, and benchmarks pass
- **PARTIAL_ALIVE**: If the dialect boundary decision is unresolved or controversial, document the open question and gate PROJ-410 until it's settled
- **REFUSED**: If shape compilation produces incorrect validation results, refuse and debug

---

## Related Tickets

- PROJ-401: Quick-Win Crate Optimizations (COMPLETE; provides ID-based foundation)
- PROJ-404: Compiled Condition IR (shares compilation philosophy)
- PROJ-502: ShEx Audit (provides ShEx feature boundary; audit context)
- PROJ-503: SHACL Audit (poses the open SHACL-SPARQL boundary question that this ticket must decide)

---

## References

- `crates/praxis-graphlaw/src/shacl.rs:1473`: `validate_shape` (main validation loop)
- `crates/praxis-graphlaw/src/shacl.rs:1972`: `validate_property_shape` (property-level validation)
- `crates/praxis-graphlaw/src/shacl.rs:1162`: Current `sh:sparql` evaluation (inline, not refused)
- `crates/praxis-graphlaw/src/shex_native.rs`: ShEx validation logic
- `crates/praxis-graphlaw/src/shex.rs:1-17`: ShEx re-export/delegation
- Article: "Graphlaw Performance Architecture" — Section "CompiledShape IR and CostClass Ordering"
- `ticket_502_shex_audit.md` / `ticket_503_shacl_audit.md`: Open audit questions (context for this ticket's dialect boundary decision)

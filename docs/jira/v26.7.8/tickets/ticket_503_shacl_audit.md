# PROJ-503 — SHACL Validation Audit & Adaptation (v26.7.8 P0)

**Status**: PLANNED  
**Scope**: Audit Rust SHACL validation crates (`shacl`, `shacl_validation`, `oxirs-shacl`) for pattern/algorithm insights and non-hot-path code adaptation  
**Dependencies**: PROJ-401 (Quick-Win Crate Optimizations) should complete first  
**Audit Targets**: 
  - `shacl` (https://docs.rs/shacl) — W3C test harness + SHACL Core validation
  - `shacl_validation` (https://docs.rs/shacl_validation) — validation trait boundaries
  - `oxirs-shacl` (https://docs.rs/oxirs-shacl) — SHACL Core/SPARQL validator with RDF reporting

---

## Overview

Three complementary Rust SHACL validation implementations provide different angles for Graphlaw:

- **`shacl`**: W3C test-suite integration and SHACL Core structure (fixture source)
- **`shacl_validation`**: Trait-based validation decomposition (AST pattern source)
- **`oxirs-shacl`**: Feature-gated SHACL Core/SPARQL with RDF-rendered diagnostics (hot-path algorithm reference, non-hot-path diagnostic adaptation)

This ticket authorizes a **focused audit** of all three for:
- W3C SHACL conformance test harness structure
- Validation trait design and focus/value-node operations
- SHACL-SPARQL boundary and feature gating
- Validation violation → RDF rendering (diagnostic artifacts)
- Constraint component organization
- Target selection and constraint evaluation
- Non-hot-path module adaptation candidates (report rendering, test harness, feature classification)

**Constraint**: Do NOT adopt SHACL-SPARQL evaluation into the hot path unless Graphlaw explicitly supports federated/remote SPARQL validation. Core validation algorithm/patterns only; reimplement in bounded forms.

---

## Audit Scope

### 1. W3C Test Harness (`shacl` crate)

- Manifest format (SHACL test definition structure)
- Conformation test vs. action test distinction
- Expected result format (pass/fail, validation reports)
- Graph and shapegraph loading
- Test oracle structure (how results are compared)
- Negative test handling (intentional validation failures)

### 2. Validation Trait Boundaries (`shacl_validation` crate)

- Validate trait signature and contract
- Focus node traversal and binding
- Value node operations (proposed values, type checking)
- Constraint evaluation decomposition
- Recursive shape handling (nested shapes, references)
- Modular validation architecture

### 3. SHACL Core/SPARQL Validator (`oxirs-shacl` crate)

- Supported SHACL Core constraints (complete list)
- SHACL-SPARQL integration and feature gating
- Validation violation structure
- RDF rendering of validation reports (`ValidationViolation::to_rdf`)
- Report severity and detail levels
- Conformation vs. non-conformation graph shape
- Unsupported constraints and refusal behavior

### 4. Shared Pattern Analysis

- AST representation overlap (Shape, Constraint, Component)
- Focus/value-node operations consistency
- Report structure and serialization
- Test fixture compatibility
- Determinism guarantees in validation order

### 5. SHACL-SPARQL Boundary

- Which validators support SPARQL constraints?
- How is SPARQL execution integrated (or refused)?
- Remote/federated validation handling (if supported)
- Scope boundary for Graphlaw: CORE_ONLY vs. SPARQL_OPTIONAL vs. FEDERATED_ONLY

### 6. Diagnostic & Report Structures

- Validation violation representation
- Path expressions and focus nodes in reports
- Severity levels (violation, warning, info)
- Remediation suggestions (if supported)
- Report RDF shape and graph structure
- JSON/Turtle serialization formats

### 7. Test Fixtures & Conformance

- W3C SHACL test-suite coverage
- Problem sizes and complexity
- Negative fixtures (intentional refusals)
- Edge cases (empty shapes, circular references, conflicting constraints)

---

## Audit Questions

| Question | Answer | Relevance |
|----------|--------|-----------|
| **Licenses**: SPDX IDs for shacl, shacl_validation, oxirs-shacl? | | Determines ADAPT_CODE vs. ADAPT_IDEA |
| **Test harness**: W3C manifest structure and oracle format? | | Fixture import candidate |
| **Validate trait**: Focus/value-node contract and decomposition? | | Hot-path validation pattern reference |
| **SHACL-SPARQL**: Supported or refused? If supported, how? | | Informs Graphlaw's SPARQL boundary |
| **Report rendering**: Can violations be rendered to RDF? JSON? | | Diagnostic artifact generation |
| **Constraint components**: Complete list and evaluation order? | | Informs Graphlaw's constraint scope |
| **Determinism**: Is constraint evaluation order deterministic? | | Receipt/replay stability requirement |
| **Unsupported features**: What is explicitly NOT supported? | | Informs Graphlaw's refusal list |

---

## Deliverables

### Audit Report (3,000–3,500 words)

**Sections:**

A. **Identity & Licenses**
   - Three crate names, versions, SPDX licenses, maintainer activity
   - Adaptation classes: ADAPT_CODE / ADAPT_CODE_ISOLATED / ADAPT_IDEA / TEST_FIXTURE_ONLY / REFUSE
   - Feature gates and inter-crate dependencies

B. **W3C Test Harness Structure** (shacl)
   - Manifest format and SPARQL/RDF structure
   - Conformation test definition
   - Action test definition (schema validation, target node binding)
   - Expected result format
   - Test oracle (how pass/fail is determined)
   - Negative test patterns

C. **Validation Trait Architecture** (shacl_validation)
   - Validate trait signature and contract
   - Focus node binding and traversal
   - Value node operations (generation, type checking)
   - Constraint evaluation decomposition
   - Recursive shape and reference handling
   - Modular validation pipeline

D. **SHACL Core Constraint Components** (oxirs-shacl)
   - Complete list of supported constraint components (sh:minCount, sh:maxCount, sh:pattern, sh:node, sh:closed, sh:ignoredProperties, etc.)
   - Constraint evaluation semantics
   - Interaction between multiple constraints
   - Default values and optional constraints
   - Unsupported SHACL Advanced Features

E. **SHACL-SPARQL Boundary**
   - SPARQL constraint integration level (full support, partial, minimal, none)
   - Remote/federated SPARQL execution (if supported)
   - Feature gating for SPARQL constraints
   - Graphlaw's scope decision (Core-only vs. SPARQL optional vs. FEDERATED_ONLY)

F. **Validation Report & RDF Rendering**
   - ValidationViolation structure
   - RDF rendering (Subject, Predicate, Object triples)
   - Report graph shape (sh:ValidationReport and sh:ValidationResult)
   - Severity levels and remediation suggestions
   - JSON/Turtle serialization

G. **Test Fixtures & Conformance**
   - W3C test-suite coverage percentage
   - Problem sizes (graph size, constraint count, recursive depth)
   - Negative fixtures (intentional refusals)
   - Edge cases and corner cases

H. **Graphlaw Integration Opportunities**
   - **Hot path**: Which validation patterns inform Graphlaw's constraint evaluation?
   - **Non-hot path**: Which modules can be directly adapted?
     - Validate trait design (ADAPT_IDEA or CLEAN_ROOM)
     - ValidationViolation → RDF rendering (ADAPT_CODE if compatible)
     - Test harness structure (TEST_FIXTURE_ONLY)
     - Constraint component definitions (ADAPT_CODE if compatible)
     - Unsupported-feature classifier (ADAPT_CODE if compatible)

I. **Recommendation & Risk**
   - Adaptation classes and license basis
   - SHACL-SPARQL boundary for Graphlaw (explicit scope)
   - Receipt/replay stability requirements
   - W3C conformance strategy
   - Timeline for integration

---

## Non-Hot-Path Adaptation Candidates

If license-compatible, these modules are candidates for direct code adaptation with attribution and isolation:

| Module | Source | Use In Graphlaw | License Requirement |
|--------|--------|---|---|
| ValidationViolation → RDF | oxirs-shacl | `diagnostic_to_rdf.rs` | Attribution + module isolation |
| Constraint component types | oxirs-shacl | `constraint_components.rs` | Attribution + tests |
| Report RDF structure | oxirs-shacl | `validation_report.rs` | Attribution + serialization tests |
| Test harness | shacl | `shacl_fixtures.rs` | Conform to W3C test license |
| Trait design reference | shacl_validation | Pattern guidance (ADAPT_IDEA) | No code copy required |

---

## Acceptance Criteria

- [ ] Full audit report completed and committed
- [ ] Licenses identified (SPDX) and adaptation classes assigned
- [ ] W3C test harness structure documented
- [ ] Validate trait architecture analyzed
- [ ] All SHACL Core constraint components enumerated
- [ ] SHACL-SPARQL boundary clearly defined
- [ ] Report RDF rendering capability assessed
- [ ] Test fixture suitability evaluated
- [ ] Non-hot-path adaptation candidates identified
- [ ] SHACL-SPARQL scope decision (Core vs. SPARQL vs. Federated) made
- [ ] Recommendations integrated into Graphlaw v26.7.8 planning

---

## Standing Rules

Mark **ALIVE** when:
- Audit report is written and accepted
- Licenses and adaptation classes are recorded for all three crates
- W3C test harness is documented
- Validate trait architecture is understood
- SHACL Core constraint components are enumerated
- SHACL-SPARQL boundary is explicit
- Non-hot-path candidates are isolated and have test plans
- Recommendations are actionable by implementation teams

---

## Related Tickets

- **PROJ-401**: Quick-Win Crate Optimizations — may reference SHACL constraint evaluation insights
- **PROJ-501**: OWL RL audit (sibling)
- **PROJ-502**: ShEx/SHACL audit (sibling, shares findings on SHACL AST/reports)
- **PROJ-504**: N3 audit (sibling)
- **PROJ-307** (future): SHACL validation expansion — will use this audit for constraint evaluation and report structures

---

## References

- shacl crate: https://docs.rs/shacl
- shacl_validation crate: https://docs.rs/shacl_validation
- oxirs-shacl crate: https://docs.rs/oxirs-shacl
- W3C SHACL specification: https://www.w3.org/TR/shacl/
- W3C SHACL test suite: https://github.com/w3c/shacl-test-suite

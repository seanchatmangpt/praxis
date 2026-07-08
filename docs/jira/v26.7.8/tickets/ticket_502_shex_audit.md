# PROJ-502 — ShEx/DCTAP Audit & Adaptation (v26.7.8 P0)

**Status**: PLANNED  
**Scope**: Audit `rudof` crate (Rust ShEx/SHACL/DCTAP support) for pattern/algorithm insights and non-hot-path code adaptation  
**Dependencies**: PROJ-401 (Quick-Win Crate Optimizations) should complete first  
**Audit Target**: `rudof` crate (https://github.com/rudof-project/rudof)

---

## Overview

The `rudof` crate provides comprehensive Rust support for ShEx (Shape Expression Language), SHACL, DCTAP (Data Catalog Vocabulary), and transformations among RDF modeling formalisms. It is the most feature-rich Rust shapes library for Graphlaw's validation work.

This ticket authorizes a **focused audit** of `rudof` for:
- ShEx and SHACL Abstract Syntax Trees (ASTs)
- Shape maps and target selection
- Validation report structures
- Conversion logic between shape formalisms
- DCTAP-inspired constraint patterns (useful for manifest validation)
- Non-hot-path module adaptation candidates (AST definitions, report rendering, shape-map parsing)

**Constraint**: Do NOT adopt its full shape-validation engine into the hot path. AST patterns and data structures only; reimplement validation logic in bounded Graphlaw forms.

---

## Audit Scope

### 1. License & Maturity Assessment

- SPDX license identifier and compatibility
- Last commit and maintenance status
- Rust edition and dependency footprint
- Feature gates (ShEx, SHACL, DCTAP, conversion)
- Public API surface and stability guarantees

### 2. ShEx Architecture

- ShEx AST representation (Shape, Expression, Semantic action)
- ShEx Core vs. ShEx Full feature coverage
- Supported shape constructs (EachOf, OneOf, Closed, Extra, Virtual, Inverse)
- Semantic action handling (how are non-SPARQL actions handled?)
- Schema serialization (ShExJ/ShExC parser + output)

### 3. SHACL Architecture

- SHACL AST representation (Shape, Constraint, Component)
- SHACL Core vs. SHACL-SPARQL boundary
- Supported constraint components (cardinality, pattern, node kind, class, datatype, range, min/max, etc.)
- SPARQL constraint integration (or refusal boundary)
- SHACL-AF (Advanced Features) scope

### 4. Shape Maps & Target Selection

- Shape map data structure (node selector → shape)
- Shape map formats (SchemaMap, ShapeMap JSON, RDF)
- Target selection strategies (class-based, closed, open, reference)
- Validation scope (triple match vs. graph partition)

### 5. Validation Report Rendering

- Validation violation structure
- Report graph representation (RDF triples or other)
- Severity levels (error, warning, info)
- Conformation vs. non-conformation report shape
- Diagnostic details and remediation suggestions

### 6. Cross-Formalism Conversion

- ShEx ↔ SHACL conversion patterns
- Loss/gain in translation (what features don't convert?)
- DCTAP → ShEx/SHACL mapping
- Canonical shape representation (if any)

### 7. Test Fixtures & Conformance

- W3C ShEx/SHACL conformance test linkage
- Test oracle structure (expected vs. actual)
- Problem sizes and coverage
- Negative fixtures (intentional refusals)

---

## Audit Questions

| Question | Answer | Relevance |
|----------|--------|-----------|
| **License**: SPDX identifier and permissive/copyleft status? | | Determines ADAPT_CODE vs. ADAPT_IDEA |
| **ShEx AST**: How are shapes and expressions represented? | | Informs Graphlaw shape parsing |
| **SHACL AST**: Constraint component structure? | | Informs Graphlaw constraint evaluation |
| **Shape maps**: Data structure and serialization? | | Non-hot-path adaptation candidate |
| **Report rendering**: Can violations be rendered as RDF or JSON? | | Diagnostic artifact generation |
| **SHACL-SPARQL**: How is it handled? Full support or refusal? | | Informs Graphlaw's SHACL-SPARQL boundary |
| **Conversion**: ShEx ↔ SHACL conversion quality? | | Shape format bridging |
| **Test fixtures**: W3C conformance suite linkage? | | Potential fixture import source |

---

## Deliverables

### Audit Report (2,500–3,000 words)

**Sections:**

A. **Identity & License**
   - Crate name, version, SPDX license, maintainer activity
   - Feature gates and optional dependencies
   - Adaptation class: ADAPT_CODE / ADAPT_CODE_ISOLATED / ADAPT_IDEA / TEST_FIXTURE_ONLY / REFUSE

B. **ShEx Architecture & Coverage**
   - ShEx AST design (Shape, Expression, Semantic action)
   - Feature coverage (Core vs. Full, supported constructs)
   - Semantic action handling (SPARQL vs. other)
   - Parser/serialization (ShExC/ShExJ)

C. **SHACL Architecture & Coverage**
   - SHACL AST design (Shape, Constraint, Component)
   - Feature coverage (Core vs. SPARQL vs. Advanced Features)
   - Constraint component list and semantics
   - SPARQL constraint integration or refusal

D. **Shape Maps & Validation Scope**
   - Shape map representation and formats
   - Target selection strategies
   - Validation scope (triple match, graph partition, reference)
   - Closed/open shape handling

E. **Validation Report & Diagnostics**
   - Violation structure and severity levels
   - Report graph shape (RDF triples or other)
   - Conformation/non-conformation distinction
   - Diagnostic detail levels

F. **Cross-Formalism Conversion**
   - ShEx ↔ SHACL conversion patterns
   - Translation loss/gain analysis
   - DCTAP mapping capabilities
   - Canonical shape representation (if any)

G. **Test Fixtures & Conformance**
   - W3C test suite linkage
   - Test oracle structure
   - Problem sizes and coverage gaps
   - Negative fixtures

H. **Graphlaw Integration Opportunities**
   - **Hot path**: Which AST patterns/data structures inform Graphlaw's shape validation?
   - **Non-hot path**: Which modules can be directly adapted?
     - ShEx/SHACL AST types (ADAPT_CODE if compatible)
     - Shape map parser (ADAPT_IDEA or CLEAN_ROOM)
     - Validation report structure (ADAPT_CODE if compatible)
     - Conversion utilities (ADAPT_CODE if compatible)
     - Test fixtures (TEST_FIXTURE_ONLY)

I. **Recommendation & Risk**
   - Adaptation class and license basis
   - SHACL-SPARQL boundary for Graphlaw (UNSUPPORTED or FEDERATED)
   - Receipt/replay stability implications
   - Timeline for integration

---

## Non-Hot-Path Adaptation Candidates

If license-compatible, these modules are candidates for direct code adaptation with attribution and isolation:

| Module | Source Pattern | Use In Graphlaw | License Requirement |
|--------|---|---|---|
| ShEx/SHACL AST types | rudof AST definitions | `shapes_ast.rs` | Attribution + module isolation |
| Shape-map parser | shape-map JSON/RDF parser | `shape_map_parser.rs` | Attribution + tests |
| Validation report structure | rudof violation representation | `validation_report.rs` | Attribution + test coverage |
| Conversion utilities | ShEx ↔ SHACL conversion | `shape_conversion.rs` | License preservation |
| Test fixture harness | W3C conformance test layout | `shape_fixtures.rs` | Conform to test license |

---

## Acceptance Criteria

- [ ] Full audit report completed and committed
- [ ] License identified (SPDX) and adaptation class assigned
- [ ] ShEx AST architecture fully documented
- [ ] SHACL AST architecture fully documented
- [ ] Shape-map data structure and formats documented
- [ ] SHACL-SPARQL boundary clearly defined (support vs. refusal)
- [ ] Validation report rendering strategy assessed
- [ ] Cross-formalism conversion coverage analyzed
- [ ] Non-hot-path adaptation candidates identified
- [ ] Test fixture suitability evaluated
- [ ] Recommendations integrated into Graphlaw v26.7.8 planning

---

## Standing Rules

Mark **ALIVE** when:
- Audit report is written and accepted
- License and adaptation class are recorded
- ShEx and SHACL ASTs are fully documented
- Non-hot-path candidates are isolated and have test plans
- SHACL-SPARQL boundary is explicit
- Recommendations are actionable by implementation teams

---

## Related Tickets

- **PROJ-401**: Quick-Win Crate Optimizations — may reference rudof AST insights
- **PROJ-501**: OWL RL audit (sibling)
- **PROJ-503**: SHACL audit (sibling, may share findings)
- **PROJ-504**: N3 audit (sibling)
- **PROJ-307** (future): Shape validation expansion — will use this audit for AST/report structures

---

## References

- rudof crate: https://github.com/rudof-project/rudof
- W3C ShEx specification: https://shex.io/
- W3C SHACL specification: https://www.w3.org/TR/shacl/
- ShEx/SHACL test suite: https://github.com/w3c/shacl-test-suite

# PROJ-501 — OWL RL Audit & Adaptation (v26.7.8 P0)

**Status**: PLANNED  
**Scope**: Audit `reasonable` crate (Rust OWL 2 RL reasoner) for pattern/algorithm insights and non-hot-path code adaptation  
**Dependencies**: PROJ-401 (Quick-Win Crate Optimizations) should complete first  
**Audit Target**: `reasonable` crate (https://docs.rs/reasonable)

---

## Overview

The `reasonable` crate is a Rust OWL 2 RL reasoner implemented using Datalog/DataFrog-style reasoning. It is the most directly relevant Rust OWL RL implementation for Graphlaw's OWL RL profile support.

This ticket authorizes a **focused audit** of `reasonable` for:
- OWL RL rule encoding and identifiers
- Bounded profile detection and scanning
- Relation layout and Datalog translation
- Diagnostic codes for inconsistent/unsupported constructs
- Non-hot-path module adaptation candidates (OWL RL rule catalog, profile scanners, unsupported-feature classifiers)

**Constraint**: Do NOT adopt its reasoner into the hot path (materialization, receipts, replay) without separate benchmarking and receipt-stability gates. Hot-path algorithm/pattern ideas only; reimplement minimal bounded forms.

---

## Audit Scope

### 1. License & Maturity Assessment

- SPDX license identifier and compatibility (permissive/copyleft/incompatible/no license)
- Last commit and active maintenance status
- Maturity level (stable, evolving, experimental)
- Rust edition and dependency footprint
- Public test coverage and fixture strategy

### 2. OWL RL Rule Identification

- How does `reasonable` represent OWL RL rules? (Datalog syntax, RDF encoding, internal AST)
- Which OWL RL profile rules are implemented? (e.g., `rdfs:subClassOf` closure, `rdf:type` inference, domain/range, inverse properties)
- Which OWL RL rules are explicitly NOT supported?
- Rule-to-predicate mapping (which RDF predicates trigger which rules)
- Stratification and rule ordering discipline

### 3. Profile Detection & Scanning

- How does `reasonable` detect the OWL RL profile boundary?
- Does it reject full OWL constructs? How?
- Does it have a public API for "is this ontology OWL RL compliant?"
- What constructs are explicitly forbidden (e.g., `owl:sameAs` without bounds)?

### 4. Relation Layout & Materialization

- How does `reasonable` translate OWL axioms to Datalog relations?
- Does it use a standard Datalog syntax (N3, Datalog, other)?
- What is its triple/relation schema? (subject, predicate, object, provenance, etc.)
- Does it track rule provenance or derivation lineage?
- Is its output deterministically ordered?

### 5. Diagnostics & Unsupported Features

- What diagnostic codes does `reasonable` emit for unsupported constructs?
- Does it classify errors (e.g., "unsupported OWL Full", "cycle in TBox", "unbounded reasoner loop")?
- How does it handle inconsistencies (contradiction detection, reporting)?
- Can diagnostics be rendered as RDF triples or SHACL violation reports?

### 6. Test Fixtures & Conformance

- Does `reasonable` have test fixtures or conformance suites?
- Does it reference OWL 2 test cases (e.g., from W3C)?
- What problem sizes does it benchmark (ontology scale, rule count, inference iterations)?
- Are there negative fixtures (features it intentionally rejects)?

---

## Audit Questions

| Question | Answer | Relevance |
|----------|--------|-----------|
| **License**: What is the SPDX identifier? Is it permissive or copyleft? | | Determines ADAPT_CODE vs. ADAPT_IDEA |
| **Rule encoding**: How are OWL RL rules represented? | | Informs Graphlaw's rule compilation |
| **Profile boundary**: How does it detect/enforce OWL RL vs. full OWL? | | Critical for REFUSED feature list |
| **Stratification**: How does it handle rule ordering and negation? | | Informs Graphlaw's stratification |
| **Determinism**: Is output deterministically ordered? | | Required for Graphlaw receipts/replay |
| **Diagnostics**: Can unsupported features be classified and reported? | | Non-hot-path adaptation candidate |
| **Test fixtures**: Does it have W3C conformance tests? | | Potential fixture import source |
| **Bounded profile**: Does it support OWL RL only, or full OWL? | | Determines Graphlaw's scope boundary |

---

## Deliverables

### Audit Report (2,000–2,500 words)

**Sections:**

A. **Identity & License**
   - Crate name, version, SPDX license, maintainer activity
   - Adaptation class: ADAPT_CODE / ADAPT_CODE_ISOLATED / ADAPT_IDEA / TEST_FIXTURE_ONLY / REFUSE

B. **OWL RL Rule Architecture**
   - How rules are encoded (Datalog, N3, RDF, internal AST)
   - Complete list of implemented OWL RL rules
   - Rules NOT implemented
   - Stratification strategy

C. **Profile Boundary & Diagnostics**
   - OWL RL vs. full OWL detection
   - Unsupported construct handling
   - Diagnostic codes and error classification
   - Examples of rejected constructs

D. **Relation Layout & Materialization**
   - Datalog schema (relations, arity, semantics)
   - Output determinism guarantee
   - Provenance/derivation tracking (if present)
   - Triple ordering and receipt implications

E. **Test Fixtures & Conformance**
   - W3C conformance test linkage
   - Problem sizes and benchmarks
   - Negative fixtures (intentional refusals)

F. **Graphlaw Integration Opportunities**
   - **Hot path**: Which algorithms/patterns inform Graphlaw's bounded OWL RL?
   - **Non-hot path**: Which modules can be directly adapted?
     - OWL RL rule catalog (ADAPT_CODE if compatible license)
     - Profile scanner / unsupported-construct classifier (ADAPT_IDEA or CLEAN_ROOM)
     - Diagnostic rendering (ADAPT_CODE if compatible)
     - Test fixtures (TEST_FIXTURE_ONLY)

G. **Recommendation & Risk**
   - Adaptation class and license basis
   - Benchmark plan if hot-path algorithm adoption is considered
   - Receipt/replay stability requirements
   - Timeline for integration

---

## Non-Hot-Path Adaptation Candidates

If license-compatible, these modules are candidates for direct code adaptation with attribution and isolation:

| Module | Source Pattern | Use In Graphlaw | License Requirement |
|--------|---|---|---|
| OWL RL rule identifiers | reasonable rule compilation | `owl_rl_rules.rs` | Attribution + module isolation |
| Unsupported-feature classifier | diagnostic codes | `unsupported_features.rs` | Attribution + tests |
| Diagnostic renderer | error/warning messages | `owl_rl_diagnostics.rs` | Attribution + test coverage |
| Test fixture structure | W3C conformance cases | `owl_rl_fixtures.rs` | Conform to test license |

---

## Acceptance Criteria

- [ ] Full audit report completed and committed
- [ ] License identified (SPDX) and adaptation class assigned
- [ ] OWL RL rule list (implemented + not implemented) documented
- [ ] Profile boundary (OWL RL vs. full OWL) clearly defined
- [ ] Unsupported features enumerated
- [ ] Non-hot-path adaptation candidates identified
- [ ] Hot-path algorithm/pattern insights documented
- [ ] Receipt/replay stability implications assessed
- [ ] Test fixture suitability evaluated
- [ ] Recommendations integrated into Graphlaw v26.7.8 planning

---

## Standing Rules

Mark **ALIVE** when:
- Audit report is written and accepted
- License and adaptation class are recorded
- OWL RL rule architecture is fully documented
- Non-hot-path candidates are isolated and have test plans
- Recommendations are actionable by implementation teams

---

## Related Tickets

- **PROJ-401**: Quick-Win Crate Optimizations — may reference reasonable algorithm insights
- **PROJ-307** (future): OWL RL capability expansion — will use this audit for scope boundaries
- **PROJ-502**: ShEx/SHACL audit (sibling)
- **PROJ-503**: SHACL audit (sibling)
- **PROJ-504**: N3 audit (sibling)

---

## References

- reasonable crate: https://docs.rs/reasonable
- reasonable GitHub: https://github.com/ndtoan/reasonable (or current maintainer)
- W3C OWL 2 RL specification: https://www.w3.org/TR/owl2-profiles/#OWL_RL
- Datalog/RDF translation patterns: https://www.w3.org/TR/owl2-profiles/#Appendix:_Datalog-style_Inference

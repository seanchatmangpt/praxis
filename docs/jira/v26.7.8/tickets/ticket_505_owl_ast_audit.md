# PROJ-505 — OWL AST & Ontology Audit (v26.7.8 P1, Optional)

**Status**: PLANNED  
**Scope**: Audit `horned-owl` crate (Rust typed OWL AST) for pattern/algorithm insights and potential code adaptation  
**Dependencies**: PROJ-401 (Quick-Win Crate Optimizations) should complete first; PROJ-501 (OWL RL) may provide context  
**Audit Target**: `horned-owl` crate (https://docs.rs/horned-owl)

---

## Overview

The `horned-owl` crate provides a comprehensive typed Rust AST for OWL ontologies and parsing/serialization support. It is the most mature Rust OWL representation library.

This ticket authorizes an **optional focused audit** of `horned-owl` for:
- Typed OWL entity representation
- Ontology AST design (Axiom, Declaration, ObjectProperty, DataProperty, Class, etc.)
- OWL profile detection (RL, DL, Full)
- Axiom normalization and simplification
- Parsing and serialization (RDF/XML, Turtle, Functional syntax)
- Non-hot-path module adaptation candidates (OWL AST types, profile detector, serializers)

**Scope limitation**: Use only if Graphlaw's OWL RL support requires more sophisticated ontology representation than simple RDF triples + rule compilation. If OWL RL v0 needs only RDF triples + `reasonable` rule patterns, this audit is **DEFER until PROJ-501 determines the need**.

---

## Audit Scope

### 1. License & Maturity Assessment

- SPDX license identifier
- Last commit and maintenance status
- Rust edition and dependency footprint
- Feature gates (if any)
- Test coverage

### 2. OWL AST Design

- Ontology structure (Imports, Annotations, Axioms)
- Entity types (Class, ObjectProperty, DataProperty, AnnotationProperty, Individual)
- Type system (how are OWL constructs strongly typed?)
- Axiom representation (SubClassOf, SubPropertyOf, ObjectPropertyDomain, etc.)
- Expression types (ClassExpression, ObjectPropertyExpression, DataRange)

### 3. OWL Profile Detection

- How are OWL profiles identified? (RL, DL, Full)
- Profile detection heuristics
- Unsupported construct classification (by profile)
- Public API for profile querying

### 4. Axiom Normalization

- How are axioms simplified or normalized?
- Canonical form (if any)
- Equivalence-preserving transformations
- Variable renaming and consistency checking

### 5. Parsing & Serialization

- Input formats (RDF/XML, Turtle, OWL Functional syntax, OWL Manchester syntax)
- Parser robustness (error recovery, validation)
- Serialization formats (same as input)
- Round-trip fidelity (parse → AST → serialize = original or equivalent?)

### 6. Test Fixtures & Conformance

- OWL test-suite linkage (W3C)
- Problem sizes and complexity
- Negative fixtures (invalid OWL)
- Edge cases

---

## Audit Questions

| Question | Answer | Relevance |
|----------|--------|-----------|
| **License**: SPDX identifier and permissive status? | | Determines ADAPT_CODE vs. ADAPT_IDEA |
| **AST design**: How are OWL entities strongly typed? | | Informs Graphlaw's ontology representation |
| **Profile detection**: Can it detect OWL RL? | | Bridges PROJ-501 OWL RL rule detection |
| **Axiom normalization**: Does it simplify axioms? | | Preprocessing candidate |
| **Parsing**: Input format support and robustness? | | Ontology loading strategy |
| **Serialization**: Can output match input format? | | Ontology export strategy |
| **Test fixtures**: W3C OWL test suite linkage? | | Fixture import candidate |

---

## Deliverables

### Audit Report (1,500–2,000 words)

**Sections:**

A. **Identity & License**
   - Crate name, version, SPDX license, maintainer activity
   - Maturity assessment
   - Adaptation class: ADAPT_CODE / ADAPT_CODE_ISOLATED / ADAPT_IDEA / TEST_FIXTURE_ONLY / REFUSE

B. **OWL AST Design**
   - Ontology structure and entity types
   - Type system design (how OWL constructs are modeled)
   - Axiom representation
   - Expression types and composition

C. **OWL Profile Support**
   - Profile detection capability (RL, DL, Full)
   - Unsupported construct classification
   - Profile-specific validation (if any)

D. **Axiom Normalization & Simplification**
   - Normalization strategy (if any)
   - Canonical forms
   - Equivalence-preserving transformations

E. **Parsing & Serialization**
   - Input format support (RDF/XML, Turtle, Functional, Manchester)
   - Parser robustness and error handling
   - Serialization fidelity (round-trip equivalence)
   - Performance characteristics

F. **Graphlaw Integration Opportunities**
   - **Conditional use**: When would Graphlaw need this?
   - **If needed**: Which modules could be directly adapted?
     - OWL AST types (ADAPT_CODE if compatible, otherwise CLEAN_ROOM)
     - Profile detector (ADAPT_IDEA or CLEAN_ROOM)
     - Serializers (ADAPT_CODE if compatible)
   - **If not needed**: What does Graphlaw use instead? (RDF triples + rule compilation)

G. **Recommendation & Risk**
   - Adaptation class and license basis
   - Feasibility of integration with PROJ-501 OWL RL work
   - When this audit becomes relevant (now vs. future)
   - Timeline if needed

---

## Conditional Acceptance Criteria

- [ ] Determine whether PROJ-501 (OWL RL) needs horned-owl (defer vs. proceed)
- [ ] If PROCEED:
  - [ ] Full audit report completed and committed
  - [ ] License identified (SPDX) and adaptation class assigned
  - [ ] OWL AST design documented
  - [ ] Profile detection capability assessed
  - [ ] Parsing/serialization strategy understood
  - [ ] Non-hot-path adaptation candidates identified
  - [ ] Integration path with PROJ-501 defined
  - [ ] Recommendations integrated into Graphlaw planning
- [ ] If DEFER:
  - [ ] Audit marked as CONDITIONAL on PROJ-501 findings
  - [ ] Re-assess after PROJ-501 completes

---

## Standing Rules

Mark **DEFERRED** when:
- PROJ-501 determines that RDF triples + reasonable rule patterns suffice (horned-owl not needed now)
- Requirements may change in future OWL expansion phases

Mark **ALIVE** when:
- PROJ-501 identifies a need for sophisticated OWL representation beyond RDF triples
- horned-owl is confirmed as the implementation partner
- Integration path with PROJ-501 is defined

---

## Related Tickets

- **PROJ-401**: Quick-Win Crate Optimizations — may reference horned-owl if AST patterns apply
- **PROJ-501**: OWL RL audit — will determine whether this ticket becomes active
- **PROJ-307** (future): OWL expansion — will use this audit if/when OWL DL or Full support is needed

---

## References

- horned-owl crate: https://docs.rs/horned-owl
- horned-owl GitHub: https://github.com/phillord/horned-owl
- W3C OWL 2 specification: https://www.w3.org/TR/owl2-overview/
- W3C OWL test suite: https://www.w3.org/TR/owl2-test/

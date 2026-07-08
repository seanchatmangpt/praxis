# PROJ-504 — N3 Reasoning Audit & Adaptation (v26.7.8 P0)

**Status**: PLANNED  
**Scope**: Audit Rust N3 reasoner crates (`oxirs-ttl::n3`, `eyeron`) for pattern/algorithm insights and non-hot-path code adaptation  
**Dependencies**: PROJ-401 (Quick-Win Crate Optimizations) should complete first  
**Audit Targets**:
  - `oxirs-ttl::n3` (https://docs.rs/oxirs-ttl) — N3 parsing, backward chaining, proof tracing, built-ins
  - `eyeron` (https://github.com/eyereasoner) — Core N3 reasoner with proof traces and denial handling

---

## Overview

Two complementary Rust N3 implementations provide different angles for Graphlaw:

- **`oxirs-ttl::n3`**: N3 parser, reasoning primitives, backward/forward chaining split, built-in boundary, proof tracing (AST and algorithm patterns)
- **`eyeron`**: Core N3 reasoner with loop control, denial detection, proof trace representation (algorithm reference, possible maturity assessment)

This ticket authorizes a **focused audit** of both for:
- N3 rule parsing and syntax handling
- Forward vs. backward chaining separation
- Proof trace representation and semantics
- Denial (negation) diagnostics and handling
- Built-in predicate boundary and classification
- Loop control and termination strategies
- Non-hot-path module adaptation candidates (proof trace structures, built-in lists, diagnostic rendering)

**Constraint**: Do NOT adopt unbounded N3 built-ins into the hot path. Core rule parsing, proof trace structures, and bounded semantics only; reimplement in Graphlaw's closed-vocabulary forms.

---

## Audit Scope

### 1. License & Maturity Assessment

- SPDX license identifiers
- Last commit and maintenance status
- Rust edition and dependency footprint
- Feature gates and optional dependencies
- Public test coverage

### 2. N3 Parsing & Rule Representation

- N3 rule syntax support (N3 Core rules, Datalog-like rules)
- Parsing strategy (hand-written, parser generator, other)
- Rule AST representation (Head, Body, Variable binding)
- Quantifier handling (universal, existential)
- Implication vs. assertion distinction
- Built-in predicate integration in parsing

### 3. Forward vs. Backward Chaining Architecture

- How is the forward/backward distinction made?
- Forward chaining: Datalog-style materialization path
- Backward chaining: Goal-driven query evaluation path
- Strategy selection (is it explicit or implicit?)
- Proof trace direction (forward or backward)
- Loop detection and termination

### 4. Proof Trace Representation

- How are proofs represented? (tree, DAG, linear, other)
- Proof node structure (rule applied, bindings, conclusion)
- Proof serialization (RDF, JSON, text, other)
- Explanation rendering (how is proof shown to users?)
- N3 proof format compatibility (if applicable)

### 5. Denial & Negation Handling

- Negation as failure (NAF) implementation
- Denial detection (explicit `false` conclusions)
- Denial rendering as diagnostic
- Interaction with proof traces (how are denials explained?)
- Stratification or loop-avoidance strategy

### 6. Built-In Predicates

- Complete list of supported built-ins (math, string, list, etc.)
- Built-in classification (deterministic, nondeterministic, side-effect, etc.)
- Semantic action handling
- Refusal boundary (which built-ins are NOT supported?)
- Custom built-in registration (if supported)

### 7. Loop Control & Termination

- How is infinite looping detected?
- Occurs check or equivalent (variable occurs in term)
- Rule cycle detection
- Tabling/memoization strategy (if used)
- Bounded depth or resource limits

### 8. Test Fixtures & Conformance

- N3 test suite linkage (W3C)
- Problem sizes and complexity
- Negative fixtures (intentional refusals)
- Edge cases (circular rules, unbounded quantifiers, etc.)

---

## Audit Questions

| Question | Answer | Relevance |
|----------|--------|-----------|
| **Licenses**: SPDX IDs for oxirs-ttl and eyeron? | | Determines ADAPT_CODE vs. ADAPT_IDEA |
| **N3 syntax**: Which N3 constructs are supported? | | Informs Graphlaw's N3 scope |
| **Forward/backward**: How are these separated? | | Hot-path reasoning strategy reference |
| **Proof traces**: Data structure and serialization? | | Non-hot-path diagnostic candidate |
| **Denial handling**: How is negation-as-failure implemented? | | Refusal and diagnostic handling |
| **Built-ins**: Complete list and refusal boundary? | | Critical for Graphlaw's closed vocabulary |
| **Loop control**: How is termination guaranteed? | | Receipt/replay stability |
| **Test fixtures**: W3C N3 test suite linkage? | | Fixture import candidate |

---

## Deliverables

### Audit Report (2,500–3,000 words)

**Sections:**

A. **Identity & Licenses**
   - Two crate names, versions, SPDX licenses, maintainer activity
   - Maturity assessment (stable, evolving, experimental)
   - Adaptation classes: ADAPT_CODE / ADAPT_CODE_ISOLATED / ADAPT_IDEA / TEST_FIXTURE_ONLY / REFUSE
   - Dependency footprint

B. **N3 Parsing & Rule Representation**
   - N3 syntax coverage (Core, extended rules)
   - Parsing strategy and error handling
   - Rule AST design (Head, Body, Variables)
   - Quantifier representation (universal ∀, existential ∃)
   - Implication vs. assertion semantics
   - Built-in predicate integration in parser

C. **Forward vs. Backward Chaining Architecture**
   - How is the forward/backward distinction made?
   - Forward chaining implementation (materialization strategy)
   - Backward chaining implementation (goal-driven search)
   - Strategy selection heuristics (if applicable)
   - Proof trace direction and construction
   - Loop control (occurs check, cycle detection, tabling)

D. **Proof Trace Representation & Rendering**
   - Proof structure (tree, DAG, linear, other)
   - Proof node representation
   - Proof serialization format (RDF triples, JSON, text)
   - Explanation rendering for user consumption
   - Proof depth and size constraints

E. **Denial & Negation Handling**
   - Negation as failure (NAF) semantics
   - Denial detection (explicit `false` conclusions)
   - Denial diagnostics and explanation
   - Interaction with proof traces
   - Stratification or well-founded semantics

F. **Built-In Predicates & Semantic Actions**
   - Complete enumeration of supported built-ins
   - Built-in classification (deterministic, nondeterministic, I/O)
   - Semantic action handling (if supported)
   - Refusal boundary (NOT supported)
   - Custom built-in registration capability

G. **Termination Strategies**
   - Loop detection techniques (occurs check, cycle detection, tabling)
   - Bounded depth or resource limits
   - Infinite loop handling and reporting
   - Soundness and completeness guarantees

H. **Test Fixtures & Conformance**
   - W3C N3 test suite linkage
   - Problem sizes and complexity tiers
   - Negative fixtures (intentional failures)
   - Edge cases (circular rules, unbounded quantifiers)

I. **Graphlaw Integration Opportunities**
   - **Hot path**: Which N3 algorithms inform Graphlaw's reasoning?
   - **Non-hot path**: Which modules can be directly adapted?
     - N3 rule parser (ADAPT_IDEA or CLEAN_ROOM)
     - Proof trace structure (ADAPT_CODE if compatible)
     - Built-in boundary list (ADAPT_CODE if compatible)
     - Denial/negation diagnostics (ADAPT_CODE if compatible)
     - Test fixtures (TEST_FIXTURE_ONLY)

J. **Recommendation & Risk**
   - Adaptation classes and license basis
   - N3 dialect scope for Graphlaw (Core vs. Extended)
   - Built-in refusal list for Graphlaw
   - Receipt/replay stability implications
   - Loop control strategy recommendation
   - Timeline for integration

---

## Non-Hot-Path Adaptation Candidates

If license-compatible, these modules are candidates for direct code adaptation with attribution and isolation:

| Module | Source | Use In Graphlaw | License Requirement |
|--------|--------|---|---|
| Proof trace structure | oxirs-ttl or eyeron | `n3_proof_trace.rs` | Attribution + module isolation |
| Built-in predicate list | oxirs-ttl | `n3_builtins.rs` | Attribution + boundary tests |
| Denial diagnostics | oxirs-ttl or eyeron | `n3_denial_diagnostics.rs` | Attribution + test coverage |
| N3 parser patterns | oxirs-ttl | Pattern guidance (ADAPT_IDEA) | No code copy if incompatible |
| Test fixture harness | W3C test suite | `n3_fixtures.rs` | Conform to test license |

---

## Acceptance Criteria

- [ ] Full audit report completed and committed
- [ ] Licenses identified (SPDX) and adaptation classes assigned
- [ ] N3 syntax support fully documented
- [ ] Forward vs. backward chaining architecture analyzed
- [ ] Proof trace representation and serialization documented
- [ ] Denial/negation handling semantics recorded
- [ ] Complete built-in predicate list enumerated
- [ ] Refusal boundary (NOT supported built-ins) documented
- [ ] Loop control and termination strategies analyzed
- [ ] Test fixture suitability evaluated
- [ ] Non-hot-path adaptation candidates identified
- [ ] N3 dialect scope decision (Core vs. Extended) made
- [ ] Built-in refusal list for Graphlaw defined
- [ ] Recommendations integrated into Graphlaw v26.7.8 planning

---

## Standing Rules

Mark **ALIVE** when:
- Audit report is written and accepted
- Licenses and adaptation classes are recorded for both crates
- N3 syntax support is fully documented
- Proof trace architecture is understood
- Built-in predicate boundary is explicit
- Denial/negation handling is documented
- Non-hot-path candidates are isolated and have test plans
- Recommendations are actionable by implementation teams

---

## Related Tickets

- **PROJ-401**: Quick-Win Crate Optimizations — may reference N3 reasoning insights
- **PROJ-501**: OWL RL audit (sibling)
- **PROJ-502**: ShEx/SHACL audit (sibling)
- **PROJ-503**: SHACL audit (sibling)
- **PROJ-307** (future): N3 reasoning expansion — will use this audit for proof traces, denial handling, and built-in boundaries

---

## References

- oxirs-ttl crate: https://docs.rs/oxirs-ttl
- eyeron project: https://github.com/eyereasoner
- W3C N3 specification: https://w3c.github.io/N3/spec/
- W3C N3 test suite: https://github.com/w3c/N3-test-suite

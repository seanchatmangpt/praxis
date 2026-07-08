# PROJ-402 — Datafrog Audit & Learning (v26.7.8 P1)

**Status**: PLANNED  
**Scope**: Technical audit of datafrog for algorithm insights; suitability as benchmark reference and future backend exploration  
**Dependencies**: PROJ-401 (Quick-Win Crate Optimizations) should complete first  
**Output**: Technical audit report + recommendations for PROJ-401 sub-tasks + benchmark methodology  

---

## Overview

Datafrog is a lightweight, embeddable Rust Datalog engine designed for simplicity and understandability. While not suitable as a drop-in backend for praxis (due to missing receipt tracking, hook semantics, and stratification), it provides **high-signal algorithmic patterns** that could improve GL's join efficiency, deduplication cost, and closure computation post-PROJ-401.

This ticket authorizes a **technical audit** of datafrog's architecture, algorithms, and performance characteristics, with focus on **concrete optimization opportunities** for praxis-graphlaw.

---

## Key Findings (Executive Summary)

### What Datafrog Does Well

| Algorithm | Technique | Praxis Applicability |
|-----------|-----------|---------------------|
| **Join** | Gallop search (exponential binary search on sorted lists) | O(log n) join per advance on clustered keys (RDF triples group by subject/predicate) |
| **Deduplication** | Log-structured batching (each batch ~2× prior size) | O(n log n) aggregate dedup vs. O(n²) full merges |
| **Multi-way joins** | Treefrog leapjoin (propose from smallest set, intersect others) | Avoids intermediate Cartesian product materialization for 3+ patterns |
| **Delta evaluation** | Explicit (recent, stable) fact tracking | Formalize semi-naive to track "new" vs. "old" facts |

### What Datafrog Deliberately Refuses

- **No negation as failure** (requires external stratification — praxis handles this)
- **No aggregation** (requires rule manipulation — praxis has explicit Aggregate support)
- **No choice points** (deterministic rule heads only)
- **No query language** (hand-write Rust code)
- **No built-in scheduler** (user controls iteration)

### Unsuitable as Full Backend

Datafrog cannot replace praxis's Datalog engine due to:
- **No receipt tracking** (praxis Invariant #2: BLAKE3 receipts on all derivations)
- **No hook semantics** (praxis requires "when-derived" callbacks)
- **No vocabulary tracking** (praxis Invariant #4: closed predicate sets)
- **No automatic stratification** (though external stratification works)

### Suitable as Algorithmic Reference

Datafrog's **patterns are directly applicable** to praxis internals post-PROJ-401, especially join optimization and efficient deduplication.

---

## High-Signal Opportunities for Praxis (Post-PROJ-401)

### 1. Gallop Algorithm for Indexed Joins

**What**: O(log n) exponential binary search on sorted lists.  
**Where**: `tripleindex.rs` join paths when matching triple patterns.  
**When**: After PROJ-401 (requires sorted symbol ID triples).  
**Expected win**: 2-5× faster joins on clustered data (typical RDF distribution).  

**Action**: Profile current tripleindex join performance; measure improvement with gallop vs. linear search.

### 2. Logarithmic Batch Structure for Deduplication

**What**: Store derived facts in log-structured batches (each ~2× prior size) rather than full merge per iteration.  
**Where**: `Reasoner::materialize` deduplication loop.  
**When**: After PROJ-401 (ID triples make dedup cheaper).  
**Expected win**: Cuts deduplication from O(n²) to O(n log n) on deep graphs.  

**Action**: Measure dedup cost on `n3_chain_depth_400` before/after batching.

### 3. Delta Iteration Formalization

**What**: Explicitly structure rule application as (recent, stable) pairs; apply rules only to new facts + old facts, not all facts.  
**Where**: `Reasoner::materialize` rule application loop.  
**When**: Post-PROJ-401 analysis phase.  
**Expected win**: Reduces rule iterations by ~30-50% on stable graphs.  

**Action**: Formalize rule application to track "recently derived" predicates; avoid re-examining stable-only rules.

### 4. Multi-Way Join via Leapjoin

**What**: For Datalog rule bodies with 3+ triple patterns, use treefrog leapjoin instead of sequential binary joins.  
**Where**: `reasoner/mod.rs` when translating rules with multiple body patterns.  
**When**: Post-PROJ-401 optimization phase (separate sub-task).  
**Expected win**: Avoids intermediate materialization; 2-3× faster on complex rules.  

**Example**: Rule with 3 patterns avoids creating intermediate triples:
```datalog
?x rdf:type ?range :-
  ?subject ?predicate ?object ,      # Pattern 1
  ?predicate rdfs:range ?range ,     # Pattern 2
  ?object a ?type ,                  # Pattern 3
  ?type rdfs:subClassOf ?range .     # Pattern 4
```

Current: join(P1, P2) → J12, join(J12, P3) → J123, join(J123, P4) → result  
With leapjoin: propose from smallest, intersect others; one pass.

### 5. Closure Analysis for Hook Eligibility

**What**: Precompute which predicates appear in rule bodies/heads to build hook eligibility sets.  
**Where**: `hooks.rs` hook evaluation path.  
**When**: After PROJ-401 (with symbol interning).  
**Expected win**: O(1) hook dispatch vs. O(n) scan for eligibility.  

**Action**: Add predicate dependency analysis during hook pack loading; cache eligible hooks by (subject_predicate, object_predicate) pair.

### 6. Cache-Friendly Symbol Sorting

**What**: After interning to u64 IDs, sort join keys by integer value (cache-friendly) instead of term objects (pointer chasing).  
**Where**: All join, sort, and merge operations in tripleindex.  
**When**: Post-PROJ-401 Phase 2 (ID triple conversion).  
**Expected win**: Reduced cache misses; 10-20% throughput gain on large graphs.  

---

## Suitability as Benchmark Comparator

### Suitable Comparisons

- **Join efficiency**: Sorted-list join speed vs. praxis tripleindex joins
- **Deduplication cost**: Merge + sort + dedup overhead per iteration
- **Iteration count**: How many passes to reach fixpoint
- **Rule application overhead**: Cost per rule per fact

### Not Suitable

- **RDF reasoning** (RDFS, OWL) — datafrog has no type hierarchy
- **Receipt/replay** — datafrog does not track provenance
- **Hook semantics** — datafrog has no "when-derived" callbacks
- **Negation performance** — datafrog explicitly bans negation

### Benchmark Methodology

**Hybrid comparison approach**:

1. **Translate subset**: Select 2-3 problems from praxis test suite (e.g., closure rules, property chains)
2. **Datafrog version**: Hand-write using datafrog sorted-list joins
3. **Praxis version**: Current implementation
4. **Measure on same data**: 10k-100k triple set
5. **Metrics**: Iterations to fixpoint, wall-clock time per iteration, memory peak
6. **Interpretation**: If datafrog is faster, investigate which algorithm (gallop, leapjoin, batching) accounts for difference

---

## Future Backend Feasibility

### Could Datafrog Replace Praxis's Datalog Engine?

**Answer: No.** The contract differences are fundamental.

| Requirement | Datafrog | Praxis | Status |
|---|---|---|---|
| Receipt tracking (BLAKE3) | ❌ None | ✓ All derivations | Breaking mismatch |
| Hook semantics | ❌ Static rules | ✓ When-derived callbacks | Breaking mismatch |
| Vocabulary enforcement | ❌ No tracking | ✓ Closed predicates (Inv. #4) | Breaking mismatch |
| Negation/Aggregation | ❌ Banned | ✓ Stratified support | Incompatible |
| Automatic stratification | ❌ Manual/external | ✓ Bellman-Ford | Different approach |

### Viable Hybrid Approach

**Instead of full replacement, adopt algorithms**:

1. **Port datafrog algorithms**: gallop, leapjoin, log-structured batching
2. **Keep praxis engine intact**: stratification, receipt tracking, hooks, vocabulary checks
3. **Benchmark comparison**: Measure datafrog on pure-Datalog subset as performance reference

This avoids rewriting core logic while gaining algorithmic improvements.

### Optional Backend Contract (Theoretical)

If praxis supported an optional datafrog-backed evaluation path:

```
Scope: Positive-only Datalog (no negation, no aggregation)
Input: Stratified rule set + initial facts + hook triggers
Output: Final facts + receipts + hook events

Datafrog would:
  - Enforce @Ord on facts (u64 symbol IDs post-PROJ-401)
  - Track derivation path for receipts
  - Emit hook events on predicate derivation
  - Return facts + receipts

Praxis would:
  - Validate rules pre-execution
  - Handle stratification
  - Integrate results into knowledge graph
  - Dispatch hook callbacks
```

**Prerequisites for this contract:**
1. PROJ-401 symbol interning (ID triples)
2. Receipt tracking wrapper around datafrog operations
3. Hook dispatch instrumentation in leapjoin
4. Vocabulary validation pre-execution
5. Benchmarking confirmation that hybrid approach is faster

**Status**: Not authorized for v26.7.8; marks future exploration post-v26.7.9.

---

## Deliverables

### Audit Report (Complete)

**File**: `docs/v26.7.8/DATAFROG_AUDIT.md` (1800+ words)

Contents:
- **A. Architecture Overview**: relations, variables, lifecycle stages, execution model
- **B. Core Algorithms**: gallop search, treefrog leapjoin, rule ordering, stratification
- **C. Data Structures**: Relation<T>, Variable<T>, lifecycle example
- **D. Benchmarks & Performance**: provided benchmarks, performance characteristics, measurement approach
- **E. Design Constraints**: explicitly not supported (NAF, aggregation, choice points, query language), pragmatic constraints (sorting, termination, cycles)
- **F. High-Signal Learnings for Praxis**: 6 concrete optimization opportunities with applicability and action items
- **G. Benchmark Comparator Suitability**: suitable and unsuitable comparisons, hybrid methodology
- **H. Future Backend Feasibility**: barriers, hybrid approach, optional backend contract

### Recommendations for PROJ-401 Sub-Tasks

- **Sub-Task 2 (ID Triples)**: Post-completion, consider gallop algorithm for join paths
- **Sub-Task 4 (FixedBitSet Closure)**: Adopt log-structured batching in closure deduplication
- **Post-PROJ-401 Phase 1**: Add leapjoin for rule bodies with 3+ patterns
- **Hook Optimization (PROJ-307 follow-up)**: Use closure analysis for hook eligibility sets

### Benchmark Plan

- **Methodology**: Hybrid comparison (translate 2-3 praxis problems to datafrog, run on same data)
- **Metrics**: Iterations to fixpoint, wall-clock time, memory peak
- **Interpretation guide**: Which algorithms (gallop, leapjoin, batching) explain performance differences

---

## Acceptance Criteria

- [ ] Audit report complete and committed to `docs/v26.7.8/`
- [ ] High-signal learnings (6 items) documented with concrete actions
- [ ] Benchmark methodology defined and included in standing/PERFORMANCE_REPORT.md
- [ ] Backend feasibility assessed (conclusion: not viable as full replacement, viable as algorithm reference)
- [ ] Recommendations integrated into PROJ-401 sub-task descriptions
- [ ] Report suitable for reference during PROJ-401 implementation phases

---

## Standing Rules

This ticket is informational and exploratory. Mark **ALIVE** when:
- Audit report is written and accepted
- Concrete learning opportunities are documented with code locations and expected wins
- Benchmark methodology is documented
- Recommendations are actionable by PROJ-401 implementers

---

## Related Tickets

- **PROJ-401**: Quick-Win Rust Crate Optimizations — should reference this audit during implementation phases
- **PROJ-307** (future): Hook optimization — will reference closure analysis findings
- **v26.7.9** (future): Optional Datalog backend exploration — would use hybrid approach findings

---

## References

- Datafrog repository: https://github.com/frankmcsherry/datafrog
- Frank McSherry's Datalog post: https://github.com/frankmcsherry/blog/blob/master/posts/2018-06-24.md
- Treefrog leapjoin paper: McSherry et al., "Scalable Datalog Queries on In-Memory Graphs" (VLDB 2016)
- Full audit report: `docs/v26.7.8/DATAFROG_AUDIT.md`

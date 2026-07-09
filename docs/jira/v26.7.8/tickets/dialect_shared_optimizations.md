# Dialect Shared Optimization Abstractions — v26.7.8

This document outlines the design of shared performance and semantic abstractions across the different reasoning and validation dialects in Graphlaw (Datalog, Notation3 (N3), SHACL, and ShEx).

---

## Prerequisite Repair Tickets (Out of Scope for Abstractions)

These issues are compilation blockers in the workspace and must be resolved separately before any of the abstraction designs can be implemented or verified.

### Blocker 1: Upstream dependency `sparesults-0.3.3` compilation failure on `oxrdf::TermRef::Triple`
* **Status**: `BLOCKED_BY_COMPILER`
* **Context**: Newer registry versions of `oxrdf` introduce the `Triple` variant, which the compiler-pinned version of `sparesults-0.3.3` does not exhaustively match.
* **Repair Action**: Explicitly pin the `oxrdf` dependency in the workspace and `praxis-graphlaw` `Cargo.toml` to `=0.3.2`.

### Blocker 2: `case_study_judge` binary compilation failure on `materialize()` return type
* **Status**: `BLOCKED_BY_COMPILER`
* **Context**: `TripleStore::materialize_owlrl()` and related materialization methods now return a `Result<(Vec<Triple>, ScanReport), String>` tuple rather than a raw collection/unit, causing compilation errors when calling `.len()` on the result.
* **Repair Action**: Update `src/bin/case_study_judge.rs` to correctly unwrap or propagate the `Result` and destructure the returned tuple to extract the derived triple vector.

---

## Executive Summary

Graphlaw's performance is currently bound by duplicated graph traversal, matching, and validation logic across Datalog, N3, SHACL, and ShEx. Rather than building isolated optimization layers within each engine, this planning pass designs a unified substrate. By generalizing symbols, indexes, query planners, closures, caches, and diagnostics, we target order-of-magnitude efficiency gains while strictly preserving dialect-specific admission/refusal boundaries.

---

## Current Duplication Map

The table below catalogs duplicated patterns identified across the current dialect implementations:

| Duplicated Pattern / Operation | Datalog (`datalog.rs`) | N3 (`rule.rs`) | SHACL (`shacl/`) | ShEx (`shex.rs`, `shex_native.rs`) |
|:---|:---|:---|:---|:---|
| **Graph Triples Traversal** | Linear scans / indices | TripleIndex scans | `index_utils` helpers | Pattern matching |
| **Recursive Cycle Detection** | Stratification check | Loop detection | Visited node/shape map | Visited node/shape label map |
| **Predicate Dispatch** | String matching | String matching | String matching | String matching |
| **Transitive Closures** | Dynamic rules | Rule chaining | Subclass matrix | Custom recursions |
| **Diagnostics / Error Logs** | Flat strings | Flat strings | RDF report triples | Flat string vectors |

---

## Shared Abstraction Map

The target outcome after implementing these abstractions: **ALIVE, pending verification.**

| Abstraction | Category | Dialects | Proposed Shared Module | Target Status |
|:---|:---|:---|:---|:---|
| Predicate Opcode Dispatch | Common Index & Storage | All | `encoding.rs`, `lib.rs` | `READY_FOR_IMPLEMENTATION` |
| Typestate Phase Boundaries | Common Index & Storage | All | `lib.rs` | `NEEDS_MORE_ANALYSIS` |
| `TripleQuery` & Snapshots | Common Index & Storage | All | `tripleindex.rs` | `READY_FOR_IMPLEMENTATION` |
| Join & Delta Solvers | Unified Query Solver | All | `rule.rs`, `solver/` | `NEEDS_DESIGN_SPIKE` |
| Cache & Query Plan Cache | Common Caching | All | `cache.rs` | `READY_FOR_IMPLEMENTATION` |
| Union-Find & Closures | Common Caching | All | `closure.rs` | `READY_FOR_IMPLEMENTATION` |
| Diagnostics & Validation | Shared Diagnostics | All | `diagnostics.rs` | `READY_FOR_IMPLEMENTATION` |
| Dirty-Region Revalidation | Shared Diagnostics | All | `validation/` | `NEEDS_DEPENDENCY_GRAPH_DESIGN` |

---

## Tickets

### PROJ-520A: Predicate Opcode Dispatch
* **Title**: Static Predicate Opcode Table for O(1) Predicate Dispatch
* **Status**: `READY_FOR_IMPLEMENTATION`
* **Scope**: Compile well-known semantic predicates into a static opcode lookup table to eliminate string-hashing and interning lookups in hot execution paths.
* **Current duplicated surface**: Multiple parts of the reasoner and shape validators repeatedly check predicates against string values or interned symbols (e.g. `rdf:type`, `rdfs:subClassOf`, `sh:property`).
* **Proposed shared abstraction**:
  - Compile well-known predicates into a static, u16-backed enum `PredicateOpcode` (e.g. `Type = 1`, `SubClassOf = 2`, `SubPropertyOf = 3`, `Path = 4`).
  - Update matching loops and indices to dispatch directly on `PredicateOpcode` values during query routing.
* **Dialects affected**: Datalog, N3, SHACL, ShEx.
* **Complexity class improvement**: Reduces predicate identification from O(hash_len) string hashes or dynamic lookup to O(1) integer comparison.
* **Correctness risks**: Incorrect handling of custom or user-defined predicates that do not have statically assigned opcodes.
* **Verification plan**: Verify that unknown or custom IRIs safely bypass opcode checking and use dynamic symbol interning without causing execution crashes or incorrect matching.
* **Benchmark plan**: Compare lookup times of core properties (e.g., `rdf:type`) using static opcodes vs. dynamic string lookup.
* **Explicit exclusions**: Custom user-defined predicates are out of scope.

---

### PROJ-520B: Typestate Phase Boundaries
* **Title**: Typestate Phase Boundaries for Pipeline Graph States
* **Status**: `NEEDS_MORE_ANALYSIS`
* **Scope**: Formally define compiler-like state transitions for the main graph store to restrict operations based on current compilation and materialization stages.
* **Current duplicated surface**: Codebase allows modifications and queries to the graph state at any execution step, risking inconsistent reads and mutations.
* **Proposed shared abstraction**:
  - Implement a generic `GraphState<State>` wrapper using compiler typestates:
    - `GraphState<Raw>`: Triples loaded from files, unindexed.
    - `GraphState<Indexed>`: Load into `TripleIndex` complete.
    - `GraphState<Compiled>`: Parsing and compilation of rules/shapes to Symbol IDs complete.
    - `GraphState<Materialized>`: Reasoning fixpoint reached, ready for validation.
    - `GraphState<Validated>`: Constraint checks completed.
* **Dialects affected**: Datalog, N3, SHACL, ShEx.
* **Complexity class improvement**: Prevents redundant validations and redundant index rebuilds, achieving O(1) pipeline state enforcement.
* **Correctness risks**: Wide API changes that can break existing integrations and test harnesses if the typestate boundary is too restrictive.
* **Verification plan**: Compile-time check of graph operations to verify state transition constraints are correctly enforced.
* **Benchmark plan**: Ensure that typestate tracking introduces zero runtime overhead.
* **Explicit exclusions**: Full API-wide refactoring of the `praxis` workspace is excluded.

---

### PROJ-521: Zero-Allocation TripleQuery and Snapshots
* **Title**: Zero-Allocation `TripleQuery` and Immutable Graph Snapshots
* **Status**: `READY_FOR_IMPLEMENTATION`
* **Scope**: Build zero-allocation index query interfaces and implement immutable graph snapshots for safe multi-agent execution.
* **Current duplicated surface**: Traversal helpers in SHACL validation and query patterns allocate temporary vectors to return matched triples.
* **Proposed shared abstraction**:
  1. **`TripleQuery` Visitor/Enum Iterator**: Expose non-allocating query interfaces:
     - **Option A (Visitor style)**: Avoid associated type bounds completely:
       ```rust
       pub trait TripleQuery {
           fn visit_s_p_o(&self, s: Option<SymbolId>, p: Option<SymbolId>, o: Option<SymbolId>, visit: &mut dyn FnMut(&Triple));
       }
       ```
     - **Option B (Enum iterator)**: Use a non-boxing, zero-allocation custom enum iterator over internal indices:
       ```rust
       pub enum TripleIter<'a> {
           Empty(std::iter::Empty<&'a Triple>),
           Slice(std::slice::Iter<'a, Triple>),
       }
       ```
  2. **Immutable Graph Snapshots**: Provide immutable graph views backed by shared read-only storage (`Arc<[Triple]>` and `Arc<TripleIndex>`). Updates must produce a new snapshot through a builder; existing snapshots must never observe mutation.
* **Dialects affected**: Datalog, N3, SHACL, ShEx.
* **Complexity class improvement**: Reduces spatial complexity of pattern queries from O(n) allocations to O(1) zero-allocation traversal.
* **Correctness risks**: Lifetime constraints on the visitor closure or enum iterator making it difficult to chain queries.
* **Verification plan**: Verify that index updates do not propagate to prior active snapshots, and that visitor queries do not allocate on the heap.
* **Benchmark plan**: Measure heap allocation frequency during large-scale SHACL validation runs.
* **Explicit exclusions**: Dynamic concurrent writing inside snapshots is excluded.

---

### PROJ-522: Unified Solver, Leapjoins, and Delta Materialization
* **Title**: Unified Solver, Join Ordering, and Semi-Naive Delta Loops
* **Status**: `NEEDS_DESIGN_SPIKE`
* **Scope**: Unify join planning and recursive evaluation using Treefrog leapjoins and delta tracking.
* **Current duplicated surface**: Datalog, N3 rule evaluation, and SHACL constraint evaluation implement separate loops for pattern matching and join evaluations.
* **Proposed shared abstraction**:
  1. **Selectivity Join Ordering**: Order body patterns based on selectivity heuristics (Exact -> PredicateObject -> SubjectPredicate -> PredicateOnly -> FullScan).
  2. **Treefrog Leapjoin Planner**: Generalize multi-way joins (3+ patterns) to propose bindings from the smallest index subset and intersect against others, avoiding intermediate cartesian tuples.
  3. **Semi-Naive Delta Materialization**: Implement `FactStore` and `DerivationGate` as shared abstractions to track `recent` vs. `stable` facts, executing rules and validating constraints incrementally.
* **Dialects affected**: Datalog, N3, SHACL.
* **Complexity class improvement**: Expected improvement: reduces avoidable Cartesian products and intermediate tuple materialization; exact complexity depends on relation sizes, variable ordering, available indexes, and output cardinality.
* **Correctness risks**: Suboptimal join plans if selectivity calculations choose poor indexes.
* **Verification plan**: Correctness is proven by equivalence against baseline nested evaluation; usefulness is proven by benchmark.
* **Benchmark plan**: Run `n3_chain_depth_400` and measure throughput gains.
* **Explicit exclusions**: Non-stratified rule bases are excluded.

---

### PROJ-523: Profile-Scoped Caching and Query Plan Cache
* **Title**: Profile-Scoped Caching and Query Plan Cache
* **Status**: `READY_FOR_IMPLEMENTATION`
* **Scope**: Shared cache key calculation and memoization of query plans and validation results.
* **Current duplicated surface**: SHACL and ShEx implement separate ad-hoc memoization maps for focus nodes and target shapes.
* **Proposed shared abstraction**:
  1. **Profile-Scoped Cache Key**: Calculate keys using the tuple:
     `(graph_hash: Blake3Hash, profile_hash: Blake3Hash, dialect_mask: u32, engine_version: u32, query_shape: u32)`
     This guarantees cache invalidation whenever the underlying graph, active dialect rules, or engine version changes.
  2. **Query Plan Cache**: Cache compiled Treefrog leapjoin plans. If different rules or shape validations match identical pattern structures, reuse the pre-calculated join execution graph.
* **Dialects affected**: Datalog, N3, SHACL, ShEx.
* **Complexity class improvement**: Reduces repeated query planning and validation from O(depth * patterns) to O(1) cache hits.
* **Correctness risks**: Stale cache hits if graph mutations do not update the `graph_hash` invalidation trigger.
* **Verification plan**: Mutation testing: mutate a single triple, verify the graph hash changes, and confirm the cache is invalidated.
* **Benchmark plan**: Measure execution speed of validation on a cyclic, high-volume shape hierarchy.
* **Explicit exclusions**: Distributed or persistent caching across runs is excluded; memory-cache only.

---

### PROJ-524: Union-Find Equivalences and Generic Closure Matrix
* **Title**: Union-Find Canonicalization and Bitset Closure Matrix
* **Status**: `READY_FOR_IMPLEMENTATION`
* **Scope**: Implement a generic union-find algorithm for term identities and a bitset-based transitive closure solver for hierarchies.
* **Current duplicated surface**: RDFS subClassOf, OWL equivalentClass, and same-as aliases compute reachable nodes using disjoint iterative searches.
* **Proposed shared abstraction**:
  1. **Union-Find Equivalence**: Standardize disjoint-set structures to group term aliases, equivalent classes, and equivalent properties.
     - **Online canonicalization**: Amortized `O(α(n))` complexity.
     - **Precompressed canonical table**: `O(1)` lookup after `O(n α(n))` preprocessing.
  2. **Generic `ClosureMatrix`**: Move the SHACL subClassOf matrix to a shared module. Backed by `FixedBitSet` from the `fixedbitset` crate.
  3. **Bitset Hashing Rule**: Implement `CanonicalReceiptMaterial` for `ClosureMatrix` ensuring only the sorted edge list `Vec<(u32, u32)>` is serialized and hashed (never raw bitset memory).
* **Dialects affected**: Datalog, N3, SHACL.
* **Complexity class improvement**: Alias query resolution drops from O(n) hops to O(1) after precompression. Transitive closure checks improve from O(edges) to O(1) bitset lookups.
* **Correctness risks**: Divergence in platform-dependent bitset layouts affecting receipt hashes.
* **Verification plan**: Verify receipt hashes are byte-identical across platforms (macOS/Linux) when compiling identical class hierarchies.
* **Benchmark plan**: Benchmark transitive closure calculations up to 10,000 nodes.
* **Explicit exclusions**: Excludes dynamic cycles in user data that violate stratification constraints.

---

### PROJ-525: Uniform Diagnostics, Shape Masks, and Dirty-Region Revalidation
* **Title**: DiagnosticBuffer, Required-Property Masks, and Dirty-Region Revalidation
* **Status**: `READY_FOR_IMPLEMENTATION` (Dirty Revalidation: `NEEDS_DEPENDENCY_GRAPH_DESIGN`)
* **Scope**: Standardize validation buffers, precompute property masks, and validate shape updates incrementally.
* **Current duplicated surface**: SHACL and ShEx validate entire graphs on every change and construct validation reports inline.
* **Proposed shared abstraction**:
  1. **Uniform `DiagnosticBuffer`**: Solvers write intermediate `DiagnosticRecord` items. Decouples solvers from report rendering (SHACL formats as RDF; ShEx formats as flat lists).
  2. **Required-Property Masks**: Compile shape constraints into static bitmasks of required properties. Replaces sequential attribute checking.
  3. **Dirty-Region Revalidation**: Map updated graph triples to affected shapes/rules. Re-evaluates only the affected dependency cone instead of full revalidation.
     - **Affected dependency cone**: Defined as dirty subjects, affected properties, dependent shapes, relevant class/property closures, rule-derived facts, and report/cache invalidation scope.
  4. **Coinductive Cycle Guard**: Generic visited set tracking `(focus_node, shape_id)` loops across SHACL/ShEx.
* **Dialects affected**: SHACL, ShEx.
* **Complexity class improvement**: Graph revalidation drops from O(graph) full scan to O(affected_dependency_cone).
* **Correctness risks**: Missed validation failures if dependency analysis fails to identify all affected shapes for a mutated region.
* **Verification plan**: Verify that adding a violating triple only triggers validation checks for shapes connected via path dependencies.
* **Benchmark plan**: Measure revalidation times after small, isolated graph edits.
* **Explicit exclusions**: Custom user-defined JavaScript validation targets are out of scope.

---

## Required Answers

### 1. What is shared?
We share the internal representations: opcodes, `TripleQuery` traits, Treefrog leapjoin query planners, `ClosureMatrix` bitsets, `CoinductiveCycleGuard` loop checkers, `DiagnosticBuffer` accumulators, and the profile-scoped caching keys.

### 2. Which dialects use it?
Datalog, Notation3 (N3), SHACL, and ShEx dialects share the core storage, solvers, caching, and diagnostics.

### 3. What remains dialect-specific?
The parser boundaries, compile-time AST-to-IR converters, unique built-ins (e.g. N3 math, SHACL string matchers), specific refusal rules, and target diagnostic renderers.

### 4. What complexity class improves?
* **Multi-way joins**: From O(n^k) to leapjoin planning (depends on relation sizes, variable ordering, available indexes, and output cardinality).
* **Transitive reachability**: From O(edges) to O(1).
* **Revalidation**: From O(graph) full scan to O(affected_dependency_cone).
* **Alias resolution**: From O(n) to O(alpha(n)) amortized, or O(1) lookup after precompression.

### 5. What correctness risk is introduced?
Caching staleness due to graph mutation mis-tracking, and receipt hashing instability if platform-dependent bitset layouts are serialized directly.

### 6. What test proves the abstraction is safe?
Cross-dialect equivalence tests (asserting that optimizations produce identical facts and violations) and platform-independent receipt byte-identity verification.

### 7. What benchmark proves the abstraction is useful?
`n3_chain_depth_400` (evaluating join efficiency), `hierarchies` (evaluating closure and matrix lookups), and incremental mutation revalidation tests.

### 8. What should be implemented first?
See the Implementation Order section below.

---

## Implementation Order

We recommend implementing these abstractions in the following sequence to respect dependency chains:

1. **Predicate opcode table** (PROJ-520A) — low risk, immediate win.
2. **SymbolId / canonical term alignment** (PROJ-520A) — needed everywhere.
3. **Immutable graph snapshot** (PROJ-521) — makes cache keys and deterministic execution sane.
4. **TripleQuery enum iterator or visitor API** (PROJ-521) — removes allocation surfaces.
5. **DiagnosticBuffer** (PROJ-525) — decouples solvers from renderers.
6. **Required-property masks and cardinality frames** (PROJ-525) — high-value SHACL/ShEx win.
7. **Union-find equivalence** (PROJ-524) — safe if canonicalization phase is explicit.
8. **ClosureMatrix** (PROJ-524) — strong but must define canonical hashing.
9. **Profile-scoped cache keys** (PROJ-523) — after graph/profile hashes are stable.
10. **Join planner / leapjoin / semi-naive delta** (PROJ-522) — bigger design surface; do after indexes stabilize.
11. **Dirty-region revalidation** (PROJ-525) — last, because it needs dependency tracking.

---

## Explicit Exclusions

* **Rayon Parallelism**: Speculative thread-level evaluation is excluded until profiling proves lock contention is resolved.
* **Full Negation-as-Failure (NAF)**: Excluded from the unified join solver to keep the execution semantics sound.
* **External SPARQL Endpoints**: Excluded from shape validation to maintain a zero-network sandbox boundary.

---

## Final Standing Assessment

**Status**: `READY_FOR_IMPLEMENTATION_WITH_PATCHES`

The JIRA tickets are concrete, structured, and contain sufficient implementation instructions and correctness gates for subsequent implementation agents to execute.

# Performance-Critical Patterns from Semantic-Web Audits

**Focus**: Latency & Throughput improvements applicable to Graphlaw

---

## PROJ-501: reasonable (OWL RL Datalog Reasoner)

### Hot-Path Optimization: Datalog Semi-Naive Materialization

**Source**: `vendor/reasonable/src/reasoner.rs` (93 KB core logic)

**Pattern**: Datalog rules encoded as **state transitions** over triple sets, not AST traversal.

**Latency Impact**: 
- ✅ Delta-driven semi-naive fixpoint materialization (new facts from round n drive round n+1; no abstract syntax overhead)
- ✅ Delta tracking prevents recomputation (only new derivations trigger subsequent rounds)
- ✅ Disjoint set union for O(α(n)) equivalent-entity lookups

**Finding**: OWL 2 RL can be expressed as ~100 Datalog rules. Apply these rules via **semi-naive fixpoint iteration**, not full-blown reasoner.

**For Graphlaw PROJ-401**:
- Rule-based evaluation beats AST walking
- Delta tracking (what changed last round) prevents recomputation
- Disjoint sets reduce equivalence checking from O(n²) to O(α(n))

---

## PROJ-503: SHACL Validation (Constraint Evaluation)

### Constraint Evaluation Optimization: Selectivity Ordering

**Source**: `vendor/shacl/Cargo.toml` optional dependencies: rayon 1.7, dashmap 6 (not required for core SHACL)

**Pattern**: 
```
For each focus node:
  Order constraints by selectivity (cardinality before pattern)
  For each constraint:
    Evaluate constraint → exit early on violation
  End for
End for focus
```

**Latency Impact**:
- ✅ Cheap constraint checks first (cardinality checks short-circuit before regex)
- ✅ Early exit on violation (no redundant checks)
- ⚠️ Parallelism adds overhead; only justified if batch size and independent count both proven large

**For Graphlaw PROJ-401**:
- Implement selectivity ordering (Phase 0) — no extra crates needed
- Skip `rayon`/`dashmap` initially (Phase 0-1); add only if profiling proves independent batches dominate runtime and overhead is justified
- If parallelism is added later: all constraint results must be sorted/canonicalized before hashing

---

### Hook/Shape Dependency Ordering (Measured-Only Optimization)

**Source**: `vendor/shacl/Cargo.toml` optional dependency: petgraph 0.8 (not required for core SHACL)

**Pattern**: 
```
Build DAG of shape dependencies:
  shape_a depends_on shape_b if shape_a references shape_b
End DAG build

For each shape in topological order:
  Evaluate shape
  Cache result
End for
```

**Latency Impact**:
- ✅ Avoids redundant re-evaluation of dependent shapes
- ✅ Topological order ensures transitive dependencies computed first
- ⚠️ DAG construction overhead; only justified if shape hierarchy is deep/complex

**For Graphlaw PROJ-401**:
- Implement small Kahn topological sort for hook/shape dependencies (may not need `petgraph`)
- Add `petgraph` only if profiling shows shape DAG complexity justifies it
- Constraint: dependency ordering is a P1 optimization, not Phase 0 requirement

---

### Hot-Path Optimization 3: IR Compilation (AST → Optimized Representation)

**Source**: `vendor/shacl/src/ir/` modules

**Pattern**:
```
Phase 1 (once): Parse RDF → AST → IR
  - Convert string IRIs to indices
  - Precompile path expressions
  - Order constraints by selectivity
  
Phase 2 (per validation): Use IR
  - O(1) shape lookup via index
  - Precompiled paths avoid re-parsing
  - Constraint ordering avoids expensive checks first
```

**Latency Impact**:
- ✅ Upfront IR compilation cost amortized over many validations
- ✅ String→index conversion eliminates per-validation IRI overhead
- ✅ Constraint selectivity ordering (sh:maxCount before sh:pattern) short-circuits violations early

**For Graphlaw PROJ-401**:
- Compile hooks to IR once on load
- Index-based shape/hook references instead of string lookups
- Order constraints by selectivity (cardinality before regex)
- Estimate 3-5x latency reduction per validation

---

## PROJ-505: horned-owl (OWL AST Design)

### Throughput Optimization 1: IRI Deduplication via Rc<str> Interning

**Source**: `vendor/horned-owl/src/model.rs` lines 165-166, 278-284

**Pattern**:
```rust
// Instead of:
let iri1 = "http://example.com/Class1".to_string();  // alloc
let iri2 = "http://example.com/Class1".to_string();  // alloc again
// Memory: 2x strings for same IRI

// Use:
let builder = Build::new_rc();
let iri1 = builder.iri("http://example.com/Class1");  // alloc once, cache
let iri2 = builder.iri("http://example.com/Class1");  // return cached Rc<str>
// Memory: 1x string shared via Rc
```

**Throughput Impact**:
- ✅ Memory reduction: O(unique_iris) instead of O(iris)
- ✅ Clone cost: O(1) Rc clone instead of O(len) string copy
- ⚠️ Equality check: Rc<str> still compares by value; true O(1) equality requires symbol interning to SymbolId (u32/u64)

**For Graphlaw PROJ-401**:
- Use `lasso` crate for SymbolId (u32) interning, not Rc<str> equality
- Convert IRI/String/Literal → SymbolId at parse boundary
- Use SymbolId in all hot-path triples and comparisons
- Estimate 4-8x memory reduction for RDF graphs with repeated IRIs
- Achieve true O(1) IRI/symbol comparisons via ID equality

---

### Throughput Optimization 2: Deterministic Ordering via BTreeSet

**Source**: `vendor/horned-owl/src/model.rs` lines 280-284

**Pattern**:
```rust
// Instead of:
let mut iris = HashSet::new();  // Unordered, unpredictable iteration
iris.insert("http://example.com/A");
iris.insert("http://example.com/B");
// Iteration order: random, differs per run

// Use:
let mut iris = BTreeSet::new();  // Ordered, deterministic iteration
iris.insert("http://example.com/A");
iris.insert("http://example.com/B");
// Iteration order: always A, B (lexicographic)
```

**Throughput Impact**:
- ✅ Deterministic iteration order (canonicalization boundary for receipts)
- ✅ Receipt stability: sorted/canonical order before hashing prevents drift
- ⚠️ IndexMap preserves insertion order only; still requires explicit canonicalization for receipt materials

**For Graphlaw PROJ-401**:
- Use `BTreeSet` or `indexmap::IndexSet` ONLY at receipt/output canonicalization boundaries (not in hot path)
- Hot-path internal maps: use `rustc_hash::FxHashMap` (faster, order-independent)
- Receipt surface: use `indexmap` + explicit sort before hash
- Estimate: receipt stability (necessary for idempotent replay)

---

## Performance Hierarchy (Latency Impact)

**Highest impact** (10-100x faster):
1. **SymbolId interning** — O(1) equality, compact representation
2. **ID-based triples** — lookup/join via integer comparison
3. **AST→IR compilation** — eliminates per-validation overhead

**Medium impact** (2-5x faster):
4. **Constraint selectivity ordering** — short-circuit failures early (cardinality before regex)
5. **Semi-naive delta tracking** — only new facts drive new rounds
6. **Bitset closures** — dense membership, O(1) union/intersection

**Measured-only** (add after profiling):
7. **Parallel independent evaluation** (rayon) — only if independent batches dominate runtime
8. **Shape dependency DAG ordering** (petgraph) — only if shape hierarchy is deep/complex

---

## Architecture Pipeline for PROJ-401

```
Raw RDF terms (strings)
  ↓ (Step 1: Symbol table)
SymbolId (u32/u64)
  ↓ (Step 2: ID triples)
IdTriple { s: SymbolId, p: SymbolId, o: TermId }
  ↓ (Step 3: Compile to IR)
CompiledDialectProfile (hooks/shapes as IR, no repeated string lookup)
  ↓ (Step 4: Deterministic evaluation)
SemiNaiveMaterializer (delta-driven, fixpoint)
  ↓ (Step 5: Closure as bitsets)
ClosureSet (dense membership, bitwise ops)
  ↓ (Step 6: Canonicalize at boundary)
CanonicalOutput (sorted, deterministic order)
  ↓ (Step 7: Hash for receipt)
ReceiptHash (BLAKE3, stable across runs)
```

---

## Implementation Phases for PROJ-401

### Phase 0 (Representation Foundation — REQUIRED FIRST):
- [ ] SymbolId interner (lasso crate)
- [ ] IdTriple representation
- [ ] Compiled dialect IR (hooks/shapes)
- [ ] Deterministic canonicalization at receipt boundary

**Rationale**: Representation wins (10-100x) vastly outweigh parallelism overhead. Phase 0 must stabilize before Phase 1.

### Phase 1 (Post-Measurement Only):
- [ ] Parallel independent evaluation (rayon) — only if profiling proves independent batches dominate runtime
- [ ] Petgraph dependency ordering — only if hook/shape DAG complexity warrants it (small Kahn sort may suffice for daily profile)
- [ ] Roaring bitmaps for closure — only if bitsets become too dense or memory-heavy

**Constraint**: Parallelism adds scheduling overhead, nondeterministic accumulation order, cache contention, and harder replay debugging. Add only after proof of gain.

---

## Benchmark Targets (Corrected)

### Must-Pass Regression (existing tests):
- `n3_chain_depth_50`, `n3_chain_depth_150`, `n3_chain_depth_400`
- `test_transitive_rule`, `test_rdf_hierarchy_10`, `test_rdf_hierarchy_100`
- `hook_trigger_datalog_small`, `hook_trigger_datalog_medium`
- `hook_receipt_replay_stability`

**Target**: No benchmark regresses >5%; receipt/replay hashes remain stable.

### New Representation Benchmarks (Phase 0):
- `symbol_intern_10k`, `symbol_intern_100k` (interning cost)
- `triple_insert_string_vs_id`, `triple_lookup_string_vs_id` (representation overhead)
- `closure_bitset_vs_hashset` (bitset perf)
- `canonical_receipt_sort_cost` (canonicalization overhead)

**Target**: Representation benchmarks establish Phase 0 baseline; subsequent improvements measured against this.

### OWL RL Conditional (Phase 1, if needed):
- `owl_rl_subclass_1k`, `owl_rl_domain_range_1k`, `owl_rl_inverse_property_1k`
- `owl_rl_same_as_refused` (explicit refusal of unsupported features)
- `owl_rl_unsupported_feature_refused` (error behavior)

---

## Status Doctrine Entry

```
PROJ_401_GOAL = SMALL_FAST_DAILY_PROFILE_CORE
SEMANTIC_ENGINE_IMPORT = REFUSED
HOT_PATH_STRING_TERMS = REFUSED
SYMBOL_ID_TRIPLES = PLANNED_P0
COMPILED_IR = PLANNED_P0
RECEIPT_CANONICALIZATION = REQUIRED
BITSET_CLOSURE = PLANNED_P0
RAYON = MEASURED_P1 (add only if profiling proves independent batches dominate)
DASHMAP = REFUSED_BY_DEFAULT (add only if contention proven)
PETGRAPH = OPTIONAL_P1 (small Kahn sort may suffice)
ROARING = OPTIONAL_SCALE_GATE (only if bitsets become too dense)
```


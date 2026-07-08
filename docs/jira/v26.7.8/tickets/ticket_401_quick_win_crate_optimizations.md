# PROJ-401 — Quick-Win Rust Crate Optimizations (v26.7.8 P0)

**Status**: PLANNED  
**Scope**: Add surgical Rust crates to accelerate GL hot surfaces without rewriting reasoning substrate  
**Dependencies**: PROJ-301..306 (v26.7.4) COMPLETE  
**Target**: Immediate implementation order, with post-measurement follow-ups

---

## Overview

Graphlaw has achieved baseline capability (hooks, semi-naive materialization, indexed joins, closure memoization). The next phase is **representation efficiency**: converting expensive string operations, unordered data structures, and sequential checks into fast, deterministic primitives.

This ticket authorizes adding **8 crate dependencies** across **6 implementation phases**, each focused on a specific GL hot surface (symbol identity, set membership, joins, closures, receipt stability, parallelism).

**Core thesis:**  
The fastest GL optimization is not "faster Rust" in the abstract. It is: `String RDF terms → interned IDs`, `HashSet closures → bitsets`, `unordered maps → deterministic receipt surfaces`, `sequential independent checks → parallel batches`.

---

## Hot Surfaces to Target

| Surface | Current Cost | Optimization Lever | Crates |
|---------|-------------|-------------------|--------|
| **Symbol identity** | String clone, allocation, hash on every term | Intern IRIs/predicates/classes to IDs | `lasso`, `string-interner` |
| **Set membership** | `HashSet<String>` closure, repeated BFS | `FixedBitSet` for dense ID closure | `fixedbitset` |
| **Join/index lookup** | SipHash overhead on post-admission maps | `rustc-hash`/`ahash` for internal maps | `rustc-hash`, `ahash` |
| **Closure computation** | Double hashing, lookup/insert patterns | SwissTable raw-entry control | `hashbrown` (P1) |
| **Receipt/replay output** | Nondeterministic HashMap iteration | Insertion-order-preserving maps | `indexmap` |
| **Parallel independent eval** | Sequential batch Datalog/hook checks | Safe data parallelism library | `rayon` (P1) |

---

## Crate Selection Rationale

### Phase 0: Immediate (symbol interning + fast hashes)

**1. `lasso` v0.7 or `string-interner` v0.x**
- **Target**: IRIs, predicates, class IDs, hook names, Datalog symbols, RDF object strings
- **Win**: Eliminate repeated string clone/allocation/hash. Compact symbol IDs enable dense bitsets in Phase 1.
- **Integration**: Introduce `SymbolId` wrapper type; keep canonical string rendering available for receipts.
- **Risk**: Low if confined to internal representation.

**2. `rustc-hash` v2**
- **Target**: Internal post-admission maps (Datalog fact maps, join indexes, symbol tables, closure worksets)
- **Win**: FxHash is ~3× faster than SipHash for non-adversarial data.
- **Integration**: Type aliases (`internal::FastMap<K,V>`) hide from API surface.
- **Receipt safety**: Never depend on HashMap iteration order. All receipt output must be explicitly sorted.
- **Risk**: Very low; compiler-proven determinism on fixed inputs.

**3. `fixedbitset` v0.5**
- **Target**: `subClassOf`/`subPropertyOf` closure, OWL RL type propagation, reachability sets, hook eligibility
- **Win**: Replace `HashSet<ClassId>` with `FixedBitSet`; bitwise operations and dense storage.
- **Integration**: Requires all closure members to be interned IDs (Phase 0 prerequisite).
- **Risk**: Low; idempotent on existing closure semantics.

**4. `smallvec` v1**
- **Target**: Small bounded vectors (Datalog atoms, hook deps, triple patterns, diagnostics)
- **Win**: Inline storage for small arrays; spill to heap only when needed.
- **Integration**: Type aliases (`SmallDeps<T> = SmallVec<[T; 4]>`).
- **Risk**: Very low; transparent to logic.

**5. `indexmap` v2**
- **Target**: Receipt/manifest surfaces, verdict records, diagnostic maps
- **Win**: Insertion-order preservation ensures deterministic iteration independent of hash values.
- **Integration**: Use only where byte-identical replay matters; internal maps remain `FxHashMap`.
- **Risk**: Very low; improves correctness/stability.

### Phase 1: Post-Measurement (advanced hashing + parallelism)

**6. `hashbrown` v0.15**
- **Target**: Intern table intern-or-get, join index construction, canonicalization maps
- **When**: Only after profiling shows double hashing or lookup/insert bottlenecks.
- **Win**: Raw-entry control for custom lookup patterns.
- **Risk**: Medium; lower-level APIs, but isolated use.

**7. `rayon` v1**
- **Target**: Independent hook batches, independent closure components, parallel rule groups
- **When**: Measured to improve batch throughput without receipt overhead.
- **Constraint**: Parallel output must be sorted/canonicalized before hashing.
- **Win**: Safe data-level parallelism.
- **Risk**: Medium; requires determinism enforcement.

**8. `roaring` v0.11**
- **Target**: Large sparse fact sets, derived fact closure deltas, large predicate posting lists
- **When**: Graph scale gate; only if FixedBitSet becomes too dense or memory-heavy.
- **Win**: Compressed bitmap set operations for sparse data.
- **Risk**: Medium; profile first.

### Do Not Add (or Gate Strictly)

- **`rayon`** — REFUSED_BY_DEFAULT. Add only after profiling proves independent batches dominate runtime. Overhead risks: scheduling, nondeterministic accumulation, cache contention, replay debugging cost.
- **`dashmap`** — REFUSED_BY_DEFAULT. Add only if benchmark proves concurrent write contention (unlikely in Phase 0-1). All outputs must be canonicalized before hashing.
- **`petgraph`** — OPTIONAL_P1. A small Kahn topological sort may suffice for the 80/20 daily profile's hook/shape dependency ordering. Don't add speculatively.
- **`roaring`** — OPTIONAL_SCALE_GATE. Add only if FixedBitSet becomes too dense or memory-heavy at scale. Profile first.
- **`datafrog`** — Useful as benchmark comparator and algorithm reference, not a mainline rewrite. (See separate audit ticket.)
- **Semantic-web library imports** — REFUSED. All semantic dialect implementations must be from audit learnings, clean-room reimplemented, never wholesale imports of reasoners/validators.
- Full RDF store replacement, SPARQL engine rewrite, async/concurrency in hot paths, persistence before in-memory stabilization.

---

## Implementation Ladder (Ordered, Each Step Depends on Prior)

### Step 1: Symbol Interning (MUST COMPLETE FIRST)

**Deliverables:**
- Introduce `SymbolId` (u32 or u64) type and interner (via `lasso`)
- Wrap IRIs, predicates, classes, hook names, Datalog symbols, RDF object strings at parse boundary
- Keep canonical `SymbolId → string` rendering for receipts/diagnostics
- Add type aliases: `type SymbolId = u32;` (or u64 if > 4B symbols needed)

**Tests:**
- Existing hook trigger/receipt tests pass unchanged
- New: interning round-trip (string → SymbolId → canonical string == original)
- New: receipt hash stability before/after interning
- New: SymbolId(1) == SymbolId(1) for same symbol, != for different symbols

**Acceptance:** All 79 existing tests pass; receipt hashes match baseline; Symbol round-trip verified.

---

### Step 2: Convert Triple Representation to ID-Based (Depends on Step 1)

**Deliverables:**
- Change internal triple from `Triple<String>` to `TripleId { s: SymbolId, p: SymbolId, o: TermId }`
- Update TripleStore, TripleIndex, query engine to use ID triples throughout
- Update join/index operations to work on IDs (integer comparison, not string equality)
- Render back to canonical RDF (TTL/N-Triples) from IDs for output/receipts

**Tests:**
- Existing triple-store tests pass
- Benchmarks: `n3_chain_depth_50/150/400` (baseline: 33.85 ms for depth 400)
- New: ID triple lookup performance matches or beats string triples

**Acceptance:** Tests pass; benchmarks do not regress; hot-path uses ID comparison, not string.

---

### Step 3: Compile Hooks/Shapes to IR (Depends on Step 2)

**Deliverables:**
- Convert `KnowledgeHook<String>` → `CompiledHook<SymbolId>` at load time
- Convert `Shape<String>` → `CompiledShape<SymbolId>` at load time
- Pre-index all condition/action/constraint references (no repeated string lookup during evaluation)
- Order constraints by selectivity (cardinality checks before regex/pattern checks)

**Tests:**
- Existing hook trigger tests pass unchanged
- New: compiled hook condition evaluation has no string lookups in hot path
- New: constraint ordering produces earlier failures for invalid nodes (perf via short-circuit)

**Acceptance:** Hook firing semantics unchanged; no repeated string lookup during condition evaluation; hook receipts stable.

---

### Step 4: Closure Representation as Bitsets (Depends on Step 1)

**Deliverables:**
- Convert `subClassOf`/`subPropertyOf` closure from `HashSet<SymbolId>` → `FixedBitSet`
- Convert hook dependency, OWL RL type propagation, reachability sets to bitsets
- Bitwise union/intersection where multiple closures are combined
- Document bitset size (max SymbolId determines allocation)

**Tests:**
- Existing closure semantics tests pass unchanged
- New: FixedBitSet closure membership matches prior HashSet behavior exactly
- New: bitwise union produces same closure as iterative add

**Acceptance:** Closure facts unchanged; bitset operations deterministic and reproducible; OWL RL closure benchmarks pass.

---

### Step 5: Fast Hash Maps (Behind Type Aliases, Depends on Step 1-4)

**Deliverables:**
- Add type aliases:
  - `type internal::FastMap<K,V> = rustc_hash::FxHashMap<K,V>` (for internal state, order-independent)
  - `type internal::CanonicalMap<K,V> = indexmap::IndexMap<K,V>` (for receipt surfaces only, insertion-order-stable)
- Replace all post-admission internal maps with `FastMap<SymbolId, ...>` 
- For receipt/manifest surfaces, use `CanonicalMap` + explicit sort before hashing (do NOT rely on insertion order for receipt stability)
- Document: "Fast internal maps can iterate in any order; receipt material MUST be sorted or canonicalized before hashing."

**Tests:**
- Datalog fact map tests
- Join index tests
- Receipt hash stability (verify sort-before-hash is applied)

**Acceptance:** Tests pass; receipts unchanged; FxHashMap latency verified; no nondeterministic iteration in receipt material.

---

### Step 6: Deterministic Receipt Boundaries

**Deliverables:**
- At receipt serialization boundary: sort all maps/sets before hashing
- Use `indexmap` only where insertion order directly maps to output order (rare)
- Document receipt canonicalization: "All receipt material is sorted lexicographically before BLAKE3 hashing"
- Verify byte-identical replay

**Tests:**
- Receipt round-trip: receipt → JSON → canonical parse == receipt
- Replay validation tests: same input always produces same receipt hash

**Acceptance:** Replay tests pass unchanged; receipt hashes stable across runs.

---

### Step 7: Post-Measurement (P1 — Only After Profiling)

**Hashbrown** (raw-entry APIs):
- Add only after profiling shows double hashing or lookup/insert bottlenecks
- Use for intern-or-get pattern if contention is proven

**Rayon** (parallelism):
- Add only after profiling proves independent hook/constraint batches dominate runtime
- Constraint: parallel output MUST be sorted/canonicalized before hashing
- Monitor: scheduling overhead, cache contention, replay debugging cost

**Roaring** (sparse bitmaps):
- Add only if FixedBitSet becomes too dense or memory-heavy at scale
- Profile first to justify

**Constraint on all P1 additions**: Do not add speculatively. Profile, measure, prove gain, then add.

---

## Benchmark Targets (Three-Tier Structure)

### Must-Pass Regression (Existing Tests — All Must Pass Unchanged)

| Benchmark | Baseline | Constraint |
|-----------|----------|-----------|
| `n3_chain_depth_50` | TBD | No regression >5% |
| `n3_chain_depth_150` | TBD | No regression >5% |
| `n3_chain_depth_400` | 33.85 ms | No regression >5% |
| `test_transitive_rule` | TBD | No regression >5% |
| `test_rdf_hierarchy_10` | TBD | No regression >5% |
| `test_rdf_hierarchy_100` | TBD | No regression >5% |
| `hook_trigger_datalog_small` | TBD | No regression >5% |
| `hook_trigger_datalog_medium` | TBD | No regression >5% |
| `hook_receipt_replay_stability` | TBD | Receipt hash matches baseline |

**Success**: All must pass; receipt/replay hashes stable; all 79 tests pass.

### New Representation Benchmarks (Phase 0 Baseline)

| Benchmark | Goal | Measures |
|-----------|------|----------|
| `symbol_intern_10k` | Establish baseline | Interning 10K unique symbols |
| `symbol_intern_100k` | Establish baseline | Interning 100K unique symbols |
| `triple_insert_string_vs_id` | Baseline for Step 2 | Triple insert cost (string vs SymbolId) |
| `triple_lookup_string_vs_id` | Baseline for Step 2 | Triple lookup cost (string vs SymbolId) |
| `closure_bitset_vs_hashset` | Baseline for Step 4 | Membership/union operations (bitset vs HashSet) |
| `canonical_receipt_sort_cost` | Baseline for Step 6 | Sorting overhead at receipt boundary |

**Success**: Establishes Phase 0 baseline; subsequent improvements measured against this.

### OWL RL Conditional (Phase 1+, Only If OWL RL Implemented)

| Benchmark | Goal | Measures |
|-----------|------|----------|
| `owl_rl_subclass_1k` | Hot-path perf | Subclass closure for 1K classes |
| `owl_rl_domain_range_1k` | Hot-path perf | Domain/range inference for 1K properties |
| `owl_rl_inverse_property_1k` | Hot-path perf | Inverse property closure for 1K properties |
| `owl_rl_same_as_refused` | Correctness | Unsupported sameAs feature properly refused |
| `owl_rl_unsupported_feature_refused` | Correctness | Other unsupported features properly refused |

**Success**: If OWL RL is added, these benchmarks establish performance and verify refusal behavior.

---

## Success Criteria (Final)

- [ ] All must-pass regression tests pass with <5% regression
- [ ] Receipt/replay hashes stable across all runs
- [ ] All 79 existing tests pass
- [ ] Negative fixtures still refuse correctly
- [ ] Phase 0 benchmarks establish baseline
- [ ] No nondeterministic iteration in receipt material

---

## Acceptance Criteria

- [ ] All crates added to `Cargo.toml` with pinned versions
- [ ] Symbol interning (Sub-Task 1) complete and tested
- [ ] ID triple conversion (Sub-Task 2) complete and benchmarked
- [ ] Fast hash maps (Sub-Task 3) in place with determinism verification
- [ ] Closure FixedBitSet (Sub-Task 4) integrated and tested
- [ ] SmallVec allocation (Sub-Task 5) profiled
- [ ] Receipt surfaces deterministic (Sub-Task 6) and replay-verified
- [ ] Before/after benchmark report generated
- [ ] All workspace tests pass
- [ ] Receipt/replay hashes stable
- [ ] Negative fixtures pass unchanged
- [ ] No nondeterministic iteration in receipt material
- [ ] Changes documented in `docs/standing/PERFORMANCE_REPORT.md`

---

## Standing Rules

- Mark **ALIVE** if tests, benchmarks, and receipt stability all pass
- Mark **PARTIAL_ALIVE** if speed improves but receipt or fixture issues remain
- Mark **REFUSED** if any crate introduces nondeterminism into standing material

---

## Related Tickets

- PROJ-301..306 (v26.7.4): Knowledge Hooks foundation — COMPLETE
- PROJ-501 (separate): Audit `datafrog` for learning opportunities (algorithm reference, possible future backend)

---

## References

- **Quick-win thesis**: String RDF terms → interned IDs; HashSet closures → bitsets; unordered maps → deterministic receipt surfaces; sequential independent checks → parallel batches
- **Crate selection rationale**: [See v26.7.8 guidance document]
- **Benchmark baseline**: `n3_chain_depth_400` currently 33.85 ms (semi-naive + selective indexing)
- **Constraint**: No rewrite of reasoning substrate. No change to hook semantics, receipt generation, refusal behavior, or replay logic.

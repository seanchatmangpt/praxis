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

### Do Not Add

- **`datafrog`** — Useful as benchmark comparator and algorithm reference, not a mainline rewrite. (See separate audit ticket.)
- Full RDF store replacement, SPARQL engine rewrite, async/concurrency in hot paths, persistence before in-memory stabilization.

---

## Implementation Roadmap

### Sub-Task 1: Symbol Interning (MUST COMPLETE FIRST)

**Deliverables:**
- Introduce `SymbolId` type and `SymbolInterner` (via `lasso` or `string-interner`)
- Wrap IRIs, predicates, classes, hook names, Datalog symbols, RDF object strings
- Keep canonical string rendering for receipts/diagnostics
- Add regression tests verifying interning transparency

**Tests:**
- Existing hook trigger/receipt tests pass unchanged
- New: interning round-trip (symbol → ID → string == original)
- New: receipt hash stability before/after interning

**Acceptance:** All 79 existing tests pass; receipt hashes match baseline.

### Sub-Task 2: Convert Triple Indexes to ID Triples

**Deliverables:**
- Change internal triple representation from `(String, String, String)` to `(SymbolId, SymbolId, SymbolId)`
- Update TripleStore, TripleIndex, query engine to use ID triples
- Update benchmarks to measure join/query performance

**Tests:**
- Existing triple-store tests pass
- Benchmarks: `n3_chain_depth_50/150/400` (baseline: 33.85 ms for depth 400)

**Acceptance:** Tests pass; benchmarks do not regress.

### Sub-Task 3: Fast Hash Maps (Behind Aliases)

**Deliverables:**
- Add `type internal::FastMap<K,V> = rustc_hash::FxHashMap<K,V>`
- Replace post-admission internal maps with FastMap
- Verify all receipt surfaces remain deterministic (sort before hashing)

**Tests:**
- Datalog fact map tests
- Join index tests
- Receipt hash stability

**Acceptance:** Tests pass; receipts unchanged.

### Sub-Task 4: Closure Representation (FixedBitSet)

**Deliverables:**
- Convert `subClassOf`/`subPropertyOf` closure to `FixedBitSet` (after interning)
- Convert hook dependency, OWL RL type propagation, reachability to bitsets
- Benchmark OWL/RDFS closure computation

**Tests:**
- Existing closure semantics tests pass
- New: FixedBitSet closure membership tests
- SHACL/OWL RL tests

**Acceptance:** Tests pass; closure performance improves or maintains baseline.

### Sub-Task 5: Small Vector Allocation

**Deliverables:**
- Replace `Vec<Term>` → `SmallVec<[Term; 3]>` in Datalog atoms
- Replace `Vec<String>` → `SmallVec<[SymbolId; 4]>` in hook dependencies
- Profile allocation reduction

**Tests:**
- Existing tests pass
- Allocation profile before/after

**Acceptance:** Tests pass; no logic changes.

### Sub-Task 6: Deterministic Receipt Surfaces

**Deliverables:**
- Use `IndexMap` for receipt payload, manifest, verdict records
- Verify byte-identical replay
- Document where insertion order matters

**Tests:**
- Receipt round-trip: receipt → JSON → canonical parse == receipt
- Replay validation tests

**Acceptance:** Replay tests pass unchanged.

### Sub-Task 7: Post-Measurement Phases (P1)

- Hashbrown raw-entry APIs (after profiling)
- Rayon parallelism (independent batches only, output sorted before hashing)
- Roaring sparse sets (if graph scale increases)

---

## Benchmark Targets (Before/After)

Run all benchmarks with optimizations and report:

| Benchmark | Baseline | Target | Measurement |
|-----------|----------|--------|-------------|
| `n3_chain_depth_50` | TBD | -20% | semi-naive + ID triples |
| `n3_chain_depth_150` | TBD | -20% | ID triples + bitset closure |
| `n3_chain_depth_400` | 33.85 ms | -15% | fast hashes + FixedBitSet |
| Transitive rule | TBD | -15% | closure bitset ops |
| RDF hierarchy | TBD | -20% | ID interning + FastMap |
| Hook trigger (Datalog small) | TBD | -10% | SmallVec + ID terms |
| Hook receipt/replay path | TBD | 0% ± 1% | deterministic, no regression |
| OWL/RDFS closure (if present) | TBD | -25% | FixedBitSet + FastMap |

**Success criteria:**
- No benchmark regresses >5%
- Receipt/replay hashes remain stable
- All 79 tests pass
- Negative fixtures still refuse correctly

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

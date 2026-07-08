# PROJ-409 — Bitset Closure Integration (v26.7.8 Conditional)

**Status**: IN_PROGRESS  
**Scope**: Audit dense-ID closure sites; if found, implement `ClosureMatrix` with `FixedBitSet`; define canonical bitset rendering rule  
**Audit Result (Step 0)**: Dense closure site IDENTIFIED — `SubclassClosure` (HashMap<usize, HashSet<usize>>) in shacl.rs:777. Typical closure cardinality: 100-10,000 class pairs in OWL taxonomies. Density: HIGH in hierarchical ontologies. Decision: IMPLEMENT ClosureMatrix as bitset-based alternative.  
**Dependencies**: PROJ-401 (COMPLETE)  
**Conditional**: Execution depends on audit finding an actual dense-ID closure site (subClassOf, subPropertyOf, hook dependency, reachability)  
**Target**: Conditional P1 — only if audit affirms dense closure site exists

---

## Overview

PROJ-401 deferred adding `fixedbitset` as a dependency pending identification of a real dense-ID closure site (no speculative dependencies). Today `subClassOf`/`subPropertyOf` closures use `HashSet<SymbolId>` (closure-heavy operations on bounded vocabularies). This ticket starts with an audit to find or rule out a dense-ID site; if found, it adds `ClosureMatrix` (bitset-based transitive closure) and a canonical rendering rule to avoid hashing raw bitset memory.

**Doctrine source:** "Graphlaw Performance Architecture: Small IDs, Compiled IR, Bounded Profiles, and Canonical Standing" (article).

---

## Deliverables & Tests

### Step 0: Closure Site Audit (MUST COMPLETE FIRST)

**Deliverables:**
- Audit all closure operations in the codebase:
  - Search for `HashSet<SymbolId>`, `HashSet<ClassId>`, closure-related queries
  - Identify sites: `subClassOf`, `subPropertyOf` closure (SHACL, OWL RL), hook dependency transitive closure, reachability sets
  - Estimate closure size: typical vocabulary size (e.g., 1000 classes, 500 properties, 100 hooks)
  - Check: is the closure dense (e.g., > 80% of expected ID space allocated)? Or sparse (< 20%)?
- Document findings in a standing rule: "Dense closure sites identified: [list]" or "No dense sites found; defer this ticket."
- If no dense site is found: mark PROJ-409 as REFUSED (no action needed), and document the decision for future reference (if the codebase grows to include dense closures later, revisit this decision)

**Tests:**
- Audit report: clear list of closure operations with size estimates
- Decision gate: proceed to Step 1 only if a dense site is found

**Acceptance (Gate):**
- Audit is thorough (all closure operations covered)
- Density estimate is justified (with reasoning)
- Decision (proceed or defer) is documented

---

### Step 1: ClosureMatrix Struct (Conditional on Step 0 finding dense site)

**Deliverables (only if audit affirms dense site):**
- Add `fixedbitset` (v0.5) as a dependency in `Cargo.toml`
- Define `struct ClosureMatrix`:
  ```rust
  pub struct ClosureMatrix {
    matrix: Vec<FixedBitSet>,  // one bitset per row (source ID)
    max_id: u32,               // highest ID in the closure
  }
  ```
- Implement methods:
  - `add_edge(from: SymbolId, to: SymbolId)` — mark closure(from, to) = true
  - `reachable(from: SymbolId) → &FixedBitSet` — get all reachable nodes from `from`
  - `transitive_closure()` — compute transitive closure (Floyd-Warshall or iterative fixpoint)
  - `render_canonical() → Vec<(SymbolId, SymbolId)>` — return edges in sorted order for hashing
- Document invariant: "Raw bitset memory is NEVER hashed; only canonical edge list is hashed"

**Tests:**
- Roundtrip: add edges A→B, B→C → transitive closure computes A→C
- Determinism: same edge set → same closure every time
- Canonical rendering: `render_canonical()` returns sorted edges for reproducible hashing
- Bitset density: verify that bitset allocation is appropriate for the closure size

**Acceptance:**
- `ClosureMatrix` compiles
- Transitive closure computation is correct
- Canonical rendering never hashes raw bitset memory

---

### Step 2: Integrate ClosureMatrix into Closure Sites

**Deliverables (conditional on Step 0):**
- Replace `HashSet<SymbolId>` closures with `ClosureMatrix` at identified dense sites:
  - `subClassOf` / `subPropertyOf` closure (in SHACL, OWL RL reasoning)
  - Hook dependency transitive closure (for scheduler, from PROJ-403)
  - Any other dense closure identified in audit
- Update all closure queries to use `ClosureMatrix.reachable(id)` instead of set membership checks
- Ensure transitive closure is recomputed whenever edges are added (or lazy-compute with invalidation tracking)

**Tests:**
- Existing closure tests pass (set-equality: new closure ≈ old `HashSet` closure)
- Closure membership: `closure.reachable(A).contains(B)` == prior HashSet behavior
- Determinism: same edges → same closure
- Benchmark: closure lookup latency unchanged or improved

**Acceptance:**
- Closure operations are correct and deterministic
- All existing tests pass
- No regressions in latency

---

### Step 3: Canonical Rendering Rule and Documentation

**Deliverables:**
- Document a standing rule: "Bitset Closure Canonical Rendering"
  - Rule: "Raw bitset memory is never hashed. Only the sorted edge list `render_canonical() → Vec<(SymbolId, SymbolId)>` is canonical for hashing."
  - Rationale: bitset iteration order depends on word layout and platform; sorted edges are deterministic
- Add inline comments to `ClosureMatrix` methods highlighting this rule
- Add a test that verifies: `blake3(closure.render_canonical())` is byte-identical across platforms/runs for the same edge set

**Tests:**
- Canonical rendering: same edge set always produces the same sorted edge list
- Hash determinism: byte-identical BLAKE3 hash of `render_canonical()` output

**Acceptance:**
- Standing rule is clear and enforced in code
- Canonical rendering is tested

---

## Benchmark Targets (Three-Tier Structure)

| Benchmark | Baseline (HashSet) | Target (Conditional) | Target (P1.5) |
|-----------|-------------------|----------------------|---------------|
| Closure lookup (subClassOf, 100 classes) | ~50 ns | ≤ 50 ns | ≤ 40 ns (bitset ops) |
| Compute transitive closure (100 edges) | ~500 µs | ≤ 500 µs | ≤ 400 µs |
| OWL RL closure materialization | ~30 ms | ≤ 30 ms | ≤ 25 ms |

---

## Success Criteria (Final)

- [ ] Audit Step 0 is complete and documents dense closure site(s) or affirms none exist
- [ ] If dense site found:
  - [ ] `fixedbitset` dependency added to `Cargo.toml`
  - [ ] `ClosureMatrix` struct with transitive closure computation
  - [ ] Canonical rendering rule documented and tested
  - [ ] Dense closure sites replaced with `ClosureMatrix`
  - [ ] All closure tests pass (set-equality verified)
  - [ ] Benchmarks run; no regressions
- [ ] If no dense site: PROJ-409 marked REFUSED with clear documentation

---

## Acceptance Criteria

- [ ] Audit is thorough and decision (proceed vs defer) is justified
- [ ] Code review: transitive closure algorithm is sound
- [ ] Canonical rendering rule is enforced (no raw bitset hashing)
- [ ] Standing: `standing.json` reports PROJ-409 as COMPLETE (if dense site) or REFUSED (if none)

---

## Standing Rules

- **ALIVE**: PROJ-409 is ALIVE when audit affirms a dense site, `ClosureMatrix` is integrated, canonical rendering is correct, and benchmarks pass
- **REFUSED**: PROJ-409 is REFUSED if audit finds no dense closure sites (no `fixedbitset` dependency added; HashSet closures sufficient)
- **PARTIAL_ALIVE**: If audit is inconclusive (e.g., closure density is borderline), document the boundary conditions and gate depending on deployment profile (small vs large graph)

---

## Related Tickets

- PROJ-401: Quick-Win Crate Optimizations (COMPLETE; deferred this ticket pending audit)
- PROJ-403: Compiled Hook IR (may depend on hook-dependency transitive closure, if that's a dense site)
- PROJ-407: Compiled Shape IR (may use subClassOf closure for SHACL validation)

---

## References

- PROJ-401 ticket: documents the "no speculative dependency" rule that gates PROJ-409
- `crates/praxis-graphlaw/src/shacl.rs:777`: `SubclassClosure` (potential target for replacement)
- `fixedbitset` crate: https://docs.rs/fixedbitset/latest/fixedbitset/
- Article: "Graphlaw Performance Architecture" — Section "Bitset Closure and Canonical Rendering"

---

## Audit Subtasks (for Step 0)

1. Search all `.rs` files for `HashSet<.*Id>`, `BTreeSet<.*Id>`, closure-related patterns
2. For each closure operation, estimate cardinality (e.g., OWL RL class count in a typical graph)
3. Determine if bitset allocation would be sparse (> 50% of IDs unused) or dense (< 50% unused)
4. Document findings in a markdown table (Operation, Cardinality Estimate, Density Class, Recommendation)
5. Decision: if any dense site found, proceed; otherwise defer

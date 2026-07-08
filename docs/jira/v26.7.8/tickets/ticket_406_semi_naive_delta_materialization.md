# PROJ-406 — Semi-Naive Delta Materialization + Duplicate Derivation Suppression (v26.7.8 P1)

**Status**: PLANNED  
**Scope**: Introduce `FactStore` with explicit delta/all sets; add `DerivationGate` for canonical provenance tracking  
**Dependencies**: PROJ-405 (Compiled Rule IR)  
**Target**: P1 — foundation for efficient incremental updates

---

## Overview

Today the materialization loop (`reasoner/mod.rs:33-612`) uses integer counter offsets (`stratum_start_counter`, `next_start_counter`) to track semi-naive deltas within `TripleIndex.triples: Vec<Triple>`. This ticket introduces an explicit `FactStore` with delta/all sets and a `DerivationGate` that records canonical `(fact, rule, sorted-premises)` traces to suppress duplicate derivations.

**Doctrine source:** "Graphlaw Performance Architecture: Small IDs, Compiled IR, Bounded Profiles, and Canonical Standing" (article).

---

## Deliverables & Tests

### Step 1: FactStore Structure

**Deliverables:**
- Define `struct FactStore`:
  ```rust
  pub struct FactStore {
    pub all_facts: FxHashSet<Triple<SymbolId>>,
    pub delta: FxHashSet<Triple<SymbolId>>,  // new facts this round
  }
  ```
- Add methods:
  - `add_fact(fact: Triple) → bool` — returns true if fact is new (not in `all_facts`), adds to both `all_facts` and `delta`
  - `take_delta() → FxHashSet<Triple>` — drain and return `delta`, reset to empty
  - `all(&self) → &FxHashSet<Triple>` — read-only access to `all_facts`
  - `delta(&self) → &FxHashSet<Triple>` — read-only access to current `delta`
- Maintain a single `FactStore` instance per materialization run (replacing the current `TripleIndex.triples: Vec<Triple>` + counter approach)

**Tests:**
- Roundtrip: add facts A, B, C → delta contains {A,B,C} → take_delta returns {A,B,C} → delta is empty
- Duplicate suppression: add fact A twice → `add_fact` returns true, then false → `all_facts` contains only one copy
- Delta isolation: facts added before `take_delta` are not in the next round's delta

**Acceptance:**
- `FactStore` compiles and integrates into `TripleStore`
- Existing triple-store tests still pass
- Delta tracking is correct (no lost facts, no spurious duplicates)

---

### Step 2: Derivation Gate and Canonical Trace

**Deliverables:**
- Define `struct DerivationGate`:
  ```rust
  pub struct DerivationGate {
    pub derivations: FxHashMap<Triple<SymbolId>, CanonicalDerivation>,
  }
  ```
- Define `struct CanonicalDerivation`:
  ```rust
  pub struct CanonicalDerivation {
    pub fact: Triple<SymbolId>,
    pub rule_id: RuleId,
    pub sorted_premises: Vec<Triple<SymbolId>>,  // bound variables that triggered this derivation
    pub timestamp: /* derivation round */,
  }
  ```
- Add methods:
  - `admit_derivation(fact: Triple, rule_id: RuleId, premises: Vec<Triple>) → bool` — returns true if this is the first time we've seen this `(fact, rule, sorted-premises)` combination, records it, and blocks future identical derivations; returns false if already recorded
  - `all_derivations(&self) → &[(Triple, CanonicalDerivation)]` — read-only access for provenance audits

**Tests:**
- Canonical trace: derive fact A via rule R with premises {P1, P2} → admission succeeds → second identical derivation is rejected
- Duplicate derivation (different rule, same fact): derive A via R1 then R2 → both are recorded separately (different rules, different records)
- Premise order: `sorted_premises` is consistently sorted (canonically ordered) so {P1, P2} and {P2, P1} are recognized as the same premise set

**Acceptance:**
- `DerivationGate` correctly suppresses duplicate derivations
- Each `(fact, rule, sorted-premises)` combination is recorded once
- Provenance traces are canonical (sorted and reproducible)

---

### Step 3: Integrate FactStore and DerivationGate into Materialization Loop

**Deliverables:**
- Replace `TripleIndex.triples: Vec<Triple>` + counter-based deltas with `FactStore`
- Update `Reasoner::materialize` (`reasoner/mod.rs:33-612`) to:
  1. Initialize `FactStore` with base facts
  2. For each stratum:
     - For each rule:
       - Query `FactStore.delta()` (new facts from prior round) against `FactStore.all()` (all known facts) — existing semi-naive query
       - For each derived head fact:
         - Call `DerivationGate::admit_derivation(head, rule_id, premises)` — if it returns true, add to `FactStore.add_fact(head)`
  3. Repeat until `FactStore.take_delta()` is empty (fixpoint)
- Update `TripleIndex` and `query_semi_naive` to use `FactStore` methods instead of raw `Vec` offsets
- Update existing code paths that assume `TripleIndex.triples: Vec<Triple>` to use `FactStore.all()` instead

**Tests:**
- Existing semi-naive materialization tests pass (facts computed are identical)
- Delta-driven correctness: each round's delta is correctly identified and processed
- Fixpoint detection: materialization terminates when delta is empty
- Provenance: `DerivationGate` records are consistent with derived facts
- Benchmarks: `test_transitive_rule` (baseline 60.4 ms); latency unchanged or improved

**Acceptance:**
- All materialization tests pass
- Facts derived are identical to baseline (set-equality)
- Fixpoint is reached correctly (no missing facts, no infinite loop)
- Provenance traces are canonical and reproducible

---

### Step 4: Document Delta-Driven Loop and Duplicate Suppression Rules

**Deliverables:**
- Add a comment block in `reasoner/mod.rs` documenting the delta-driven semi-naive algorithm
- Document the invariant: "All facts in delta are new (not in prior rounds); fixpoint is when delta is empty"
- Document the deduplication rule: "Each `(fact, rule, sorted-premises)` is admitted once; re-derivation via the same rule is suppressed"
- Add a reference to the article's "Canonical Standing" section

**Tests:**
- Documentation review: algorithm is clearly explained and invariants are explicit
- Code matches documentation (no surprise edge cases)

**Acceptance:**
- Algorithm is documented for future maintainers
- Invariants are clear and enforced

---

## Benchmark Targets (Three-Tier Structure)

| Benchmark | Baseline | Target (P1) | Target (P1.5) |
|-----------|----------|------------|---------------|
| Materialize transitive rule | ~60.4 ms | ≤ 60.4 ms | ≤ 55 ms (FactStore overhead negligible) |
| Blue River Dam delta | ~304 µs | ≤ 304 µs | ≤ 275 µs |
| Hierarchy 1000 | ~8.3 ms | ≤ 8.3 ms | ≤ 7.5 ms |

---

## Success Criteria (Final)

- [ ] `FactStore` struct with `add_fact`, `take_delta`, `all`, `delta` methods defined
- [ ] `DerivationGate` with canonical provenance tracking implemented
- [ ] Materialization loop refactored to use `FactStore` and `DerivationGate`
- [ ] Semi-naive query paths use `FactStore.delta()` and `FactStore.all()`
- [ ] All materialization tests pass (facts are identical to baseline)
- [ ] Fixpoint detection is correct (no infinite loops, no missing facts)
- [ ] Benchmarks run; no regressions
- [ ] Algorithm and invariants documented

---

## Acceptance Criteria

- [ ] Code review: no unsafe code; delta tracking is sound
- [ ] Integration: PROJ-407 and downstream tickets can assume `FactStore` and provenance traces
- [ ] Standing: `standing.json` reports PROJ-406 as COMPLETE after `just verify-all` passes

---

## Standing Rules

- **ALIVE**: PROJ-406 is ALIVE when `FactStore` and `DerivationGate` are integrated, all materialization tests pass, and benchmarks are stable
- **PARTIAL_ALIVE**: If duplicate suppression has edge cases (e.g., certain premise orders), document and fix before PROJ-410 (Standing Hardening) depends on it
- **REFUSED**: If materialization produces incorrect facts or infinite loops, refuse and debug

---

## Related Tickets

- PROJ-405: Compiled Rule IR (required dependency; provides ordered rule bodies)
- PROJ-407: Compiled Shape IR (uses same delta-driven validation loop)
- PROJ-410: Canonical Standing Boundary Hardening (uses provenance traces from DerivationGate)

---

## References

- `crates/praxis-graphlaw/src/reasoner/mod.rs:33-612`: Materialization loop (main target)
- `crates/praxis-graphlaw/src/reasoner/mod.rs:50-608`: Stratified fixpoint loop with counter offsets
- `crates/praxis-graphlaw/src/tripleindex.rs:11-17`: `TripleIndex` (to be refactored)
- `crates/praxis-graphlaw/src/queryengine/mod.rs:292`: `query_semi_naive` (to use `FactStore`)
- Article: "Graphlaw Performance Architecture" — Sections "FactStore" and "DerivationGate"

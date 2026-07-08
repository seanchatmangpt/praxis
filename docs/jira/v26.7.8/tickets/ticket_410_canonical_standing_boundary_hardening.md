# PROJ-410 — Canonical Standing Boundary Hardening (v26.7.8 P1/Cross-Cutting)

**Status**: PLANNED  
**Scope**: Introduce `RuntimeState`/`StandingState` separation; generalize `DiagnosticBuffer`/`CanonicalReceiptMaterial` builders; document canonical-sort gate for future parallelism  
**Dependencies**: PROJ-401 (COMPLETE); PROJ-406 (Semi-Naive Delta Materialization)  
**Target**: P1/Cross-cutting — foundation for standing/diagnostics hardening and safe parallelism

---

## Overview

Today `TripleStore` (`lib.rs:61-72`) is a single monolithic struct bundling rules, indices, reasoner, hooks, receipts, verdicts, etc. with no runtime-vs-standing split. Hook receipts are built via ad-hoc `canonicalize_quads` calls and blake3-hashed (`reasoner/mod.rs:565,575-580`), and there is no documented gate for canonical-key sorting before parallelism (today there is no rayon dependency, but this ticket lays the groundwork for safe parallelism when it's added in the future). This ticket introduces:
1. A `RuntimeState` / `StandingState` separation to isolate mutable and immutable concerns
2. A generalized `DiagnosticBuffer`/`CanonicalReceiptMaterial` builder pattern (extending the existing `HookReceipt` approach)
3. A `Scratch` per-round arena for rule/validation temporaries
4. A documented (docs-only, no new dependency) gate requiring canonical-key sorting before any measured parallelism

**Doctrine source:** "Graphlaw Performance Architecture: Small IDs, Compiled IR, Bounded Profiles, and Canonical Standing" (article).

---

## Deliverables & Tests

### Step 1: RuntimeState / StandingState Separation

**Deliverables:**
- Define `struct RuntimeState`:
  ```rust
  pub struct RuntimeState {
    pub triple_index: TripleIndex,  // mutable fact store
    pub derivation_log: DerivationGate,  // from PROJ-406
    pub round_scratch: Scratch,  // per-round temporaries
  }
  ```
- Define `struct StandingState`:
  ```rust
  pub struct StandingState {
    pub rules: Vec<CompiledRule>,  // from PROJ-405
    pub shapes: Vec<CompiledShape>,  // from PROJ-407
    pub hooks: Vec<CompiledHook>,  // from PROJ-403
    pub iri_interner: SymbolTable,  // from PROJ-401
    pub receipts: Vec<CanonicalReceipt>,  // canonical materialization receipts
  }
  ```
- Refactor `TripleStore` to wrap `RuntimeState` and `StandingState`:
  ```rust
  pub struct TripleStore {
    pub runtime: RuntimeState,
    pub standing: StandingState,
  }
  ```
- Document: "RuntimeState is mutable and round-dependent; StandingState is immutable and reusable across multiple materialization runs."

**Tests:**
- Existing `TripleStore` tests pass (field paths updated to `ts.runtime.triple_index`, `ts.standing.rules`, etc.)
- No semantic change; refactoring only affects internal organization

**Acceptance:**
- Code compiles with new state separation
- All existing tests pass
- Field access patterns are clear and consistent

---

### Step 2: DiagnosticBuffer and CanonicalReceiptMaterial Builders

**Deliverables:**
- Define `struct DiagnosticBuffer`:
  ```rust
  pub struct DiagnosticBuffer {
    pub hook_records: Vec<(HookId, Vec<Triple>)>,  // per-hook, canonical facts added
    pub validation_records: Vec<(ShapeId, Vec<ValidationError>)>,  // per-shape errors
    pub derivation_traces: Vec<CanonicalDerivation>,  // from DerivationGate
  }
  ```
- Define `trait CanonicalReceiptMaterial`:
  ```rust
  pub trait CanonicalReceiptMaterial {
    fn to_canonical_form(&self) -> Vec<u8>;  // sorted/canonical bytes for hashing
  }
  ```
- Implement `CanonicalReceiptMaterial` for:
  - `Vec<Triple>` — canonically sort by (s, p, o)
  - `Vec<CanonicalDerivation>` — canonically sort by (fact, rule, premises)
  - `DiagnosticBuffer` — concatenate canonical forms of each record
- Generalize the existing `HookReceipt` pattern (`hooks.rs:667-672`, built via `canonicalize_quads`) to use the new `CanonicalReceiptMaterial` trait

**Tests:**
- Roundtrip: `DiagnosticBuffer` → `to_canonical_form()` → BLAKE3 hash → byte-identical across runs
- Determinism: same facts/errors/derivations → same canonical form
- Hook receipts: refactor existing `HookReceipt` to use `CanonicalReceiptMaterial`; verify byte-identical to prior receipts

**Acceptance:**
- `CanonicalReceiptMaterial` trait is implemented correctly
- Canonical forms are deterministic and reproducible
- All existing receipt tests pass with byte-identical outputs

---

### Step 3: Scratch Arena for Per-Round Temporaries

**Deliverables:**
- Define `struct Scratch`:
  ```rust
  pub struct Scratch {
    pub var_bindings: FxHashMap<Var, SymbolId>,  // working bindings during rule evaluation
    pub temp_facts: FxHashSet<Triple>,  // temporary facts during one round
    pub join_results: Vec<Vec<SymbolId>>,  // pre-allocated join result buffers
  }
  ```
- Implement arena-like allocation:
  - `Scratch::new()` — allocate
  - `Scratch::reset()` — clear all fields but keep allocations
  - Use `reset()` at the start of each materialization round (no heap re-allocation)
- Update materialization loop (`reasoner/mod.rs`) to use `RuntimeState.round_scratch` instead of creating temporary HashMap/HashSet each round

**Tests:**
- Scratch allocation: verify that `Scratch::reset()` does not re-allocate (reuses underlying Vec/HashMap capacity)
- Materialize 100 rounds: verify no spurious allocations beyond the initial `Scratch::new()`
- Correctness: facts derived using scratch temporaries match baseline (no missing/spurious facts)

**Acceptance:**
- Scratch arena reduces allocation churn per round
- Correctness is verified
- No regressions in materialization latency

---

### Step 4: Canonical-Sort Gate for Future Parallelism

**Deliverables:**
- Write a standing rule (docs-only, no new dependency) in `docs/standing/CANONICAL_PARALLELISM_GATE.md`:
  ```markdown
  # Canonical-Sort Gate for Measured Parallelism

  When rayon (or any parallelism library) is added to praxis-graphlaw:

  1. Every parallel batch (e.g., `rayon::par_iter`) MUST sort results by a canonical key
     before writing to standing state (receipts, DiagnosticBuffer, TripleStore).

  2. Canonical key examples:
     - For triples: (s_id, p_id, o_id) lexicographic order
     - For derivations: (fact_id, rule_id, premises_id_hash) order
     - For validation errors: (shape_id, node_id) order

  3. Example:
     ```rust
     // WRONG: parallel results may have nondeterministic order
     let results = (0..batches)
       .par_iter()
       .map(|batch| compute(batch))
       .collect::<Vec<_>>();
     standing.receipts.extend(results);  // BAD: nondeterministic order

     // RIGHT: canonical sort before writing to standing
     let mut results = (0..batches)
       .par_iter()
       .map(|batch| compute(batch))
       .collect::<Vec<_>>();
     results.sort_by_key(|r| canonical_key(r));  // Canonical order
     standing.receipts.extend(results);  // OK: deterministic
     ```

  4. Audit checklist before adding rayon:
     - [ ] All parallel batch code sorts results before standing writes
     - [ ] Canonical keys are documented per result type
     - [ ] Tests verify byte-identical receipts across multiple runs
  ```
- Add this rule to `docs/standing/` (or append to `REALITY_INDEX.md`)
- Document prior art: `lib.rs:334-352` sorts output for diffable canonical rendering; cite this as precedent
- Add a comment block in key materialization/evaluation sites (e.g., `reasoner/mod.rs`, `shacl.rs:1473`) referencing this gate

**Tests:**
- No tests needed (documentation rule); but verify:
  - Rule is findable in `docs/standing/`
  - Comments in code link to the rule
  - Future parallelism work checks this rule before adding rayon

**Acceptance:**
- Standing rule is documented and linked from code
- Expectation is clear for future parallelism work

---

### Step 5: Integration Test: Round-Trip State Separation

**Deliverables:**
- Add integration test covering:
  - Load rules, shapes, hooks into `StandingState`
  - Materialize facts into `RuntimeState`
  - Build `DiagnosticBuffer` and canonical receipt
  - Verify `TripleStore` field access paths work correctly
  - Verify round-to-round state separation (reset `RuntimeState.round_scratch` between rounds, not `StandingState`)

**Tests:**
- 5-10 new integration tests covering state separation scenarios

**Acceptance:**
- All new tests pass
- State separation is working as designed

---

## Benchmark Targets (Three-Tier Structure)

| Benchmark | Baseline | Target (P1) | Target (P1.5) |
|-----------|----------|------------|---------------|
| Materialize transitive rule (60.4 ms baseline) | 60.4 ms | ≤ 60.4 ms | ≤ 58 ms (scratch reuse savings) |
| Build diagnostic buffer (10 hooks × 100 facts) | ~10 ms | ≤ 10 ms | ≤ 9 ms |
| Canonical receipt generation | ~5 ms | ≤ 5 ms | ≤ 4.5 ms |

---

## Success Criteria (Final)

- [ ] `RuntimeState` and `StandingState` structs defined
- [ ] `TripleStore` refactored to wrap both states
- [ ] `DiagnosticBuffer` and `CanonicalReceiptMaterial` trait implemented
- [ ] Existing `HookReceipt` refactored to use `CanonicalReceiptMaterial`
- [ ] `Scratch` arena allocated once per materialization, reset per round
- [ ] Canonical-sort parallelism gate documented in `docs/standing/`
- [ ] All existing tests pass
- [ ] 5-10 new integration tests added for state separation
- [ ] Benchmarks run; no regressions

---

## Acceptance Criteria

- [ ] Code review: state separation is sound; no cross-boundary data leaks
- [ ] Canonical receipt material is deterministic (byte-identical across runs)
- [ ] Scratch arena reduces allocation churn without correctness impact
- [ ] Standing rule is documented and linked from key code sites
- [ ] Standing: `standing.json` reports PROJ-410 as COMPLETE after `just verify-all` passes

---

## Standing Rules

- **ALIVE**: PROJ-410 is ALIVE when state separation is integrated, canonical receipt builders work, scratch arena is active, and all tests pass
- **PARTIAL_ALIVE**: If state separation exposes edge cases (e.g., some fields belong in both states), document and refactor until clear
- **REFUSED**: If refactoring breaks existing tests or introduces regressions, refuse and debug

---

## Related Tickets

- PROJ-401: Quick-Win Crate Optimizations (COMPLETE; provides foundation)
- PROJ-405: Compiled Rule IR (feeds rules into `StandingState`)
- PROJ-406: Semi-Naive Delta Materialization (provides `DerivationGate` for diagnostic buffer)
- PROJ-407: Compiled Shape IR (feeds shapes into `StandingState`)

---

## References

- `crates/praxis-graphlaw/src/lib.rs:61-72`: `TripleStore` (target for refactoring)
- `crates/praxis-graphlaw/src/lib.rs:334-352`: Canonical sorted output for diffability (prior art for canonical-sort gate)
- `crates/praxis-graphlaw/src/hooks.rs:667-672`: `HookReceipt` (existing pattern to generalize)
- `crates/praxis-graphlaw/src/reasoner/mod.rs:33-612`: Materialization loop (uses `Scratch`)
- Article: "Graphlaw Performance Architecture" — Sections "RuntimeState/StandingState Separation" and "Canonical Standing Boundary"

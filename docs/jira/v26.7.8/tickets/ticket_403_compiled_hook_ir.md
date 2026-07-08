# PROJ-403 — Compiled Hook IR (v26.7.8 P0)

**Status**: PLANNED  
**Scope**: Introduce `HookId`/`EventId` newtypes and ID-based hook scheduler, add `smallvec` dependency  
**Dependencies**: PROJ-401 (COMPLETE)  
**Target**: P0 — unblock PROJ-404

---

## Overview

Today hooks are identified by string IRI (`hooks.rs:214`), collected in `Vec<KnowledgeHook>`, and scheduled via string-keyed topological sort (`schedule_hooks`, `hooks.rs:533-586`). This ticket introduces `HookId`/`EventId` newtypes, a `CompiledHook` struct, and an ID-based scheduler to replace string-keyed lookups and tie-breaks.

**Doctrine source:** "Graphlaw Performance Architecture: Small IDs, Compiled IR, Bounded Profiles, and Canonical Standing" (article).

---

## Deliverables & Tests

### Step 1: Hook ID Types and Compiled Hook Struct

**Deliverables:**
- Define `newtype! HookId(u32)` and `newtype! EventId(u32)` for hook/event identity
- Define `struct CompiledHook`:
  ```rust
  pub struct CompiledHook {
    pub id: HookId,
    pub iri: SymbolId,  // from PROJ-401 interning
    pub event: EventId,
    pub priority: u8,
    pub after: SmallVec<[HookId; 4]>,  // hook dependencies
    pub condition: /* ... */,
    pub effect: /* ... */,
  }
  ```
- Introduce `smallvec` crate (v1) as a new dependency in `Cargo.toml`
- Maintain a global or in-scope `HookId → CompiledHook` table during materialization

**Tests:**
- Roundtrip: parse `KnowledgeHook` from RDF → assign `HookId` → compile to `CompiledHook` → render back to RDF ≈ original
- Hook ID uniqueness: no two hooks assigned the same `HookId`
- SmallVec inline storage verified: `sizeof(SmallVec<[HookId;4]>)` ≤ pointer-sized overhead for 0-4 elements
- Existing hook trigger tests (79 tests in `tests/knowledge_hooks_e2e.rs`) pass unchanged

**Acceptance:**
- All 79 existing hook tests pass
- `CompiledHook` struct compiles with no compiler errors
- Receipt hashes match baseline (no ID assignment introduces nondeterminism)

---

### Step 2: ID-Based Hook Scheduler

**Deliverables:**
- Rewrite `schedule_hooks` to operate on `&[CompiledHook]` instead of `&[KnowledgeHook]`
- Use `HookId` and `after: SmallVec<[HookId; 4]>` directly for dependency tracking (no string comparisons)
- Kahn's-algorithm topo sort on `HookId` edges; tie-break by `(priority: u8, id: HookId)` lexicographic order (not `(priority, iri_string)`)
- Verify no unknown dependency (check all `after` IDs exist in the hook table)
- Return ordered `Vec<CompiledHook>` in execution order

**Tests:**
- Existing ordering tests: hooks with explicit `after` dependencies fire in order
- Tie-break: equal-priority hooks with no `after` edge fire in ascending `HookId` order
- Cycle detection: attempting to schedule a cyclic dependency set rejects with `Refusal::HookSchedulingCycle`
- Bench target: `schedule_hooks` latency unchanged or improved (baseline: `schedule_hooks` in `bencher` suite, currently ~negligible, < 1µs for 10 hooks)

**Acceptance:**
- Existing hook ordering semantics preserved
- No string-keyed comparisons in the scheduler
- Cycle/unknown-dependency detection still fires with clear error

---

### Step 3: Integrate CompiledHook into Hook Evaluation

**Deliverables:**
- Update `evaluate_hooks` (`hooks.rs:1518`) to iterate over `&[CompiledHook]` instead of `&[KnowledgeHook]`
- Update `HookVerdictRecord` to store `hook_id: HookId` instead of `hook_iri: String` and `hook_name: String` (keep string rendering for logs/receipts only)
- Update `HookReceipt` to track which `HookId`s added/retracted facts (not strings)
- Maintain canonical rendering: `HookId → SymbolId → string` for any external output

**Tests:**
- Hook execution: condition evaluation is functionally identical before/after
- Hook receipts: byte-identical to baseline (same facts added/retracted, same order)
- Verdict records: track `HookId` correctly and render to strings on demand
- Existing 79 hook tests pass

**Acceptance:**
- Hook evaluation semantics preserved
- Receipts stable and verifiable
- All existing tests pass

---

## Benchmark Targets (Three-Tier Structure)

| Benchmark | Baseline | Target (P0) | Target (P1) |
|-----------|----------|------------|------------|
| Schedule 10 hooks | ~0.5 µs | ≤ 0.5 µs (no regression) | ≤ 0.3 µs (string-elim speedup) |
| Evaluate 10 hooks (baseline N3 rule set) | ~1 ms | ≤ 1 ms (no regression) | ≤ 0.95 ms |
| Hook receipt generation (10 hooks × 100 facts) | ~10 ms | ≤ 10 ms | ≤ 9 ms |

---

## Success Criteria (Final)

- [ ] `HookId`, `EventId`, `CompiledHook` struct defined and compile-clean
- [ ] `schedule_hooks` rewritten to use `HookId` (no string comparisons)
- [ ] `evaluate_hooks` updated to use `CompiledHook`
- [ ] All 79 existing hook tests pass
- [ ] Hook receipts byte-identical to baseline
- [ ] `smallvec` dependency added to `Cargo.toml`; no other new dependencies added
- [ ] Benchmark suite runs; no regressions in hook latency

---

## Acceptance Criteria

- [ ] Code review: no unsafe code outside `CompiledHook` struct internals (if any); all ID assignments verified deterministic
- [ ] Integration: `PROJ-404` can assume `CompiledHook` and `HookId` as a stable foundation
- [ ] Standing: `standing.json` reports PROJ-403 as COMPLETE after `just verify-all` passes

---

## Standing Rules

- **ALIVE**: PROJ-403 is ALIVE when all three Deliverable steps pass their Acceptance criteria and 79 hook tests pass
- **PARTIAL_ALIVE**: If hook scheduling works but evaluation has issues (unlikely), document and gate PROJ-404
- **REFUSED**: If `smallvec` addition causes dependency conflict or build failure, refuse and revert to bounded `Vec` + a comment

---

## Related Tickets

- PROJ-401: Quick-Win Crate Optimizations (COMPLETE; provides `SymbolId` type)
- PROJ-404: Compiled Condition IR (depends on this)
- PROJ-408: Compiled Delta Template IR (depends on this for template-based hook actions)

---

## References

- `crates/praxis-graphlaw/src/hooks.rs`: Hook representation and scheduling (lines 214-586)
- `crates/praxis-graphlaw/src/hooks.rs:1518-1556`: Hook evaluation
- `crates/praxis-graphlaw/tests/knowledge_hooks_e2e.rs`: Hook acceptance tests (72 tests)
- Article: "Graphlaw Performance Architecture" — Section "CompiledHook Structure"
- `smallvec` crate: https://docs.rs/smallvec/latest/smallvec/

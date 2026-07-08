# PROJ-408 — Compiled Delta Template IR (v26.7.8 P1)

**Status**: PLANNED  
**Scope**: Replace runtime placeholder-string scanning with pre-compiled binding-slot templates for hook action/effect projection  
**Dependencies**: PROJ-403 (Compiled Hook IR)  
**Target**: P1 — optimize hook action rendering

---

## Overview

Today hook effects (actions that add/retract/derive facts) are templated with placeholder strings like `?0`, `?1` that are scanned at runtime during fact projection. This ticket introduces `TemplatePart`, `CompiledDeltaTemplate`, and `CompiledTripleTemplate` to pre-parse templates at compile time and map placeholders to binding slots, so projection is a direct slot-lookup instead of string scanning.

**Doctrine source:** "Graphlaw Performance Architecture: Small IDs, Compiled IR, Bounded Profiles, and Canonical Standing" (article).

---

## Deliverables & Tests

### Step 1: Template Compilation

**Deliverables:**
- Define `enum TemplatePart`:
  ```rust
  pub enum TemplatePart {
    Literal { value: SymbolId },  // constant IRI/string
    Binding { slot: usize },       // ?0, ?1, ... (index into result bindings)
  }
  ```
- Define `struct CompiledTripleTemplate`:
  ```rust
  pub struct CompiledTripleTemplate {
    pub subject: TemplatePart,
    pub predicate: TemplatePart,
    pub object: TemplatePart,
  }
  ```
- Define `struct CompiledDeltaTemplate`:
  ```rust
  pub struct CompiledDeltaTemplate {
    pub triples: Vec<CompiledTripleTemplate>,
    pub max_binding_slot: usize,  // highest slot index (e.g., 3 for ?0..?3)
  }
  ```
- Compile hook effects (e.g., `?0 ex:links ?1`) into `CompiledDeltaTemplate` at hook load time
- Validate at compile time: all placeholders reference valid binding slots (slots that are actually bound by the hook condition)

**Tests:**
- Parse roundtrip: template string → `CompiledDeltaTemplate` → render back to template string ≈ original
- Binding slot validation: attempt to compile a template with `?5` when only 3 bindings exist → reject with clear error at load time
- Determinism: same template string → same `CompiledDeltaTemplate` every time

**Acceptance:**
- `CompiledDeltaTemplate` and `CompiledTripleTemplate` compile
- Template validation is correct (no out-of-bounds slot refs)
- All valid templates parse correctly

---

### Step 2: Template Projection During Hook Evaluation

**Deliverables:**
- At hook evaluation time (when a condition matches and bindings are available):
  - For each `CompiledTripleTemplate`:
    - For each `TemplatePart`: if `Literal`, use the constant `SymbolId`; if `Binding { slot }`, look up `bindings[slot]`
    - Construct the concrete triple `(subject, predicate, object)`
  - Add the triple to the hook effect (add/retract/derive as specified)
- Replace all runtime placeholder-scanning code with this direct slot-lookup projection
- Ensure projection is deterministic (same bindings → same projected facts every time)

**Tests:**
- Projection correctness: template `?0 ex:links ?1` with bindings `[node_A, node_B]` projects to `(node_A, ex:links, node_B)`
- Determinism: same template + bindings → same projection every run
- Existing hook effect tests pass (subset of 79 hook tests in `tests/knowledge_hooks_e2e.rs`)
- Benchmark: hook action projection latency (baseline: negligible, < 1 µs per projection); no regression

**Acceptance:**
- Projection is correct and deterministic
- All existing hook tests pass
- No regression in latency

---

### Step 3: Error Handling for Template Validity

**Deliverables:**
- At hook load time, if a template contains invalid placeholders:
  - Reject with `Refusal::InvalidHookTemplate { reason: String }`
  - Refuse the entire hook (do not allow it to be scheduled)
- Document: template validity is checked at compile/load time, not at projection time
- Add a clear error message: "Hook template contains undefined placeholder ?5 (only 4 bindings available)"

**Tests:**
- Load a hook with an invalid template → rejected at load time with clear message
- Existing hook tests still pass

**Acceptance:**
- Invalid templates are rejected early (at load time)
- Error messages are clear and actionable

---

### Step 4: Integration into Hook Compilation (PROJ-403)

**Deliverables:**
- Update `CompiledHook` (from PROJ-403) to store `effect: CompiledDeltaTemplate` (instead of raw template string)
- Update `validate_and_extract_hooks` to compile hook effects into `CompiledDeltaTemplate` as part of hook load
- Update `evaluate_hooks` to use `CompiledDeltaTemplate` projection (direct slot lookup, no string scanning)

**Tests:**
- Roundtrip: RDF hook definition → `CompiledHook` with compiled effect → projection → output ≈ input
- All 79 existing hook tests pass
- Benchmark: hook action evaluation unchanged or improved

**Acceptance:**
- Hook compilation and evaluation use compiled templates
- All existing tests pass

---

## Benchmark Targets (Three-Tier Structure)

| Benchmark | Baseline | Target (P1) | Target (P1.5) |
|-----------|----------|------------|---------------|
| Project 10 hook actions (100 facts) | ~1 ms | ≤ 1 ms | ≤ 0.9 ms (slot-lookup vs string scan) |
| Load 20 hooks (with templates) | ~2 ms | ≤ 2 ms | ≤ 1.8 ms |
| Hook receipt generation with templates | ~10 ms | ≤ 10 ms | ≤ 9 ms |

---

## Success Criteria (Final)

- [ ] `TemplatePart`, `CompiledTripleTemplate`, `CompiledDeltaTemplate` structs defined
- [ ] Template parsing and compilation is deterministic
- [ ] Template projection uses direct slot lookup (no string scanning)
- [ ] Template validation is done at load time (invalid templates rejected early)
- [ ] All 79 existing hook tests pass
- [ ] Benchmarks run; no regressions
- [ ] Error messages for invalid templates are clear

---

## Acceptance Criteria

- [ ] Code review: template compilation is sound (no edge cases in slot mapping)
- [ ] Integration: hook effects are fully compiled and projection is slot-based
- [ ] Standing: `standing.json` reports PROJ-408 as COMPLETE after `just verify-all` passes

---

## Standing Rules

- **ALIVE**: PROJ-408 is ALIVE when `CompiledDeltaTemplate` is integrated into hook evaluation, projection uses direct slot lookup, and all tests pass
- **PARTIAL_ALIVE**: If edge cases in template parsing emerge, document them and gate PROJ-410
- **REFUSED**: If projection produces incorrect facts, refuse and debug

---

## Related Tickets

- PROJ-403: Compiled Hook IR (required dependency; provides `CompiledHook` struct)
- PROJ-401: Quick-Win Crate Optimizations (COMPLETE; provides ID-based foundation)

---

## References

- `crates/praxis-graphlaw/src/hooks.rs:214-224`: `KnowledgeHook` struct (today's template field)
- `crates/praxis-graphlaw/src/hooks.rs:1518-1556`: Hook evaluation and effect projection (today's string-scanning location)
- `crates/praxis-graphlaw/tests/knowledge_hooks_e2e.rs`: Hook acceptance tests (focus on effect/action tests)
- Article: "Graphlaw Performance Architecture" — Section "CompiledDeltaTemplate and Template Compilation"

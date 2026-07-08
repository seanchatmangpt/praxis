# PROJ-405 — Compiled Rule IR + Join Selectivity Ordering (v26.7.8 P1)

**Status**: IN_PROGRESS  
**Scope**: Introduce `CompiledRule` with `Selectivity` enum; reorder rule body patterns by selectivity heuristic  
**Dependencies**: PROJ-401 (COMPLETE; provides ID-based pattern matching)  
**Target**: P1 — foundational for semi-naive and delta materialization

**SCOPED**: SymbolId assumed in ticket text does not exist; using String representations. No interner side effect added.  
**IMPLEMENTATION STATUS**: Core structures (Selectivity enum, PatternStep, CompiledRule) and functions (order_body_patterns, classify_pattern_selectivity) implemented and tested in rule.rs. Integration into RuleIndex, backwardchaining.rs, queryengine/mod.rs, and csprite.rs pending.

---

## Overview

Today Datalog/N3 rules are evaluated by iterating the rule body left-to-right with no selectivity analysis (e.g., `backwardchaining.rs:238`, `csprite.rs:231,292`). This ticket introduces `CompiledRule` (head, body, driving_atom, selectivity) and an `order_body_patterns` heuristic to reorder body atoms by selectivity (exact matches first, then predicate+object, then subject+predicate, ..., finally full-scan patterns).

**Doctrine source:** "Graphlaw Performance Architecture: Small IDs, Compiled IR, Bounded Profiles, and Canonical Standing" (article).

---

## Deliverables & Tests

### Step 1: Selectivity Enum and Classification

**Deliverables:**
- Define `enum Selectivity`:
  ```rust
  pub enum Selectivity {
    Exact { bound_vars: usize },           // All vars bound (fact lookup)
    PredicateObject { s_unbound: bool },   // p,o bound (2 bindings)
    SubjectPredicate { o_unbound: bool },  // s,p bound (2 bindings)
    PredicateOnly { so_unbound: bool },    // p bound (1 binding)
    OneUnbound,                             // 1 var bound anywhere
    FullScan,                               // 0 vars bound (Cartesian)
  }
  ```
- Add classification function `classify_pattern_selectivity(pattern: &Triple, bound_vars: &HashSet<Var>) → Selectivity` to assess each body pattern against already-bound variables
- Document tie-break: if two patterns have the same selectivity, order by pattern index (preserve relative order, stable sort)

**Tests:**
- Pattern classification: exact → pred+obj → ... → full-scan sequence verified
- Bound-var propagation: after evaluating pattern A, variables bound by A are marked as bound for pattern B selectivity classification
- Roundtrip: original rule → compile → order → compute selectivity of each pattern in order

**Acceptance:**
- `classify_pattern_selectivity` correctly identifies selectivity class of each pattern
- Bound-var propagation is correct (no false-positive "bound" claims)

---

### Step 2: CompiledRule Struct

**Deliverables:**
- Define `struct CompiledRule`:
  ```rust
  pub struct CompiledRule {
    pub original_rule: Rc<Rule>,  // original unparsed rule for error msgs
    pub head: Triple<SymbolId>,
    pub body: Vec<PatternStep>,
    pub driving_atom: usize,      // index of first body pattern to start with
  }
  ```
- Define `struct PatternStep`:
  ```rust
  pub struct PatternStep {
    pub pattern: Triple<SymbolId>,
    pub selectivity: Selectivity,
    pub new_vars: HashSet<Var>,   // variables bound by this pattern (for next step)
  }
  ```
- Add `order_body_patterns(rule: &Rule, existing_bindings: &HashSet<Var>) → Vec<PatternStep>` function that:
  1. Classify selectivity of each body pattern
  2. Sort by selectivity (best first)
  3. Recompute selectivity of each subsequent pattern given newly-bound vars
  4. Return ordered `Vec<PatternStep>`

**Tests:**
- Original rule semantics preserved: reordered rule produces same results as original (set-equality of derived facts)
- Selectivity ordering validated: each pattern in the ordered sequence is more selective than patterns evaluated later
- Determinism: same rule → same ordering every time (no randomness)

**Acceptance:**
- `CompiledRule` and `PatternStep` structs compile
- `order_body_patterns` produces correct reordering
- Semantic preservation: no spurious facts added, no correct facts lost

---

### Step 3: Integrate CompiledRule into Rule Evaluation Paths

**Deliverables:**
- Compile rules at load time: `ruleindex.rs` (RuleIndex) wraps each `Rule` in a `CompiledRule`
- Update `backwardchaining.rs:238` (main backward-chaining loop) to iterate body patterns in `CompiledRule` order instead of declaration order
- Update `backwardchaining.rs:486,519` (alternative substituted-body path) similarly
- Update `csprite.rs:231,292,426` (CSprite iterative variant) to use ordered body patterns
- Update `queryengine/mod.rs` (semi-naive query dispatch) to use ordered patterns

**Tests:**
- Existing backward-chaining tests pass (rules derive same facts)
- Existing CSprite tests pass
- Existing rule-indexing tests pass
- Benchmark: `datalog_stratify_layers_20/50/200` (baseline: 12-118 µs); latency unchanged or improved
- Benchmark: `n3_chain_depth_50/150/400` (baseline: 0.86-26.2 ms); latency unchanged or improved

**Acceptance:**
- All rule evaluation tests pass
- Rule evaluation results (derived facts) are identical to baseline
- No regressions in benchmarks; ideally improvements in join latency

---

## Benchmark Targets (Three-Tier Structure)

| Benchmark | Baseline | Target (P1) | Target (P1.5) |
|-----------|----------|------------|---------------|
| Stratify 20 rules | ~12 µs | ≤ 12 µs | ≤ 11 µs (reordering + caching) |
| Stratify 200 rules | ~118 µs | ≤ 118 µs | ≤ 100 µs |
| N3 chain depth 400 | ~26.2 ms | ≤ 26.2 ms | ≤ 24 ms (early binding) |
| Datalog aggregate 1000 facts | ~1.31 ms | ≤ 1.31 ms | ≤ 1.2 ms |

---

## Success Criteria (Final)

- [ ] `Selectivity` enum and `PatternStep` struct defined
- [ ] `order_body_patterns` function implemented and tested
- [ ] `CompiledRule` struct integrates into `RuleIndex`
- [ ] Backward-chaining loop iterates body in selectivity order
- [ ] CSprite loop uses ordered patterns
- [ ] All rule evaluation tests pass (set-equality verified)
- [ ] Benchmarks run; no regressions
- [ ] Determinism verified: same rule → same ordering every run

---

## Acceptance Criteria

- [ ] Code review: selectivity classification logic is sound (no missing edge cases)
- [ ] Integration: `PROJ-406` (Delta Materialization) can assume ordered body patterns
- [ ] Standing: `standing.json` reports PROJ-405 as COMPLETE after `just verify-all` passes

---

## Standing Rules

- **ALIVE**: PROJ-405 is ALIVE when `CompiledRule` is integrated into all rule-evaluation paths, selectivity ordering is correct, and benchmarks pass
- **PARTIAL_ALIVE**: If selectivity classification has edge cases (e.g., certain variable patterns), document them; update order_body_patterns to handle them
- **REFUSED**: If reordering produces incorrect results (spurious facts or missing facts), refuse and debug

---

## Related Tickets

- PROJ-401: Quick-Win Crate Optimizations (COMPLETE; provides ID-based pattern matching)
- PROJ-404: Compiled Condition IR (no direct dependency, but shares similar compilation philosophy)
- PROJ-406: Semi-Naive Delta Materialization (depends on this for correct rule compilation)

---

## References

- `crates/praxis-graphlaw/src/rule.rs:27-31`: `Rule` struct
- `crates/praxis-graphlaw/src/ruleindex.rs:9-22`: `RuleIndex` struct
- `crates/praxis-graphlaw/src/backwardchaining.rs:238-268`: Main backward-chaining loop
- `crates/praxis-graphlaw/src/backwardchaining.rs:486,519`: Alternative paths
- `crates/praxis-graphlaw/src/csprite.rs:231,292,426`: CSprite rule evaluation
- Article: "Graphlaw Performance Architecture" — Section "CompiledRule IR and Selectivity Heuristic"

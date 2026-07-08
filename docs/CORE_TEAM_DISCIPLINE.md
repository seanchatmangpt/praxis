# Core Team Discipline — praxis-graphlaw Rust Engineering Standards

This document establishes the engineering standards for praxis-graphlaw's Rust reasoning core, modeled after Rust language team discipline for std and rustc. Every rule is binding for code review, not advisory.

---

## 1. Invariants Are Contracts, Not Aspirations

The 8 invariants in `CLAUDE.md` are hard requirements — violations are bugs, not code-review discussion items.

| # | Invariant | Enforcement |
|---|-----------|------------|
| 1 | No panics/silent defaults | `cargo build` with `#![deny(unreachable_patterns)]` + clippy + code review |
| 2 | Receipts computed (BLAKE3), never asserted | All receipt generation paths audited; tests check byte-identical output across runs |
| 3 | No wall clock in hash/receipt | `grep -r "SystemTime\|now()\|Instant" crates/praxis-graphlaw/src/` blocked by pre-commit hook |
| 4 | Closed vocabularies | Vocabulary map in code or `docs/v26.7.*/PUBLIC_ONTOLOGY_MAPPING.md`; unknown predicates return `Refusal::UnknownVocabulary` |
| 5 | Deterministic under fixed seed | CI runs `praxis-graphlaw` tests 10× consecutively; receipts must be byte-identical all 10 runs |
| 6 | No algorithmic surprises | `docs/ALGORITHM_COMPLEXITY.md` documents every data structure; bounds must be O(1), O(n), O(n log n), O(n²) classified |
| 7 | Zero unsafe except crypto verification | `grep unsafe` shows only `blake3::Hasher` or `unsafe { std::mem::transmute }` in TryFrom impls; every line audited in PR |
| 8 | Error paths tested rigorously | Every `Refusal` variant exercised in `tests/`; coverage report checked per PROJ-401 standing rules |

---

## 2. Code Quality Bars (No Exceptions)

### 2.1 Performance Is Measured, Not Guessed

- Every function with a "hot" comment or called in a loop has a benchmark.
- Benchmarks are run before and after changes; regressions ≥ 5% require explanation and alternative.
- No micro-optimizations without measured impact (use `perf record`, not "I think this is faster").
- Baseline benchmarks committed to repo (`benches/` directory); pull requests include `cargo bench` output.

### 2.2 Tests Are Specifications, Not Validation

- Unit tests document the expected behavior for normal and edge cases.
- Property-based tests (via `proptest`) cover invariant-preservation:
  - Interning: intern(A) == intern(A) for same string, never panics
  - Closures: transitive_closure is idempotent: TC(TC(S)) == TC(S)
  - Receipts: BLAKE3(R1) == BLAKE3(R1) across runs, never panics
- Fuzzing targets: `libFuzzer` on Turtle parsing, Datalog rule parsing, hook condition evaluation
- Error-path tests: every `Refusal` variant has ≥ 1 dedicated test

### 2.3 Documentation Is Code-Adjacent, Not Separate

- Every public type has a doc comment explaining its invariant (not just what it holds, but what it guarantees)
- Every algorithm (rule materialization, shape validation, hook scheduling) has a doc comment citing:
  - Reference paper or RFC (if applicable)
  - Time/space complexity
  - Known limitations or unsupported features
- No vague comments like "handles edge cases"; list the edge cases explicitly

### 2.4 Refactoring Is Justified, Not Aesthetic

- No "let's make this more Rust-idiomatic" refactors without measuring impact
- Refactor only if:
  - It fixes a bug (invariant violation)
  - It improves measured performance (≥ 5%)
  - It reduces algorithmic complexity class
  - It enables a new feature required by standing tickets
- Every refactor includes before/after benchmark numbers in the commit message

### 2.5 Dependencies Are Pinned and Audited

- No `semver` ranges in production deps (e.g., `blake3 = "0.3"` not `blake3 = "0.3.*"`)
- Every new dependency is:
  - Audited for security (use `cargo-audit`)
  - Checked for maintenance status (active maintainer, recent updates)
  - Justified in a ticket (PROJ-401 documents all 8 crate additions)
  - Added only if it's provably faster or more correct than the alternative
- Vendored code is prohibited unless the crate is unmaintained; prefer fork + `[patch.crates-io]`

---

## 3. Review Discipline

### 3.1 Every PR Requires

- [ ] Before/after benchmark numbers (or explanation why none apply)
- [ ] Invariant check: which of the 8 does this touch? How is it verified?
- [ ] Standing impact: does this ticket close a standing gate? How?
- [ ] Error path coverage: does every new error type have a test?
- [ ] Complexity statement: "This change is O(n) addition to rule body evaluation; prior was O(n), no regression"

### 3.2 Code Review Checklist (No Shortcuts)

- [ ] No `.unwrap()` outside clearly-safe contexts (e.g., `Option::Some(x).unwrap()` after a prior `is_some()` guard)
- [ ] No `.to_string()` in hot paths (intern SymbolId instead)
- [ ] No ad-hoc string parsing (use existing parsers or refuse early with `Refusal`)
- [ ] All match arms handled (no wildcard `_` unless justified by a comment)
- [ ] No `clone()` in a loop (pre-allocate or use references)
- [ ] Every `unsafe` block has a comment explaining why it's safe
- [ ] No debug `println!` or `eprintln!` (use structured logging if needed)
- [ ] Determinism verified: all iteration over maps is explicitly sorted or uses `IndexMap`

### 3.3 Approval Authority

- At least one core-team author (Sean Chatman or designated reviewer) must approve.
- Approval means: "I have verified this against all 8 invariants and the code quality bars above."
- No approval via "looks good to me" or "LGTM"—approval requires a written explanation of what invariants/standards were verified.

---

## 4. Testing Discipline

### 4.1 Coverage Targets

- Line coverage ≥ 90% for critical paths (hooks, rules, shapes, receipts)
- Branch coverage ≥ 85% (all error paths and condition branches tested)
- Measured via `tarpaulin` or `llvm-cov` before marking ticket COMPLETE

### 4.2 Test Organization

- `tests/` — integration tests (end-to-end scenarios)
- `src/*/tests.rs` — unit tests (per-module, inline or co-located)
- `benches/` — benchmark tests (measured performance)
- `fuzz/` — fuzzing targets (if applicable)

### 4.3 Negative Test Discipline

- For every feature, write tests for what it **does not** accept
- Examples:
  - Hook scheduling: test that cyclic dependencies are rejected
  - Rule evaluation: test that unstratified negation is rejected
  - Shape validation: test that SHACL-SPARQL is handled per dialect boundary
- Coverage report must show these refusal paths are exercised

---

## 5. Performance Expectations

### 5.1 Benchmark Tiers

Every benchmark has three tiers:

| Tier | Meaning | Gate |
|------|---------|------|
| Baseline | Existing measured performance (from prior release) | No change ≥ 5% is a regression |
| Target (P0/P1) | Goal for this milestone | Ticket definition specifies target |
| P1.5 | Stretch goal after cost-ordering/optimization | Attempted only after P0/P1 stable |

### 5.2 Documented Algorithmic Complexity

Add to `docs/ALGORITHM_COMPLEXITY.md`:

```markdown
## Materialization Loop (reasoner/mod.rs)

- **Input**: Rules R, facts F, maximum stratum S
- **Output**: Fixed-point facts F'
- **Time**: O(S * |F| * |R|) where |F| grows per stratum
- **Space**: O(|F| + |R|) — facts and rules never shrink
- **Known issue**: No semi-naive optimization for rule bodies containing unions (N3 rules)
- **Gate**: Supports up to 10K facts, 100 rules, 10 strata in v26.7.8 (measured; see benches/hierarchies.rs)
```

---

## 6. Semantic Integrity (No Silent Failures)

### 6.1 Unsupported Feature Handling

Every unsupported feature must:
1. Be documented in the ticket (e.g., PROJ-404 lists unsupported OWL RL constructs)
2. Return `Refusal::Unsupported { reason, feature }` at load/parse time (not silently skipped)
3. Have a test that verifies it is rejected
4. Be listed in `docs/SEMANTIC_PROFILE_DOCTRINE.md` by dialect

### 6.2 Standing and Readiness Gates

- Use `just verify-all` before marking any ticket COMPLETE
- Standing ladder rungs (from `docs/standing/STANDING_SCHEMA.md`):
  - Rung 1: BUILT (code compiles)
  - Rung 2: TESTED (all tests pass)
  - Rung 3: RECEIPTED (deterministic output, byte-identical across runs)
  - Rung 4-5: OCEL_PROVEN (formal proof for critical paths, if applicable)
  - Rung 7+: PUBLISH_READY (approved for public release)
- Claim only the rung your ticket actually reaches; do not skip rungs

---

## 7. Documentation Standards

### 7.1 Inline Comments

- Explain the **why**, not the **what**
- Bad: `let x = y + 1;  // Add one to y`
- Good: `let x = y + 1;  // Next stratum number (strata are 0-indexed)`
- Only comment if the why is non-obvious from context

### 7.2 Doc Comments

```rust
/// Materialize facts under a set of rules until fixpoint.
///
/// # Invariants
/// - Input facts are immutable; new facts are added to `fact_store.delta()` only
/// - All derived facts are deterministic (same rules + facts → same derivation every run)
/// - Fixpoint is reached when `fact_store.take_delta().is_empty()`
///
/// # Complexity
/// O(S * |F| * |R|) worst case, where S is the maximum rule stratum
///
/// # Errors
/// Returns `Refusal::*` if any rule is malformed or unsupported
pub fn materialize(rules: &[Rule], fact_store: &mut FactStore) -> Result<(), Refusal> {
    // ...
}
```

### 7.3 Ticket References

- Commit messages cite the ticket: "PROJ-405: Add CompiledRule IR"
- Architectural decisions cite the article or RFC: "Per 'Graphlaw Performance Architecture' section X, we compile rules at load time"
- Refactoring PRs cite the prior code section and explain the change: "Refactored schedule_hooks (hooks.rs:533-586) to use HookId instead of String; topo sort remains Kahn's algorithm, but with O(1) tie-breaks instead of string comparisons"

---

## 8. Errors and Refusals

### 8.1 Refusal Taxonomy

Define all `Refusal` variants in `lib.rs`; every variant must:
- Have a descriptive name (not `Error`, `Failed`, or `Other`)
- Include a reason string for debugging
- Be explicitly matched somewhere in tests

Example:
```rust
pub enum Refusal {
    HookSchedulingCycle { hook_iris: Vec<String> },
    UnknownVocabulary { predicate: String, known_predicates: &'static str },
    UnsupportedFeature { feature: &'static str, reason: &'static str },
    ParseError { input: String, expected: &'static str },
}
```

### 8.2 Error Handling in Hot Paths

- Use `Result<T, Refusal>` (not `Option<T>` with a default)
- Propagate with `?` operator
- At API boundaries, convert to HTTP status or log message with full context

---

## 9. Continuous Integration Discipline

### 9.1 Pre-Commit Hooks (Enforced)

```bash
# examples (real hooks in .git/hooks/pre-commit)
grep -r "\.unwrap()" crates/praxis-graphlaw/src/ && exit 1
grep -r "println!\|eprintln!" crates/praxis-graphlaw/src/ && exit 1
grep -r "SystemTime\|Instant::now()" crates/praxis-graphlaw/src/ && exit 1
cargo test --release --lib && cargo bench --no-fail-fast
```

### 9.2 CI Pipeline

1. **Format**: `cargo fmt --check` (no formatting diff)
2. **Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **Test**: `cargo test --release` (all tests)
4. **Benchmark**: `cargo bench` (baseline unchanged ≥ 5% → fail)
5. **Coverage**: `tarpaulin --out Html --output-dir coverage` (≥ 90% line, ≥ 85% branch)
6. **Standing**: `just standing` (updates `target/praxis-standing/standing.json`)

### 9.3 Release Gate

Before publishing to crates.io or arXiv:
1. All tickets in milestone are TESTED (rung 2) or RECEIPTED (rung 3)
2. `just verify-all` passes
3. At least one core-team review per ticket
4. Changelog is written and committed
5. Git tag is created (`v26.7.8`, etc.) and pushed

---

## 10. Exceptions and Escalations

If a rule cannot be followed:
1. Document why in a GitHub issue (cite the rule number and reason)
2. Get explicit written approval from Sean Chatman or designated core-team member
3. Add a comment in the code linking to the issue and approval
4. Set a deadline for resolution (e.g., "revisit in v26.7.9")

Example:
```rust
// EXCEPTION: Invariant #7 (zero unsafe) temporarily violated to work around
// rustc codegen issue with large enums. See github.com/rust-lang/rust#12345
// Approved by Sean Chatman 2026-07-08; revisit in v26.7.9.
unsafe {
    // transmute to avoid enum bloat
}
```

---

## References

- `CLAUDE.md` — Project-wide rules (FIX FORWARD ONLY, no reset --hard, etc.)
- `docs/rust-anti-patterns.md` — Specific anti-patterns and their sources
- `docs/ALGORITHM_COMPLEXITY.md` — Complexity bounds per data structure and function
- `docs/SEMANTIC_PROFILE_DOCTRINE.md` — Dialect-by-dialect supported/unsupported features
- `docs/standing/CLAUDE_CODE_POLICY.md` — Standing index and verification gates
- `tests/` — Test organization and coverage targets
- `.git/hooks/pre-commit` — Enforced checks before commit

# Praxis — v26.7.8 (Rust Core Team Discipline for Antigravity Agents)

Milestone: v26.7.4 archived at [index.md (v26.7.4)](file:///Users/sac/praxis/docs/jira/archive/v26.7.4/tickets/index.md).
v26.7.8 (PROJ-401..410, 501-505) in progress at [index.md (v26.7.8)](file:///Users/sac/praxis/docs/jira/v26.7.8/tickets/index.md).

This project maintains **AGI-level Rust core-team code discipline** — every invariant, test, and performance assumption is binding, not aspirational. See [CORE_TEAM_DISCIPLINE.md](file:///Users/sac/praxis/docs/CORE_TEAM_DISCIPLINE.md) for full engineering standards.

---

## 1. Invariants (Violation = Bug, Not a Code-Review Note)

1. **No panics/silent defaults** — Every error is a typed `Refusal` variant (`lib.rs`). Banned: `.unwrap()`, `.expect()`, `panic!()` in fallible code. Banned: silent swallows like `.ok()`, `.unwrap_or_default()`.
2. **Receipts are computed (BLAKE3), never asserted** — All facts in canonical N-Quads order.
3. **No wall clock in hash/receipt paths** — Time only from graph OWL-Time literals. Banned: `SystemTime`, `Instant::now()`, `std::time::` in logic paths.
4. **Closed vocabularies** (`wf:`, `hook:`, `prayer-kernel:`, `agent:`) — Unknown predicates refused by name.
5. **Deterministic under fixed seed** — Same inputs → byte-identical receipts, no randomness.
6. **No algorithmic surprises** — All O(n) bounds documented; no hidden quadratic behavior.
7. **Zero unsafe code except cryptographic verification** — Every unsafe block audited and must have a `// SAFETY:` explanation comment.
8. **Error paths tested as rigorously as happy paths** — Refusal variants have end-to-end tests.

---

## 2. Standing Gates & Policy

Before claiming any artifact is real/tested/ready:
1. Read [standing.json](file:///Users/sac/praxis/target/praxis-standing/standing.json) + [REALITY_INDEX.md](file:///Users/sac/praxis/docs/standing/REALITY_INDEX.md).
2. If missing/stale, run `just standing` first.
3. Never trust prior-agent summaries, README claims, or code comments over the standing index.
4. Never say "production-ready" (or pilot/publish/publication-ready) unscoped — every readiness claim requires a stated scope.
5. Full policy: [CLAUDE_CODE_POLICY.md](file:///Users/sac/praxis/docs/standing/CLAUDE_CODE_POLICY.md).

---

## 3. Commands

- `just verify-all` — DoD gate, run before claiming a ticket done.
- `just test-changed` — Fast inner loop.

---

## 4. Antigravity Agent Guidelines & Rules

### A. No Overclaiming Vocabulary
Use precise vocabulary to describe the status of your work:
- **ALIVE**: Verified in the current session.
- **PARTIAL**: Name the gaps clearly.
- **BLOCKED**: Cite specific file and line (`file:line`).
- **MOCKED**: Explicitly state if a mock stands in for real logic.
- **REFUSED/UNSUPPORTED**: By design.
- **UNVERIFIED**: Default state (do not inflate).
- *Forbidden words without context/command*: "substantially complete", "should work", "production-ready" (unscoped).

### B. Rust Code Review & Verification Checklist
Before submitting/finalizing, ensure you verify:
- [ ] **No panics/unwrap**: Checked via grep; new code has zero `.unwrap()` outside assertions.
- [ ] **Determinism**: Ran `cargo test` 5×; outputs are byte-identical.
- [ ] **Safety**: No unsafe code outside crypto; every `unsafe` block has a `// SAFETY:` comment.
- [ ] **Performance**: Benchmarks show no regression (targets: check ticket details).
- [ ] **Test coverage**: Line coverage ≥ 90% and branch coverage ≥ 85% on critical paths (hooks, rules, shapes, receipts).
- [ ] **Errors**: Every new Refusal variant has ≥ 1 test; no `.ok()` swallowing errors.

---

## 5. Post-Tool Hooks & Formatting
* Rust files (`.rs`) format automatically via `cargo fmt --manifest-path Cargo.toml` on write/edit.

---

## 6. See Also

- [CLAUDE.md](file:///Users/sac/praxis/CLAUDE.md) — Original Claude Code reference
- [CORE_TEAM_DISCIPLINE.md](file:///Users/sac/praxis/docs/CORE_TEAM_DISCIPLINE.md) — Full engineering standards
- [rust-anti-patterns.md](file:///Users/sac/praxis/docs/rust-anti-patterns.md) — Project-scoped Rust anti-patterns
- [no-overclaiming.md](file:///Users/sac/praxis/.claude/rules/no-overclaiming.md) — Status vocabulary guidelines
- [praxis-rust-discipline.md](file:///Users/sac/praxis/.claude/rules/praxis-rust-discipline.md) — Rust discipline checkers
- [rust-agi-core-team.md](file:///Users/sac/praxis/.claude/rules/rust-agi-core-team.md) — Core team standards deep-dive
- [ALGORITHM_COMPLEXITY.md](file:///Users/sac/praxis/docs/ALGORITHM_COMPLEXITY.md) — Complexity bounds
- [SEMANTIC_PROFILE_DOCTRINE.md](file:///Users/sac/praxis/docs/standing/SEMANTIC_PROFILE_DOCTRINE.md) — Dialects profile strategy

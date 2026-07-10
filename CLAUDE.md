# Praxis — v26.7.8 (Rust Core Team Discipline)

Milestone: v26.7.4 (PROJ-302..306) archived at `docs/jira/archive/v26.7.4/tickets/index.md`.
v26.7.8 (PROJ-401..410, 501-505) in progress at `docs/jira/v26.7.8/tickets/index.md`.

This repo maintains **AGI-level Rust core-team code discipline** — every invariant,
test, and performance assumption is binding, not aspirational. See
`docs/CORE_TEAM_DISCIPLINE.md` for full engineering standards.

Chatman Engine is the concrete realization of μ (`A = μ(O*)`) — not workflow
automation, document generation, or an ontology project; see
`docs/CHATMAN_EQUATION.md` for the full formulation.

## Invariants (violation = the bug, not a code-review note)

1. **No panics/silent defaults** — every error is a typed `Refusal` variant (`lib.rs`).
2. **Receipts are computed (BLAKE3), never asserted** — all facts in canonical N-Quads order.
3. **No wall clock in hash/receipt paths** — time only from graph OWL-Time literals.
4. **Closed vocabularies** (`wf:`, `hook:`, `prayer-kernel:`, `agent:`) — unknown predicates refused by name.
5. **Deterministic under fixed seed** — same inputs → byte-identical receipts, no randomness.
6. **No algorithmic surprises** — all O(n) bounds documented; no hidden quadratic lurking.
7. **Zero unsafe code except cryptographic verification** — every unsafe block audited.
8. **Error paths tested as rigorously as happy paths** — Refusal variants have end-to-end tests.

## Standing

Before claiming any artifact is real/tested/ready, read
`target/praxis-standing/standing.json` + `docs/standing/REALITY_INDEX.md`;
if missing/stale, run `just standing` first. Never trust prior-agent
summaries, README claims, or code comments over the standing index. Never
say "production-ready" (or pilot/publish/publication-ready) unscoped —
every readiness claim requires a stated scope. External actions (crates.io
publish, arXiv submission, repo-visibility changes) are side effects, not
blockers. Full policy: `docs/standing/CLAUDE_CODE_POLICY.md`.

## Commands

`just verify-all` — DoD gate, run before claiming a ticket done.
`just test-changed` — fast inner loop.

## Custom agents — use these, don't reinvent their briefing

Three project agents in `.claude/agents/` carry this repo's recurring constraints so they don't
need restating per-prompt: `chatman-rust` (any `crates/praxis-graphlaw/src/chatman/` change —
sealed-receipt invariants, just-only build discipline), `ttl-ontology` (RDF/Turtle vocabulary or
instance-data work — public-ontology-first doctrine, STRIPS8 boundary), `release-doc`
(`docs/releases/vX.Y.Z/` PRD/ARD/RELEASE_CONTROL work — house template, shared Claims
Reconciliation table). Prefer dispatching to the matching agent over a bare `general-purpose`
one for these three areas.

## Build hygiene

Never invoke `cargo` directly — use `just <recipe>`. If a needed cargo invocation has no
matching recipe, add one to `justfile` rather than running cargo ad hoc; do not work around
this by shelling out. Never run two `just` invocations that both build/test/check concurrently
(they serialize on the `target/` lock and silently double wall-clock time — check
`ps aux | grep cargo` first). Avoid `--release` (full LTO, codegen-units=1) for iteration; scope
by exact `--test <binary>`, not substring match. Details: `docs/BUILD_CACHING.md`.

## Reporting

State findings, not verdicts. No unearned hedges, no ranking docs by assumed
authority, no quality adjectives ("messy," "stale") the task didn't ask for.
Explore mines requirements; exploit rewrites clean-room from the invariant, never
ports the explored source.

## Size

This file must stay ≤ 200 lines. If it needs to grow past that, split into a new
`docs/` doc and link it rather than expanding CLAUDE.md itself.

## See Also

- **`docs/CORE_TEAM_DISCIPLINE.md`** — Full engineering standards for Rust core-team level code quality (performance measurement, testing discipline, review checklist, determinism guarantees, complexity documentation, invariant enforcement)
- `docs/CHATMAN_EQUATION.md` — Chatman Engine as the concrete realization of μ in `A = μ(O*)`
- **`~/.claude/rules/tools.md`** — Tool usage rules (LSP-first navigation, markdown document standards, cross-document consistency)
- `docs/rust-anti-patterns.md` — Project-scoped Rust anti-patterns and enforcement rules
- `.claude/rules/no-overclaiming.md` — required status vocabulary; forbidden completion
  phrases; applies repo-wide (Rust + JS/TS)
- `docs/ALGORITHM_COMPLEXITY.md` — Complexity bounds per function and data structure (to be created during v26.7.8)
- `docs/standing/SEMANTIC_PROFILE_DOCTRINE.md` — 80/20 profile strategy for semantic dialects (OWL RL, SHACL, ShEx, N3, Datalog)
- `docs/standing/CLAUDE_CODE_POLICY.md` — Standing index verification gates and readiness claims
- `docs/BUILD_CACHING.md` — cargo lock contention, LTO/release-profile cost, sccache setup

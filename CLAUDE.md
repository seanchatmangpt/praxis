# Praxis — v26.7.4

Milestone: v26.7.4 (PROJ-302..306) archived at `docs/jira/archive/v26.7.4/tickets/index.md`.
v26.7.8 scaffolded at `docs/jira/v26.7.8/tickets/index.md` — awaiting ticket specification.

## Invariants (violation = the bug)

1. No panics/silent defaults — every error is a typed `Refusal` variant (`lib.rs`).
2. Receipts are computed (BLAKE3, genesis-folded), never asserted-in.
3. No wall clock in any hash/receipt path — time only from graph OWL-Time literals.
4. Closed vocabularies (`wf:`, `hook:`, `prayer-kernel:`, `agent:`) — unknown
   predicates refused by name, paired with `docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md`.
5. `praxis-synthesis` deps frozen to exactly: pddl-index, chatman-common, blake3,
   serde, serde_json, thiserror (`tests/no_llm_runtime.rs` enforces).
6. Smallest diff, reuse first — no new subsystems where a const table suffices.

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

## Reporting

State findings, not verdicts. No unearned hedges, no ranking docs by assumed
authority, no quality adjectives ("messy," "stale") the task didn't ask for.
Explore mines requirements; exploit rewrites clean-room from the invariant, never
ports the explored source.

## Size

This file must stay ≤ 200 lines. If it needs to grow past that, split into a new
`docs/` doc and link it rather than expanding CLAUDE.md itself.

## See Also

- `docs/rust-anti-patterns.md` — Project-scoped Rust anti-patterns and enforcement rules.

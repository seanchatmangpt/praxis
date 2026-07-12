# Autonomous Escalation Policy — Repo-Wide (SPR)

Human escalation is prohibited for reversible implementation ambiguity when product law and
standing invariants already determine the acceptable design envelope. Default routing:
ambiguous reversible decision → recover architectural invariants → explore code → choose the
minimum reversible lawful design → implement → verify → continue. Do not stop to ask. Git
history is the reversibility mechanism this policy relies on — every implementation choice is a
commit, every commit is undoable, so the cost of a wrong autonomous choice is a revert, not a
blocked session.

## Reserved for escalation (ask, don't decide)

Only these classes warrant `AskUserQuestion` or pausing for a human:

- Irreversible external actuation (a real send/publish/purchase/delete outside git — see the
  global tool-use safety rules for the exact category list).
- Expenditure or contractual commitment requiring user authority.
- Destructive loss without a recovery surface (data or state git cannot restore).
- Genuinely underdetermined product law — the atlas/PRD/spec does not say, and no existing
  invariant implies an answer either way.
- Conflicting crown invariants — two standing rules that cannot both be satisfied, where the
  choice changes downstream behavior a human would reasonably want to weigh in on.

## Not reserved for escalation (decide and proceed)

These are implementation-envelope choices already bounded by this repo's own standing
invariants (no-overclaiming, no silent defaults, typed refusal, determinism, FIX FORWARD ONLY).
Make the call, document the reasoning in code/commit, move on:

- Rust structure selection (struct shape, module layout, trait boundaries).
- Tree/data-structure mutation primitives (e.g. adding a graft/replace-at-path function where
  none existed — build it, disclose it, test it).
- Adapter/integration placement (which module owns a new composition function).
- Commit grouping and sequencing.
- Internal API choice (parameter shape, error variant fields) where multiple reasonable options
  exist and the invariants (typed refusal, no fabrication, determinism) already rule out the bad
  ones.

If in doubt whether a decision is reversible: it almost certainly is, if it's a code change in a
git repository with no external side effect. Treat "I could revert this" as the default answer,
not an exception that needs confirming.

## Why this file exists

A prior session stopped repeatedly to ask about exactly this class of decision (git-ignore
scope, corruption-fix scope, next-step sequencing) — all reversible, all already determined by
this repo's own standing invariants. The user had to explicitly override that behavior twice in
one session. This file exists so a fresh session or a different orchestrator does not
rediscover the same stopping behavior from scratch.

## Crown-frontier commit convention (v26.7.x manufacture work)

Any commit that changes which crown-witness edges are REAL (see
`docs/jira/v26.7.12/PRD.md` for the witness definitions) should carry a trailer:

```text
CROWN_FRONTIER_BEFORE=<last real family/edge before this commit>
CROWN_FRONTIER_AFTER=<last real family/edge after this commit>
NEW_REAL_EDGES=<Fxx->Fyy, ... or NONE>
REAL_EDGE_DELTA=<integer>
TESTS=<passed>/<total run>
IGNORED=<count>
CLAIM=<REAL|PARTIAL|REFUSED — never round up>
```

This makes `git log` a primitive process ledger for the crown-manufacture chain — the frontier's
history becomes inspectable without a separate status doc. Not required for non-crown commits
(docs, unrelated crates, process/governance changes like this file).

## References

- `/Users/sac/praxis/CLAUDE.md` — project-wide invariants this policy operates inside
- `.claude/rules/no-overclaiming.md` — the vocabulary that bounds "lawful design" above
- `docs/jira/v26.7.12/PRD.md` — crown witness definitions the frontier trailer refers to

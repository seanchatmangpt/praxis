# Claude Code Standing-Consumption Policy

This is the policy Claude Code (or any agent) follows in this repo before
claiming any artifact is real, tested, or ready.

## Before claiming anything

1. Read `target/praxis-standing/standing.json` (the compiled
   `praxis-standing.v1` index — schema in `STANDING_SCHEMA.md`) and
   `docs/standing/REALITY_INDEX.md` (the ggen-rendered human summary of the
   same index).
2. If either is missing, or `standing.json`'s `generated_at_utc` looks stale
   (see `ANTI-LLM-STANDING-006` in `CLAIM_RULES.md`, default threshold
   86,400s), **run `just standing` first**. Do not reason from memory of a
   prior run.
3. Only then state what an artifact's standing actually is — quote the
   `standing` list and `ladder_level` for the specific artifact `id`, not a
   release-wide impression.

## Rules

- **Never trust prior-agent summaries, README claims, or code comments over
  the standing index.** A comment saying "fully tested" or a prior
  transcript saying "this is production-ready" is not evidence; the index
  is. If they disagree, the index wins and the doc/comment is out of date.
- **Never say "production-ready" (or pilot-ready/publish-ready/publication-ready)
  unscoped.** Every one of those four claims requires a stated scope — see
  the scoped-readiness rule in `STANDING_SCHEMA.md` and
  `ANTI-LLM-STANDING-001` in `CLAIM_RULES.md`. "Production-ready" with no
  named scope is not a claim, it is a category error.
- **External actions are side effects, not blockers.** `cargo publish`,
  arXiv submission, repository-visibility changes — see
  `EXTERNAL_OPERATOR_SIDE_EFFECTS.md`. An artifact can be fully
  `PUBLISH_READY` (rung 7) while the real publish is still pending operator
  action; do not describe that as "blocked" or "not ready."
- **If evidence is absent, run the gate — don't assert.** If a claim would
  need `TESTED`/`RECEIPTED`/`OCEL_PROVEN`/etc. and the index does not show
  it for that artifact, the correct action is to run the underlying
  command/test/OCEL pass and refresh the index (`just standing`), not to
  write the claim and hope the index catches up later.
- **State findings, not verdicts** (root `CLAUDE.md`'s Reporting section
  applies here too): report exactly what `standing.json` says for the
  artifact in question — statuses, ladder level, scope, evidence pointers —
  rather than a summarizing adjective.

## Quick reference

| Question | Where to look |
|---|---|
| Is artifact X built/tested? | `standing.json` → artifact `X`'s `standing` list; ladder rungs 1–2 |
| Is X receipted and receipt-verified? | rung 3 (`RECEIPTED`); `RECEIPT_VERIFIED` is off-ladder but implies it |
| Is X OCEL/wasm4pm proven? | rungs 4–5 |
| Is X ready to publish/pilot/go to production? | rungs 7–9, **and** check `scope` is non-empty |
| What's left before the next rung? | `cargo-cicd claude_context show` — prints exactly this, per artifact |
| Is the whole picture summarized anywhere? | `docs/standing/REALITY_INDEX.md` (generated; do not hand-edit) |

## If `just standing` itself fails

Report the exact failure (command, exit code, stderr) rather than falling
back to an old index or a guess. See
`docs/standing/ANTI_LLM_CHEAT_LSP_POLICY.md` for how the LSP treats a
missing/stale index (`ANTI-LLM-STANDING-000`/`006`) — the same discipline
applies to any agent reasoning about readiness by hand.

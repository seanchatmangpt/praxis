# External Operator Side Effects

`EXTERNAL_OPERATOR_SIDE_EFFECT` is a standing status (and
`external_operator_side_effects` an artifact field) for actions that require
a human operator holding external credentials or making an external, often
irreversible, decision. These are never blockers on an artifact's own
standing — see `docs/standing/STANDING_SCHEMA.md`'s relationship to
`NO_TERMINAL_BLOCKERS.md`'s `RESOLVED_BY_EXTERNAL_OPERATOR_SIDE_EFFECT`
route — they are typed, packaged, and dry-run-verified locally, then handed
off. `docs/releases/v26.7.6/FINAL_STATUS.md` is the source of truth for
current status of every side effect below; this document is the reusable
checklist template, not a duplicate status tracker.

## 1. crates.io publish (`praxis-graphlaw`)

**Current status** (per `FINAL_STATUS.md`): local packaging fresh-verified
this pass — `cargo publish --dry-run --allow-dirty -p praxis-graphlaw` → exit
0, 2026-07-06T19:44:59Z
(`docs/releases/v26.7.6/ocel/raw/cargo-publish-dry-run.txt`). Real publish
not yet performed — **ALIVE_EXCEPT_EXTERNAL_PUBLISH** for that lane.

**Operator checklist**:

- [ ] `cargo login` (requires an crates.io API token — operator credential,
      never entered by an agent)
- [ ] Decide whether to bump `crates/praxis-graphlaw/Cargo.toml` version
      (e.g. `26.7.5` → `26.7.6`) — a one-line change, operator's call
- [ ] `cargo publish -p praxis-graphlaw`
- [ ] After publish, update the artifact's `external_operator_side_effects`
      entry in the next `standing refresh` pass to record completion (do not
      hand-edit `standing.json` — it is compiled, not asserted)

## 2. arXiv submission

**Current status** (per `FINAL_STATUS.md`): **ALIVE_EXCEPT_EXTERNAL_SUBMISSION**
for that lane. Artifact bundle built:
`docs/releases/v26.7.6/arxiv-package/arxiv-submission.tar.gz`
(see `docs/releases/v26.7.6/arxiv-package/MANIFEST.md`).

**Operator checklist**:

- [ ] Make the artifact bundle / repository public
      (`ARXIV_READINESS.md` Sec. 11 blocker 2 — a visibility change, itself
      covered under item 3 below if it means flipping repo visibility)
- [ ] Upload `docs/releases/v26.7.6/arxiv-package/arxiv-submission.tar.gz`
      at https://arxiv.org/submit
- [ ] Category: cs.SE primary, cs.LO cross-list
- [ ] After submission, record the arXiv identifier as an `artifact`
      evidence entry on the paper's standing artifact at the next refresh

## 3. Repository visibility change

Changing a repository from private to public (a prerequisite for both the
arXiv bundle and any public crates.io source link) is an access-control
change and falls under the same operator-only category as the other two —
it is explicitly listed as a prohibited-for-agents action ("modifying access
controls or sharing permissions on any resource") independent of the
standing schema.

**Operator checklist**:

- [ ] Confirm which repository/bundle needs public visibility (the arXiv
      artifact bundle, not necessarily the full private working repo)
- [ ] Review the bundle contents for anything that should stay private
      before flipping visibility (credentials, unrelated WIP, personal data)
- [ ] Perform the visibility change directly in the hosting provider's UI —
      no agent action substitutes for this
- [ ] Record the change as a dated line in `FINAL_STATUS.md`'s evidence
      table, not as a new standing status (visibility is not itself part of
      `praxis-standing.v1`)

## Why these are typed, not blockers

Praxis invariant 1 (no panics/silent defaults — every error is typed) is
mirrored here: rather than a claim silently failing or a doc describing
progress as "blocked," each external action gets its own typed status
(`EXTERNAL_OPERATOR_SIDE_EFFECT`) and a checklist. `just standing` and the
gates it feeds never wait on operator action to report a clean local state —
they report `PUBLISH_READY`/`PUBLICATION_READY` (rungs 7, off-ladder) as
already earned, and list the remaining external action as a side effect
field, not a red gate.

# RELEASE_CONTROL — v26.7.14

Status: DRAFT. Single control surface for `PRD.md` and `ARD.md` in this directory. Both
documents' Status lines tie to this file. If this file and either document disagree, this
file wins.

## 1. Evidentiary floor

There is no single audit gate spanning v26.7.14, matching v26.7.13's own per-theme (rather than
milestone-wide) evidentiary floor. The evidentiary floor is per-claim: each of the five claims in
the Claims Reconciliation table is independently gated by its own cited commit(s), test evidence,
or task-tracker record, and each claim's status (ALIVE / ALIVE-with-disclosed-limitations /
PARTIAL_ALIVE) is the literal disposition in that table, not a paraphrase. `PRD.md` Sec. 14
states the aggregate verdict verbatim: "five claims independently ALIVE,
ALIVE-with-disclosed-limitations, or PARTIAL_ALIVE as scoped above, with all forward-looking and
unstarted work (task #139, TOGAF increments 2/3, bribery-case Stages 3–4) held to PLANNED or
explicitly zero-evidence — no row in this document rounds up." This control file adopts that
verdict as-is; it does not strengthen it.

Claim 5 (crown-external bribery-case chain) carries the additional caveat that its real,
run-verified Stage 1–2 evidence is entirely uncommitted (`git status` shows `??` for every
bribery-case path) — PARTIAL_ALIVE status reflects both the DoD gap (Stages 3–4 absent) and the
uncommitted state, neither of which is treated as resolved by the other.

## 2. Named exclusions (verbatim, reused identically in PRD.md and ARD.md)

`ARD.md` Sec. 16 defers its own "Hard exclusions" list to this exact enumeration in `PRD.md`
Sec. 12 ("Full itemization of these lives in `PRD.md` Sec. 12, not restated here to avoid
drift"). This control file reproduces that same list verbatim so all three documents agree
word-for-word:

1. **M&A / deal-desk case study — the PDDL8/POWL v2/Arazzo/Erlang chain (M&A-C4–C6)** — task
   #139, in_progress, not the zero-files state this line previously described. Real work has
   since landed (commit `66be8e6b`): FIBO/GLEIF/OMG-Commons vocabulary-fit research,
   `packs/ma-case-study-pack/` (ontology, SHACL shapes, a tested Knowledge Hook, 4/4 passing
   tests) — see open item 5 and `packs/ma-case-study-pack/STANDING.md` (M&A-C1–C6) for the exact
   boundary. This exclusion narrows to what remains genuinely absent: no PDDL8 deal-progression
   model, no POWL v2/Arazzo/Erlang dispatch chain, no multi-party Little's Law queue observation
   — `THESIS.md` §33.12's ruling that the M&A case is PLANNED future work for v26.7.14 standing
   still governs the case as a whole, even with M&A-C1–C3 real.
2. **TOGAF ADM increments 2 and 3** — tickets #100 (`ea-adm` bench category + roles +
   `meridian-adm` bundle) and #101 (F09 recursion + crown witness + v26.7.13 docs). Zero
   commits against either ticket this release; both remain PLANNED, unchanged from v26.7.13's
   own identical exclusion.

**Disclosed correction to a proposed third exclusion, checked and dropped:** an earlier framing
of this release's scope proposed "the Increment 2 permission-seam CLI verbs (`plan present`,
`plan check`, `plan step --approved`), named in this repo's own earlier v26.7.13 grounding work
and never built" as a third named exclusion. A grep across `crates/cng/src/` this session found
this to be factually incorrect: `plan present`, `plan check`, and `plan step` are real, wired
`#[verb(...)]` CLI entries (`crates/cng/src/main.rs:477,498,515`), backed by
`crates/cng/src/plan_approval.rs` (with a companion `plan_approval_test.rs`), landed in commit
`f676f08e` ("feat(cng): add plan present/check/step approval-seam CLI verbs (Operation Dogfood
Increment 2)") — already part of v26.7.13's own commit history, predating this milestone
entirely. This item is dropped from the exclusions list above rather than asserted as true. Per
this repo's no-overclaiming discipline, a false exclusion is exactly as much of an overclaim as a
false completion claim — silently dropping it without a note would itself be an undisclosed
correction, so it is recorded here instead.

Two items are explicitly not gated by this release's evidentiary floor, carried forward from the
crown-external bribery-case chain's own DoD (Claim 5, not a separate exclusion): Stage 3 (real
Erlang/BEAM dispatch) and Stage 4 (OCEL/receipt/replay + 7 adversarial refusal tests + runbook)
have zero evidence anywhere in the repo — see `PRD.md`/`ARD.md`'s verbatim CheckCrown verdict.

## 3. Standing-index disclosure

Per `docs/standing/CLAUDE_CODE_POLICY.md` ("if they disagree, the index wins and the
doc/comment is out of date"), `target/praxis-standing/standing.json` and
`docs/standing/REALITY_INDEX.md` are authoritative over any standing claim in `PRD.md` or
`ARD.md` if the two diverge. `ARD.md` Sec. 5 discloses that this milestone's docs were authored
from targeted greps/`wc -l`/`git log` checks against the live tree, not from a freshly re-run
`just standing` in the authoring session — neither `PRD.md` nor `ARD.md` claims a ladder level
for any v26.7.14 claim, and this control file does not add one.

Standing-policy vocabulary for this release: the ladder rungs are DISCOVERED → BUILDS → TESTED
→ RECEIPTED → … (per-artifact, quoted from the compiled index, not paraphrased). Per
`ANTI-LLM-STANDING-001`, "production-ready" (or pilot/publish/publication-ready) is never used
unscoped anywhere in this release's docs — every readiness claim requires a stated scope. If a
fresh `just standing` run this release cycle produces a ladder reading that conflicts with any
claim's status in the Claims Reconciliation table, the compiled index wins and the table is out
of date until corrected in the same commit as the standing refresh.

## 4. Claims Reconciliation governance

The `## Claims Reconciliation` table is authored once, in `PRD.md` — that file is authoritative
for claim status, scope, and evidence. `ARD.md` reproduces the identical table verbatim (not by
reference) per this milestone's explicit mirroring requirement (`ARD.md`'s own "Claims
Reconciliation" section: "the two files must never drift on status, scope, or evidence for the
same claim number"). The Claim 5 verbatim CheckCrown verdict is likewise duplicated verbatim in
both documents, not summarized by reference, for the same reason. Any status change to a claim
requires updating both `PRD.md` and `ARD.md` in the same commit; a change landed in only one file
is a defect in that commit, not a valid interim state. Task/ticket numbers cited in the table
must resolve against this milestone's actual tracker records (#100, #101, #132–#134, #137, #138,
#139) — not fabricated or renumbered; two claims (self-monitoring pack, bootstrap catalog) are
correctly marked UNTRACKED rather than assigned a fabricated ticket number.

## 5. Open items tracked against ticket status

Eight-item disclosure register, reconciled against this cycle's re-verification work. Every
RESOLVED row cites its exact commit; every OPEN/PARTIAL row states exactly what remains undone.
No row here rounds a status up beyond what its cited evidence supports.

| # | Item | Status | Severity | Ticket |
|---|---|---|---|---|
| 1 | Crown-external bribery-case chain (Stages 1–2 run-verified through F14, uncommitted) | PARTIAL_ALIVE — see verbatim CheckCrown verdict in `PRD.md`/`ARD.md` | HIGH | #137 |
| 2 | Crown-external bribery-case chain Stages 3–4 (Erlang/BEAM dispatch, OCEL/receipt/replay, refusal tests, runbook) | OPEN — zero evidence anywhere in the repo | HIGH | #137 |
| 3 | Self-monitoring pack: blank-node aliasing bug in `hooks/construct.rs::instantiate_term_pattern` | OPEN — disclosed, demonstrated on the broadened-topic counterfactual, not fixed; lives in shared engine code, not the pack | MEDIUM | N/A (no ticket covers this pack) |
| 4 | Self-monitoring pack: zero-firing false negative on real transcript (topic-tag + turn-kind classification granularity) | OPEN by design — an NLP-grade classifier is explicitly out of scope for this pack; both root causes are named with exact turn numbers, not hidden | MEDIUM | N/A (no ticket covers this pack) |
| 5 | M&A/deal-desk case study + FIBO/LEI ontology extension | OPEN/PARTIAL — vocabulary-fit research (FIBO/GLEIF-LEI/OMG-Commons, 8/10 concepts), `packs/ma-case-study-pack/{ontology.ttl,shapes.ttl,COMPETENCY_QUESTIONS.md}`, a validating case fixture + 1 tested Knowledge Hook (`fixtures/*.ttl`, `crates/praxis-graphlaw/tests/ma_case_hook_actuation.rs`, 4/4 passing) now exist (claims M&A-C1–C3, `STANDING.md`); PDDL8/POWL v2/Arazzo/Erlang dispatch chain (M&A-C4–C6) remains zero files — task #139 still open, not resolved by this progress | HIGH | #139 |
| 6 | TOGAF ADM increments 2 and 3 | OPEN — zero commits against either ticket | HIGH | #100, #101 |
| 7 | Proposed "Increment 2 permission-seam CLI verbs never built" exclusion | CORRECTED/DROPPED — checked against the live tree this session and found the verbs already exist (`crates/cng/src/main.rs:477,498,515`, commit `f676f08e`, v26.7.13); disclosed in Sec. 2 rather than silently applied | INFO | N/A |
| 8 | `docs/releases/v26.7.14/PRESS_RELEASE.md` — working-backwards narrative announcing v26.7.14 as a completed release | DISCLOSED — fenced by its own "Working-Backwards Status Fence" (bottom of file): actual release standing is controlled by this control file's claims ledger and Definition of Done, not the press release; the press release's own fence already discloses the crown-external chain (item 1–2 above) and task #139 (item 5 above) as not-yet-real, which this control file confirms rather than contradicts | INFO | N/A (not a numbered ticket; no claim in the press release is load-bearing for release status) |

`PRD.md`'s verbatim CheckCrown verdict is the authoritative status source for items 1–2 above —
this register defers to it rather than restating its evidence, the same deferral pattern
v26.7.13's `RELEASE_CONTROL.md` used for its own crown-witness repair item against
`CROWN_STATUS.md`. Item 8 defers to `PRESS_RELEASE.md`'s own status fence rather than restating
it; this control file does not merge that narrative into its own claims and remains
authoritative only over the five-claim table above should any apparent conflict arise.

## 6. Documents governed by this control surface

- `docs/releases/v26.7.14/PRD.md`
- `docs/releases/v26.7.14/ARD.md`
- `docs/releases/v26.7.14/RELEASE_CONTROL.md` (this file)
- `docs/releases/v26.7.14/PRESS_RELEASE.md` — additional governed document, existing prior to
  this control file, in the same disclosed/fenced category as v26.7.13's own `PRESS_RELEASE.md`
  row: working-backwards narrative announcing v26.7.14 as a completed release, fenced by its own
  "Working-Backwards Status Fence" — see open item 8. The real Claims Reconciliation table above
  (`PRD.md`/`ARD.md`), not the press release, governs standing for v26.7.14; this control file's
  claims ledger and Definition of Done remain authoritative over any narrative claim in it.
- `docs/standing/BOOTSTRAP_COLD_START_LIMITATIONS.md` — additional governed document, published
  this cycle; 20-item bootstrap/cold-start structural limitations catalog, cross-checked against
  `THESIS.md` and live repo greps (Claims row 4); authoritative for its own 20 items, which this
  control file does not restate.
- `packs/self-monitoring-pack/` — additional governed artifact, added this cycle; turn-lifecycle
  vocabulary + real `kh:Hook` SPARQL-CONSTRUCT actuation + real-session verification harness
  (Claims row 3); its own README.md is authoritative for the CLASSIFICATION-IS-INPUT FENCE, the
  blank-node aliasing bug, and the zero-firing false-negative investigation, which this control
  file's open items 3–4 defer to rather than restate.
- `docs/releases/v26.7.14/THESIS.md` — additional governed document, adopted verbatim this
  cycle; the v26.7.14 formal dissertation (Chapter 30 deterministic-envelope calculus, Chapter
  31 bootstrap/cold-start formalization, Chapter 32 reachability/correspondence boundary,
  Chapter 33 public-ontology scope and semantic fit including §33.12's explicit ruling that the
  M&A case is PLANNED future work, Chapter 34 the Fortune-5 external-crown compliance case,
  Chapter 35 current standing, Appendix N source/claim reconciliation). Its own Chapter 35 and
  Appendix N.6 ("Claims the candidate must refuse") are, per `N.1`'s own source-hierarchy
  ordering, subordinate to this control file's Claims Reconciliation table and ticket records —
  the thesis states this explicitly ("No lower item silently overrides a higher standing
  authority") — but nothing in this control file may promote a claim past what the thesis's own
  stricter reconciliation (e.g. `F5-C1`–`F5-C15`, all PLANNED/UNVERIFIED) supports, either.

This file wins on conflict with either `PRD.md` or `ARD.md`. Where `docs/releases/v26.7.14/
THESIS.md` states a stricter (more conservative) standing than this file for the same claim —
notably Chapter 35's treatment of the crown-external chain as PLANNED rather than this file's
PARTIAL_ALIVE — the more conservative reading governs until reconciled in the same commit.

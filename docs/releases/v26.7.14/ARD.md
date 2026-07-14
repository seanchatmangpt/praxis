# ARD — Architecture Reference Document, v26.7.14

Status: DRAFT, tied to `RELEASE_CONTROL.md` (single control surface). Every claim in this
document cites a file, test, commit, or task-tracker record in this repository. Rows without
evidence are marked PLANNED or UNKNOWN, never asserted. Companion to `docs/releases/v26.7.14/
PRD.md`, sharing its Claims Reconciliation table verbatim per this milestone's house template —
the two documents must not diverge on status for the same claim number. This ARD carries the
architectural detail behind the five claims (Sec. 2 below expands rows referenced from the
table); it does not itself introduce a new forward-architecture design the way v26.7.13's ARD
did — no v26.7.14 claim proposes new PLANNED struct shapes, and Sec. 4's object model below
documents the RDF/Rust shapes that already exist and were exercised this milestone.

## Claims Reconciliation

Identical table to `PRD.md`, reproduced verbatim below (not summarized by reference) per this
milestone's explicit mirroring requirement — the two files must never drift on status, scope, or
evidence for the same claim number. Status vocabulary: **ALIVE** (verified, executes, cited
test/receipt passes), **ALIVE-with-disclosed-limitations** (real and runs, but the Scope/caveat
column names confirmed limitations that must accompany any restatement of this claim),
**PARTIAL** (real but narrower than the claim — gap named explicitly), **PARTIAL_ALIVE** (real,
run-verified evidence exists but is uncommitted and/or the chain does not yet reach its stated
Definition-of-Done bar — this repo's `OPERATION_DOGFOOD_PRD.md` convention), **PLANNED**
(roadmap/ticket only, no code path), **UNKNOWN** (not yet investigated to a verdict), **MOCKED**
(a stand-in exists where the claim implies the real thing).

| # | Claim | Status | Scope / caveat | Evidence | Ticket |
|---|---|---|---|---|---|
| 1 | Formal MFW PhD thesis adoption | ALIVE | Adopted verbatim from an external source this cycle. Per v26.7.13's own `RELEASE_CONTROL.md` open item 15, the thesis is DISCLOSED against its own twelve-class epistemic-type register, and companion `THESIS_GROUNDING.md` records adoption-time re-checks and deltas (Sec. 26.4 crown verdicts confirmed but edge inventory refined; Sec. 26.6 blocker list accurate but omits B4/B7; Sec. 26.3 mfact numbers not re-verified). This document does not re-litigate that grounding; it cites it as-is. | commit `ee042f54` ("docs(v26.7.13): adopt formal MFW PhD thesis + grounding companion"); `docs/releases/v26.7.13/THESIS.md`; `docs/releases/v26.7.13/THESIS_GROUNDING.md` | #138 (completed) |
| 2 | Operation Dogfood Increment 1 (`dogfood-lifecycle-pack`) | ALIVE | Carried forward, not new this milestone — confirmed still present and unchanged in shape this session: `ontology.ttl` (160 lines) + `shapes.ttl` (104 lines) + 4 hook scripts (`dogfood-lifecycle-capture.sh` 83 lines, `dogfood-lifecycle-session-end.sh` 127 lines, `dogfood-lifecycle-receipt-spotcheck.sh` 92 lines, `cng-plan-admission-guard.sh` 138 lines) under `packs/dogfood-lifecycle-pack/`. The PostToolUse capture hook's matcher covers exactly 9 tool types (`Bash\|Edit\|Write\|Read\|Grep\|Glob\|Task\|WebFetch\|WebSearch`), each captured as a `dfl:ToolEvent` PROV-O activity node. Session-end validation now performs real SHACL shape-conformance (not parse-only) per v26.7.13 `RELEASE_CONTROL.md` Sec. 6's "Local tooling disclosures," with the caveat disclosed there that the live-installed `.claude/hooks/` copy may lag the tracked `packs/` source and the guard remains opt-in, not default-on for arbitrary sessions — that caveat is reproduced here, not resolved this milestone. | `packs/dogfood-lifecycle-pack/{ontology.ttl,shapes.ttl,hooks/}`; commits `8a49a8d9`, `c997a593` (v26.7.13 Increment 1 build); `649cbdbb` ("feat(dogfood-lifecycle-pack): hash-chained receipts + fix a live capture-hook race" — the session-end validation/receipt half of this claim); v26.7.13 `RELEASE_CONTROL.md` Sec. 6 | #132, #133, #134 (completed) |
| 3 | Self-monitoring conversational-discipline proof-of-concept (`packs/self-monitoring-pack/`) | ALIVE-with-disclosed-limitations | Turn-kind classification is an INPUT to the hook, never derived from text — `hook.ttl`'s SPARQL CONSTRUCT only ever reads `smon:turnKind`/`dcterms:subject`/`smon:sequenceIndex`/`smon:immediatelyFollows`, all fixture hand-asserted or heuristic-tagged facts; no NLP classifier exists or is stubbed (README's own "CLASSIFICATION-IS-INPUT FENCE"). A real, source-confirmed blank-node aliasing bug exists in `praxis-graphlaw`'s `hooks/construct.rs::instantiate_term_pattern` (echoes a CONSTRUCT template's blank-node label verbatim per solution row instead of minting a fresh node per binding); demonstrated on the broadened-topic counterfactual, where 4 `TripleStore::materialize()` rows collapse onto one aliased `_:esc` identity instead of 3 distinct obligations. The hook produced **zero firings** on this session's own real transcript (1720 turns, 10,326 triples) under its default keyword heuristic — a confirmed false negative, root-caused to two independent, named causes (topic-tag granularity: "status" vs. "cli-swarm" tagged to the same underlying question; turn-kind granularity: a real survey-shaped reply classifies as `Other` because it matches none of `transcript_to_turtle.py`'s `SURVEY_PATTERNS` regexes) — disclosed with exact turn numbers, not silently hidden. A disclosed, never-default counterfactual (`broaden_topic_experiment.py`) confirms the derivation *mechanism* itself is correct given correctly-classified input: 3 `EscalationObligation` nodes derived, earliest 18 turns (~14 minutes) before the real human frustration turn (`turn-1681`). | `packs/self-monitoring-pack/{README.md,ontology.ttl,hook.ttl,shapes.ttl,fixtures/,scripts/}`; `crates/praxis-graphlaw/tests/self_monitoring_hook_actuation.rs` (175 lines, 3/3 passing); `crates/praxis-graphlaw/tests/self_monitoring_real_session_actuation.rs` (565 lines, cross-validated via both `TripleStore` and independent `oxigraph::store::Store`); commit `0f910eec` ("feat(self-monitoring-pack): add hook-actuation pack, adversarially re-verified ALIVE") | UNTRACKED (no PROJ/task number covers this pack's build) |
| 4 | Bootstrap/cold-start limitations catalog | ALIVE | Published, 20 items across 5 parts (I. First mile, II. Bootstrap and root-of-trust, III. Last mile, IV. Recursive/self-referential bootstrap, V. Gaps found by adversarial review) — item count confirmed this session via direct header grep. Cross-checked against real `THESIS.md` citations this session (item 5 cites Sec. 12.11 "Differential verification" as the mechanism detecting actuator drift) and live repo greps (item 1's FIBO-over-private-predicate framing checked against the vendored `crates/praxis-graphlaw/ontologies/industry/financial/fibo-master/` corpus, which exists on disk). Item 13 ("Self-referential limit") discloses that the process which built MFW's own admission/planning machinery was not itself running under MFW's governance — no plan-digest-bound permission, no receipted actuation, for the commits that built the trust apparatus in the first place. | `docs/standing/BOOTSTRAP_COLD_START_LIMITATIONS.md` (265 lines, 20 numbered items, confirmed via `grep -c "^### "`); commit `5ab74266` ("docs(standing): catalog 20 MFW bootstrap/cold-start structural limitations") | UNTRACKED (no PROJ/task number covers this doc's authoring) |
| 5 | Crown-external bribery-case chain (task #137, workflow `wx3ow1zmx`) | PARTIAL_ALIVE | Stages 1–2 (RDF case fixture + compliance PDDL8 domain + Knowledge Hook obligation derivation; a new `multifractal-workflow` `[[bin]]` driving admission→hooks→PDDL→POWL v2→Arazzo artifact written to disk→AIR compile) are real and demonstrably **run**, not just written — `target/crown-bribery-case/` holds four completed run directories with the full 01–07 artifact set through F14. None of this is committed (`git status` shows every bribery-case path as `??`); task #137 remains `pending` in the tracker. Stages 3–4 (real Erlang/BEAM dispatch via `call_dispatch_statem_bridge`→Broker→multi-engine dispatch→re-admission→OCEL→receipt/replay→case-closure RDF; 7 adversarial refusal tests + runbook) have **zero evidence anywhere in the repo** — no hit for `call_dispatch_statem_bridge` in `crown-bribery-case.rs`, no `.erl` file mentions bribery, no runbook file exists. This status and its evidence are reproduced verbatim from this workflow's own prior CheckCrown stage below (`## Crown-external bribery-case — verbatim CheckCrown verdict`), not re-derived or re-worded here. | See `## Crown-external bribery-case — verbatim CheckCrown verdict` below | #137 (pending); no linked commit — the only commit matching "bribery"/"wx3ow1zmx" in `git log --all` is `353cb784` (`docs/releases/v26.7.14/PRESS_RELEASE.md`), which explicitly disclaims the build as not-yet-landed |

## Crown-external bribery-case — verbatim CheckCrown verdict

Reproduced verbatim from this workflow's prior CheckCrown stage output, not softened,
strengthened, or paraphrased. This is the authoritative evidence for Claims row 5 above.

**Task #137 status:** Task #137 ("Build crown-external bribery-case end-to-end (workflow
wx3ow1zmx)") shows status=pending in TaskList/TaskGet (confirmed twice: once via explicit
TaskList/TaskGet call, once via the auto-reminder's task snapshot). Description: "4-stage
sequential build: (1) RDF case fixture + compliance PDDL8 domain + Knowledge Hook obligation
derivation; (2) new multifractal-workflow [[bin]] driving admission->hooks->PDDL->POWL
v2->real Arazzo artifact written to disk->AIR compile; (3) real Erlang/BEAM dispatch via
call_dispatch_statem_bridge->Broker->multi-engine dispatch->re-admission->OCEL->receipt/
replay->case-closure RDF, adversarially re-verified; (4) 7 adversarial refusal tests +
runbook. Must not substitute crown_local; must traverse real Arazzo+Erlang. Only ALIVE if the
real CLI run produces the full chain with receipts/OCEL and replay verifies." No linked commit
exists (git log --all --grep for "bribery"/"wx3ow1zmx" finds only
docs/releases/v26.7.14/PRESS_RELEASE.md's commit 353cb784, which explicitly disclaims the
build as not-yet-landed). All bribery-case work found on disk (fixtures/, tests, bin) is
untracked (git status shows `??`), i.e. not committed.

**CROWN_STATUS.md snapshot:** `docs/jira/v26.7.12/CROWN_STATUS.md` read directly this session
(this doc covers the GENERAL crown-witness edge census, F02-F25; it does not itself mention
"bribery" anywhere — the bribery-case build is a separate, more specific fixture-driven task
that references crown_local as a reuse pattern). Literal current values:
`LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` = **false**;
`EXTERNAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` = **false**;
`OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` = **false** (requires both). Edge census table:
`REAL_EDGE_COUNT` (full) = **20**; `PARTIAL_REAL_EDGE` = **3** (F08→F09 shared prefix;
F10→F12 EXTERNAL; F18→F19 LOCAL); `TEST_ONLY_EDGE` = 0; `MISSING_EDGE_COUNT` = **0**;
`REFUSED_EDGE_COUNT` = 0. This exactly matches the state the caller supplied as "last known"
(LocalCrownReal=false, ExternalCrownReal=false, PARTIAL_REAL_EDGE at F08→F09 shared prefix
plus F18→F19 LOCAL and F10→F12 EXTERNAL) — no drift detected in this doc since that check.

**Recent relevant commits:**
1. `353cb784` docs(v26.7.14): add working-backwards press release, explicitly fenced — body
   explicitly states: "the crown-external chain it describes as closed and live-tested is
   still the in-progress background workflow wx3ow1zmx (task #137), with
   docs/jira/v26.7.12/CROWN_STATUS.md as of this session recording LocalCrownReal = false and
   ExternalCrownReal = false" — this is the ONLY commit in git log --all matching 'bribery' or
   'wx3ow1zmx', and it is a docs-only commit disclaiming completion, not a build commit.
2. No other commit in `git log --oneline -15` (5ab74266 .. 3322bf2d) touches bribery-case,
   wx3ow1zmx, or crown-bribery-case.rs — those 15 are unrelated feature/docs commits
   (self-monitoring-pack, dogfood-lifecycle-pack, cng plan-approval verbs, GGEN SHACL shapes,
   etc.).
3. `git status --short` shows `crates/multifractal-workflow/fixtures/bribery-case/*`,
   `crates/multifractal-workflow/src/bin/crown-bribery-case.rs`,
   `crates/multifractal-workflow/tests/bribery_case_fixture.rs`,
   `tests/bribery_case_pddl.rs`, and the Cargo.toml `[[bin]]` entries adding it are all `??`
   (untracked) — Stage 1 and Stage 2 code exists on disk but nothing has been committed.

**Verdict:** PARTIAL_ALIVE

**Evidence:** The crown-external bribery-case chain (task #137, workflow wx3ow1zmx) is
PARTIAL_ALIVE, not ALIVE and not merely PLANNED: Stage 1 (fixtures/bribery-case/{case.ttl,
hook.ttl,pddl-domain.ttl,pddl-problem-{closable,blocked}.ttl,shapes.ttl,DESIGN.md} plus 12 real
tests across tests/bribery_case_pddl.rs (3 tests) and
crates/multifractal-workflow/tests/bribery_case_fixture.rs (9 tests)) and Stage 2
(crates/multifractal-workflow/src/bin/crown-bribery-case.rs, 904 lines, wired into Cargo.toml
as a real [[bin]], driving F02 admission -> Knowledge-Hook obligation derivation -> F08 PDDL8
planning -> F09/F10 growth/POWL-v2 geometry -> F13 Arazzo manufacture+disk-write -> F14 AIR
compile) are demonstrably real and have actually been RUN, not just written:
target/crown-bribery-case/ contains four separate completed run directories
(stage3-reverify-1, adversarial-verify-run-A, live-verify-1, determinism-check-2), each with
the full 01-through-07 artifact set (admitted case, derived obligations, PDDL problem/domain
text, plan tape, POWL v2 model, Arazzo artifact + receipt) -- real evidence of chain execution
through F14. However, none of this is committed (all `??` untracked in git status), task #137
itself is still `pending` in the tracker, and Stage 3 (real Erlang/BEAM dispatch via
call_dispatch_statem_bridge -> Broker -> multi-engine dispatch -> re-admission -> OCEL ->
receipt/replay -> case-closure RDF) and Stage 4 (7 adversarial refusal tests + runbook) have
zero evidence anywhere in the repo: `grep call_dispatch_statem_bridge crown-bribery-case.rs`
is empty, no .erl file mentions bribery, and no runbook file exists. The task's own DoD states
it is "Only ALIVE if the real CLI run produces the full chain with receipts/OCEL and replay
verifies" -- that bar is unmet (chain stops at F14, no OCEL/receipt/replay for this specific
case). The broader (non-bribery-specific) crown-witness machinery this build reuses is itself
still false/false per CROWN_STATUS.md (LOCAL and EXTERNAL contiguous-path markers both false,
3 PARTIAL_REAL_EDGE among 23 unioned edges), unchanged since the caller's last known check, and
the most recent commit touching this topic (353cb784) is a docs commit that itself explicitly
disclaims the bribery-case build as still in-progress. Conclusion: real, run-verified partial
build (Stages 1-2) sitting uncommitted; Stages 3-4 not started -- PARTIAL_ALIVE, not ALIVE.

## 1. Architecture summary

v26.7.14 is a carry-forward-and-disclosure release across five claims (table above) — no claim
modifies the Chatman Engine S1–S6 pipeline or the `A = μ(O*)` equation
(`docs/CHATMAN_EQUATION.md`). Two claims touch machinery adjacent to μ: the self-monitoring pack
(Claim 3) reuses the existing `kh:Hook` SPARQL-CONSTRUCT actuation mechanism against a new,
non-compliance domain, and the crown-external bribery-case build (Claim 5) extends the
admission→hooks→PDDL8→POWL v2→Arazzo pipeline toward a specific external case, stopping at F14
(AIR compile) uncommitted. This document is not a forward-architecture design document the way
v26.7.13's ARD was — it records what already exists and was exercised this milestone, not a
ratified-but-unbuilt design.

## 2. Components

**Claim components** (architectural detail behind the Claims Reconciliation table):

| Claim | Component | Location | Status |
|---|---|---|---|
| 1 | Formal MFW PhD thesis + grounding companion | `docs/releases/v26.7.13/{THESIS.md,THESIS_GROUNDING.md}` | ALIVE |
| 2 | `dogfood-lifecycle-pack` ontology/shapes/hooks/receipts | `packs/dogfood-lifecycle-pack/` | ALIVE, carried forward |
| 3 | Self-monitoring `kh:Hook` + turn-lifecycle vocabulary + real-session actuation tests | `packs/self-monitoring-pack/`; `crates/praxis-graphlaw/tests/self_monitoring_*.rs` | ALIVE-with-disclosed-limitations |
| 4 | Bootstrap/cold-start limitations catalog (20 items) | `docs/standing/BOOTSTRAP_COLD_START_LIMITATIONS.md` | ALIVE |
| 5 | `crown-bribery-case` bin + RDF fixtures + PDDL8 domain + run artifacts | `crates/multifractal-workflow/{src/bin/crown-bribery-case.rs,fixtures/bribery-case/}`; `target/crown-bribery-case/` | PARTIAL_ALIVE, uncommitted |

No new PLANNED component is proposed by this release; Sec. 4's object model documents existing
shapes these five components exercise or extend by data, not by new type definitions.

## 3. Core invariant

This repo's existing eight invariants (`CLAUDE.md`) apply unchanged; two are singled out here
because Claims 3 and 5 touch surfaces that could otherwise erode them silently:

1. **No panics/silent defaults.** The self-monitoring pack's zero-firing result on the real
   transcript (Claim 3) is a typed, investigated, disclosed false negative — not a silent
   under-report. Both `self_monitoring_hook_actuation.rs` and
   `self_monitoring_real_session_actuation.rs` assert exact row counts (including `0`) rather
   than a loose "no crash" check.
2. **Receipts computed, never asserted.** The crown-bribery-case build's `target/
   crown-bribery-case/` run directories carry the same 01-through-07 artifact set (admitted
   case, derived obligations, PDDL text, plan tape, POWL v2 model, Arazzo artifact + receipt)
   used elsewhere in this repo's receipt discipline; Stage 5's absence (no case-closure RDF, no
   OCEL, no replay) is why Claim 5 stays PARTIAL_ALIVE rather than ALIVE — a receipt for F02–F14
   is not asserted to cover the stages that have no receipt at all.

## 4. Object model

Unlike v26.7.13's ARD, no struct or predicate below is PLANNED — every shape here is existing,
exercised RDF vocabulary or existing Rust that this milestone's claims read, write, or extend by
data.

```turtle
# EXISTING, exercised this milestone — packs/self-monitoring-pack/ontology.ttl
# Turn-lifecycle vocabulary: PROV-O / DCTERMS / SKOS + disclosed smon: terms.
# dfl:Session (reused from dogfood-lifecycle-pack, Claim 2) is the session anchor;
# smon:Turn does not mint its own session class.
smon:Turn a rdfs:Class ;
    rdfs:subClassOf prov:Activity .
# smon:turnKind: closed 5-concept scheme (GroundingQuestion, SurveyResponse, RunResponse,
# BlockerResponse, Other) -- read by hook.ttl's CONSTRUCT WHERE clause, never derived by it.
# dcterms:subject carries the "grounding_topic" tag (reused Dublin Core term, not minted).
# smon:immediatelyFollows: a direct turn-adjacency edge, chosen over FILTER(?i2 = ?i1 + 1)
# because crates/praxis-graphlaw/src/sparql/plan.rs::extract_expression does not match
# Expression::Add/Subtract (falls into the `_ => PlanExpression::Done` catch-all) -- confirmed
# this milestone by direct inspection, not assumed.
smon:EscalationObligation a rdfs:Class .
# One blank node per firing -- disclosed, bounded-scope choice (see Rule model, Sec. 6, and
# the blank-node aliasing bug this bounded scope did not anticipate).
```

```rust
// EXISTING -- crates/multifractal-workflow/src/bin/crown-bribery-case.rs (904 lines)
// Drives F02 admission -> Knowledge-Hook obligation derivation -> F08 PDDL8 planning ->
// F09/F10 growth/POWL-v2 geometry -> F13 Arazzo manufacture+disk-write -> F14 AIR compile.
// Stages 3-4 (Erlang/BEAM dispatch, OCEL/receipt/replay, refusal tests) are NOT represented
// anywhere in this binary -- confirmed via `grep call_dispatch_statem_bridge` (zero hits).
```

## 5. Standing model

This ARD was authored from this-session greps/`wc -l`/`git log` checks against the live tree
(Sec. 2, 4, and the verbatim CheckCrown verdict above) — it was **not** verified against a
freshly re-run `just standing` in this authoring session. Per `docs/standing/
CLAUDE_CODE_POLICY.md` ("if they disagree, the index wins and the doc/comment is out of date"),
`target/praxis-standing/standing.json` and `docs/standing/REALITY_INDEX.md` are authoritative
over any standing claim in this document if the two diverge; this document does not itself claim
a ladder level for any v26.7.14 claim.

Standing-policy vocabulary for this and all v26.7.14 release docs: the ladder rungs are
DISCOVERED → BUILDS → TESTED → RECEIPTED → … (per-artifact, quoted from the compiled index, not
paraphrased); "production-ready" (or pilot/publish/publication-ready) is never used unscoped —
every readiness claim requires a stated scope (`ANTI-LLM-STANDING-001`). Claim 5's PARTIAL_ALIVE
status is the closest this release comes to a rung statement, and it is explicitly not a ladder
level — it is this repo's `OPERATION_DOGFOOD_PRD.md` convention for "real, run-verified,
uncommitted, incomplete against its own DoD."

## 6. Rule model

The self-monitoring pack's hook is the one rule this milestone actually exercises against new
data (Claim 3), and it surfaces a real defect in the shared rule-evaluation engine, stated
honestly:

- `hook.ttl`'s SPARQL CONSTRUCT encodes: `grounding_question(Q) ∧ same_system(Q, Q_prev) ∧
  prior_response_was_survey(Q_prev) → derive(escalate_to_build)`. The WHERE clause pattern-
  matches over already-classified facts (`smon:turnKind`, `dcterms:subject`,
  `smon:sequenceIndex`, `smon:immediatelyFollows`); it does not classify raw text (Doctrine
  Sec. 6 of `PRD.md`).
- `crates/praxis-graphlaw/src/hooks/construct.rs::instantiate_term_pattern` echoes a CONSTRUCT
  template's blank-node label verbatim per solution row rather than minting a fresh blank node
  per binding — a pre-existing engine limitation (`crates/multifractal-workflow/fixtures/
  bribery-case/hook.ttl`'s header already discloses the same limitation for a different hook).
  This is SAFE only when at most one qualifying triple exists per `materialize()` call. The
  self-monitoring pack's broadened-topic counterfactual is the first fixture in this repo to
  actually exercise the multi-firing case, and it demonstrates the failure concretely: 4 real
  `EscalationObligation` rows collapse onto identical `_:esc` blank-node identities instead of 3
  distinct obligations (`turn-1656↔turn-1663`, `turn-1656↔turn-1703`, `turn-1663↔turn-1703` all
  alias together). This is a named follow-up, not solved this release.
- The default keyword heuristic (`transcript_to_turtle.py`) that produces `smon:turnKind` and
  `dcterms:subject` for real transcript data is disclosed as a heuristic, not NLU, and its
  granularity is the confirmed root cause of the zero-firing result on real data (Claim 3;
  Doctrine Sec. 6 of `PRD.md`).

## 7. Planner domain

The crown-bribery-case build (Claim 5) is this milestone's only planner-touching surface, and it
extends an existing pattern rather than introducing a new planner:

`crates/multifractal-workflow/src/bin/crown-bribery-case.rs` reuses the same F02→F08→F09/F10
admission-to-POWL-geometry pipeline `crown_local.rs` already exercises for the general
crown-witness census (`docs/jira/v26.7.12/CROWN_STATUS.md`), applied to a fixture-specific PDDL8
domain (`fixtures/bribery-case/{pddl-domain.ttl,pddl-problem-{closable,blocked}.ttl}`) rather
than a new planner implementation. The task's own DoD explicitly forbids substituting
`crown_local` for the real chain — confirmed this milestone via the verbatim CheckCrown verdict
above: real Stage 1–2 execution through F14, not a `crown_local`-driven simulation.

Four completed run directories exist in `target/crown-bribery-case/` (`stage3-reverify-1`,
`adversarial-verify-run-A`, `live-verify-1`, `determinism-check-2`), each with the full
01-through-07 artifact set — this is real evidence the chain executes deterministically across
independent runs through F14, though none of the four run names implies Stage 3 (Erlang
dispatch) was reached; `stage3-reverify-1`'s naming is aspirational for the run's *next* goal,
not evidence Stage 3 code exists (confirmed: `grep call_dispatch_statem_bridge
crown-bribery-case.rs` is empty).

## 8. CLI architecture

`crates/multifractal-workflow/src/bin/crown-bribery-case.rs` is a real, wired `[[bin]]` target
in `Cargo.toml` (Claim 5) — but it and its supporting fixtures/tests are entirely untracked
(`git status --short` shows `??` for every bribery-case path). Separately, and not part of any
v26.7.14 claim: `cng`'s `plan present` / `plan check` / `plan step` approval-seam CLI verbs
(`crates/cng/src/main.rs:477,498,515`, backed by `crates/cng/src/plan_approval.rs`) already
exist, landed in commit `f676f08e` during v26.7.13 — a proposed v26.7.14 exclusion naming these
verbs as "never built" was checked against the live tree this session and found not to hold; see
`PRD.md` Sec. 12 and `RELEASE_CONTROL.md` Sec. 2 for the disclosed correction. No new CLI surface
is introduced by any v26.7.14 claim.

## 9. File architecture

Files this milestone's five claims are grounded in (line counts confirmed this session via
`wc -l`):

```text
docs/releases/v26.7.13/THESIS.md                              formal MFW PhD thesis (Claim 1)
docs/releases/v26.7.13/THESIS_GROUNDING.md                     adoption-time re-check record
packs/dogfood-lifecycle-pack/ontology.ttl        160 lines     dfl: vocabulary (Claim 2)
packs/dogfood-lifecycle-pack/shapes.ttl          104 lines     SHACL shapes (Claim 2)
packs/dogfood-lifecycle-pack/hooks/*.sh          440 lines     4 hook scripts (Claim 2)
packs/self-monitoring-pack/README.md                           full disclosure + verification log
packs/self-monitoring-pack/{ontology,hook,shapes}.ttl           smon: vocabulary + hook (Claim 3)
crates/praxis-graphlaw/tests/self_monitoring_hook_actuation.rs        175 lines  3/3 passing
crates/praxis-graphlaw/tests/self_monitoring_real_session_actuation.rs 565 lines real-session tests
docs/standing/BOOTSTRAP_COLD_START_LIMITATIONS.md 265 lines    20-item catalog (Claim 4)
crates/multifractal-workflow/src/bin/crown-bribery-case.rs      904 lines  Stage 1-2 driver (Claim 5, untracked)
tests/bribery_case_pddl.rs                       188 lines     3 tests (untracked)
crates/multifractal-workflow/tests/bribery_case_fixture.rs      412 lines  9 tests (untracked)
```

No proposed forward-architecture module (à la v26.7.13's `crates/praxis-graphlaw/src/
architecture/`) exists or is proposed by this milestone.

## 10. Dataflow

No v26.7.14 claim touches the Chatman Engine S1–S6 dataflow directly (Sec. 1). The one dataflow
this release extends by real, run data is Claim 5's admission→hooks→PDDL8→POWL-v2→Arazzo→AIR
chain, stopping at F14:

1. **F02 admission** — the bribery-case RDF fixture (`case.ttl`) is admitted, real.
2. **Knowledge Hook obligation derivation** — `hook.ttl` derives compliance obligations from
   the admitted case, real, same `kh:Hook` mechanism the self-monitoring pack (Claim 3) reuses
   for a different domain.
3. **F08 PDDL8 planning** — the fixture's PDDL8 domain/problem pair produces a real plan tape,
   confirmed present in all four `target/crown-bribery-case/` run directories.
4. **F09/F10 POWL v2 geometry** — real, present in the same run directories.
5. **F13 Arazzo manufacture + disk write** — real, an actual Arazzo artifact on disk, not
   synthesized only in memory.
6. **F14 AIR compile** — real, the chain's current stopping point.
7. **Stage 3: real Erlang/BEAM dispatch** — zero evidence (Sec. 7, 8).
8. **Stage 4: OCEL, receipt, replay, case-closure RDF, adversarial refusal tests, runbook** —
   zero evidence.

Steps 1–6 are demonstrated by real, deterministic, multi-run evidence; steps 7–8 are named as
absent, not implied in-progress.

## 11. Design system

This release's vocabulary extends, rather than replaces, this repo's existing legal-industrial
register (admission, standing, receipt, refusal, replay — established in `docs/releases/
v26.7.6/ARD.md` Sec. 11, carried through v26.7.9's and v26.7.13's ARD Sec. 11): the
self-monitoring pack (Claim 3) adds `smon:Turn`/`smon:EscalationObligation` as a conversational-
domain instance of the same `kh:Hook` mechanism, deliberately reusing `dcterms:subject` and
`dfl:Session` rather than minting bridge terms where an existing public-ontology term already
fits (per this repo's public-ontology-first doctrine). The no-overclaiming vocabulary itself
(ALIVE/ALIVE-with-disclosed-limitations/PARTIAL/PARTIAL_ALIVE/PLANNED/UNKNOWN/MOCKED,
`.claude/rules/no-overclaiming.md`) is the design system for every standing claim in this
document, including the self-monitoring pack's own CLASSIFICATION-IS-INPUT FENCE (`PRD.md`
Doctrine Sec. 6), which this ARD treats as a worked example of the same discipline applied to a
new domain rather than a new discipline.

## 12. Demo architecture

The closest existing demo-shaped artifacts for this release are: the self-monitoring pack's
Stage 3 real-session run (`just test-bin self_monitoring_real_session_actuation`, 1720-turn,
10,326-triple real transcript, cross-validated through two independent SPARQL engines) and the
crown-bribery-case build's four `target/crown-bribery-case/` run directories
(`stage3-reverify-1`, `adversarial-verify-run-A`, `live-verify-1`, `determinism-check-2`).
Neither is a polished demo fixture in the SOC2/TOGAF sense (v26.7.13 ARD Sec. 12) — both are
verification-run artifacts, and the bribery-case ones are entirely untracked, not part of this
repository's committed demo surface.

## 13. Market architecture

Architecture behind any external-facing claim about this release, scoped per `.claude/rules/
no-overclaiming.md`:

- **`PRESS_RELEASE.md`'s working-backwards narrative** describes v26.7.14 as a completed
  release with a closed crown-external chain and a delivered deal-desk case study. Neither is
  true as of this document: Claim 5 is PARTIAL_ALIVE and uncommitted (Stage 2 of 4), and the
  deal-desk case study is task #139 with zero files. Any external reference to v26.7.14 must
  cite this ARD/PRD/RELEASE_CONTROL, not the press release's narrative tense.
- **Self-monitoring pack** (`packs/self-monitoring-pack/`): any external claim must state
  ALIVE-with-disclosed-limitations, never unscoped ALIVE — the zero-firing result on real data
  and the blank-node aliasing bug are load-bearing parts of the claim, not footnotes.
- **Bootstrap/cold-start limitations catalog**: the 20 items are a disclosure surface, not a
  remediation roadmap — no external claim may imply any item is fixed by virtue of being
  cataloged.
- **Crown-external chain** (Claim 5): no external claim may reference the bribery-case chain as
  "closed," "live-tested," or "real Erlang dispatch" — Stage 3–4 have zero evidence, and Stage
  1–2's real evidence is uncommitted.

## 14. Adversarial architecture

This release's adversarial-review mechanism is concentrated in the self-monitoring pack's own
three-part adversarial check (Claim 3), which this ARD treats as a worked example other packs
should follow: (1) synthetic Run/BlockerResponse pairs injected into the real, dense session
graph to confirm the hook still correctly does *not* fire on correctly-handled cases; (2) a
root-cause investigation of the real zero-firing result, finding two independent, named causes
rather than stopping at "it didn't fire"; (3) a disclosed, never-default counterfactual
(`broaden_topic_experiment.py`) that isolates the derivation mechanism from the classification
gap, proving the mechanism itself is correct and would have derived the escalation 18 turns
before the real human said so explicitly — and, in doing so, surfacing the blank-node aliasing
bug for the first time under real multi-firing conditions. Separately, the crown-bribery-case
build's `adversarial-verify-run-A` run directory name signals an adversarial-verification pass
was performed against Stage 1–2, though (per Sec. 7) this milestone found no evidence any such
pass extended to Stage 3–4, which do not exist yet.

## 15. Final-day outputs

| Output | Where | Status at authoring |
|---|---|---|
| Formal MFW PhD thesis + grounding companion | `docs/releases/v26.7.13/{THESIS.md,THESIS_GROUNDING.md}` | ALIVE, carried forward |
| `dogfood-lifecycle-pack` | `packs/dogfood-lifecycle-pack/` | ALIVE, carried forward, receipts hash-chained |
| Self-monitoring pack | `packs/self-monitoring-pack/` | ALIVE-with-disclosed-limitations, 2 named bugs |
| Bootstrap/cold-start limitations catalog | `docs/standing/BOOTSTRAP_COLD_START_LIMITATIONS.md` | ALIVE, 20 items |
| Crown-external bribery-case chain | `crates/multifractal-workflow/{src/bin/crown-bribery-case.rs,fixtures/bribery-case/}` | PARTIAL_ALIVE, uncommitted, F14 stop |
| `docs/releases/v26.7.14/PRD.md` | this directory | exists, DRAFT |
| `docs/releases/v26.7.14/ARD.md` | this directory | this document, DRAFT |
| `docs/releases/v26.7.14/RELEASE_CONTROL.md` | this directory | exists, DRAFT |
| M&A/deal-desk case study + FIBO/LEI extension | task #139 | zero files, PLANNED |
| TOGAF ADM increments 2/3 | tasks #100, #101 | zero commits, PLANNED |

## 16. Definition of done

1. Each of the five claims is independently gated per the Claims Reconciliation table above —
   its own commit, test, or task-tracker evidence, no claim rounded up beyond its stated status.
2. Every claim in this document cites a specific file, test, commit, or task-tracker record that
   exists; the crown-external bribery-case verdict (Sec. "verbatim CheckCrown verdict") was
   checked this session against the live tree (`git log`, `git status`, `grep`, `find`) rather
   than assumed from a prior session's summary.
3. No invariant of Sec. 3 is violated by any v26.7.14 claim — the self-monitoring pack's
   zero-firing result is a typed, asserted, investigated outcome, not a silent default; the
   bribery-case build's receipt discipline stops exactly where its real evidence stops (F14),
   not where a narrative would prefer it to stop.
4. Per `docs/standing/CLAUDE_CODE_POLICY.md`: this document was not verified against a freshly
   re-run `just standing` in this authoring session (Sec. 5) — the compiled `standing.json`/
   `REALITY_INDEX.md`, not this ARD, is authoritative if the two diverge.
5. A proposed exclusion (the `plan present`/`plan check`/`plan step` CLI verbs) was checked
   against the live tree rather than asserted from a prior framing, and found not to hold; the
   correction is disclosed in Sec. 8 and in `PRD.md` Sec. 12, not silently applied.

**Hard exclusions** (not gate failures — scope boundaries, matching `PRD.md` Sec. 12):
M&A/deal-desk case study + FIBO/LEI ontology extension (task #139, zero files); TOGAF ADM
increments 2 and 3 (#100, #101, zero commits); crown-external bribery-case Stages 3–4 (zero
evidence, Claims row 5); the self-monitoring pack's blank-node aliasing bug in
`hooks/construct.rs` (disclosed, not fixed); the self-monitoring pack's classification false
negative (an NLP-grade classifier is out of scope by design). Full itemization of these lives in
`PRD.md` Sec. 12, not restated here to avoid drift between the two documents.

Anything short of the five points above stays UNKNOWN in `RELEASE_CONTROL.md`. That file, not
this ARD alone, is the single control surface for what "done" means for v26.7.14.

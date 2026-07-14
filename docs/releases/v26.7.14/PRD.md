# v26.7.14 — Product Requirements Document

Status: DRAFT, tied to `RELEASE_CONTROL.md` (single control surface). Every claim in this
document cites a file, test, commit, or task-tracker record in this repository. Rows without
evidence are marked PLANNED or UNKNOWN, never asserted. This release reconciles five
independently landed or carried-forward surfaces against the repo's no-overclaiming vocabulary
(`.claude/rules/no-overclaiming.md`) — see `## Claims Reconciliation` below.
`docs/releases/v26.7.14/PRESS_RELEASE.md` is a separate, already-existing working-backwards
artifact this document does not touch; it is fenced, not authoritative — see Sec. 12 and
`RELEASE_CONTROL.md` open item 6.

## Claims Reconciliation

Every shipped-work or carried-forward claim for v26.7.14 is reconciled below against this
repository's evidentiary vocabulary (`.claude/rules/no-overclaiming.md`). This table is the
single source of truth for claim status; narrative sections elsewhere in this document must not
assert a status stronger than the row below. Status vocabulary: **ALIVE** (verified, executes,
cited test/receipt passes), **ALIVE-with-disclosed-limitations** (real and runs, but the
Scope/caveat column names confirmed limitations that must accompany any restatement of this
claim — a qualifier this document uses, not a rounding-down of ALIVE), **PARTIAL** (real but
narrower than the claim — gap named explicitly), **PARTIAL_ALIVE** (real, run-verified evidence
exists but is uncommitted and/or the chain does not yet reach its stated Definition-of-Done bar
— this repo's `OPERATION_DOGFOOD_PRD.md` convention, distinct from PARTIAL's
narrower-but-committed scope), **PLANNED** (roadmap/ticket only, no code path), **UNKNOWN** (not
yet investigated to a verdict), **MOCKED** (a stand-in exists where the claim implies the real
thing).

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

## 1. Product summary

v26.7.14 is a carry-forward-and-disclosure release across five surfaces: adoption of the formal
MFW PhD thesis (carried in from v26.7.13's own final commits); a reconfirmation of Operation
Dogfood Increment 1's `dogfood-lifecycle-pack`; a new self-monitoring conversational-discipline
proof-of-concept applying the existing `kh:Hook` SPARQL-CONSTRUCT mechanism to Claude Code's own
turn history, with two disclosed real bugs (a blank-node aliasing defect and a confirmed
false-negative on this session's real transcript); a published 20-item bootstrap/cold-start
limitations catalog; and a partial, uncommitted, run-verified build of the crown-external
bribery-case chain (Stages 1–2 of 4). It ships no new externally-facing product surface, and its
own working-backwards press release (`PRESS_RELEASE.md`) is explicitly fenced and non-authoritative
— see Sec. 12. Every claim below is scoped to the file, commit, or test that grounds it — see
`## Claims Reconciliation`.

## 2. Narrative frame

This release is a disclosure-and-partial-progress pass, not a new-capability launch. Its most
consequential finding is not a fix but a confirmed gap: the self-monitoring pack's own hook,
applying the exact mechanism this repo already trusts for compliance-obligation derivation,
produced zero firings on this session's real transcript — and the investigation traced that
false negative to two named, disclosed causes rather than either hiding it or quietly loosening
the hook until it fired. The crown-external bribery-case chain follows the same discipline: real
run evidence through F14 is claimed as real, and the absence of Stage 3/4 evidence is stated as
absence, not implied-in-progress. `PRESS_RELEASE.md`'s own "Working-Backwards Status Fence"
already discloses exactly this gap for the crown-external chain and for task #139's deal-desk
case study; this document does not soften that disclosure, and where the press release's implied
scope (an "Increment 2 permission-seam" gap) was checked against the live tree and found not to
hold, that correction is disclosed explicitly in Sec. 12 rather than silently applied.

## 3. Customer problem

A release that carries forward prior work and adds a self-diagnostic proof-of-concept needs the
same evidentiary floor as a release that ships new product surface — otherwise "carried forward"
becomes a laundering mechanism for claims that were never re-checked. The "customer" for this
release is this repo's own future maintainer and auditor sessions, who need the self-monitoring
pack's two disclosed defects (blank-node aliasing, classification false-negative), the
bribery-case chain's exact stopping point (F14, Stage 2 of 4), and the bootstrap catalog's own
self-referential limit (item 13) recorded precisely, not rounded toward a headline.

## 4. Product position

**Five independently disclosed surfaces — explicitly not a unified new capability, and
explicitly not the crown-external closure `PRESS_RELEASE.md` narrates.** Three things are out of
scope for this release and must be disclosed before any confidence-building narrative: the
M&A/deal-desk case study and its associated FIBO/LEI ontology extension (task #139, zero files),
TOGAF ADM increments 2 and 3 (tasks #100/#101, zero commits), and Stages 3–4 of the
crown-external bribery-case chain (real Erlang/BEAM dispatch, OCEL/receipt/replay, adversarial
refusal tests, runbook — all zero evidence). A fourth item was checked and found not to be a
genuine exclusion this release — see Sec. 12's disclosed correction.

## 5. Core equation

Chatman Engine remains the concrete realization of μ in the Chatman Equation
`A = μ(O*)`, `R = receipt(A)` (`docs/CHATMAN_EQUATION.md`) — `O*` the admitted observation
graph, `μ` the lawful manufacturing transformation, `A` a standing-bearing artifact, `R` a
receipt proving consequence. No v26.7.14 claim modifies the S1–S6 engine pipeline itself. The
self-monitoring pack (Claim 3) is the one surface that exercises μ's existing hook-actuation
machinery against a new domain (conversational turns rather than compliance obligations) without
changing it — the blank-node aliasing bug it surfaces is a pre-existing defect in
`hooks/construct.rs`, not a defect introduced this release. The crown-external bribery-case
build (Claim 5) is the one surface that extends μ's admission→hooks→PDDL→POWL pipeline toward a
new external case, but stops at F14 (AIR compile), uncommitted, before reaching the receipt/
replay stage that would let it claim a new `R`.

## 6. Doctrine

**CLASSIFICATION-IS-INPUT FENCE (verbatim, non-negotiable):** the self-monitoring pack's hook
(Claim 3) treats turn-kind classification as an INPUT the hook reads, never something the hook
derives from text. `hook.ttl`'s SPARQL CONSTRUCT query only ever reads `smon:turnKind`,
`dcterms:subject`, `smon:sequenceIndex`, and `smon:immediatelyFollows` — every one of those is a
fixture hand-asserted or heuristic-tagged fact, never a turn-content literal (no turn-content
literal is even modeled in this vocabulary). The fence is structural, not just prose: building an
NLP classifier that reads Claude Code's actual output and assigns `smon:turnKind` is a separate,
explicitly out-of-scope problem this pack does not attempt, does not stub, and does not claim to
solve — the same discipline the SOC2 compliance-overclaim fence (`docs/releases/v26.7.13/PRD.md`
Sec. 6) applies to a different domain. This same discipline governs every claim in this document
per `.claude/rules/no-overclaiming.md`: forbidden bare phrases do not appear anywhere in this
release's docs without a command-and-output tied to the claim in the same breath, and uncommitted
work (Claim 5) is never described as shipped.

## 7. Primary release goal

Disclose all five claims with cited commit, test, and task-tracker evidence; name the
self-monitoring pack's two real bugs (blank-node aliasing, classification false-negative)
precisely enough that a future session can act on either without re-deriving them; and state the
crown-external bribery-case chain's exact stopping point (Stage 2 of 4, uncommitted) rather than
letting `PRESS_RELEASE.md`'s working-backwards narrative stand uncorrected in this milestone's
real control surface.

## 8. MVP definition

The MVP is the five claims, each independently gated:

1. **Formal MFW PhD thesis adoption.** ALIVE. `THESIS.md` + `THESIS_GROUNDING.md` exist, cited,
   with v26.7.13's own disclosed adoption-time deltas unretracted.
2. **Operation Dogfood Increment 1.** ALIVE, carried forward. `dogfood-lifecycle-pack`
   unchanged in shape this session; hash-chained receipts and the SHACL-real session-end fix
   both confirmed still present.
3. **Self-monitoring conversational-discipline PoC.** ALIVE-with-disclosed-limitations. Real
   hook, real tests, zero firings on the real transcript under the default heuristic — a
   confirmed false negative with two named root causes; a real blank-node aliasing bug in
   shared engine code, disclosed not fixed.
4. **Bootstrap/cold-start limitations catalog.** ALIVE. Published, 20 items, cross-checked
   against `THESIS.md` and live repo greps this session.
5. **Crown-external bribery-case chain.** PARTIAL_ALIVE. Stages 1–2 real and run-verified
   (through F14); uncommitted; Stages 3–4 zero evidence; task #137 still pending.

## 9. Personas

- **Founder-operator.** Needs this document's Claims Reconciliation table, not
  `PRESS_RELEASE.md`'s working-backwards narrative, as the standing surface for what actually
  shipped this cycle.
- **AI agent / future session.** Consumes the self-monitoring pack's two disclosed bugs
  (Claim 3) to avoid re-discovering the blank-node aliasing defect or the classification false
  negative from scratch, and consumes Claim 5's exact stopping point (F14, Stage 2 of 4) to know
  precisely what remains before resuming task #137.
- **Adversarial reviewer.** Served directly by the self-monitoring pack's own adversarial
  checks (1)–(3) — synthetic Run/BlockerResponse injection into the real session graph, the
  root-cause investigation of the zero-firing result, and the broadened-topic counterfactual
  that isolates the mechanism from the classification gap — and by this document's disclosed
  correction of a proposed exclusion that did not hold (Sec. 12).
- **Compliance/legal-industrial-vocabulary reader.** Must be told, not left to discover, that
  the crown-external bribery-case chain (Claim 5) is a partial, uncommitted, F14-stopping build
  — not the closed, live-tested chain `PRESS_RELEASE.md` narrates from a future vantage point.

## 10. Functional requirements

| # | Requirement | Evidence surface |
|---|---|---|
| F1 | Formal MFW PhD thesis, adopted with a grounding companion recording adoption-time deltas | `docs/releases/v26.7.13/THESIS.md`, `THESIS_GROUNDING.md`; commit `ee042f54` |
| F2 | `dogfood-lifecycle-pack`: ontology + SHACL shapes + 9-tool-type PostToolUse capture + session-end SHACL validation + hash-chained receipts | `packs/dogfood-lifecycle-pack/`; commits `8a49a8d9`, `c997a593`, `649cbdbb` |
| F3 | Self-monitoring pack: `kh:Hook` SPARQL-CONSTRUCT mechanism applied to conversational turns, with classification-is-input fence structurally enforced | `packs/self-monitoring-pack/`; `crates/praxis-graphlaw/tests/self_monitoring_{hook_actuation,real_session_actuation}.rs`; commit `0f910eec` |
| F4 | Bootstrap/cold-start limitations catalog, 20 items, cross-checked against thesis + live repo | `docs/standing/BOOTSTRAP_COLD_START_LIMITATIONS.md`; commit `5ab74266` |
| F5 | Crown-external bribery-case chain, Stages 1–2 (admission→hooks→PDDL8→POWL v2→Arazzo→AIR), run-verified | `crates/multifractal-workflow/{fixtures/bribery-case/,src/bin/crown-bribery-case.rs,tests/bribery_case_fixture.rs}`; `tests/bribery_case_pddl.rs`; `target/crown-bribery-case/` run directories (all uncommitted) |

## 11. Non-functional requirements

1. **Determinism.** No v26.7.14 claim touches a non-deterministic code path; the self-monitoring
   pack's own adversarial checks (Sec. 8, Claim 3) confirm the same unmodified CONSTRUCT query
   produces identical results across `TripleStore` and an independent `oxigraph::store::Store`.
2. **Typed refusal completeness.** Unchanged by this release; no theme modifies `Refusal`/
   `CngRefusal` taxonomies.
3. **No wall clock in receipt/hash paths.** Unchanged by this release; the bribery-case build's
   `target/crown-bribery-case/` run directories are step-ordered artifacts, not timestamp-keyed.
4. **Classification-is-input structural fence.** See Doctrine Sec. 6 — duplicated here only by
   reference, not restated, to avoid drift between the two sections.
5. **Uncommitted-work discipline.** Claim 5's Stage 1–2 evidence is real and run-verified but
   explicitly not claimed as "shipped" — nothing in this document implies task #137 is closed or
   that the bribery-case artifacts are part of this repository's committed history.

## 12. Out of scope

1. **M&A / deal-desk case study and FIBO/LEI ontology extension** — task #139, zero files, not
   started. `git log --oneline --all --grep="139\|deal-desk\|M&A" -i` finds no build commit;
   `TaskGet` confirms status=pending ("Future: M&A case study — LLM-as-decomposition-layer
   stress test").
2. **TOGAF ADM increments 2 and 3** — tickets #100 (`ea-adm` bench category + roles +
   `meridian-adm` bundle) and #101 (F09 recursion + crown witness + v26.7.13 docs). Zero
   commits against either ticket this release; both remain PLANNED, unchanged from v26.7.13's
   own identical exclusion.
3. **Crown-external bribery-case Stages 3–4** — real Erlang/BEAM dispatch via
   `call_dispatch_statem_bridge`, OCEL/receipt/replay, case-closure RDF, and the 7 adversarial
   refusal tests + runbook. Zero evidence anywhere in the repo (Claims row 5).
4. **Self-monitoring pack's blank-node aliasing bug** (`hooks/construct.rs::
   instantiate_term_pattern`) — disclosed, demonstrated on the broadened-topic counterfactual,
   not fixed this release; lives in shared engine code, not the pack itself.
5. **Self-monitoring pack's classification false negative** — an NLP-grade turn-kind/topic
   classifier is explicitly out of this pack's scope by design (Doctrine Sec. 6); the two named
   root causes (topic-tag granularity, turn-kind granularity) remain unaddressed.

**Disclosed correction to a proposed exclusion, checked and dropped:** an earlier framing of
this release's scope proposed a third named exclusion — "the Increment 2 permission-seam CLI
verbs (`plan present`, `plan check`, `plan step --approved`), named in this repo's own earlier
v26.7.13 grounding work and never built." A grep across `crates/cng/src/` this session found
this to be factually incorrect: `plan present`, `plan check`, and `plan step` are real, wired
`#[verb(...)]` CLI entries (`crates/cng/src/main.rs:477,498,515`), backed by
`crates/cng/src/plan_approval.rs` (with `plan_approval_test.rs`), landed in commit `f676f08e`
("feat(cng): add plan present/check/step approval-seam CLI verbs (Operation Dogfood
Increment 2)") — already part of v26.7.13's own commit history, not a v26.7.14 gap. This item
is dropped from the exclusions list above rather than asserted as true; per this repo's
no-overclaiming discipline, a false exclusion is exactly as much of an overclaim as a false
completion claim. See `RELEASE_CONTROL.md` Sec. 2 for the identical, verbatim-reused
disclosure.

## 13. Day-one finish plan

1. Resume task #137: build Stage 3 (real Erlang/BEAM dispatch via
   `call_dispatch_statem_bridge`→Broker→multi-engine dispatch→re-admission→OCEL→receipt/replay)
   against the already-run-verified Stage 1–2 artifacts in `target/crown-bribery-case/`.
2. Commit the bribery-case Stage 1–2 work currently sitting `??` untracked, or explicitly
   record why it remains uncommitted going into the next milestone.
3. Investigate the self-monitoring pack's blank-node aliasing bug in
   `hooks/construct.rs::instantiate_term_pattern` — a shared-engine fix, not scoped to the pack.
4. Decide whether to invest in a real turn-kind/topic classifier for the self-monitoring pack,
   or explicitly re-park classification as permanently out of scope for this pack's design.
5. Open task #139 (M&A/deal-desk case study + FIBO/LEI) or explicitly re-park it for a later
   milestone, consistent with TOGAF increments 2/3's existing PLANNED disposition.

## 14. Acceptance criteria

Each row's status is the literal Claims Reconciliation disposition, not a paraphrase.

| # | Criterion | Proof required | Status |
|---|---|---|---|
| 1 | Formal MFW PhD thesis adoption | `THESIS.md` + `THESIS_GROUNDING.md` exist, commit `ee042f54` | ALIVE |
| 2 | Operation Dogfood Increment 1 (`dogfood-lifecycle-pack`) | ontology/shapes/hooks present, 9-tool-type matcher confirmed, `649cbdbb` receipts | ALIVE |
| 3 | Self-monitoring conversational-discipline PoC | 3/3 + real-session tests passing; zero-firing + blank-node-aliasing bugs named | ALIVE-with-disclosed-limitations |
| 4 | Bootstrap/cold-start limitations catalog | 20 items confirmed, cross-checked against `THESIS.md` and live greps | ALIVE |
| 5 | Crown-external bribery-case chain | Stage 1–2 run-verified through F14; Stage 3–4 zero evidence; uncommitted | PARTIAL_ALIVE |
| 6 | M&A/deal-desk case study + FIBO/LEI extension | zero files, task #139 pending | PLANNED — does not invalidate rows 1–5 |
| 7 | TOGAF ADM increments 2–3 | zero commits against #100/#101 | PLANNED — does not invalidate rows 1–5 |

Verdict: five claims independently ALIVE, ALIVE-with-disclosed-limitations, or PARTIAL_ALIVE as
scoped above, with all forward-looking and unstarted work (task #139, TOGAF increments 2/3,
bribery-case Stages 3–4) held to PLANNED or explicitly zero-evidence — no row in this document
rounds up, and the one proposed exclusion that did not survive a grep check is disclosed as
dropped (Sec. 12) rather than silently asserted.

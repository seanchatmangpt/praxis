# RELEASE_CONTROL — v26.7.13

Status: DRAFT. Single control surface for `PRD.md` and `ARD.md` in this directory. Both
documents' Status lines tie to this file. If this file and either document disagree, this
file wins.

## 1. Evidentiary floor

There is no single audit gate spanning v26.7.13, unlike v26.7.9's Gate F verdict. The
evidentiary floor is per-theme: each of the eight themes (A–H) in the Claims Reconciliation
table is independently gated by its own cited commit(s) and test evidence, and each theme's
status (ALIVE / ALIVE per fix / ALIVE, scoped / PARTIAL) is the literal disposition in that
table, not a paraphrase. `PRD.md` Sec. 14 states the aggregate verdict verbatim: "eight themes
independently ALIVE or PARTIAL as scoped above, with all forward-looking and unstarted work
(R8, TOGAF increments 2/3, the ratified Rust-only architecture) held to PLANNED — no row in
this document rounds up." This control file adopts that verdict as-is; it does not strengthen
it.

Two items are explicitly advisory / not run this release, per `PRD.md` Sec. 12 item 8: mutation
testing and line coverage. No claim is made either way for either. Theme B's ~40-fix count is
approximate and representative-sampled, not an exhaustive re-verified tally (`PRD.md` Claims
row 2) — each cited commit was verified once, per its own commit message, this session.

## 2. Named exclusions (verbatim, reused identically in PRD.md and ARD.md)

`ARD.md` Sec. 16 defers its own "Hard exclusions" list to this exact enumeration in `PRD.md`
Sec. 12 ("Full itemization of these lives in `PRD.md` Sec. 12, not restated there to avoid
drift"). This control file reproduces that same list verbatim so all three documents agree
word-for-word:

1. **TOGAF ADM increments 2 and 3** — tickets #100 (`ea-adm` bench category + roles +
   `meridian-adm` bundle) and #101 (F09 recursion + crown witness + v26.7.13 docs). Zero
   commits against either ticket this release; both PLANNED.
2. **The ratified Rust-only forward architecture** — canonical `ArchitectureSnapshot`
   carrier, truthful `SearchOutcome` algebra, dependency-footprinted semantic caching,
   Datalog specialization behind a differential promotion gate, `TraceEq`-guarded search
   reduction, six-obligation cross-slice composition, `PlanWitness`/`plancheck` verifier.
   Every item is PLANNED or UNKNOWN per the ARD, which this document does not itself detail;
   kernel-level proof authority is explicitly DEFERRED/EXCLUDED by the ratified design.
3. **Crown-witness repair R8** — `F08→F09`, `F18→F19`, `F10→F12` remain `PARTIAL_REAL_EDGE`;
   repair unstarted (Claims row 1).
4. **`Binding::len()` HashMap-iteration-order column-length issue** (`bindings.rs:22`) — found
   during Theme F's investigation, reachability via real query-constructed bindings
   unconfirmed; open, not fixed this release.
5. **67 pre-existing `cng` clippy findings** (workday/measurement/`otel_*`/runner +
   `jira_routes` formatting) — pre-existing debt, untouched by this release.
6. **`EXPECTED_FACTORY_HEAD` pinning risk** in
   `clients/autonomic-platform/tests/run-evidence-pass.mjs` — disclosed by
   `docs/GGEN_PARITY.md`, still pinned against a chain head that Theme C's clean sync has
   since moved; not re-pinned this release.
7. **tier2/tier3 `knowledge_hooks_e2e` residual failures** — `test_b6_multi_strata_evaluation`
   is fixed by Theme F's `strip_comments` correction, but a separate hook/rule-interleaving
   gap remains (hook-derived facts not visible to N3 rules within the same `materialize()`
   call); tier3's three named failures (`test_c3_construct_empty_no_receipt`,
   `test_c3_datalog_construct_delta_cascade`, `test_c3_threshold_count_window_concurrency`)
   are named and confirmed unrelated to Theme F's fix, root causes not investigated this
   release.
8. **Mutation testing and line coverage** — not run this release; no claim made either way.

## 3. Standing-index disclosure

Per `docs/standing/CLAUDE_CODE_POLICY.md` ("if they disagree, the index wins and the
doc/comment is out of date"), `target/praxis-standing/standing.json` and
`docs/standing/REALITY_INDEX.md` are authoritative over any standing claim in `PRD.md` or
`ARD.md` if the two diverge. `ARD.md` Sec. 5 discloses that this milestone's docs were authored
from targeted greps/`wc -l` checks against the live tree, not from a freshly re-run
`just standing` in the authoring session — neither `PRD.md` nor `ARD.md` claims a ladder level
for any v26.7.13 theme or for the forward architecture, and this control file does not add one.

Standing-policy vocabulary for this release: the ladder rungs are DISCOVERED → BUILDS → TESTED
→ RECEIPTED → … (per-artifact, quoted from the compiled index, not paraphrased). Per
`ANTI-LLM-STANDING-001`, "production-ready" (or pilot/publish/publication-ready) is never used
unscoped anywhere in this release's docs — every readiness claim requires a stated scope. If a
fresh `just standing` run this release cycle produces a ladder reading that conflicts with any
theme's status in the Claims Reconciliation table, the compiled index wins and the table is
out of date until corrected in the same commit as the standing refresh.

## 4. Claims Reconciliation governance

The `## Claims Reconciliation` table is authored once, in `PRD.md` — that file is authoritative
for claim status, scope, and evidence. `ARD.md` reproduces the identical table verbatim (not by
reference) per this milestone's explicit mirroring requirement (`ARD.md` Sec. "Claims
Reconciliation": "the two files must never drift on status, scope, or evidence for the same
claim number"). Any status change to a claim requires updating both files in the same commit;
a change landed in only one file is a defect in that commit, not a valid interim state. PROJ
ticket numbers cited in the table must resolve against this milestone's actual ticket records
(R8, #99–#121, finding #13, #85) — not fabricated or renumbered.

## 5. Open items tracked against ticket status

Ten-item disclosure register, reconciled against this cycle's re-verification work. Every
RESOLVED row cites its exact commit; every OPEN/PARTIAL row states exactly what remains
undone. No row here rounds a status up beyond what its cited evidence supports.

| # | Item | Status | Severity | Ticket |
|---|---|---|---|---|
| 1 | Crown-witness repair R8 — 3 `PARTIAL_REAL_EDGE` edges (`F08→F09`, `F18→F19`, `F10→F12`) | OPEN — repair unstarted | HIGH | R8 |
| 2 | ggen `dogfood_regression` closure gap | RESOLVED (commit `f8319978`) — no action owed | INFO | N/A (Theme C, #102–#105/#111) |
| 3 | tier2/tier3 `knowledge_hooks_e2e` residual failures — re-verified this cycle | PARTIAL — `test_b6_multi_strata_evaluation`'s original truncation bug is FIXED, confirmed live; it now fails on a SEPARATE, newly-found bug: hook-derived facts not visible to N3 rules within the same `materialize()` call, likely a stratification-vs-hook-evaluation ordering gap — new open item, not yet fixed. tier3's `test_c3_construct_empty_no_receipt` (receipt-generation logic issue) and `test_c3_datalog_construct_delta_cascade` (`Datalog predicate missing argument`, likely a FILTER-clause-in-`kh:program` parsing gap) are confirmed unrelated to `strip_comments`; neither fixed. `test_c3_threshold_count_window_concurrency` remains tier3's third named failure, root cause not investigated this release. | MEDIUM | UNTRACKED (multi-file scope; commit `bf982815` re-verification) |
| 4 | `Binding::len()` HashMap-iteration-order column-length issue (`bindings.rs:22`) | OPEN — reachability via real query-constructed bindings unconfirmed | LOW-MEDIUM | N/A |
| 5 | 67 pre-existing `cng` clippy findings (workday/measurement/`otel_*`/runner + `jira_routes` fmt) | OPEN — untouched this release | MEDIUM | N/A |
| 6 | `cng` ↔ `multifractal-workflow` dev-dependency constraint (`soc2_growth` test-scoped) | DISCLOSED (BY DESIGN) — structural constraint, not a defect | MEDIUM | N/A |
| 7 | `run-evidence-pass.mjs` pinned `EXPECTED_FACTORY_HEAD` | OPEN — still pinned against a chain head Theme C's clean sync has since moved | LOW | #102–#105, #111 |
| 8 | arazzo `arazzo_runner_broker_test` hang (task #85) | CLOSED (unsupported-by-record) — two independent Explore agents found no repo evidence; latest record 17/17 passing, re-verified 3x | INFO | #85 |
| 9 | `crates/multifractal-workflow/ignore_tests.py` (scratch script, could mass-mute 16 honest-refusal tests) | RESOLVED (commit `ad0fe530`) — file deleted | INFO | N/A |
| 10 | Solace→Arclight rename | RESOLVED (commits `8f461232`, `7b6a08e0`, `bf982815`'s `roles_test.rs` fix) — independently re-verified twice by separate dogfood audits (`wl8n77q65`, `w04f1v4su`), zero remaining discrepancies | INFO | #107–#119 |
| 11 | v26.7.13 Dry-Run Publish DoD (6-gate/falsifier/outcome-algebra) — separate from the eight-theme A-H release this control file otherwise governs | REFUSED — see `DRY_RUN_PUBLISH_VERDICT.md` for the authoritative gate-by-gate breakdown; Gate 1 fails on clean-worktree and PRD/ARD/RELEASE_CONTROL-sync checkboxes, Gates 2-6 have zero executed evidence because `crates/cng/src/bench/dry_run_publish.rs`/`_test.rs` is still absent (the pack's 9 templates + 9 rendered fixtures were authored this cycle — c97adb8f, a60d724d — and parse-validate via `ggen graph validate --files`; nothing executes the gates yet, so the verdict stays REFUSED; see the 2026-07-14 addendum in `DRY_RUN_PUBLISH_VERDICT.md`) | HIGH | N/A (not a numbered ticket; tracked only via the verdict doc) |
| 12 | Operation Dogfood PRD — separate 12-claim (C1-C12) Claims Reconciliation table plus a Grounding Appendix, distinct from this control file's eight-theme A-H table | DISCLOSED — `OPERATION_DOGFOOD_PRD.md` is target-state functional requirements (FR-1..FR-22, NFR-1..NFR-12) for making Claude Code's own lifecycle MFW-governed; its own table shows 1 ALIVE (C1), 3 PARTIAL_ALIVE (C9, C10, C11), 6 PLANNED (C2-C7), and 2 REFUSED (C8 real dry-run publish, C12 autonomous external publication) — no row here rounds that up; this control file does not merge or restate that table | HIGH | N/A (not a numbered ticket; tracked only via the PRD's own table) |
| 13 | `VISION_2030.md` — 2030 target-state thesis adopted verbatim from an external source this cycle | DISCLOSED — fenced by its own "Working-Backwards Status Fence" (top of file): describes an aspirational 2030 end state, not v26.7.13 standing; current standing remains governed exclusively by this file and the Claims Reconciliation tables in `PRD.md`/`ARD.md`/`OPERATION_DOGFOOD_PRD.md`, which win on any apparent disagreement | INFO | N/A (not a numbered ticket; no claim in this document is load-bearing for release status) |
| 14 | `PRESS_RELEASE.md` — working-backwards narrative announcing v26.7.13 as a completed release | DISCLOSED — fenced by its own "Working-Backwards Status Fence" (bottom of file): actual release standing is controlled by this control file's claims ledger, Definition of Done, receipts, and replay report; no claim in the narrative supersedes a `PARTIAL_ALIVE`/`BLOCKED`/`REFUSED`/`UNKNOWN`/`UNSUPPORTED` verdict produced by the real release run | INFO | N/A (not a numbered ticket; no claim in this document is load-bearing for release status) |

`DRY_RUN_PUBLISH_VERDICT.md` is the authoritative status source for the dry-run-publish gate
specifically — this register's row 11 defers to it rather than restating its evidence, the same
deferral pattern row 1 uses for crown-witness repair R8 against `CROWN_STATUS.md`. Rows 12-14
similarly defer to each named document's own status fence or claims table rather than restating
it here; this control file does not merge those tables into its own and remains authoritative
only over the eight-theme A-H release should any apparent conflict arise.

## 6. Documents governed by this control surface

- `docs/releases/v26.7.13/PRD.md`
- `docs/releases/v26.7.13/ARD.md`
- `docs/releases/v26.7.13/RELEASE_CONTROL.md` (this file)
- `docs/releases/v26.7.13/DRY_RUN_PUBLISH_VERDICT.md` — additional governed document, added this
  cycle; authoritative for the separate v26.7.13 Dry-Run Publish DoD gate status (open item 11).
- `docs/releases/v26.7.13/OPERATION_DOGFOOD_PRD.md` — additional governed document, adopted this
  cycle; functional/non-functional requirements (FR-1..FR-22, NFR-1..NFR-12) for making the
  Claude Code lifecycle itself MFW-governed, plus its own 12-claim Claims Reconciliation table
  and a Grounding Appendix; authoritative only for its own claims (open item 12), not merged into
  this file's eight-theme A-H table.
- `docs/releases/v26.7.13/VISION_2030.md` — additional governed document, adopted this cycle;
  2030 target-state thesis, not a claim about current v26.7.13 standing — see open item 13 and
  the document's own "Working-Backwards Status Fence".
- `docs/releases/v26.7.13/PRESS_RELEASE.md` — additional governed document, adopted this cycle;
  working-backwards narrative announcing v26.7.13 as a completed release, fenced by its own
  "Working-Backwards Status Fence" — see open item 14; this control file's claims ledger and
  Definition of Done remain authoritative over any narrative claim.

This file wins on conflict with either `PRD.md` or `ARD.md`.

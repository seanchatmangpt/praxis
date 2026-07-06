# Lane 4 — OCEL v2 and wasm4pm Process Validation

Status: DONE.

## Lane name

Lane 4 — OCEL v2 and wasm4pm Process Validation
(`docs/case-studies/autonomic-standing-factory/lane-reports/lane-4-ocel-wasm4pm.md`).
Work performed in `/Users/sac/praxis`.

## Concurrency disclosure

Before starting, `git status`/`git log` showed a clean tree (12 commits
ahead of `origin/main`, no uncommitted diff) — no live concurrent edit at
lane start. During the lane, this repo's shared `target/` build cache was
repeatedly clobbered by what is almost certainly another concurrent
session: `target/debug/my-conforming-project` flipped between having and
lacking the `ggen`-gated `plan run` subcommand three separate times over
the course of this lane, with no `cargo build` of mine in flight at the
moments it changed (confirmed via `ps aux` showing no active cargo/rustc
process at one such transition). This is the same class of contention
Lane 2's report already documented (`~/.cargo/.global-cache` lock
contention). Mitigation: this lane's driver never invokes
`target/debug/*` directly — it snapshots the three binaries it needs
(`my-conforming-project`, `ocel_process_validate`, `case_study_judge`,
each built with `--features ggen`) to `/tmp/lane4_bins/` before running,
and the driver refuses to start if a pinned binary is missing. All
evidence in this report was captured through the pinned snapshots, not
the contended shared path.

A second, more serious concurrent-edit symptom appeared after editing
`crates/praxis-graphlaw/src/lib.rs`: a hook reformatted that file (as
disclosed by the harness), and `git status` subsequently showed 57 other
files across the whole `praxis-graphlaw` crate as modified. Investigated
before touching anything further: every one of those 57 diffs was
confirmed to be pure `rustfmt` reformatting (line-wrap/import-order only —
e.g. `has_class`/`get_shape_messages` in `shacl.rs` are the same
functions before and after, just re-wrapped), not semantic drift from
another session. Per "commit only files you touched," all 57 files were
restored with `git checkout --` (discarding only the incidental
reformatting, never anyone's real work — there was none to discard), and
`crates/praxis-graphlaw/src/lib.rs` was kept as the one file with a real,
intentional change. Rebuilt and re-ran the full `praxis-graphlaw` test
suite after the revert to confirm nothing was lost (still 100% green, see
below).

## Files inspected

- `docs/case-studies/autonomic-standing-factory/CASE_STUDY_CONTROL.md`,
  `lane-reports/lane-{1,2,3}-*.md` — confirmed the real files those lanes
  claim exist actually exist: `src/bin/case_study_judge.rs` +
  `case-study/final_graphlaw_verdict.json` (Lane 2);
  `src/bin/ocel_process_validate.rs`'s `--model case-study` flag +
  `case-study/powl_model.json` + `case-study/pddl-out/plan.json` (Lane 3);
  `target/praxis-standing/{standing.json,standing.ttl,standing.ocel.json}`
  (Lane 1).
- `src/bin/ocel_process_validate.rs` (full file) — the release-loop model,
  the Lane 3 case-study model, the `Report`/`ModelReport` shapes, the
  `membership_violations`/`ModelView` decision procedure.
- `/Users/sac/wasm4pm-compat/src/ocel/process_conformance.rs`,
  `/Users/sac/wasm4pm-compat/src/ocel/validate.rs`,
  `/Users/sac/wasm4pm-compat/src/ocel.rs` (read-only) — confirmed the exact
  `ChildKind` semantics (`Once` = exactly 1, not 0-or-1) and the OCEDO/OCPQ
  integrity checks (`E2O_EMPTY`, `UNDECLARED_EVENT_TYPE`, etc.) before
  authoring the driver's event/object roster.
- `clients/autonomic-platform/tests/run-evidence-pass.mjs` — the v26.7.6
  OCEL evidence driver, reused for its raw-capture/sha256/Shape-A
  event-object wire conventions.
- `clients/autonomic-platform/package.json`,
  `docs/releases/v26.7.6/CLIENT_SURFACES.md` — confirmed `npm run build`
  (vite build) is the recorded working client build command.
- `src/verbs/receipt.rs`, `src/ops.rs` (`receipt_validate_payload`,
  `archive_validated_records`) — confirmed `receipt validate` only
  *additively* archives to `data/validated_receipts/`, never moves/deletes
  the source ledger, before running it against Lane 3's
  `case-study/pddl-receipts`.
- `docs/case-studies/autonomic-standing-factory/case-study/graphlaw_judgment.ttl` —
  confirmed Criteria 1-5's `praxis:satisfied` list is hand-authored seed
  data, not derived from live file-existence checks, before deciding NOT
  to hand-edit it to flip Criteria 6-9 (that is Lane 6's claim-promotion
  job, per the control ledger's lane table).
- `crates/praxis-graphlaw/src/lib.rs`, `src/bin/case_study_judge.rs` (lines
  ~440-530) — investigated a real non-determinism finding (below).

## Files changed

- `src/bin/ocel_process_validate.rs`:
  - Added `CASE_STUDY_WASM4PM_VALIDATION_PATH`,
    `CASE_STUDY_REQUIRED_OBJECT_TYPES`, `STANDING_OCEL_VALIDATION_PATH`.
  - Fixed 3 real gaps in Lane 3's `CASE_STUDY_CHILD_SPECS`/
    `CASE_STUDY_ORDER_LABEL_PAIRS` (found while building the real driver,
    per the ticket's "fix the generator or the Lane-3 process model, never
    weaken the validator"): added the missing `utc_clock_captured` leaf;
    changed `standing_emitted` from `Once` to `AtLeastOnce` (the real
    driver runs 5 standing verbs); dropped `final_verdict_rendered`
    entirely (deferred to Lane 6 per the ticket — the order chain now goes
    `wasm4pm_process_validated -> case_study_finished` directly).
  - Added `model_ref`/`ocel_ref` fields to the existing `Report` struct
    (additive; release `run()`'s output shape stays backward-compatible —
    no consumer of `docs/releases/v26.7.6/ocel/wasm4pm-process-validation.json`
    reads an exact key set).
  - Split `run_case_study` into `run_case_study_model_only` (Lane 3's
    original, unchanged behavior with no log arg) and the new
    `run_case_study_validate` (Lane 4: full integrity + UTC-ordering +
    process-conformance + object-participation pass against a given
    case-study OCEL log, writing `case-study/wasm4pm_validation.json`).
  - Added a third `ModelKind::StandingIntegrity` (`--model
    standing-integrity <log>`) — structural-only OCEDO/OCPQ `validate`
    (no process model applies to a standing snapshot), writing
    `case-study/standing_ocel_validation.json`.
  - The v26.7.6 release model (`CHILD_SPECS`, `ORDER_LABEL_PAIRS`,
    `release_loop_model`, `run()`) is byte-for-byte unchanged.
- `crates/praxis-graphlaw/src/lib.rs` — `content_to_string` now sorts a
  cloned copy of the triple vector by each triple's own decoded text
  before serializing (real determinism bug found and fixed; see below).
  This is the only file changed in that crate — see the concurrency
  disclosure above for the 57 reformatted-then-reverted files.
- `docs/case-studies/autonomic-standing-factory/case-study/run-case-study-pass.mjs`
  (new) — the Lane 4 evidence driver. Parameterizes the same
  raw-capture/sha256/Shape-A conventions as
  `clients/autonomic-platform/tests/run-evidence-pass.mjs` rather than
  literally reusing it (different command pipeline, different target
  process model, writes directly to the final log with no Playwright
  merge step).
- `docs/case-studies/autonomic-standing-factory/CASE_STUDY_CONTROL.md` —
  phase rows 12-14 (this lane's own rows) updated to DONE with evidence
  pointers; "Final verdict source path" section updated with the real,
  honest re-run result (see "Findings not fixed" below).
- Generated (by re-running `case_study_judge` as part of the driver):
  `case-study/final_graphlaw_verdict.json`, `graphlaw_derived.ttl`,
  `graphlaw_judgment_report.md` — `generated_at_utc`/`graph_hash` refreshed
  to the live re-run values; criteria satisfaction unchanged (still
  Criteria 1-5 only — see below).
- Generated (by re-running `ocel_process_validate --model case-study` with
  no log arg, per Lane 3's model-only path): `case-study/powl_model.json`
  — now 16 children / 114 order pairs reflecting the fixed model (same
  counts as before the fix, coincidentally, since `utc_clock_captured` was
  added and `final_verdict_rendered` was removed).
- Real side effects of running real standing/receipt commands (included
  per Lane 1's own precedent of committing dogfood-run evidence):
  `.cargo-cicd/ocel/events.jsonl`, `.ggen-v2/receipt-log.jsonl`,
  `.ggen-v2/receipt.json`, `.cargo-cicd/receipts/standing-refresh-*.json`
  (append-only receipt ledgers from the real `cargo-cicd standing refresh`
  / `just standing` invocations this lane's driver made).

## Commands run

All from `/Users/sac/praxis` unless noted. The full, real command list
with UTC start/finish/exit/stdout-sha256 is in
`case-study/raw/*.txt` (one file per command) and folded into
`case-study/ocel_case_study.json`'s events. Summary:

| Command | Exit | Evidence |
|---|---|---|
| `date -u +%Y-%m-%dT%H:%M:%S.000Z` | 0 | `raw/utc-clock.txt` |
| `cargo-cicd standing refresh` | 0 | `raw/standing-refresh.txt` |
| `cargo-cicd standing report` | 0 | `raw/standing-report.txt` |
| `cargo-cicd standing verify` | 0 | `raw/standing-verify.txt` |
| `cargo-cicd claude_context show` | 0 | `raw/claude-context-show.txt` |
| `just standing` | 0 | `raw/just-standing.txt` |
| `case_study_judge` (pass 1: shacl/shex/n3/datalog evidence) | 1 (expected — `NotReadyWithReasons` is an honest non-zero exit, not a tool failure) | `raw/case-study-judge-pass1.txt` |
| `my-conforming-project plan run --goal case-study/pddl/goal.ttl --out-dir /tmp/lane4_pddl_det_recheck ...` (fresh 3rd-independent-run determinism re-check) | 0 | `raw/pddl-plan-determinism-recheck.txt` — `powl_chain_hash` identical to the canonical `case-study/pddl-out/plan.json` |
| `ocel_process_validate --model case-study` (regenerate `powl_model.json`) | 0 | `raw/powl-model-compile.txt` |
| `npm run build` (cwd `clients/autonomic-platform`) | 0 | `raw/client-build.txt` — vite build, 29 modules, `dist/` written |
| `my-conforming-project receipt validate --dir case-study/pddl-receipts` | 0 | `raw/receipt-validate-case-study.txt` — all 5 stages Pass |
| `case_study_judge` (pass 2: final verdict, after client/receipts/benchmarks evidence) | 1 (same honest reason) | `raw/case-study-judge-pass2.txt` |
| `ocel_process_validate <intermediate-log> --model case-study` (expected-incomplete intermediate check — evidence for the `wasm4pm_process_validated` event itself) | 1 (expected: 2 violations, exactly the 2 events not yet in the intermediate log) | `raw/wasm4pm-intermediate-check.txt` |
| `ocel_process_validate case-study/ocel_case_study.json --model case-study` (**the authoritative pass**) | 0 | `case-study/wasm4pm_validation.json` |
| `ocel_process_validate target/praxis-standing/standing.ocel.json --model standing-integrity` | 0 | `case-study/standing_ocel_validation.json` |
| `cargo build --features ggen --bin my-conforming-project --bin ocel_process_validate --bin case_study_judge` | 0 | repeated several times (see concurrency disclosure) |
| `cargo test --bin ocel_process_validate` | 0 | 8/8 (unchanged) |
| `cargo test --bin case_study_judge` | 0 | 5/5 (unchanged) |
| `cargo test -p praxis-graphlaw` | 0 | 147+ passed / 0 failed / 7 ignored across ~36 test binaries |
| `cargo test --features ggen --workspace --lib --bins` | 0 | every crate `test result: ok`, 0 failed, run twice (once pre-fix, once post-fix) |

## Artifacts produced

- `docs/case-studies/autonomic-standing-factory/case-study/run-case-study-pass.mjs`
  — the driver (new, 400+ lines).
- `docs/case-studies/autonomic-standing-factory/case-study/ocel_case_study.json`
  — OCEL 2.0 Shape-A log, 20 events / 11 objects / 16 declared event types /
  11 declared object types. sha256 `5260a884bd70bb0c598843f9cfa650b67100cc4d057c352ef8adde43ebb8c8cb`.
  All AT-MINIMUM-required event types present: `case_study_started`,
  `utc_clock_captured`, `standing_emitted` (x5, one per standing verb),
  `shacl_validated`, `shex_validated`, `n3_materialized`, `datalog_closed`,
  `pddl_plan_generated`, `powl_model_compiled`, `client_smoked`,
  `receipts_verified`, `benchmarks_attached`, `graphlaw_judgment_emitted`,
  `ocel_log_written`, `wasm4pm_process_validated`, `case_study_finished`.
  Objects: `case_study`, `standing_envelope`, `ocel_log`,
  `graphlaw_judgment`, `process_validation`, `client_surface`, plus
  `pddl_plan`/`powl_workflow`/`receipt_chain`/`benchmark_result` and one
  `final_verdict` placeholder object (`standing: "not_yet_produced"`) for
  Lane 6 to wire its `final_verdict_rendered` event to without restructuring
  this log.
- `docs/case-studies/autonomic-standing-factory/case-study/wasm4pm_validation.json`
  — `{is_conforming: true, fitness: 1.0, violations: [], model_ref:
  "case-study", ocel_ref: "docs/case-studies/.../ocel_case_study.json",
  ...}`. sha256 `62403c522bb610694529451d9ad6d31e328ed7c8e28b242c62505f37181c092a`.
- `docs/case-studies/autonomic-standing-factory/case-study/standing_ocel_validation.json`
  — `{valid: true, event_count: 28, object_count: 28, parse_errors: []}`
  for Lane 1's `target/praxis-standing/standing.ocel.json`. sha256
  `843e4d6f2471679f7cb89e88d1d60545b54d5e9425ec7631dec8c8ad6f7e9772`.
- `docs/case-studies/autonomic-standing-factory/case-study/raw/*.txt` (13
  files) — full command evidence.
- Regenerated: `case-study/powl_model.json`,
  `case-study/final_graphlaw_verdict.json`, `case-study/graphlaw_derived.ttl`,
  `case-study/graphlaw_judgment_report.md`.

## Tests passed

- `cargo test --bin ocel_process_validate`: 8/8, unchanged (the release
  model's own test suite — `canonical_trace_is_a_member`,
  `missing_required_event_is_rejected`, `order_violation_is_rejected`,
  `repeated_once_event_is_rejected`, `broken_benchmark_pattern_is_rejected`,
  `project_dedupe_drops_foreign_and_collapses_repeats`,
  `utc_parser_accepts_z_and_rejects_offsets_and_regressions`,
  `membership_agrees_with_language_upto`) — confirms the v26.7.6 model
  regression stays green after the case-study-model touches, per the
  ticket's explicit requirement.
- `cargo test --bin case_study_judge`: 5/5, unchanged.
- `cargo test -p praxis-graphlaw`: every test binary `ok`, 0 failed (run
  both before and after the `content_to_string` fix; the fix touches only
  serialization order, no test asserted a specific order so none needed
  updating).
- `cargo test --features ggen --workspace --lib --bins`: every crate
  `test result: ok`, 0 failed (run twice: once to confirm the Lane 4 code
  changes, once more after the `praxis-graphlaw` determinism fix).
- `ocel_process_validate ... --model case-study` (authoritative pass):
  `is_conforming: true`, `fitness: 1.0`, `violations: []` — **the ticket's
  acceptance criterion, met without weakening the validator.**
- `ocel_process_validate ... --model standing-integrity`: `valid: true`,
  `event_count: 28`, `object_count: 28`, `parse_errors: []`.
- `receipt validate --dir case-study/pddl-receipts`: all 5 stages Pass
  (schema, chain_recompute, chain_linkage, monotonic, token_replay).
- 3-way PDDL determinism re-check: the driver's fresh `plan run` into a
  throwaway dir produced `powl_chain_hash` identical to the canonical
  `case-study/pddl-out/plan.json` (Lane 3's 2 runs + this lane's 1 = 3
  independent confirmations).

## Failures found

1. **Lane 3's case-study process model had 3 real gaps** (not visible
   until a real driver tried to satisfy it): missing `utc_clock_captured`
   event entirely; `standing_emitted` modeled as exactly-once when the real
   pipeline legitimately emits it 5 times; `final_verdict_rendered` modeled
   as required-in-this-log when it is actually Lane 6's event, produced
   from evidence this lane does not have (client/Playwright + claim
   promotion). Fixed forward in `src/bin/ocel_process_validate.rs`'s
   `CASE_STUDY_CHILD_SPECS`/`CASE_STUDY_ORDER_LABEL_PAIRS` (the process
   model, not the validator — the validator's decision procedure itself
   was never touched).
2. **Real non-determinism in `case_study_judge`'s `graph_hash`**, found by
   running the judge binary 3x over byte-identical inputs and diffing:
   the derived-triple *text* differed every run (different line order),
   even though `diff <(sort run_a.ttl) <(sort run_b.ttl)` was empty (same
   triple *set*). Root cause: `praxis_graphlaw::TripleStore::content_to_string`
   serialized `self.triple_index.triples` (a `Vec<Triple>`) in whatever
   order the forward-chaining materializer last left it, and that order is
   not guaranteed stable across independent process invocations. This
   violates the operating law's "no nondeterministic hash drift in
   canonical judgment artifacts" — `graph_hash` is exactly such an
   artifact (embedded in `final_graphlaw_verdict.json`, referenced by the
   case-study's own `graphlaw_judgment_emitted` OCEL event). **Fixed
   forward** in `crates/praxis-graphlaw/src/lib.rs`: `content_to_string`
   now sorts a cloned copy of the triples by each triple's own decoded
   text (not by the store's internal interned ids, which are themselves
   insertion-order-dependent) before serializing. Re-verified: 3
   independent `case_study_judge` runs after the fix produce byte-identical
   `graph_hash`. This is an output-ordering fix only — no materializer/
   rule-evaluation logic changed, confirmed by `cargo test -p
   praxis-graphlaw` staying 100% green before and after.
3. **A hook-triggered whole-crate `cargo fmt` reformatted 57 unrelated
   `praxis-graphlaw` files** after the `lib.rs` edit (see concurrency
   disclosure). Verified every diff was reformatting-only (not logic) and
   reverted all 57 via `git checkout --`, keeping only the intentional
   `lib.rs` change.
4. **Shared `target/debug/my-conforming-project` repeatedly lost its
   `ggen` feature** mid-lane due to contended concurrent `cargo build`
   activity in this same checkout (see concurrency disclosure). Not a
   defect in this lane's own changes; mitigated by pinning snapshot
   binaries in `/tmp/lane4_bins/` for the driver's own use.
5. **Criteria 6-9 do not flip to `satisfied` even though real Lane 3/4
   evidence now exists on disk** (`case-study/pddl-out/plan.json`,
   `case-study/powl_model.json`, `case-study/ocel_case_study.json`,
   `case-study/wasm4pm_validation.json`) — confirmed this is because
   `case-study/graphlaw_judgment.ttl`'s `praxis:satisfied` list is
   hand-authored seed data (Criteria 1-5 only), not derived from live
   file-existence checks. This is **not a bug to fix in this lane** — per
   the control ledger's own lane table, promoting satisfied criteria from
   landed evidence is Lane 6's "claim promotion" job. Re-running
   `case_study_judge` twice in this lane's own driver (before and after
   landing client/receipts/benchmark evidence) correctly reproduced the
   same `NotReadyWithReasons` verdict both times (now byte-identical
   `graph_hash` too, post-fix) — honestly showing that Lane 4 landing
   evidence, by itself, does not silently promote claims. `verdict.md`
   in `docs/case-studies/autonomic-standing-factory/CASE_STUDY_CONTROL.md`'s
   "Final verdict source path" section was updated to record this finding
   for Lane 6.

## Repairs made

- `src/bin/ocel_process_validate.rs`: 3 case-study model fixes (item 1
  above), `Report` struct `model_ref`/`ocel_ref` extension, new
  `run_case_study_validate` + `run_standing_integrity` code paths.
- `crates/praxis-graphlaw/src/lib.rs`: `content_to_string` determinism fix
  (item 2 above) — the validator (`ocel_process_validate`) was never
  weakened; the fix is in the artifact generator, per the ticket's
  explicit instruction.

## Remaining external side effects

- `.cargo-cicd/ocel/events.jsonl`, `.ggen-v2/receipt-log.jsonl`,
  `.ggen-v2/receipt.json`, `.cargo-cicd/receipts/standing-refresh-*.json`
  — append-only receipt/event ledgers, real side effects of the real
  `cargo-cicd standing refresh` / `just standing` invocations this lane's
  driver made (same class Lane 1 already committed from its own dogfood
  proof). Not a blocker; not fabricated.
- `data/validated_receipts/*.json` — additive archive written by `receipt
  validate` on success (confirmed non-destructive to the source ledger
  before running it).
- `clients/autonomic-platform/dist/` (npm build output) — untracked build
  artifact from the `client_smoked` step; not committed (build output,
  not source).
- The pinned binary snapshot directory `/tmp/lane4_bins/` is outside the
  repo (system temp) and is not part of any commit.

## Handoff to next lane

- **Lane 5** (Autonomic Platform display + Playwright smoke): the
  `final_verdict:autonomic-standing-factory` object in
  `case-study/ocel_case_study.json` is a declared placeholder
  (`standing: "not_yet_produced"`) waiting for Lane 6's
  `final_verdict_rendered` event to reference it — no restructuring of
  this log needed. Lane 5's own OCEL/Playwright evidence, if it needs to
  extend this same log rather than write a separate one, should follow the
  same Shape-A wire conventions used here (see
  `run-case-study-pass.mjs`'s `addEvent`/`addObject` helpers).
- **Lane 6** (evidence manifest, claim promotion, generated verdict): real
  evidence for Criteria 6-9 now exists on disk (paths above) — promoting
  them in `case-study/graphlaw_judgment.ttl`'s `praxis:satisfied` list (and
  re-running `case_study_judge`, whose `graph_hash` is now deterministic)
  is the concrete next action to make the verdict fact recompute.
- **Lane 7** (Integration Gate Auditor): can independently re-run `cargo
  run --bin ocel_process_validate -- docs/case-studies/autonomic-standing-factory/case-study/ocel_case_study.json --model case-study`
  and `cargo run --bin ocel_process_validate -- target/praxis-standing/standing.ocel.json --model standing-integrity`
  to re-verify this lane's two headline claims without trusting this
  report's prose.

## Evidence paths

- `docs/case-studies/autonomic-standing-factory/case-study/ocel_case_study.json`
- `docs/case-studies/autonomic-standing-factory/case-study/wasm4pm_validation.json`
- `docs/case-studies/autonomic-standing-factory/case-study/standing_ocel_validation.json`
- `docs/case-studies/autonomic-standing-factory/case-study/raw/*.txt`
- `docs/case-studies/autonomic-standing-factory/case-study/run-case-study-pass.mjs`
- `src/bin/ocel_process_validate.rs`
- `crates/praxis-graphlaw/src/lib.rs`
- `docs/case-studies/autonomic-standing-factory/CASE_STUDY_CONTROL.md`

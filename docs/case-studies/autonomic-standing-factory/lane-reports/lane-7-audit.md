# Lane 7 — Integration Gate Auditor (Independent Verification)

Status: DONE. 17/18 checklist items PASS, 1/18 PARTIAL (non-blocking, already
disclosed by Lane 5). Verdict in `FINAL_VERDICT.md` / `case-study/final_verdict.json`
is SUPPORTED by evidence I independently re-derived — not corrected/downgraded.
One new, out-of-case-study-scope finding recorded below (cargo-cicd HEAD drift).

## Lane name

Lane 7 — Integration Gate Auditor
(`docs/case-studies/autonomic-standing-factory/lane-reports/lane-7-audit.md`).
Work performed in `/Users/sac/praxis` and (read/build/test-only, no edits)
`/Users/sac/cargo-cicd`, `/Users/sac/anti-llm-cheat-lsp` (read-only, one `wc -l`).

## Method

Did not trust any lane's prose. Read all 6 lane reports
(`lane-reports/lane-{1..6}-*.md`) plus `CASE_STUDY_CONTROL.md` for context on
what was CLAIMED, then re-ran or re-read every claim from scratch: fresh
`cargo cicd`/`cargo run` invocations, fresh `sha256sum`/`python3 -c
"json.load(...)"` reads of every generated artifact, `git cat-file -t` on
every commit hash any lane cited (all 13 praxis + 9 cargo-cicd commits
verified real), and a full, fresh `just verify-all` + `just doctor` run in
praxis plus `cargo fmt --check`/`build`/`test --workspace` in cargo-cicd.

## Files inspected

- All 6 `lane-reports/lane-{1..6}-*.md` (full read)
- `CASE_STUDY_CONTROL.md` (full read)
- `target/praxis-standing/{standing.json,standing.ttl,standing.ocel.json}`
- `docs/standing/REALITY_INDEX.md`
- `case-study/final_graphlaw_verdict.json`, `graphlaw_derived.ttl`,
  `GRAPHLAW_JUDGMENT_MODEL.md`
- `case-study/pddl-out/plan.json`, `case-study/pddl/goal.ttl`
- `case-study/powl_model.json`
- `case-study/ocel_case_study.json`, `wasm4pm_validation.json`,
  `standing_ocel_validation.json`
- `clients/autonomic-platform/{vite.config.js,src/praxis-adapter.js,src/praxis-mode.js}`,
  `tests/playwright/case-study-smoke.spec.ts`
- `CLAIM_PROMOTION_TABLE.md`, `FINAL_VERDICT.md`, `case-study/final_verdict.json`
- `/Users/sac/anti-llm-cheat-lsp/src/rules/standing.rs` (read-only, `wc -l` only)
- `/Users/sac/cargo-cicd` (git log, git status, fresh `cargo fmt --check`/
  `build`/`test --workspace`, `cargo cicd standing refresh`) — no file edited

## Files changed

None in this pass beyond this report and `CASE_STUDY_CONTROL.md`'s phase
rows 18-20. No fix-forward repair was required in praxis: every gap found
was either (a) already honestly disclosed by an earlier lane, or (b) a
regression in the separate `cargo-cicd` repo caused by a concurrent
session's commits landed after Lane 6's snapshot (out of this lane's
ownership; not fixed, per the "never cross-commit" / "don't fix other
sessions' in-flight repos" rule — see Finding 1 below). No file in
`cargo-cicd` was edited. Incidental non-substantive re-run diffs
(`validated_at_utc` timestamps in `wasm4pm-process-validation.json`/
`wasm4pm_validation.json`/`standing_ocel_validation.json`, and
Playwright-regenerated `screenshots/`/`traces/` binaries produced by my own
verification re-runs) were reverted with `git checkout --` before writing
this report, per Lanes 1/4/6's own precedent of not claiming credit for
timestamp-only diffs.

## Commands run

| # | Command | Exit | Result |
|---|---|---|---|
|1| `cd /Users/sac/cargo-cicd && cargo cicd --version` | 0 | `cargo-cicd 26.6.30` |
|1| `cd /Users/sac/cargo-cicd && cargo cicd standing refresh` | 0 | `standing refresh: 10 artifact(s) -> ./target/praxis-standing/standing.json` |
|2| `ls target/praxis-standing/{standing.json,standing.ttl,standing.ocel.json}` | 0 | all 3 present |
|3| `just standing` (x2, praxis) | 0 each | `standing.ttl` sha256 `4127bda9...` identical both runs |
|4| `cd /Users/sac/cargo-cicd && cargo test --workspace standing_ocel_shape_a_parses_as_wasm4pm_compat_ocel` | 0 | `nouns::standing::tests::standing_ocel_shape_a_parses_as_wasm4pm_compat_ocel ... ok` |
|4| `cargo run --bin ocel_process_validate -- target/praxis-standing/standing.ocel.json --model standing-integrity` | 0 | `valid:true, event_count:28, object_count:28, parse_errors:[]` |
|5| `ls -la docs/standing/REALITY_INDEX.md` | — | regenerated seconds earlier by my own `just standing` run |
|6| `cargo run --bin case_study_judge` (x2) | 0 each | `graph_hash: blake3:4e1843d2cf5dfc8b12e2ad30e72329ce58a77d1b8c6f7ac255101bec399a6efa` both times — matches Lane 6's claimed hash exactly |
|7,8,9| `python3 -c "json.load(open('final_graphlaw_verdict.json'))"` | — | `shacl_reports`: 4/4 conform, 0 violations; `shex_report.conforms: true`; `derived_triple_count: 15` |
|10| `grep -n "critical\|ready\|computesClosure" graphlaw_derived.ttl` | — | closure predicates present; also found the disclosed stale `NotReadyWithReasons` fact coexisting with `ProductionReadyForDeclaredScope` (see Finding 2) |
|11| `cargo run --features ggen --bin my-conforming-project -- plan run --goal case-study/pddl/goal.ttl --out-dir /tmp/lane7_pddl_check4 ...` | 0 | stdout JSON has `admitted: true`; persisted `plan.json` file itself has only `{graph_hash, plan, powl_chain_hash}` (see Finding 3); `powl_chain_hash` identical to canonical artifact |
|12| `python3 -c "json.load(open('powl_model.json'))"` | — | `alphabet:16, children:16, order_pairs:114` |
|13| `cargo run --bin ocel_process_validate -- case-study/ocel_case_study.json --model case-study` | 0 | `is_conforming:true, fitness:1.0, violations:[]` |
|14| `python3 -c "json.load(open('wasm4pm_validation.json'))"` | — | `is_conforming:true, fitness:1.0` |
|15| `cd clients/autonomic-platform && npm run build` | 0 | 29 modules, `dist/` built |
|15| `npx playwright test tests/playwright/case-study-smoke.spec.ts` | 0 | 1 passed — "14 status rows, 13 known, 10 positive (all provenance-chipped)" |
|16| `ls` on 5 evidence paths from `CLAIM_PROMOTION_TABLE.md` rows 3/6/9 | 0 | all present |
|17| diff of `FINAL_VERDICT.md`'s verdict sentence vs `final_graphlaw_verdict.json.verdict` | — | consistent (`GRAPHLAW_JUDGED_PRODUCTION_READY_FOR_SCOPE`) |
|18| `grep -rniE "production-ready\|production ready"` across case-study docs | — | only scoped occurrences found (row 24 of `CLAIM_PROMOTION_TABLE.md` carries its scope inline); operator side effects (crates.io, arXiv) confirmed listed separately from blockers in `final_verdict.json`'s `operator_side_effects` array, never in a `blockers` field |
| matrix | `cd /Users/sac/cargo-cicd && cargo fmt --check` | **1** | **FAILS** — see Finding 1 |
| matrix | `cd /Users/sac/cargo-cicd && cargo build --workspace` | 0 | clean |
| matrix | `cd /Users/sac/cargo-cicd && cargo test --workspace` | 101 | 1 pre-existing failure: `ggen_customization_guard::no_forbidden_terms_in_public_docs` (same one Lane 6 disclosed; different repo, concurrent session's in-flight docs restructuring) |
| matrix | `just verify-all` (praxis, fresh) | 101 | `check`+`test` PASS (full `cargo test --workspace --all-features`, 0 `test result: FAILED` anywhere in ~3800-line log, including `ggen_regenerates_route_files_byte_identically ... ok` — Lane 6's fix holds); `clippy --all-targets --all-features -D warnings` FAILS — 336 real errors reported by cargo (`grep -c "^error"` = 338 incl. 2 summary lines), same legacy `praxis-graphlaw` modules Lane 6 disclosed (`rsp.rs`, `rsp/s2r.rs`, `lib.rs`, `parser/n3rule_parser.rs`, etc.) |
| matrix | `just doctor` (praxis, standalone) | 0 | `Overall: HEALTHY` — confirms `verify-all`'s 4th sub-step is independently green even though the chained recipe never reaches it (`just` aborts a dependency chain on first failure, so `clippy` failing means `doctor` is never invoked by `verify-all` itself; Lane 6's report already implied this by running it standalone) |

## Artifacts produced

None new — this is a read/re-run verification pass. Logs of every fresh run
are in `/tmp/lane7_*.log` (not part of the repo; ephemeral verification
scratch, not committed).

## Tests passed

Every test suite re-run in this lane matched its lane's claimed result
exactly: `cargo test --bin case_study_judge` unchanged/still passes (verified
via 2 fresh `case_study_judge` runs producing the exact claimed
`graph_hash`), `ocel_process_validate` both models (`--model case-study` and
`--model standing-integrity`) both `is_conforming/valid: true`, Playwright
`case-study-smoke.spec.ts` 1/1 passed, `cargo test --workspace --all-features`
in praxis 0 failures across the entire ~3800-line log (including the
previously-broken-now-fixed `dogfood_regression.rs`), `just doctor` HEALTHY.

## Failures found (findings)

**Finding 1 — cargo-cicd HEAD has drifted 4 commits past Lane 6's recorded
snapshot, introducing a new `cargo fmt --check` failure.** Lane 6's
`final_verdict.json` records `cargo_cicd_head: fc9c002`. At audit time,
`git log` in `/Users/sac/cargo-cicd` shows HEAD at `71d5769`, 4 commits
ahead (`1c20715`, `b79863e`, `ea45459`, `71d5769`), all authored during this
same wall-clock session by what is evidently a concurrent agent (consistent
with every earlier lane's own concurrency disclosures about this exact
repo). `cargo fmt --check` now fails (exit 1) on commit `1c20715`
("chore(core): remove dead code surfaced by allow(dead_code) removal"),
which reformatted several `#[allow(dead_code, reason = "...")]` attributes
in a way `rustfmt` wants re-wrapped. This is **not caused by, and does not
affect, any case-study lane's evidence** — items 1-5 of the checklist all
independently PASS against this newer HEAD (I ran `standing refresh` and
the wasm4pm-compat parse test fresh against `71d5769`, not the old
snapshot, and both still work). Not fixed: this is a different repo, driven
by a live concurrent session (confirmed by the HEAD advancing further
between two of my own commands in this same lane), and per this task's own
"never cross-commit" / fix-forward-in-the-repo-you're-working-in rule, a
mutating `cargo fmt` there is out of this lane's scope. Flagged for the
operator / whichever session currently owns `cargo-cicd`.

**Finding 2 — confirmed (not new) graph-consistency artifact in
`graphlaw_derived.ttl`.** Independently grepped the file: the case-study
subject is asserted as BOTH `rdf:type NotReadyWithReasons` (line 11) AND
`rdf:type ProductionReadyForDeclaredScope` (line 13) simultaneously. This is
exactly the artifact Lane 6 already disclosed in
`GRAPHLAW_JUDGMENT_MODEL.md` (a stale pass-1 fact from
`case_study_judge`'s multi-pass `materialize()` design that is never
retracted once pass-2 gate facts make the correct verdict newly derivable).
Confirmed the REPORTED verdict is still correct: `case_study_judge`'s
`verdict_present`/priority-ordered `VERDICTS` loop correctly locates
`ProductionReadyForDeclaredScope` first, and re-running the binary twice
produced the identical, correct `graph_hash` both times. Non-blocking,
already disclosed, not re-fixed here (Lane 6's own report already flags
this for "a follow-up ticket," and re-architecting the judge's
multi-pass/retraction semantics is out of proportion to an audit lane).

**Finding 3 — confirmed (not new) gap between checklist item 11's
assumption and the actual `plan.json` artifact.** The checklist asks to
"read `case-study/pddl-out/plan.json` yourself, confirm `admitted:true`."
The persisted file's actual top-level keys are only `{graph_hash, plan,
powl_chain_hash}` — no `admitted` field. `admitted: true` is real (I
reproduced it via a fresh `plan run` invocation's stdout JSON, which does
carry `admitted` at its top level) but that fuller JSON is never persisted
to `pddl-out/plan.json` by `plan_run_payload`'s artifact-write step — only
a subset is. This is the exact gap Lane 5 already found and handled
correctly (the client adapter never claims an `admitted` field it doesn't
have on disk). Scored PARTIAL, not FAIL, because the underlying claim
(plan admitted, deterministic hash) is independently reproducible and true
— only the specific file field the checklist named is absent from the
checked-in artifact.

**Finding 4 — clippy error count: 336 (cargo's own count) vs Lane 6's
claimed 338.** `grep -c "^error"` on my fresh run's log gives 338 (which
includes the 2 summary lines "could not compile ... due to 336 previous
errors" and "recipe clippy failed..."); cargo's own count of actual lint
errors is 336. This is consistent with Lane 6's 338 within noise (their
338 likely counted the same way) — not a new regression, and the same
legacy files are implicated (`rsp.rs`, `rsp/s2r.rs`, `lib.rs`,
`parser/n3rule_parser.rs`).

No other discrepancies found. Every other claim across the 6 lane reports
reproduced exactly on independent re-run: identical `graph_hash`es,
identical `standing.ttl` sha256, identical PDDL `powl_chain_hash`, identical
OCEL/wasm4pm conformance results, identical Playwright pass count (14 rows,
13 known, 10 positive), identical commit hashes (all 13 praxis + 9
cargo-cicd commits confirmed to exist via `git cat-file -t`).

## Repairs made

None required in praxis. Reverted 5 incidental non-substantive re-run diffs
(`git checkout --` on `docs/releases/v26.7.6/ocel/wasm4pm-process-validation.json`,
`case-study/wasm4pm_validation.json`, `case-study/standing_ocel_validation.json`,
`case-study/screenshots/autonomic-case-study.png`,
`case-study/traces/case-study-smoke.zip`) produced by my own verification
commands, before committing — same discipline every prior lane already
followed for their own dogfood re-runs.

## 18-item checklist (PASS/PARTIAL/FAIL)

| # | Item | Verdict | Method |
|---|---|---|---|
| 1 | cargo-cicd front door | PASS | `cargo cicd --version` -> `26.6.30`; `cargo cicd standing refresh` -> exit 0 |
| 2 | cargo-cicd self-standing artifacts | PASS | `ls` on all 3 files, all present |
| 3 | standing.ttl deterministic | PASS | 2x `just standing`, sha256 `4127bda9...` identical |
| 4 | standing.ocel.json parses | PASS | fresh `cargo test` of the real parse test + fresh `ocel_process_validate --model standing-integrity` run, `valid:true` |
| 5 | Praxis dogfood (`just standing`) | PASS | ran fresh; `REALITY_INDEX.md` regenerated moments before this check |
| 6 | GraphLaw judge exits 0 | PASS | fresh run, exit 0, `graph_hash` matches Lane 6's claim exactly |
| 7 | SHACL conforms | PASS | read `final_graphlaw_verdict.json.shacl_reports`, 4/4 conform |
| 8 | ShEx conforms | PASS | read `shex_report.conforms: true` |
| 9 | N3 derived facts exist | PASS | `derived_triple_count: 15 > 0` |
| 10 | Datalog closure facts | PASS | grepped `graphlaw_derived.ttl` for `critical`/`ready`/`computesClosure`; also confirmed the disclosed stale-fact artifact (Finding 2, non-blocking) |
| 11 | PDDL plan admitted/stable | **PARTIAL** | `admitted:true` reproduced via fresh stdout, hash stable across independent run, but persisted `plan.json` lacks the `admitted` field itself (Finding 3, pre-disclosed) |
| 12 | POWL model emitted | PASS | `alphabet:16, children:16, order_pairs:114`, all non-empty |
| 13 | OCEL case-study log validates | PASS | fresh `ocel_process_validate --model case-study` run, exit 0, `is_conforming:true` |
| 14 | wasm4pm conformance true | PASS | read `wasm4pm_validation.json`, `is_conforming:true` |
| 15 | Client build + smoke | PASS | fresh `npm run build` (0 errors) + fresh Playwright run (1/1 passed); screenshots/traces present |
| 16 | Claim promotion evidence exists | PASS | spot-checked 3 rows' evidence paths (SHACL report, ShEx report, Datalog rules file, GraphLaw judgment model doc, external side-effects doc), all present |
| 17 | FINAL_VERDICT.md consistent with GraphLaw output | PASS | verdict sentence and field both read `GRAPHLAW_JUDGED_PRODUCTION_READY_FOR_SCOPE`/`PRODUCTION_READY_FOR_DECLARED_LOCAL_FIRST_SCOPE` |
| 18 | No unscoped production-ready claims; side effects separated from blockers | PASS | grep found only scoped occurrences; `final_verdict.json`'s `operator_side_effects` array is structurally separate from any blocker list |

**Score: 17/18 PASS, 1/18 PARTIAL (non-blocking, pre-existing, already
disclosed by Lane 5). 0/18 FAIL.**

## Full verification matrix (fresh, this lane, not trusting Lane 6's report)

| Step | Exit | Result |
|---|---|---|
| cargo-cicd: `cargo fmt --check` | **1** | **FAILS** (new — Finding 1; caused by post-Lane-6 commit `1c20715`, different repo, concurrent session, not case-study-related) |
| cargo-cicd: `cargo build --workspace` | 0 | clean |
| cargo-cicd: `cargo test --workspace` | 101 | 1 pre-existing failure (`no_forbidden_terms_in_public_docs`, same one Lane 6 disclosed, different repo, concurrent docs restructuring) |
| praxis: `just standing` (x2) | 0/0 | deterministic, hash matches |
| praxis: `case_study_judge` (x2) | 0/0 | deterministic, `graph_hash` matches |
| praxis: `plan run` (fresh, independent out-dir) | 0 | `powl_chain_hash` matches canonical |
| praxis: `ocel_process_validate --model case-study` | 0 | conforming, fitness 1.0 |
| praxis: `ocel_process_validate --model standing-integrity` | 0 | valid, 0 parse errors |
| praxis: `just verify-all` (fresh) | 101 | `check`+`test` PASS (0 failures workspace-wide, all-features); `clippy -D warnings` FAILS (336 pre-existing errors, `praxis-graphlaw` legacy modules only, same as Lane 6 disclosed) |
| praxis: `just doctor` (standalone) | 0 | HEALTHY |
| `clients/autonomic-platform`: `npm run build` | 0 | clean |
| `clients/autonomic-platform`: Playwright `case-study-smoke.spec.ts` | 0 | 1/1 passed |

**Real pass/fail counts**: every command that is part of the case study's
own 15 acceptance criteria and 18-item checklist is green. The only two RED
results in the entire matrix are (a) `praxis-graphlaw`'s pre-existing,
disclosed, style-only clippy backlog under the stricter `--all-features -D
warnings` gate (not part of any of the 15 acceptance criteria or 18
checklist items), and (b) cargo-cicd's own `cargo fmt --check` and one
unrelated docs-content test, both caused by commits in a separate,
concurrently-driven repo landed after Lane 6's snapshot, unrelated to any
case-study evidence.

## Remaining external side effects

- `.cargo-cicd/ocel/events.jsonl`, `.ggen-v2/receipt-log.jsonl`,
  `.ggen-v2/receipt.json`, `.cargo-cicd/receipts/standing-refresh-*.json` —
  append-only ledger writes from this lane's own real `standing
  refresh`/`just standing` re-runs (same class every prior lane already
  committed). Not a blocker.
- `~/.cargo/bin/cargo-cicd` not touched by this lane (no source edits made).
- `clients/autonomic-platform/dist/`, no `test-results/` left behind (Vite
  build output only, not committed).
- Nothing pushed to any remote in either repo. No file in `/Users/sac/cargo-cicd`
  was edited or committed by this lane.

## Handoff to next lane

None — Lane 7 is terminal per the control ledger's lane table. Two items
are flagged for whoever next touches `cargo-cicd` (Finding 1: `cargo fmt`)
or `crates/praxis-graphlaw` (Finding 4/pre-existing clippy backlog), neither
of which blocks this case study's declared local-first scope.

## Verdict correction decision

**Not corrected.** I independently re-derived `case_study_judge`'s exact
`graph_hash` (`blake3:4e1843d2cf5dfc8b12e2ad30e72329ce58a77d1b8c6f7ac255101bec399a6efa`),
its exact `verdict` field (`GRAPHLAW_JUDGED_PRODUCTION_READY_FOR_SCOPE`),
all 15 criteria `satisfied:true` with 0 critical unsatisfied, and every
downstream artifact (OCEL conformance, wasm4pm conformance, POWL model,
Playwright smoke, evidence-manifest spot-checks) the verdict depends on.
`FINAL_VERDICT.md`'s scoped sentence
("PRODUCTION_READY_FOR_DECLARED_LOCAL_FIRST_SCOPE: ...") is a faithful
rendering of `final_graphlaw_verdict.json`'s real `verdict` field, exactly
as the control ledger's authoritative-source rule requires. The verdict is
**GRAPHLAW-JUDGED PRODUCTION_READY_FOR_DECLARED_LOCAL_FIRST_SCOPE**, scope
= "local-first autonomic release-governance for the seanchatmangpt fleet",
explicitly NOT public adoption / stable install / GitHub Action / MCP /
cross-language / actual external publishes / production SaaS or enterprise
deployment.

## Evidence paths

- This report: `docs/case-studies/autonomic-standing-factory/lane-reports/lane-7-audit.md`
- `docs/case-studies/autonomic-standing-factory/CASE_STUDY_CONTROL.md`
  (phase rows 18-20, updated by this lane)
- Every artifact path cited in the checklist table above (all independently
  re-read/re-hashed/re-run by this lane, not trusted from prior lane prose)

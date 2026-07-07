# Autonomic Standing Factory — Case Study Control Ledger

Status: IN PROGRESS. This file is the workflow ledger — every phase row must
point to evidence (a command, a file path, a test name). Prose without a
pointer does not count as a phase being done.

## Scope

Local-first autonomic release-governance for the seanchatmangpt fleet,
including: standing emission, GraphLaw judgment, process validation, evidence
reports, deterministic regeneration, and client display.

## Non-goals

Public adoption, stable install, GitHub Action, MCP, cross-language support,
actual crates.io publish, actual arXiv submission, production SaaS
deployment, enterprise production deployment. These are external or
downstream of this case study.

## Working repos

- `/Users/sac/cargo-cicd` — standing emitter (Lane 1)
- `/Users/sac/praxis` — standing judge + planner + process model + client +
  reports (Lanes 2–6)
- `/Users/sac/wasm4pm`, `/Users/sac/wasm4pm-compat` — process validation
  substrate (read/consumed by Lane 4, not edited unless a hard defect blocks
  the case study)
- `/Users/sac/ggen` — touched only if pack/template ownership requires it
  (not expected)
- `/Users/sac/anti-llm-cheat-lsp` — policy handoff notes only

## Lane ownership

| Lane | Owner scope | Report path |
|---|---|---|
| 1 | cargo-cicd front door + standing compiler | `lane-reports/lane-1-cargo-cicd.md` |
| 2 | GraphLaw judgment model (SHACL/ShEx/N3/Datalog) | `lane-reports/lane-2-graphlaw.md` |
| 3 | PDDL repair planner + POWL process model | `lane-reports/lane-3-pddl-powl.md` |
| 4 | OCEL v2 + wasm4pm process validation | `lane-reports/lane-4-ocel-wasm4pm.md` |
| 5 | Autonomic Platform display + Playwright smoke | `lane-reports/lane-5-client.md` |
| 6 | Evidence manifest, claim promotion, generated reports | `lane-reports/lane-6-reports.md` |
| 7 | Integration Gate Auditor | `lane-reports/lane-7-audit.md` |

## Acceptance criteria (15, from PRODUCTION_READINESS.md)

1. canonical standing evidence exists
2. GraphLaw validates shapes
3. GraphLaw validates graph structure
4. GraphLaw derives readiness facts
5. GraphLaw computes closure over blockers/dependencies
6. PDDL produces/records a lawful repair/action plan
7. POWL models the execution process
8. OCEL records the case-study run
9. wasm4pm validates process conformance
10. receipts verify where applicable
11. benchmark evidence exists where performance claims are made
12. Autonomic Platform displays case-study state with provenance
13. Claude Code policy consumes or points to standing
14. unsupported claims are diagnosable
15. external operator side effects are separated from release blockers

## Phase status

| Phase | Status | Evidence |
|---|---|---|
| 0 Control ledger + scaffold | DONE | this file; `case-study/{raw,shapes,shex,rules,pddl,pddl-out,pddl-receipts,ocel,screenshots,traces}/` created |
| 1 cargo-cicd front door | DONE | `lane-reports/lane-1-cargo-cicd.md` — dispatch/version/schema-id already fixed by a concurrent session; this lane added Shape-A OCEL emission, fixed 2 failing workspace-crate-ingestion tests, and fixed a real TTL determinism gap (Command evidence `utc` leak) found by dogfooding. One pre-existing, out-of-scope `--all-features` clippy/compile break left unfixed (documented) |
| 2 Praxis dogfood standing | DONE | `lane-reports/lane-1-cargo-cicd.md` (dogfood proof folded into Lane 1 handoff) — `just standing` run twice with no `ggen.lock` deletion, `standing.ttl` sha256 identical both times (`4127bda9...`), `docs/standing/REALITY_INDEX.md` regenerated with 12 `RustCrate` rows |
| 3 Case-study docs | DONE | `CASE_STUDY.md`, `PRODUCTION_READINESS.md` |
| 4 Evidence→RDF bridge | DONE | `case-study/graphlaw_judgment.ttl` — reuses cargo-cicd's real `praxis:` namespace, references `target/praxis-standing/standing.{json,ttl,ocel.json}` by path+sha256, seeds the 15-criterion requires/satisfied/critical dependency model (1-5 satisfied, 6-15 honestly unsatisfied) |
| 5 SHACL shapes | DONE | `case-study/shapes/*.shacl.ttl` (4 files); real run: all 4 conform, 0 violations (`case-study/shacl-report.{json,md}`) |
| 6 ShEx topology | DONE | `case-study/shex/case-study.shex` (7 shapes); real run: conforms, 0 failures (`case-study/shex-report.{json,md}`) |
| 7 N3 judgment rules | DONE | `case-study/rules/judgment.n3` — verdict facts typed as `rdf:type` classes (not a shared `hasVerdict` predicate) to keep the stratifier's negation safe; includes 1 `=> false.` denial rule (`case-study/n3-report.md`) |
| 8 Datalog readiness closure | DONE | `case-study/rules/readiness.dl.n3` — transitive closure + stratified negation over requires/satisfied/critical/blocks/depends_on; real run derives `unsatisfiedDependencyCount = 10` via the Rust `add_rule_with_aggregate` API (no text-syntax aggregate exists) (`case-study/datalog-report.md`) |
| 9 GraphLaw judge bin | DONE | `src/bin/case_study_judge.rs`, `case-study/final_graphlaw_verdict.json` — real verdict `NotReadyWithReasons` (10/15 criteria unsatisfied, several critical), derived only via SPARQL over materialized facts, never hand-written; 5 unit tests incl. an evidence-removal verdict-flip proof and a SHACL/ShEx-violation-blocks-derivation proof, all green |
| 10 PDDL repair domain | DONE | `case-study/pddl/goal.ttl` (16-action repair domain incl. lawful claim demote/re-promote); `case-study/pddl-out/plan.json`, `case-study/pddl_plan.json` — `admitted:true`, 16-step plan, `powl_chain_hash` identical across two independent runs (`lane-reports/lane-3-pddl-powl.md`) |
| 11 POWL process model | DONE | `case-study/powl_model.json` (16 children, 114 order pairs) via `ocel_process_validate --model case-study`; v26.7.6 `CHILD_SPECS`/`ORDER_LABEL_PAIRS` untouched, 8/8 release-model tests still green (`lane-reports/lane-3-pddl-powl.md`) |
| 12 Evidence driver + OCEL capture | DONE | `case-study/run-case-study-pass.mjs` (Node driver, reuses `clients/autonomic-platform/tests/run-evidence-pass.mjs`'s raw-capture/sha256/Shape-A conventions); `case-study/ocel_case_study.json` — 20 events, 11 objects, all 16 minimum-required event types present (`standing_emitted` 5x); real command evidence in `case-study/raw/*.txt` |
| 13 wasm4pm process validation | DONE | `case-study/wasm4pm_validation.json` — `is_conforming: true, fitness: 1.0, violations: []` (via `cargo run --bin ocel_process_validate -- case-study/ocel_case_study.json --model case-study`); required 3 Lane-3 process-model fixes first (`src/bin/ocel_process_validate.rs`: added missing `utc_clock_captured`, made `standing_emitted` `AtLeastOnce`, deferred `final_verdict_rendered` to Lane 6) — see `lane-reports/lane-4-ocel-wasm4pm.md` |
| 14 Standing-process validation payoff | DONE | `case-study/standing_ocel_validation.json` — `{valid: true, event_count: 28, object_count: 28, parse_errors: []}` for Lane 1's `target/praxis-standing/standing.ocel.json`, via new `--model standing-integrity` mode on `ocel_process_validate` |
| 15 Autonomic Platform screen | DONE | `clients/autonomic-platform/src/praxis-adapter.js` (`getCaseStudy()`, 14 provenance-wrapped fields, 13 real / 1 UNKNOWN), `src/praxis-mode.js` (`PraxisCaseStudyScreen`), `src/AutonomicPlatform.js` (`casestudy` screen), `vite.config.js` (5 new `/praxis-artifacts/case-study/*` routes); `tests/playwright/case-study-smoke.spec.ts` — 1/1 passed, 14 status rows (13 known + 1 UNKNOWN), 9 positive rows all provenance-chip-verified structurally; `npm run build` passed; `case-study/screenshots/autonomic-case-study.png`, `case-study/traces/case-study-smoke.zip`; `AUTONOMIC_PLATFORM_REPORT.md`, `lane-reports/lane-5-client.md` |
| 16 Reports + manifests | DONE | `EVIDENCE_MANIFEST.md` + `case-study/evidence_manifest.json` (37 artifacts, real sha256/blake3 recomputed by Lane 6), `GRAPHLAW_JUDGMENT_MODEL.md`, `PROCESS_MODEL.md`, `PDDL_REPAIR_PLAN.md`, `POWL_EXECUTION_MODEL.md`, `OCEL_REPLAY_REPORT.md`, `WASM4PM_VALIDATION_REPORT.md`, `CLAIM_PROMOTION_TABLE.md` (14/14 claims PROMOTED), `FINAL_VERDICT.md` + `case-study/final_verdict.json` — real verdict `GRAPHLAW_JUDGED_PRODUCTION_READY_FOR_SCOPE` (15/15 criteria, `unsatisfied_dependency_count: 0`), generated from `case-study/final_graphlaw_verdict.json` per the control ledger's own authoritative-source rule. Lane 6 promoted Criteria 6-15 in `case-study/graphlaw_judgment.ttl` with real Lane 3-5 evidence, found and fixed a real bug in `case_study_judge.rs`'s `verdict_present` (see `GRAPHLAW_JUDGMENT_MODEL.md`), and found+fixed an unrelated real bug in `crates/ggen/tests/dogfood_regression.rs` (tempdir pack-staging gap) during verification |
| 17 Full verification matrix | PARTIAL | Lane 6 ran a broad verification pass ahead of Lane 7 (see `lane-reports/lane-6-reports.md`'s verification section): `cargo test --workspace --all-features` and `cargo check --workspace --all-features` green; `cargo clippy --all-targets --all-features -D warnings` RED (338 pre-existing, unrelated style-only lints in `crates/praxis-graphlaw` legacy modules, disclosed not fixed — see `CLAIM_PROMOTION_TABLE.md` row 12); `just doctor` green; case-study-specific commands (`plan run`, `receipt validate`, `ocel_process_validate` both models, client build + Playwright) all green and re-verified independently by Lane 6. The FULL independent audit matrix (re-deriving every lane's headline claim from a clean vantage point) remains Lane 7's own job |
| 18 Integration Gate Auditor | PENDING | `lane-reports/lane-7-audit.md` |
| 19 Commits | PENDING | git log, both repos |
| 20 Final chat report | PENDING | this session's final message |

## Commands run

(appended by each lane as it executes — see lane reports for the authoritative
per-command record with cwd/exit-code/UTC window)

## Artifacts generated

(see `case-study/evidence_manifest.json` once Lane 6 runs — authoritative
list with hashes)

## Commits produced

(appended per repo as lanes commit — see `git log --oneline` in
`/Users/sac/cargo-cicd` and `/Users/sac/praxis` for ground truth)

## Final verdict source path

`docs/case-studies/autonomic-standing-factory/case-study/final_graphlaw_verdict.json`
— the ONLY authoritative source for the verdict sentence in `FINAL_VERDICT.md`.
Updated by Lane 6: after promoting Criteria 6-15 in
`case-study/graphlaw_judgment.ttl` with real Lane 3-5 evidence (see
`CLAIM_PROMOTION_TABLE.md`) and fixing a real bug in `case_study_judge.rs`'s
`verdict_present` (a SPARQL query that never bound its projected variable,
so it always returned `false` regardless of the graph's actual derived
facts — see `GRAPHLAW_JUDGMENT_MODEL.md`), the current real value is
`raw_verdict_fact: "ProductionReadyForDeclaredScope"` (`verdict:
"GRAPHLAW_JUDGED_PRODUCTION_READY_FOR_SCOPE"`), `unsatisfied_dependency_count:
0`, all 15 criteria `satisfied: true`. Confirmed deterministic across 4
independent `cargo run --bin case_study_judge` runs (`graph_hash`
`blake3:4e1843d2cf5dfc8b12e2ad30e72329ce58a77d1b8c6f7ac255101bec399a6efa`
every time). `FINAL_VERDICT.md` is Lane 6's rendered output of this exact
field.

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
| 12 Evidence driver + OCEL capture | PENDING | `case-study/ocel_case_study.json` |
| 13 wasm4pm process validation | PENDING | `case-study/wasm4pm_validation.json` |
| 14 Standing-process validation payoff | PENDING | `case-study/standing_ocel_validation.json` |
| 15 Autonomic Platform screen | PENDING | `case-study/screenshots/`, `case-study/traces/`, `AUTONOMIC_PLATFORM_REPORT.md` |
| 16 Reports + manifests | PENDING | `EVIDENCE_MANIFEST.md`, `CLAIM_PROMOTION_TABLE.md`, `FINAL_VERDICT.md` |
| 17 Full verification matrix | PENDING | `lane-reports/lane-7-audit.md` |
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
Produced by Lane 2 (`src/bin/case_study_judge.rs`): current real value
`raw_verdict_fact: "NotReadyWithReasons"` (`verdict:
"GRAPHLAW_JUDGED_NOT_READY_WITH_RECEIPTED_REASONS"`) — honest given Lanes
4-5 have not yet landed OCEL/wasm4pm/client evidence; re-run
`cargo run --bin case_study_judge` after those lanes land to see the
verdict fact recompute. `FINAL_VERDICT.md` itself is Lane 6's output, not
yet produced.

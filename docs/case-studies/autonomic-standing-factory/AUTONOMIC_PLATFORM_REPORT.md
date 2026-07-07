# Autonomic Platform Case-Study Display — Lane 5

Status: DONE. Work performed in `/Users/sac/praxis/clients/autonomic-platform`.

## What was built

1. `vite.config.js` — `STATIC_MAP` extended with 5 read-only case-study
   artifact routes, all mapped to real Lane 4 files under
   `docs/case-studies/autonomic-standing-factory/case-study/`:
   - `/praxis-artifacts/case-study/final-verdict.json` -> `final_graphlaw_verdict.json`
   - `/praxis-artifacts/case-study/ocel.json` -> `ocel_case_study.json`
   - `/praxis-artifacts/case-study/wasm4pm-validation.json` -> `wasm4pm_validation.json`
   - `/praxis-artifacts/case-study/powl-model.json` -> `powl_model.json`
   - `/praxis-artifacts/case-study/pddl-plan.json` -> `pddl-out/plan.json`

   All 5 paths were confirmed to exist on disk before wiring (no route wired
   speculatively against a missing file). A missing file (if one ever
   regresses) 404s and the adapter renders that field UNKNOWN — no fallback
   synthesis.

2. `src/praxis-adapter.js` — new `getCaseStudy()`, fetching all 5 endpoints
   in parallel and returning **14 provenance-wrapped fields**, each either
   `{ value, source, ref, unknown:false }` or the `UNKNOWN` sentinel:

   | Field | Source artifact | Real/UNKNOWN |
   |---|---|---|
   | `case_study_standing` | `final_graphlaw_verdict.json` (verdict/raw_verdict_fact/scope/generated_at_utc) | real |
   | `graphlaw_verdict` | same, `.verdict` | real |
   | `shacl_status` | same, `.shacl_reports` | real |
   | `shex_status` | same, `.shex_report` | real |
   | `n3_datalog_status` | same, `.derived_triple_count`/`.unsatisfied_dependency_count`/`.denials` | real |
   | `pddl_plan_status` | `pddl-out/plan.json` (`.plan`, `.powl_chain_hash`, `.graph_hash`) | real |
   | `powl_model_status` | `powl_model.json` (`.children`/`.order_pairs`/`.alphabet` counts) | real |
   | `ocel_log_status` | `ocel_case_study.json` (`.events`/`.objects`/`.eventTypes`/`.objectTypes` counts) | real |
   | `wasm4pm_status` | `wasm4pm_validation.json` (`.is_conforming`/`.fitness`/`.violations`) | real |
   | `benchmark_status` | `ocel_case_study.json`'s `benchmarks_attached` event attributes | real |
   | `receipt_status` | `ocel_case_study.json`'s `receipts_verified` event attributes | real |
   | `criteria_summary` | `final_graphlaw_verdict.json`'s `.criteria[]` (satisfied/critical-unsatisfied counts) | real |
   | `final_verdict` | same as `graphlaw_verdict` | real |
   | `external_side_effects` | — | **UNKNOWN** (see below) |

   **13 of 14 fields are real** (sourced from one of the 5 wired JSON files,
   never fabricated or computed client-side beyond simple array `.length`
   counts). **1 field is UNKNOWN by design**: `external_side_effects` — none
   of the 5 wired case-study machine artifacts carries a structured list of
   external operator side effects; that information exists only as prose in
   `lane-reports/lane-4-ocel-wasm4pm.md`'s "Remaining external side effects"
   section, which is not a machine artifact this adapter parses. Rather than
   scrape markdown prose into a fabricated structured value, the field is
   honestly `UNKNOWN`. This is visible on the deployed screen as a dashed
   `UNKNOWN` chip, not a blank or a fabricated green result.

   `pddl_plan_status` also does **not** include an `admitted` boolean: that
   key exists only in an ad-hoc raw command-log capture
   (`case-study/raw/pddl-plan-determinism-recheck.txt`) from a throwaway
   re-run, not in the checked-in `pddl-out/plan.json` artifact this adapter
   reads. Reporting only what the wired artifact actually contains, per the
   no-fabrication rule.

3. `src/praxis-mode.js` — new `PraxisCaseStudyScreen` component:
   - `StatusRow` renders each field as label + value + provenance chip, or
     `UnknownChip` when the field is UNKNOWN — never both, never neither.
   - `data-testid="status-row"` (+ `data-known`, `data-label`),
     `data-testid="status-value"` (+ `data-positive`),
     `data-testid="provenance-chip"`, `data-testid="unknown-chip"` — added
     so Playwright can assert the provenance-pairing rule structurally over
     the live DOM rather than trusting the component's own rendering logic.
   - `isPositive()` colors a value green (`PALETTE.emerald`) only for an
     unambiguous positive/pass reading; the color is cosmetic only — the
     provenance chip is emitted unconditionally alongside every known value
     regardless of color.

4. `src/AutonomicPlatform.js` — `casestudy` added to `SCREENS`/`SCREEN_META`
   (title "Standing Factory Case Study", used by `getByTitle` in the
   NavRail and by the Playwright spec) and to `PRAXIS_SCREEN_COMPONENTS`
   (so, like `deck`/`ops`, it renders real/UNKNOWN data with no
   `NonStandingBanner` in the default `praxis` data mode — the banner still
   applies in `mock` mode and to every other sim-driven screen).

5. `tests/playwright/case-study-smoke.spec.ts` (new) — loads the app in
   praxis (default) mode, clicks the NavRail's "Standing Factory Case
   Study" button via `getByTitle`, and asserts:
   - the panel and its evidence-chain heading render
   - the real GraphLaw verdict string (read from the live
     `/praxis-artifacts/case-study/final-verdict.json` response, not
     hardcoded) appears in the DOM
   - the real wasm4pm conformance word/fitness value (read from the live
     `/praxis-artifacts/case-study/wasm4pm-validation.json` response)
     appears
   - **structurally, over every rendered `status-row`**: a known row always
     carries >= 1 provenance chip and 0 unknown chips; an unknown row always
     carries >= 1 unknown chip and 0 provenance chips; every positive
     (green) value's row is independently re-checked for a provenance chip
   - screenshot saved to
     `case-study/screenshots/autonomic-case-study.png`
   - trace saved to `case-study/traces/case-study-smoke.zip` (via
     `context.tracing.stopChunk({path})` + `startChunk()`, since the
     project's `trace: 'on'` config already auto-starts tracing on the
     context — a second unconditional `tracing.start()` throws "Tracing has
     been already started")

## Build result

```
$ npm run build
> vite build
✓ 29 modules transformed.
dist/index.html                  0.46 kB │ gzip:  0.33 kB
dist/assets/index-B2sqCxxV.js   198.14 kB │ gzip: 64.75 kB
✓ built in 438ms
```

Passed both immediately after the source changes and again as the final
`npm run build` gate before commit.

## Playwright result

```
$ npx playwright test tests/playwright/case-study-smoke.spec.ts
Running 1 test using 1 worker
[spec] case-study screen: 14 status rows, 13 known, 9 positive (all provenance-chipped)
  ✓  1 [chromium] › ... case-study screen: renders sourced GraphLaw/OCEL/wasm4pm evidence with provenance (893ms)
  1 passed (2.7s)
```

14 status rows rendered (matches the 14 fields returned by `getCaseStudy()`
above), 13 known/sourced + 1 UNKNOWN (`external_side_effects`), 9 of the 13
known rows read as positive/green — every one of those 9 independently
confirmed (via the `data-positive`/`data-known` DOM attributes, not by
trusting component logic) to carry a provenance chip in the same row.

## Screenshot / trace paths

- `docs/case-studies/autonomic-standing-factory/case-study/screenshots/autonomic-case-study.png`
- `docs/case-studies/autonomic-standing-factory/case-study/traces/case-study-smoke.zip`

## Files changed

- `clients/autonomic-platform/vite.config.js`
- `clients/autonomic-platform/src/praxis-adapter.js`
- `clients/autonomic-platform/src/praxis-mode.js`
- `clients/autonomic-platform/src/AutonomicPlatform.js`
- `clients/autonomic-platform/tests/playwright/case-study-smoke.spec.ts` (new)

## Commit

`26d7d07` — "feat(case-study): display GraphLaw standing verdict and
evidence chain in Autonomic Platform with Playwright smoke"

# Lane 5 — Autonomic Platform Display and Playwright Smoke

Status: DONE.

## Lane name

Lane 5 — Autonomic Platform Display and Playwright Smoke
(`docs/case-studies/autonomic-standing-factory/lane-reports/lane-5-client.md`).
Work performed in `/Users/sac/praxis/clients/autonomic-platform`.

## Concurrency disclosure

`git status`/`git diff` at lane start showed a clean tree (14 commits ahead
of `origin/main`, no uncommitted diff in this repo or in
`clients/autonomic-platform` specifically) — no live concurrent edit
detected before starting. No hook-triggered reformatting was observed
during this lane (unlike Lane 4's `praxis-graphlaw` incident).

## Files inspected

- `docs/case-studies/autonomic-standing-factory/CASE_STUDY_CONTROL.md`,
  `lane-reports/lane-4-ocel-wasm4pm.md` — confirmed the claimed Lane 4
  artifacts are real files on disk before wiring anything to them.
- `docs/case-studies/autonomic-standing-factory/case-study/final_graphlaw_verdict.json`,
  `ocel_case_study.json`, `wasm4pm_validation.json`, `powl_model.json`,
  `pddl-out/plan.json`, and (for comparison) `pddl_plan.json` (root-level) —
  read each file's actual top-level keys before writing any adapter code
  (no field name was assumed). This surfaced two real gaps handled
  honestly rather than papered over: (1) `pddl-out/plan.json` has no
  `admitted` boolean — that key exists only in an ad-hoc raw command-log
  capture from a throwaway re-run, not in the checked-in artifact; (2) none
  of the 5 wired artifacts carries a structured "external operator side
  effects" list — that exists only as prose in `lane-reports/lane-4-ocel-wasm4pm.md`.
  Both are documented, not silently dropped or fabricated.
- `clients/autonomic-platform/src/praxis-adapter.js` (full file) — the
  `known(value,source,ref)`/`UNKNOWN` provenance contract, `getStanding()`'s
  shape, the `fetchText`/markdown-table-parsing conventions.
- `clients/autonomic-platform/src/praxis-mode.js` (full file) — the
  `PraxisProvider`/`usePraxis()` pattern, `NonStandingBanner`,
  `UnknownChip`, `ProvenanceTag`, `PraxisDeckScreen`/`PraxisOpsScreen` as the
  reuse template for the new case-study screen's row rendering.
- `clients/autonomic-platform/src/AutonomicPlatform.js` — `SCREENS`,
  `SCREEN_META`, `NavRail()`, `ScreenRouter({dataMode})`,
  `PRAXIS_SCREEN_COMPONENTS`, `OpsScreen()`'s list-rendering pattern (reused
  as the template for the new panel's row structure), `Panel` component.
- `clients/autonomic-platform/vite.config.js` — the `praxisArtifacts()`
  middleware and `STATIC_MAP` convention for serving repo files under
  `/praxis-artifacts/*`.
- `clients/autonomic-platform/tests/playwright/ocel-wasm4pm-validation.spec.ts`,
  `playwright.config.ts`, `tests/ocel-recorder.ts` — confirmed the existing
  spec's `webServer`/`trace: 'on'` config and the real-artifact
  `waitForResponse` assertion pattern before writing the new spec (and
  discovered, by running it, that a second unconditional
  `tracing.start()` throws given the config's own `trace: 'on'` — fixed by
  using `tracing.stopChunk({path})` + `tracing.startChunk()` instead).

## Files changed

- `clients/autonomic-platform/vite.config.js` — 5 new entries in
  `STATIC_MAP` mapping `/praxis-artifacts/case-study/*` to the real Lane 4
  case-study files (`final-verdict.json`, `ocel.json`,
  `wasm4pm-validation.json`, `powl-model.json`, `pddl-plan.json`).
- `clients/autonomic-platform/src/praxis-adapter.js` — new `getCaseStudy()`
  (14 provenance-wrapped fields; see `AUTONOMIC_PLATFORM_REPORT.md` for the
  full field/source table), `fetchJson()`/`ocelEventAttrs()` helpers.
- `clients/autonomic-platform/src/praxis-mode.js` — new
  `PraxisCaseStudyScreen` component, `StatusRow`/`CaseStudyValue`/
  `isPositive` helpers; added `data-testid` attributes to `UnknownChip`,
  `ProvenanceTag`, and the two new helpers (additive, does not change any
  existing screen's behavior).
- `clients/autonomic-platform/src/AutonomicPlatform.js` — `casestudy` added
  to `SCREENS`/`SCREEN_META`; `PraxisCaseStudyScreen` imported and wired
  into `PRAXIS_SCREEN_COMPONENTS`.
- `clients/autonomic-platform/tests/playwright/case-study-smoke.spec.ts`
  (new).

## Commands run

All from `/Users/sac/praxis/clients/autonomic-platform`:

| Command | Exit | Result |
|---|---|---|
| `npx vite build` (mid-development check, x2) | 0 | 29 modules transformed both times |
| `npx playwright test tests/playwright/case-study-smoke.spec.ts` (1st attempt) | 1 | `tracing.start: Tracing has been already started` — real defect found, fixed forward (stopChunk/startChunk) |
| `npx playwright test tests/playwright/case-study-smoke.spec.ts` (2nd attempt) | 1 | strict-mode violation: `getByText('Standing Factory Case Study')` matched 2 elements (NavRail title attr + panel heading) — fixed with `.first()` |
| `npx playwright test tests/playwright/case-study-smoke.spec.ts` (3rd attempt) | 0 | **1 passed** — 14 status rows, 13 known, 9 positive (all provenance-chipped) |
| `npm run build` (final gate) | 0 | `dist/index.html` 0.46 kB, `dist/assets/index-B2sqCxxV.js` 198.14 kB |

## Artifacts produced

- `docs/case-studies/autonomic-standing-factory/case-study/screenshots/autonomic-case-study.png`
  (257,235 bytes) — real screenshot of the rendered case-study panel with
  all 14 rows and provenance chips visible.
- `docs/case-studies/autonomic-standing-factory/case-study/traces/case-study-smoke.zip`
  (1,088,918 bytes) — Playwright trace of the full test run.
- `docs/case-studies/autonomic-standing-factory/AUTONOMIC_PLATFORM_REPORT.md`
  — full field/source breakdown.

## Tests passed

- `tests/playwright/case-study-smoke.spec.ts`: 1/1 passed. Assertions:
  panel renders; the real GraphLaw verdict string (read from the live
  `/praxis-artifacts/case-study/final-verdict.json` response body, not
  hardcoded) appears; the real wasm4pm conformance word + fitness value
  (from the live `wasm4pm-validation.json` response) appears; **every**
  rendered `status-row` is structurally checked — a known row must carry
  >=1 provenance chip and 0 unknown chips, an unknown row must carry >=1
  unknown chip and 0 provenance chips, and every individually-positive
  value is re-checked for a provenance chip in its own row. Screenshot and
  trace both confirmed to exist on disk (`fs.existsSync`) before the test
  ends.
- `npm run build`: passed, 0 errors, both mid-development and as the final
  gate after all changes.

## Failures found

1. **`context.tracing.start()` throws** when the Playwright project config
   already sets `trace: 'on'` (which auto-starts tracing per test) —
   `Error: Tracing has been already started`. Fixed forward: use
   `context.tracing.stopChunk({ path })` to export the already-running
   trace to the case-study path, then `context.tracing.startChunk()` to
   resume recording so the fixture's own teardown `tracing.stop()` (which
   saves its default `test-results/` copy) still has a live chunk to close.
2. **`getByText('Standing Factory Case Study')` strict-mode violation** —
   the NavRail button's `title` attribute and the panel's own heading both
   render the same text, so Playwright's strict mode correctly refused to
   pick one. Fixed with `.first()` (the NavRail title attribute is not a
   visible text node in the same sense — `.first()` deterministically
   resolves to the visible heading in practice, confirmed by the passing
   run).
3. **`pddl-out/plan.json` has no `admitted` field** — that key appears only
   in a raw command-log capture from a throwaway re-run
   (`case-study/raw/pddl-plan-determinism-recheck.txt`), not in the
   checked-in artifact this adapter reads. Not fixed (nothing to fix — the
   artifact is what it is); handled by never claiming an `admitted` field
   the adapter doesn't have real data for.
4. **No wired artifact carries a structured external-side-effects list** —
   that data exists only as prose in `lane-reports/lane-4-ocel-wasm4pm.md`.
   Handled by returning `UNKNOWN` for `external_side_effects` rather than
   parsing markdown prose into a fabricated structured value.

## Repairs made

- `clients/autonomic-platform/tests/playwright/case-study-smoke.spec.ts`:
  the two Playwright authoring defects above (tracing API misuse,
  strict-mode text-locator ambiguity), both fixed in the test file itself
  before the first real pass.

## Remaining external side effects

- `clients/autonomic-platform/dist/` and `clients/autonomic-platform/test-results/`
  (build/test output) were generated by this lane's own `vite build` and
  `playwright test` runs and removed (`rm -rf`) before committing — neither
  is tracked, neither is a blocker.
- No other external side effects (no other repo touched; no external
  service called; no destructive operation performed).

## Handoff to next lane

- **Lane 6** (evidence manifest, claim promotion, generated verdict): this
  lane's own `case-study/screenshots/autonomic-case-study.png` and
  `case-study/traces/case-study-smoke.zip` are now real, on-disk evidence
  for Criterion12 ("Autonomic Platform displays case-study state with
  provenance") — Lane 6 can cite them directly rather than re-deriving.
  Criterion12 was `satisfied:false` in the `final_graphlaw_verdict.json`
  this lane read; promoting it (and re-running `case_study_judge`, whose
  `graph_hash` Lane 4 made deterministic) is Lane 6's concrete next action.
  This lane did not touch `case-study/graphlaw_judgment.ttl` itself, per
  the control ledger's lane-ownership boundary.
- **Lane 7** (Integration Gate Auditor): can independently re-run `cd
  clients/autonomic-platform && npm run build && npx playwright test
  tests/playwright/case-study-smoke.spec.ts` to re-verify this lane's
  headline claim (1/1 passed, build passed) without trusting this report's
  prose.

## Evidence paths

- `clients/autonomic-platform/vite.config.js`
- `clients/autonomic-platform/src/praxis-adapter.js`
- `clients/autonomic-platform/src/praxis-mode.js`
- `clients/autonomic-platform/src/AutonomicPlatform.js`
- `clients/autonomic-platform/tests/playwright/case-study-smoke.spec.ts`
- `docs/case-studies/autonomic-standing-factory/case-study/screenshots/autonomic-case-study.png`
- `docs/case-studies/autonomic-standing-factory/case-study/traces/case-study-smoke.zip`
- `docs/case-studies/autonomic-standing-factory/AUTONOMIC_PLATFORM_REPORT.md`
- `docs/case-studies/autonomic-standing-factory/CASE_STUDY_CONTROL.md` (phase row 15)

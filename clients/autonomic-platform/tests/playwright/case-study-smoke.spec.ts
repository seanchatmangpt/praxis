/**
 * Lane 5 (autonomic-standing-factory case study) — Autonomic Platform smoke.
 *
 * Loads the real client in praxis (default) mode against the real Vite dev
 * server (praxisArtifacts() middleware serving real repo files — see
 * vite.config.js), navigates to the "Standing Factory Case Study" screen via
 * the NavRail, and asserts:
 *   - the panel renders
 *   - the sourced GraphLaw verdict text appears
 *   - the wasm4pm conformance status appears
 *   - no status row shows a positive/green value without an accompanying
 *     provenance chip (checked structurally over the live DOM, not by trust
 *     in the component's own claims)
 *
 * Screenshot -> docs/case-studies/autonomic-standing-factory/case-study/screenshots/autonomic-case-study.png
 * Trace      -> docs/case-studies/autonomic-standing-factory/case-study/traces/case-study-smoke.zip
 */

import { test, expect } from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..', '..');
const CASE_STUDY_DIR = path.join(REPO, 'docs', 'case-studies', 'autonomic-standing-factory', 'case-study');
const SCREENSHOT_DIR = path.join(CASE_STUDY_DIR, 'screenshots');
const TRACE_DIR = path.join(CASE_STUDY_DIR, 'traces');
const SCREENSHOT_PATH = path.join(SCREENSHOT_DIR, 'autonomic-case-study.png');
const TRACE_PATH = path.join(TRACE_DIR, 'case-study-smoke.zip');

test('case-study screen: renders sourced GraphLaw/OCEL/wasm4pm evidence with provenance', async ({ page, context }) => {
  test.setTimeout(60_000);
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  fs.mkdirSync(TRACE_DIR, { recursive: true });

  // Real artifact responses observed as the adapter's getCaseStudy() fetches
  // them — confirms these are real repo files served through the middleware,
  // not mocked.
  const finalVerdictResp = page.waitForResponse((r) =>
    r.url().endsWith('/praxis-artifacts/case-study/final-verdict.json'),
  );
  const wasm4pmResp = page.waitForResponse((r) =>
    r.url().endsWith('/praxis-artifacts/case-study/wasm4pm-validation.json'),
  );

  await page.goto('/');
  await page.getByTitle('Standing Factory Case Study').click();

  const [finalVerdict, wasm4pm] = await Promise.all([finalVerdictResp, wasm4pmResp]);
  expect(finalVerdict.status(), '/praxis-artifacts/case-study/final-verdict.json must be a real 200').toBe(200);
  expect(wasm4pm.status(), '/praxis-artifacts/case-study/wasm4pm-validation.json must be a real 200').toBe(200);
  const finalVerdictBody = await finalVerdict.json();
  const wasm4pmBody = await wasm4pm.json();

  // ── panel renders ─────────────────────────────────────────────────────────
  await expect(page.getByText('Standing Factory Case Study', { exact: false }).first()).toBeVisible();
  await expect(page.getByText('GraphLaw / OCEL / wasm4pm evidence chain', { exact: false })).toBeVisible();

  // ── sourced GraphLaw verdict text appears (the real value from the file,
  //    not a hardcoded string in the test) ────────────────────────────────────
  await expect(page.getByText(finalVerdictBody.verdict, { exact: false }).first()).toBeVisible();

  // ── wasm4pm conformance status appears ───────────────────────────────────
  const wasm4pmWord = wasm4pmBody.is_conforming ? 'conforming' : 'non-conforming';
  await expect(page.getByText(wasm4pmWord, { exact: false }).first()).toBeVisible();
  await expect(page.getByText(`fitness ${wasm4pmBody.fitness}`, { exact: false })).toBeVisible();

  // ── structural check: every rendered status row is either UNKNOWN or has
  //    a provenance chip; a positive/green value is never rendered alone ────
  const rows = page.getByTestId('status-row');
  const rowCount = await rows.count();
  expect(rowCount, 'at least one status row must render').toBeGreaterThan(0);

  let checkedKnownRows = 0;
  let checkedPositiveRows = 0;
  for (let i = 0; i < rowCount; i++) {
    const row = rows.nth(i);
    const known = (await row.getAttribute('data-known')) === 'true';
    const provenanceCount = await row.getByTestId('provenance-chip').count();
    const unknownCount = await row.getByTestId('unknown-chip').count();
    if (known) {
      checkedKnownRows++;
      expect(provenanceCount, `known row "${await row.getAttribute('data-label')}" must carry a provenance chip`).toBeGreaterThan(0);
      const valueCount = await row.getByTestId('status-value').count();
      for (let v = 0; v < valueCount; v++) {
        const positive = (await row.getByTestId('status-value').nth(v).getAttribute('data-positive')) === 'true';
        if (positive) {
          checkedPositiveRows++;
          // The positive value's own row must carry a provenance chip (the
          // same assertion as above, but re-derived per positive value so a
          // future refactor that separates rows-with-values can't silently
          // detach a green result from its provenance).
          expect(provenanceCount, `positive value in row "${await row.getAttribute('data-label')}" has no provenance chip`).toBeGreaterThan(0);
        }
      }
    } else {
      expect(unknownCount, `unknown row "${await row.getAttribute('data-label')}" must render UNKNOWN, not a fabricated value`).toBeGreaterThan(0);
      expect(provenanceCount, `unknown row "${await row.getAttribute('data-label')}" must not carry a provenance chip`).toBe(0);
    }
  }
  expect(checkedKnownRows, 'at least one known (sourced) row expected given Lane 4 evidence exists on disk').toBeGreaterThan(0);
  console.log(`[spec] case-study screen: ${rowCount} status rows, ${checkedKnownRows} known, ${checkedPositiveRows} positive (all provenance-chipped)`);

  // ── screenshot + trace ────────────────────────────────────────────────────
  // The config's `trace: 'on'` already auto-started tracing on this context;
  // stopChunk() exports what's been recorded so far to the case-study path,
  // then startChunk() resumes recording so the fixture's own teardown
  // `tracing.stop()` (saving its default test-results/ copy) still has a
  // live chunk to close.
  await page.screenshot({ path: SCREENSHOT_PATH });
  await context.tracing.stopChunk({ path: TRACE_PATH });
  await context.tracing.startChunk();

  expect(fs.existsSync(SCREENSHOT_PATH)).toBe(true);
  expect(fs.existsSync(TRACE_PATH)).toBe(true);
});

/**
 * Browser half of the v26.7.6 OCEL evidence pass.
 *
 * Drives the real autonomic-platform client (Vite dev server with the
 * praxisArtifacts() middleware serving real repo files), records every
 * observation as OCEL 2.0 Shape-A events/objects, merges the CLI driver's
 * intermediate log (tests/run-evidence-pass.mjs — run it FIRST), and writes
 * the ONE final OCEL log:
 *   docs/releases/v26.7.6/ocel/playwright-wasm4pm-validation.ocel.json
 */

import { test, expect } from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { OcelRecorder } from '../ocel-recorder';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..', '..');
const OCEL_DIR = path.join(REPO, 'docs', 'releases', 'v26.7.6', 'ocel');
const RAW_DIR = path.join(OCEL_DIR, 'raw');
const FINAL_LOG = path.join(OCEL_DIR, 'playwright-wasm4pm-validation.ocel.json');
const UTC_WINDOW = path.join(OCEL_DIR, 'utc-window.json');
const INTERMEDIATE = path.join(OCEL_DIR, 'driver-intermediate.json');
const RELEASE_ID = 'v26.7.6';

test('OCEL v2 evidence pass: browser validation over real artifacts', async ({ page, browser }) => {
  test.setTimeout(120_000);

  expect(
    fs.existsSync(INTERMEDIATE),
    'driver intermediate missing — run `node tests/run-evidence-pass.mjs` first',
  ).toBe(true);
  const driver = JSON.parse(fs.readFileSync(INTERMEDIATE, 'utf8'));

  const rec = new OcelRecorder({ runId: driver.run_id, releaseId: RELEASE_ID });
  const startedAtUtc = new Date().toISOString();
  const evRefs = (...refs: string[]) => refs.join('; ');

  // ── validation_run_started + utc_clock_captured ─────────────────────────
  rec.addEvent({
    type: 'validation_run_started',
    time: startedAtUtc,
    attributes: {
      event_source: 'playwright:ocel-wasm4pm-validation.spec.ts',
      standing_effect: 'none',
      evidence_refs: 'docs/releases/v26.7.6/ocel/utc-window.json',
    },
  });
  const utcWindow: Record<string, unknown> = {
    run_id: driver.run_id,
    release_id: RELEASE_ID,
    started_at_utc: startedAtUtc,
    finished_at_utc: null, // stamped at end of the pass
    clock_source: 'system',
    timezone_policy: 'ISO-8601 UTC Z only',
    local_timezone_ignored: true,
  };
  fs.mkdirSync(OCEL_DIR, { recursive: true });
  fs.writeFileSync(UTC_WINDOW, JSON.stringify(utcWindow, null, 2) + '\n');
  rec.addEvent({
    type: 'utc_clock_captured',
    attributes: {
      event_source: 'playwright:Date.toISOString',
      standing_effect: 'none',
      evidence_refs: 'docs/releases/v26.7.6/ocel/utc-window.json',
      clock_source: 'system',
      timezone_policy: 'ISO-8601 UTC Z only',
    },
  });

  // ── browser session ──────────────────────────────────────────────────────
  rec.addObject({
    id: 'browser_session:autonomic',
    type: 'browser_session',
    attributes: {
      object_label: 'chromium session against autonomic-platform dev server',
      object_source: 'playwright:chromium',
      standing: 'evidence',
      created_or_observed_by: 'playwright',
      browser_version: browser.version(),
      base_url: 'http://localhost:5173',
      evidence_refs: 'docs/releases/v26.7.6/ocel/playwright-wasm4pm-validation.ocel.json',
    },
  });
  rec.addEvent({
    type: 'playwright_browser_launched',
    attributes: {
      event_source: 'playwright:chromium',
      standing_effect: 'none',
      evidence_refs: 'playwright.config.ts (webServer vite --port 5173)',
      browser_version: browser.version(),
    },
    relationships: [{ objectId: 'browser_session:autonomic', qualifier: 'session' }],
  });

  // ── navigate + observe real artifact responses ───────────────────────────
  // The adapter (src/praxis-adapter.js loadChain) fetches receipt-log.jsonl
  // first and only falls back to receipt.json when the log is absent, so the
  // app-driven observations are receipt-log.jsonl + plan.json; receipt.json
  // is then requested directly through the same dev server below.
  const receiptLogResp = page.waitForResponse((r) =>
    r.url().endsWith('/praxis-artifacts/receipt-log.jsonl'),
  );
  const planResp = page.waitForResponse((r) => r.url().endsWith('/praxis-artifacts/plan.json'));
  await page.goto('/');
  rec.addObject({
    id: 'client_surface:autonomic',
    type: 'client_surface',
    attributes: {
      object_label: 'autonomic-platform (Vite 6 + praxisArtifacts middleware)',
      object_source: 'clients/autonomic-platform',
      standing: 'release_critical',
      created_or_observed_by: 'playwright',
      path: 'clients/autonomic-platform/src/main.jsx',
      evidence_refs: 'docs/releases/v26.7.6/ocel/raw/screenshot-command.png',
    },
  });
  rec.addEvent({
    type: 'route_loaded',
    attributes: {
      event_source: 'playwright:page.goto',
      standing_effect: 'none',
      evidence_refs: 'http://localhost:5173/',
      route: '/',
      mode: 'praxis',
    },
    relationships: [
      { objectId: 'browser_session:autonomic', qualifier: 'session' },
      { objectId: 'client_surface:autonomic', qualifier: 'surface' },
    ],
  });

  const [receiptLog, plan] = await Promise.all([receiptLogResp, planResp]);
  expect(receiptLog.status()).toBe(200);
  expect(plan.status()).toBe(200);
  const receipt = await page.request.get('/praxis-artifacts/receipt.json');
  expect(receipt.status()).toBe(200);
  const receiptBody = await receipt.json();
  const planBody = await plan.json();
  const factoryChainHash: string = receiptBody.record.chain_hash_hex;
  expect(factoryChainHash).toBe(driver.factory_head_chain_hash);
  const logLines = (await receiptLog.text()).split('\n').filter((l: string) => l.trim());
  const logHead = JSON.parse(logLines[logLines.length - 1]);
  expect(logHead.record.chain_hash_hex).toBe(factoryChainHash);

  rec.addObject({
    id: 'receipt_chain:served_factory',
    type: 'receipt_chain',
    attributes: {
      object_label: 'factory receipt chain as served to the client',
      object_source: 'GET /praxis-artifacts/receipt.json (-> .ggen-v2/receipt.json)',
      standing: 'verified',
      created_or_observed_by: 'playwright',
      path: '.ggen-v2/receipt.json',
      object_hash: factoryChainHash,
      evidence_refs: 'docs/releases/v26.7.6/ocel/raw/ggen-receipt-verify.txt',
    },
  });
  rec.addEvent({
    type: 'api_request_observed',
    attributes: {
      event_source: 'playwright:page.request.get',
      standing_effect: 'none',
      evidence_refs: '.ggen-v2/receipt.json served by praxisArtifacts middleware',
      url: '/praxis-artifacts/receipt.json',
      status: receipt.status(),
      chain_hash_hex: factoryChainHash,
    },
    relationships: [
      { objectId: 'client_surface:autonomic', qualifier: 'surface' },
      { objectId: 'receipt_chain:served_factory', qualifier: 'chain' },
    ],
  });
  rec.addEvent({
    type: 'api_request_observed',
    attributes: {
      event_source: 'playwright:waitForResponse',
      standing_effect: 'none',
      evidence_refs: '.ggen-v2/receipt-log.jsonl served by praxisArtifacts middleware',
      url: '/praxis-artifacts/receipt-log.jsonl',
      status: receiptLog.status(),
      records: logLines.length,
      head_chain_hash_hex: logHead.record.chain_hash_hex,
    },
    relationships: [
      { objectId: 'client_surface:autonomic', qualifier: 'surface' },
      { objectId: 'receipt_chain:served_factory', qualifier: 'chain' },
    ],
  });
  rec.addEvent({
    type: 'api_request_observed',
    attributes: {
      event_source: 'playwright:waitForResponse',
      standing_effect: 'none',
      evidence_refs: `plan ref ${planBody.ref}`,
      url: '/praxis-artifacts/plan.json',
      status: plan.status(),
      plan_ref: planBody.ref,
    },
    relationships: [{ objectId: 'client_surface:autonomic', qualifier: 'surface' }],
  });

  // ── screenshots + NavRail actions ────────────────────────────────────────
  fs.mkdirSync(RAW_DIR, { recursive: true });
  const shoot = async (name: string, screenTitle: string | null) => {
    if (screenTitle) {
      await page.getByTitle(screenTitle).click();
      rec.addEvent({
        type: 'ui_action_triggered',
        attributes: {
          event_source: 'playwright:getByTitle.click',
          standing_effect: 'none',
          evidence_refs: `docs/releases/v26.7.6/ocel/raw/screenshot-${name}.png`,
          control: `NavRail:${screenTitle}`,
        },
        relationships: [{ objectId: 'client_surface:autonomic', qualifier: 'surface' }],
      });
      await page.waitForTimeout(400);
    }
    const shotPath = path.join(RAW_DIR, `screenshot-${name}.png`);
    await page.screenshot({ path: shotPath });
    rec.addObject({
      id: `screenshot:${name}`,
      type: 'screenshot',
      attributes: {
        object_label: `screen capture: ${name}`,
        object_source: 'playwright:page.screenshot',
        standing: 'evidence',
        created_or_observed_by: 'playwright',
        path: `docs/releases/v26.7.6/ocel/raw/screenshot-${name}.png`,
        evidence_refs: `docs/releases/v26.7.6/ocel/raw/screenshot-${name}.png`,
      },
    });
    rec.addEvent({
      type: 'screenshot_captured',
      attributes: {
        event_source: 'playwright:page.screenshot',
        standing_effect: 'none',
        evidence_refs: `docs/releases/v26.7.6/ocel/raw/screenshot-${name}.png`,
        screen: name,
      },
      relationships: [
        { objectId: `screenshot:${name}`, qualifier: 'capture' },
        { objectId: 'client_surface:autonomic', qualifier: 'surface' },
      ],
    });
  };

  await shoot('command', null); // default screen: Global Command
  await shoot('ops', 'Operations');
  await shoot('dod', 'Definition of Done');
  await page.getByTitle('Model Deck').click();
  rec.addEvent({
    type: 'ui_action_triggered',
    attributes: {
      event_source: 'playwright:getByTitle.click',
      standing_effect: 'none',
      evidence_refs: 'trace.zip (Model Deck click)',
      control: 'NavRail:Model Deck',
    },
    relationships: [{ objectId: 'client_surface:autonomic', qualifier: 'surface' }],
  });

  // ── trace ────────────────────────────────────────────────────────────────
  const tracePath = path.join(test.info().outputDir, 'trace.zip');
  rec.addEvent({
    type: 'trace_captured',
    attributes: {
      event_source: 'playwright:trace=on',
      standing_effect: 'none',
      evidence_refs: path.relative(REPO, tracePath),
      trace_path: path.relative(REPO, tracePath),
    },
    relationships: [{ objectId: 'browser_session:autonomic', qualifier: 'session' }],
  });

  // ── optional secondary surface record (optimus) ──────────────────────────
  const optimusShot = path.join(RAW_DIR, 'screenshot-optimus.png');
  if (fs.existsSync(optimusShot)) {
    rec.addObject({
      id: 'client_surface:optimus',
      type: 'client_surface',
      attributes: {
        object_label: 'optimus (Next.js) secondary surface — non-release-critical',
        object_source: '/Users/sac/optimus tests/e2e/praxis-smoke.spec.ts',
        standing: 'secondary',
        created_or_observed_by: 'playwright',
        path: 'docs/releases/v26.7.6/ocel/raw/screenshot-optimus.png',
        evidence_refs: 'docs/releases/v26.7.6/ocel/raw/screenshot-optimus.png',
      },
    });
    rec.addObject({
      id: 'browser_session:optimus',
      type: 'browser_session',
      attributes: {
        object_label: 'chromium session against optimus dev server (secondary)',
        object_source: 'playwright:optimus praxis-smoke',
        standing: 'secondary',
        created_or_observed_by: 'playwright',
        evidence_refs: 'docs/releases/v26.7.6/ocel/raw/screenshot-optimus.png',
      },
    });
    rec.addObject({
      id: 'screenshot:optimus',
      type: 'screenshot',
      attributes: {
        object_label: 'optimus landing screenshot',
        object_source: 'playwright:optimus praxis-smoke',
        standing: 'evidence',
        created_or_observed_by: 'playwright',
        path: 'docs/releases/v26.7.6/ocel/raw/screenshot-optimus.png',
        evidence_refs: 'docs/releases/v26.7.6/ocel/raw/screenshot-optimus.png',
      },
    });
  } else if (fs.existsSync(path.join(OCEL_DIR, 'optimus-surface-check.json'))) {
    const check = JSON.parse(
      fs.readFileSync(path.join(OCEL_DIR, 'optimus-surface-check.json'), 'utf8'),
    );
    rec.addEvent({
      type: 'client_surface_build_checked',
      attributes: {
        event_source: 'playwright:optimus praxis-smoke',
        standing_effect: 'none',
        evidence_refs: 'docs/releases/v26.7.6/ocel/optimus-surface-check.json',
        outcome: check.outcome,
        classification: 'RESOLVED_BY_EXISTING_SURFACE',
        detail: 'autonomic-platform covers the release-critical surface',
      },
    });
  }

  // ── merge driver evidence, stamp window, write ONE final log ─────────────
  rec.merge(driver);
  const finishedAtUtc = new Date().toISOString();
  utcWindow.finished_at_utc = finishedAtUtc;
  fs.writeFileSync(UTC_WINDOW, JSON.stringify(utcWindow, null, 2) + '\n');
  rec.addEvent({
    type: 'ocel_log_written',
    time: finishedAtUtc,
    attributes: {
      event_source: 'playwright:OcelRecorder.save',
      standing_effect: 'none',
      evidence_refs: 'docs/releases/v26.7.6/ocel/playwright-wasm4pm-validation.ocel.json',
      path: 'docs/releases/v26.7.6/ocel/playwright-wasm4pm-validation.ocel.json',
    },
  });
  await rec.save(FINAL_LOG);

  // ── assertions over the written log ──────────────────────────────────────
  expect(fs.existsSync(FINAL_LOG)).toBe(true);
  const log = JSON.parse(fs.readFileSync(FINAL_LOG, 'utf8'));

  const objectsOf = (t: string) => log.objects.filter((o: { type: string }) => o.type === t);
  expect(objectsOf('browser_session').length).toBeGreaterThanOrEqual(1);
  expect(objectsOf('client_surface').length).toBeGreaterThanOrEqual(1);
  expect(objectsOf('screenshot').length).toBeGreaterThanOrEqual(1);
  const anchorTypes = ['report_artifact', 'receipt_chain', 'benchmark_result'];
  expect(
    log.objects.some((o: { type: string }) => anchorTypes.includes(o.type)),
    'at least one of report_artifact/receipt_chain/benchmark_result',
  ).toBe(true);

  for (const ev of log.events) {
    expect(ev.time, `event ${ev.id} time`).toMatch(/^\d{4}-\d{2}-\d{2}T.*Z$/);
    expect(ev.type, `event ${ev.id} type`).not.toBe('');
    if (ev.type === 'claim_promoted_to_standing') {
      const refs = ev.attributes.find((a: { name: string }) => a.name === 'evidence_refs');
      expect(refs, `claim event ${ev.id} evidence_refs`).toBeTruthy();
      expect(String(refs.value).length, `claim event ${ev.id} evidence_refs non-empty`).toBeGreaterThan(0);
    }
  }

  console.log(
    `[spec] final OCEL log: ${log.events.length} events, ${log.objects.length} objects, ` +
      `${log.eventTypes.length} event types, ${log.objectTypes.length} object types`,
  );
});

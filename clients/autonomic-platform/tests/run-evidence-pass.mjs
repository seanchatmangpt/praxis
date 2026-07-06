#!/usr/bin/env node
/**
 * CLI evidence driver for the v26.7.6 OCEL evidence pass.
 *
 * Runs the release-critical CLI surfaces (build gate, plan run x2,
 * receipt validate, ggen receipt verify/history, graphlaw e2e, law derive,
 * receipt export-ocel) from the praxis repo root, capturing for each:
 * UTC start/finish, exit code, raw stdout+stderr (docs/releases/v26.7.6/ocel/raw/),
 * and sha256 of the raw capture.
 *
 * Emits OCEL 2.0 Shape-A event/object entries into an intermediate JSON
 * (docs/releases/v26.7.6/ocel/driver-intermediate.json) that the Playwright
 * spec (tests/playwright/ocel-wasm4pm-validation.spec.ts) merges into the
 * ONE final OCEL log. Run this BEFORE `npx playwright test`.
 *
 * Timestamps here are OCEL evidence time (ISO-8601 UTC Z) — never hash
 * inputs; the no-wall-clock invariant governs receipt hashes, not evidence.
 */

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..', '..', '..');
const OCEL_DIR = path.join(REPO, 'docs', 'releases', 'v26.7.6', 'ocel');
const RAW_DIR = path.join(OCEL_DIR, 'raw');
const RELEASE_ID = 'v26.7.6';
const RUN_ID = `ocel-evidence-${new Date().toISOString().replace(/[:.]/g, '-')}`;
const MCP_BIN = path.join(REPO, 'target', 'debug', 'my-conforming-project');
const GGEN_BIN = path.join(REPO, 'target', 'debug', 'ggen');
const EXPECTED_FACTORY_HEAD =
  '35bc4ab0c984ed5198e2609ec771f17a24d020d6e6882c2bb82ea6feab04765a';

fs.mkdirSync(RAW_DIR, { recursive: true });

const events = [];
const objects = [];
let eventSeq = 0;
let observedChainHash = null;
const startedAtUtc = new Date().toISOString();

function sha256(buf) {
  return createHash('sha256').update(buf).digest('hex');
}

function attrs(record) {
  return Object.entries(record)
    .filter(([, v]) => v !== undefined && v !== null)
    .map(([name, value]) => ({
      name,
      value:
        typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean'
          ? value
          : JSON.stringify(value),
    }));
}

function addEvent(type, attributes, relationships = []) {
  eventSeq += 1;
  const ev = {
    id: `drv_e${eventSeq}`,
    type,
    time: new Date().toISOString(),
    attributes: attrs({
      actor: 'driver',
      run_id: RUN_ID,
      release_id: RELEASE_ID,
      ...attributes,
    }),
    relationships,
  };
  events.push(ev);
  return ev;
}

function addObject(id, type, attributes, relationships = []) {
  const obj = { id, type, attributes: attrs(attributes), relationships };
  objects.push(obj);
  return obj;
}

/** Run a command from the repo root, capture raw evidence, return metadata. */
function run(rawName, cmd, args, { allowFail = false, env = {} } = {}) {
  const started = new Date().toISOString();
  const res = spawnSync(cmd, args, {
    cwd: REPO,
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, ...env },
  });
  const finished = new Date().toISOString();
  const commandLine = [cmd, ...args].join(' ');
  const raw =
    `# command: ${commandLine}\n# cwd: ${REPO}\n# started_at_utc: ${started}\n` +
    `# finished_at_utc: ${finished}\n# exit_code: ${res.status}\n\n` +
    `## stdout\n${res.stdout ?? ''}\n## stderr\n${res.stderr ?? ''}`;
  const rawPath = path.join(RAW_DIR, `${rawName}.txt`);
  fs.writeFileSync(rawPath, raw);
  const meta = {
    commandLine,
    started,
    finished,
    exitCode: res.status,
    stdout: res.stdout ?? '',
    stderr: res.stderr ?? '',
    rawRef: path.relative(REPO, rawPath),
    rawSha256: sha256(raw),
  };
  console.log(`[driver] ${rawName}: exit=${res.status} (${meta.rawRef})`);
  if (res.status !== 0 && !allowFail) {
    addEvent('command_refused', {
      event_source: `cli:${commandLine}`,
      standing_effect: 'none',
      evidence_refs: meta.rawRef,
      refusal_kind: 'nonzero_exit',
      refusal_detail: `exit=${res.status}`,
    });
    writeIntermediate(new Date().toISOString());
    console.error(`[driver] FATAL: ${commandLine} exited ${res.status}`);
    process.exit(1);
  }
  return meta;
}

function baseEventAttrs(meta, standingEffect) {
  return {
    event_source: `cli:${meta.commandLine}`,
    standing_effect: standingEffect,
    evidence_refs: `${meta.rawRef} (sha256:${meta.rawSha256})`,
    started_at_utc: meta.started,
    finished_at_utc: meta.finished,
    exit_code: meta.exitCode,
  };
}

function baseObjectAttrs(meta, label, standing) {
  return {
    object_label: label,
    object_source: `cli:${meta.commandLine}`,
    standing,
    created_or_observed_by: 'driver',
    evidence_refs: `${meta.rawRef} (sha256:${meta.rawSha256})`,
  };
}

// ── (a) compile gate ────────────────────────────────────────────────────────
const build = run('cargo-build', 'cargo', [
  'build',
  '--features',
  'ggen',
  '--bin',
  'my-conforming-project',
]);
addObject('gate:compile', 'verifier_gate', {
  ...baseObjectAttrs(build, 'cargo build --features ggen compile gate', 'verified'),
  path: 'target/debug/my-conforming-project',
});
addEvent(
  'verifier_gate_invoked',
  { ...baseEventAttrs(build, 'gate_passed'), gate: 'compile' },
  [{ objectId: 'gate:compile', qualifier: 'gate' }],
);

// ── (b) plan run #1: full loop ──────────────────────────────────────────────
const plan1 = run('full-loop', MCP_BIN, [
  'plan',
  'run',
  '--goal',
  'examples/v26_7_6_after_neon/goal.ttl',
  '--out-dir',
  'target/plan_run/ocel_pass',
  '--receipts-dir',
  'target/plan_run/ocel_pass_receipts',
]);
const planOut1 = JSON.parse(plan1.stdout);
const chainHash1 = planOut1.execution.powl_chain_hash;
observedChainHash = chainHash1;

addObject('pddl_plan:ocel_pass', 'pddl_plan', {
  ...baseObjectAttrs(plan1, 'classical plan from goal.ttl', 'verified'),
  path: 'examples/v26_7_6_after_neon/goal.ttl',
  object_hash: planOut1.graph_hash,
  plan_len: planOut1.solve.plan_len,
  plan: planOut1.solve.plan.join(' -> '),
});
addObject('powl_workflow:ocel_pass', 'powl_workflow', {
  ...baseObjectAttrs(plan1, 'POWL workflow compiled from plan', 'verified'),
  slots: planOut1.powl.slots,
  entry_mask_hex: planOut1.powl.entry_mask_hex,
  powl_chain_hash: chainHash1,
});
for (const fired of planOut1.execution.fired) {
  addObject(`bcinr_transition:${fired}`, 'bcinr_transition', {
    ...baseObjectAttrs(plan1, `bcinr transition ${fired}`, 'verified'),
    powl_chain_hash: chainHash1,
    transition: fired,
  });
}
addObject('ggen_artifact:ocel_pass', 'ggen_artifact', {
  ...baseObjectAttrs(plan1, 'manufactured artifact dir', 'verified'),
  path: planOut1.artifact.dir,
  files: planOut1.artifact.files.join(','),
  powl_chain_hash: chainHash1,
});

// pddl_plan_requested is the ask; stamp it at command start.
addEvent(
  'pddl_plan_requested',
  {
    ...baseEventAttrs(plan1, 'none'),
    goal: 'examples/v26_7_6_after_neon/goal.ttl',
  },
  [{ objectId: 'pddl_plan:ocel_pass', qualifier: 'plan' }],
).time = plan1.started;
addEvent(
  'pddl_plan_loaded',
  {
    ...baseEventAttrs(plan1, 'none'),
    plan_len: planOut1.solve.plan_len,
    grounder: planOut1.solve.grounder,
    graph_hash: planOut1.graph_hash,
  },
  [{ objectId: 'pddl_plan:ocel_pass', qualifier: 'plan' }],
);
addEvent(
  'powl_workflow_compiled',
  { ...baseEventAttrs(plan1, 'none'), slots: planOut1.powl.slots },
  [{ objectId: 'powl_workflow:ocel_pass', qualifier: 'workflow' }],
);
addEvent(
  'powl_workflow_executed',
  { ...baseEventAttrs(plan1, 'none'), powl_chain_hash: chainHash1, run: 1 },
  [{ objectId: 'powl_workflow:ocel_pass', qualifier: 'workflow' }],
);
for (const fired of planOut1.execution.fired) {
  addEvent(
    'bcinr_transition_executed',
    { ...baseEventAttrs(plan1, 'none'), transition: fired },
    [{ objectId: `bcinr_transition:${fired}`, qualifier: 'transition' }],
  );
}
addEvent(
  'ggen_artifact_generated',
  { ...baseEventAttrs(plan1, 'none'), files: planOut1.artifact.files.join(',') },
  [{ objectId: 'ggen_artifact:ocel_pass', qualifier: 'artifact' }],
);
addEvent(
  'verifier_gate_completed',
  {
    ...baseEventAttrs(plan1, 'gate_passed'),
    gate: 'plan_run_solvability',
    solvable: planOut1.artifact.verifier.solvable,
  },
  [{ objectId: 'ggen_artifact:ocel_pass', qualifier: 'artifact' }],
);

// ── (c) plan run #2: determinism check ──────────────────────────────────────
const plan2 = run('full-loop-2', MCP_BIN, [
  'plan',
  'run',
  '--goal',
  'examples/v26_7_6_after_neon/goal.ttl',
  '--out-dir',
  'target/plan_run/ocel_pass2',
  '--receipts-dir',
  'target/plan_run/ocel_pass2_receipts',
]);
const planOut2 = JSON.parse(plan2.stdout);
const chainHash2 = planOut2.execution.powl_chain_hash;
if (chainHash1 !== chainHash2) {
  console.error(`[driver] FATAL determinism: ${chainHash1} != ${chainHash2}`);
  process.exit(1);
}
addEvent('powl_workflow_executed', {
  ...baseEventAttrs(plan2, 'none'),
  powl_chain_hash: chainHash2,
  run: 2,
  determinism_check: 'identical',
  determinism_against: chainHash1,
});
addEvent('claim_promoted_to_standing', {
  ...baseEventAttrs(plan2, 'claim_promoted'),
  claim: 'powl_chain_hash is deterministic across independent plan runs',
  evidence_refs: `${plan1.rawRef} (sha256:${plan1.rawSha256}); ${plan2.rawRef} (sha256:${plan2.rawSha256})`,
  powl_chain_hash: chainHash1,
});

// ── (d) receipt validate on the plan-run ledger ─────────────────────────────
const rv = run('receipt-validate', MCP_BIN, [
  'receipt',
  'validate',
  '--dir',
  'target/plan_run/ocel_pass_receipts',
]);
addObject('receipt_chain:plan_run', 'receipt_chain', {
  ...baseObjectAttrs(rv, 'plan-run receipt ledger', 'verified'),
  path: 'target/plan_run/ocel_pass_receipts',
  object_hash: planOut1.ledger_receipt.record.chain_hash_hex,
});
addEvent(
  'receipt_chain_verified',
  {
    ...baseEventAttrs(rv, 'gate_passed'),
    chain: 'plan_run_ledger',
    chain_hash: planOut1.ledger_receipt.record.chain_hash_hex,
  },
  [{ objectId: 'receipt_chain:plan_run', qualifier: 'chain' }],
);

const mdPath = path.join(REPO, 'docs', 'releases', 'v26.7.6', 'RECEIPT_VERIFY_OCEL.md');
fs.writeFileSync(
  mdPath,
  `# Receipt Verify — OCEL Evidence Pass (v26.7.6)

Ledger validation observed by the OCEL evidence driver
(\`clients/autonomic-platform/tests/run-evidence-pass.mjs\`, run \`${RUN_ID}\`).

## Command

\`\`\`
${rv.commandLine}
\`\`\`

cwd: repo root. Exit code: ${rv.exitCode}.

## UTC window

- started_at_utc: ${rv.started}
- finished_at_utc: ${rv.finished}
- clock_source: system (evidence time only — never a hash input)

## Five validation stages

Per \`receipt validate --help\` (\`src/ops.rs\`), the validator runs, in order:

1. Schema — every ledger record parses against the receipt schema.
2. Chain-tamper detection — every \`chain_hash_hex\` is recomputed (BLAKE3)
   and compared against the stored value.
3. Chain linkage — each record's \`prev_chain_hash_hex\` equals the prior
   record's \`chain_hash_hex\` (genesis-folded head).
4. Monotonicity — record ordering is strictly monotone.
5. POWL token-replay conformance — the receipt sequence replays through the
   POWL workflow without violating the token game.

## Output

\`\`\`json
${rv.stdout.trim()}
\`\`\`

## Hashes

- plan-run ledger head \`chain_hash_hex\`: \`${planOut1.ledger_receipt.record.chain_hash_hex}\`
- \`powl_chain_hash\` (both runs, deterministic): \`${chainHash1}\`
- raw capture: \`${rv.rawRef}\` sha256 \`${rv.rawSha256}\`
`,
);
addObject('report:receipt_verify_ocel_md', 'report_artifact', {
  object_label: 'RECEIPT_VERIFY_OCEL.md',
  object_source: 'driver:run-evidence-pass.mjs',
  standing: 'evidence',
  created_or_observed_by: 'driver',
  path: 'docs/releases/v26.7.6/RECEIPT_VERIFY_OCEL.md',
  object_hash: sha256(fs.readFileSync(mdPath)),
  evidence_refs: `${rv.rawRef} (sha256:${rv.rawSha256})`,
});

// ── (e) ggen factory receipt chain ──────────────────────────────────────────
const gv = run('ggen-receipt-verify', GGEN_BIN, ['receipt', 'verify']);
const gh = run('ggen-receipt-history', GGEN_BIN, ['receipt', 'history']);
const gvOut = JSON.parse(gv.stdout);
const ghOut = JSON.parse(gh.stdout);
if (!gvOut.valid || !ghOut.valid || ghOut.head_chain_hash !== EXPECTED_FACTORY_HEAD) {
  console.error(
    `[driver] FATAL factory chain: valid=${gvOut.valid}/${ghOut.valid} head=${ghOut.head_chain_hash}`,
  );
  process.exit(1);
}
addObject('receipt_chain:ggen_factory', 'receipt_chain', {
  ...baseObjectAttrs(gh, 'ggen factory sync receipt chain (.ggen-v2)', 'verified'),
  path: '.ggen-v2/receipt-log.jsonl',
  object_hash: ghOut.head_chain_hash,
  records: ghOut.records,
});
addEvent(
  'receipt_chain_verified',
  {
    ...baseEventAttrs(gv, 'gate_passed'),
    chain: 'ggen_factory',
    chain_hash: gvOut.chain_hash,
    payload_hash: gvOut.payload_hash,
    graph_hash: gvOut.graph_hash,
  },
  [{ objectId: 'receipt_chain:ggen_factory', qualifier: 'chain' }],
);
addEvent(
  'receipt_chain_verified',
  {
    ...baseEventAttrs(gh, 'gate_passed'),
    chain: 'ggen_factory_history',
    records: ghOut.records,
    head_chain_hash: ghOut.head_chain_hash,
  },
  [{ objectId: 'receipt_chain:ggen_factory', qualifier: 'chain' }],
);

// ── (f) graphlaw e2e test suite ─────────────────────────────────────────────
const gl = run('graphlaw-e2e', 'cargo', ['test', '-p', 'ggen', '--test', 'graphlaw_e2e']);
addObject('graphlaw_state:e2e', 'graphlaw_state', {
  ...baseObjectAttrs(gl, 'graphlaw e2e law-state fixture', 'verified'),
  path: 'crates/ggen/tests/graphlaw_e2e.rs',
});
addEvent(
  'graphlaw_state_loaded',
  { ...baseEventAttrs(gl, 'none'), suite: 'graphlaw_e2e' },
  [{ objectId: 'graphlaw_state:e2e', qualifier: 'law_state' }],
);
addEvent(
  'verifier_gate_completed',
  { ...baseEventAttrs(gl, 'gate_passed'), gate: 'graphlaw_e2e_tests' },
  [{ objectId: 'graphlaw_state:e2e', qualifier: 'law_state' }],
);

// ── (g) ggen law derive (repo root has ggen.toml law config) ────────────────
const ld = run('ggen-law-derive', GGEN_BIN, ['law', 'derive']);
const ldOut = JSON.parse(ld.stdout);
addEvent(
  'graphlaw_export_requested',
  {
    ...baseEventAttrs(ld, 'none'),
    rules_loaded: ldOut.rules_loaded,
    derived: ldOut.derived,
    graph_hash: ldOut.graph_hash,
  },
  [{ objectId: 'graphlaw_state:e2e', qualifier: 'law_state' }],
);

// ── (h) receipt export-ocel: ledger as OCEL 2.0 ─────────────────────────────
// `receipt export-ocel` reads the configured receipts.dir; point it at the
// plan-run ledger via the documented PRAXIS_CONFIG__* env override layer
// (src/config.rs) so the export covers the ledger this pass just validated.
const exportRel = 'docs/releases/v26.7.6/ocel/ledger-export.ocel.json';
const xo = run('receipt-export-ocel', MCP_BIN, ['receipt', 'export-ocel', '--out', exportRel], {
  env: { PRAXIS_CONFIG__RECEIPTS__DIR: 'target/plan_run/ocel_pass_receipts' },
});
const exportAbs = path.join(REPO, exportRel);
addObject('report:ledger_export_ocel', 'report_artifact', {
  ...baseObjectAttrs(xo, 'receipt ledger exported as OCEL 2.0', 'evidence'),
  path: exportRel,
  object_hash: fs.existsSync(exportAbs) ? sha256(fs.readFileSync(exportAbs)) : 'absent',
});
addEvent(
  'ggen_artifact_generated',
  { ...baseEventAttrs(xo, 'none'), artifact: exportRel },
  [{ objectId: 'report:ledger_export_ocel', qualifier: 'artifact' }],
);

// ── intermediate log ─────────────────────────────────────────────────────────
function writeIntermediate(finishedAtUtc) {
  const intermediate = {
    run_id: RUN_ID,
    release_id: RELEASE_ID,
    started_at_utc: startedAtUtc,
    finished_at_utc: finishedAtUtc,
    powl_chain_hash: observedChainHash,
    factory_head_chain_hash: EXPECTED_FACTORY_HEAD,
    events,
    objects,
  };
  fs.writeFileSync(
    path.join(OCEL_DIR, 'driver-intermediate.json'),
    JSON.stringify(intermediate, null, 2) + '\n',
  );
}
writeIntermediate(new Date().toISOString());
console.log(
  `[driver] OK: ${events.length} events, ${objects.length} objects -> docs/releases/v26.7.6/ocel/driver-intermediate.json`,
);

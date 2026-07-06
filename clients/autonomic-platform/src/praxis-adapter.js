/**
 * praxis-adapter.js
 * -----------------------------------------------------------------------------
 * Read path of the Praxis CLIENT_ADAPTER_CONTRACT
 * (docs/releases/v26.7.6/CLIENT_ADAPTER_CONTRACT.md) for this client.
 *
 * Sources consumed verbatim (served in dev by the `praxis-artifacts` Vite
 * middleware, see vite.config.js):
 *   - /praxis-artifacts/receipt.json       (.ggen-v2/receipt.json — latest receipt)
 *   - /praxis-artifacts/receipt-log.jsonl  (.ggen-v2/receipt-log.jsonl — chain)
 *   - /praxis-artifacts/plan.json          (target/plan_run/<run>/plan.json wrapped
 *                                           as { ref, data }, incl. powl_chain_hash)
 *   - /praxis-artifacts/registry.md        (docs/releases/v26.7.6/
 *                                           BREED_ALGORITHM_REGISTRY.md — parsed tables)
 *
 * Contract rules honored here:
 *   - Read-only: nothing is written, nothing is synthesized. A value that has
 *     no source is the UNKNOWN sentinel, never a fabricated default.
 *   - Every value returned carries { source, ref } provenance where
 *     source ∈ { "receipt", "law-export", "plan", "report" }.
 */

const RECEIPT_JSON = '/praxis-artifacts/receipt.json';
const RECEIPT_LOG = '/praxis-artifacts/receipt-log.jsonl';
const PLAN_JSON = '/praxis-artifacts/plan.json';
const REGISTRY_MD = '/praxis-artifacts/registry.md';

const REGISTRY_REF = 'docs/releases/v26.7.6/BREED_ALGORITHM_REGISTRY.md';
const RECEIPT_LOG_REF = '.ggen-v2/receipt-log.jsonl';
const RECEIPT_JSON_REF = '.ggen-v2/receipt.json';

/** The one lawful "no source" value. Rendered as UNKNOWN, never green. */
export const UNKNOWN = Object.freeze({ value: null, source: null, ref: null, unknown: true });

const known = (value, source, ref) => ({ value, source, ref, unknown: false });

async function fetchText(url) {
  try {
    const res = await fetch(url);
    if (!res.ok) return null;
    return await res.text();
  } catch {
    return null;
  }
}

/* ---------------- registry markdown table parsing ---------------- */

// Parses every GFM table in the document into arrays of row objects keyed by
// lower-cased header cell (backticks stripped from cell values).
export function parseMarkdownTables(md) {
  const tables = [];
  const lines = md.split('\n');
  for (let i = 0; i < lines.length - 1; i++) {
    if (!lines[i].trim().startsWith('|')) continue;
    if (!/^\s*\|[\s|:-]+\|\s*$/.test(lines[i + 1] || '')) continue;
    const headers = lines[i].split('|').slice(1, -1).map((h) => h.trim().toLowerCase());
    const rows = [];
    let j = i + 2;
    for (; j < lines.length && lines[j].trim().startsWith('|'); j++) {
      const cells = lines[j].split('|').slice(1, -1).map((c) => c.trim().replace(/^`|`$/g, ''));
      const row = {};
      headers.forEach((h, k) => { row[h] = cells[k] ?? ''; });
      rows.push(row);
    }
    tables.push({ headers, rows });
    i = j;
  }
  return tables;
}

/**
 * Breed/algorithm registry -> card records. No rarity is fabricated: the
 * registry (as generated) carries Status/Standing, not speedTier/qualityTier;
 * when those columns are present they are passed through as `speedTier` /
 * `qualityTier`, otherwise the law-derived standing is the only badge.
 */
async function loadRegistry() {
  const md = await fetchText(REGISTRY_MD);
  if (md == null) return UNKNOWN;
  const tables = parseMarkdownTables(md);
  const cards = [];
  for (const t of tables) {
    const idKey = t.headers.find((h) => h === 'breedid' || h === 'algorithmid');
    if (!idKey) continue;
    for (const r of t.rows) {
      cards.push({
        id: r[idKey],
        kind: idKey === 'breedid' ? 'breed' : 'algorithm',
        label: r.label || r[idKey],
        standing: r.status || r.standing || null,
        category: r.category || (idKey === 'breedid' ? 'cognition' : null),
        citation: r.citation || null,
        speedTier: r.speedtier || null,
        qualityTier: r.qualitytier || null,
        provenance: { source: 'report', ref: REGISTRY_REF },
      });
    }
  }
  return known(cards, 'report', REGISTRY_REF);
}

/* ---------------- receipt chain ---------------- */

async function loadChain() {
  const log = await fetchText(RECEIPT_LOG);
  if (log != null) {
    const records = log.split('\n').filter((l) => l.trim()).map((l) => JSON.parse(l));
    return known(records, 'receipt', RECEIPT_LOG_REF);
  }
  // Fall back to the single latest receipt.
  const single = await fetchText(RECEIPT_JSON);
  if (single != null) return known([JSON.parse(single)], 'receipt', RECEIPT_JSON_REF);
  return UNKNOWN;
}

/* ---------------- plan ---------------- */

async function loadPlan() {
  const txt = await fetchText(PLAN_JSON);
  if (txt == null) return UNKNOWN;
  const { ref, data } = JSON.parse(txt);
  return known(data, 'plan', ref);
}

/* ---------------- public surface ---------------- */

/**
 * getStanding() -> { artifacts, blockers, plan, chain_head, provenance }
 * Every field is either UNKNOWN or { value, source, ref }.
 *
 *   artifacts  : registry cards (report-sourced)
 *   blockers   : receipt records whose andon is not "Green" — the typed
 *                refusal/blocker surface. An empty array is a lawful state.
 *   plan       : plan.json content incl. powl_chain_hash (plan-sourced)
 *   chain_head : current_chain_hash of the last receipt (receipt-sourced);
 *                .value also carries receipt_count for the HUD.
 */
export async function getStanding() {
  const [artifacts, chain, plan] = await Promise.all([loadRegistry(), loadChain(), loadPlan()]);

  let chainHead = UNKNOWN;
  let blockers = UNKNOWN;
  if (!chain.unknown) {
    const records = chain.value;
    const last = records[records.length - 1];
    const rec = last.record || last;
    chainHead = known(
      { chain_hash: rec.chain_hash_hex, andon: rec.andon, activity: rec.activity, receipt_count: records.length },
      chain.source, chain.ref,
    );
    blockers = known(
      records
        .map((r) => r.record || r)
        .filter((r) => r.andon && r.andon !== 'Green')
        .map((r) => ({ refusal: r.andon, activity: r.activity, chain_hash: r.chain_hash_hex })),
      chain.source, chain.ref,
    );
  }

  return {
    artifacts,
    blockers,
    plan,
    chain_head: chainHead,
    provenance: {
      registry: REGISTRY_REF,
      receipts: chain.unknown ? null : chain.ref,
      plan: plan.unknown ? null : plan.ref,
    },
  };
}

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

/* ---------------- case study (Lane 5, autonomic-standing-factory) ---------------- */

const CASE_STUDY_DIR = 'docs/case-studies/autonomic-standing-factory/case-study';
const CS_FINAL_VERDICT_URL = '/praxis-artifacts/case-study/final-verdict.json';
const CS_OCEL_URL = '/praxis-artifacts/case-study/ocel.json';
const CS_WASM4PM_URL = '/praxis-artifacts/case-study/wasm4pm-validation.json';
const CS_POWL_URL = '/praxis-artifacts/case-study/powl-model.json';
const CS_PDDL_URL = '/praxis-artifacts/case-study/pddl-plan.json';

const CS_FINAL_VERDICT_REF = `${CASE_STUDY_DIR}/final_graphlaw_verdict.json`;
const CS_OCEL_REF = `${CASE_STUDY_DIR}/ocel_case_study.json`;
const CS_WASM4PM_REF = `${CASE_STUDY_DIR}/wasm4pm_validation.json`;
const CS_POWL_REF = `${CASE_STUDY_DIR}/powl_model.json`;
const CS_PDDL_REF = `${CASE_STUDY_DIR}/pddl-out/plan.json`;

async function fetchJson(url) {
  const txt = await fetchText(url);
  if (txt == null) return null;
  try { return JSON.parse(txt); } catch { return null; }
}

// Finds an OCEL 2.0 event of `type` and returns its attributes as a plain
// { name: value } map (the wire shape is an array of { name, value }).
function ocelEventAttrs(ocel, type) {
  if (!ocel || !Array.isArray(ocel.events)) return null;
  const ev = ocel.events.find((e) => e.type === type);
  if (!ev) return null;
  const attrs = {};
  for (const a of ev.attributes || []) attrs[a.name] = a.value;
  return attrs;
}

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

/**
 * getCaseStudy() -> the autonomic-standing-factory case-study status rows.
 * Every field is either UNKNOWN or { value, source, ref }. Sourced from the
 * Lane 4 evidence artifacts served under /praxis-artifacts/case-study/* (see
 * vite.config.js). A missing/unreachable artifact makes every field derived
 * from it UNKNOWN — nothing here is computed client-side or defaulted.
 */
export async function getCaseStudy() {
  const [verdictDoc, ocelDoc, wasm4pmDoc, powlDoc, pddlDoc] = await Promise.all([
    fetchJson(CS_FINAL_VERDICT_URL),
    fetchJson(CS_OCEL_URL),
    fetchJson(CS_WASM4PM_URL),
    fetchJson(CS_POWL_URL),
    fetchJson(CS_PDDL_URL),
  ]);

  const v = verdictDoc == null ? UNKNOWN : known(verdictDoc, 'graphlaw', CS_FINAL_VERDICT_REF);
  const ocel = ocelDoc == null ? UNKNOWN : known(ocelDoc, 'ocel', CS_OCEL_REF);
  const wasm4pm = wasm4pmDoc == null ? UNKNOWN : known(wasm4pmDoc, 'wasm4pm', CS_WASM4PM_REF);
  const powl = powlDoc == null ? UNKNOWN : known(powlDoc, 'powl', CS_POWL_REF);
  const pddl = pddlDoc == null ? UNKNOWN : known(pddlDoc, 'plan', CS_PDDL_REF);

  const caseStudyStanding = v.unknown ? UNKNOWN : known(
    { verdict: v.value.verdict, raw_verdict_fact: v.value.raw_verdict_fact, scope: v.value.scope, generated_at_utc: v.value.generated_at_utc },
    'graphlaw', CS_FINAL_VERDICT_REF,
  );

  const graphlawVerdict = v.unknown ? UNKNOWN : known(v.value.verdict, 'graphlaw', CS_FINAL_VERDICT_REF);

  const shaclStatus = v.unknown ? UNKNOWN : known(
    { conforms_all: (v.value.shacl_reports || []).every((r) => r.conforms), reports: v.value.shacl_reports },
    'graphlaw', CS_FINAL_VERDICT_REF,
  );

  const shexStatus = v.unknown ? UNKNOWN : known(v.value.shex_report, 'graphlaw', CS_FINAL_VERDICT_REF);

  const n3DatalogStatus = v.unknown ? UNKNOWN : known(
    {
      derived_triple_count: v.value.derived_triple_count,
      unsatisfied_dependency_count: v.value.unsatisfied_dependency_count,
      denials_count: (v.value.denials || []).length,
    },
    'graphlaw', CS_FINAL_VERDICT_REF,
  );

  // No "admitted" boolean is present in the wired pddl-out/plan.json artifact
  // itself (it appears only in an ad-hoc raw command-log capture, not in the
  // machine artifact) — reporting only what this artifact actually carries.
  const pddlPlanStatus = pddl.unknown ? UNKNOWN : known(
    { step_count: (pddl.value.plan || []).length, steps: pddl.value.plan, powl_chain_hash: pddl.value.powl_chain_hash, graph_hash: pddl.value.graph_hash },
    'plan', CS_PDDL_REF,
  );

  const powlModelStatus = powl.unknown ? UNKNOWN : known(
    { children_count: (powl.value.children || []).length, order_pairs_count: (powl.value.order_pairs || []).length, alphabet_count: (powl.value.alphabet || []).length },
    'powl', CS_POWL_REF,
  );

  const ocelLogStatus = ocel.unknown ? UNKNOWN : known(
    {
      event_count: (ocel.value.events || []).length,
      object_count: (ocel.value.objects || []).length,
      event_type_count: (ocel.value.eventTypes || []).length,
      object_type_count: (ocel.value.objectTypes || []).length,
    },
    'ocel', CS_OCEL_REF,
  );

  const wasm4pmStatus = wasm4pm.unknown ? UNKNOWN : known(
    {
      is_conforming: wasm4pm.value.is_conforming,
      fitness: wasm4pm.value.fitness,
      violations_count: (wasm4pm.value.violations || []).length,
      model_ref: wasm4pm.value.model_ref,
      ocel_ref: wasm4pm.value.ocel_ref,
    },
    'wasm4pm', CS_WASM4PM_REF,
  );

  // Benchmark/receipt status are derived from the OCEL log's own recorded
  // events (the artifacts above carry no dedicated benchmark/receipt file) —
  // UNKNOWN if the OCEL log is unreachable or lacks the event.
  const benchAttrs = ocel.unknown ? null : ocelEventAttrs(ocel.value, 'benchmarks_attached');
  const benchmarkStatus = benchAttrs == null ? UNKNOWN : known(
    { reused: benchAttrs.reused, note: benchAttrs.note, evidence_refs: benchAttrs.evidence_refs },
    'ocel', CS_OCEL_REF,
  );

  const receiptAttrs = ocel.unknown ? null : ocelEventAttrs(ocel.value, 'receipts_verified');
  const receiptStatus = receiptAttrs == null ? UNKNOWN : known(
    { verdict_ok: receiptAttrs.verdict_ok, exit_code: receiptAttrs.exit_code, evidence_refs: receiptAttrs.evidence_refs },
    'ocel', CS_OCEL_REF,
  );

  // No wired case-study artifact carries a structured external-side-effect
  // list (it exists only as prose in the Lane 4 report markdown) — honestly
  // UNKNOWN rather than parsed out of free text.
  const externalSideEffects = UNKNOWN;

  const criteriaSummary = v.unknown ? UNKNOWN : known(
    (() => {
      const criteria = v.value.criteria || [];
      const satisfied = criteria.filter((c) => c.satisfied);
      const unsatisfiedCritical = criteria.filter((c) => c.critical && !c.satisfied);
      return { total: criteria.length, satisfied_count: satisfied.length, unsatisfied_critical_count: unsatisfiedCritical.length };
    })(),
    'graphlaw', CS_FINAL_VERDICT_REF,
  );

  const finalVerdict = graphlawVerdict;

  return {
    case_study_standing: caseStudyStanding,
    graphlaw_verdict: graphlawVerdict,
    shacl_status: shaclStatus,
    shex_status: shexStatus,
    n3_datalog_status: n3DatalogStatus,
    pddl_plan_status: pddlPlanStatus,
    powl_model_status: powlModelStatus,
    ocel_log_status: ocelLogStatus,
    wasm4pm_status: wasm4pmStatus,
    benchmark_status: benchmarkStatus,
    receipt_status: receiptStatus,
    external_side_effects: externalSideEffects,
    criteria_summary: criteriaSummary,
    final_verdict: finalVerdict,
    provenance: {
      final_verdict: v.unknown ? null : CS_FINAL_VERDICT_REF,
      ocel: ocel.unknown ? null : CS_OCEL_REF,
      wasm4pm: wasm4pm.unknown ? null : CS_WASM4PM_REF,
      powl: powl.unknown ? null : CS_POWL_REF,
      pddl_plan: pddl.unknown ? null : CS_PDDL_REF,
    },
  };
}

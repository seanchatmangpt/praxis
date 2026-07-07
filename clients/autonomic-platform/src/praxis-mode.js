/**
 * praxis-mode.js
 * -----------------------------------------------------------------------------
 * PraxisProvider mode — the standing-mapped data layer that sits alongside the
 * bundled supabase-mock mode. Doctrine (CLIENT_SURFACES.md): clients display
 * and command standing, they never create it. Everything rendered here either
 * carries { source, ref } provenance from praxis-adapter.js or renders as
 * UNKNOWN; any screen still driven by the simulation/mock carries the
 * persistent NON-STANDING banner.
 *
 * Standing-mapped screens:
 *   DECK — cards from BREED_ALGORITHM_REGISTRY.md (report-sourced)
 *   OPS  — incidents = typed refusals/blockers from the receipt chain
 *   HUD  — chain head + receipt count from the receipt ledger
 * Mock-labeled screens: GLOBE/COMMAND, ARENA, and every other sim screen.
 */

import React, { createContext, useContext, useEffect, useState } from 'react';
import { getStanding, getCaseStudy, getMfactStanding, UNKNOWN } from './praxis-adapter.js';
import { PALETTE, Panel } from './AutonomicPlatform.js';

const mono = "'JetBrains Mono', ui-monospace, monospace";
const sans = "'Space Grotesk', system-ui, sans-serif";

/* ---------------- provider ---------------- */

const PraxisContext = createContext(null);

const EMPTY = { artifacts: UNKNOWN, blockers: UNKNOWN, plan: UNKNOWN, chain_head: UNKNOWN, provenance: {}, mfact: null };

export function PraxisProvider({ children }) {
  const [standing, setStanding] = useState(EMPTY);
  useEffect(() => {
    let live = true;
    const poll = () =>
      Promise.all([getStanding(), getMfactStanding().catch(() => null)])
        .then(([s, m]) => { if (live) setStanding({ ...s, mfact: m }); })
        .catch(() => {});
    poll();
    // Receipts are the sync primitive: poll the chain; a changed chain head is
    // the only signal that standing moved (adapter contract rule 4).
    const t = setInterval(poll, 15000);
    return () => { live = false; clearInterval(t); };
  }, []);
  return <PraxisContext.Provider value={standing}>{children}</PraxisContext.Provider>;
}

export function usePraxis() {
  const ctx = useContext(PraxisContext);
  if (!ctx) throw new Error('usePraxis() must be used inside <PraxisProvider>');
  return ctx;
}

/* ---------------- shared presentation ---------------- */

/** Persistent banner for any screen whose data comes from the mock/simulation. */
export function NonStandingBanner({ label = 'NON-STANDING · simulated/mock data · maps to no receipt, law export, plan, or report' }) {
  return (
    <div style={{ flex: '0 0 auto', padding: '6px 14px', background: 'rgba(255,177,61,0.14)', borderBottom: '1px solid rgba(255,177,61,0.45)', color: PALETTE.amber, font: `600 10px ${mono}`, letterSpacing: 0.6 }}>
      ⚠ {label}
    </div>
  );
}

/** Visually-distinct UNKNOWN chip — never green (adapter contract rule 2). */
export function UnknownChip({ label }) {
  return (
    <span data-testid="unknown-chip" style={{ font: `600 10px ${mono}`, color: PALETTE.dim, border: `1px dashed ${PALETTE.dim}`, borderRadius: 5, padding: '2px 7px' }}>
      {label ? `${label} · ` : ''}UNKNOWN
    </span>
  );
}

// Prop is `refPath`, not `ref`: `ref` is a reserved React prop, and passing a
// string to it on a function component throws at render ("Function components
// cannot have string refs"), blanking the standing HUD.
function ProvenanceTag({ source, refPath }) {
  return (
    <span data-testid="provenance-chip" title={refPath} style={{ font: `500 8px ${mono}`, color: PALETTE.dim, border: `1px solid ${PALETTE.line2}`, borderRadius: 5, padding: '1px 5px' }}>
      {source}:{refPath}
    </span>
  );
}

// Lazy (function, not module-level object): this module is in an import
// cycle with AutonomicPlatform.js, so reading PALETTE during module
// evaluation is a TDZ ReferenceError ("Cannot access 'PALETTE' before
// initialization") that blanks the whole app. Render-time reads are safe —
// the cycle has fully initialized by then.
const standingColor = (standing) =>
  ({
    EvidenceBound: PALETTE.cyan,
    REPLAYABLE: PALETTE.emerald,
    Verified: PALETTE.emerald,
    Blocked: PALETTE.magenta,
  })[standing];

/* ---------------- HUD strip (chain head + receipt count) ---------------- */

export function PraxisHud() {
  const { chain_head, plan } = usePraxis();
  return (
    <div style={{ height: 34, flex: '0 0 34px', display: 'flex', alignItems: 'center', gap: 14, padding: '0 22px', borderBottom: `1px solid ${PALETTE.line}`, background: 'rgba(7,12,22,0.8)', fontFamily: mono }}>
      <span style={{ font: `600 9px ${mono}`, letterSpacing: 0.8, color: PALETTE.dim }}>PRAXIS STANDING</span>
      {chain_head.unknown ? (
        <UnknownChip label="CHAIN HEAD" />
      ) : (
        <>
          <span style={{ font: `600 10px ${mono}`, color: PALETTE.emerald }}>
            HEAD {String(chain_head.value.chain_hash).slice(0, 16)}…
          </span>
          <span style={{ font: `600 10px ${mono}`, color: PALETTE.cyan }}>
            RECEIPTS {chain_head.value.receipt_count}
          </span>
          <ProvenanceTag source={chain_head.source} refPath={chain_head.ref} />
        </>
      )}
      <MfactHudSegment />
      <div style={{ flex: 1 }} />
      {plan.unknown ? (
        <UnknownChip label="PLAN" />
      ) : (
        <>
          <span style={{ font: `500 9px ${mono}`, color: PALETTE.violet }}>
            POWL {String(plan.value.powl_chain_hash || '').slice(0, 23)}…
          </span>
          <ProvenanceTag source={plan.source} refPath={plan.ref} />
        </>
      )}
    </div>
  );
}

/* ---------------- mfact rail segment (Lean/Lake manufacturing standing) ---------------- */

// Display-only projection of mfact's certified receipts (adapter contract:
// clients display and command standing; they never create it). UNKNOWN when
// the artifact bridge cannot reach /Users/sac/mfact/release/*.
function MfactHudSegment() {
  const { mfact } = usePraxis();
  if (!mfact || mfact.core.unknown) return <UnknownChip label="MFACT" />;
  const core = mfact.core.value;
  const packets = mfact.packets.unknown ? null : mfact.packets.value;
  const alive = packets ? packets.packets.filter((p) => p.status === 'ALIVE').length : 0;
  const total = packets ? packets.packets.length : 0;
  const lanes = mfact.lanes.unknown ? null : mfact.lanes.value;
  return (
    <>
      <span style={{ font: `600 10px ${mono}`, color: PALETTE.emerald }}>
        MFACT {core.release} {core.status}
      </span>
      <span style={{ font: `600 10px ${mono}`, color: PALETTE.cyan }}>
        PROVEN {core.coreProven}/{core.coreTotalDecls}
      </span>
      {packets && (
        <span style={{ font: `600 10px ${mono}`, color: alive === total ? PALETTE.emerald : PALETTE.amber }}>
          PACKETS {alive}/{total} · {packets.publicationActuation}
        </span>
      )}
      {lanes && (
        <span style={{ font: `500 9px ${mono}`, color: PALETTE.violet }}>
          REPLAY {lanes.replay} · DOCS {lanes.docsLane} · CROWN {lanes.wfnetCrownTheorem}
        </span>
      )}
      <ProvenanceTag source={mfact.core.source} refPath={mfact.core.ref} />
    </>
  );
}

/* ---------------- DECK — registry-driven cards ---------------- */

export function PraxisDeckScreen() {
  const { artifacts } = usePraxis();
  if (artifacts.unknown) {
    return (
      <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 10, color: PALETTE.dim, fontFamily: sans }}>
        <UnknownChip label="BREED/ALGORITHM REGISTRY" />
        <span style={{ font: `400 11px ${mono}` }}>docs/releases/v26.7.6/BREED_ALGORITHM_REGISTRY.md not reachable — nothing is rendered in its place</span>
      </div>
    );
  }
  const cards = artifacts.value;
  return (
    <div style={{ flex: 1, padding: 22, overflowY: 'auto' }}>
      <div style={{ marginBottom: 12, display: 'flex', gap: 10, alignItems: 'center' }}>
        <span style={{ font: `600 11px ${mono}`, color: PALETTE.hi }}>{cards.length} admitted breeds/algorithms</span>
        <ProvenanceTag source={artifacts.source} refPath={artifacts.ref} />
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))', gap: 14 }}>
        {cards.map((c) => {
          const col = standingColor(c.standing) || PALETTE.mid;
          // Rarity tier comes only from speedTier/qualityTier when the registry
          // carries them; otherwise the law-derived standing is the only badge.
          const tier = c.speedTier || c.qualityTier;
          return (
            <article key={`${c.kind}:${c.id}`} style={{ border: `1px solid ${col}`, borderRadius: 14, padding: 14, background: PALETTE.panel2 }}>
              <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
                <span style={{ font: `700 13px ${sans}`, color: PALETTE.hi }}>{c.label}</span>
                <span style={{ font: `600 8px ${mono}`, color: col }}>{c.standing || 'UNKNOWN'}</span>
              </header>
              <p style={{ margin: 0, font: `500 9px ${mono}`, color: PALETTE.dim }}>
                {c.kind.toUpperCase()} · {c.id}{c.category ? ` · ${c.category}` : ''}{tier ? ` · TIER ${tier}` : ''}
              </p>
              {c.citation && (
                <p style={{ margin: '8px 0 0', font: `400 10px ${sans}`, color: PALETTE.mid, maxHeight: 56, overflow: 'hidden' }}>{c.citation}</p>
              )}
              <footer style={{ marginTop: 8 }}>
                <ProvenanceTag source={c.provenance.source} refPath={c.provenance.ref} />
              </footer>
            </article>
          );
        })}
      </div>
    </div>
  );
}

/* ---------------- OPS — typed refusals / blockers ---------------- */

export function PraxisOpsScreen() {
  const { blockers, plan } = usePraxis();
  return (
    <div style={{ flex: 1, padding: 22, display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16, overflowY: 'auto', alignContent: 'start' }}>
      <Panel title="Incidents — typed refusals / blockers" tag="RECEIPT CHAIN">
        {blockers.unknown ? (
          <UnknownChip label="BLOCKERS" />
        ) : blockers.value.length === 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
            <span style={{ font: `500 11px ${mono}`, color: PALETTE.emerald }}>No non-Green receipts in the chain.</span>
            <ProvenanceTag source={blockers.source} refPath={blockers.ref} />
          </div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            {blockers.value.map((b) => (
              <div key={b.chain_hash} style={{ display: 'flex', flexDirection: 'column', gap: 2, paddingBottom: 8, borderBottom: `1px solid ${PALETTE.line}` }}>
                <span style={{ font: `600 12px ${sans}`, color: PALETTE.magenta }}>{b.refusal}</span>
                <span style={{ font: `400 10px ${mono}`, color: PALETTE.mid }}>{b.activity}</span>
                <ProvenanceTag source="receipt" refPath={b.chain_hash} />
              </div>
            ))}
          </div>
        )}
      </Panel>
      <Panel title="Plan" tag="POWL">
        {plan.unknown ? (
          <UnknownChip label="PLAN" />
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {(plan.value.plan || []).map((step, i) => (
              <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <span style={{ width: 18, font: `700 11px ${mono}`, color: PALETTE.dim }}>{i + 1}</span>
                <span style={{ font: `500 12px ${mono}`, color: PALETTE.hi }}>{step}</span>
              </div>
            ))}
            <span style={{ font: `400 9px ${mono}`, color: PALETTE.dim, wordBreak: 'break-all' }}>
              powl_chain_hash: {plan.value.powl_chain_hash || 'absent'}
            </span>
            <ProvenanceTag source={plan.source} refPath={plan.ref} />
          </div>
        )}
      </Panel>
    </div>
  );
}

/* ---------------- CASE STUDY — autonomic-standing-factory Lane 5 ---------------- */

// True only for values that read as an unambiguous pass/positive result.
// Used only for color; every row still renders its ProvenanceTag regardless
// of color, so a green value is never shown without provenance next to it.
function isPositive(value) {
  if (value === true) return true;
  if (typeof value === 'string') return /^(true|conforming|pass|ready)/i.test(value) && !/not[_ -]?ready/i.test(value);
  return false;
}

function CaseStudyValue({ children, positive }) {
  return (
    <span data-testid="status-value" data-positive={positive ? 'true' : 'false'} style={{ font: `600 12px ${mono}`, color: positive ? PALETTE.emerald : PALETTE.hi }}>{children}</span>
  );
}

// One status row: label + rendered value (or UNKNOWN) + provenance chip.
// `field` is a praxis-adapter known()/UNKNOWN object; `render(value)` returns
// the value's display node. The ProvenanceTag is always emitted alongside a
// known value — there is no code path that renders a value without it.
// data-testid="status-row" + data-known lets Playwright assert, over the
// live DOM, that no positive/green value ever renders without a sibling
// provenance-chip in the same row.
function StatusRow({ label, field, render }) {
  return (
    <div data-testid="status-row" data-label={label} data-known={field.unknown ? 'false' : 'true'} style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '7px 0', borderBottom: `1px solid ${PALETTE.line}` }}>
      <span style={{ flex: '0 0 220px', font: `600 10px ${mono}`, color: PALETTE.dim, letterSpacing: 0.4 }}>{label}</span>
      <div style={{ flex: 1, display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
        {field.unknown ? (
          <UnknownChip label={label} />
        ) : (
          <>
            {render(field.value)}
            <ProvenanceTag source={field.source} refPath={field.ref} />
          </>
        )}
      </div>
    </div>
  );
}

export function PraxisCaseStudyScreen() {
  const [cs, setCs] = useState(null);
  useEffect(() => {
    let live = true;
    getCaseStudy().then((r) => { if (live) setCs(r); }).catch(() => {});
    return () => { live = false; };
  }, []);

  if (cs == null) {
    return (
      <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: PALETTE.dim, fontFamily: mono, fontSize: 11 }}>
        loading case-study evidence…
      </div>
    );
  }

  return (
    <div style={{ flex: 1, padding: 22, overflowY: 'auto' }}>
      <div style={{ marginBottom: 14, display: 'flex', alignItems: 'center', gap: 10 }}>
        <span style={{ font: `700 15px ${sans}`, color: PALETTE.hi }}>Standing Factory Case Study</span>
        <span style={{ font: `500 9px ${mono}`, color: PALETTE.dim }}>scope: autonomic-standing-factory (local-first, seanchatmangpt fleet)</span>
      </div>
      <Panel title="GraphLaw / OCEL / wasm4pm evidence chain" tag="CASE STUDY">
        <StatusRow
          label="CASE-STUDY STANDING"
          field={cs.case_study_standing}
          render={(v) => <CaseStudyValue positive={isPositive(v.verdict)}>{v.verdict} · {v.scope}</CaseStudyValue>}
        />
        <StatusRow
          label="GRAPHLAW VERDICT"
          field={cs.graphlaw_verdict}
          render={(v) => <CaseStudyValue positive={isPositive(v)}>{v}</CaseStudyValue>}
        />
        <StatusRow
          label="SHACL"
          field={cs.shacl_status}
          render={(v) => <CaseStudyValue positive={v.conforms_all}>{v.conforms_all ? 'all conform' : 'violations present'} · {v.reports.length} shapes</CaseStudyValue>}
        />
        <StatusRow
          label="SHEX"
          field={cs.shex_status}
          render={(v) => <CaseStudyValue positive={v.conforms}>{v.conforms ? 'conforms' : 'failures'} · {v.failure_count} failures</CaseStudyValue>}
        />
        <StatusRow
          label="N3 / DATALOG CLOSURE"
          field={cs.n3_datalog_status}
          render={(v) => <CaseStudyValue positive={v.denials_count === 0}>{v.derived_triple_count} derived triples · {v.unsatisfied_dependency_count} unsatisfied deps · {v.denials_count} denials</CaseStudyValue>}
        />
        <StatusRow
          label="PDDL REPAIR PLAN"
          field={cs.pddl_plan_status}
          render={(v) => <CaseStudyValue positive={v.step_count > 0}>{v.step_count} steps · powl_chain_hash {String(v.powl_chain_hash || '').slice(0, 20)}…</CaseStudyValue>}
        />
        <StatusRow
          label="POWL PROCESS MODEL"
          field={cs.powl_model_status}
          render={(v) => <CaseStudyValue positive={v.children_count > 0}>{v.children_count} children · {v.order_pairs_count} order pairs</CaseStudyValue>}
        />
        <StatusRow
          label="OCEL LOG"
          field={cs.ocel_log_status}
          render={(v) => <CaseStudyValue positive={v.event_count > 0}>{v.event_count} events · {v.object_count} objects</CaseStudyValue>}
        />
        <StatusRow
          label="WASM4PM CONFORMANCE"
          field={cs.wasm4pm_status}
          render={(v) => <CaseStudyValue positive={v.is_conforming}>{v.is_conforming ? 'conforming' : 'non-conforming'} · fitness {v.fitness} · {v.violations_count} violations</CaseStudyValue>}
        />
        <StatusRow
          label="BENCHMARKS"
          field={cs.benchmark_status}
          render={(v) => <CaseStudyValue positive={!!v.reused}>{v.reused ? 'reused (not re-run)' : 'attached'}</CaseStudyValue>}
        />
        <StatusRow
          label="RECEIPTS"
          field={cs.receipt_status}
          render={(v) => <CaseStudyValue positive={!!v.verdict_ok}>{v.verdict_ok ? 'verified' : 'not verified'} · exit {v.exit_code}</CaseStudyValue>}
        />
        <StatusRow
          label="CRITERIA (15-point)"
          field={cs.criteria_summary}
          render={(v) => <CaseStudyValue positive={v.unsatisfied_critical_count === 0}>{v.satisfied_count}/{v.total} satisfied · {v.unsatisfied_critical_count} critical unsatisfied</CaseStudyValue>}
        />
        <StatusRow
          label="EXTERNAL SIDE EFFECTS"
          field={cs.external_side_effects}
          render={(v) => <CaseStudyValue>{JSON.stringify(v)}</CaseStudyValue>}
        />
        <StatusRow
          label="FINAL VERDICT"
          field={cs.final_verdict}
          render={(v) => <CaseStudyValue positive={isPositive(v)}>{v}</CaseStudyValue>}
        />
      </Panel>
    </div>
  );
}

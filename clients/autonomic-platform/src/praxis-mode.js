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
import { getStanding, UNKNOWN } from './praxis-adapter.js';
import { PALETTE, Panel } from './AutonomicPlatform.js';

const mono = "'JetBrains Mono', ui-monospace, monospace";
const sans = "'Space Grotesk', system-ui, sans-serif";

/* ---------------- provider ---------------- */

const PraxisContext = createContext(null);

const EMPTY = { artifacts: UNKNOWN, blockers: UNKNOWN, plan: UNKNOWN, chain_head: UNKNOWN, provenance: {} };

export function PraxisProvider({ children }) {
  const [standing, setStanding] = useState(EMPTY);
  useEffect(() => {
    let live = true;
    const poll = () => getStanding().then((s) => { if (live) setStanding(s); }).catch(() => {});
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
    <span style={{ font: `600 10px ${mono}`, color: PALETTE.dim, border: `1px dashed ${PALETTE.dim}`, borderRadius: 5, padding: '2px 7px' }}>
      {label ? `${label} · ` : ''}UNKNOWN
    </span>
  );
}

function ProvenanceTag({ source, ref: refPath }) {
  return (
    <span title={refPath} style={{ font: `500 8px ${mono}`, color: PALETTE.dim, border: `1px solid ${PALETTE.line2}`, borderRadius: 5, padding: '1px 5px' }}>
      {source}:{refPath}
    </span>
  );
}

const STANDING_COLORS = {
  EvidenceBound: PALETTE.cyan,
  REPLAYABLE: PALETTE.emerald,
  Verified: PALETTE.emerald,
  Blocked: PALETTE.magenta,
};

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
          <ProvenanceTag source={chain_head.source} ref={chain_head.ref} />
        </>
      )}
      <div style={{ flex: 1 }} />
      {plan.unknown ? (
        <UnknownChip label="PLAN" />
      ) : (
        <>
          <span style={{ font: `500 9px ${mono}`, color: PALETTE.violet }}>
            POWL {String(plan.value.powl_chain_hash || '').slice(0, 23)}…
          </span>
          <ProvenanceTag source={plan.source} ref={plan.ref} />
        </>
      )}
    </div>
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
        <ProvenanceTag source={artifacts.source} ref={artifacts.ref} />
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))', gap: 14 }}>
        {cards.map((c) => {
          const col = STANDING_COLORS[c.standing] || PALETTE.mid;
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
                <ProvenanceTag source={c.provenance.source} ref={c.provenance.ref} />
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
            <ProvenanceTag source={blockers.source} ref={blockers.ref} />
          </div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            {blockers.value.map((b) => (
              <div key={b.chain_hash} style={{ display: 'flex', flexDirection: 'column', gap: 2, paddingBottom: 8, borderBottom: `1px solid ${PALETTE.line}` }}>
                <span style={{ font: `600 12px ${sans}`, color: PALETTE.magenta }}>{b.refusal}</span>
                <span style={{ font: `400 10px ${mono}`, color: PALETTE.mid }}>{b.activity}</span>
                <ProvenanceTag source="receipt" ref={b.chain_hash} />
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
            <ProvenanceTag source={plan.source} ref={plan.ref} />
          </div>
        )}
      </Panel>
    </div>
  );
}

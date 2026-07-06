/**
 * PRAXIS ADOPTION NOTE (2026-07-06) — GAP TYPED, NOT WIRED.
 * This module is adopted as-is and is NOT imported by the app. Gap type:
 * unverified third-party API binding (@unrdf/kgc-4d README could not be
 * fetched when this was written; see the API-binding note below). Future
 * lane: instead of projecting simulated client state into RDF, consume the
 * Praxis `law export` output (materialized graph as N-Triples) per
 * docs/releases/v26.7.6/CLIENT_ADAPTER_CONTRACT.md. Do not wire this file
 * until that lane replaces the mock-state projection.
 */
/**
 * kgc4d-integration.js
 * -----------------------------------------------------------------------------
 * Integration of the npm package  @unrdf/kgc-4d  into the Autonomic Platform.
 *
 * "4D knowledge graph" = the platform's live state projected into RDF quads on
 * two axes simultaneously:
 *   • SPATIAL  — each hub/agent carries geo coordinates + load/health literals
 *   • TEMPORAL — every simulation tick is written into its own named graph
 *                (apx:tick/<n>), so an entity's history is a 4D trajectory
 *                (x, y, status, t) you can query with SPARQL.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠ API-binding note
 * I was unable to fetch the exact @unrdf/kgc-4d README in this environment, so
 * the store binding below is written against the *verified* sibling package API
 * (@unrdf/oxigraph: `createStore`, `dataFactory`, `store.add/has/query`,
 * `store.load/dump`). All package-specific calls are isolated in ONE place —
 * `createBackend()` — and it probes several likely export names. If kgc-4d uses
 * a different entry point, adjust the `FACTORY_NAMES` / method names there only;
 * the rest of this file is package-agnostic.
 * ─────────────────────────────────────────────────────────────────────────────
 *
 * Install:   npm i @unrdf/kgc-4d
 * Optional:  npm i @unrdf/oxigraph   (fallback store backend)
 */

import { useEffect, useMemo, useRef, useState } from 'react';
// Tap the live platform state from the module we built earlier.
import { useGame } from './AutonomicPlatform.js';

/* ============================================================================
 * Ontology — IRIs for the Autonomic Platform knowledge graph
 * ==========================================================================*/

export const NS = {
  apx: 'https://autonomic.platform/ns#',          // classes & predicates
  hub: 'https://autonomic.platform/hub/',          // hub instances
  agent: 'https://autonomic.platform/agent/',      // agent instances
  tick: 'https://autonomic.platform/tick/',        // temporal named graphs
  geo: 'http://www.w3.org/2003/01/geo/wgs84_pos#', // lat / long
  xsd: 'http://www.w3.org/2001/XMLSchema#',
  rdf: 'http://www.w3.org/1999/02/22-rdf-syntax-ns#',
};

const IRI = {
  Hub: NS.apx + 'Hub',
  Agent: NS.apx + 'Agent',
  Snapshot: NS.apx + 'Snapshot',
  type: NS.rdf + 'type',
  status: NS.apx + 'status',
  load: NS.apx + 'load',
  sector: NS.apx + 'sector',
  region: NS.apx + 'region',
  uptime: NS.apx + 'uptime',
  latency: NS.apx + 'latency',
  agentStatus: NS.apx + 'agentStatus',
  atTick: NS.apx + 'atTick',
  timestamp: NS.apx + 'timestamp',
  integrity: NS.apx + 'integrity',
  threats: NS.apx + 'threats',
  lat: NS.geo + 'lat',
  long: NS.geo + 'long',
};

const PREFIXES = `
  PREFIX apx: <${NS.apx}>
  PREFIX hub: <${NS.hub}>
  PREFIX agent: <${NS.agent}>
  PREFIX tick: <${NS.tick}>
  PREFIX geo: <${NS.geo}>
  PREFIX xsd: <${NS.xsd}>
  PREFIX rdf: <${NS.rdf}>
`;

/* ============================================================================
 * Backend adapter — the ONLY place that touches the npm package.
 * ==========================================================================*/

// Likely factory export names across the @unrdf family / kgc-4d.
const FACTORY_NAMES = ['createKGC4D', 'create4DStore', 'createStore', 'create', 'createGraph'];

function resolveFactory(mod) {
  if (!mod) return null;
  for (const name of FACTORY_NAMES) {
    if (typeof mod[name] === 'function') return mod[name];
  }
  if (mod.default) {
    if (typeof mod.default === 'function') return mod.default;
    for (const name of FACTORY_NAMES) {
      if (typeof mod.default[name] === 'function') return mod.default[name];
    }
  }
  // Class-style export (e.g. `KGC4D`) — wrap construction in a factory.
  const Cls = mod.KGC4D || mod.FourD || mod.Graph || (mod.default && mod.default.KGC4D);
  if (typeof Cls === 'function') return (opts) => new Cls(opts);
  return null;
}

function resolveDataFactory(mod) {
  return (mod && (mod.dataFactory || mod.DataFactory || (mod.default && mod.default.dataFactory))) || null;
}

/**
 * Loads @unrdf/kgc-4d, falling back to @unrdf/oxigraph, and normalizes whatever
 * it returns to a stable shape: { store, df, kind }.
 *   store.add(quad) / store.has(quad) / store.query(sparql)
 *   df.namedNode / df.literal / df.quad
 */
export async function createBackend(opts = {}) {
  let mod = null;
  let kind = '@unrdf/kgc-4d';
  try {
    mod = await import('@unrdf/kgc-4d');
  } catch (e) {
    // Fallback to the verified sibling backend so the integration still runs.
    try {
      mod = await import('@unrdf/oxigraph');
      kind = '@unrdf/oxigraph (fallback)';
    } catch (e2) {
      throw new Error(
        'kgc4d-integration: could not import @unrdf/kgc-4d or @unrdf/oxigraph. ' +
        'Run `npm i @unrdf/kgc-4d`. (' + e.message + ')'
      );
    }
  }

  const factory = resolveFactory(mod);
  if (!factory) {
    throw new Error(
      'kgc4d-integration: no recognizable store factory export. ' +
      'Inspect the package and add its export name to FACTORY_NAMES in createBackend().'
    );
  }

  const rawStore = factory(opts);
  const store = rawStore && typeof rawStore.then === 'function' ? await rawStore : rawStore;

  // Data factory: prefer the package's, else a minimal RDF/JS-style shim.
  const df = resolveDataFactory(mod) || makeDataFactoryShim();

  return { store, df, kind, module: mod };
}

// Minimal RDF/JS DataFactory shim (used only if the package doesn't expose one).
function makeDataFactoryShim() {
  const term = (termType, value, extra = {}) => ({ termType, value, ...extra });
  return {
    namedNode: (value) => term('NamedNode', value),
    literal: (value, dtOrLang) => {
      if (typeof dtOrLang === 'string') return term('Literal', String(value), { language: dtOrLang, datatype: term('NamedNode', NS.xsd + 'string') });
      return term('Literal', String(value), { datatype: dtOrLang || term('NamedNode', NS.xsd + 'string') });
    },
    quad: (s, p, o, g) => ({ termType: 'Quad', subject: s, predicate: p, object: o, graph: g || term('DefaultGraph', '') }),
  };
}

/* ============================================================================
 * KnowledgeGraph4D — projects platform state into the 4D graph
 * ==========================================================================*/

export class KnowledgeGraph4D {
  constructor(backend) {
    this.store = backend.store;
    this.df = backend.df;
    this.kind = backend.kind;
    this.lastTick = -1;
  }

  static async open(opts) {
    return new KnowledgeGraph4D(await createBackend(opts));
  }

  // ---- term helpers ----
  _node(iri) { return this.df.namedNode(iri); }
  _int(n) { return this.df.literal(String(Math.round(n)), this._node(NS.xsd + 'integer')); }
  _dec(n) { return this.df.literal(String(n), this._node(NS.xsd + 'decimal')); }
  _str(s) { return this.df.literal(String(s)); }
  _dt(iso) { return this.df.literal(iso, this._node(NS.xsd + 'dateTime')); }

  _add(s, p, o, g) {
    const q = this.df.quad(s, p, o, g);
    // Support both add(quad) and addQuad(quad) shaped stores.
    if (typeof this.store.add === 'function') this.store.add(q);
    else if (typeof this.store.addQuad === 'function') this.store.addQuad(q);
    else throw new Error('Backend store exposes neither add() nor addQuad().');
  }

  /**
   * Write one temporal slice of the platform into named graph apx:tick/<n>.
   * Call once per simulation tick (the hook below does this automatically).
   */
  snapshot(state) {
    const tick = state.gtick;
    if (tick === this.lastTick) return tick; // dedupe
    this.lastTick = tick;

    const g = this._node(NS.tick + tick);
    const tickNode = this._node(NS.tick + tick);
    const nowIso = new Date().toISOString();

    // Snapshot metadata
    this._add(tickNode, this._node(IRI.type), this._node(IRI.Snapshot), g);
    this._add(tickNode, this._node(IRI.timestamp), this._dt(nowIso), g);
    this._add(tickNode, this._node(IRI.integrity), this._dec(state.metrics.integrity), g);
    this._add(tickNode, this._node(IRI.threats), this._int(state.metrics.threats), g);

    // Hubs — spatial (lat/long/load) + status, per tick (the temporal axis)
    state.hubs.forEach((h) => {
      const s = this._node(NS.hub + h.id);
      this._add(s, this._node(IRI.type), this._node(IRI.Hub), g);
      this._add(s, this._node(IRI.lat), this._dec(h.lat), g);
      this._add(s, this._node(IRI.long), this._dec(h.lng), g);
      this._add(s, this._node(IRI.load), this._int(h.load), g);
      this._add(s, this._node(IRI.region), this._str(h.region), g);
      this._add(s, this._node(IRI.sector), this._str(h.sector), g);
      this._add(s, this._node(IRI.status), this._str(state.statuses[h.id]), g);
      this._add(s, this._node(IRI.atTick), this._int(tick), g);
    });

    // Agents — live telemetry per tick
    state.agents.forEach((a) => {
      const s = this._node(NS.agent + a.id);
      this._add(s, this._node(IRI.type), this._node(IRI.Agent), g);
      this._add(s, this._node(IRI.uptime), this._dec(a.uptime), g);
      this._add(s, this._node(IRI.latency), this._int(a.latency), g);
      this._add(s, this._node(IRI.agentStatus), this._str(a.status), g);
      this._add(s, this._node(IRI.atTick), this._int(tick), g);
    });

    return tick;
  }

  /** Raw SPARQL passthrough (prefixes auto-prepended). */
  query(sparql) {
    if (typeof this.store.query !== 'function') {
      throw new Error('Backend store has no query() method — use match()/getQuads() instead.');
    }
    return this.store.query(PREFIXES + sparql);
  }

  /** Full temporal history of one hub: [{ tick, status, load }]. */
  hubHistory(hubId) {
    return this.query(`
      SELECT ?tick ?status ?load WHERE {
        GRAPH ?g {
          hub:${hubId} apx:atTick ?tick ;
                       apx:status ?status ;
                       apx:load ?load .
        }
      } ORDER BY ?tick
    `);
  }

  /** Everything recorded at a given tick. */
  at(tick) {
    return this.query(`
      SELECT ?s ?p ?o WHERE { GRAPH tick:${tick} { ?s ?p ?o } }
    `);
  }

  /** Hubs that were disrupted at any point — the 4D "hotspots". */
  disruptionHotspots() {
    return this.query(`
      SELECT ?hub (COUNT(?g) AS ?disruptedTicks) WHERE {
        GRAPH ?g { ?hub a apx:Hub ; apx:status "disrupted" }
      } GROUP BY ?hub ORDER BY DESC(?disruptedTicks)
    `);
  }

  /** Serialize the whole graph (if the backend supports dump/serialize). */
  dump(format = 'application/n-quads') {
    if (typeof this.store.dump === 'function') return this.store.dump(format);
    if (typeof this.store.serialize === 'function') return this.store.serialize(format);
    return null;
  }

  get size() {
    if (typeof this.store.size === 'number') return this.store.size;
    if (typeof this.store.size === 'function') return this.store.size();
    if (typeof this.store.countQuads === 'function') return this.store.countQuads();
    return undefined;
  }
}

/* ============================================================================
 * React hook — auto-snapshots platform ticks into the 4D graph
 * ----------------------------------------------------------------------------
 * Usage (inside any component under <GameProvider>):
 *
 *   const { kg, ready, tick, query } = useKnowledgeGraph4D();
 *   // kg.hubHistory(4), kg.disruptionHotspots(), query('SELECT ...')
 * ==========================================================================*/

export function useKnowledgeGraph4D({ autoSnapshot = true, maxTicks = 500 } = {}) {
  const { state } = useGame();
  const kgRef = useRef(null);
  const [ready, setReady] = useState(false);
  const [err, setErr] = useState(null);
  const [recordedTick, setRecordedTick] = useState(-1);

  // Open the backend once.
  useEffect(() => {
    let cancelled = false;
    KnowledgeGraph4D.open()
      .then((kg) => { if (!cancelled) { kgRef.current = kg; setReady(true); } })
      .catch((e) => { if (!cancelled) setErr(e); });
    return () => { cancelled = true; };
  }, []);

  // Snapshot each new tick.
  useEffect(() => {
    if (!ready || !autoSnapshot || !kgRef.current) return;
    if (state.gtick > recordedTick && state.gtick <= maxTicks) {
      const t = kgRef.current.snapshot(state);
      setRecordedTick(t);
    }
  }, [ready, autoSnapshot, state, recordedTick, maxTicks]);

  const api = useMemo(() => ({
    kg: kgRef.current,
    ready,
    error: err,
    tick: recordedTick,
    backend: kgRef.current?.kind,
    query: (sparql) => kgRef.current?.query(sparql),
    hubHistory: (id) => kgRef.current?.hubHistory(id),
    hotspots: () => kgRef.current?.disruptionHotspots(),
    snapshotNow: () => kgRef.current?.snapshot(state),
  }), [ready, err, recordedTick, state]);

  return api;
}

/* ============================================================================
 * Standalone (non-React) usage example
 * ==========================================================================*/

export async function demo(getState) {
  const kg = await KnowledgeGraph4D.open();
  console.log('4D backend:', kg.kind);
  // Feed it a few ticks of state (getState() returns your platform state object).
  for (let i = 0; i < 5; i++) kg.snapshot(getState());
  console.log('quads:', kg.size);
  console.log('hotspots:', await kg.disruptionHotspots());
  return kg;
}

export default KnowledgeGraph4D;

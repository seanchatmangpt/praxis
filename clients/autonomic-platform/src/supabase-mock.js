/**
 * supabase-mock.js
 * -----------------------------------------------------------------------------
 * A dependency-free, in-browser MOCK of the @supabase/supabase-js v2 client.
 * It mirrors the real API surface closely enough that you can develop the entire
 * app against it and later swap in `import { createClient } from '@supabase/supabase-js'`
 * with (mostly) no call-site changes.
 *
 * Capabilities covered:
 *   • auth        — getSession/getUser, signUp, signInWithPassword,
 *                   signInWithOAuth, signInWithOtp, signOut, onAuthStateChange
 *   • database    — from(table) PostgREST-style builder: select/insert/update/
 *                   upsert/delete + eq/neq/gt/gte/lt/lte/like/ilike/in/is/order/
 *                   limit/range/single/maybeSingle, count, thenable (await)
 *   • rpc         — supabase.rpc(fn, args)  (Postgres functions)
 *   • realtime    — supabase.channel(name).on('postgres_changes'|'broadcast'|
 *                   'presence', ...).subscribe(); track/presenceState/send
 *   • storage     — supabase.storage.from(bucket): upload/download/list/remove/
 *                   getPublicUrl/createSignedUrl; listBuckets/createBucket
 *   • functions   — supabase.functions.invoke(name, { body })  (Edge Functions)
 *
 * A built-in simulation loop continuously mutates the database (agent telemetry,
 * new incidents, model deploys) so realtime subscriptions feel alive with no
 * backend. Disable with createClient(url, key, { mock: { simulate: false } }).
 * -----------------------------------------------------------------------------
 */

/* ============================================================================
 * Tiny utilities
 * ==========================================================================*/

const nowIso = () => new Date().toISOString();
const uid = (p = 'id') => `${p}_${Math.random().toString(36).slice(2, 10)}`;
const clone = (v) => (typeof structuredClone === 'function' ? structuredClone(v) : JSON.parse(JSON.stringify(v)));
const ok = (data, extra = {}) => ({ data, error: null, status: 200, statusText: 'OK', ...extra });
const fail = (message, code = '400') => ({ data: null, error: { message, code, name: 'MockError' }, status: Number(code) || 400, statusText: 'Error' });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const rand = (n) => Math.floor(Math.random() * n);
const pick = (arr) => arr[rand(arr.length)];

/* ============================================================================
 * Seed data — the Ten Four fleet-logistics domain as Postgres tables
 * ---------------------------------------------------------------------------
 * Ten Four ("10-4 — message received") is a freight operations platform for
 * carriers. The same data is consumed by HUMAN users (drivers + dispatchers)
 * and by AI agents over MCP / A2A, so the row shapes are deliberately flat and
 * tool-call friendly.
 * ==========================================================================*/

function seedDatabase() {
  // power units
  const trucks = [
    { id: 'T-117', unit: '117', make: 'Freightliner', model: 'Cascadia', year: 2023, equipment: 'reefer', status: 'rolling', odometer: 318422, fuel_pct: 64, location: 'I-80 — Wendover, UT' },
    { id: 'T-203', unit: '203', make: 'Kenworth', model: 'T680', year: 2022, equipment: 'dry_van', status: 'rolling', odometer: 442109, fuel_pct: 41, location: 'I-40 — Amarillo, TX' },
    { id: 'T-205', unit: '205', make: 'Peterbilt', model: '579', year: 2024, equipment: 'flatbed', status: 'available', odometer: 96755, fuel_pct: 88, location: 'Laredo, TX yard' },
    { id: 'T-219', unit: '219', make: 'Volvo', model: 'VNL 860', year: 2023, equipment: 'reefer', status: 'maintenance', odometer: 511980, fuel_pct: 52, location: 'Phoenix, AZ shop' },
    { id: 'T-244', unit: '244', make: 'Freightliner', model: 'Cascadia', year: 2024, equipment: 'dry_van', status: 'available', odometer: 61240, fuel_pct: 93, location: 'Joliet, IL yard' },
    { id: 'T-260', unit: '260', make: 'Mack', model: 'Anthem', year: 2021, equipment: 'flatbed', status: 'idle', odometer: 689301, fuel_pct: 30, location: 'Atlanta, GA yard' },
  ];
  // driver roster (HOS in remaining minutes of the 11h drive clock)
  const drivers = [
    { id: 'D-1042', name: 'Marcus Hale', cdl_class: 'A', status: 'driving', hos_remaining: 214, home_terminal: 'Joliet, IL', truck_id: 'T-117', mpg: 7.8, rating: 4.9, loads_done: 612 },
    { id: 'D-1188', name: 'Dana Whitfield', cdl_class: 'A', status: 'on_duty', hos_remaining: 388, home_terminal: 'Laredo, TX', truck_id: 'T-203', mpg: 7.1, rating: 4.7, loads_done: 433 },
    { id: 'D-1206', name: 'Theo Park', cdl_class: 'A', status: 'off_duty', hos_remaining: 660, home_terminal: 'Phoenix, AZ', truck_id: 'T-205', mpg: 6.9, rating: 4.8, loads_done: 277 },
    { id: 'D-1233', name: 'Renata Cruz', cdl_class: 'A', status: 'sleeper', hos_remaining: 95, home_terminal: 'Atlanta, GA', truck_id: 'T-219', mpg: 8.2, rating: 5.0, loads_done: 854 },
    { id: 'D-1290', name: 'Sam Okafor', cdl_class: 'A', status: 'driving', hos_remaining: 142, home_terminal: 'Joliet, IL', truck_id: 'T-244', mpg: 7.5, rating: 4.6, loads_done: 198 },
  ];
  // load board
  const equip = ['reefer', 'dry_van', 'flatbed'];
  const loads = [
    { id: 'L-88021', ref: 'TF-88021', origin: 'Laredo, TX', dest: 'Joliet, IL', miles: 1187, rate: 3088, rpm: 2.60, commodity: 'Produce', equipment: 'reefer', weight: 42000, status: 'available', broker: 'Sunbelt Brokerage', pickup_at: '2026-06-15T14:00:00Z', deliver_by: '2026-06-17T11:00:00Z' },
    { id: 'L-88044', ref: 'TF-88044', origin: 'Joliet, IL', dest: 'Atlanta, GA', miles: 716, rate: 1646, rpm: 2.30, commodity: 'Packaged goods', equipment: 'dry_van', weight: 38000, status: 'available', broker: 'Keystone Logistics', pickup_at: '2026-06-15T18:00:00Z', deliver_by: '2026-06-16T20:00:00Z' },
    { id: 'L-88052', ref: 'TF-88052', origin: 'Phoenix, AZ', dest: 'Laredo, TX', miles: 928, rate: 1762, rpm: 1.90, commodity: 'Steel coil', equipment: 'flatbed', weight: 47500, status: 'available', broker: 'Rio Grande Freight', pickup_at: '2026-06-16T09:00:00Z', deliver_by: '2026-06-17T17:00:00Z' },
    { id: 'L-88060', ref: 'TF-88060', origin: 'Atlanta, GA', dest: 'Dallas, TX', miles: 781, rate: 1953, rpm: 2.50, commodity: 'Frozen foods', equipment: 'reefer', weight: 41000, status: 'booked', broker: 'Sunbelt Brokerage', pickup_at: '2026-06-15T22:00:00Z', deliver_by: '2026-06-17T08:00:00Z' },
    { id: 'L-88077', ref: 'TF-88077', origin: 'Joliet, IL', dest: 'Denver, CO', miles: 1003, rate: 2107, rpm: 2.10, commodity: 'Auto parts', equipment: 'dry_van', weight: 36000, status: 'in_transit', broker: 'Keystone Logistics', pickup_at: '2026-06-14T12:00:00Z', deliver_by: '2026-06-16T10:00:00Z' },
    { id: 'L-88091', ref: 'TF-88091', origin: 'Dallas, TX', dest: 'Phoenix, AZ', miles: 1067, rate: 2347, rpm: 2.20, commodity: 'Building materials', equipment: 'flatbed', weight: 46000, status: 'available', broker: 'Rio Grande Freight', pickup_at: '2026-06-16T15:00:00Z', deliver_by: '2026-06-18T12:00:00Z' },
  ];
  // active trips / runs
  const runs = [
    { id: uid('run'), load_ref: 'TF-88077', driver_id: 'D-1042', truck_id: 'T-117', status: 'rolling', progress: 68, miles_remaining: 321, eta: '2026-06-16T09:40:00Z', next_stop: 'Fuel — Wendover, UT' },
    { id: uid('run'), load_ref: 'TF-88060', driver_id: 'D-1233', truck_id: 'T-219', status: 'planned', progress: 0, miles_remaining: 781, eta: '2026-06-17T07:30:00Z', next_stop: 'Pickup — Atlanta, GA' },
    { id: uid('run'), load_ref: 'TF-99002', driver_id: 'D-1188', truck_id: 'T-203', status: 'arrived', progress: 100, miles_remaining: 0, eta: 'now', next_stop: 'Receiver — Amarillo, TX' },
  ];
  // market lanes
  const lanes = [
    { id: uid('ln'), origin: 'Laredo, TX', dest: 'Joliet, IL', miles: 1187, avg_rpm: 2.58, volume: 412, trend: 'up' },
    { id: uid('ln'), origin: 'Joliet, IL', dest: 'Atlanta, GA', miles: 716, avg_rpm: 2.28, volume: 287, trend: 'flat' },
    { id: uid('ln'), origin: 'Phoenix, AZ', dest: 'Laredo, TX', miles: 928, avg_rpm: 1.92, volume: 153, trend: 'down' },
    { id: uid('ln'), origin: 'Atlanta, GA', dest: 'Dallas, TX', miles: 781, avg_rpm: 2.49, volume: 341, trend: 'up' },
  ];
  // driver settlements (pay)
  const settlements = [
    { id: uid('set'), driver_id: 'D-1042', period: 'Wk 24', miles: 3140, gross: 7536, deductions: 612, net: 6924, status: 'paid' },
    { id: uid('set'), driver_id: 'D-1188', period: 'Wk 24', miles: 2884, gross: 6634, deductions: 540, net: 6094, status: 'processing' },
    { id: uid('set'), driver_id: 'D-1233', period: 'Wk 24', miles: 3402, gross: 8505, deductions: 705, net: 7800, status: 'open' },
  ];
  // dispatch messages
  const messages = [
    { id: uid('msg'), from_handle: 'dispatch', to_handle: 'D-1042', channel: 'load', body: 'TF-88077 detention approved — 2 hrs at receiver.', urgent: false, created_at: nowIso() },
    { id: uid('msg'), from_handle: 'D-1233', to_handle: 'dispatch', channel: 'load', body: 'Reefer pre-cool to -10°F confirmed for TF-88060.', urgent: false, created_at: nowIso() },
    { id: uid('msg'), from_handle: 'safety', to_handle: 'all', channel: 'broadcast', body: 'Winter advisory I-80 Wyoming — chain law in effect.', urgent: true, created_at: nowIso() },
  ];
  // DVIR inspections
  const inspections = [
    { id: uid('insp'), truck_id: 'T-117', driver_id: 'D-1042', kind: 'pre_trip', result: 'pass', defects: 0, created_at: nowIso() },
    { id: uid('insp'), truck_id: 'T-219', driver_id: 'D-1233', kind: 'pre_trip', result: 'defect', defects: 2, created_at: nowIso() },
    { id: uid('insp'), truck_id: 'T-203', driver_id: 'D-1188', kind: 'post_trip', result: 'pass', defects: 0, created_at: nowIso() },
  ];
  // fuel stops / receipts
  const fuel_stops = [
    { id: uid('fs'), truck_id: 'T-117', driver_id: 'D-1042', station: 'Loves #412', gallons: 148.2, price: 3.94, total: 583.91, location: 'Wendover, UT', created_at: nowIso() },
    { id: uid('fs'), truck_id: 'T-203', driver_id: 'D-1188', station: 'Pilot #88', gallons: 132.6, price: 3.81, total: 505.21, location: 'Amarillo, TX', created_at: nowIso() },
  ];
  // brokers / shippers
  const customers = [
    { id: uid('cus'), name: 'Sunbelt Brokerage', kind: 'broker', credit: 'A', loads: 142, on_time_pct: 97 },
    { id: uid('cus'), name: 'Keystone Logistics', kind: 'broker', credit: 'A-', loads: 98, on_time_pct: 94 },
    { id: uid('cus'), name: 'Rio Grande Freight', kind: 'shipper', credit: 'B+', loads: 61, on_time_pct: 89 },
  ];
  // AI agents operating over MCP / A2A (consumed via the Vercel AI SDK)
  const agents = [
    { id: 0, name: 'DISPATCHER-AI', role: 'Load matching', protocol: 'A2A', model: 'claude-sonnet', status: 'active', actions: 1284, color: '#FFB627' },
    { id: 1, name: 'ROUTE-OPTIMIZER', role: 'Routing & fuel', protocol: 'MCP', model: 'claude-sonnet', status: 'active', actions: 902, color: '#2EC4B6' },
    { id: 2, name: 'ETA-PREDICTOR', role: 'Arrival forecast', protocol: 'MCP', model: 'claude-haiku', status: 'active', actions: 3471, color: '#3A86FF' },
    { id: 3, name: 'RATE-NEGOTIATOR', role: 'Spot pricing', protocol: 'A2A', model: 'claude-sonnet', status: 'idle', actions: 188, color: '#E76F51' },
    { id: 4, name: 'COMPLIANCE-BOT', role: 'HOS & DVIR', protocol: 'MCP', model: 'claude-haiku', status: 'active', actions: 661, color: '#8338EC' },
    { id: 5, name: 'SETTLEMENT-AI', role: 'Driver pay', protocol: 'A2A', model: 'claude-sonnet', status: 'idle', actions: 74, color: '#06A77D' },
  ];
  // platform users
  const profiles = [
    { id: 'u_demo', handle: 'dispatch', display_name: 'River Mason', role: 'dispatcher', terminal: 'Joliet, IL', loads_done: 0, rating: 4.9 },
    { id: 'u_marcus', handle: 'mhale', display_name: 'Marcus Hale', role: 'driver', terminal: 'Joliet, IL', loads_done: 612, rating: 4.9 },
    { id: 'u_renata', handle: 'rcruz', display_name: 'Renata Cruz', role: 'driver', terminal: 'Atlanta, GA', loads_done: 854, rating: 5.0 },
    { id: 'u_admin', handle: 'fleetadmin', display_name: 'Priya Nair', role: 'admin', terminal: 'HQ', loads_done: 0, rating: 5.0 },
  ];

  return { trucks, drivers, loads, runs, lanes, settlements, messages, inspections, fuel_stops, customers, agents, profiles };
}

// Storage buckets seeded with a few "files".
function seedStorage() {
  return {
    'pod-documents': [
      { name: 'TF-88077-bol.pdf', id: uid('f'), size: 184_000, mimetype: 'application/pdf', created_at: nowIso() },
      { name: 'TF-88060-ratecon.pdf', id: uid('f'), size: 142_000, mimetype: 'application/pdf', created_at: nowIso() },
    ],
    'fuel-receipts': [
      { name: 'loves-412-0615.jpg', id: uid('f'), size: 512_000, mimetype: 'image/jpeg', created_at: nowIso() },
      { name: 'pilot-88-0615.jpg', id: uid('f'), size: 488_000, mimetype: 'image/jpeg', created_at: nowIso() },
    ],
    'avatars': [{ name: 'mhale.png', id: uid('f'), size: 64_000, mimetype: 'image/png', created_at: nowIso() }],
  };
}

/* ============================================================================
 * Edge Functions + RPC (Postgres functions) registry
 * ==========================================================================*/

function buildFunctions(db) {
  // Edge Functions — invoked via supabase.functions.invoke(name, { body }).
  // These are the WRITE actions, callable identically by a human tapping a
  // button or by an AI agent over MCP / A2A through the Vercel AI SDK.
  const edge = {
    // a driver/dispatcher books an available load → creates a planned run
    'book-load': async ({ load_id, driver_id }) => {
      const l = db.loads.find((x) => x.id === load_id);
      if (!l) return fail('load not found', '404');
      if (l.status !== 'available') return fail('load no longer available', '409');
      const old = { ...l };
      l.status = 'booked';
      db._emit('loads', 'UPDATE', l, old);
      const drv = db.drivers.find((d) => d.id === driver_id);
      const run = { id: uid('run'), load_ref: l.ref, driver_id: driver_id || null, truck_id: drv ? drv.truck_id : null, status: 'planned', progress: 0, miles_remaining: l.miles, eta: l.deliver_by, next_stop: `Pickup — ${l.origin}` };
      db.runs.unshift(run);
      db._emit('runs', 'INSERT', run);
      return ok({ booked: true, load_ref: l.ref, run_id: run.id, rate: l.rate });
    },
    // dispatcher assigns a load to a specific driver + truck
    'dispatch-load': async ({ load_id, driver_id, truck_id }) => {
      const l = db.loads.find((x) => x.id === load_id);
      if (!l) return fail('load not found', '404');
      const old = { ...l };
      l.status = 'in_transit';
      db._emit('loads', 'UPDATE', l, old);
      const run = { id: uid('run'), load_ref: l.ref, driver_id, truck_id, status: 'rolling', progress: 1, miles_remaining: l.miles, eta: l.deliver_by, next_stop: `Pickup — ${l.origin}` };
      db.runs.unshift(run);
      db._emit('runs', 'INSERT', run);
      return ok({ dispatched: true, run_id: run.id });
    },
    // recompute progress + ETA on an active run (AI ETA-PREDICTOR or manual)
    'update-eta': async ({ run_id, advance = 12 }) => {
      const r = db.runs.find((x) => x.id === run_id);
      if (!r) return fail('run not found', '404');
      const old = { ...r };
      r.progress = Math.min(100, r.progress + advance);
      r.miles_remaining = Math.max(0, Math.round(r.miles_remaining * (1 - advance / 100)));
      if (r.progress >= 100) { r.status = 'arrived'; r.miles_remaining = 0; r.next_stop = 'At receiver'; }
      else r.status = 'rolling';
      db._emit('runs', 'UPDATE', r, old);
      return ok({ progress: r.progress, miles_remaining: r.miles_remaining, status: r.status });
    },
    // mark a rolling run delivered → closes the load
    'complete-run': async ({ run_id }) => {
      const r = db.runs.find((x) => x.id === run_id);
      if (!r) return fail('run not found', '404');
      const old = { ...r };
      r.status = 'delivered'; r.progress = 100; r.miles_remaining = 0; r.next_stop = 'Delivered';
      db._emit('runs', 'UPDATE', r, old);
      const l = db.loads.find((x) => x.ref === r.load_ref);
      if (l) { const lo = { ...l }; l.status = 'delivered'; db._emit('loads', 'UPDATE', l, lo); }
      return ok({ delivered: true, load_ref: r.load_ref });
    },
    // log a fuel stop / receipt
    'log-fuel': async ({ truck_id, driver_id, gallons = 120, price = 3.89, station = 'TA Travel Center', location = 'En route' }) => {
      const total = +(gallons * price).toFixed(2);
      const rec = { id: uid('fs'), truck_id, driver_id, station, gallons, price, total, location, created_at: nowIso() };
      db.fuel_stops.unshift(rec);
      db._emit('fuel_stops', 'INSERT', rec);
      const t = db.trucks.find((x) => x.id === truck_id);
      if (t) { const old = { ...t }; t.fuel_pct = Math.min(100, t.fuel_pct + 40); db._emit('trucks', 'UPDATE', t, old); }
      return ok({ logged: true, total, fuel_pct: t ? t.fuel_pct : null });
    },
    // submit a DVIR pre/post-trip inspection
    'submit-dvir': async ({ truck_id, driver_id, kind = 'pre_trip', defects = 0 }) => {
      const rec = { id: uid('insp'), truck_id, driver_id, kind, result: defects > 0 ? 'defect' : 'pass', defects, created_at: nowIso() };
      db.inspections.unshift(rec);
      db._emit('inspections', 'INSERT', rec);
      if (defects > 0) { const t = db.trucks.find((x) => x.id === truck_id); if (t) { const old = { ...t }; t.status = 'maintenance'; db._emit('trucks', 'UPDATE', t, old); } }
      return ok({ filed: true, result: rec.result, defects });
    },
    // send a dispatch message
    'send-message': async ({ from_handle = 'dispatch', to_handle, channel = 'load', body, urgent = false }) => {
      const rec = { id: uid('msg'), from_handle, to_handle, channel, body, urgent, created_at: nowIso() };
      db.messages.unshift(rec);
      db._emit('messages', 'INSERT', rec);
      return ok({ sent: true, id: rec.id });
    },
    // AI ROUTE-OPTIMIZER — trims miles + reports fuel saved on a run
    'optimize-route': async ({ run_id }) => {
      const r = db.runs.find((x) => x.id === run_id);
      if (!r) return fail('run not found', '404');
      const old = { ...r };
      const saved = 20 + rand(60);
      r.miles_remaining = Math.max(0, r.miles_remaining - saved);
      db._emit('runs', 'UPDATE', r, old);
      return ok({ optimized: true, miles_saved: saved, fuel_saved_gal: +(saved / 7).toFixed(1) });
    },
    // generic AI action sink (MCP / A2A) — bumps an agent's action counter
    'agent-act': async ({ agent_id, action = 'tool_call' }) => {
      const a = db.agents.find((x) => x.id === agent_id);
      if (!a) return fail('agent not found', '404');
      const old = { ...a };
      a.actions += 1; a.status = 'active';
      db._emit('agents', 'UPDATE', a, old);
      return ok({ ok: true, agent: a.name, actions: a.actions, action });
    },
  };
  // Postgres RPC — invoked via supabase.rpc(name, args). READ / aggregate side.
  const rpc = {
    // fleet-wide health snapshot
    fleet_health: () => {
      const rolling = db.runs.filter((r) => r.status === 'rolling').length;
      const available = db.trucks.filter((t) => t.status === 'available').length;
      const maint = db.trucks.filter((t) => t.status === 'maintenance').length;
      const openLoads = db.loads.filter((l) => l.status === 'available').length;
      return ok({ trucks_rolling: rolling, trucks_available: available, trucks_maintenance: maint, open_loads: openLoads, on_time_pct: +(96 - maint * 1.5).toFixed(1) });
    },
    // remaining hours of service for a driver (11h drive clock)
    driver_hos: ({ driver_id }) => {
      const d = db.drivers.find((x) => x.id === driver_id);
      if (!d) return fail('driver not found', '404');
      return ok({ driver: d.name, status: d.status, drive_remaining_min: d.hos_remaining, drive_remaining_hrs: +(d.hos_remaining / 60).toFixed(1), violation_risk: d.hos_remaining < 120 });
    },
    // market rate for an origin→dest lane
    lane_rate: ({ origin, dest }) => {
      const l = db.lanes.find((x) => x.origin === origin && x.dest === dest);
      if (!l) return ok({ found: false, avg_rpm: null });
      return ok({ found: true, origin, dest, miles: l.miles, avg_rpm: l.avg_rpm, est_total: +(l.avg_rpm * l.miles).toFixed(0), trend: l.trend });
    },
    // settlement summary for a driver
    driver_pay_summary: ({ driver_id }) => {
      const rows = db.settlements.filter((s) => s.driver_id === driver_id);
      const gross = rows.reduce((a, s) => a + s.gross, 0);
      const net = rows.reduce((a, s) => a + s.net, 0);
      const miles = rows.reduce((a, s) => a + s.miles, 0);
      return ok({ driver_id, periods: rows.length, gross, net, miles, rpm: miles ? +(gross / miles).toFixed(2) : 0 });
    },
    // top lanes ranked by rate-per-mile
    rank_lanes: ({ limit = 5 } = {}) => {
      const ranked = [...db.lanes].sort((a, b) => b.avg_rpm - a.avg_rpm).slice(0, limit)
        .map((l, i) => ({ rank: i + 1, lane: `${l.origin} → ${l.dest}`, avg_rpm: l.avg_rpm, volume: l.volume }));
      return ok({ lanes: ranked });
    },
    // suggest the best available truck for an equipment type
    assign_best_truck: ({ equipment }) => {
      const candidates = db.trucks.filter((t) => t.status === 'available' && (!equipment || t.equipment === equipment))
        .sort((a, b) => b.fuel_pct - a.fuel_pct);
      const best = candidates[0];
      return best ? ok({ truck_id: best.id, unit: best.unit, equipment: best.equipment, fuel_pct: best.fuel_pct }) : ok({ truck_id: null, reason: 'no available unit' });
    },
  };
  return { edge, rpc };
}

/* ============================================================================
 * Realtime bus — channels, postgres_changes, broadcast, presence
 * ==========================================================================*/

class RealtimeChannel {
  constructor(name, bus) {
    this.name = name;
    this.bus = bus;
    this.state = 'closed';
    this._listeners = []; // { kind, filter, cb }
    this._presence = {};   // key -> [states]
    this._presenceCbs = { sync: [], join: [], leave: [] };
  }
  on(kind, filterOrCb, maybeCb) {
    if (kind === 'presence') {
      const ev = (filterOrCb && filterOrCb.event) || 'sync';
      (this._presenceCbs[ev] = this._presenceCbs[ev] || []).push(maybeCb);
      return this;
    }
    const filter = typeof filterOrCb === 'object' ? filterOrCb : {};
    const cb = maybeCb || filterOrCb;
    this._listeners.push({ kind, filter, cb });
    return this;
  }
  subscribe(cb) {
    this.state = 'joined';
    this.bus._channels.add(this);
    cb && cb('SUBSCRIBED');
    return this;
  }
  // Broadcast send
  send({ type, event, payload }) {
    if (type === 'broadcast') this.bus._deliverBroadcast(this.name, event, payload);
    return Promise.resolve('ok');
  }
  // Presence
  track(stateObj) {
    const key = stateObj.key || uid('presence');
    this._presence[key] = [{ ...stateObj, online_at: nowIso() }];
    this._firePresence('join', key);
    this._firePresence('sync');
    return Promise.resolve('ok');
  }
  untrack() {
    const keys = Object.keys(this._presence);
    this._presence = {};
    keys.forEach((k) => this._firePresence('leave', k));
    this._firePresence('sync');
    return Promise.resolve('ok');
  }
  presenceState() { return clone(this._presence); }
  _firePresence(ev, key) { (this._presenceCbs[ev] || []).forEach((cb) => cb({ key })); }
  unsubscribe() { this.state = 'closed'; this.bus._channels.delete(this); return Promise.resolve('ok'); }
}

class RealtimeBus {
  constructor() { this._channels = new Set(); }
  channel(name) { return new RealtimeChannel(name, this); }
  removeChannel(ch) { return ch.unsubscribe(); }
  getChannels() { return [...this._channels]; }
  // DB change fan-out
  emitPostgresChange(table, eventType, row, oldRow) {
    const payload = {
      schema: 'public', table, eventType, commit_timestamp: nowIso(),
      new: eventType === 'DELETE' ? {} : clone(row),
      old: eventType === 'INSERT' ? {} : clone(oldRow || row),
    };
    this._channels.forEach((ch) => {
      ch._listeners.forEach((l) => {
        if (l.kind !== 'postgres_changes') return;
        const f = l.filter || {};
        if (f.table && f.table !== table) return;
        if (f.event && f.event !== '*' && f.event !== eventType) return;
        if (f.filter && !matchEqFilter(f.filter, payload.new, payload.old)) return;
        l.cb(payload);
      });
    });
  }
  _deliverBroadcast(name, event, payload) {
    this._channels.forEach((ch) => {
      if (ch.name !== name) return;
      ch._listeners.forEach((l) => { if (l.kind === 'broadcast' && (!l.filter.event || l.filter.event === event)) l.cb({ event, payload }); });
    });
  }
}

function matchEqFilter(filterStr, newRow, oldRow) {
  // supports "col=eq.value"
  const m = /^(\w+)=eq\.(.+)$/.exec(filterStr);
  if (!m) return true;
  const [, col, val] = m;
  const row = newRow && Object.keys(newRow).length ? newRow : oldRow;
  return row && String(row[col]) === val;
}

/* ============================================================================
 * Database — PostgREST-style query builder (thenable)
 * ==========================================================================*/

const FILTER_OPS = {
  eq: (a, b) => a === b || String(a) === String(b),
  neq: (a, b) => String(a) !== String(b),
  gt: (a, b) => a > b,
  gte: (a, b) => a >= b,
  lt: (a, b) => a < b,
  lte: (a, b) => a <= b,
  like: (a, b) => new RegExp('^' + String(b).replace(/%/g, '.*') + '$').test(String(a)),
  ilike: (a, b) => new RegExp('^' + String(b).replace(/%/g, '.*') + '$', 'i').test(String(a)),
  in: (a, b) => b.includes(a),
  is: (a, b) => (b === null ? a == null : a === b),
};

class QueryBuilder {
  constructor(db, table) {
    this.db = db;
    this.table = table;
    this._op = 'select';
    this._columns = '*';
    this._filters = [];
    this._order = null;
    this._limit = null;
    this._range = null;
    this._payload = null;
    this._single = false;
    this._maybeSingle = false;
    this._wantCount = null;
    this._returning = true;
  }
  select(columns = '*', opts = {}) {
    if (this._op === 'select') this._op = 'select';
    this._columns = columns;
    if (opts.count) this._wantCount = opts.count;
    this._didSelect = true;
    return this;
  }
  insert(payload, opts = {}) { this._op = 'insert'; this._payload = payload; this._returning = opts.returning !== 'minimal'; return this; }
  upsert(payload, opts = {}) { this._op = 'upsert'; this._payload = payload; this._onConflict = opts.onConflict || 'id'; return this; }
  update(payload) { this._op = 'update'; this._payload = payload; return this; }
  delete() { this._op = 'delete'; return this; }
  // filters
  _addFilter(op, column, value) { this._filters.push({ op, column, value }); return this; }
  eq(c, v) { return this._addFilter('eq', c, v); }
  neq(c, v) { return this._addFilter('neq', c, v); }
  gt(c, v) { return this._addFilter('gt', c, v); }
  gte(c, v) { return this._addFilter('gte', c, v); }
  lt(c, v) { return this._addFilter('lt', c, v); }
  lte(c, v) { return this._addFilter('lte', c, v); }
  like(c, v) { return this._addFilter('like', c, v); }
  ilike(c, v) { return this._addFilter('ilike', c, v); }
  in(c, v) { return this._addFilter('in', c, v); }
  is(c, v) { return this._addFilter('is', c, v); }
  match(obj) { Object.entries(obj).forEach(([c, v]) => this._addFilter('eq', c, v)); return this; }
  order(column, { ascending = true } = {}) { this._order = { column, ascending }; return this; }
  limit(n) { this._limit = n; return this; }
  range(from, to) { this._range = [from, to]; return this; }
  single() { this._single = true; return this; }
  maybeSingle() { this._maybeSingle = true; return this; }

  _applyFilters(rows) {
    return rows.filter((r) => this._filters.every((f) => (FILTER_OPS[f.op] || FILTER_OPS.eq)(r[f.column], f.value)));
  }

  _run() {
    const db = this.db;
    if (!db.tables[this.table]) return fail(`relation "${this.table}" does not exist`, '404');
    let rows = db.tables[this.table];

    if (this._op === 'insert' || this._op === 'upsert') {
      const incoming = Array.isArray(this._payload) ? this._payload : [this._payload];
      const inserted = [];
      incoming.forEach((row) => {
        const rec = { ...row };
        if (rec.id == null) rec.id = uid(this.table.slice(0, 3));
        if (this._op === 'upsert') {
          const key = this._onConflict || 'id';
          const existing = rows.findIndex((r) => r[key] === rec[key]);
          if (existing >= 0) { const old = rows[existing]; rows[existing] = { ...old, ...rec }; inserted.push(rows[existing]); db._emit(this.table, 'UPDATE', rows[existing], old); return; }
        }
        if (rec.created_at == null) rec.created_at = nowIso();
        rows.push(rec); inserted.push(rec); db._emit(this.table, 'INSERT', rec);
      });
      const data = this._returning ? clone(inserted) : null;
      return ok(this._single ? data && data[0] : data);
    }

    if (this._op === 'update') {
      const targets = this._applyFilters(rows);
      const updated = targets.map((r) => { const old = { ...r }; Object.assign(r, this._payload); db._emit(this.table, 'UPDATE', r, old); return r; });
      return ok(this._single ? clone(updated[0]) : clone(updated));
    }

    if (this._op === 'delete') {
      const targets = this._applyFilters(rows);
      db.tables[this.table] = rows.filter((r) => !targets.includes(r));
      targets.forEach((r) => db._emit(this.table, 'DELETE', r));
      return ok(clone(targets));
    }

    // select
    let out = this._applyFilters(rows);
    const count = out.length;
    if (this._order) {
      const { column, ascending } = this._order;
      out = [...out].sort((a, b) => (a[column] > b[column] ? 1 : a[column] < b[column] ? -1 : 0) * (ascending ? 1 : -1));
    }
    if (this._range) out = out.slice(this._range[0], this._range[1] + 1);
    if (this._limit != null) out = out.slice(0, this._limit);
    out = clone(out);
    if (this._single) {
      if (out.length !== 1) return fail('JSON object requested, multiple (or no) rows returned', '406');
      return ok(out[0], { count: this._wantCount ? count : null });
    }
    if (this._maybeSingle) return ok(out[0] ?? null, { count: this._wantCount ? count : null });
    return ok(out, { count: this._wantCount ? count : null });
  }

  // thenable — `await supabase.from(...).select()` resolves here
  then(resolve, reject) {
    return sleep(40 + rand(60))
      .then(() => this._run())
      .then(resolve, reject);
  }
}

class Database {
  constructor(bus, opts) {
    this.bus = bus;
    this.tables = seedDatabase();
    this.opts = opts;
  }
  from(table) { return new QueryBuilder(this, table); }
  _emit(table, type, row, oldRow) { this.bus.emitPostgresChange(table, type, row, oldRow); }
}

/* ============================================================================
 * Auth
 * ==========================================================================*/

const OAUTH_PROVIDERS = ['github', 'google', 'gitlab', 'azure', 'discord'];

class AuthClient {
  constructor() {
    this._session = null;
    this._listeners = [];
    this._users = [{ id: 'u_demo', email: 'commander@autonomic.dev', user_metadata: { handle: 'commander' } }];
  }
  _mkSession(user) {
    return {
      access_token: 'mock.' + uid('jwt'),
      refresh_token: uid('refresh'),
      expires_at: Math.floor(Date.now() / 1000) + 3600,
      token_type: 'bearer',
      user,
    };
  }
  _set(session, event) {
    this._session = session;
    this._listeners.forEach((cb) => cb(event, session));
  }
  async getSession() { await sleep(20); return ok({ session: this._session }); }
  async getUser() { await sleep(20); return this._session ? ok({ user: this._session.user }) : fail('Auth session missing', '401'); }
  async signUp({ email, password, options }) {
    await sleep(120);
    if (!email || !password) return fail('email and password required', '422');
    const user = { id: uid('u'), email, user_metadata: options?.data || {} };
    this._users.push(user);
    const session = this._mkSession(user);
    this._set(session, 'SIGNED_IN');
    return ok({ user, session });
  }
  async signInWithPassword({ email, password }) {
    await sleep(120);
    if (!email || !password) return fail('Invalid login credentials', '400');
    const user = this._users.find((u) => u.email === email) || { id: 'u_demo', email, user_metadata: { handle: email.split('@')[0] } };
    const session = this._mkSession(user);
    this._set(session, 'SIGNED_IN');
    return ok({ user, session });
  }
  async signInWithOtp({ email }) { await sleep(120); return ok({ user: null, session: null, messageId: uid('otp') }); }
  async signInWithOAuth({ provider }) {
    await sleep(150);
    if (!OAUTH_PROVIDERS.includes(provider)) return fail(`provider ${provider} not enabled`, '400');
    const user = { id: uid('u'), email: `user@${provider}.oauth`, app_metadata: { provider }, user_metadata: { handle: `${provider}-user` } };
    const session = this._mkSession(user);
    this._set(session, 'SIGNED_IN');
    return ok({ provider, url: `https://mock.supabase.co/auth/v1/authorize?provider=${provider}`, session });
  }
  async updateUser(attrs) {
    if (!this._session) return fail('not authenticated', '401');
    Object.assign(this._session.user, attrs);
    this._set(this._session, 'USER_UPDATED');
    return ok({ user: this._session.user });
  }
  async signOut() { await sleep(60); this._set(null, 'SIGNED_OUT'); return ok(null); }
  onAuthStateChange(cb) {
    this._listeners.push(cb);
    // supabase fires INITIAL_SESSION async
    sleep(0).then(() => cb('INITIAL_SESSION', this._session));
    return { data: { subscription: { id: uid('sub'), unsubscribe: () => { this._listeners = this._listeners.filter((l) => l !== cb); } } } };
  }
  get providers() { return [...OAUTH_PROVIDERS]; }
}

/* ============================================================================
 * Storage
 * ==========================================================================*/

class StorageBucketApi {
  constructor(store, bucket) { this.store = store; this.bucket = bucket; }
  async upload(path, file, opts = {}) {
    await sleep(120);
    const files = (this.store[this.bucket] = this.store[this.bucket] || []);
    const size = file?.size ?? (typeof file === 'string' ? file.length : 100_000);
    const existing = files.find((f) => f.name === path);
    if (existing && !opts.upsert) return fail('The resource already exists', '409');
    const rec = { name: path, id: uid('f'), size, mimetype: file?.type || 'application/octet-stream', created_at: nowIso() };
    if (existing) Object.assign(existing, rec); else files.push(rec);
    return ok({ path, id: rec.id, fullPath: `${this.bucket}/${path}` });
  }
  async download(path) {
    await sleep(80);
    const f = (this.store[this.bucket] || []).find((x) => x.name === path);
    if (!f) return fail('Object not found', '404');
    return ok(new Blob([`mock-bytes:${path}`], { type: f.mimetype }));
  }
  async list(prefix = '') {
    await sleep(60);
    const files = (this.store[this.bucket] || []).filter((f) => f.name.startsWith(prefix));
    return ok(clone(files));
  }
  async remove(paths) {
    await sleep(80);
    const list = Array.isArray(paths) ? paths : [paths];
    this.store[this.bucket] = (this.store[this.bucket] || []).filter((f) => !list.includes(f.name));
    return ok(list.map((p) => ({ name: p })));
  }
  getPublicUrl(path) { return { data: { publicUrl: `https://mock.supabase.co/storage/v1/object/public/${this.bucket}/${path}` } }; }
  async createSignedUrl(path, expiresIn = 3600) {
    await sleep(40);
    return ok({ signedUrl: `https://mock.supabase.co/storage/v1/object/sign/${this.bucket}/${path}?token=${uid('tok')}&exp=${expiresIn}` });
  }
}

class StorageClient {
  constructor() { this._store = seedStorage(); }
  from(bucket) { return new StorageBucketApi(this._store, bucket); }
  async listBuckets() { await sleep(40); return ok(Object.keys(this._store).map((name) => ({ id: name, name, public: name !== 'avatars', created_at: nowIso() }))); }
  async createBucket(name, opts = {}) { this._store[name] = this._store[name] || []; return ok({ name }); }
  async getBucket(name) { return this._store[name] ? ok({ id: name, name }) : fail('Bucket not found', '404'); }
}

/* ============================================================================
 * Functions (Edge)
 * ==========================================================================*/

class FunctionsClient {
  constructor(registry) { this._fns = registry; }
  async invoke(name, { body } = {}) {
    await sleep(140);
    const fn = this._fns[name];
    if (!fn) return fail(`Function "${name}" not found`, '404');
    try { return await fn(body || {}); } catch (e) { return fail(e.message, '500'); }
  }
}

/* ============================================================================
 * Built-in simulation — keeps realtime alive
 * ==========================================================================*/

function startSimulation(db, bus) {
  // trucks roll: burn fuel, advance odometer, drift location
  const cities = ['Wendover, UT', 'Amarillo, TX', 'Flagstaff, AZ', 'Salina, KS', 'Effingham, IL', 'Sweetwater, TX', 'Barstow, CA', 'Big Spring, TX'];
  const trucksTouch = () => {
    const t = pick(db.tables.trucks);
    const old = { ...t };
    if (t.status === 'rolling') {
      t.odometer += 1 + rand(3);
      t.fuel_pct = Math.max(4, t.fuel_pct - (Math.random() < 0.5 ? 1 : 0));
      if (Math.random() < 0.15) t.location = `I-${pick(['80', '40', '10', '70', '35'])} — ${pick(cities)}`;
      db._emit('trucks', 'UPDATE', t, old);
    }
  };
  // active runs make progress toward delivery
  const runsTouch = () => {
    const r = db.tables.runs.find((x) => x.status === 'rolling');
    if (!r) return;
    const old = { ...r };
    r.progress = Math.min(100, r.progress + (1 + rand(3)));
    r.miles_remaining = Math.max(0, Math.round(r.miles_remaining * 0.97) - rand(5));
    if (r.progress >= 100 || r.miles_remaining <= 0) { r.status = 'arrived'; r.progress = 100; r.miles_remaining = 0; r.next_stop = 'At receiver'; }
    db._emit('runs', 'UPDATE', r, old);
  };
  // drivers burn down their HOS drive clock while driving
  const driversTouch = () => {
    const d = pick(db.tables.drivers);
    const old = { ...d };
    if (d.status === 'driving') {
      d.hos_remaining = Math.max(0, d.hos_remaining - (2 + rand(3)));
      if (d.hos_remaining === 0) d.status = 'off_duty';
      db._emit('drivers', 'UPDATE', d, old);
    } else if (Math.random() < 0.06) {
      const cyc = { off_duty: 'on_duty', on_duty: 'driving', driving: 'sleeper', sleeper: 'off_duty' };
      d.status = cyc[d.status] || 'on_duty';
      db._emit('drivers', 'UPDATE', d, old);
    }
  };
  // AI agents continuously act over MCP / A2A
  const agentsTouch = () => {
    const a = pick(db.tables.agents);
    const old = { ...a };
    if (Math.random() < 0.7) { a.actions += 1 + rand(4); a.status = 'active'; }
    else if (Math.random() < 0.2) a.status = a.status === 'active' ? 'idle' : 'active';
    db._emit('agents', 'UPDATE', a, old);
  };
  // occasional fresh load posts to the board, occasional dispatch broadcast
  const boardTitles = [
    ['Reefer', 'reefer', 'Produce'], ['Dry van', 'dry_van', 'Retail freight'],
    ['Flatbed', 'flatbed', 'Lumber'], ['Reefer', 'reefer', 'Dairy'], ['Dry van', 'dry_van', 'Paper goods'],
  ];
  const origins = ['Laredo, TX', 'Joliet, IL', 'Phoenix, AZ', 'Atlanta, GA', 'Dallas, TX', 'Denver, CO'];
  const maybeLoad = () => {
    if (Math.random() > 0.3) return;
    const [, equip, commodity] = pick(boardTitles);
    const o = pick(origins); let d = pick(origins); if (d === o) d = pick(origins);
    const miles = 400 + rand(900);
    const rpm = +(1.8 + Math.random() * 1.0).toFixed(2);
    const n = 88100 + rand(900);
    const rec = { id: 'L-' + n, ref: 'TF-' + n, origin: o, dest: d, miles, rate: Math.round(miles * rpm), rpm, commodity, equipment: equip, weight: 34000 + rand(14000), status: 'available', broker: pick(['Sunbelt Brokerage', 'Keystone Logistics', 'Rio Grande Freight']), pickup_at: nowIso(), deliver_by: nowIso() };
    db.tables.loads.unshift(rec);
    if (db.tables.loads.length > 30) db.tables.loads.pop();
    db._emit('loads', 'INSERT', rec);
  };
  const maybeMessage = () => {
    if (Math.random() > 0.2) return;
    const lines = ['Check call received — on schedule.', 'Detention clock started at receiver.', 'Lumper fee approved.', 'Scale ticket uploaded.', 'Appointment confirmed for 0800.'];
    const rec = { id: uid('msg'), from_handle: pick(['dispatch', 'D-1042', 'D-1188', 'safety']), to_handle: 'dispatch', channel: 'load', body: pick(lines), urgent: Math.random() < 0.15, created_at: nowIso() };
    db.tables.messages.unshift(rec);
    if (db.tables.messages.length > 40) db.tables.messages.pop();
    db._emit('messages', 'INSERT', rec);
  };
  const t1 = setInterval(trucksTouch, 1300);
  const t2 = setInterval(runsTouch, 1600);
  const t3 = setInterval(driversTouch, 1500);
  const t4 = setInterval(agentsTouch, 900);
  const t5 = setInterval(maybeLoad, 2400);
  const t6 = setInterval(maybeMessage, 2800);
  return () => { [t1, t2, t3, t4, t5, t6].forEach(clearInterval); };
}

/* ============================================================================
 * createClient — the public entry point (mirrors @supabase/supabase-js)
 * ==========================================================================*/

export function createClient(supabaseUrl = 'https://mock.supabase.co', supabaseKey = 'mock-anon-key', options = {}) {
  const bus = new RealtimeBus();
  const db = new Database(bus, options);
  const fnDb = { _emit: (t, type, row, old) => db._emit(t, type, row, old) };
  Object.keys(db.tables).forEach((name) => {
    Object.defineProperty(fnDb, name, { get: () => db.tables[name], enumerable: true });
  });
  const { edge, rpc } = buildFunctions(fnDb);

  const auth = new AuthClient();
  const storage = new StorageClient();
  const functions = new FunctionsClient(edge);

  let stopSim = null;
  if (options.mock?.simulate !== false) stopSim = startSimulation(db, bus);

  const client = {
    supabaseUrl,
    supabaseKey,
    auth,
    storage,
    functions,
    from: (table) => db.from(table),
    rpc: async (fn, args = {}) => { await sleep(60); const f = rpc[fn]; return f ? f(args) : fail(`function ${fn}() does not exist`, '404'); },
    channel: (name) => bus.channel(name),
    removeChannel: (ch) => bus.removeChannel(ch),
    removeAllChannels: () => { bus.getChannels().forEach((c) => c.unsubscribe()); return Promise.resolve('ok'); },
    getChannels: () => bus.getChannels(),
    // mock-only introspection helpers
    _mock: {
      db,
      bus,
      tables: () => db.tables,
      rpcNames: Object.keys(rpc),
      edgeNames: Object.keys(edge),
      stopSimulation: () => stopSim && stopSim(),
    },
  };
  return client;
}

export const MOCK_META = {
  tables: ['trucks', 'drivers', 'loads', 'runs', 'lanes', 'settlements', 'messages', 'inspections', 'fuel_stops', 'customers', 'agents', 'profiles'],
  buckets: ['pod-documents', 'fuel-receipts', 'avatars'],
  edgeFunctions: ['book-load', 'dispatch-load', 'update-eta', 'complete-run', 'log-fuel', 'submit-dvir', 'send-message', 'optimize-route', 'agent-act'],
  rpc: ['fleet_health', 'driver_hos', 'lane_rate', 'driver_pay_summary', 'rank_lanes', 'assign_best_truck'],
  oauthProviders: OAUTH_PROVIDERS,
};

export default createClient;

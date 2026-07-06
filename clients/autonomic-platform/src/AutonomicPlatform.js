/**
 * AutonomicPlatform.js
 * -----------------------------------------------------------------------------
 * React components, context, reducer and data model for the Autonomic Process
 * Intelligence Platform — extracted from the monolithic design prototype and the
 * FEATURE_SPEC.md "Combinatorial Maximalism" expansion.
 *
 * This is a developer-handoff module. It uses modern React (function components,
 * hooks, useReducer + Context). JSX is used throughout; build with Vite / CRA /
 * Babel (the file is named .js because that is the platform's convention — JSX in
 * .js compiles fine under those toolchains).
 *
 * Layout of this file:
 *   1. Constants & seed data        — pure data, no React
 *   2. createInitialState()         — the full game-state factory
 *   3. gameReducer()                — TICK simulation + all UI/meta actions
 *   4. GameContext + GameProvider   — runs the sim loop, exposes state/dispatch
 *   5. Hooks                        — useGame, useScreen, useDerived, useKeyboardNav
 *   6. Screen components            — HUD + Command/Arena/Deck/Ops/Leaderboards
 *
 * The 3D GLOBE (deck.gl) and ARENA (three.js) renderers are intentionally left as
 * mount-point components with documented lifecycle hooks; port the imperative
 * deck.gl / three.js code from the prototype into <GlobeCanvas/> and <ArenaCanvas/>.
 * -----------------------------------------------------------------------------
 */

import React, {
  createContext,
  useContext,
  useReducer,
  useEffect,
  useRef,
  useMemo,
  useCallback,
} from 'react';

// Praxis standing mode (adoption 2026-07-06): provider + standing-mapped
// DECK/OPS/HUD, shared NON-STANDING banner. See praxis-mode.js and README.md.
import {
  PraxisProvider,
  PraxisHud,
  PraxisDeckScreen,
  PraxisOpsScreen,
  PraxisCaseStudyScreen,
  NonStandingBanner,
} from './praxis-mode.js';

/* ============================================================================
 * 1. CONSTANTS & SEED DATA
 * ==========================================================================*/

export const PALETTE = {
  bg: '#05080f',
  panel: 'rgba(11,19,34,0.74)',
  panel2: 'rgba(14,23,40,0.92)',
  line: 'rgba(96,170,235,0.16)',
  line2: 'rgba(96,170,235,0.30)',
  cyan: '#33e1ff',
  emerald: '#34e6a8',
  amber: '#ffb13d',
  magenta: '#ff4d72',
  violet: '#a98bff',
  hi: '#e9f2ff',
  mid: '#93a7c6',
  dim: '#5b6f8c',
};

// Ordered nav / keyboard-shortcut screen list (1–9,0 map to first ten).
export const SCREENS = [
  'command', 'arena', 'deck', 'ops', 'agents', 'dod',
  'leaderboards', 'battlepass', 'prestige', 'talents',
  'raids', 'cosmetics', 'guilds', 'roguelike', 'casestudy',
];

export const SCREEN_META = {
  command:      { title: 'Global Command',   tag: 'COMMAND',   sub: 'Live planetary operations · trade arcs · self-healing hubs' },
  arena:        { title: 'Agent Arena',       tag: 'ARENA',     sub: '5v5 agent swarm vs disruption forces · real-time combat' },
  deck:         { title: 'Model Deck',        tag: 'DECK',      sub: 'AutoML models as collectible cards · synergies & combos' },
  ops:          { title: 'Operations',        tag: 'OPS',       sub: 'NPC operators · quests · incidents · achievements' },
  agents:       { title: 'Agent Roster',      tag: 'AGENTS',    sub: '10 archetypes · talents · live telemetry' },
  dod:          { title: 'Definition of Done',tag: 'DoD',       sub: 'Task pipeline · DoD checklists · gating' },
  leaderboards: { title: 'Leaderboards',      tag: 'RANKED',    sub: 'Global ELO · per-agent · per-model · seasonal' },
  battlepass:   { title: 'Battle Pass',       tag: 'SEASON',    sub: '100-tier track · daily & weekly challenges' },
  prestige:     { title: 'Prestige',          tag: 'ASCEND',    sub: 'Reset to level 1 · 2x XP multiplier · prestige rank' },
  talents:      { title: 'Talent Trees',      tag: 'TALENTS',   sub: 'Per-agent 9-talent unlock grids · loadouts' },
  raids:        { title: 'Raid Bosses',       tag: 'RAID',      sub: 'World threats · 3-phase co-op · threat waveforms' },
  cosmetics:    { title: 'Cosmetics Shop',    tag: 'SHOP',      sub: 'Agent skins · card backs · globe themes' },
  guilds:       { title: 'Guilds',            tag: 'GUILD',     sub: 'Roster · vault · guild wars · co-op quests' },
  roguelike:    { title: 'Endless Ops',       tag: 'ROGUE',     sub: 'Procedural campaigns · escalating modifiers · run stats' },
  casestudy:    { title: 'Standing Factory Case Study', tag: 'CASE STUDY', sub: 'GraphLaw verdict · SHACL/ShEx/N3/Datalog · PDDL/POWL · OCEL · wasm4pm' },
};

// Hub raw rows: [name, lng, lat, region, sector]
export const HUBS_RAW = [
  ['Los Angeles', -118.24, 34.05, 'AMER', 'Logistics'], ['New York', -74.0, 40.71, 'AMER', 'Finance'],
  ['Chicago', -87.63, 41.88, 'AMER', 'Manufacturing'], ['Houston', -95.37, 29.76, 'AMER', 'Energy'],
  ['Mexico City', -99.13, 19.43, 'AMER', 'Manufacturing'], ['São Paulo', -46.63, -23.55, 'AMER', 'Logistics'],
  ['Santiago', -70.66, -33.45, 'AMER', 'Energy'], ['Toronto', -79.38, 43.65, 'AMER', 'Finance'],
  ['London', -0.13, 51.51, 'EMEA', 'Finance'], ['Rotterdam', 4.48, 51.92, 'EMEA', 'Logistics'],
  ['Frankfurt', 8.68, 50.11, 'EMEA', 'Telecom'], ['Madrid', -3.70, 40.42, 'EMEA', 'Energy'],
  ['Lagos', 3.38, 6.52, 'EMEA', 'Energy'], ['Johannesburg', 28.04, -26.20, 'EMEA', 'Manufacturing'],
  ['Cairo', 31.24, 30.04, 'EMEA', 'Logistics'], ['Dubai', 55.27, 25.20, 'EMEA', 'Logistics'],
  ['Istanbul', 28.98, 41.01, 'EMEA', 'Telecom'], ['Moscow', 37.62, 55.75, 'EMEA', 'Energy'],
  ['Mumbai', 72.88, 19.08, 'APAC', 'Finance'], ['Bengaluru', 77.59, 12.97, 'APAC', 'Telecom'],
  ['Singapore', 103.82, 1.35, 'APAC', 'Logistics'], ['Bangkok', 100.50, 13.76, 'APAC', 'Manufacturing'],
  ['Shanghai', 121.47, 31.23, 'APAC', 'Manufacturing'], ['Shenzhen', 114.06, 22.54, 'APAC', 'Telecom'],
  ['Hong Kong', 114.17, 22.32, 'APAC', 'Finance'], ['Tokyo', 139.69, 35.69, 'APAC', 'Finance'],
  ['Seoul', 126.98, 37.57, 'APAC', 'Telecom'], ['Sydney', 151.21, -33.87, 'APAC', 'Energy'],
  ['Jakarta', 106.85, -6.21, 'APAC', 'Logistics'], ['Taipei', 121.56, 25.03, 'APAC', 'Manufacturing'],
];

export const STATUS_COL = { healthy: [52, 230, 168], risk: [255, 177, 61], disrupted: [255, 77, 114] };
export const STATUS_CSS = { healthy: PALETTE.emerald, risk: PALETTE.amber, disrupted: PALETTE.magenta };
export const STATUS_TINT = { healthy: 'rgba(52,230,168,0.14)', risk: 'rgba(255,177,61,0.14)', disrupted: 'rgba(255,77,114,0.14)' };
export const STATUS_LABEL = { healthy: 'OPTIMAL', risk: 'AT RISK', disrupted: 'DISRUPTED' };

// 10 agent archetypes
export const AGENT_ARCHETYPES = [
  { id: 0, name: 'ORCHESTRATOR', emoji: '🎼', desc: 'Coordinates swarm behavior, optimizes routing', color: '#33e1ff', role: 'COMMAND', abilities: ['Reroute', 'Optimize', 'Coordinate'], stats: { int: 95, spd: 82, res: 76, agi: 88 } },
  { id: 1, name: 'SENTINEL', emoji: '🛡️', desc: 'Detects threats, triggers defenses, pattern match', color: '#34e6a8', role: 'DEFENSE', abilities: ['Detect', 'Shield', 'Alert'], stats: { int: 88, spd: 74, res: 94, agi: 71 } },
  { id: 2, name: 'SCALER', emoji: '📈', desc: 'Auto-scales compute, load balances, capacity plan', color: '#ffb13d', role: 'OPS', abilities: ['Scale', 'Balance', 'Predict'], stats: { int: 92, spd: 86, res: 79, agi: 85 } },
  { id: 3, name: 'SWARM-MIND', emoji: '🐝', desc: 'Distributed decision-making, multi-agent consensus', color: '#ff4d72', role: 'COLLECTIVE', abilities: ['Vote', 'Swarm', 'Adapt'], stats: { int: 86, spd: 89, res: 72, agi: 91 } },
  { id: 4, name: 'ANALYST', emoji: '📊', desc: 'Processes telemetry, finds patterns, forecasts', color: '#a98bff', role: 'INTELLIGENCE', abilities: ['Analyze', 'Predict', 'Report'], stats: { int: 97, spd: 68, res: 74, agi: 64 } },
  { id: 5, name: 'NEGOTIATOR', emoji: '🤝', desc: 'Manages SLAs, broker resources, cost optimizes', color: '#00d9ff', role: 'BUSINESS', abilities: ['Negotiate', 'Optimize', 'Contract'], stats: { int: 89, spd: 76, res: 81, agi: 79 } },
  { id: 6, name: 'HEALER', emoji: '⚕️', desc: 'Self-healing, fault recovery, chaos resilience', color: '#34e6a8', role: 'RESILIENCE', abilities: ['Repair', 'Recover', 'Restore'], stats: { int: 84, spd: 72, res: 96, agi: 68 } },
  { id: 7, name: 'EXPLORER', emoji: '🗺️', desc: 'Discovery, A/B testing, novel strategies', color: '#ffb13d', role: 'INNOVATION', abilities: ['Explore', 'Test', 'Discover'], stats: { int: 90, spd: 91, res: 68, agi: 93 } },
  { id: 8, name: 'VALIDATOR', emoji: '✓', desc: 'Verification, compliance, audit trails', color: '#33e1ff', role: 'GOVERNANCE', abilities: ['Validate', 'Audit', 'Certify'], stats: { int: 93, spd: 71, res: 89, agi: 73 } },
  { id: 9, name: 'ORACLE', emoji: '🔮', desc: 'Forecasting, causality, meta-reasoning', color: '#a98bff', role: 'FORESIGHT', abilities: ['Forecast', 'Reason', 'Guide'], stats: { int: 98, spd: 65, res: 77, agi: 62 } },
];

// Live feed pool: [text, agent, color]
export const FEED_POOL = [
  ['Rerouted 2,430 TEU around Red Sea congestion', 'ORCHESTRATOR-7', PALETTE.cyan],
  ['Retrained fraud model v14 → 99.97% recall', 'AUTOML-CORE', PALETTE.violet],
  ['Shed 38MW load, balanced PJM interconnect', 'SCALER-3', PALETTE.emerald],
  ['Isolated BGP anomaly on APAC backbone', 'SENTINEL-2', PALETTE.amber],
  ['Spun up 12 inference replicas in ap-south', 'SWARM-MIND-9', PALETTE.cyan],
  ['Quarantined 1,118 suspicious card auths', 'SENTINEL-8', PALETTE.magenta],
  ['Pre-positioned safety stock at 4 DCs', 'ORCHESTRATOR-1', PALETTE.emerald],
  ['Negotiated spot freight at -11% vs index', 'NEGOTIATOR-4', PALETTE.cyan],
  ['Auto-failover Frankfurt → Madrid in 840ms', 'HEALER-6', PALETTE.emerald],
  ['Throttled DDoS surge, 2.1Tbps absorbed', 'SENTINEL-5', PALETTE.amber],
  ['Deployed champion model to 9 regions', 'ANALYST-3', PALETTE.violet],
  ['Resolved incident #4471 with no human touch', 'ORCHESTRATOR-2', PALETTE.emerald],
  ['Anomaly detected in feature space', 'ANALYST-7', PALETTE.violet],
  ['Validated 47 contracts for compliance', 'VALIDATOR-1', PALETTE.cyan],
  ['Forecast: demand spike +23% next 6h', 'ORACLE-4', PALETTE.violet],
  ['Discovered novel routing path -14% latency', 'EXPLORER-9', PALETTE.amber],
];

export const HUB_ACTIONS = {
  healthy: ['Holding optimal throughput, no action required', 'Continuous auto-tuning within tolerance'],
  risk: ['Pre-positioning buffer capacity, monitoring drift', 'Diverting 18% volume to redundant lane'],
  disrupted: ['Executing self-heal: rerouting + replica scale-out', 'Failover engaged, ETA to recovery 6m12s'],
};

// ARENA — 5v5 swarm
export const BLUE_DEF = [
  { id: 'ORCH', name: 'ORCHESTRATOR-7', role: 'CARRY', lane: 'SUPPLY', c: PALETTE.cyan, rgb: '51,225,255', ult: 'GLOBAL REROUTE', elo: 2840 },
  { id: 'FRAUD', name: 'FRAUD-SWARM', role: 'ASSASSIN', lane: 'FRAUD', c: PALETTE.magenta, rgb: '255,77,114', ult: 'MASS QUARANTINE', elo: 2710 },
  { id: 'GRID', name: 'GRID-AGENT-3', role: 'TANK', lane: 'GRID', c: PALETTE.emerald, rgb: '52,230,168', ult: 'LOAD SHED WALL', elo: 2655 },
  { id: 'NET', name: 'NET-SENTINEL', role: 'SUPPORT', lane: 'NET', c: PALETTE.amber, rgb: '255,177,61', ult: 'TRAFFIC SHIELD', elo: 2590 },
  { id: 'ML', name: 'AUTOML-CORE', role: 'MAGE', lane: 'MID', c: PALETTE.violet, rgb: '169,139,255', ult: 'HOT RETRAIN', elo: 2980 },
];
export const RED_DEF = [
  { id: 'BLK', name: 'PORT BLACKOUT', lane: 'SUPPLY', tier: 'ELITE' },
  { id: 'RING', name: 'CARD-FRAUD RING', lane: 'FRAUD', tier: 'BOSS' },
  { id: 'CASC', name: 'GRID CASCADE', lane: 'GRID', tier: 'ELITE' },
  { id: 'BOT', name: 'BOTNET SURGE', lane: 'NET', tier: 'MINION' },
  { id: 'SHOCK', name: 'DEMAND SHOCK', lane: 'MID', tier: 'BOSS' },
];
export const COMBAT_VERBS = [
  ['neutralized', PALETTE.emerald], ['intercepted', PALETTE.cyan], ['quarantined', PALETTE.magenta],
  ['out-maneuvered', PALETTE.amber], ['countered', PALETTE.violet], ['absorbed a strike from', PALETTE.mid],
];

// DECK — AutoML model cards
export const RARITY = {
  MYTHIC: { c: '#ff6ad5', glow: 'rgba(255,106,213,0.55)', label: 'MYTHIC' },
  LEGENDARY: { c: '#ffb13d', glow: 'rgba(255,177,61,0.5)', label: 'LEGENDARY' },
  EPIC: { c: '#a98bff', glow: 'rgba(169,139,255,0.5)', label: 'EPIC' },
  RARE: { c: '#33e1ff', glow: 'rgba(51,225,255,0.45)', label: 'RARE' },
  COMMON: { c: '#93a7c6', glow: 'rgba(147,167,198,0.3)', label: 'COMMON' },
};
export const CARDS_DEF = [
  { name: 'XGBoost Sentinel', type: 'FRAUD · GRADIENT BOOST', rarity: 'MYTHIC', atk: 99, def: 94, mana: 7, ability: 'Flags fraud rings 40ms before authorization. +12% recall aura to all FRAUD cards.', deployed: 9 },
  { name: 'TempoFormer-XL', type: 'DEMAND · TRANSFORMER', rarity: 'LEGENDARY', atk: 96, def: 88, mana: 9, ability: 'Forecasts demand shock 14 days out. Pre-positions stock automatically.', deployed: 6 },
  { name: 'GridGNN', type: 'ENERGY · GRAPH NET', rarity: 'LEGENDARY', atk: 92, def: 97, mana: 8, ability: 'Balances interconnect load. Immune to cascade failure.', deployed: 7 },
  { name: 'RouteOpt-Q', type: 'SUPPLY · RL POLICY', rarity: 'EPIC', atk: 90, def: 82, mana: 6, ability: 'Re-solves 2.4M-node routing every 800ms. Combo: +RouteOpt synergy.', deployed: 11 },
  { name: 'AnomalyDiffuser', type: 'NETWORK · DIFFUSION', rarity: 'EPIC', atk: 88, def: 85, mana: 6, ability: 'Denoises BGP traffic, isolates anomalies. Counters BOTNET SURGE.', deployed: 5 },
  { name: 'PriceProbe', type: 'PROCURE · BANDIT', rarity: 'RARE', atk: 81, def: 74, mana: 4, ability: 'Explores spot freight markets, exploits -11% arbitrage windows.', deployed: 8 },
  { name: 'LatencyLite', type: 'EDGE · DISTILLED', rarity: 'RARE', atk: 78, def: 80, mana: 3, ability: '7ms inference at the edge. Low compute, deployable anywhere.', deployed: 14 },
  { name: 'DriftWatch', type: 'MLOPS · MONITOR', rarity: 'COMMON', atk: 62, def: 71, mana: 2, ability: 'Watches feature drift. Auto-triggers retrain on champion decay.', deployed: 30 },
  { name: 'ReplicaScaler', type: 'INFRA · AUTOSCALE', rarity: 'COMMON', atk: 60, def: 76, mana: 2, ability: 'Spins inference replicas to match load. No human approval.', deployed: 30 },
];
export const CARD_SYNERGY_SETS = [
  ['neural-net', 'swarm-logic'], ['cascade'], ['neural-net'], ['swarm-logic', 'cascade'], ['swarm-logic'],
];
export const CARD_DEPLOY_COUNTS = [47, 89, 156, 23, 102];

// OPS — operators, quests, achievements
export const NPC_DEF = [
  { id: 'A', name: 'ATLAS', spec: 'Supply Ops', c: PALETTE.cyan, rgb: '51,225,255' },
  { id: 'V', name: 'VESPER', spec: 'Fraud Ops', c: PALETTE.magenta, rgb: '255,77,114' },
  { id: 'O', name: 'ORION', spec: 'Grid Ops', c: PALETTE.emerald, rgb: '52,230,168' },
  { id: 'N', name: 'NOVA', spec: 'Net Ops', c: PALETTE.amber, rgb: '255,177,61' },
  { id: 'H', name: 'HELIOS', spec: 'AutoML Ops', c: PALETTE.violet, rgb: '169,139,255' },
];
export const NPC_TASKS = ['Rebalancing 4 DCs', 'Retraining champion', 'Hunting fraud ring', 'Shedding 38MW', 'Patching BGP route', 'Negotiating freight', 'Scaling replicas', 'Drafting new model', 'Simulating shock', 'Auditing 1.1M auths'];
export const QUEST_DEF = [
  { id: 'q1', name: 'Operation Clean Sweep', kind: 'MAIN', desc: 'Resolve every disruption with zero human touch', goal: 12, reward: '2,400 XP · MYTHIC PACK' },
  { id: 'q2', name: 'Sub-50ms Continent', kind: 'SIDE', desc: 'Hold APAC latency under 50ms for 1h', goal: 8, reward: '900 XP · EPIC CORE' },
  { id: 'q3', name: 'Fraud Ring Bounty', kind: 'BOUNTY', desc: 'Quarantine 3 coordinated fraud rings', goal: 3, reward: '1,500 XP · LEGENDARY' },
  { id: 'q4', name: 'Carbon-Negative Grid', kind: 'SIDE', desc: 'Route 80% load through renewables', goal: 10, reward: '1,100 XP · TITAN BADGE' },
];
export const QUEST_PROG_FRAC = [0.66, 0.5, 0.66, 0.4];
export const ACHV_DEF = [
  { name: 'First Self-Heal', icon: '✦', got: true }, { name: '1M Actions', icon: '⬡', got: true },
  { name: 'Zero-Touch Week', icon: '◈', got: true }, { name: 'Fraud Slayer', icon: '⚔', got: true },
  { name: 'Grid Guardian', icon: '⬢', got: true }, { name: 'Model Master', icon: '★', got: false },
  { name: 'Continental MVP', icon: '♆', got: false }, { name: 'Singularity', icon: '∞', got: false },
];

// Simulation cadence (ms)
export const TICK_INTERVALS = { clock: 1000, sim: 1700, game: 1300 };

/* ============================================================================
 * 2. INITIAL STATE FACTORY
 * ==========================================================================*/

function buildHubs() {
  return HUBS_RAW.map((h, i) => ({
    id: i, name: h[0], pos: [h[1], h[2]], lng: h[1], lat: h[2],
    region: h[3], sector: h[4], load: 38 + ((i * 37) % 56),
  }));
}

function buildStatuses(hubs) {
  const statuses = {};
  hubs.forEach((h, i) => {
    statuses[i] = i % 11 === 4 ? 'disrupted' : i % 4 === 0 ? 'risk' : 'healthy';
  });
  return statuses;
}

function buildRoutes(n) {
  const routes = [];
  for (let k = 0; k < 36; k++) {
    let s = (k * 7 + 3) % n, t = (k * 13 + 5) % n;
    if (t === s) t = (t + 1) % n;
    routes.push({ s, t, vol: 0.4 + ((k * 11) % 10) / 10 });
  }
  return routes;
}

export function createInitialState() {
  const hubs = buildHubs();
  return {
    // ---- UI ----
    screen: 'command',
    selectedHubIdx: null,
    agentSelectedIdx: null,
    clock: '00:00:00',
    celebrating: false,
    celebrate: null, // { text, id }
    gtick: 0,

    // ---- COMMAND globe ----
    hubs,
    statuses: buildStatuses(hubs),
    routes: buildRoutes(hubs.length),
    metrics: { actions: 1284, models: 47, integrity: 99.2, threats: 3, thrpt: 4.8 },
    feed: [],

    // ---- meta / battle pass / prestige / cosmetics ----
    level: 47, xp: 6420, xpMax: 10000, season: 7, tier: 'PLATINUM',
    tierPct: 72, streak: 214, combo: 1, mult: 1.0,
    prestige: 2, prestigeMult: 2.0, eloRating: 2847,
    battlePass: { tier: 48, progress: 0, seasonStart: 1718000000, challenges: {} },
    cosmetics: { agentSkins: {}, cardBacks: {}, globeTheme: 'default' },
    guildId: null, guildRole: null,

    // ---- synergies & talents ----
    cardSynergies: {},
    agentTalents: { 0: [0, 2, 4], 1: [1, 3], 2: [0, 1], 3: [4], 4: [] },

    // ---- raids & waveforms ----
    activeRaid: null,
    raidCosplayers: [],
    threatWaveform: { mag: 0.0, eta: 180, phase: 0 },

    // ---- roguelike campaign ----
    rogueRun: null,
    rogueRunStats: { longestStreak: 23, maxXpEarned: 8900, runsCompleted: 12 },

    // ---- Definition of Done ----
    tasks: [
      { id: 0, name: 'Deploy v1.2.0 to prod', dod: [true, true, true, false, true], status: 'IN_PROGRESS' },
      { id: 1, name: 'Migrate APAC region', dod: [true, true, false, false, false], status: 'BLOCKED' },
      { id: 2, name: 'Rebalance sharding', dod: [true, false, false, false, false], status: 'BACKLOG' },
      { id: 3, name: 'Audit compliance', dod: [true, true, true, true, true], status: 'DONE' },
    ],
    dodChecklist: [
      { item: 'Code review +2 approvals', icon: '👁️', priority: 'CRITICAL' },
      { item: 'Unit tests ≥95% coverage', icon: '🧪', priority: 'CRITICAL' },
      { item: 'Perf tests < 100ms p99', icon: '⚡', priority: 'HIGH' },
      { item: 'Security scan 0 critical', icon: '🔒', priority: 'CRITICAL' },
      { item: 'Documentation updated', icon: '📖', priority: 'MEDIUM' },
    ],
    dodChecked: {}, // `${taskId}-${itemIdx}` -> bool

    // ---- 10-agent roster ----
    agents: AGENT_ARCHETYPES.map((a, i) => ({
      ...a, lvl: 12 + i, xp: 1200 + i * 340,
      status: i < 3 ? 'ACTIVE' : 'IDLE',
      uptime: 99.2 - i * 0.8, latency: 14 + i * 2, tasks: 3 - (i % 3),
    })),

    // ---- ARENA match ----
    matchSec: 642, blueScore: 31, redScore: 18, gold: 41200, teamXp: 78, winProb: 0.82,
    blue: BLUE_DEF.map((a, i) => ({
      ...a, lvl: 14 - (i % 3), hp: 70 + (i * 7) % 30, mana: 40 + (i * 13) % 55,
      k: 8 + (i * 5) % 14, d: (i * 3) % 6, a: 11 + (i * 7) % 20, cd: (i * 17) % 100, kp: 0,
      synergies: ['neural-net', 'cascade'][i % 2],
    })),
    red: RED_DEF.map((r, i) => ({ ...r, hp: 40 + (i * 23) % 55 })),
    lanes: BLUE_DEF.map((a, i) => ({ name: a.lane, c: a.c, tower: 60 + (i * 13) % 40, push: (i % 2 ? 1 : -1) * (0.2 + (i % 3) * 0.1) })),
    log: [],

    // ---- quests / NPCs / cards ----
    quests: QUEST_DEF.map((q, i) => ({ ...q, prog: Math.max(1, Math.floor(q.goal * QUEST_PROG_FRAC[i])) })),
    npcs: NPC_DEF.map((n, i) => ({ ...n, lvl: 30 + i * 4, task: NPC_TASKS[i], hp: 80 + (i * 7) % 20, busy: true, actions: 1200 + i * 340 })),
    cards: CARDS_DEF.map((c, i) => ({ ...c, synergies: CARD_SYNERGY_SETS[i % 5], deployCount: CARD_DEPLOY_COUNTS[i % 5] })),
    championIdx: 0,

    // ---- static-ish leaderboard + guild data ----
    leaderboards: {
      global: [{ name: 'ShadowMind', elo: 3421, prestige: 4 }, { name: 'NeuralKnight', elo: 3156, prestige: 3 }, { name: 'CascadeQueen', elo: 3089, prestige: 3 }],
      agents: [{ name: 'ATLAS', wins: 187, elo: 2956 }, { name: 'HELIOS', wins: 143, elo: 2834 }, { name: 'VESPER', wins: 156, elo: 2902 }],
      models: [{ name: 'DeepSeek-7B', deployments: 892, elo: 3012 }, { name: 'Falcon-13B', deployments: 756, elo: 2887 }, { name: 'Mistral-8x7B', deployments: 634, elo: 2945 }],
    },
    guilds: {
      myGuild: { name: 'Autonomic Collective', members: 14, warRank: 7, vault: { cosmetics: 3, xpMultiplier: 1.5 } },
      activeRaids: [{ name: 'Dark Network Intrusion', phase: 2, hp: 1200, maxHp: 3000, cosplayers: ['ShadowMind', 'NeuralKnight'] }],
    },
  };
}

/* ============================================================================
 * 3. REDUCER  —  pure, immutable. TICK runs the full simulation step.
 * ==========================================================================*/

const rand = (n) => Math.floor(Math.random() * n);
const pick = (arr) => arr[rand(arr.length)];
const pad2 = (x) => String(x).padStart(2, '0');

export const ACTIONS = {
  SET_SCREEN: 'SET_SCREEN',
  CYCLE_SCREEN: 'CYCLE_SCREEN',
  SELECT_HUB: 'SELECT_HUB',
  SELECT_AGENT: 'SELECT_AGENT',
  RESET_SELECTION: 'RESET_SELECTION',
  CLOCK_TICK: 'CLOCK_TICK',
  SIM_TICK: 'SIM_TICK',
  GAME_TICK: 'GAME_TICK',
  SEED_FEED: 'SEED_FEED',
  CELEBRATE: 'CELEBRATE',
  CLEAR_CELEBRATE: 'CLEAR_CELEBRATE',
  TOGGLE_DOD: 'TOGGLE_DOD',
  PRESTIGE: 'PRESTIGE',
  SET_GLOBE_THEME: 'SET_GLOBE_THEME',
  EQUIP_AGENT_SKIN: 'EQUIP_AGENT_SKIN',
  SET_AGENT_TALENTS: 'SET_AGENT_TALENTS',
  START_ROGUE_RUN: 'START_ROGUE_RUN',
  END_ROGUE_RUN: 'END_ROGUE_RUN',
  START_RAID: 'START_RAID',
};

function mkFeedItem(ago) {
  const p = pick(FEED_POOL);
  const m = Math.floor(ago), s = rand(59);
  return { id: Math.random(), text: p[0], agent: p[1], color: p[2], t: m === 0 ? `${s}s ago` : `${m}m ${s}s ago` };
}

// Returns a celebrate patch if a milestone text fired this tick. Caller merges it.
function withCelebrate(state, text) {
  return { celebrate: { text, id: Math.random() }, celebrating: true };
}

export function gameReducer(state, action) {
  switch (action.type) {
    /* ---------------- UI / navigation ---------------- */
    case ACTIONS.SET_SCREEN:
      return { ...state, screen: action.screen };

    case ACTIONS.CYCLE_SCREEN: {
      const i = SCREENS.indexOf(state.screen);
      const next = (i + action.dir + SCREENS.length) % SCREENS.length;
      return { ...state, screen: SCREENS[next] };
    }

    case ACTIONS.SELECT_HUB:
      return { ...state, selectedHubIdx: action.idx };

    case ACTIONS.SELECT_AGENT:
      return { ...state, agentSelectedIdx: action.idx };

    case ACTIONS.RESET_SELECTION:
      return { ...state, screen: 'command', selectedHubIdx: null, agentSelectedIdx: null };

    /* ---------------- clock ---------------- */
    case ACTIONS.CLOCK_TICK: {
      const d = new Date();
      return { ...state, clock: `${pad2(d.getUTCHours())}:${pad2(d.getUTCMinutes())}:${pad2(d.getUTCSeconds())}` };
    }

    /* ---------------- feed seed ---------------- */
    case ACTIONS.SEED_FEED: {
      const feed = [];
      for (let i = 0; i < 6; i++) feed.push(mkFeedItem(i * 4));
      return { ...state, feed };
    }

    /* ---------------- self-heal sim tick (hub statuses + metrics + feed) ---------------- */
    case ACTIONS.SIM_TICK: {
      const statuses = { ...state.statuses };
      const idxs = Object.keys(statuses);
      const dis = idxs.filter((i) => statuses[i] === 'disrupted');
      const rsk = idxs.filter((i) => statuses[i] === 'risk');
      if (dis.length && Math.random() < 0.6) statuses[dis[0]] = 'risk';
      else if (rsk.length && Math.random() < 0.5) statuses[rsk[0]] = 'healthy';
      else if (Math.random() < 0.4) statuses[idxs[rand(idxs.length)]] = Math.random() < 0.5 ? 'risk' : 'disrupted';

      const M = state.metrics;
      const integ = Math.min(99.99, Math.max(96.5, M.integrity + (Math.random() - 0.42) * 0.25));
      const threats = Math.max(0, Math.round((dis.length + rsk.length) / 3));
      const metrics = {
        actions: M.actions + rand(9) + 2,
        models: 44 + rand(9),
        integrity: +integ.toFixed(2),
        threats,
        thrpt: +(4.2 + Math.random() * 1.4).toFixed(1),
      };
      const feed = [mkFeedItem(0), ...state.feed].slice(0, 7);
      return { ...state, statuses, metrics, feed };
    }

    /* ---------------- game tick: arena + meta + all 9 expansions ---------------- */
    case ACTIONS.GAME_TICK: {
      const next = { ...state };
      let celebratePatch = null;
      const celebrate = (text) => { celebratePatch = withCelebrate(state, text); };

      next.gtick = state.gtick + 1;
      next.matchSec = state.matchSec + 7;

      // --- combat event ---
      const blue = state.blue.map((a) => ({ ...a }));
      const red = state.red.map((r) => ({ ...r }));
      const bIdx = rand(5), rIdx = rand(5);
      const b = blue[bIdx], r = red[rIdx];
      const verb = pick(COMBAT_VERBS);
      const win = Math.random() < 0.74;
      let combo = state.combo, blueScore = state.blueScore, redScore = state.redScore, gold = state.gold;
      if (win) {
        b.k++; b.a += Math.random() < 0.5 ? 1 : 0; blueScore++; gold += 120 + rand(340);
        r.hp = Math.max(8, r.hp - (12 + rand(22)));
        combo = Math.min(9, combo + 1);
      } else {
        b.d++; redScore++; b.hp = Math.max(15, b.hp - 10); combo = 1;
      }
      const mult = +(1 + combo * 0.15).toFixed(2);
      const log = [{ id: Math.random(), who: b.name, c: b.c, verb: verb[0], vc: verb[1], target: r.name, win, t: 'now' }, ...state.log].slice(0, 9);

      // regen / drift
      blue.forEach((a) => {
        a.hp = Math.min(100, a.hp + (Math.random() < 0.6 ? 4 : 0));
        a.mana = Math.min(100, a.mana + 6);
        a.cd = (a.cd + 9) % 100;
        a.kp = Math.round((100 * (a.k + a.a)) / (blueScore + 1));
      });
      red.forEach((x) => { if (x.hp < 25 && Math.random() < 0.4) x.hp = 45 + rand(45); });
      const lanes = state.lanes.map((l) => ({ ...l, tower: Math.max(8, Math.min(100, l.tower + (Math.random() - 0.45) * 9)) }));
      const teamXp = (Math.min(100, state.teamXp + 1.5)) % 100;
      const winProb = Math.max(0.5, Math.min(0.97, blueScore / (blueScore + redScore)));
      next.blue = blue; next.red = red; next.lanes = lanes;
      next.blueScore = blueScore; next.redScore = redScore; next.gold = gold;
      next.combo = combo; next.mult = mult; next.teamXp = teamXp; next.winProb = winProb; next.log = log;

      // --- meta / xp / prestige (#3) ---
      const gainMultiplier = state.prestigeMult || 1.0;
      const gain = Math.floor((30 + rand(70) * combo) * gainMultiplier);
      let xp = state.xp + gain;
      let level = state.level, xpMax = state.xpMax, tierPct = state.tierPct;

      // --- leaderboard ELO (#1) ---
      const eloRating = Math.max(800, Math.min(4000, state.eloRating + (win ? 18 : -12)));

      // --- battle pass (#2) ---
      const battlePass = { ...state.battlePass };
      battlePass.progress = (battlePass.progress || 0) + (Math.random() < 0.4 ? 1 : 0);
      if (battlePass.progress >= 10) { battlePass.tier++; battlePass.progress = 0; celebrate('BATTLE PASS TIER ' + battlePass.tier); }

      // --- synergy detection (#4) ---
      const blueSyn = {};
      blue.forEach((a) => { const k = a.synergies || ''; blueSyn[k] = (blueSyn[k] || 0) + 1; });
      Object.values(blueSyn).forEach((cnt) => { if (cnt >= 3) combo = Math.min(9, combo + 0.5); });
      next.combo = combo;

      // --- talent effects (#5) ---
      const talents = state.agentTalents[0] || [];
      const talentBoost = talents.includes(0) ? 1.1 : 1.0;
      blue.forEach((a) => { a.a = Math.floor(a.a * talentBoost); });

      // --- threat waveform + raid boss (#6) ---
      const threatWaveform = { ...state.threatWaveform };
      threatWaveform.mag = Math.max(0, Math.min(1, threatWaveform.mag + (Math.random() - 0.45) * 0.08));
      threatWaveform.eta = Math.max(0, threatWaveform.eta - 1);
      next.threatWaveform = threatWaveform;
      let activeRaid = state.activeRaid;
      if (activeRaid) {
        const hp = Math.max(0, activeRaid.hp - 15 - rand(30));
        if (hp === 0) { celebrate('RAID DEFEATED'); activeRaid = null; }
        else activeRaid = { ...activeRaid, hp };
      }
      next.activeRaid = activeRaid;

      // --- guild co-op raid (#8) ---
      const guilds = { ...state.guilds };
      if (guilds.activeRaids && guilds.activeRaids.length) {
        guilds.activeRaids = guilds.activeRaids.map((rd, i) => (i === 0 ? { ...rd, hp: Math.max(0, rd.hp - 8) } : rd));
      }
      next.guilds = guilds;

      // --- roguelike run (#9) ---
      if (state.rogueRun) {
        next.rogueRun = {
          ...state.rogueRun,
          xpEarned: (state.rogueRun.xpEarned || 0) + gain,
          streak: Math.min(25, (state.rogueRun.streak || 1) + (win ? 1 : 0)),
        };
      }

      // --- agent live drift ---
      next.agents = state.agents.map((a) => {
        const na = { ...a };
        na.uptime = Math.max(94, Math.min(99.99, +(na.uptime + (Math.random() - 0.38) * 0.04).toFixed(2)));
        na.latency = Math.max(8, Math.min(80, (na.latency + (Math.random() - 0.5) * 2) | 0));
        na.xp += rand(12) * (na.status === 'ACTIVE' ? 2 : 1);
        if (na.xp >= 2000) { na.xp -= 2000; na.lvl++; }
        na.tasks = Math.max(0, Math.min(8, na.tasks + (Math.random() < 0.1 ? (Math.random() < 0.5 ? 1 : -1) : 0)));
        if (Math.random() < 0.03) na.status = na.status === 'ACTIVE' ? 'IDLE' : 'ACTIVE';
        return na;
      });

      // --- task DoD auto-progress ---
      next.tasks = state.tasks.map((t) => {
        if (t.status !== 'IN_PROGRESS' || Math.random() >= 0.04) return t;
        const dod = [...t.dod];
        const idx = dod.findIndex((d) => !d);
        if (idx < 0) return t;
        dod[idx] = true;
        let status = t.status;
        if (dod.every(Boolean)) { status = 'DONE'; celebrate('TASK DONE: ' + t.name); }
        return { ...t, dod, status };
      });

      // --- level up ---
      if (xp >= xpMax) { xp -= xpMax; level++; xpMax = Math.round(xpMax * 1.04); tierPct = Math.min(100, tierPct + 6); celebrate('LEVEL ' + level + ' REACHED'); }
      next.xp = xp; next.level = level; next.xpMax = xpMax; next.tierPct = tierPct;
      next.eloRating = eloRating; next.battlePass = battlePass;
      next.streak = 214 + Math.floor(next.gtick / 40);

      // --- quests ---
      next.quests = state.quests.map((q) => {
        if (q.prog >= q.goal || Math.random() >= 0.25) return q;
        const prog = q.prog + 1;
        if (prog >= q.goal) celebrate('QUEST COMPLETE · ' + q.name);
        return { ...q, prog };
      });

      // --- npc tasks rotate ---
      next.npcs = state.npcs.map((n) => (Math.random() < 0.3 ? { ...n, task: pick(NPC_TASKS), actions: n.actions + rand(40) } : n));

      // --- champion rotation ---
      if (Math.random() < 0.12) next.championIdx = rand(3);

      return celebratePatch ? { ...next, ...celebratePatch } : next;
    }

    /* ---------------- celebration banner ---------------- */
    case ACTIONS.CELEBRATE:
      return { ...state, celebrate: { text: action.text, id: Math.random() }, celebrating: true };
    case ACTIONS.CLEAR_CELEBRATE:
      return { ...state, celebrate: null, celebrating: false };

    /* ---------------- DoD ---------------- */
    case ACTIONS.TOGGLE_DOD: {
      const key = `${action.taskId}-${action.itemIdx}`;
      return { ...state, dodChecked: { ...state.dodChecked, [key]: !state.dodChecked[key] } };
    }

    /* ---------------- prestige (#3) ---------------- */
    case ACTIONS.PRESTIGE: {
      if (state.level < 100) return state; // gate: only at level cap
      return {
        ...state,
        level: 1, xp: 0, xpMax: 10000,
        prestige: state.prestige + 1,
        prestigeMult: +(state.prestigeMult + 0.5).toFixed(2),
        ...withCelebrate(state, 'PRESTIGE ' + (state.prestige + 1)),
      };
    }

    /* ---------------- cosmetics (#7) ---------------- */
    case ACTIONS.SET_GLOBE_THEME:
      return { ...state, cosmetics: { ...state.cosmetics, globeTheme: action.theme } };
    case ACTIONS.EQUIP_AGENT_SKIN:
      return { ...state, cosmetics: { ...state.cosmetics, agentSkins: { ...state.cosmetics.agentSkins, [action.agentId]: action.skin } } };

    /* ---------------- talents (#5) ---------------- */
    case ACTIONS.SET_AGENT_TALENTS:
      return { ...state, agentTalents: { ...state.agentTalents, [action.agentId]: action.talents } };

    /* ---------------- roguelike (#9) ---------------- */
    case ACTIONS.START_ROGUE_RUN:
      return { ...state, rogueRun: { runId: Math.random().toString(36).slice(2), prestige: state.prestige, nodes: action.nodes || [], xpEarned: 0, streak: 1, loot: [] } };
    case ACTIONS.END_ROGUE_RUN: {
      const run = state.rogueRun;
      const rogueRunStats = run
        ? {
            longestStreak: Math.max(state.rogueRunStats.longestStreak, run.streak || 0),
            maxXpEarned: Math.max(state.rogueRunStats.maxXpEarned, run.xpEarned || 0),
            runsCompleted: state.rogueRunStats.runsCompleted + 1,
          }
        : state.rogueRunStats;
      return { ...state, rogueRun: null, rogueRunStats };
    }

    /* ---------------- raids (#6) ---------------- */
    case ACTIONS.START_RAID:
      return { ...state, activeRaid: { name: action.name, hp: action.maxHp, maxHp: action.maxHp, phaseNum: 1, cosplayers: [] } };

    default:
      return state;
  }
}

/* ============================================================================
 * 4. CONTEXT + PROVIDER  (runs the simulation loop)
 * ==========================================================================*/

const GameContext = createContext(null);

/**
 * GameProvider — wrap the app. Seeds the feed + combat log on mount, then runs
 * three independent intervals (clock / self-heal sim / game tick) and a global
 * keyboard-nav listener. Celebrations auto-clear after 2.6s.
 *
 * Pass `autoRun={false}` in tests/Storybook to freeze the simulation.
 */
export function GameProvider({ children, autoRun = true }) {
  const [state, dispatch] = useReducer(gameReducer, undefined, createInitialState);
  const celebrateTimer = useRef(null);

  // Seed feed + a few combat ticks once.
  useEffect(() => {
    dispatch({ type: ACTIONS.SEED_FEED });
    for (let i = 0; i < 6; i++) dispatch({ type: ACTIONS.GAME_TICK });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Simulation loops.
  useEffect(() => {
    if (!autoRun) return undefined;
    const clock = setInterval(() => dispatch({ type: ACTIONS.CLOCK_TICK }), TICK_INTERVALS.clock);
    const sim = setInterval(() => dispatch({ type: ACTIONS.SIM_TICK }), TICK_INTERVALS.sim);
    const game = setInterval(() => dispatch({ type: ACTIONS.GAME_TICK }), TICK_INTERVALS.game);
    return () => { clearInterval(clock); clearInterval(sim); clearInterval(game); };
  }, [autoRun]);

  // Auto-clear celebration banner.
  useEffect(() => {
    if (!state.celebrate) return undefined;
    clearTimeout(celebrateTimer.current);
    celebrateTimer.current = setTimeout(() => dispatch({ type: ACTIONS.CLEAR_CELEBRATE }), 2600);
    return () => clearTimeout(celebrateTimer.current);
  }, [state.celebrate]);

  // Keyboard navigation (1–9,0 + arrows + g/a/d/o/l/r/p + Esc).
  useEffect(() => {
    const onKey = (e) => {
      if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
      const k = e.key;
      const n = parseInt(k, 10);
      if (!isNaN(n)) { const idx = n === 0 ? 9 : n - 1; if (SCREENS[idx]) dispatch({ type: ACTIONS.SET_SCREEN, screen: SCREENS[idx] }); return; }
      if (k === 'ArrowRight' || k === ']') dispatch({ type: ACTIONS.CYCLE_SCREEN, dir: 1 });
      if (k === 'ArrowLeft' || k === '[') dispatch({ type: ACTIONS.CYCLE_SCREEN, dir: -1 });
      if (k === 'Escape') dispatch({ type: ACTIONS.RESET_SELECTION });
      const map = { g: 'command', a: 'arena', d: 'deck', o: 'ops', l: 'leaderboards', r: 'raids', p: 'battlepass' };
      if (map[k]) dispatch({ type: ACTIONS.SET_SCREEN, screen: map[k] });
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  const value = useMemo(() => ({ state, dispatch }), [state]);
  return <GameContext.Provider value={value}>{children}</GameContext.Provider>;
}

/* ============================================================================
 * 5. HOOKS
 * ==========================================================================*/

export function useGame() {
  const ctx = useContext(GameContext);
  if (!ctx) throw new Error('useGame() must be used inside <GameProvider>');
  return ctx;
}

// Current screen + bound setters.
export function useScreen() {
  const { state, dispatch } = useGame();
  const setScreen = useCallback((screen) => dispatch({ type: ACTIONS.SET_SCREEN, screen }), [dispatch]);
  const cycle = useCallback((dir) => dispatch({ type: ACTIONS.CYCLE_SCREEN, dir }), [dispatch]);
  return { screen: state.screen, meta: SCREEN_META[state.screen], setScreen, cycle };
}

// Bound action creators — keeps components free of raw dispatch.
export function useActions() {
  const { dispatch } = useGame();
  return useMemo(() => ({
    setScreen: (screen) => dispatch({ type: ACTIONS.SET_SCREEN, screen }),
    selectHub: (idx) => dispatch({ type: ACTIONS.SELECT_HUB, idx }),
    selectAgent: (idx) => dispatch({ type: ACTIONS.SELECT_AGENT, idx }),
    toggleDod: (taskId, itemIdx) => dispatch({ type: ACTIONS.TOGGLE_DOD, taskId, itemIdx }),
    prestige: () => dispatch({ type: ACTIONS.PRESTIGE }),
    setGlobeTheme: (theme) => dispatch({ type: ACTIONS.SET_GLOBE_THEME, theme }),
    equipAgentSkin: (agentId, skin) => dispatch({ type: ACTIONS.EQUIP_AGENT_SKIN, agentId, skin }),
    setAgentTalents: (agentId, talents) => dispatch({ type: ACTIONS.SET_AGENT_TALENTS, agentId, talents }),
    startRogueRun: (nodes) => dispatch({ type: ACTIONS.START_ROGUE_RUN, nodes }),
    endRogueRun: () => dispatch({ type: ACTIONS.END_ROGUE_RUN }),
    startRaid: (name, maxHp) => dispatch({ type: ACTIONS.START_RAID, name, maxHp }),
    celebrate: (text) => dispatch({ type: ACTIONS.CELEBRATE, text }),
  }), [dispatch]);
}

// Derived / computed values used across the HUD and screens.
export function useDerived() {
  const { state } = useGame();
  return useMemo(() => {
    const xpPct = `${Math.min(100, Math.round((state.xp / state.xpMax) * 100))}%`;
    const counts = { healthy: 0, risk: 0, disrupted: 0 };
    Object.values(state.statuses).forEach((s) => { counts[s]++; });
    const selectedHub = state.selectedHubIdx != null ? state.hubs[state.selectedHubIdx] : null;
    return {
      xpPct,
      xpText: `${state.xp.toLocaleString()} / ${state.xpMax.toLocaleString()} XP`,
      tierPctStr: `${Math.round(state.tierPct)}%`,
      hubCounts: counts,
      selectedHub,
      selectedHubStatus: selectedHub ? state.statuses[selectedHub.id] : null,
    };
  }, [state]);
}

/* ============================================================================
 * 6. PRESENTATIONAL HELPERS + SCREEN SCAFFOLDS
 * ----------------------------------------------------------------------------
 * Inline styles mirror the prototype's neon/glassmorphism look. These are
 * working scaffolds wired to live context state — flesh out per FEATURE_SPEC.md.
 * ==========================================================================*/

const mono = "'JetBrains Mono', ui-monospace, monospace";
const sans = "'Space Grotesk', system-ui, sans-serif";

export function Panel({ title, tag, right, children, style }) {
  return (
    <section style={{ background: PALETTE.panel, border: `1px solid ${PALETTE.line}`, borderRadius: 14, padding: 16, color: PALETTE.hi, fontFamily: sans, ...style }}>
      {(title || right) && (
        <header style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12 }}>
          {title && <h3 style={{ margin: 0, font: `600 13px ${sans}`, letterSpacing: 0.3 }}>{title}</h3>}
          {tag && <span style={{ font: `500 9px ${mono}`, color: PALETTE.cyan, border: `1px solid ${PALETTE.line2}`, borderRadius: 5, padding: '2px 6px' }}>{tag}</span>}
          <div style={{ flex: 1 }} />
          {right}
        </header>
      )}
      {children}
    </section>
  );
}

export function StatChip({ label, value, unit, color = PALETTE.hi }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 1, padding: '5px 13px', borderRadius: 9, background: PALETTE.panel, border: `1px solid ${PALETTE.line}`, minWidth: 84 }}>
      <span style={{ font: `400 8px ${mono}`, letterSpacing: 0.6, color: PALETTE.dim }}>{label}</span>
      <span style={{ font: `600 14px ${mono}`, color, lineHeight: 1 }}>
        {value}
        {unit && <span style={{ fontSize: 8, color: PALETTE.dim, marginLeft: 2 }}>{unit}</span>}
      </span>
    </div>
  );
}

export function ProgressBar({ pct, from, to, height = 4, width = 150 }) {
  return (
    <div style={{ width, height, borderRadius: 3, background: 'rgba(255,255,255,0.08)', overflow: 'hidden' }}>
      <div style={{ height: '100%', width: pct, borderRadius: 3, background: `linear-gradient(90deg, ${from}, ${to})`, boxShadow: `0 0 8px ${from}`, transition: 'width .5s' }} />
    </div>
  );
}

// ---- Navigation rail ----
export function NavRail() {
  const { screen, setScreen } = useScreen();
  return (
    <aside style={{ width: 78, flex: '0 0 78px', display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '16px 0 14px', borderRight: `1px solid ${PALETTE.line}`, background: 'linear-gradient(180deg,rgba(10,17,31,0.9),rgba(7,12,22,0.9))' }}>
      <nav style={{ marginTop: 26, display: 'flex', flexDirection: 'column', gap: 10, width: '100%', alignItems: 'center', flex: 1 }}>
        {SCREENS.map((s) => (
          <button
            key={s}
            onClick={() => setScreen(s)}
            title={SCREEN_META[s].title}
            style={{
              position: 'relative', width: 52, height: 52, border: 'none', borderRadius: 14, cursor: 'pointer',
              background: screen === s ? 'rgba(51,225,255,0.14)' : 'transparent',
              color: screen === s ? PALETTE.cyan : PALETTE.dim,
              display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 3, transition: 'all .18s',
            }}
          >
            <span style={{ font: `600 7px ${mono}`, letterSpacing: 0.5 }}>{SCREEN_META[s].tag}</span>
            {screen === s && <span style={{ position: 'absolute', left: -1, top: '50%', transform: 'translateY(-50%)', width: 3, height: 24, borderRadius: 3, background: PALETTE.cyan, boxShadow: `0 0 10px ${PALETTE.cyan}` }} />}
          </button>
        ))}
      </nav>
    </aside>
  );
}

// ---- Top bar + gamification HUD ----
export function TopBar({ dataMode = 'mock' }) {
  const { state } = useGame();
  const { meta } = useScreen();
  // Sim-driven metric chips map to no receipt/law-export/plan/report; in
  // praxis mode they are suppressed (PraxisHud carries the sourced values).
  const chips = dataMode === 'praxis' ? [] : [
    { label: 'ACTIONS', value: state.metrics.actions.toLocaleString(), color: PALETTE.cyan },
    { label: 'MODELS', value: state.metrics.models, color: PALETTE.violet },
    { label: 'INTEGRITY', value: state.metrics.integrity, unit: '%', color: PALETTE.emerald },
    { label: 'THREATS', value: state.metrics.threats, color: state.metrics.threats ? PALETTE.amber : PALETTE.dim },
    { label: 'THRPT', value: state.metrics.thrpt, unit: 'M/s', color: PALETTE.cyan },
  ];
  return (
    <header style={{ height: 62, flex: '0 0 62px', display: 'flex', alignItems: 'center', gap: 22, padding: '0 22px', borderBottom: `1px solid ${PALETTE.line}`, background: 'rgba(7,12,22,0.6)', backdropFilter: 'blur(8px)' }}>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 1, minWidth: 200 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 9 }}>
          <span style={{ font: `600 15px ${sans}` }}>{meta.title}</span>
          <span style={{ font: `500 9px ${mono}`, color: PALETTE.cyan, border: `1px solid ${PALETTE.line2}`, borderRadius: 5, padding: '2px 6px' }}>{meta.tag}</span>
        </div>
        <span style={{ font: `400 10px ${mono}`, color: PALETTE.dim }}>{meta.sub}</span>
      </div>
      <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'flex-end', gap: 9 }}>
        {chips.map((c) => <StatChip key={c.label} {...c} />)}
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: 1, paddingLeft: 6, borderLeft: `1px solid ${PALETTE.line}` }}>
          <span style={{ font: `600 12px ${mono}` }}>{state.clock}</span>
          <span style={{ font: `400 8px ${mono}`, color: PALETTE.dim }}>UTC · GLOBAL</span>
        </div>
      </div>
    </header>
  );
}

export function MetaHud() {
  const { state } = useGame();
  const { xpPct, xpText } = useDerived();
  return (
    <div style={{ height: 40, flex: '0 0 40px', display: 'flex', alignItems: 'center', gap: 16, padding: '0 22px', borderBottom: `1px solid ${PALETTE.line}`, background: 'linear-gradient(90deg,rgba(13,22,40,0.85),rgba(9,15,27,0.5))' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 9 }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', width: 30, height: 30, borderRadius: 9, background: `linear-gradient(145deg, ${PALETTE.violet}, #6d4dff)`, font: `700 12px ${mono}`, color: '#0a0714', boxShadow: '0 0 14px rgba(169,139,255,0.5)' }}>{state.level}</div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
          <span style={{ font: `600 8px ${mono}`, letterSpacing: 0.8, color: PALETTE.dim }}>PLATFORM LVL · {xpText}</span>
          <ProgressBar pct={xpPct} from={PALETTE.violet} to={PALETTE.cyan} />
        </div>
      </div>
      <div style={{ width: 1, height: 22, background: PALETTE.line }} />
      <div style={{ display: 'flex', alignItems: 'center', gap: 9 }}>
        <span style={{ font: `600 9px ${mono}`, letterSpacing: 0.8, color: PALETTE.amber, border: '1px solid rgba(255,177,61,0.35)', borderRadius: 5, padding: '2px 7px' }}>SEASON {state.season} · {state.tier}</span>
        <ProgressBar pct={`${state.tierPct}%`} from={PALETTE.amber} to="#ffe08a" width={120} />
      </div>
      <div style={{ flex: 1 }} />
      <div style={{ display: 'flex', alignItems: 'center', gap: 7, padding: '4px 11px', borderRadius: 8, background: 'rgba(255,77,114,0.08)', border: '1px solid rgba(255,77,114,0.25)' }}>
        <span style={{ fontSize: 13, lineHeight: 1 }}>🔥</span>
        <div style={{ display: 'flex', flexDirection: 'column', lineHeight: 1.1 }}>
          <span style={{ font: `700 13px ${mono}`, color: PALETTE.magenta }}>{state.streak}</span>
          <span style={{ font: `500 7px ${mono}`, letterSpacing: 0.5, color: PALETTE.dim }}>DAY ZERO-TOUCH STREAK</span>
        </div>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 7, padding: '4px 11px', borderRadius: 8, background: 'rgba(51,225,255,0.08)', border: '1px solid rgba(51,225,255,0.25)' }}>
        <span style={{ font: `700 13px ${mono}`, color: PALETTE.cyan }}>{state.eloRating}</span>
        <span style={{ font: `500 7px ${mono}`, letterSpacing: 0.5, color: PALETTE.dim }}>ELO · P{state.prestige}</span>
      </div>
    </div>
  );
}

export function CelebrationBanner() {
  const { state } = useGame();
  if (!state.celebrate) return null;
  return (
    <div style={{ position: 'fixed', top: 110, left: '50%', transform: 'translateX(-50%)', zIndex: 100, padding: '12px 28px', borderRadius: 12, background: 'linear-gradient(145deg,rgba(169,139,255,0.95),rgba(51,225,255,0.9))', color: '#06121f', font: `700 16px ${mono}`, letterSpacing: 1, boxShadow: '0 0 40px rgba(51,225,255,0.6)' }}>
      ✦ {state.celebrate.text}
    </div>
  );
}

/* ---- 3D renderer mount points (port imperative code from the prototype) ---- */

// deck.gl globe. On mount, load deck.gl, init a Deck instance into containerRef,
// and drive buildLayers()/animate() from state.hubs/statuses/routes.
export function GlobeCanvas() {
  const containerRef = useRef(null);
  useEffect(() => {
    // TODO: port loadDeck() → initDeck() → animate() RAF loop here.
    // Read state via useGame() in the parent and pass hubs/statuses/routes as props.
    return () => { /* finalize deck instance + cancel RAF */ };
  }, []);
  return <div ref={containerRef} style={{ position: 'absolute', inset: 0 }} />;
}

// three.js Tron arena. Same pattern: load three, build scene, render lanes/units.
export function ArenaCanvas() {
  const containerRef = useRef(null);
  useEffect(() => {
    // TODO: port loadThree() → scene/lanes/units → render loop here.
    return () => { /* dispose renderer + geometries */ };
  }, []);
  return <div ref={containerRef} style={{ position: 'absolute', inset: 0 }} />;
}

/* ---- Screen scaffolds ---- */

export function CommandScreen() {
  const { state } = useGame();
  const { hubCounts, selectedHub, selectedHubStatus } = useDerived();
  const { selectHub } = useActions();
  return (
    <div style={{ position: 'relative', flex: 1, display: 'grid', gridTemplateColumns: '1fr 320px', minHeight: 0 }}>
      <div style={{ position: 'relative' }}>
        <GlobeCanvas />
        <div style={{ position: 'absolute', left: 16, top: 16, display: 'flex', gap: 8 }}>
          {Object.entries(hubCounts).map(([k, v]) => (
            <StatChip key={k} label={STATUS_LABEL[k]} value={v} color={STATUS_CSS[k]} />
          ))}
        </div>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 12, padding: 16, borderLeft: `1px solid ${PALETTE.line}`, overflowY: 'auto' }}>
        <Panel title="Live Feed" tag="AUTONOMIC">
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {state.feed.map((f) => (
              <div key={f.id} style={{ display: 'flex', flexDirection: 'column', gap: 2, paddingBottom: 8, borderBottom: `1px solid ${PALETTE.line}` }}>
                <span style={{ font: `500 12px ${sans}`, color: PALETTE.hi }}>{f.text}</span>
                <span style={{ font: `500 9px ${mono}`, color: f.color }}>{f.agent} · {f.t}</span>
              </div>
            ))}
          </div>
        </Panel>
        {selectedHub && (
          <Panel title={selectedHub.name} tag={selectedHubStatus && STATUS_LABEL[selectedHubStatus]} right={<button onClick={() => selectHub(null)} style={{ background: 'none', border: 'none', color: PALETTE.dim, cursor: 'pointer' }}>✕</button>}>
            <p style={{ margin: 0, font: `400 11px ${mono}`, color: PALETTE.mid }}>{selectedHub.region} · {selectedHub.sector} · load {selectedHub.load}%</p>
            <p style={{ marginTop: 8, font: `400 12px ${sans}`, color: PALETTE.hi }}>{HUB_ACTIONS[selectedHubStatus]?.[0]}</p>
          </Panel>
        )}
      </div>
    </div>
  );
}

export function ArenaScreen() {
  const { state } = useGame();
  return (
    <div style={{ position: 'relative', flex: 1, display: 'grid', gridTemplateColumns: '1fr 300px', minHeight: 0 }}>
      <div style={{ position: 'relative' }}><ArenaCanvas /></div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 12, padding: 16, borderLeft: `1px solid ${PALETTE.line}`, overflowY: 'auto' }}>
        <Panel title="Blue Swarm" tag={`WIN ${Math.round(state.winProb * 100)}%`}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {state.blue.map((a) => (
              <div key={a.id} style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <span style={{ width: 8, height: 8, borderRadius: '50%', background: a.c, boxShadow: `0 0 8px ${a.c}` }} />
                <span style={{ flex: 1, font: `600 11px ${mono}`, color: PALETTE.hi }}>{a.name}</span>
                <span style={{ font: `500 10px ${mono}`, color: PALETTE.dim }}>{a.k}/{a.d}/{a.a}</span>
              </div>
            ))}
          </div>
        </Panel>
        <Panel title="Combat Log" tag={`${state.blueScore}–${state.redScore}`}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
            {state.log.map((l) => (
              <span key={l.id} style={{ font: `500 11px ${sans}` }}>
                <span style={{ color: l.c }}>{l.who}</span>{' '}
                <span style={{ color: l.vc }}>{l.verb}</span>{' '}
                <span style={{ color: PALETTE.mid }}>{l.target}</span>
              </span>
            ))}
          </div>
        </Panel>
      </div>
    </div>
  );
}

export function DeckScreen() {
  const { state } = useGame();
  return (
    <div style={{ flex: 1, padding: 22, overflowY: 'auto' }}>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(220px, 1fr))', gap: 16 }}>
        {state.cards.map((c) => {
          const r = RARITY[c.rarity];
          return (
            <article key={c.name} style={{ border: `1px solid ${r.c}`, borderRadius: 14, padding: 14, background: PALETTE.panel2, boxShadow: `0 0 22px ${r.glow}` }}>
              <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
                <span style={{ font: `700 13px ${sans}`, color: PALETTE.hi }}>{c.name}</span>
                <span style={{ font: `600 8px ${mono}`, color: r.c }}>{r.label}</span>
              </header>
              <p style={{ margin: 0, font: `500 9px ${mono}`, color: PALETTE.dim }}>{c.type}</p>
              <p style={{ margin: '10px 0', font: `400 11px ${sans}`, color: PALETTE.mid, minHeight: 48 }}>{c.ability}</p>
              <footer style={{ display: 'flex', gap: 10, font: `600 11px ${mono}` }}>
                <span style={{ color: PALETTE.magenta }}>ATK {c.atk}</span>
                <span style={{ color: PALETTE.cyan }}>DEF {c.def}</span>
                <span style={{ color: PALETTE.amber, marginLeft: 'auto' }}>⬡ {c.deployCount}</span>
              </footer>
              <div style={{ display: 'flex', gap: 5, marginTop: 8, flexWrap: 'wrap' }}>
                {c.synergies.map((s) => <span key={s} style={{ font: `500 8px ${mono}`, color: PALETTE.violet, border: '1px solid rgba(169,139,255,0.35)', borderRadius: 5, padding: '1px 5px' }}>{s}</span>)}
              </div>
            </article>
          );
        })}
      </div>
    </div>
  );
}

export function OpsScreen() {
  const { state } = useGame();
  return (
    <div style={{ flex: 1, padding: 22, display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16, overflowY: 'auto' }}>
      <Panel title="Operators" tag="NPC">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
          {state.npcs.map((n) => (
            <div key={n.id} style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <div style={{ width: 32, height: 32, borderRadius: 8, display: 'flex', alignItems: 'center', justifyContent: 'center', font: `700 13px ${mono}`, color: '#06121f', background: n.c }}>{n.id}</div>
              <div style={{ flex: 1 }}>
                <div style={{ font: `600 12px ${sans}`, color: PALETTE.hi }}>{n.name} <span style={{ color: PALETTE.dim, fontWeight: 400, fontSize: 10 }}>LV{n.lvl}</span></div>
                <div style={{ font: `400 10px ${mono}`, color: PALETTE.mid }}>{n.task}</div>
              </div>
              <span style={{ font: `500 10px ${mono}`, color: n.c }}>{n.actions.toLocaleString()}</span>
            </div>
          ))}
        </div>
      </Panel>
      <Panel title="Quests" tag="ACTIVE">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          {state.quests.map((q) => (
            <div key={q.id}>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ font: `600 12px ${sans}`, color: PALETTE.hi }}>{q.name}</span>
                <span style={{ font: `600 9px ${mono}`, color: PALETTE.amber }}>{q.kind}</span>
              </div>
              <div style={{ font: `400 10px ${mono}`, color: PALETTE.dim, margin: '3px 0 6px' }}>{q.desc}</div>
              <ProgressBar pct={`${Math.round((q.prog / q.goal) * 100)}%`} from={PALETTE.emerald} to={PALETTE.cyan} width="100%" />
            </div>
          ))}
        </div>
      </Panel>
    </div>
  );
}

export function LeaderboardsScreen() {
  const { state } = useGame();
  const cols = [
    { key: 'global', title: 'Global ELO', rows: state.leaderboards.global, val: (r) => r.elo, sub: (r) => `P${r.prestige}` },
    { key: 'agents', title: 'Per-Agent', rows: state.leaderboards.agents, val: (r) => r.elo, sub: (r) => `${r.wins} W` },
    { key: 'models', title: 'Per-Model', rows: state.leaderboards.models, val: (r) => r.elo, sub: (r) => `${r.deployments} dep` },
  ];
  return (
    <div style={{ flex: 1, padding: 22, display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 16, overflowY: 'auto' }}>
      {cols.map((c) => (
        <Panel key={c.key} title={c.title} tag="RANKED">
          <ol style={{ margin: 0, padding: 0, listStyle: 'none', display: 'flex', flexDirection: 'column', gap: 8 }}>
            {c.rows.map((r, i) => (
              <li key={r.name} style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                <span style={{ width: 18, font: `700 12px ${mono}`, color: i === 0 ? PALETTE.amber : PALETTE.dim }}>{i + 1}</span>
                <span style={{ flex: 1, font: `600 12px ${sans}`, color: PALETTE.hi }}>{r.name}</span>
                <span style={{ font: `600 11px ${mono}`, color: PALETTE.cyan }}>{c.val(r)}</span>
                <span style={{ font: `500 9px ${mono}`, color: PALETTE.dim }}>{c.sub(r)}</span>
              </li>
            ))}
          </ol>
        </Panel>
      ))}
    </div>
  );
}

// Placeholder for screens still to be built out (battlepass, prestige, talents,
// raids, cosmetics, guilds, roguelike, agents, dod). Wire each to context state.
export function PlaceholderScreen({ screen }) {
  const meta = SCREEN_META[screen] || {};
  return (
    <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 8, color: PALETTE.dim, fontFamily: sans }}>
      <span style={{ font: `700 22px ${sans}`, color: PALETTE.hi }}>{meta.title}</span>
      <span style={{ font: `400 12px ${mono}` }}>{meta.sub}</span>
      <span style={{ font: `500 10px ${mono}`, color: PALETTE.cyan, marginTop: 8 }}>TODO · build per FEATURE_SPEC.md</span>
    </div>
  );
}

const SCREEN_COMPONENTS = {
  command: CommandScreen,
  arena: ArenaScreen,
  deck: DeckScreen,
  ops: OpsScreen,
  leaderboards: LeaderboardsScreen,
};

// Screens with a real standing source in praxis mode (adapter-backed).
const PRAXIS_SCREEN_COMPONENTS = {
  deck: PraxisDeckScreen,
  ops: PraxisOpsScreen,
  casestudy: PraxisCaseStudyScreen,
};

export function ScreenRouter({ dataMode = 'mock' }) {
  const { screen } = useScreen();
  if (dataMode === 'praxis' && PRAXIS_SCREEN_COMPONENTS[screen]) {
    const Cmp = PRAXIS_SCREEN_COMPONENTS[screen];
    return <Cmp />;
  }
  // Everything else (GLOBE/command, ARENA, leaderboards, placeholders, and all
  // screens in mock mode) is simulation-driven: persistent NON-STANDING banner.
  const Cmp = SCREEN_COMPONENTS[screen];
  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
      <NonStandingBanner />
      {Cmp ? <Cmp /> : <PlaceholderScreen screen={screen} />}
    </div>
  );
}

/* ---- Root layout ---- */

// dataMode: 'praxis' (default — standing-mapped DECK/OPS/HUD via the adapter)
// or 'mock' (supabase-mock/simulation everywhere, every screen NON-STANDING).
export function AutonomicPlatform({ dataMode = 'praxis' }) {
  return (
    <PraxisProvider>
      <GameProvider>
        <div style={{ position: 'fixed', inset: 0, display: 'flex', overflow: 'hidden', fontFamily: sans, color: PALETTE.hi, background: `radial-gradient(120% 90% at 75% 10%, rgba(28,60,110,0.30), transparent 55%), radial-gradient(90% 80% at 10% 95%, rgba(90,50,140,0.18), transparent 60%), ${PALETTE.bg}` }}>
          <NavRail />
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
            <TopBar dataMode={dataMode} />
            {dataMode === 'praxis' ? <PraxisHud /> : <MetaHud />}
            <ScreenRouter dataMode={dataMode} />
          </div>
          <CelebrationBanner />
        </div>
      </GameProvider>
    </PraxisProvider>
  );
}

export default AutonomicPlatform;

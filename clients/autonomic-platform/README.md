# Autonomic Platform — Praxis control-room client (experimental)

Role: experimental web control-room client surface for Praxis. Adopted into
version control from the design-canvas handoff bundle of 2026-07-06
("Autonomic process intelligence platform"); the sibling `TenFourApp`/
`ios-frame` files are a separate product and were deliberately left behind.

## Standing doctrine

From `docs/releases/v26.7.6/CLIENT_SURFACES.md` and
`CLIENT_ADAPTER_CONTRACT.md`: **clients display and command standing; they
never create it.** Every rendered status maps to a provenance source
(`receipt | law-export | plan | report`) via `{ source, ref }`, or it renders
as UNKNOWN — visually distinct, never green. Screens driven by the bundled
simulation/mock carry a persistent NON-STANDING banner
(`NonStandingBanner` in `src/praxis-mode.js`).

## Modes

- **praxis** (default): `src/praxis-adapter.js` implements the adapter
  contract's read path against real repo artifacts:
  - `.ggen-v2/receipt.json` + `.ggen-v2/receipt-log.jsonl` — receipt chain
  - `target/plan_run/*/plan.json` (incl. `powl_chain_hash`) — plan
  - `docs/releases/v26.7.6/BREED_ALGORITHM_REGISTRY.md` — parsed tables

  Standing-mapped screens: **DECK** (registry cards; rarity tier only when the
  registry carries speedTier/qualityTier — the current projection carries
  law-derived Status/Standing, which is shown as the only badge), **OPS**
  (incidents = non-Green receipts, i.e. typed refusals/blockers; empty state
  is lawful) and plan steps, **HUD strip** (chain head hash + receipt count).
  **GLOBE/COMMAND and ARENA remain mock-driven and NON-STANDING**, as does
  every other simulation screen. Absent source files render UNKNOWN — no
  fabricated data in praxis mode.
- **mock** (`?mode=mock`): the bundle's original simulation +
  `supabase-mock.js` demo; every screen carries the NON-STANDING banner.

## Dev artifact mechanism

In dev, `vite.config.js` registers a `praxis-artifacts` middleware mapping
`/praxis-artifacts/{receipt.json,receipt-log.jsonl,registry.md,plan.json}` to
the repo files above (plan.json is resolved to the first
`target/plan_run/*/plan.json` and wrapped as `{ ref, data }`);
`server.fs.allow` is widened to the repo root. A production host must serve
the same paths; if it doesn't, the client shows UNKNOWN, not stale data.

## Build / run

```sh
npm install
npm run build   # vite build
npm run dev     # dev server with the artifact bridge
```

3D canvases (`GlobeCanvas` deck.gl, `ArenaCanvas` three.js) are documented
mount points only — deck.gl/three are intentionally not dependencies yet.
`src/kgc4d-integration.js` is adopted un-wired (unverified `@unrdf/kgc-4d`
API; see its header note).

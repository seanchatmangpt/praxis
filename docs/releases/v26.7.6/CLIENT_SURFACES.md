# Client Surfaces — Inventory, Classification, Role Mapping

Doctrine: **clients display and command standing; they do not create standing.**
Every badge, chart, queue, button, and report in a client must map to a GraphLaw
fact, planner state, POWL workflow state, bcinr transition result, ggen artifact,
verifier output, receipt, or generated report. If a UI value cannot point to one of
those, it is not allowed to present as standing.

Survey date: 2026-07-06. Scope: /Users/sac top level, ~/dev, ~/chatmangpt,
~/praxis, ~/wasm4pm (packages/apps/playground). ~90 package.json scanned, 44
framework matches. Praxis itself contains no Next/Expo/Nuxt client.

## Role assignments (best candidate per role)

| Role | Codebase | Framework | Classification | Evidence |
|---|---|---|---|---|
| **Enterprise web control room** | `/Users/sac/optimus` (optimus-prime-platform) | Next.js 15.5.5 | **ALIVE (standalone)** — 963 src files, 25 pages, 143 components; dashboard/admin-dashboard/agent-dashboard/report-card/knowledge-graph/executive-ai pages; own API routes incl. `/api/ggen`, `/api/report-card`, `/api/metrics`, `/api/health` | last commit 2025-10-29 |
| **Mobile operator console** | `/Users/sac/pcp` (pcp-monorepo) | Expo 56 / RN 0.85 | **ALIVE (standalone)** — 1536 src files, 51 pages; screens: verify, validation, audit, receipts, outbox, consequence-supervision, actor-lab, process, intelligence, admin | last commit 2026-06-02 |
| **Browser shell** | `/Users/sac/dashboard.bak` (pqc-evidence-center) | Nuxt 4.0.3 | **PARTIAL** — 88 real pages (threat-intel, incident-response, security-compliance, govern/measure/manage), Supabase-wired; **not a git repo** (no history — must be adopted into version control before it can carry standing) | no git |
| Browser shell (alternate, in-tree) | `/Users/sac/wasm4pm/apps/playground-web` | Nuxt 4.4.6 | PARTIAL — committed 2026-07-05, Nuxt UI dashboard, 38 pages, backend not wired; shallower but lowest integration friction | 2026-07-05 |

Runner-up (mobile): `/Users/sac/zoeapp` (@truex/membrane-client, Expo 56,
Supabase+OpenAI, 62 pages, 2026-06-01).

### Incoming prototype bundle (2026-07-06)

`~/Downloads/Autonomic process intelligence platform.zip` (design-canvas handoff,
16 files) — classification **PARTIAL (prototype, not yet a repo)**:

- `TenFourApp.jsx` — WEX "10 Four" fleet/logistics console, 21 screens, 5 roles
  (driver/dispatcher/owner/shift_manager/ml_ops), Expo-portable ("swap two
  imports"), ships SupabaseProvider + full supabase-mock (runs with zero backend).
  Fourth mobile-console candidate alongside `~/pcp`.
- `AutonomicPlatform.js` + `FEATURE_SPEC.md` — gamified PI control room (deck.gl
  globe, Three.js arena, TCG model deck, ops) with a Combinatorial-Maximalism
  expansion spec. Browser-shell/world-interface lineage.
- `kgc4d-integration.js` — @unrdf/kgc-4d binding projecting live state into RDF
  named graphs per tick (spatial+temporal, SPARQL-queryable). Client-side
  visualization pattern adjacent to GraphLaw receipt ingest; JS lineage, not a
  core-engine candidate. API binding unverified (noted in file).

Next action: adopt into version control if promoted to a role; until then it
carries no standing.

## Declared sources (acceptance criteria)

| Criterion | Status |
|---|---|
| Standing source | **BLOCKED_TYPED** — no Praxis client adapter exists yet. Required: a read-only surface exporting GraphLaw-derived state + receipts (candidates: ggen-generated JSON reports, `law export` output, `.ggen-v2/receipt.json` chain). Clients currently expect Supabase (pcp, dashboard.bak, zoeapp) or self-hosted Next API routes (optimus). Next action: define the client adapter contract (report/receipt JSON schema) in v26.7.6 docs; wire in a follow-up release. |
| Receipt source | **BLOCKED_TYPED** — same adapter gap. Receipt chain exists (`.ggen-v2/receipt.json`, `receipt verify` verb); no client consumes it yet. |
| Build command per client (executed 2026-07-06) | **optimus: PASS** — `next build` completed (route table emitted, static+dynamic pages, middleware 35.4 kB) despite npm-install fallback noise. **pcp: BLOCKED_TYPED** — `npx expo export --platform web` fails: unresolved modules `@/src/components/AvatarRelativeProjection` (from `src/app/(auth)/_layout.tsx`) and `@/src/hooks/useOcelEvidence` (from `(tabs)/process.tsx`) — source files missing from repo; next action: restore/stub the missing modules or point imports at existing components. **dashboard.bak: BLOCKED_TYPED** — `nuxi build` fails in vite/postcss (`Cannot read properties of null (reading 'matches')` in parseStyles) — CSS compile failure; retry with clean pnpm install in progress; next action if persistent: bisect the offending stylesheet. |
| Command surface | **PLANNED** — clients may invoke lawful actions only through declared adapters (planner/workflow verbs); no direct mutation of law-state. |

## Non-candidates (classified)

- **Scaffolds/stubs** (unmodified starters, mostly no git): ~/app, ~/nuxt-layer,
  ~/nuxt-ui-pro-landing, ~/practice; ~/dev/{chat, chatui, content-app, dora,
  igp-comps, lexi, marcus-campbell-web, marcus-campbell-landing, nuxt-helpdesk,
  legal, nuxtgen, mq6, neuromancer-matrix, nuxt-phoenix, render, nuxt-json-server,
  npilot, ocr-demo, petstore, rrnuxt, baif, org, web} — STALE/OUT_OF_SCOPE.
- **Templates**: ~/expo-supabase-ai-template, ~/dev/{dash, my-dashboard,
  cracking-coding-platform, rag(customized but no git)} — OUT_OF_SCOPE.
- **Libraries, not apps**: ~/ai (Vercel AI SDK monorepo), ~/zoela (42 components,
  0 pages), ~/dev/{hygen-nuxt-module, nuxt-py, wkflo-module} — OUT_OF_SCOPE.
- **Working but off-role**: ~/neako (graph viz, Nuxt), ~/dogturk (remo-dash),
  ~/dev/trialbase.bak (no git), ~/ai-chatbot (stock scaffold) — OUT_OF_SCOPE for
  this release.

## Release acceptance state (this document's criteria)

- [x] Next.js code inventoried, classified, mapped to control-room role.
- [x] Expo code inventoried, classified, mapped to mobile-console role.
- [x] Nuxt code inventoried, classified, mapped to browser-shell role (with the
      no-git caveat typed).
- [ ] Build command executed per client (typed above as UNVERIFIED-RUN).
- [x] Standing source declared (adapter gap typed as blocker).
- [x] Receipt source declared (adapter gap typed as blocker).
- [ ] C4 diagrams include client surfaces (pending C4.md pass).
- [ ] FORTUNE5_READINESS includes client surfaces (pending).
- [ ] FINAL_STATUS reports client standing (pending).

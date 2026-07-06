# Client Adapter Contract — Standing/Receipt Export for Client Surfaces

Doctrine: clients display and command standing; they never create it. This
contract defines the only lawful data path between Praxis core and any client
surface (Next.js control room, Expo console, Nuxt shell, Autonomic Platform).

## Sources of truth (what exists today)

| Source | Location | Producer | Status |
|---|---|---|---|
| Receipt chain | `.ggen-v2/receipt.json` | `crates/ggen` sync pipeline (`praxis_core::receipt_record`, BLAKE3 chain, `ts_ns = 0`) | EXISTS |
| Law-state export | `law export` verb (materialized graph as N-Triples) | `crates/ggen` law verbs over `praxis-graphlaw` | EXISTS (v26.7.6) |
| Plan artifacts | `plan run --out-dir` → `domain.pddl`, `problem.pddl`, `plan.json` + `powl_chain_hash` | root binary `plan run` (see `examples/v26_7_6_after_neon/README.md`) | EXISTS (v26.7.6) |
| Generated reports | ggen-rendered markdown/JSON under `docs/releases/*` and template outputs | ggen sync | EXISTS |
| Breed/algorithm registry | `docs/releases/v26.7.6/BREED_ALGORITHM_REGISTRY.md` + admitted TTL facts | ggen from breeds/algorithms TTL | EXISTS (v26.7.6) |

## The contract

1. **Read-only by default.** Clients consume files/exports above verbatim. A
   client MUST NOT synthesize a standing value that does not appear in a source.
2. **Every rendered status maps to a source pointer.** UI elements carry a
   provenance reference: `{ source: "receipt" | "law-export" | "plan" | "report",
   ref: <chain hash | triple subject | file path> }`. A value without provenance
   renders as UNKNOWN, visually distinct — never as a green state.
3. **Commands go through verbs, not writes.** A client triggers work only by
   invoking the CLI verb surface (`plan run`, `law derive`, `receipt verify`, …)
   through a thin local adapter (HTTP shim or process exec). The adapter passes
   arguments through; it holds no business logic and no state.
4. **Receipts are the sync primitive.** Clients poll or tail the receipt chain;
   a change in `current_chain_hash` is the only signal that standing moved.
   Wall-clock timestamps, if displayed, come from file mtimes and are labeled
   display-only (never part of standing).
5. **Demo mode is labeled.** Mock backends (e.g. `supabase-mock.js` in the
   Autonomic Platform bundle) are permitted for demos but every screen rendered
   from a mock carries a persistent NON-STANDING banner.

## JSON projection (planned, typed as the next patch)

A `report client-state` verb (or ggen template) projecting the above into one
stable JSON document for clients:

```json
{
  "generated_from": { "chain_hash": "blake3:...", "law_export": "sha/hash" },
  "lanes": [{ "id": "...", "wip": 0, "arrivals": 0, "time_to_standing_ns": 0 }],
  "artifacts": [{ "id": "...", "standing": "Verified|Blocked|...", "receipt": "blake3:..." }],
  "blockers": [{ "refusal": "TypedVariantName", "surface": "...", "next_action": "..." }],
  "plan": { "goal": "...", "steps": [], "powl_chain_hash": "blake3:..." }
}
```

Status: PLANNED — until it lands, clients read the raw sources above directly.
Little's Law lane metrics (wip/arrivals/time-to-standing) require receipt-chain
aggregation that does not exist yet; typed as the follow-on patch, tracked with
the Divan benchmark phase (`little_law_snapshot`).

## Per-client wiring status

| Client | Reads today | Gap |
|---|---|---|
| optimus (Next) | nothing (own `/api/ggen` route is a stub for its old backend) | point API routes at receipt chain + plan artifacts |
| pcp (Expo) | Supabase | build blocked (missing modules) before wiring is relevant |
| dashboard.bak (Nuxt) | Supabase | build blocked (generation skew) |
| Autonomic Platform (prototype) | supabase-mock | conversion phase (task: standing-mapped screens) |

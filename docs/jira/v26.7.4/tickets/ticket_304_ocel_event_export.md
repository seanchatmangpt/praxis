# Ticket: OCEL V2 Event Export from Existing Firing Receipts — CLOSED

## Title
Add `firing::to_ocel_event` — a pure-function OCEL 2.0 JSON renderer over `HookFiringReceipt` (PROJ-304) — **STATUS: CLOSED**

## Description
The vision doc's "OCEL V2 = call-detail record" claim is answerable without any new event
model: `HookFiringReceipt` (`firing.rs`) already carries everything an OCEL 2.0 event needs —
an identity (`outcome_hash`), related objects (bound capability/handler IRIs from `bindings`),
and (after PROJ-301/303) a timestamp anchor and an attribution anchor. This ticket adds one
pure function, `to_ocel_event(receipt: &HookFiringReceipt) -> serde_json::Value`, rendering:

```json
{
  "id": "<outcome_hash>",
  "type": "hook-firing",
  "time": "<time_anchor, if any RealityAddressRecord was bound for this firing — else omitted>",
  "relationships": [ {"objectId": "<handler/capability IRI>", "qualifier": "handler-binding"} ],
  "attributes": { "outcome": "Completed|Refused", "hook_hash": "...", "event_hash": "..." }
}
```

No wall clock is invented — if no `RealityAddressRecord` was bound for the firing (i.e. no
`OWL-Time` anchor existed in the source graph), the `time` field is omitted, not
defaulted to a fabricated value; this is an honest OCEL export, not a synthetic one.

## Acceptance Criteria
- `to_ocel_event` is a pure function taking `&HookFiringReceipt` (and optionally
  `Option<&RealityAddressRecord>` for the time/attribution fields) and returning a
  `serde_json::Value` — no I/O, no new receipt chain, no new hash folded into `firing.rs`'s
  existing chain (this is a read-only projection of existing receipt data).
- A test renders a completed firing and a refused firing, asserting both produce valid JSON
  with the expected `id`/`relationships`/`attributes` shape, and that a firing with no bound
  `RealityAddressRecord` omits `time` rather than fabricating one.
- No new Cargo dependency (an OCEL-conformance-checking crate is explicitly out of scope —
  hand-render the minimal shape needed for the fields this crate actually has evidence for).

## Dependencies
PROJ-301 (time anchor), PROJ-303 (authority/attribution anchor).

## Verification Mechanism
1. `cargo test -p praxis-synthesis --lib firing::` or a new `tests/ocel_export.rs` — green.
2. Manual `cargo run` or test-assisted `jq` inspection of one rendered event confirming valid
   JSON and no fabricated timestamp on an unanchored firing.

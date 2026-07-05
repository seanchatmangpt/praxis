# Milestone Overview: v26.7.4 — Reality-Addressed Capability OS (Build Phase)

Unlike `docs/jira/v26.7.3/tickets/` (which audited an external vision document and mostly
concluded "already closed" or "refuse as unfalsifiable"), this milestone's job is to **close
the gap from vision to reality**: take the pieces of the v26.7.4 vision (SPR: reality
addressing, no-private-abstraction, authority ledger, OCEL trace, cognitive breeds, human-
unavailable execution) that name a concrete mechanism, and either point at the existing code
that already implements it, or build the missing piece — using the smallest amount of new
code that closes the gap, reusing everything already in `crates/praxis-synthesis/src/`.

## What's already closed this milestone

**PROJ-301 (Reality Addressing Layer) is DONE**, not a future ticket: `src/reality.rs` now
exists — `RealityAddressRecord::bind` binds an admitted subject to `OWL-Time`
(`inXSDDateTimeStamp`), `GeoSPARQL` (`asWKT`), and `PROV-O` (`wasAttributedTo`) anchors read
straight from the already-parsed graph triples, refusing (`Refusal::RealityAddressIllFormed`)
a subject with zero anchors rather than returning a hollow "addressed" record. 4 tests green.
This directly answers the vision doc's "ContentAddress / RealityAddress / ReceiptAddress"
three-way split: content address and receipt address already existed
(`chatman_common::provenance::content_address`, `firing.rs`'s outer chain); reality address was
the one genuinely missing piece, and it's now real code, not a claim.

## Execution sequence & dependency graph

```
[PROJ-301 reality addressing]  DONE
               |
               v
[PROJ-302 no-private-abstraction gate]   DONE
               |
               v
[PROJ-303 authority ledger]              DONE
               |
               v
[PROJ-304 OCEL V2 event export]          DONE
               |
               v
[PROJ-305 cognitive breed registry]      (promote the v26.7.3 PROJ-206 doc mapping into a
               |                          const table so it's compiled, not just prose)
               v
[PROJ-306 human-unavailable audit]       (prove fire_hooks never blocks on live human input;
                                          reuses no_llm_runtime.rs's existing tripwire pattern)
```

---

## Ticket index

### 1. [ticket_301_reality_addressing.md](ticket_301_reality_addressing.md)
* **JIRA ID**: PROJ-301 — **STATUS: CLOSED**
* Reality-address binding to public ontologies (OWL-Time/GeoSPARQL/PROV-O), computed hash,
  zero-anchor refusal. `src/reality.rs`, 4 tests, wired into `lib.rs` exports and the
  `Refusal` enum.

### 2. [ticket_302_no_private_abstraction_gate.md](ticket_302_no_private_abstraction_gate.md)
* **JIRA ID**: PROJ-302 — **STATUS: CLOSED**
* Extend `graph::vocab_check`'s closed-world predicate table so every `wf:`/`hook:`/
  `prayer-kernel:`/`agent:` predicate addition going forward must be paired with an entry in
  a new `docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md`, checked by a test that greps the vocab
  tables against the doc. Reuses the existing closed-vocabulary-table pattern already in
  `graph.rs`/`hooks.rs`/`kernel.rs`/`agent_registry.rs` — no new mechanism, just a
  cross-check test tying private vocabulary growth to public-ontology justification.
* **Dependencies**: PROJ-301 (this is the doctrine PROJ-301 established, made structural).

### 3. [ticket_303_authority_ledger.md](ticket_303_authority_ledger.md)
* **JIRA ID**: PROJ-303 — **STATUS: CLOSED**
* Bind every fired action's outcome to an admitted authority source by reusing
  `RealityAddressRecord`'s `provenance_anchor` (PROV-O `wasAttributedTo`) as the authority
  predicate on the capability/action node, and refusing a `ground-action` firing whose action
  node has no provenance anchor. This is "AuthorityLedger" from the vision doc, implemented as
  a firing-time check reusing `reality.rs` and `firing.rs`, not a new ledger subsystem.
* **Dependencies**: PROJ-301.

### 4. [ticket_304_ocel_event_export.md](ticket_304_ocel_event_export.md)
* **JIRA ID**: PROJ-304 — **STATUS: CLOSED**
* A pure-function exporter, `firing::to_ocel_event`, that renders one `HookFiringReceipt` as
  an OCEL 2.0-shaped JSON event (event id = `outcome_hash`, object ids = bound capability/
  handler IRIs, timestamp = the record's `time_anchor` if a `RealityAddressRecord` was bound
  for that firing, otherwise omitted — no wall clock invented). Reuses the existing receipt
  fields; adds no new event model, no new receipt chain.
* **Dependencies**: PROJ-301, PROJ-303 (needs the authority anchor to populate OCEL's
  attribution-adjacent fields honestly).

### 5. [ticket_305_cognitive_breed_registry.md](ticket_305_cognitive_breed_registry.md)
* **JIRA ID**: PROJ-305
* Promote `docs/v26.7.3/COGNITIVE_BREED_MAPPING.md` (from ticket PROJ-206) into a small
  `pub const BREED_MODULE_MAP: &[(&str, &str)]` table in a new `src/breeds.rs`, with a test
  asserting every cited module path actually exists as a module in `lib.rs`. This makes the
  breed-to-code mapping compile-checked rather than doc-only prose that can silently drift.
  No new `Breed` trait or runtime dispatch — this is a documentation-integrity mechanism, not
  a new abstraction layer.
* **Dependencies**: the v26.7.3 PROJ-206 doc must exist first.

### 6. [ticket_306_human_unavailable_audit.md](ticket_306_human_unavailable_audit.md)
* **JIRA ID**: PROJ-306
* Prove the "post-RQ execution requires admitted authority, not live human interaction" claim:
  a test asserting `fire_hooks` and `replay_firing` contain no blocking I/O, no stdin read, no
  interactive prompt — reusing the existing `tests/no_llm_runtime.rs` symbol-absence-tripwire
  pattern, extended to also assert absence of `std::io::stdin`/`dialoguer`/interactive-crate
  symbols in `crates/praxis-synthesis/src/`.
* **Dependencies**: PROJ-303 (the authority-ledger check is what makes "no human needed" true
  rather than merely "no human called," so it should land first).

---

## What is explicitly NOT built this milestone (see v26.7.3's Refuse list, still binding)

CapabilityAtlas / ProxyAttributionLedger / ResourceGravityEngine / ObjectiveCommissioningEngine
/ PresenceProjectionGateway / AdvisorConflictRadar / Sovereign Compile Mode / Blue River Dam
business strategy / wealth-market wedge — these remain product-vision and business-strategy
language with no concrete mechanism distinguishable from PROJ-301..306 above. If a real,
specific use case demands one of these, it gets its own scoped ticket at that time — not built
speculatively now.

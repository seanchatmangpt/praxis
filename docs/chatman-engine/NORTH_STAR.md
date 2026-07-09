# Chatman Engine — North Star

What v26.7.9 is ultimately for. Every agent working on this system should read this before
optimizing locally: the implementation choices below are not style — they are the load-bearing
mechanisms of the end goal.

## The bet

Every current system has a gap between *stated rules* and *executed behavior*, patched by
humans and vibes. The engine's bet is that this gap can be closed with types, hashes, and
refusals. If the bet pays off even partially, "the software does what the documents say" stops
being an aspiration and becomes a build artifact.

## What the architecture implies at its logical extreme

1. **Organizations that can prove themselves.** "No unreceipted actuation" + input-specific
   replay means every state change carries a cryptographic chain back to admitted graph
   meaning. An auditor doesn't read reports — they re-execute the process. Compliance becomes
   `verify_replay()`; fraud becomes a hash mismatch, not a deposition.
2. **Business logic goes extinct as a category of software.** Rules live in RDF; the engine
   lowers them mechanically to the cheapest lawful execution surface. Changing the business
   means editing triples, not shipping a release. "We implemented the policy wrong in the
   code" ceases to be a bug class — the policy IS the executable.
3. **Agent swarms that cannot go rogue.** Agents are witnesses, not authorities; they cannot
   override profile gates; nondeterministic operators refuse without receipt metadata. The
   type system makes unauthorized actuation a compile error — AI autonomy with the safety
   profile of a mechanical interlock. The sealed `AdmittedTransition` is digital
   lockout-tagout.
4. **Governance at nanosecond speed.** The 8-constraint hot path (one byte, 256-state
   precomputed tables, branchless masks, ≤8 ticks) is lawful admission at hardware speed.
   Today governance is slow and speed is ungoverned; the engine collapses the dichotomy —
   law that runs faster than the thing it governs.
5. **Replayable institutions.** OCEL as observed execution material + deterministic replay
   means any process — supply chain, election count, clinical trial — can be handed to a
   skeptic as (inputs, receipts) and re-derived bit-for-bit. Trust becomes recomputation.
   The Refusal taxonomy proves not just what happened but *what was refused and why* — the
   thing no current audit system captures.
6. **The diagram atlas as error-correcting code for meaning.** 240 projections of one
   hyperdimensional semantic object so that no single interpretation — human or LLM — can
   silently reintroduce business logic through information loss. The spec cannot drift from
   the system because the system is manufactured from the spec, and the projections gauge
   the drift.

## What this means for your lane

- A shortcut that weakens a receipt, widens a refusal into a catch-all, adds an unwrap, or
  lets a test pass vacuously is not a small compromise — it reopens the exact gap the whole
  system exists to close.
- Determinism, typed refusals, sorted-before-hashed material, and fail-loud tests are the
  product, not overhead on the product.
- The auditor's verdict (ADMITTED_DRY_RUN_PUBLISHABLE | PARTIAL | REFUSED) is the only
  status that matters. Builders never self-grade; that discipline is item 1 above applied
  to ourselves.

## See Also

- `DEFINITION_OF_DONE.md` — the concrete gates this vision compiles down to
- `ceng/PROJECT.md` — architecture layers
- `diagrams/atlas/` — the 240-projection design atlas; `diagrams/asbuilt/` — as-built gauges

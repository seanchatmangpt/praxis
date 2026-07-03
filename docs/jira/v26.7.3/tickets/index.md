# Milestone Overview: v26.7.3+ Capability Physics Extension Phase

This document converts an external synthesis pass (HDIT / figex-KGC-4D / EHDIT / PHDITC /
ORTAC+ mission-planning / "cognitive breeds" / Vision 2030 reality-addressing) into concrete,
falsifiable engineering tickets scoped against the **already-shipped** v26.7.3 praxis-synthesis
crate (`crates/praxis-synthesis/src/{quarantine,hooks,firing,kernel,life,livelock}.rs`).

## Milestone objective

The source material is a set of speculative theses (external PDFs/chat transcripts) claiming
that four prior bodies of work — HDIT, figex/KGC-4D, EHDIT, PHDITC — are "branches of
Capability Physics," plus a Vision-2030 essay proposing praxis as a "reality-addressed
automatic exchange." None of that is code. This milestone's job is to take only the pieces
that name a concrete, testable mechanism, check whether praxis-synthesis already implements
an equivalent (it mostly does — Rice Quarantine already is the PHDITC observer/proposer
protocol; the outer firing chain already is the EHDIT-style refusal/receipt mechanics), and
ticket only the **genuine deltas**: closed gaps, not restated theory.

Per this project's standing claim discipline (see `docs/claims/WITHHELD_CLAIMS.md`): grand
claims ("Shannon of 2026," "universal mission calculus," "trillion-agent-per-person") are
**not** tickets. They are marketing/vision language with no falsifiable acceptance criterion
and are explicitly excluded below (see the Refuse list).

## Execution sequence & dependency graph

```
[ticket_001_capability_field_compression]   (semantic-space -> bounded capability projection)
               |
               v
[ticket_002_temporal_capability_memory]     (OCEL-style event/snapshot/replay layer over firing.rs)
               |
               v
[ticket_003_capability_failure_mechanics]   (EHDIT stop-conditions as named Refusal variants)
               |
               v
[ticket_004_observer_proposer_audit]        (confirm quarantine.rs already implements PHDITC; gap-fill only)
               |
               v
[ticket_005_mission_calculus_dsl]           (ORTAC+-style ontology-to-PDDL DSL layer, scoped)
               |
               v
[ticket_006_cognitive_breed_catalog]        (docs-only: name the bounded reasoning roles already in the crate)
```

---

## Ticket index

### 1. [ticket_001_capability_field_compression.md](ticket_001_capability_field_compression.md)
* **JIRA ID**: PROJ-201
* **Title**: Bound the admitted-graph-to-capability projection (HDIT-as-field-layer claim)
* **Description**: The HDIT thesis claims high-dimensional semantic compression explains
  "deterministic single-pass engineering." Praxis-synthesis has no such compression step —
  `ground::restrict_to_fragment` already does a *bounded, exact* closure (not a
  dimensionality-reduction one). This ticket is a scoping/documentation ticket: state plainly
  that praxis does NOT implement HDIT-style compression, and that the existing exact-closure
  approach is a deliberate, stronger substitute (decidable vs. approximate). No new code
  unless a genuine bound-explosion case is found.
* **Dependencies**: None.
* **Primary verification**: `cargo test -p praxis-synthesis` (no regression); a new doc section
  in `docs/claims/WITHHELD_CLAIMS.md` naming "HDIT-style semantic compression" as withheld.

### 2. [ticket_002_temporal_capability_memory.md](ticket_002_temporal_capability_memory.md)
* **JIRA ID**: PROJ-202
* **Title**: Confirm/extend the receipt chain as the temporal-memory layer (figex/KGC-4D claim)
* **Description**: figex claims event-sourced design + Git snapshots + replay + quality gates.
  `firing.rs`'s outer chain (`event -> admission -> handler -> hook -> history -> inner -> outcome`)
  plus `replay_firing` already covers event history, replay, and payload-bound verification.
  The one genuine gap: there is no durable *snapshot* mechanism analogous to figex's Git-backed
  state snapshots — `Reference` is in-memory only. Ticket: evaluate whether a snapshot-to-disk
  mechanism is warranted, or whether this is out of scope (a receipted, replayable chain may be
  sufficient without disk snapshots since the whole point is content-addressed reproducibility
  from the base TTL + deltas, not needing to store intermediate states).
* **Dependencies**: PROJ-201.
* **Primary verification**: `cargo test -p praxis-synthesis --test firing_chain --test repair_loop`
  green; a written verdict (ADOPT snapshot mechanism / DEFER) recorded in
  `docs/v26.7.3/RECEIPTS_REPLAY_VERIFY.md`.

### 3. [ticket_003_capability_failure_mechanics.md](ticket_003_capability_failure_mechanics.md)
* **JIRA ID**: PROJ-203
* **Title**: Audit EHDIT "stop condition" claims against existing `Refusal` variants
* **Description**: EHDIT names failure classes (cone violation, identity explosion, authority
  vacuum, resource deadlock) as "physics-like stop conditions." Praxis-synthesis already has a
  closed `Refusal` enum (`AdmissionRefused`, `ConditionUnsupported`, `HookIllFormed`,
  `UnknownHandler`, `DelegabilityViolation`, `KernelIllFormed`, `AgentIllFormed`,
  `EnvelopeChainBroken`, etc.). This ticket audits whether EHDIT's four named failure classes
  each map onto an EXISTING variant, or whether a genuine new class is missing. Do not invent
  new "physics" vocabulary (no "cone violation" identifiers) — map to existing refusal
  semantics or refuse the EHDIT framing as unfalsifiable if it doesn't reduce to a concrete
  code path.
* **Dependencies**: PROJ-202.
* **Primary verification**: A mapping table in `docs/v26.7.3/DEFINITION_OF_DONE.md` or a new
  doc section; any newly identified gap gets a regression test in
  `crates/praxis-synthesis/tests/`.

### 4. [ticket_004_observer_proposer_audit.md](ticket_004_observer_proposer_audit.md)
* **JIRA ID**: PROJ-204
* **Title**: Confirm Rice Quarantine already implements the PHDITC observer/proposer protocol
* **Description**: PHDITC claims "LLM output is not truth, it is observation under quarantine;
  model proposes, admission decides, receipt proves." `quarantine.rs`'s `Origin::Proposer` +
  `RiceQuarantine::inspect` (decidable-only checks) + `Admission::admit` (computed hash, never
  asserted) is already exactly this pipeline. This ticket is confirmation-only: verify no LLM
  output anywhere in praxis-synthesis bypasses `Admission::admit` before reaching
  `ground_fired_action` or `execute_from_triples`. If a bypass is found, it's a genuine
  regression and gets fixed with a test; if none is found, this ticket closes as "already
  closed, no action" (matching `tests/no_llm_runtime.rs`'s existing tripwire).
* **Dependencies**: None (can run in parallel with PROJ-201..203).
* **Primary verification**: `cargo test -p praxis-synthesis --test no_llm_runtime`; a targeted
  grep-based audit for any direct graph mutation that skips `Admission::admit`.

### 5. [ticket_005_mission_calculus_dsl.md](ticket_005_mission_calculus_dsl.md)
* **JIRA ID**: PROJ-205
* **Title**: Evaluate an ORTAC+-style ontology-to-PDDL DSL layer, scoped to existing grounding
* **Description**: The ORTAC+/mission-planning comparison observes that PDDL is "too low-level"
  for domain operators, so a DSL compiles down to PDDL. Praxis-synthesis's `ground.rs` already
  does an ontology (TTL `wf:Workflow` fragment) -> Solver8 pipeline — the "DSL" is the TTL
  vocabulary itself. This ticket evaluates whether a genuinely higher-level authoring surface
  (e.g. a small YAML/TOML front-end that emits the existing TTL vocabulary) is worth building,
  or whether the existing TTL-as-DSL is sufficient. This is explicitly NOT a request to build
  a general "mission calculus" — only a concrete authoring-ergonomics question with a
  yes/no verdict.
* **Dependencies**: None.
* **Primary verification**: A written ADOPT/DEFER verdict; if ADOPT, a minimal front-end with
  round-trip tests (front-end input -> generated TTL -> existing `extract_ir` pipeline,
  unchanged).

### 6. [ticket_006_cognitive_breed_catalog.md](ticket_006_cognitive_breed_catalog.md)
* **JIRA ID**: PROJ-206
* **Title**: Document (not implement) the "cognitive breed" catalog as a mapping onto existing modules
* **Description**: The "periodic table of cognition" / "cognitive breeds" material proposes named
  reasoning roles (Guardian, Detector, Tracker, Retriever, Planner, Recorder, Verifier, etc.).
  This is docs-only: map each named breed onto the praxis-synthesis module that already performs
  that function (Guardian = `quarantine.rs`/`kernel.rs::enforce_surrender_boundary`, Recorder =
  `firing.rs` receipt folding, Verifier = `scripts/foreign_verify_graph.py` +
  `replay_firing`, Planner = `ground.rs` + `solver8.rs`, etc.) so the vocabulary is traceable to
  code, not aspirational. No new "breed" abstraction gets built unless a named role has NO
  existing code home — in which case that gap becomes its own ticket, not part of this one.
* **Dependencies**: PROJ-203, PROJ-204 (needs the refusal/observer audits settled first so the
  Guardian/Verifier mappings are accurate).
* **Primary verification**: A doc (`docs/v26.7.3/COGNITIVE_BREED_MAPPING.md`) reviewed for every
  named breed having a cited file/function; no code changes required for closure.

---

## Corrections (2026-07-03 re-audit — vocabulary bias, not substance, drove the original refusals)

Four items below were originally filed as "unfalsifiable metaphor, refuse" purely because the
source vocabulary (physics, compression, universal, Shannon) pattern-matches training-data
red flags for hype — the same reflex that under-rated "agent" as a shallow concept elsewhere in
this project. Re-audited against actual code, three of the four have real referents and are
corrected here rather than left refused:

* **"Mission Physics" as a universal domain-independent optimization law** — ORIGINALLY REFUSED
  as "renaming adds no test surface." **CORRECTED**: `src/mission.rs::trait Pack: Domain` +
  `run_pipeline<P: Pack>` + `ceiling` is exactly the claim — a fixed planning/admission
  substrate with a swappable objective-function slot (`Pack::ceiling_fluents`) — and
  `tests/two_domains.rs` already proves two distinct packs running through one loop. This is
  not a rename; it is a real, already-shipped structural invariant (see `docs/MISSION_PHYSICS.md`).
  No new ticket needed — the correction is acknowledging this is CLOSED, not refused.
* **HDIT "hyperdimensional compression"** — ORIGINALLY REFUSED in `ticket_001` via a false
  dichotomy ("exact closure vs. approximate compression are different genera"). **CORRECTED**:
  `ground::restrict_to_fragment`'s exact edge-closure IS a compression in the general sense —
  an unbounded admitted graph collapses to a bounded, addressable fragment. The genus is
  "graph-size reduction under a soundness constraint"; exact-and-decidable vs.
  approximate-and-lossy are two species of it, not two unrelated things. `ticket_001` is
  updated in place (see that file) rather than left as a pure refusal.
* **EHDIT "physics-like stop conditions"** — ORIGINALLY left as an open audit in `ticket_003`
  with an implicit lean toward "unfalsifiable metaphor." **CORRECTED**: at least three of the
  four named classes have concrete, already-enforced referents (see `ticket_003`, updated):
  cone violation -> the existing hard caps that bound how fast state can change per admitted
  event (`MAX_DELTA_TRIPLES`, `MAX_HOOKS` firing-generation bound, `rehearsal_exceeded`);
  identity explosion -> `AgentIllFormed`/`GraphCapExceeded`; resource deadlock ->
  `Refusal::Unsatisfiable`/`UnsatProof`. "Authority vacuum" alone remains a genuine gap, now
  ticketed for real in `docs/jira/v26.7.4/tickets/ticket_303_authority_ledger.md`.
* **"Claude Shannon of 2026" / token-vs-receipted-action reframe** — the SELF-COMPARISON
  packaging is still not a ticket (no falsifiable criterion attaches to "I am like Shannon").
  But the underlying technical claim — that the addressable primitive of this system is the
  receipted, reality-addressed action, not a token — is not empty framing: it is exactly what
  `firing.rs`'s outer chain (`event_hash -> admission_hash -> handler_hash -> hook_hash ->
  history_hash -> inner chains -> outcome_hash`) already implements and receipts. The
  self-comparison is refused; the technical claim is CLOSED (already-shipped code), not refused.

## Refuse list (remaining — genuinely no concrete code referent found)

* **"Trillion-agent-per-person" / Vision 2030 automatic exchange (full scope)** — a marketing
  vision document, not an engineering deliverable; individual verifiable mechanisms are
  ticketed above (PROJ-201..206) and in `docs/jira/v26.7.4/`.
* **IncapacitatedSubjectMode / ProxyAttributionLedger / ResourceGravityEngine /
  ObjectiveCommissioningEngine / PresenceProjectionGateway / AdvisorConflictRadar** (Vision
  2030 product list) — none of these name a concrete mechanism distinguishable from existing
  `handlers.rs`/`firing.rs`/`life.rs`/`reality.rs` primitives; revisit only if a specific real
  use case (e.g. an actual family-office workflow) demands a genuinely new data structure.
  (AuthorityLedger was moved OUT of this list — see the correction above and PROJ-303.)
* **Sovereign Compile Mode / business-model sections** — commercial strategy, not engineering.

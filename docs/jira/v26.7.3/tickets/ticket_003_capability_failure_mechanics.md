# Ticket: Audit EHDIT "Stop Condition" Claims Against Existing Refusal Variants

## Title
Map EHDIT failure-theorem classes onto the existing `Refusal` enum, or refuse the framing (PROJ-203)

## Description
The EHDIT ("Enterprise HDIT") material claims enterprise programs fail when "physics-like stop
conditions" are violated, naming classes such as cone violation, inertia, curvature, identity
explosion, and authority vacuum. Praxis-synthesis already has a closed, append-only `Refusal`
enum (`crates/praxis-synthesis/src/lib.rs`) with concrete variants: `AdmissionRefused`,
`ConditionUnsupported`, `HookIllFormed`, `UnknownHandler`, `DelegabilityViolation`,
`KernelIllFormed`, `AgentIllFormed`, `EnvelopeChainBroken`, `GraphCapExceeded`,
`UnknownPredicate`, `WorkflowIllFormed`, and others.

This ticket performs a strict audit: for each of EHDIT's four named failure classes, either
(a) identify the EXISTING `Refusal` variant it reduces to (e.g. "authority vacuum" plausibly
maps to `UnknownHandler` or `DelegabilityViolation`; "identity explosion" plausibly maps to
`AgentIllFormed` or a graph-cap violation), or (b) conclude the EHDIT class does not correspond
to any concrete, testable failure mode in this codebase and is therefore unfalsifiable
metaphor, not an engineering requirement — in which case it is refused, not implemented.

No new "physics" vocabulary (no `ConeViolation`, `CurvatureExceeded`, etc. as identifiers) may
be added to the `Refusal` enum as a result of this ticket unless the audit finds a genuinely
missing, concretely-reproducible failure mode with no existing variant — in which case the new
variant gets a plain engineering name consistent with the existing style, not EHDIT's metaphor
vocabulary.

## Audit result — CORRECTED 2026-07-03 (see index.md's "Corrections" section)
The original pass here left this as an open, lean-toward-refuse audit. Re-run against the
actual code (`crates/praxis-synthesis/src/lib.rs`'s `Refusal` enum, `delta.rs`, `hooks.rs`,
`livelock.rs`), three of EHDIT's four named classes have concrete, already-enforced referents
— this was a vocabulary-triggered under-classification, not a substance-based one:

| EHDIT class | Verdict | Referent |
|---|---|---|
| Cone violation (state cannot change faster than a bounded propagation rate) | MAPPED | `delta.rs::MAX_DELTA_TRIPLES` (64/event cap), `hooks.rs::MAX_HOOKS` (firing-generation bound), `livelock.rs::rehearsal_exceeded` — all three are already-enforced rate/velocity bounds on how much state can move per admitted step. |
| Identity explosion (entities lose a shared basis for comparison) | MAPPED | `Refusal::AgentIllFormed` (agent registry shape law) and `Refusal::GraphCapExceeded` (unbounded triple growth). |
| Resource deadlock (partial-order schedule cannot resolve) | MAPPED | `Refusal::Unsatisfiable` / `Refusal::UnsatProof` (Solver8's certified-impossibility path, `sequence.rs`/`solver8.rs`). |
| Authority vacuum (action force without a lawful authority vector) | GENUINE GAP, now ticketed | No existing variant enforces "an action must cite its authority source before firing" — `handlers.rs`/`firing.rs` check WHO may execute (delegability), not whether authority was DECLARED at all. This is real and is now `docs/jira/v26.7.4/tickets/ticket_303_authority_ledger.md` (PROJ-303), reusing `reality.rs`'s PROV-O anchor rather than inventing new vocabulary. |

## Acceptance Criteria
- [x] Mapping table above, citing existing variants for 3/4 classes.
- [x] Zero new physics-metaphor identifiers added to `crates/praxis-synthesis/src/lib.rs`.
- [x] The one genuine gap (authority vacuum) ships as PROJ-303 in v26.7.4: a plain-named
  refusal check reusing `Refusal::WorkflowIllFormed`, not a new "AuthorityVacuum" identifier.
- Full suite green after any change: `cargo test -p praxis-synthesis` (unaffected by this
  ticket itself, since it added no code — the code lands under PROJ-303).

## Dependencies
PROJ-202.

## Verification Mechanism
1. Read `crates/praxis-synthesis/src/lib.rs`'s `Refusal` enum in full via the Read tool before
   claiming any mapping (do not map from memory).
2. `cargo test -p praxis-synthesis` — green before and after.
3. `cargo clippy -p praxis-synthesis --all-targets -- -D warnings` — clean if any variant added.

# PROJ-760: AtomVM Wrapper — Execute AIR Transition Semantics

Rail E (Equivalence), part 1/3. PRD `docs/jira/v26.7.11/PRD.md:1071-1072` (§22, "AtomVM
wrapper → differential corpus → semantic drift refusal") and §7.9 lines 425-429:

> AtomVM SHALL execute the same AIR transition semantics.
> The product SHALL NOT maintain a separate semantic implementation.

## Scope

Replace the AtomVM-side wrapper so it actually drives the AIR transition core, instead of the
unrelated code currently checked in under that name. Concretely:

- Rewrite `apps/atomvm_runner/src/atomvm_runner.erl` so its exported functions call into the
  pure Erlang transition core (`apps/air_core/src/air_core.erl`, Rail C output) rather than the
  current `start_cosmic_inflation/1`, `spawn_multiverse/1`, `reverse_heat_death/1`,
  `generate_negative_entropy/2` exports, which spawn dummy processes that sleep and print
  "Heat death successfully reversed" — no reference to AIR, no state digest, no result digest,
  no refusal class, no command sequence.
- Wire a real build/test entrypoint (a `just` recipe, per this repo's build-hygiene rule) for
  the `apps/atomvm_runner`, `apps/arazzo_atomvm`, `apps/air_core` Erlang tree. None exists today:
  no `rebar.config` anywhere outside `_build/`, `rebar.lock` at repo root is the empty list `[]`,
  and no `justfile` recipe references rebar/atomvm/otp_runner/erlang. The `.beam` files under
  `apps/` were compiled ad hoc, outside any tracked recipe.
- Retire `apps/arazzo_atomvm/PROOF_OF_EQUIVALENCE.md`'s prose "proof by structural induction" as
  the equivalence evidence of record — it is a written argument, not machine-checked, and cites
  no test run. PROJ-761's differential corpus is the actual evidence surface.

Out of scope: the differential corpus itself (PROJ-761) and the drift-refusal type (PROJ-762).

## Dependencies

- Rail C — pure Erlang transition core (`apps/air_core`, PRD §7.6-7.8) must expose the AIR
  state/event model this wrapper calls into. Not yet ticketed in this range.
- Rail D — OTP supervision path (PRD §7.7) is the equivalence counterpart this wrapper must
  match. Not yet ticketed in this range.

## Status

OPEN. File targets and the missing-build-entrypoint gap are concretely identified from this
session's reconciliation of PRD §7.9 (status: MOCKED); no further audit is needed before a
developer can start. Direct evidence:
`crates/wasm4pm-arazzo/src/compile.rs:140-146` — the current AIR compiler's own doc comment
states "the planned AIR execution architecture (Erlang transition core, OTP/AtomVM runners) —
none of that exists in this repo today."

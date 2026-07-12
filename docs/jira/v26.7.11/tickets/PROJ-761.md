# PROJ-761: OTP/AtomVM Differential Conformance Corpus

Rail E (Equivalence), part 2/3. PRD `docs/jira/v26.7.11/PRD.md:1071-1072` (§22, "differential
corpus" clause), §7.9 lines 431-436, requirement 24 (line 832: "OTP/AtomVM differential
conformance corpus"), and DoD line 1096 ("AtomVM passes the shared semantic conformance corpus").

> For identical AIR and identical ordered admitted event corpus, OTP and AtomVM SHALL produce
> equivalent: state digest; result digest; refusal class; command sequence.

## Scope

- Build an ordered, admitted-event corpus (a shared fixture set) that can be replayed against
  both the OTP runner and the PROJ-760 AtomVM wrapper for the same AIR artifact.
- Build a differential harness that runs the corpus through both runners and asserts equality
  of the four dimensions named in PRD 431-435: state digest, result digest, refusal class,
  command sequence. None of these four comparisons exist anywhere in the repo today.
- Model the harness on the pattern already working for other rails at `tests/differential.rs`
  (a real, functioning differential harness for PDDL planners and POWL/Petri-net conformance,
  per its header at `tests/differential.rs:1-20`) — that file currently has zero mentions of
  AtomVM or OTP and does not cover this rail.

Out of scope: making the AtomVM wrapper itself correct (PROJ-760) and the refusal raised on a
detected divergence (PROJ-762).

## Dependencies

- PROJ-760 — the AtomVM wrapper must actually execute AIR transition semantics before its
  output is meaningful to diff against OTP.

## Status

**ALIVE.** Built and independently verified later in this same session (the PLANNED framing
above describes this ticket's starting state; re-verified fresh, not taken on report).
`apps/arazzo_runner/test/arazzo_runner_atomvm_differential_test.erl` (530 lines) now exists:
a 6-event ordered corpus (linear segment, AND-join, one genuine timeout failure) driven through
both the OTP path (`arazzo_runner_workflow` + broker) and the AtomVM path
(`arazzo_atomvm_workflow` directly), asserting equality across all 4 PRD §7.9 dimensions (state
digest, result digest, refusal class, command sequence). Command sequence — the one dimension
AtomVM doesn't natively expose — is captured via standard BEAM call tracing
(`erlang:trace/3`), not a NIF or source modification.

Direct evidence, re-verified fresh this session:

- `grep -n "?assertEqual\|?assert(" apps/arazzo_runner/test/arazzo_runner_atomvm_differential_test.erl | wc -l`
  → 14 real assertions, including golden-digest comparisons (`?GOLDEN_STATE_DIGEST`,
  `?GOLDEN_RESULT_DIGEST`) and OTP-vs-AtomVM cross-path equality checks
  (`?assertEqual(CmdOtp, CmdAtom)`, `?assertEqual(StateOtp, StateAtom)`,
  `?assertEqual(ResultOtp, ResultAtom)`, `?assertEqual(RefusalOtp, RefusalAtom)`) — this is not
  a vacuous pass; it fails on divergence.
- `rebar3 eunit --module arazzo_runner_atomvm_differential_test` → `3 tests, 0 failures`.

Disclosed, unchanged scope: one corpus (6 events), not an exhaustive AIR-program equivalence
proof — the event translator only covers `result`/`timeout` reaction classes. PROJ-762 (the
typed refusal fired on detected divergence) remains PLANNED, not built by this ticket.

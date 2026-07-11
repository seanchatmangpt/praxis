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

PLANNED. This session's reconciliation of PRD §7.9 (status: MOCKED) found no differential
harness, no shared corpus, and no build entrypoint for this rail: `grep -rniI differential . |
grep -v docs/jira` and `grep -rniI 'atomvm' . | grep -iE 'test|corpus|differential|equival'`
(repo root, excluding `_build/`) returned no matches outside this ticket's own citations. The
sole existing test, `apps/arazzo_atomvm/test/arazzo_atomvm_SUITE.erl:1-11`, has a body of
`runner_equivalence_test() -> ok.` with a comment admitting the comparison was never
implemented ("we cannot inspect the internal state of gen_statem easily"). This is greenfield
work, not a scoped fix — PLANNED, not OPEN.

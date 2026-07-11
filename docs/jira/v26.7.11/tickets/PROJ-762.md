# PROJ-762: Semantic Drift Refusal

Rail E (Equivalence), part 3/3. PRD `docs/jira/v26.7.11/PRD.md:1071-1072` (§22, "semantic drift
refusal" clause), with the divergence conditions defined at §7.9 lines 431-436 and the negative-
fixture pattern required by requirement 25 (line 833: "Negative fixtures for every prohibited
authority escape").

## Scope

- Define a typed refusal (per this repo's `Refusal`-variant discipline — specific name, not
  "Error") that fires when PROJ-761's differential harness detects that OTP and AtomVM diverge
  on any of the four equivalence dimensions (state digest, result digest, refusal class, command
  sequence) for identical AIR and identical event corpus.
- Add a negative fixture that seeds an intentional divergence between the two runners and
  asserts the refusal fires — mirroring the "authority-escape negative fixtures refuse" DoD
  criterion (PRD line 1104) applied to this rail specifically.

Out of scope: the AtomVM wrapper (PROJ-760) and the differential harness/corpus that computes
the four digests this refusal consumes (PROJ-761).

## Dependencies

- PROJ-761 — the differential harness must exist and be able to compute/compare the four
  equivalence dimensions before a refusal can be raised on a detected divergence between them.

## Status

PLANNED. This session's reconciliation of PRD §7.9 (status: MOCKED) found no comparison logic
of any kind between OTP and AtomVM output (see PROJ-761's status evidence), and therefore no
refusal type could exist to consume it — none was found under `crates/*/src/lib.rs` or the
Erlang tree. Nothing to attach this refusal to exists yet, so this ticket cannot start ahead of
PROJ-761.

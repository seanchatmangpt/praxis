# Proof of Equivalence: AtomVM Actor Loop vs. OTP gen_statem Runner

> **UNVERIFIED — this is a design sketch, not a proof.** It has no machine-checked backing
> (see PROJ-769, docs/jira/v26.7.11/tickets/index.md, for the real Lean formalization target).
> Its own premise does not currently match the codebase: the actual OTP runner
> (`apps/arazzo_runner/src/arazzo_runner_workflow.erl`) is a hand-rolled `receive` loop, not
> `gen_statem` as assumed throughout section 2.1 below. `air_core:transition/2` also does not
> currently return the `{ok, S'}` / `{io_request, Req, S'}` result shape this document assumes
> (see `apps/air_core/src/air_core.erl`) — that is PROJ-755/756 scope, landed, but under the
> `{context(), [command()]}` shape, not the one this document assumes. Treat the structural
> argument below as a target design, not a verified claim, until PROJ-761 (the real differential
> conformance corpus) exists and actually compares the two runners.
>
> **PROJ-760 retires this document as the equivalence evidence of record.** A prose argument by
> structural induction, however carefully written, is not machine-checked and cites no test run
> — it cannot be cited as evidence that the OTP and AtomVM wrappers are equivalent. The actual
> evidence surface is PROJ-761's differential conformance corpus (shared ordered admitted-event
> corpus, comparing state digest, result digest, refusal class, and command sequence across both
> runners). This document is kept as a design sketch only.
>
> **PROJ-761 status: ALIVE for the corpus it covers.**
> `apps/arazzo_runner/test/arazzo_runner_atomvm_differential_test.erl` drives the OTP
> (`arazzo_runner_workflow`) and AtomVM (`atomvm_runner`/`arazzo_atomvm_workflow`) paths through
> one identical ordered admitted-event corpus (a linear segment, a real AND-join, and one genuine
> failure/refusal) and asserts state digest, result digest, refusal class, and command sequence
> are identical between them — verified (`rebar3 eunit`, 5 consecutive full-suite runs
> byte-identical). This is real evidence for that one corpus, not a general equivalence proof
> covering every AIR program shape; see that test file's own module doc for the exact method
> (including a disclosed, real asymmetry: the AtomVM wrapper computes but discards air_core's
> dispatch_step commands, so command sequence had to be captured via Erlang call tracing rather
> than a native accessor on that side).

## 1. Theorem

Let $S$ be the pure state of an Arazzo workflow, defined as `context()` in `air_core`.
Let $T : E \times S \rightarrow R$ be the pure transition function `air_core:transition/2`, where $E$ is the set of events and $R$ is the set of results (e.g., `{ok, S'}`, `{io_request, Req, S'}`, etc.).

The OTP runner (implemented via `gen_statem`) and the AtomVM runner (implemented via a pure `receive ... end` actor loop) are strictly isomorphic in their trace of $S$ and the resulting AIR semantics and receipts.

## 2. Structural Mapping

### 2.1 State Custody
- **OTP Runner**: The workflow state $S$ is stored within the `gen_statem` `Data` map under the key `core_state`. State transitions are handled by returning `{keep_state, Data#{core_state => S'}}` or `{next_state, StateName, Data#{core_state => S'}}`.
- **AtomVM Runner**: The workflow state $S$ is passed directly as the argument `CoreState` to the recursive tail-call functions `loop/2` and `loop_waiting_for_io/2`.

### 2.2 Event Dispatch and Transition
Both runners delegate all domain logic to the identical pure function $T$.
- When an event $e \in E$ is received, both runners compute $r = T(e, S)$.
- If $r = \{ok, S'\}$, the OTP runner returns `{keep_state, Data'}` (or `{next_state, idle, Data'}`), maintaining the idle state. The AtomVM runner tail-calls `loop(WorkflowId, S')`, maintaining the idle state.
- If $r = \{io\_request, Req, S'\}$, both runners execute the side-effect `spawn_worker(Req)`. The OTP runner transitions to `waiting_for_io` via `{next_state, waiting_for_io, Data'}`. The AtomVM runner transitions to `waiting_for_io` via tail-calling `loop_waiting_for_io(WorkflowId, S')`.

### 2.3 Receipt Generation
Receipts in the Arazzo AIR model are generated strictly within `air_core:transition/2` (specifically accumulated in `Context#{history}`).
Because both runners act strictly as custodians of $S$ and do not modify $S$ outside of $T$, the history and receipts accumulated within $S$ are bit-for-bit identical for any identical trace of events $E$. 

## 3. Conclusion
By structural induction over the trace of events, we have proven that the AtomVM runner's pure actor loop generates the exact same state transitions, AIR semantics, and cryptographic receipts as the `gen_statem` OTP runner, while satisfying the constraint of avoiding OTP behaviors for execution on constrained microcontrollers.

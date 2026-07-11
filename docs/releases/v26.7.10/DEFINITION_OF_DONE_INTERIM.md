# DEFINITION_OF_DONE_INTERIM — Recursive Workflow v26.7.10 (interim milestone)

⚠️ SUPERSEDED (2026-07-10): this document is the prior v26.7.10 DoD, preserved verbatim as an
explicitly recorded **interim milestone**. It was closed at commit `31c236f` with all 11
markers derived TRUE (`RELEASE_CONTROL.md` §8, `DOD_SIGNOFF.md`). The governing DoD for
v26.7.10-revised is `DEFINITION_OF_DONE.md` in this directory (PROJ-730). Nothing below is
discarded — the closure stands as evidence of the substrate the revised DoD builds on.

Version: v26.7.10 (interim). Doctrine document pointed to by `RELEASE_CONTROL.md` (PROJ-606).
`RELEASE_CONTROL.md` remains the single control surface; if this document and
`RELEASE_CONTROL.md` disagree, `RELEASE_CONTROL.md` wins. Every clause below carries a status
from the no-overclaiming vocabulary (ALIVE / PARTIAL / MOCKED / UNVERIFIED / BLOCKED / REFUSED
/ PLANNED). Status verdicts are derived from evidence cited in `RELEASE_CONTROL.md`, never
asserted here independently.

## 1. Core sentence

Recursive Workflow is production-ready only when it can manufacture work, dispatch it across
system and human boundaries, admit the returned consequences, close parent-child obligations,
and autonomically continue until the enterprise goal reaches lawful terminal standing.

Status: UNVERIFIED as a whole. The manufacture/receipt/replay segment is ALIVE per
`RELEASE_CONTROL.md` §1; dispatch, admission of external consequences, parent-child closure,
and autonomic continuation are PLANNED (PROJ-608..622).

## 2. Behavioral definition of done

This is a behavioral DoD, not feature completion. The benchmark models a complete enterprise
through the lawful activities of a single operator — a topology claim, not an organizational
claim. Governing law: `A = μ(O*)` (see `docs/CHATMAN_EQUATION.md`).

The operator never:

1. Wonders what to do next.
2. Manually connects plans to tasks.
3. Reconstructs provenance.
4. Redraws workflows.
5. Determines ownership.
6. Routes evidence.
7. Reconciles completed work.
8. Loses replay.
9. Performs semantic glue the graph can construct.
10. Remembers open loops.
11. Polls external systems without a declared workflow.
12. Decides which compensation follows a failure.

Status: UNVERIFIED (workday-mode behavioral run is PLANNED, PROJ-608..622).

## 3. Governing autonomic loop

```text
admitted graph state
  → role inference
  → lawful next-state derivation
  → workflow manufacture
  → recursive decomposition
  → execution routing
  → internal actuation or external dispatch
  → consequence observation
  → evidence ingestion
  → conformance validation
  → OCEL construction
  → receipt
  → replay
  → standing update
  → next workflow manufacture
```

Formal statement:

```text
S_{n+1} = Admit(Observe(Dispatch(μ(S_n))))
R_n     = receipt(S_n, W_n, D_n, C_n, S_{n+1})
```

Terminal states: `GOAL_SATISFIED`, `REFUSED`, `BLOCKED`, `UNSUPPORTED`,
`HUMAN_ADMISSION_REQUIRED`.

Status: PARTIAL. The role-inference → manufacture → OCEL → receipt → replay arc is ALIVE per
`RELEASE_CONTROL.md` §1/§7; `Dispatch`, `Observe`, `Admit`, and loop continuation are
UNVERIFIED/PLANNED.

## 4. Hook-morphism law

Hooks are mandatory: inference without actuation is incomplete.

```text
H: (A, S) → (A', R)
∀h ∈ H, R = receipt(h(A));  ¬R ⟹ h has no standing
```

Zero-unreceipted-actuation, extended to dispatch and acceptance:

```text
{c : actuate(c) ∧ ¬(R ⊢ c)} = ∅
{d : dispatch(d) ∧ ¬(R_d ⊢ d)} = ∅
{c : acceptExternalConsequence(c) ∧ ¬Admitted(c)} = ∅
```

Checked by SPARQL over the emitted OCEL graph (`queries/metric-hook-actuations.rq`,
`queries/metric-dispatch-closure.rq` — on disk, delivered under PROJ-614).

Status: ALIVE at closure — `ZERO_UNRECEIPTED_ACTUATION` and `HOOK_ACTUATION_PROVEN` derived
TRUE (§14; evidence in `DOD_SIGNOFF.md`).

## 5. Dialect Registry Invariant

Every registered dialect defines: admitted inputs, manufactured outputs, authority boundary,
quarantine boundary, refusal vocabulary, receipt schema, replay semantics, executable routing.
A definition lacking any of these cannot be admitted into the registry. The registry is
executable law, not documentation.

Dialect roles:

- **PDDL** — admitted role-possibility.
- **POWL** — manufactured role-execution (canonical).
- **Datalog** — stable role/obligation derivation.
- **SPARQL CONSTRUCT** — workflow consequence materialization.
- **N3** — quarantined bounded refinement/refusal (cold route only, never actuation — mirrors
  `crates/praxis-graphlaw/src/chatman/router.rs`).
- **Arazzo** — external API workflow orchestration surface (registered dialect; does NOT
  replace POWL).
- **OCEL** — execution history.
- Humans are never semantic middleware.

Status: PARTIAL. PDDL/POWL/Datalog/SPARQL-CONSTRUCT/OCEL roles are ALIVE in the existing
benchmark per `RELEASE_CONTROL.md` §1; the registry as executable law, N3 quarantine wiring in
`cng`, and Arazzo are UNVERIFIED/PLANNED.

## 6. HookStanding lifecycle

Hooks are themselves manufactured artifacts, with lifecycle:

```text
DECLARED → REGISTERED → ADMITTED → AUTHORIZED → READY → EXECUTED → RECEIPTED → REPLAYABLE
```

`REFUSED` is reachable from any pre-terminal state.

Status: UNVERIFIED/PLANNED (PROJ-608..622). Note: ChatmanEngine adoption is deferred; this
release targets the TripleStore hook surface.

## 7. External dispatch doctrine

Three execution classes: `LOCAL_ACTUATION`, `EXTERNAL_MACHINE_DISPATCH`,
`EXTERNAL_HUMAN_DISPATCH`. Human work is a typed external execution surface, not an exception.

Broker exclusivity: dialect → manufactured action artifact → broker → local hook or external
dispatch adapter. No dialect, LLM, script, CLI, agent, or integration dispatches directly.

Dispatch contract required fields (missing field ⇒ refused before leaving the broker):

1. Dispatch id
2. Workflow instance id
3. Parent workflow id
4. Recursive depth
5. Target actor/system
6. Required role
7. Declared authority
8. Input artifact set
9. Expected output artifact set
10. Process model/activity identity
11. Deadline/timeout
12. Idempotency key
13. Correlation id
14. Callback/collection surface
15. Retry law
16. Escalation law
17. Compensation law
18. Refusal conditions
19. Receipt requirements
20. Replay requirements

Status: UNVERIFIED/PLANNED (PROJ-608..622). This release's external dispatch will be
loopback-real — a deterministic local outbox/inbox — with live network endpoints explicitly
out of scope. Synthesized human consequences are MOCKED-HUMAN.

## 8. Dispatch state machine and readmission

Dispatch state machine (no implicit completion):

```text
MANUFACTURED, DISPATCH_READY, DISPATCHED, ACKNOWLEDGED, IN_PROGRESS, RESULT_RETURNED,
ADMITTED, COMPLETED, REFUSED, TIMED_OUT, COMPENSATING, BLOCKED, UNKNOWN
```

External execution does not create standing. Return path:

```text
external result → candidate artifact → provenance verification → identity/correlation check
  → authority verification → structural validation → semantic conformance
  → admission or typed refusal
```

`O_external ≠ O*` until admission.

Status: UNVERIFIED/PLANNED (PROJ-608..622).

## 9. Callback/polling law

Transport (webhook, queue, email, API response, file arrival, DB event, OTEL signal, OCEL
event, human submission, polling) never determines standing; all returns enter the same
admission pipeline. Polling is a registered workflow activity with bounded frequency, timeout,
termination condition, and receipt. Unbounded polling is prohibited.

Status: UNVERIFIED/PLANNED (PROJ-608..622).

## 10. Recursive dispatch and parent-child closure

Children carry identity, parent, authority, depth, expected consequence, and receipt chain.
Depth, fan-out, and cost are bounded. Parent closure laws:

```text
ALL_CHILDREN_REQUIRED | ANY_CHILD_SUFFICIENT | QUORUM_REQUIRED | ORDERED_SUBSET_REQUIRED
| POLICY_DECIDES | FIRST_CONFORMANT_RESULT
```

Closure law is represented in the graph and evaluated through the registered dialect, never
inferred.

Status: UNVERIFIED/PLANNED (closure laws PROJ-608..622). Recursive attachment itself is ALIVE
at depth 2 per `RELEASE_CONTROL.md` §1 (`RECURSIVE_ATTACHMENTS=8`).

## 11. Compensation

Failed conformance produces a refusal as OCEL evidence plus declared retry, escalation, and
compensation. Compensation is itself a workflow — with authority, inputs, expected
consequence, receipt, and replay. Partial external execution is never silently discarded.

Status: UNVERIFIED/PLANNED (PROJ-608..622).

## 12. LLM edge-only

The LLM may: draft outbound work, summarize returned evidence, translate, formulate bounded
clarification questions, explain refusals.

The LLM may not: create standing, select undeclared recipients, invent authority, mark
external work complete, admit evidence, dispatch directly, bypass the broker.

Status: UNVERIFIED/PLANNED as an enforced boundary (PROJ-608..622); doctrine binding now.

## 13. Autonomic completion criterion — production chain

Done means the full `cng` chain runs autonomically end to end:

```text
import → admit → derive → CONSTRUCT → PDDL plan → POWL manufacture → recursive attachment
  → role inference → capability routing → broker selection
  → local actuation or external dispatch → acknowledgement tracking
  → external result ingestion → consequence admission → conformance → OCEL → receipt → replay
  → next standing state
```

Status: PARTIAL. Segments through recursive attachment/role inference/OCEL/receipt/replay are
ALIVE per `RELEASE_CONTROL.md` §1/§7; routing, broker, dispatch, ingestion, admission, and
loop continuation are UNVERIFIED/PLANNED.

## 14. Success markers

Each marker is derived from SPARQL over the emitted OCEL graph — never asserted.

All 11 markers derived TRUE via SPARQL on the closing-session `just cng-workday-verify` run
(seed=616, ticks=8, rpm=125) — command + output citation in `RELEASE_CONTROL.md` §8 and
`DOD_SIGNOFF.md`.

| Marker | Status (2026-07-10, closure) |
|---|---|
| `AUTONOMIC_LOOP_CLOSED` | ALIVE — VERIFIED-TRUE (`DOD_SIGNOFF.md` §3) |
| `EXTERNAL_WORKFLOW_DISPATCH_PROVEN` | ALIVE — VERIFIED-TRUE (loopback-real; no live network) |
| `EXTERNAL_RESULT_READMISSION_PROVEN` | ALIVE — VERIFIED-TRUE (loopback-real) |
| `RECURSIVE_CHILD_CLOSURE_PROVEN` | ALIVE — VERIFIED-TRUE (post `marker-child-closure.rq` fix) |
| `TIMEOUT_ESCALATION_PROVEN` | ALIVE — VERIFIED-TRUE |
| `COMPENSATION_WORKFLOW_PROVEN` | ALIVE — VERIFIED-TRUE |
| `ONE_PERSON_RECURSIVE_WORKFLOW_PROVEN` | ALIVE — VERIFIED-TRUE (workday mode; MOCKED-HUMAN) |
| `GRAPHLAW_DIALECT_CLOSURE` | ALIVE — VERIFIED-TRUE |
| `HOOK_ACTUATION_PROVEN` | ALIVE — VERIFIED-TRUE (64/64 receipted) |
| `ZERO_UNRECEIPTED_ACTUATION` | ALIVE — VERIFIED-TRUE (`metric-hook-actuations.rq`) |
| `V26_7_10_PRODUCTION_READY` | ALIVE — VERIFIED-TRUE, scoped per §7: loopback-real dispatch, MOCKED-HUMAN consequences, live network endpoints out of scope |

## 15. Current status summary (2026-07-10, closure)

- PROJ-606..622 are closed per `RELEASE_CONTROL.md` §8: PROJ-615 CUT (ed25519 deferred;
  `signatures: []` deliberately empty, PARTIAL by design), everything else ALIVE with the
  consolidated-build evidence cited there.
- The Fortune-5 benchmark machinery — OCEL authority, receipts, replay, Datalog roles — is
  ALIVE per `RELEASE_CONTROL.md` §1 and §7.
- External dispatch is loopback-real (deterministic local outbox/inbox); live network
  endpoints are explicitly out of scope for v26.7.10 and remain UNVERIFIED.
- Synthesized human consequences are MOCKED-HUMAN, and are said to be so wherever they appear.
- ChatmanEngine adoption is deferred; this release uses the TripleStore hook surface.
- LLM edge-only (§12) remains doctrine, UNVERIFIED as an enforced runtime boundary.

## See Also

- `docs/releases/v26.7.10/RELEASE_CONTROL.md` — single control surface; wins on disagreement
- `docs/releases/v26.7.10/PRD.md` — product requirements, authoritative Claims Reconciliation
- `docs/releases/v26.7.10/ARD.md` — architecture requirements
- `docs/CHATMAN_EQUATION.md` — `A = μ(O*)` formulation
- `docs/standing/SEMANTIC_PROFILE_DOCTRINE.md` — 80/20 profile strategy for dialects
- `crates/cng/BENCHMARK.md` — benchmark doctrine (being created under PROJ-607)

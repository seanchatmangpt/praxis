# PROJ-620 — Recursive dispatch, parent-child closure, timeout/escalation/compensation

Status: DONE (session-verified via this session's green `just cng-test-bench` — 31 lib tests
+ integration suites passing, recorded in session logs; `RELEASE_CONTROL.md` Sec. 8)

## Summary

Dispatched workflows may manufacture child dispatches under bounded depth/fan-out/cost with
declared authority and receipt closure, reusing the 8-ary attachment machinery. Parents
declare exactly one closure law in the graph (`ALL_CHILDREN_REQUIRED | ANY_CHILD_SUFFICIENT |
QUORUM_REQUIRED | ORDERED_SUBSET_REQUIRED | POLICY_DECIDES | FIRST_CONFORMANT_RESULT`),
evaluated via Datalog/SPARQL as a registered dialect, never inferred. Expired deadlines
(logical ticks) ⇒ `TIMED_OUT` ⇒ the declared escalation or compensation workflow is
manufactured — compensation is itself a workflow with authority, inputs, expected consequence,
receipt, and replay. Failed conformance emits the refusal as OCEL evidence plus the declared
retry/escalation/compensation; partial external execution is never silently discarded.

## Acceptance criteria

1. Child dispatches bounded in depth, fan-out, and cost; each child carries identity, parent,
   authority, depth, expected consequence, and receipt chain.
2. Closure law read from the graph and evaluated by the registered dialect; negative test:
   child completes but closure law unsatisfied ⇒ parent stays open.
3. Deadline expiry manufactures the declared escalation/compensation workflow, receipted.
4. Refused consequences appear as OCEL evidence; no partial result dropped silently.

## Verification

`just cng-test-bench` after the wave lands: closure-law, timeout, and compensation tests
green; `RECURSIVE_CHILD_CLOSURE_PROVEN`, `TIMEOUT_ESCALATION_PROVEN`,
`COMPENSATION_WORKFLOW_PROVEN` markers derive true via SPARQL (PROJ-622).

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 10, 11
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8

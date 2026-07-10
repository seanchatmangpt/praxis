# PROJ-622 — SPARQL-derived success markers

Status: IN_PROGRESS — UNVERIFIED pending consolidated final build (agent wave running this
session; no marker may flip until the queries run over a real workday bundle)

## Summary

The workday run ends by deriving the DoD Sec. 14 marker set (`AUTONOMIC_LOOP_CLOSED`,
`EXTERNAL_WORKFLOW_DISPATCH_PROVEN`, `EXTERNAL_RESULT_READMISSION_PROVEN`,
`RECURSIVE_CHILD_CLOSURE_PROVEN`, `TIMEOUT_ESCALATION_PROVEN`,
`COMPENSATION_WORKFLOW_PROVEN`, `ONE_PERSON_RECURSIVE_WORKFLOW_PROVEN`,
`GRAPHLAW_DIALECT_CLOSURE`, `HOOK_ACTUATION_PROVEN`, `ZERO_UNRECEIPTED_ACTUATION`,
`V26_7_10_PRODUCTION_READY`) from SPARQL over the emitted OCEL graph — each marker bound to
its on-disk query, never asserted by Rust code. Any false marker ⇒ nonzero exit plus a typed
refusal. Optionally wire a daily scheduled `just cng-workday` run — to be proposed at
execution time, not assumed.

## Acceptance criteria

1. One on-disk `.rq` per marker; marker verdicts come only from query results.
2. Any false marker ⇒ nonzero exit and a typed `CngRefusal`; no partial-success exit 0.
3. `V26_7_10_PRODUCTION_READY` is the conjunction of all other markers, never independently
   assertable.
4. Marker report is part of the receipted evidence bundle (replayable).

## Verification

`just cng-workday --seed S` once implemented: all markers derive true on a conformant run;
negative test with a doctored graph shows the named false marker and nonzero exit.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 14
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8

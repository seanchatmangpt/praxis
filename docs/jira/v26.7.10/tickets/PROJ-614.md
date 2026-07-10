# PROJ-614 — Graph-authoritative metrics closure

Status: IN_PROGRESS — UNVERIFIED pending consolidated final build (agent wave running this
session; query names `metric-hook-actuations.rq`/`metric-dispatch-closure.rq` made
authoritative here, old names folded; no green build cited yet)

## Summary

Make the metric corpus real and graph-authoritative: `metric-replay.rq` returns actual replay
verification triples (replay writes them); land the missing `metric-derived-roles.rq`; add
`metric-hook-actuations.rq` (transitions vs. transitions-with-`ex:hookDeltaHash` vs.
mismatches) and `metric-dispatch-closure.rq` (open external workflows, unacknowledged/overdue
dispatches, returned-but-unadmitted results, refused consequences, compensating workflows,
completed parent-child trees). The reconcile gate gains: unreceipted actuations > 0 OR
unreceipted dispatches > 0 OR unadmitted-accepted consequences > 0 ⇒ refuse. All laws are
checked by SPARQL over the emitted OCEL graph; Rust counters are telemetry only.

## Acceptance criteria

1. `metric-replay.rq` > 0 on a real run (no longer deterministically 0);
   `metric-derived-roles.rq` exists and returns rows.
2. `metric-hook-actuations.rq` and `metric-dispatch-closure.rq` exist on disk; no inline
   SPARQL.
3. Reconcile gate refuses on any of the three zero-tolerance conditions (negative test:
   dropped `ex:hookDeltaHash` ⇒ reconcile refuses).
4. Every SELECT authority result matches the Rust telemetry or the run refuses (existing
   `CNG_R09` discipline extended).

## Verification

`just cng-test-bench` once implemented: metric and reconcile-gate tests green; SPARQL over a
workday bundle shows zero unreceipted actuations and dispatches.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 4, 14
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8

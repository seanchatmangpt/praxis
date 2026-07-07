# PDDL Repair Plan

Real domain/problem/plan from Lane 3, re-verified by Lane 4 and Lane 6.
Source: `case-study/pddl/goal.ttl` (domain+problem TTL),
`case-study/pddl-out/{domain.pddl,problem.pddl,plan.json}`.

## Domain

11 types, 18 predicates, 16 action schemas — each within PDDL8 bounds
(`wasm4pm-compat::pddl::PDDL8_MAX_*` = 8 params / 8 pre-conjuncts / 8
effect-conjuncts; this domain uses <=3/<=3/<=2, arity 1 throughout).

## Problem

11 objects, 3 init facts, 1 goal atom:
`(ready-for-scope autonomic-standing-factory-local-first)`.

The init state honestly encodes a bad prior state —
`(claim-promoted autonomic-standing-factory-local-first)` is true from the
start (an earlier evidence-less/unlawful promotion). The only path to the
goal requires the planner to `demote-claim` that premature promotion first.

## Plan (admitted: true, 16 steps)

```
1. classify-external-side-effect
2. demote-claim
3. emit-standing
4. materialize-n3
5. validate-shacl
6. validate-shex
7. compute-datalog-closure
8. solve-pddl-plan
9. compile-powl-model
10. attach-benchmarks
11. smoke-client
12. verify-receipts
13. record-ocel
14. validate-wasm4pm-process
15. promote-claim
16. render-final-verdict
```

`promote-claim` is gated on `claim-demoted`, `datalog-closed`, and
`wasm4pm-validated` all holding on the correct objects; `render-final-verdict`
is gated on both the lawful promotion and the side-effect classification.

`admitted: true` is reported in the command's own stdout capture (e.g.
`case-study/raw/pddl-plan-determinism-recheck.txt`) — the checked-in
`plan.json` artifact itself records `graph_hash` and `plan`/`powl_chain_hash`
but not a separate `admitted` field (confirmed by Lane 5's adapter-building
pass and re-confirmed by Lane 6 reading the file directly).

`graph_hash`: `780bdcf99e56b541af788190aa91f4f1ab5e255989a4564e6a80aeb71d3de814`
`powl_chain_hash`: `blake3:d9d50a2f561f0c54fd9e655cac6ef4c96b99b91bbb09d2148a804a68608cb658`

## Determinism proof

Four independent `plan run` invocations over the same `goal.ttl`, into four
different `--out-dir`s, all produced the identical `powl_chain_hash`
(`blake3:d9d50a2f...cb658`):

| run | out-dir | who |
|---|---|---|
| A | `/tmp/pdl_det_run_a` | Lane 3 |
| B | `/tmp/pdl_det_run_b` | Lane 3 |
| C | (Lane 4 driver, ad hoc) | Lane 4 |
| D | `/tmp/lane6_pddl_recheck` | Lane 6 |

Matches invariant 3 (no wall clock in the hash path).

## Receipt verification (Criterion10)

`receipt validate --dir case-study/pddl-receipts` — real run, re-confirmed
by Lane 6:

```json
{
  "verdict": {
    "ok": true,
    "stages": [
      { "stage": "schema", "outcome": "Pass" },
      { "stage": "chain_recompute", "outcome": "Pass" },
      { "stage": "chain_linkage", "outcome": "Pass" },
      { "stage": "monotonic", "outcome": "Pass" },
      { "stage": "token_replay", "outcome": "Pass" }
    ],
    "records_checked": 1
  }
}
```

All 5 stages pass.

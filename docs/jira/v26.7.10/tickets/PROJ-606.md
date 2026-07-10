# PROJ-606 — v26.7.10 DEFINITION_OF_DONE.md (autonomic-loop doctrine)

Status: CLOSED (delivered this session)

## Summary

Author `docs/releases/v26.7.10/DEFINITION_OF_DONE.md`: the full autonomic-loop doctrine —
governing loop (`S_{n+1} = Admit(Observe(Dispatch(μ(S_n))))`, `R_n` binds the transition),
hook-morphism law plus zero-unreceipted actuation/dispatch/acceptance laws, Dialect Registry
Invariant, HookStanding and 13-state dispatch machines, 20-field dispatch contract,
parent-child closure laws, bounded-polling law, compensation-as-workflow, LLM-edge-only
constraints, autonomic completion criterion, and the 11 success markers (`AUTONOMIC_LOOP_CLOSED`
through `V26_7_10_PRODUCTION_READY`). Topology sentence: "the benchmark models a complete
enterprise through the lawful activities of a single operator."

## Acceptance criteria

1. Document exists at `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` with every doctrine
   clause from the plan present (sections 1-15 including the success-marker table).
2. Every clause carries a no-overclaiming status tag (ALIVE/PARTIAL/MOCKED/UNVERIFIED/PLANNED),
   derived from `RELEASE_CONTROL.md` evidence, never asserted independently.
3. Linked from `RELEASE_CONTROL.md` Sec. 6; states that `RELEASE_CONTROL.md` wins on
   disagreement.

## Verification

Delivered this session: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` exists with sections
1-15, the marker table (Sec. 14), per-clause statuses, and the `RELEASE_CONTROL.md` pointer in
its header and See Also. `RELEASE_CONTROL.md` Sec. 6 lists it as governed.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (the deliverable)
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 6 and Sec. 8

# PROJ-613 — Dialect Registry as executable law + HookStanding lifecycle

Status: ALIVE (session-verified via `just cng-test-bench`: `CNG_R14` registry gate green;
RELEASE_CONTROL.md Sec. 8 flips on the final gate — PROJ-617)

## Summary

`crates/cng/hooks/dialect-registry.ttl` plus a closed SHACL shape
(`dialect-registry.shape.ttl`) making all 8 Dialect Registry Invariant fields mandatory.
Entries: datalog, shacl, shex, sparql, owl-rl (opt-in), n3 (quarantine field "cold-route only,
no actuation" — declarative mirror of `chatman/router.rs`), delta/threshold/count/window,
powl, pddl, ocel, arazzo (external API orchestration). Validated at workday start via
`validate_shacl`; a missing field refuses with `CngRefusal::DialectRegistryRefused`
(`CNG_R14`, `crates/cng/src/powl.rs:92-97`). HookStanding states
(DECLARED→…→REPLAYABLE) are emitted as RDF as each hook passes load→…→replay. Code landed this
session in `crates/cng/hooks/dialect-registry{,.shape}.ttl` and `crates/cng/src/bench/hooks.rs`.

## Acceptance criteria

1. Registry validated at workday start; entry missing any of the 8 invariant fields ⇒
   `CNG_R14` `DialectRegistryRefused` (negative test: stripped receipt-schema field refuses).
2. Shape is closed — unknown registry predicates refuse by name.
3. N3 entry carries the quarantine field; no execution path routes n3 to actuation.
4. HookStanding lifecycle states appear in the graph per hook.

## Verification

`just cng-test-bench` — registry gate and stripped-field negative test green this session
(orchestrator-verified). Shared Sec. 8 verdict unchanged pending PROJ-617 sign-off.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 5, 6
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8

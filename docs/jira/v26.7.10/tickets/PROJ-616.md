# PROJ-616 — Verification harness: recipes, byte-identity, tamper negatives

Status: DONE (consolidated final build green this session: `just cng-test-bench` includes all
5 tamper negatives + in-process determinism gate; `just cng-workday-verify` seed=616 — two
same-seed runs byte-identical; `RELEASE_CONTROL.md` Sec. 8)

## Summary

Just recipes `cng-workday` (exists in `justfile:205`) and `cng-workday-verify` — just-only,
no `--release`, no concurrent builds per `docs/BUILD_CACHING.md`. Determinism gate: two
same-seed runs byte-identical across the full evidence bundle (OCEL, manifest, receipt chain
including `hook_hash` and dispatch receipts). Tamper negatives, written with
chicago-tdd-tools: mutated OCEL triple ⇒ audit replay refuses; dropped `ex:hookDeltaHash` ⇒
reconcile refuses; stripped registry field ⇒ `CNG_R14`; forged inbox consequence with wrong
correlation id ⇒ `CNG_R17` `ExternalConsequenceRefused`; child completes but closure law
unsatisfied ⇒ parent stays open.

## Acceptance criteria

1. `just cng-workday --seed S` twice ⇒ `diff -r` clean across bundles.
2. `just cng-workday-verify <bundle>` exits 0 on a pristine bundle and nonzero, with the
   named `CngRefusal` code, on each of the five tamper negatives.
3. `cng evidence replay <bundle>` ⇒ `AUDIT_RESULT=CONFORMANT`.
4. All negatives implemented with chicago-tdd-tools; no `--release` in any recipe.

## Verification

`just cng-test-bench` plus the two recipes above once implemented; expected markers:
byte-identical diff, `AUDIT_RESULT=CONFORMANT`, five named refusal codes on tampering.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 13, 14
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8

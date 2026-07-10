# PROJ-612 — graphlaw hook pack actuation (receipted per-transition hooks)

Status: ALIVE (session-verified via `just cng-test-bench`: hook actuation 64/64 receipted,
byte-identical same-seed runs; RELEASE_CONTROL.md Sec. 8 flips on the final gate — PROJ-617)

## Summary

`crates/cng/hooks/workday-pack.ttl` (kh:/hook: vocabulary): one hook per category, `kind
datalog|sparql`, `effect emit-delta`, seed+tick idempotency keys, no `kind n3` actuation
hooks. The workday loop wires `TripleStore::load_hook_pack` → `evaluate_hooks` per transition
→ `get_hook_receipts`; each `delta_hash` lands as `ex:hookDeltaHash` on the transition receipt
node; the run `hook_hash` folds into the BLAKE3 receipt chain. A transition without a hook
receipt refuses with `CngRefusal::UnreceiptedActuation` (`CNG_R13`,
`crates/cng/src/powl.rs:81-86`). Code landed this session in `crates/cng/hooks/workday-pack.ttl`
(+ `workday-pack-2.ttl`) and `crates/cng/src/bench/hooks.rs` (tests in `hooks_test.rs`).

## Acceptance criteria

1. Every workday transition carries `ex:hookDeltaHash` from a real `HookReceipt`; missing
   receipt ⇒ `CNG_R13` `UnreceiptedActuation`.
2. Run `hook_hash` folds into the BLAKE3 chain; two same-seed runs byte-identical
   (determinism spike).
3. Hook pack is SHACL-closed (`kh:HookShape`); no `kind n3` hook admitted.
4. Consumes the graphlaw hook surface (`praxis-graphlaw/src/lib.rs:473,536`) only — no edits
   to `crates/praxis-graphlaw/`.

## Verification

`just cng-test-bench` — 64/64 transitions receipted, `CNG_R13` negative test, and same-seed
byte-identity spike green this session (orchestrator-verified). Shared Sec. 8 verdict
unchanged pending PROJ-617 sign-off.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 4, 6
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8

# PROJ-608 — `benchmark workday` verb (single-operator deterministic day)

Status: ALIVE (session-verified via `just cng-test-bench`; RELEASE_CONTROL.md Sec. 8 flips on
the final gate — PROJ-617)

## Summary

New `cng benchmark workday` verb: roster of 1, deterministic logical-tick day (splitmix64
seed, no wall clock in digests); per tick the loop runs admit → derive role → manufacture →
attach → execute → receipt. Reuses `manufacture_set`; the tick lives in the graph as
`ex:logicalTick`. Code landed this session in `crates/cng/src/bench/workday.rs` (tests in
`workday_test.rs`), behind `#[cfg(feature = "bench")]`.

## Acceptance criteria

1. `just cng-workday --seed S` runs a full logical-tick day for one operator and emits an
   evidence bundle (OCEL, manifest, BLAKE3 receipt chain).
2. No `SystemTime`/`Instant::now` in any digest path; time is `ex:logicalTick` in the graph.
3. Two same-seed runs are byte-identical across the bundle (gated end-to-end under PROJ-616).
4. Fortune-5 `benchmark run` path unchanged.

## Verification

`just cng-test-bench` — workday tests green this session (orchestrator-verified). Full
byte-identity and marker gates land under PROJ-616/622; this ticket's ALIVE claim is scoped to
the test suite run this session, and the shared Sec. 8 verdict is not upgraded here.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` Sec. 2, 3, 13
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 8

# PROJ-749 — Decompose-to-dispatch bridge (Track P / Track E integration)

Status: ALIVE (mechanism, on a non-potato fixture) — evidenced this session (uncommitted;
HEAD `1f3f9bc`, Phase 6 commit not run)

Track: closure (beyond the original v26.7.10-revised plan's PROJ-701..731 range and beyond
the first closure round's PROJ-733/734/739..748; filed this session, second synthesis pass).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730), §2 (governing
claim) and §8 (potato canonical scenario). Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

## Summary

Before this ticket, Track P (`crates/cng/src/bench/decomp/`, no-LLM decomposition,
PROJ-701..713) and Track E (`crates/cng/src/bench/{dispatch,engine}.rs`, multi-engine
execution, PROJ-720..729) were each verified independently but never stitched together —
`docs/jira/v26.7.10/tickets/index.md`'s own "Execution sequence" note said so explicitly:
"Tracks P and E are independent until integration." This ticket closes that gap at the
mechanism level with a new bridge module, `crates/cng/src/bench/decomp/dispatch_bridge.rs`
(new file, registered via `pub mod dispatch_bridge;` in `crates/cng/src/bench/decomp/mod.rs`),
plus a new integration test,
`crates/cng/tests/cng_decompose_to_dispatch_integration.rs`.

Two crate-public entry points, deliberately not leaking the crate-private `DispatchContract`
type across the crate boundary:

- `subworkflow_to_contract` (`pub(super)`) — converts one `SubworkflowPlan` (Track P output)
  into a content-derived `DispatchContract` (dispatch id keyed on
  `subworkflow.id|role|problem_digest`); no wall clock, no PID.
- `dispatch_subworkflow_to_engine(root, subworkflow, target_engine) ->
  Result<SubworkflowDispatchHandle, CngRefusal>` — renders and shape-checks the contract
  (`CNG_R15`), then writes it atomically into the target engine's REAL filesystem inbox
  (`EngineBundle` layout, PROJ-722).
- `collect_subworkflow_consequence(root, handle, max_polls, poll_wait_ms) ->
  Result<SubworkflowDispatchOutcome, CngRefusal>` — bounded poll of the target engine's REAL
  outbox, then the same lawful re-entry pipeline the coordinator uses
  (`collect_consequence`: provenance -> correlation -> authority -> structural -> semantic).

## Evidence (this session)

`cargo test -p cng --features bench --test cng_decompose_to_dispatch_integration`: 2/2
passed, 1.76s.

1. `kitchen_decomposition_splits_into_helper_and_main` — the same guaranteed-split "kitchen
   two-chain" fixture `decomp_test.rs` uses in-crate, reconstructed here (the in-crate
   fixture is `#[cfg(test)]`-private and cannot be imported from an external integration
   test), confirms a real `decompose()` run derives a genuine 2-actor split (`helper`,
   `main`), not hardcoded.
2. `decomposed_subworkflows_dispatch_to_real_engines_and_are_admitted` — decomposes the same
   fixture, bridges `helper` to engine `H` and `main` to engine `M` (distinct target
   engines), spawns two REAL, independently-run `cng engine serve` OS processes
   (`CARGO_BIN_EXE_cng`, mirroring `tests/cng_multi_engine.rs`'s `serve_to_budget`) to
   completion, then collects and asserts both consequences `admitted: true`. Verified
   directly against on-disk evidence (not just green asserts): real inbox/outbox files and
   `engines/{H,M}/receipts/serve-report.json` exist under
   `target/chatman/cng-tests/decompose-to-dispatch-it/kitchen-dispatch-root/`.

Investigation finding that shaped the fixture choice: the canonical potato scenario
(PROJ-712) is not usable for this test — potato's real `decompose()` output selects
`DecompositionOutcome::NoAdmissibleDecomposition` (single-actor;
`decomp:subworkflowCount "1"` in its own `decomposition-result.ttl`), so it has nothing to
dispatch across two engines. This was verified directly against the emitted graph before
writing the fixture substitution, not assumed.

Regression (reported by the authoring agent this session; not independently re-run by this
doc pass): the existing 15 in-crate `decomp`/`workday` unit tests and the 7
`cng_multi_engine` integration tests continue to pass; `cargo build -p cng --features bench
--tests` succeeds with no new warnings.

## Honest boundary — what this does NOT prove

`engine.rs::run_serve_loop` derives its own deterministic PDDL artifact set from
`blake3(dispatch_id)` (`write_set`, category hardcoded `"email-routing"`) regardless of
contract content — confirmed directly against on-disk evidence
(`engines/H/admissions/<dispatch_id>/fragment-*.domain.ttl` is the engine's own synthetic
manufacture, not the helper subworkflow's actual PDDL). `DispatchContract` does not yet carry
a PDDL payload; `engine.rs`'s own module doc names this open work as "PROJ-710 -> PROJ-723".
So this ticket proves:

1. A real `decompose()` run derives a genuine multi-actor split, not hardcoded.
2. Each subworkflow's identity converts deterministically into a valid, shape-conformant
   `DispatchContract`.
3. That contract round-trips through a REAL, independently-spawned second OS process's full
   admission + 5-stage lawful re-entry pipeline.
4. Receipts of that round trip are durable on disk.

It does NOT prove that the remote engine executed the dispatched subworkflow's OWN plan, nor
that combining the two engines' outputs reconstructs or closes the original problem's global
goal — no machinery in the crate today makes that claim checkable (no payload-carrying
contract exists yet). It also does NOT prove the potato scenario itself dispatches across
H/M (see `DEFINITION_OF_DONE.md` §8 / `DOD_SIGNOFF.md` §8) — potato's own `decompose()`
output has no split to dispatch.

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` §2, §8
- `docs/releases/v26.7.10/DOD_SIGNOFF.md` §2, §8, G8 row (reconciled this pass)
- `docs/jira/v26.7.10/tickets/PROJ-710.md`, `PROJ-712.md`, `PROJ-720.md`, `PROJ-722.md`,
  `PROJ-723.md` (constituent mechanisms this ticket bridges)
- `crates/cng/src/bench/decomp/dispatch_bridge.rs`,
  `crates/cng/tests/cng_decompose_to_dispatch_integration.rs`

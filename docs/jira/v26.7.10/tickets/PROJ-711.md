# PROJ-711 — IPC corpus generators (5 domains x 20 seeded problems)

Status: ALIVE (full 5x20 corpus scale independently run) — evidenced this session (uncommitted;
HEAD `40f6020`, Phase 6 commit not run)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Clean-room `src/bench/ipc/{barman,blocksworld,termes,tyreworld,grippers}.rs` — never copied.
`(seed, size) → PDDL` via splitmix64; deterministic size-backoff solvability gate (blind-BFS
bound is the honest constraint); 20 problems per domain. Honest PARTIAL at the declared bound
beats heuristic-planner scope creep (DoD §19). Gate: G14.

## Evidence (this session)

`crates/cng/src/bench/ipc/{barman,blocksworld,grippers,termes,tyreworld}.rs` on disk. `cargo
test -p cng --features bench --test cng_ipc_corpus`: 10/10 passed, 1.79s this session,
including `ipc_corpus_seeds_plan_decompose_and_regenerate_byte_identically`
(`cng_ipc_corpus.rs:136-188`), which per its own header comment covers seeds 0..3 per domain
("the full 20-seed corpus is the benchmark run, not this unit test").

## Evidence (follow-up round — full 5x20 scale)

New file `crates/cng/tests/cng_ipc_corpus_full_scale.rs` (does not modify `cng_ipc_corpus.rs`
or any domain generator file) calls only the existing public `cng::bench::ipc`/`cng::bench::
decomp` surface, using the full 20-entry `IPC_CORPUS_SEEDS` (`crates/cng/src/bench/ipc/
mod.rs:63`) instead of the existing test's hardcoded `0..3u64`, asserting the same three
properties per `(domain, seed)` pair: plan found, byte-identical regeneration, typed decompose
outcome with `candidate_receipts[0] == "0-single"`.

Command: `CARGO_TARGET_DIR=target/agent-711 cargo test -p cng --features bench --test
cng_ipc_corpus_full_scale -- --nocapture`. Two consecutive runs, both green, no
failures/panics/timeouts: run 1 (cold, full workspace rebuild) 11.66s test-internal; run 2
(warm build) 11.79s test-internal, 15.68s real for the whole `cargo test` invocation. Per-domain
breakdown (run 2, 20 seeds each, 100 `decompose()` calls total): barman 3.183s (0.159s/seed),
blocksworld 5.542s (slowest, 0.277s/seed, no super-linear growth across the 20-seed range),
grippers 1.906s (0.095s/seed), termes 0.589s (0.029s/seed), tyreworld 0.568s (0.028s/seed).
`generate_solvable`'s size-backoff found a plan for every one of the 100 pairs on the first
attempt at `max_size` (no backoff iterations forced).

The full DoD §19 scale (5 domains x 20 seeds = 100 problems) is therefore now independently
executed and cited this session — PROJ-711 upgrades from `ALIVE (mech) / PARTIAL (scale)` to
`ALIVE (full scale)`. No boundary remains at the declared corpus width.

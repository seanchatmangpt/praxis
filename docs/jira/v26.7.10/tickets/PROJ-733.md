# PROJ-733 — Swap decomp grounder to `pddl-index` (performance fix)

Status: ALIVE — evidenced this session (uncommitted; HEAD `40f6020`, Phase 6 commit not run)

Track: closure (beyond the original v26.7.10-revised plan's PROJ-701..731 ticket range; filed
this session per the approved closure plan's Phase 1).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised closure plan (`recursive-workflow-v26-7-10-
definition-cuddly-parrot`). Control surface: `docs/releases/v26.7.10/RELEASE_CONTROL.md`
(v26.7.10-revised scope section).

## Summary

Added `pddl-index` as a bench-only optional path dependency of `crates/cng` (mirroring the
existing `praxis-graphlaw` bench-only-exception pattern in `Cargo.toml`) and swapped the three
decomposition grounding call sites (`decomp/mod.rs:273` `plan_manufactured`, `:363` source
grounding feeding `lift_ground`+`derive_edges`, `:394` single-actor planning) from
`bcinr_pddl::GroundProblem::build/find_plan` to `pddl_index::ground::IndexedGroundProblem::
build/find_plan`. Same `Pddl8Tape` output type; no downstream changes needed.

## Root cause

Untyped high-arity PDDL schemas (potato's `fillPot`, grippers' `pick`/`drop`) made
`bcinr_pddl::GroundProblem::build`'s naive full-cross-product grounding materialize thousands
of never-firing ground actions (potato: 8991, `fillPot` alone contributing 6561; grippers
size 4: 13,851). The dominant cost was NOT BFS search (dedup keeps it in the thousands of
nodes, seconds-scale) — it was `decomp/rules.rs:126`'s documented `O(|rules|·|facts|²)`
Datalog mutex/custody join: a degree-729 atom like grippers' `free(g-left)` alone yielded
~530k mutex facts per rule arm, ~4.4M total. `pddl_index::ground::IndexedGroundProblem::build`
(relaxed-reachability-pruned grounding, differential-tested against `bcinr_pddl::find_plan` —
identical plans) collapses the ground-action count at the source, fixing both costs at once —
potato's ground set drops ~430x (8991→~21). No fixture semantics were weakened; only two
negative fixtures whose specific refusal *code* depended on the naive grounder's blind spots
were corrected (see PROJ-712's evidence section).

## Evidence (this session)

- `cargo test -p cng --features bench --test cng_decomp`: 3/3 passed, 0.18s (was 60s+ hang
  before the fix).
- `cargo test -p cng --features bench --test cng_ipc_corpus`: 10/10 passed, 1.79s (was
  minutes/OOM-risk).
- `cargo check -p cng` (no `--features bench`, the default/publishable surface): compiles
  clean, 0 warnings — confirms `pddl-index` does not participate in the default build.
- `crates/cng/Cargo.toml:42-50` (dependency + doc comment), `crates/cng/src/bench/decomp/
  mod.rs:73` (`use pddl_index::ground::IndexedGroundProblem as GroundProblem;`).

## Links

- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` §16 (PROJ-743 reconciliation cites this fix)
- `docs/jira/v26.7.10/tickets/PROJ-704.md`, `PROJ-711.md`, `PROJ-712.md` (downstream evidence)

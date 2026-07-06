# Planner Surface — v26.7.6

Census of what already connects the planner loop (PDDL plan -> POWL workflow
-> bcinr execution -> ggen artifact -> receipt), and the gaps closed to run
it as one command.

## What existed before this release

| Stage | Location | Evidence |
|-------|----------|----------|
| Graph facts -> PDDL8 (manufacture) | `src/mfg.rs` (`load_graph`, `extract_domain`, `extract_problem`, `enforce_pddl8`, `emit_domain`, `emit_problem`, `manufacture`) | `tests/mfg_golden.rs` pins the 5-step lawobject plan |
| PDDL8 parse/ground/solve | `bcinr-pddl` (`domain_from_pddl`, `problem_from_pddl`, `GroundProblem::find_plan`) + `crates/pddl-index` (indexed grounder, auto-selected by `pddl_index::should_use_indexed`) | `src/ops.rs::plan_solve_payload` (shared by CLI verb `plan solve` and MCP `plan_solve`) |
| Temporal solve/analyze/execute | `bcinr-pddl` (`GroundTemporalProblem`, `analyze_schedule`, `execute::execute_temporal_plan`) | `src/verbs/plan.rs` (`solve`, `analyze`, `execute`) |
| POWL model + compile + tick | `bcinr-powl` (`compiler::compile_powl` -> `PowlTape`, `scheduler::scheduler_tick`, acyclicity + reachability checks) and `crates/powl2-decompose` (`Powl`, `WfNet`, decompose/recompose) | in-crate tests in `bcinr-powl/src/scheduler.rs`, `crates/powl2-decompose` |
| Receipted causal frames | `bcinr-powl-receipt` (`OcelCausalFrame`, `OcelCausalReceipt::genesis/chain/canonical_hash` — genesis-folded BLAKE3, `ts_ns` caller-supplied) | `bcinr-powl-receipt/src/causal_receipt.rs` byte-layout tests |
| Receipt ledger (append-only JSONL chain) | `src/ops.rs::receipt_issue_payload` + `src/verbs/receipt.rs` (`issue`, `validate`, `replay`, `export-ocel`) | `src/ops.rs` receipt tests |
| Shape validation of manufactured PDDL | `src/mfg.rs::validate` (parse/ground/solve round-trip) | `tests/mfg_golden.rs` |

## What was missing (the gaps this release closes)

1. **No single command** running graph -> plan -> workflow -> execution ->
   artifact -> receipt. Each stage had its own verb (`mfg pddl`,
   `plan solve`, `receipt issue`) but nothing composed them.
2. **Plan -> POWL compilation was unreachable from praxis**: `bcinr-powl`'s
   compiler was a dependency of the workspace but never called from `src/`.
3. **No step-level receipted execution**: `bcinr-powl-receipt`'s
   `OcelCausalReceipt` was used only for `DenialPolarity` constants
   (`src/corpus.rs`); no praxis path chained `OcelCausalFrame`s.
4. **No artifact + verifier gate wired into the loop**: `mfg::manufacture`
   returned text; nothing wrote the artifact and gated it behind
   `mfg::validate` before receipting.

## What this release adds

- `src/plan_run.rs` (feature `ggen`): `plan_run_payload` — the composed
  vertical slice. Graph load + hash (`mfg::load_graph`/`graph_hash_hex`),
  manufacture (`mfg::manufacture`), solve (`ops::plan_solve_payload`,
  classical mode, indexed/naive grounder auto-select), plan -> POWL sequence
  (`bcinr_powl::compiler::compile_powl`, acyclicity + reachability enforced
  by the compiler), execution via `bcinr_powl::scheduler::scheduler_tick`
  with one `OcelCausalFrame` chained per fired atom (`ts_ns = 0`, run id =
  BLAKE3 of the source graph hash — no wall clock anywhere in the hash
  path), artifact write (`domain.pddl`, `problem.pddl`, `plan.json`),
  verifier gate (`mfg::validate` must report `solvable`), and a final
  ledger receipt (`ops::receipt_issue_payload`, `ts_ns: 0`).
- `plan run` CLI verb (`src/verbs/plan.rs`, feature `ggen`).
- Demo fixture: `examples/v26_7_6_after_neon/` (goal TTL + facts query +
  artifact template note + README with the exact command).
- Tests: `tests/plan_run_e2e.rs` — plan legality, POWL dry-run
  (compile-only), full loop, and two-run determinism (identical
  `powl_chain_hash` across runs; ledger receipt hashes differ only by
  ledger position, by design of the append-only chain).

## Deliberate non-goals

- No solver was rebuilt: `bcinr-pddl`/`pddl-index` already expose
  `find_plan` (invariant 6, reuse first). `wasm4pm-planner` remains the
  independent cross-check lane (see root `Cargo.toml` comment) and is not
  in this path.
- Nothing added to `crates/praxis-synthesis` (frozen deps, invariant 5).
- Choice/loop POWL shapes are not exercised: a classical plan is a total
  order, so the compiled workflow is `Sequence` only. `XorChoice`/`Loop`
  stay available in `bcinr-powl` for future branching plans.

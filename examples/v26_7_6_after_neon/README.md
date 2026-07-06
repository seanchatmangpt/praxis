# after-neon — planner vertical-slice demo (v26.7.6)

One command runs the whole loop: goal graph -> PDDL plan -> POWL workflow ->
receipted bcinr execution -> manufactured artifact behind the solvability
verifier -> receipt folded into the ledger chain.

## Files

- `goal.ttl` — the `pdl:` goal ontology (domain, closed action vocabulary,
  and the `after-neon-case-001` problem instance).
- `rules.rq` — SPARQL projection of the closed action vocabulary
  (invariant 4), runnable via `mfg facts`.
- `template.md` — the artifact contract: what `plan run` writes and why.

## The command

From the repository root:

```sh
cargo run --features ggen --bin my-conforming-project -- plan run \
  --goal examples/v26_7_6_after_neon/goal.ttl \
  --out-dir target/plan_run/after_neon
```

Expected: `"admitted": true`, plan
`grant-standing -> ground-blueprint -> manufacture-artifact -> fold-receipt`
(pinned by `tests/plan_run_e2e.rs::full_loop_after_neon_fixture`), a
`powl_chain_hash` starting with `blake3:`, and
`target/plan_run/after_neon/{domain.pddl,problem.pddl,plan.json}` on disk.

Determinism: running the command twice yields the same `powl_chain_hash` —
no wall clock enters the frame hash path (`ts_ns = 0`; run id is BLAKE3 of
the source graph hash). Pinned by
`tests/plan_run_e2e.rs::two_runs_identical_chain_hashes`.

## Stage-by-stage (all reused surfaces)

| Stage | Surface |
|-------|---------|
| Graph facts + hash | `src/mfg.rs::load_graph` / `graph_hash_hex` |
| Manufacture PDDL8 | `src/mfg.rs::manufacture` |
| Solve | `src/ops.rs::plan_solve_payload` (`bcinr-pddl` + `pddl-index`) |
| Plan -> POWL | `bcinr_powl::compiler::compile_powl` |
| Receipted execution | `bcinr_powl::scheduler::scheduler_tick` + `bcinr_powl_receipt::causal_receipt` |
| Verifier gate | `src/mfg.rs::validate` (must report `solvable`) |
| Ledger receipt | `src/ops.rs::receipt_issue_payload` |

See `docs/releases/v26.7.6/PLANNER_SURFACE.md` for the full census.

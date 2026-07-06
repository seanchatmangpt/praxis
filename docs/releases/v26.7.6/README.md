# Praxis v26.7.6 "After Neon" — The Release Explained

Praxis manufactures standing for AI-generated technical work.

Language enters as raw material. GraphLaw admits and derives law-state. PDDL
plans the path. POWL executes the workflow. bcinr accelerates lawful
transitions. ggen manufactures artifacts. Lean, Lake, tests, and graph
validators act as gauges. Receipts preserve results. Reports publish the
standing.

## The loop, stage by stage

| Stage | What it does | Where it lives |
|---|---|---|
| Language in | Raw claims, rules, and specs enter as Turtle/N3 and JSON payloads | `[law].rules` documents, JSON law objects (`src/verbs/law.rs`) |
| Law-state | GraphLaw loads rules, materializes to fixpoint, runs SHACL gates; violations are typed FM-LAW refusals | `crates/praxis-graphlaw`, `ggen law load|derive|validate|explain|export` |
| Plan | The path from current state to goal is searched, not scripted | `plan route|solve|analyze|execute` (root CLI), pddl-index, wasm4pm-planner (`Cargo.toml:98,116`) |
| Workflow | Solved plans compile to a POWL tape and execute in plan order, deterministically | `src/plan_run.rs:67 compile_plan_to_powl`, `:89 execute_receipted` |
| Hot path | Lawful transitions run against the bcinr crates (powl, pddl, powl-receipt) | `Cargo.toml:96-100` (bcinr-powl-receipt, bcinr-pddl, bcinr-powl) |
| Factory | Artifacts are generated through the five-stage pipeline: resolve, enrich, extract, render, write | `ggen sync run` (`crates/ggen`) |
| Gauges | Lean 4/Lake kernel-checks every `.lean` file, refuses `sorry`/`admit`/unauthorized `axiom`; graph validator refuses unknown predicates by name; `cargo test` gates the rest | `praxis-l4 l4 verify|no-sorry`, `ggen graph validate` |
| Evidence | Every result folds into a BLAKE3, genesis-folded receipt chain; chains are recomputed to verify, never trusted | `ggen receipt verify|history`, `receipt validate` (root CLI), `crates/ggen/tests/receipt_chain_e2e.rs` |
| Publication | Standing is published as reports and release docs, each claim citing its receipt | `praxis-l4 l4 report`, `frontier matrix|summary|counts`, `docs/releases/v26.7.6/` |

## What "After Neon" means

The neon era shipped generation without institutions: output you could not
audit, agents you could not hold to account. After Neon is the civic phase.
This release does not add generative capacity; it closes the loop that gives
generated work standing — admission by law, execution under workflow, gauging
by kernel, evidence by computed receipt.

The operating rules are invariants, not aspirations (`RELEASE_CONTROL.md`
Sec. 4): every error a typed `Refusal`; receipts computed, never asserted; no
wall clock in any hash path; closed vocabularies with unknown predicates
refused by name; `praxis-synthesis` dependencies frozen and test-enforced
(`crates/praxis-synthesis/tests/no_llm_runtime.rs`).

## What is new in v26.7.6

- **Native graph law engine.** `crates/praxis-graphlaw` (clean-room adoption
  from the roxi lineage — N3/Datalog/SHACL) is in the workspace and surfaced
  through the ggen CLI as the `law` noun. This replaces the coupling to the
  frozen external `~/ggen` repo's `ggen-graph` (`INVENTORY.md`, "Known
  couplings").
- **Lean/Lake admission gate as a release gauge.** `crates/praxis-lean`
  (`praxis-l4` binary): kernel verification, no-sorry audit, corpus
  reconciliation, receipt reporting.
- **Full-loop fixture.** `tests/plan_run_e2e.rs` exercises language → plan →
  POWL → receipts in one run, with a determinism test
  (`two_runs_identical_chain_hashes`). At time of writing both full-loop tests
  refuse on a missing `PRAXIS_SIGNING_KEY` — a typed refusal, and the first item
  on the day-one finish plan (`PRD.md` Sec. 13).

## Status honesty

This README makes no green-badge claim. The release ships when the seven exit
criteria in `RELEASE_CONTROL.md` Sec. 5 each carry a proof artifact. Current
verified baseline: `cargo check --workspace` exit 0 (2026-07-06,
`RELEASE_CONTROL.md` Sec. 8); command surface enumerated by running the actual
binaries (`CLI.md`).

## Start here

```
just test-changed     # fast inner loop
just verify-all       # Definition-of-Done gate
cargo run -p ggen --bin ggen -- --help            # factory + law surface
cargo run --bin my-conforming-project -- --help   # admission/plan/receipt surface
cargo run -p praxis-lean --bin praxis-l4 -- l4 --help  # kernel gauge
```

Full command inventory with implementation status: `CLI.md`. Requirements and
acceptance criteria: `PRD.md`. Live control surface: `RELEASE_CONTROL.md`.

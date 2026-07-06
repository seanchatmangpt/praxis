# Praxis v26.7.6 "After Neon" — CLI Command Surface

Method: every status below was determined by running the actual binaries on
2026-07-06 (`cargo run -p ggen --bin ggen -- <noun> --help`,
`cargo run --bin my-conforming-project -- <verb> --help`,
`cargo run -p praxis-lean --bin praxis-l4 -- l4 --help`), not by reading
source or guessing. "PARTIAL" means the capability exists under a different
command name or a subset of the requested verbs; the actual command is cited.

## Binaries observed

| Binary | Package | Invocation | Nouns/verbs observed |
|---|---|---|---|
| `ggen` | `crates/ggen` | `cargo run -p ggen --bin ggen` | `law`, `doctor`, `receipt`, `sync`, `graph` |
| `my-conforming-project` | workspace root | `cargo run --bin my-conforming-project` | `law`, `plan`, `receipt`, `synth`, `verifier`, `frontier`, `dod`, `doctor`, `config`, `example` |
| `praxis-l4` | `crates/praxis-lean` | `cargo run -p praxis-lean --bin praxis-l4` | `l4` → `init`, `report`, `no-sorry`, `verify`, `reconcile`, `index-build` |
| `dod` | workspace root | `cargo run --bin dod` | DoD gate; ran 2026-07-06, exited HARD FAILURE on the two signing-key test failures (typed refusal, see below) |

All three CLIs expose `--format`, `--select`, `--introspect` (JSON Schema for
LLM tool-calling), `--structured-errors`, `--autonomic` (observed in every
`--help` output).

## Required command surface — status

| Required command | Status | Evidence / actual surface |
|---|---|---|
| `admit language` | PLANNED | No `admit` noun on any binary. Nearest surface: root `law admit`/`law judge` accept a JSON law-object payload via stdin or argument (`my-conforming-project law --help`); language-level admission (Turtle/N3 in) runs through `ggen law load`. A dedicated `admit language` verb does not exist. |
| `admit file` | PLANNED | Same: no `admit` noun. File-shaped admission today is `ggen law load` ([law].rules files) and `praxis-l4 l4 verify` (.lean files). |
| `admit claim` | PARTIAL | Root `law judge` / `law admit`: wraps a JSON claim in `praxis_core::LawObject`, runs `DefaultLaw` Judge/Admit (plus prolog8 admission when atom/rule present); malformed input is a hard Err, domain denial a typed verdict (verified via `--help`). Not named `admit claim`. |
| `law load` | IMPLEMENTED | `ggen law load` — loads every [law].rules file into the GraphLaw engine, reports rule count per file, refuses on first unparseable rule document (verified via `--help`). |
| `law validate` | IMPLEMENTED | `ggen law validate` — materializes rules, runs every [law].shapes SHACL gate and the denial check; any violation is a typed FM-LAW refusal, non-zero exit (verified via `--help`). |
| `law derive` | IMPLEMENTED | `ggen law derive` — materializes to fixpoint, reports derived-triple count plus post-materialization graph hash (verified via `--help`). |
| `law explain` | IMPLEMENTED | `ggen law explain` — rules-loaded count plus full derived-triple diff as canonical N-Triples (verified via `--help`). |
| `law export` | IMPLEMENTED | `ggen law export` — dumps fully materialized graph as canonical N-Triples with BLAKE3 state hash (verified via `--help`). |
| `plan goal` | PLANNED | Not present. Root `plan` verb exposes `route`, `solve`, `analyze`, `execute` (enumerated from `--help`). Goal declaration currently rides inside `plan solve` input rather than a separate verb. |
| `plan solve` | IMPLEMENTED | Root `plan solve` (verified via `--help` enumeration). |
| `plan step` | PLANNED | Not present as a verb. Step extraction exists as library code (`src/plan_run.rs:46 plan_step_names`). |
| `plan run` | PARTIAL | Named `plan execute` on the CLI; the full run path is `src/plan_run.rs:149 plan_run_payload` exercised by `tests/plan_run_e2e.rs`. Both full-loop tests currently refuse on missing `PRAXIS_SIGNING_KEY`/`PRAXIS_SIGNING_KEY_FILE` (observed 2026-07-06) — typed refusal; key provisioning is day-one finish item 1 (`PRD.md` Sec. 13). |
| `plan replan` | PLANNED | Not present on any binary. |
| `workflow compile` | PARTIAL | No `workflow` noun. Compilation exists as `src/plan_run.rs:67 compile_plan_to_powl` (plan → POWL tape), invoked by `plan execute`; Stage-1 WF-net → POWL 2.0 decomposition lives in `crates/powl2-decompose` (library, no bin target — `crates/powl2-decompose/Cargo.toml`). |
| `workflow run` | PARTIAL | `src/plan_run.rs:89 execute_receipted` runs the compiled tape in plan order, deterministically (unit test `execute_fires_plan_order_and_is_deterministic`, `src/plan_run.rs:295`). Surfaced through `plan execute`, not a `workflow` noun. |
| `bcinr status` | PLANNED | No bcinr CLI in this workspace. bcinr integration is library-level: bcinr-powl-receipt 26.6.24, bcinr-pddl 26.6.26, bcinr-powl 26.6.25 (`Cargo.toml:96-100`). |
| `bcinr transition` | PLANNED | Same — library-level only. |
| `bcinr bench` | PLANNED | Same; benchmark surface tracked as Phase 3b (Blue River Dam Divan benchmarks), not started. |
| `gen lean` | PLANNED | No `gen lean` command. The factory surface is `ggen sync run` (five-stage pipeline: resolve, enrich, extract, render, write — verified via `--help`); Lean-targeted generation is not a distinct verb yet. |
| `gen lake` | PLANNED | Not present. `praxis-l4 l4 init` scaffolds a Lake package (lean-toolchain, lakefile.lean, Praxis.lean) but is a gauge-side scaffold, not factory generation. |
| `gen report` | PARTIAL | `praxis-l4 l4 report` writes a VerificationReport (status counts, missing receipts, duplicate labels) as JSON for downstream ggen/LaTeX rendering (verified via `--help`). Not under a `gen` noun. |
| `lean run` | PARTIAL | Named `praxis-l4 l4 verify`: walks a directory of .lean files, kernel-checks each, runs the no-sorry audit, appends a genesis-folded receipt per file (verified via `--help`). Companion: `l4 no-sorry` refuses `sorry`/`admit`/unauthorized `axiom`. |
| `lake run` | PARTIAL | Lake is driven inside `praxis-l4 l4 verify`/`l4 init` (`crates/praxis-lean`, "Lean 4/Lake kernel admission gate" per binary help); no standalone `lake run` verb. |
| `receipt write` | PARTIAL | Named `receipt issue` on the root CLI (dispatcher: issue, validate, show, replay, export-ocel — from `receipt --help`); gauge-side receipts are appended by `praxis-l4 l4 verify`. |
| `receipt reconcile` | PARTIAL | Two surfaces, neither named exactly this: `praxis-l4 l4 reconcile` (cross-references corpus index against receipt ledger; orphans reported as typed refusals) and root `receipt validate` (schema, chain-tamper recompute, linkage, monotonicity, POWL token-replay conformance — verified via `--help`). Chain verification also at `ggen receipt verify|history` (`crates/ggen/tests/receipt_chain_e2e.rs`). |
| `frontier list` | PARTIAL | Named `frontier matrix` / `frontier summary` / `frontier counts` (enumerated from root CLI `--help`); Lane 10 frontier receipt, matrix construction shared with `tests/frontier_matrix.rs`. No `list` verb. |
| `frontier probe` | PLANNED | Not present. |
| `report proof` | PARTIAL | `praxis-l4 l4 report` covers the proof-corpus report (status counts, missing receipts, duplicate labels). No `report` noun on any binary. |
| `report planner` | PLANNED | Not present; `plan analyze` exists but is a plan-time verb, not a published report. |
| `report market` | PLANNED | Not present. Nearest artifact: `revenue_demo` bin target (`Cargo.toml:122-124`); market reporting is Phase 3b, not started. |
| `publish paper` | PLANNED | No `publish` noun on any binary. Publication today is docs-in-repo (`docs/releases/v26.7.6/`) plus `l4 report` JSON "for downstream ggen/LaTeX rendering" (binary help). |
| `challenge audit` | PLANNED | No `challenge` noun on any binary. Nearest adversarial surfaces: `receipt validate` (chain-tamper recompute) and `ggen doctor run` (drift/staleness). |
| `challenge explain` | PARTIAL | Explanation of derivations exists as `ggen law explain` (full derived-triple diff); not under a `challenge` noun. |

## Tally

IMPLEMENTED 6 · PARTIAL 12 · PLANNED 14 · BLOCKED 0 · OUT_OF_SCOPE 0 (of 32
required commands). No status above is asserted from source reading alone;
each cites the binary `--help` run or the test that exercises it.

## Refusal-completeness note

Exit criterion 5 (`RELEASE_CONTROL.md` Sec. 5) requires refusal tests per
command. Current observed refusal behavior: the full-loop tests refuse (typed
error string, no panic) on missing signing key
(`tests/plan_run_e2e.rs:109`, observed 2026-07-06); `ggen law validate`
documents violations as typed FM-LAW refusals with non-zero exit; `praxis-l4 l4
reconcile` reports orphans "as typed refusals, not silently" (binary help).
Per-command refusal tests for the remaining surface are not yet written — that
work is tracked under exit criterion 5, status NOT STARTED.

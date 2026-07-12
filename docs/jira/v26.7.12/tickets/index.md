# Milestone Overview: v26.7.12 — Multifractal Workflow Architecture Atlas

Generated after a 30-agent parallel wire-phase-1 pass (2026-07-12) that wrote real content into
every `crates/multifractal-workflow/src/f<NN>_*.rs` module, replacing the earlier Wire-phase-0
skeleton stubs. This index's **Verdict** and **Cited Reuse Source** columns are taken from each
family's own module doc comment (`crates/multifractal-workflow/src/lib.rs`'s table and each
`f<NN>_*.rs` file's header); per that same lib.rs doc comment, ticket numbers not spelled out
verbatim in a family's own text are inferred by the `V12-0<NN>` pattern, not independently
re-confirmed against `/Users/sac/Downloads/v26.7.12_mermaid_atlas/` by every agent. The **Wired
Status** column is this session's own finding, verified by direct `grep` of the checked-in source
for always-refusing placeholder functions (`NotYetImplemented`/`NotImplemented`/"always
Err"/"always refuses" markers) plus the compile/test evidence below — not by re-trusting each
family agent's own prose claims.

## Compile verification (this session)

- `just check` (`cargo check --workspace --all-features`) — **exit 0, twice**, zero errors across
  the whole workspace, confirming `multifractal-workflow` and every pre-existing v26.7.11 crate
  compile together. Log: one `dead_code` warning in `f22_compensation.rs:258` (unused `ALL`
  const); no other multifractal-workflow warnings, no errors anywhere.
- `just multifractal-workflow-check` (`--tests`, whole crate incl. test targets) — exit 0.
- `just multifractal-workflow-test-long --lib` (added this session, 900s timeout; the crate's
  existing 300s recipe was insufficient for a first-time link of all 30 families' test binaries
  in the shared target dir) — **373 passed, 0 failed, 1 ignored** (374 total). The 1 ignored test
  (`f17_atomvm_runtime`'s real subprocess bridge to the Erlang eunit suite) was run separately
  (`-- --ignored`) and also **passed**. All 374 of this crate's tests pass.
- No source file belonging to an existing v26.7.11 crate was modified by this wire-phase-1 pass
  itself (confirmed via `git diff --stat` against each family's claimed touched-files list); the
  workspace root `Cargo.toml`/`Cargo.lock`/`justfile`/`ggen.toml` diffs are additive
  (crate registration + new recipes + new ggen packs). Unrelated uncommitted diffs exist elsewhere
  in this fast-moving shared repo (e.g. `crates/wasm4pm-arazzo/src/lower.rs`,
  `crates/cng/src/bench/dispatch.rs`) — these predate or are concurrent with, but not caused by,
  this wire-phase-1 pass (confirmed by mtime and by the absence of any family report claiming to
  touch them); they are not this ticket table's concern and were not modified by this session.
- No breakage attributable to `multifractal-workflow` was found; nothing required fixing.

## Verdict distribution (from `lib.rs`'s own table, read directly this session)

28 of 30 families are labeled **MIXED** (some real reuse/generation + some disclosed
hand-write gap). 2 are labeled **ALREADY_BUILT** (F12, F20 — thin wrappers over existing,
already-tested code). Zero families are labeled purely REUSE_ADAPT-only,
GGEN_GENERATABLE-only, or HAND_WRITE_REQUIRED-only — every family mixes at least two of the
three buckets in practice.

## Ticket index

| # | Family | Ticket | Verdict | Cited reuse/adaptation source | Wired status |
|---|--------|--------|---------|-------------------------------|--------------|
| 1 | Standing Algebra | V12-001 | MIXED | `praxis_graphlaw::shacl`/`chatman::abi::Receipt`; `bcinr_powl_receipt::replay::PowlReplayVerifier` | Real; no always-refusing stub fn found. L7 idempotency ledger disclosed in-memory-only. |
| 2 | Observation Admission | V12-002 | MIXED | `praxis_graphlaw` SHACL engine (Gate 5 only; Gates 1-4/6-7 hand-written) | Real; no always-refusing stub fn found. No cross-restart durability, no OWL-RL closure (disclosed, minimal closed-vocab check substituted). |
| 3 | Semantic Contraction | V12-003 | MIXED | `praxis_graphlaw::TripleStore` (OWL-RL/SHACL/ShEx); same-crate reuse of `f05_datalog_closure`/`f06_n3_quarantine` | Real; no always-refusing stub fn found. Found and disclosed (did not fix) a real F05 test bug. |
| 4 | GraphLaw Dialect Registry | V12-004 | MIXED (self-reported PARTIAL) | `cng` PROJ-613 SHACL closed-shape pattern (adapted, not imported — `cng`-private); `oxigraph` SPARQL | Structural gap, not stub-wrapped: 5 named L2 pipeline components exist only as catalog data, not live subsystems. |
| 5 | Datalog Closure | V12-005 | MIXED | `praxis_graphlaw::TripleStore::add_rules`/`materialize`; `datalog::validate_rules` stratifier | Real; no always-refusing stub fn found. Reported PARTIAL for lack of a green whole-crate test at the time — resolved: this session's whole-crate run passes. |
| 6 | N3 Quarantine and Refinement | V12-006 | MIXED | `praxis_graphlaw::chatman::router::DialectRouter`; `N3Executor`/`N3CostBound`/`N3Builtin` | Real; no always-refusing stub fn found. `N3Executor` still not called from `ChatmanEngine`'s own production pipeline (disclosed). |
| 7 | SHACL and ShEx Admission | V12-007 | MIXED | `praxis_graphlaw::TripleStore::validate_shacl`/`validate_shex_c`; `shacl::ShapesGraph::parse` | Real; no always-refusing stub fn found. Durability is real but in-process only (persist/restore tested, no disk auto-wiring). |
| 8 | PDDL Planning and Action-Hook Binding | V12-008 | MIXED | `bcinr_pddl::ground::GroundProblem`/`execute::execute_tape`; `f08_pddl_planning/projector.rs` mirrors `ChatmanEngine::compute_pddl_plan` | `hook_binder::bind_actions` always returns `Refusal::NoAdmissiblePlan` (verified this session) — `run_pipeline` therefore never reaches success end-to-end. |
| 9 | MFW Growth Operator | V12-009 | MIXED | `praxis_graphlaw::chatman::closure::RecursiveSocketClosure::is_closed`; `pddl_index::solve_indexed` | 2 core stub fns always refuse (verified this session): `resolve_continuation_goal`, `manufacture_and_bind_child` — the family's central "growth" capability. |
| 10 | POWL Recursive Process Geometry | V12-010 | MIXED | `powl2_decompose` (`decompose_wf_net`, `Powl`, `ChoiceGraph`, ... re-exported directly) | Real for core decomposition/geometry (20 tests). L6 receipt chain, L7 replay/idempotency, L8 claim markers are entirely absent — disclosed, not stub-wrapped. |
| 11 | BCINR Local Runtime | V12-011 | MIXED | `bcinr_powl::compiler::compile_powl`/`scheduler::scheduler_tick`; `bcinr_powl_receipt::causal_receipt` | 2 stub fns always refuse (verified this session): `detect_external_socket`, `admit_duplicate_or_stale` (L7). |
| 12 | POWL External Cut and Projection | V12-012 | ALREADY_BUILT | `praxis_graphlaw::chatman::{powl_projection,engine}`; `powl2_decompose`; `praxis_core::arazzo::ChatmanRailAbCompiler` | Real thin re-export, exercised end-to-end by a real compiler in tests. 1 stub fn always refuses (verified this session): `check_external_cut_chaos_recovery` (L7). |
| 13 | Arazzo Generated Artifact | V12-013 | MIXED | `praxis_core::arazzo` (`admit_manufactured_arazzo`, `render_arazzo_document`, `ChatmanRailAbCompiler`) | Real thin re-export, exercised end-to-end. 1 stub fn always refuses (verified this session): `check_idempotency_and_correlation` (L7). |
| 14 | wasm4pm Arazzo Compiler | V12-014 | MIXED | `wasm4pm_arazzo` (`DocumentIndex::add_document`, `resolve::normalize_uris`, `lower`, `ArazzoNormalizer`, `AirCompiler`) | Real 5-function pipeline composition. 2 stub fns always refuse (verified this session): `durability::admit_idempotent`, `persist_receipt_head` (L7/L8). |
| 15 | AIR Semantic Core | V12-015 | MIXED | `apps/air_core/src/air_core.erl` (read, not linked — no Rust build target exposes an `rlib`) | No functional Rust↔Erlang bridge exists (structural, disclosed: `air_core_nif` is `cdylib`-only). Module is real vocabulary/catalog data mirroring the Erlang source, 0 direct external-crate calls. |
| 16 | Erlang OTP Outer Runner | V12-016 | MIXED | `apps/arazzo_runner/src/arazzo_runner_broker.erl`/`_workflow.erl`/`_identity.erl` (read, not linked) | 3 stub fns always refuse (verified this session): `check_gen_statem_lifecycle_wired`, `check_dispatch_worker_supervisor_wired`, `check_program_registry_wired`. Zero admission-chain logic ported to Rust (by design — stays in Erlang). |
| 17 | AtomVM Edge Runtime | V12-017 | MIXED | `just erlang-test-atomvm-differential` (real subprocess bridge to the real Erlang eunit differential suite, PROJ-761/762) | Real and functioning — re-run independently this session (`-- --ignored`), **1/1 passed**. 1 stub fn always refuses: `live_atomvm_target_evidence` (no `atomvm` binary present on this machine, genuinely absent). |
| 18 | Broker and Zero Unreceipted Actuation | V12-018 | MIXED | `cng::bench::dispatch.rs` state-table pattern (adapted); `arazzo_runner_broker.erl` idempotency/token pattern (adapted) | Real, hand-typed from adapted patterns (0 direct external-crate calls — adaptation, not linkage). No always-refusing stub fn found. 19/19 own tests incl. a 16-thread concurrency test. |
| 19 | Hooks and Action-Capability Resolution | V12-019 | MIXED | `praxis_graphlaw::hooks::{validate_and_extract_hooks,compile_hooks,schedule_hooks,hook_hash}` | Real; no always-refusing stub fn found. Idempotency ledger disclosed in-memory-only. |
| 20 | External Dispatch and Re-admission | V12-020 | ALREADY_BUILT | `cng::bench::decomp::dispatch_bridge` (`dispatch_subworkflow_to_engine`, `collect_subworkflow_consequence`) | Real thin wrapper with real filesystem-I/O tests. No always-refusing stub fn found. Cannot construct a generic `DispatchContract` (crate-private in `cng`, disclosed). |
| 21 | Parent-Child Closure | V12-021 | MIXED | `praxis_graphlaw::chatman::closure::RecursiveSocketClosure`; `powl2_decompose::ParentChildClosure` | Real composition fn (`admit_child_and_evaluate`). No always-refusing stub fn found — but no L6 receipt chain, no cascading closure, no L7 durability exist at all (disclosed absence, not stub-wrapped). |
| 22 | Timeout Retry Escalation and Compensation | V12-022 | MIXED | `praxis_graphlaw::chatman::compensation::{CompensationLedger,manufacture_compensation_workflow,...}` | Reused ledger is real (16/16 upstream tests). **All 5 of this family's own novel stage functions always refuse** `NotYetImplemented` (verified this session): `detect_timeout`, `evaluate_retry_policy`, `resolve_escalation`, `dispatch_broker_recovery`, `admit_recovery_attempt_idempotently`. |
| 23 | OpenTelemetry RDF Admission | V12-023 | MIXED | `cng::{otel_rdf,otel_ocel,otel_receipt}` (`admit`, `project_admitted_spans`, `receipt_otel_to_ocel`, ...) | Real thin-wrap composition (`admit_project_receipt`), exercised end-to-end incl. two refusal-path tests. No always-refusing stub fn found. |
| 24 | SPARQL CONSTRUCT to OCEL | V12-024 | MIXED | `cng::otel_ocel::project_otel_to_ocel` (real SPARQL CONSTRUCT); `cng::otel_receipt` | Real end-to-end `run_construct()`. 2 stub fns always refuse (verified this session): `idempotency_gate`, `mfw_feedback_adapter`. |
| 25 | Receipts and Replay | V12-025 | MIXED | `ChatmanEngine::verify_replay` fail-fast-compare pattern (adapted); `cng::otel_receipt` PROV-O writer pattern (adapted) | Real, hand-typed from adapted patterns (16/16 own tests). 1 stub fn always refuses: `chaos_gate::admit_for_replay` (L7). |
| 26 | Public Ontology Self Play | V12-026 | MIXED | `praxis_graphlaw::TripleStore` (Datalog/SHACL); `pddl_index::solve_indexed`; `chatman::powl_projection` | Real for 4.5 of 9 pipeline stages (20 tests, real end-to-end chain Turtle→N3→SHACL→PDDL→POWL→hook→receipt). **3 of 9 stages always refuse** `NotYetImplemented` (verified this session): D2 Scenario Generator, D8 Discovery Miner, D9 CONSTRUCT Capitalizer. |
| 27 | Western Electric Workflow Genesis | V12-027 | MIXED | `wasm4pm/wasm4pm/src/spc.rs::check_western_electric_rules` (ported, Rules 1/2/4 only); same-crate `f08_pddl_planning` reuse; `cng::powl` | Real (23 tests incl. all ported rules + full pipeline). Rule 3 (monotone trend) explicitly not ported — a disclosed partial omission, not a full always-refuse stub. |
| 28 | Multi-Breed Executable Process Science | V12-028 | MIXED | `wasm4pm_cognition::breeds::dispatch_breed` (4 real breeds: Bayesian/event-calculus/Allen-temporal/abductive-IBE); `wasm4pm_planner` (this crate is its first real consumer in the workspace) | Real (20 tests, 2 real bugs found+fixed via test failures). 1 stub fn always refuses: `locate_scale` (Scale Analyzer). Hand-written SPC substitutes for a version-pin-blocked reuse target (disclosed). |
| 29 | Thermodynamic Capability Roadmap | V12-029 | MIXED | `bcinr_powl_receipt::replay::PowlReplayVerifier`; `praxis_graphlaw::chatman::abi::Receipt`; `cargo-cicd` standing model (read for vocabulary only, not invoked) | Real — all 8 pipeline stages built (24 tests), several deliberately generic/reduced-scope (disclosed: no live self-play feed, no OS-level sandbox, no live `standing.json` parse). No always-refusing stub fn found. |
| 30 | GGEN Dynamic Project State and Release Admission | V12-030 | MIXED | `crates/ggen::graph::{DeterministicGraph,GraphEngine}`; `packs/jira-tracking-pack` SPARQL property-path pattern (adapted) | Real (19 tests, durable JSONL receipt-head journal survives a simulated restart). No always-refusing stub fn found. Verifier Aggregator does not itself spawn `cargo test`/`just verify-all` (disclosed scope limit). |

## Hand-written percentage (mechanical measurement, this session)

Total crate LOC: **26,868**. Ggen-generated LOC (`*_generated.rs` + `*_vocab.rs`, real `ggen sync
run` output, mechanically distinguishable from hand-typed files): **1,889 (7.0%)**.

This is the only bucket cleanly separable by mechanism rather than by trusting each family's own
REUSE_ADAPT-vs-HAND_WRITE_REQUIRED self-classification. A direct grep for explicit calls into the
reused/adapted external crates (`praxis_graphlaw::`, `cng::`, `bcinr_*::`, `wasm4pm_*::`,
`powl2_decompose::`, `pddl_index::`, `praxis_core::`, `ggen::`) across the 29 primary
(non-generated) module files found **331 call sites across 24,979 lines** — roughly one
reuse/adaptation call site per 75 lines, including in files whose own doc comments claim
"REUSE_ADAPT" (e.g. F18's 1,226-line `f18_broker_law.rs` has **zero** such call sites; its reuse
is pattern-adaptation from reading `cng`/Erlang source, not linked calls — legitimate per the
product owner's own "real adaptation" framing, but mechanically indistinguishable from novel
hand-writing).

**Real number, not rounded favorably:** on the only mechanically-verifiable split (ggen-generated
vs. everything else), this pass produced **~7% generated scaffolding and ~93% hand-typed Rust**
(24,979 of 26,868 lines) — far from the product owner's ~20%-hand-write / ~80%-reuse-or-generated
target. Some fraction of that 93% is legitimate REUSE_ADAPT (calling into or closely mirroring
existing code) rather than novel invention, but auditing each family's adaptation claim against
its cited source line-by-line was outside this session's scope; that self-reported split is
UNVERIFIED here. Separately, 13 of 30 families ship at least one always-refusing placeholder
function for a named, real capability (rows 8, 9, 11, 12, 13, 14, 16, 17, 22, 24, 25, 26, 28
above) — code that is 100% hand-typed and 0% functional, working against the 20% target in the
opposite direction from "legitimate adaptation."

## See Also

- `../PRD.md` — requirements skeleton, links to the atlas as the source of truth
- `/Users/sac/Downloads/v26.7.12_mermaid_atlas/` — the atlas itself (240 diagrams, 30 families)
- `crates/multifractal-workflow/src/lib.rs` — module table this index's Verdict column is drawn from
- `docs/jira/v26.7.11/tickets/index.md` — predecessor milestone (not modified by this pass)

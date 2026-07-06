# ARD — Architecture Reference Document, Praxis v26.7.6 "After Neon"

Companion to `C4.md` (diagram index + fence table) and `RELEASE_CONTROL.md`
(exit criteria). Every claim below cites a file, test, or receipt; rows that
cannot are marked UNKNOWN or GAP.

## 1. Architecture summary

Praxis manufactures standing for AI-generated technical work. Nothing enters
the ecosystem by assertion: facts are admitted by law-state (GraphLaw), plans
are searched (PDDL), execution takes workflow shape (POWL) and runs on a
deterministic hot path (bcinr), artifacts are manufactured (ggen), gauged
(Lean/Lake + cargo test + `mfg::validate`), and evidenced by computed receipts
(BLAKE3, genesis-folded, `ts_ns=0`). v26.7.6 closes this loop natively:
`crates/praxis-graphlaw` (roxi adoption, commit `88216f2`) is the default
`GraphEngine` inside ggen (commit `564543d`), and the vertical loop runs as a
composed slice (`src/plan_run.rs`, commit `8336f29`, proven by
`tests/plan_run_e2e.rs`).

```
law-state -> plan search -> workflow -> hot path -> factory -> gauges -> evidence -> publication
```

(`RELEASE_CONTROL.md` Sec. 2 carries the fully annotated version of this loop.)

## 2. Components

| Component | Location | Role (see fence table in `C4.md`) |
|---|---|---|
| GraphLaw engine | `crates/praxis-graphlaw` + seam `crates/ggen/src/graph.rs` | meaning + standing |
| PDDL planner | `bcinr-pddl` (path dep, root `Cargo.toml:97`) + `crates/pddl-index` | search |
| POWL workflow engine | `bcinr-powl` (`Cargo.toml:100`) + `crates/powl2-decompose` | workflow shape |
| bcinr hot path | `/Users/sac/bcinr/crates/` (logic/powl/powl-receipt) | execution |
| ggen factory | `crates/ggen` (graph.rs, sync.rs, template.rs, verbs/) | manufacturing |
| Verifier gates | `crates/praxis-lean`, `src/mfg.rs::validate`, `just verify-all` | judgment |
| Receipt chain | `bcinr-powl-receipt` + `src/ops.rs::receipt_issue_payload` | evidence |
| Report publisher | ggen sync projections into `docs/releases/v26.7.6/` | publication |
| CLI + MCP | `src/main.rs`, `src/verbs/`, `src/bin/mcp_lawobject_server.rs` | command surface |

Full crate census with ALIVE/PARTIAL classifications: `INVENTORY.md`
("Praxis workspace crates" table).

## 3. Core invariant

Standing is derived, never asserted. Concretely (RELEASE_CONTROL.md Sec. 4):

1. No panics/silent defaults — every error a typed `Refusal`/`AppError`
   variant. GraphLaw introduced `FM-LAW-001..013`, all typed via
   `AppError::fm_law` (`crates/ggen/src/error.rs`; enumerated in
   `GRAPHLAW_FEATURES.md`).
2. Receipts computed (BLAKE3, genesis-folded), never asserted-in
   (`bcinr-powl-receipt/src/causal_receipt.rs`).
3. No wall clock in any hash/receipt path — `ts_ns=0` throughout
   `src/plan_run.rs`; run id = BLAKE3 of the source graph hash.
4. Closed vocabularies (`wf:`, `hook:`, `prayer-kernel:`, `agent:`) — unknown
   predicates refused by name (`docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md`).
5. `praxis-synthesis` deps frozen (`crates/praxis-synthesis/tests/no_llm_runtime.rs`).
6. Smallest diff, reuse first; `crates/ggen` denies unsafe/todo!/print_stdout
   (`crates/ggen/src/lib.rs` header lints).

## 4. Object model

The unit of work is the Fused Law Object: obligation + lifecycle + receipt +
OCEL, defined in `crates/praxis-core` (its `lib.rs` header states exactly
this; `#![deny(unsafe_code)]`, `#![warn(missing_docs)]`). Objects surface as:

- RDF individuals in the admitted graph (`schema/praxis.ttl`, pack ontologies
  under `packs/`).
- PDDL objects after manufacture (`src/mfg.rs::extract_domain/extract_problem`).
- OCEL causal frames at execution (`OcelCausalFrame`, one per fired atom).
- Ledger entries in `.ggen-v2/receipt.json` (append-only JSONL chain).

## 5. Standing model

Standing is a law-derived predicate over admitted facts, never a stored flag:

- Breed/algorithm standing derives from `ontology/rules/breed_standing.n3`;
  `BREED_ALGORITHM_REGISTRY.md` is a pure SPARQL projection of the result
  (55 cognition breeds, each `EvidenceBound` with a literature citation).
- Denial rules (`{ body } => false.`) refuse sync on inconsistency
  (`FM-LAW-011`, `tests/graphlaw_e2e.rs::denial_violation_refuses_sync`).
- Clients display and command standing; they do not create it
  (`CLIENT_SURFACES.md` doctrine line). The client adapter is a typed gap:
  BLOCKED_TYPED in that document.
- GAP: a standing-specific vocabulary/fixture pack is Partial
  (`GRAPHLAW_FEATURES.md`, "Standing derivation" row).

## 6. Rule model

Three law layers, all inside `crates/praxis-graphlaw`, all gated in ggen sync:

| Layer | Engine surface | ggen surface | Proof |
|---|---|---|---|
| N3 forward chaining | `rule.rs`, `reasoner/`, `csprite.rs` | `ggen.toml [law].rules` | `tests/graphlaw_e2e.rs::when_guard_passes_only_after_n3_materialization` |
| Datalog (stratified negation, aggregates) | `datalog.rs` (`validate_rules` strata) | engine-level; no ggen fixture yet (GAP) | `GRAPHLAW_FEATURES.md` Datalog row |
| SHACL / ShEx | `shacl.rs`, `shex*.rs` | `[law].shapes` pre-render gate; ShEx wired but unconsumed (GAP) | `tests/graphlaw_e2e.rs::shacl_violation_refuses_sync_naming_focus_node` |

Backward chaining exists in the engine (`backwardchaining.rs`) but is not
surfaced as a verb (GAP, per `GRAPHLAW_FEATURES.md`).

## 7. Planner domain

- Facts → PDDL8 manufacture: `src/mfg.rs` (`load_graph`, `extract_domain`,
  `extract_problem`, `enforce_pddl8`, `emit_domain`, `emit_problem`,
  `manufacture`); the 5-step lawobject plan is pinned by `tests/mfg_golden.rs`.
- Solve: `bcinr-pddl` `GroundProblem::find_plan`; indexed grounding
  auto-selected by `pddl_index::should_use_indexed`
  (`src/ops.rs::plan_solve_payload`).
- Temporal lane: `GroundTemporalProblem`, `analyze_schedule`,
  `execute_temporal_plan` via `src/verbs/plan.rs` (solve/analyze/execute).
- Independent cross-check lane: `wasm4pm-planner` (root `Cargo.toml:114-116`),
  deliberately outside the standing path.
- Goal proposals: `ops::propose_revenue_payload` / `propose_goal_payload`
  emit ranked candidates and a splice-ready PDDL goal atom
  (`src/bin/mcp_lawobject_server.rs`).

## 8. CLI architecture

- Root bin: `src/main.rs` + `src/cli.rs`; verbs under `src/verbs/`
  (plan, receipt, mfg, ...). `plan run` is the composed vertical verb
  (feature `ggen`, `src/verbs/plan.rs`).
- ggen bin: `crates/ggen/src/main.rs`; law verbs `load/validate/derive/
  explain/export` are generated routes from `schema/praxis.ttl`
  `praxis:CmdGgenLaw*` instances into `crates/ggen/src/verbs/law.rs`, with
  handlers in `verbs/handlers.rs` — the CLI surface itself is manufactured
  from the graph.
- MCP servers: `src/bin/mcp_lawobject_server.rs` (plan_solve,
  propose_revenue, propose_goal), `src/bin/mcp_server.rs`.
- Refusal-completeness across the surface is exit criterion 5 and is NOT yet
  done (`RELEASE_CONTROL.md` Sec. 5).

## 9. File architecture

```
/Users/sac/praxis
├── src/                    root CLI, mfg.rs, ops.rs, plan_run.rs, verbs/, bin/
├── crates/
│   ├── praxis-graphlaw/    law engine (roxi adoption)
│   ├── ggen/               factory (graph.rs seam, sync.rs, template.rs)
│   ├── praxis-lean/        Lean/Lake admission gate
│   ├── pddl-index/         indexed grounding
│   ├── powl2-decompose/    WF-net <-> POWL 2.0
│   ├── praxis-core/        Fused Law Object
│   ├── praxis-synthesis/   frozen-dep synthesis pipeline
│   └── ...                 full census in INVENTORY.md
├── schema/praxis.ttl       command + ontology graph
├── packs/                  fact packs (e.g. wasm4pm-facts-pack)
├── examples/v26_7_6_after_neon/   demo fixture
├── tests/                  plan_run_e2e.rs, mfg_golden.rs, ...
├── benches/                bench_main.rs, receipt_validate.rs
├── docs/architecture/c4/   the 7 diagrams (this release)
├── docs/releases/v26.7.6/  release doc suite
└── .ggen-v2/receipt.json   append-only receipt ledger
```

Sibling path deps: `/Users/sac/bcinr`, `/Users/sac/wasm4pm`,
`/Users/sac/wasm4pm-compat` (root `Cargo.toml:96-116`; the `lsp-max` /
`wasm4pm-compat` patch story is documented at `Cargo.toml:153-163`).

## 10. Dataflow

The single loop (diagram 6, `docs/architecture/c4/06_dynamic_loop.puml`):

1. Turtle facts in → GraphLaw materialize (N3/Datalog) → SHACL/denial gates.
2. Admitted graph → `mfg::manufacture` → `domain.pddl` + `problem.pddl`.
3. `find_plan` → plan → `compile_powl` → `PowlTape`.
4. `scheduler_tick` fires atoms; one `OcelCausalFrame` each, `ts_ns=0`.
5. Artifact write + `mfg::validate` (must report solvable).
6. Final ledger receipt (`receipt_issue_payload`, `ts_ns: 0`).
7. Receipts and new facts re-enter law-state; replan.

Determinism: two consecutive runs produce identical `powl_chain_hash`
(`tests/plan_run_e2e.rs`); ledger receipt hashes differ only by ledger
position, by design of the append-only chain (`PLANNER_SURFACE.md`).

## 11. Post-cyberpunk design system

The documentation and product register: civic infrastructure after the neon
wears off. Vocabulary is legal-industrial — law-state, admission, standing,
factory, gauge, receipt, replay — not mystical or hype-driven. Rules:

- No naked success claims: every claim cites a test/receipt/file
  (`RELEASE_CONTROL.md` header rule; enforced editorially in every doc in
  this suite).
- Reports are projections of the graph, marked GENERATED, never hand-edited
  (`BREED_ALGORITHM_REGISTRY.md` header).
- Refusals are named, typed, and legible (`FM-LAW-*` codes) — the system
  says no in writing, like a permit office, not silently like a black box.

## 12. Demo architecture

- Fixture: `examples/v26_7_6_after_neon/` (goal TTL + facts query + artifact
  template note + README with the exact command), per `PLANNER_SURFACE.md`.
- Command: `plan run` (feature `ggen`) executes the full loop of Sec. 10.
- Exit criterion 3 (`RELEASE_CONTROL.md`): one-command demo, deterministic
  across 2 runs, byte-identical receipts — status NOT STARTED → the e2e
  determinism test exists (`tests/plan_run_e2e.rs`); the receipts-captured
  control-doc row remains to be flipped by the release controller.

## 13. Market architecture

- Client doctrine: clients display and command standing, never create it
  (`CLIENT_SURFACES.md`). Role-mapped candidates: optimus (Next.js web
  control room), pcp (Expo mobile operator console), dashboard.bak /
  wasm4pm playground-web (browser shell) — classifications and evidence in
  that document.
- The Praxis→client adapter (report/receipt JSON contract) is BLOCKED_TYPED —
  a declared, typed gap, not a silent one (`CLIENT_SURFACES.md` acceptance
  table).
- North star: Blue River Dam working-backwards PRFAQ, explicitly
  non-standing (commit `7c161ad`, `docs/` vision suite).

## 14. Adversarial architecture

- Review stance: state findings, not verdicts; explore mines requirements,
  exploit rewrites clean-room from the invariant (`CLAUDE.md` Reporting).
- Attack surface is enumerated, not hidden: `RELEASE_CONTROL.md` Sec. 5
  keeps unproven rows UNKNOWN/NOT STARTED; `INVENTORY.md` classifies every
  surface with evidence including BROKEN/MISSING/DUPLICATE.
- Anti-gaming gates: `tests/no_llm_runtime.rs` (no LLM in the synthesis
  runtime), `crates/praxis-lean/src/no_sorry.rs` (no admitted `sorry`),
  `ocel/anti_llm_cheat_lsp_ocel.json` (prior-release anti-cheat artifact).
- Determinism itself is adversarial armor: byte-identical replays make
  fabricated evidence detectable (`two_runs_same_fixture_same_graph_hash_
  and_valid_chain`, `crates/ggen/tests/graphlaw_e2e.rs`).

## 15. Final-day outputs

Per `RELEASE_CONTROL.md` Sec. 5, the release closes when all seven exit
criteria carry proof. Deliverables of record:

| Output | Where | Status at authoring |
|---|---|---|
| 15 release docs | `docs/releases/v26.7.6/` | 8 files present (`ls`): this doc, C4.md, RELEASE_CONTROL, INVENTORY, GRAPHLAW_FEATURES, PLANNER_SURFACE, CLIENT_SURFACES, BREED_ALGORITHM_REGISTRY |
| 7 C4 diagrams | `docs/architecture/c4/*.puml` | present (this task) |
| GraphLaw-in-ggen e2e proof | `crates/ggen/tests/graphlaw_e2e.rs` | tests exist per `GRAPHLAW_FEATURES.md` |
| Vertical-loop demo + determinism | `tests/plan_run_e2e.rs`, `examples/v26_7_6_after_neon/` | exists per `PLANNER_SURFACE.md`; control-doc row not yet flipped |
| Breed/algorithm admission | commit `6e8f5a2`, `BREED_ALGORITHM_REGISTRY.md` | done |
| `just verify-all` green | receipts section of `RELEASE_CONTROL.md` | UNKNOWN until captured |

## 16. Definition of done

A ticket, feature, or release row is done only when:

1. `just verify-all` passes (the DoD gate, `justfile`).
2. The claim cites a specific test, receipt, or file that exists.
3. No invariant of Sec. 3 is violated by the diff.
4. The change is the smallest diff that closes the gap (reuse first).
5. Receipts for the run are computed and chained — never asserted.

Anything short of all five stays UNKNOWN in `RELEASE_CONTROL.md`. That table,
not this document, is the single control surface for release status.

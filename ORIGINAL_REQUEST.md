# Original User Request

## Initial Request — 2026-06-22T05:25:49Z

Upgrade the `~/praxis` boilerplate generator by integrating architectural insights from the Chatman ecosystem (`rocket-craft`, `lsp-max`, and generative typestates). Catalog each Rust library to identify abstractions and components that can be extracted and contributed to the upgraded generator, and apply these upgrades to the codebase.

Working directory: `~/praxis`
Integrity mode: development

## Requirements

### R1. Ecosystem Catalog and Abstraction
Catalog the Rust libraries in the `~/rocket-craft` and `~/lsp-max` workspaces. Identify and document architectural patterns, abstractions, and components (such as Generative Typestates, `RulePackServer`, and the `ggen` µ-pipeline) that can be abstracted and contributed to the `praxis` generator.

### R2. Praxis Generator Upgrade
Upgrade the `~/praxis` boilerplate generator codebase. The upgraded generator must produce boilerplate that natively implements the "Post-Chatman Equation" ($A = \mu(O^*)$) ecosystem insights, specifically targeting the emission of typestate-driven configurations and `RulePackServer` structures over manual scaffolding.

## Acceptance Criteria

### Documentation
- [ ] A comprehensive Markdown catalog exists detailing the analyzed Rust libraries, extracted abstractions, and integration strategies.

### Implementation & Verification
- [ ] The `~/praxis` codebase contains the implemented Rust code upgrades.
- [ ] Executing the upgraded `praxis` generator successfully emits a sample boilerplate project.
- [ ] The emitted sample project successfully compiles (`cargo check` passes).
- [ ] A programmatic verification script confirms the emitted project structurally conforms to Post-Chatman principles (e.g., detects `PhantomData` typestates or `RulePackServer` implementations).

## Follow-up — 2026-06-24T03:16:08Z

# Teamwork Project Prompt — Draft

> Status: Ready for launch — awaiting user approval.
> Goal: Craft prompt → get user approval → delegate to teamwork_preview

Audit the `/Users/sac/praxis` codebase to ensure there are no stub implementations or incomplete capabilities. The team must use the `~/anti-llm-cheat-lsp/teamwork-preview` tool/script to perform the check, and apply strict Rust core team reasoning to identify and resolve any stubs (e.g., `todo!()`, `unimplemented!()`, or placeholder logic).

Working directory: /Users/sac/praxis
Integrity mode: development

## Requirements

### R1. Audit Capabilities
Run the `~/anti-llm-cheat-lsp/teamwork-preview` tool against the `/Users/sac/praxis` codebase to identify any capabilities containing stub code or placeholders.

### R2. Replace Stubs with Implementations
For every stub identified, implement the missing functionality following strict Rust core team reasoning. Do not leave any `todo!()`, `unimplemented!()`, or placeholder logic behind.

## Acceptance Criteria

### Audit Passes Cleanly
- [ ] Running `~/anti-llm-cheat-lsp/teamwork-preview` exits with no errors or warnings regarding stubs in the codebase.

---
*Next: when approved → delegate via invoke_subagent (see Delegation Protocol)*

## Follow-up — 2026-06-29T18:54:59Z

Implement the 4 JIRA integration tickets in the `/Users/sac/praxis` workspace.

Working directory: `/Users/sac/praxis`
Integrity mode: development

## Requirements

### R1. Configure cargo-cicd (Ticket 001)
- Create `/Users/sac/praxis/cicd.toml` with:
  ```toml
  [target]
  max_size_gb = 5.0
  prune_after_days = 7

  [test.changed]
  base = "origin/main"

  [autonomic]
  enabled = true
  mode = "suggest"
  ```
- Update or create `/Users/sac/praxis/justfile` to define:
  - `test-changed`: `cargo cicd test changed`
  - `clean-stale`: `cargo cicd target prune`
- Create or update `/Users/sac/praxis/.github/workflows/ci.yml` to run `cargo cicd workspace doctor`.

### R2. Integrate chicago-tdd-tools Configuration Layering (Ticket 002)
- Add `chicago-tdd-tools = "=26.6.29"` (registry version, no path override) to `crates/praxis-retrofit/Cargo.toml`.
- Refactor TOML loading in `repo_registry.rs` to support:
  - Upward parent directory scans for `repos.toml`.
  - Override path layering using the `PRAXIS_REGISTRY_PATH` environment variable.

### R3. Property-Based Configuration Invariants (Ticket 003)
- Declare `proptest` dev-dependency.
- Implement type-level poka-yoke wrappers (`PositiveUsize`, `BoundedU32`) to enforce TOML invariants (`crate_count` and `priority_score`).
- Implement property-based test harness at `crates/praxis-retrofit/tests/property_tests.rs`.

### R4. Ocel Event Tracing & Build Traces (Ticket 004)
- Add a compile-time hook to `build.rs` to log build metrics.
- Configure `OcelCollector` as a `DiagnosticSink` in tests to write logs to `target/praxis/evidence/events.ocel.json`.

## Acceptance Criteria

### Execution & Verification
- [ ] `cargo check --workspace` and `cargo test --workspace` run and pass cleanly.
- [ ] `cargo cicd workspace doctor` and `cargo cicd target prune` exit successfully (status 0).
- [ ] All verification steps listed in `ticket_001_setup_cicd.md`, `ticket_002_config_layering.md`, `ticket_003_property_testing.md`, and `ticket_004_ocel_tracing.md` pass successfully.
- [ ] There are no `todo!` or placeholder segments remaining in the modified codebases.

## Follow-up — 2026-06-29T19:56:10Z

Research the filesystems of `~/praxis` and `~/wasm4pm`, design a combinatorial maximalism integration strategy, and construct a working prototype using Praxis as a live testbed.

Working directory: `/Users/sac/praxis`
Integrity mode: development

## Requirements

### R1. Filesystem & Capabilities Research
Analyze `/Users/sac/praxis` and `/Users/sac/wasm4pm` filesystems. Identify and map all validation schemas, Ocel 2.0 log definitions, compiled WASM court rules, and Petri Nets/POWL models to document how they complement each other.

### R2. Detailed Integration Blueprint
Create a comprehensive research blueprint at `/Users/sac/praxis/docs/wasm4pm-integration/blueprint.md` detailing the integration architecture across three core dimensions:
- **Conformance**: Replaying Praxis Ocel 2.0 logs against wasm4pm Petri Net PNML models.
- **Cryptography**: Signing and verifying Praxis compliance receipts using wasm4pm cryptographic public/private keys.
- **Autonomic Loop**: Feeding wasm court verdicts directly into cargo-cicd autonomic warning and suggest loop systems.

### R3. Working Prototype Script & Demo
Develop a working prototype script at `/Users/sac/praxis/docs/wasm4pm-integration/run_demo.sh` (or a similar execution script) that:
- Reads a Praxis OCEL 2.0 log file (e.g. `target/praxis/evidence/events.ocel.json`).
- Validates its conformance or signing using a `wasm4pm` WASM rule or validator CLI.
- Outputs a verdict and logs the execution output.

## Acceptance Criteria

### Execution & Deliverables
- [ ] The folder `/Users/sac/praxis/docs/wasm4pm-integration/` contains both `blueprint.md` and `run_demo.sh`.
- [ ] Running the prototype script `/Users/sac/praxis/docs/wasm4pm-integration/run_demo.sh` completes successfully and displays the conformance / verification verdict.
- [ ] No stubs, placeholders, or `todo!` markers are left in the generated blueprint or prototype script files.

## Follow-up — 2026-06-29T21:26:23Z

Review the codebases (`praxis`, `wasm4pm`, `chicago-tdd-tools`) and thesis drafts/home files to design a roadmap and structured JIRA tickets for the `v26.6.31` release phase.

Working directory: `/Users/sac/praxis`
Integrity mode: development

## Requirements

### R1. Comprehensive Review & Synthesis
Analyze the source code and design documentation in `/Users/sac/praxis`, `/Users/sac/wasm4pm`, `/Users/sac/chicago-tdd-tools`, and relevant files in the home directory `~/`. Synthesize:
- Sean Chatman's thesis draft on **operational physics** and **impossibility harvesting**.
- Existing `cargo-cicd` compliance gates and the `wasm4pm` verification courts.

### R2. Strategic Roadmap Generation
Design a comprehensive release roadmap for the next development phase (`v26.6.31`), detailing how to implement next-generation verification methods, impossibility sensors, and WASM-based compliance courts in the testbed.

### R3. JIRA Ticket Generation
Generate a structured folder of individual JIRA tickets at `/Users/sac/praxis/docs/jira/v26.6.31/tickets/`.
- Each ticket must be written to its own markdown file (e.g. `ticket_001_xxx.md`).
- Each ticket must contain: Title, Description, Acceptance Criteria, and Dependencies.
- An index document (`index.md`) must be provided containing the execution sequence, dependency graph, and ticket summaries.
- At least 4 distinct tickets must be generated, outlining concrete, actionable engineering tasks.

## Acceptance Criteria

### Deliverables & Formatting
- [ ] The target directory `/Users/sac/praxis/docs/jira/v26.6.31/tickets/` contains `index.md` and all ticket markdown files.
- [ ] At least 4 distinct tickets are generated, covering the execution of thesis-driven impossibility harvesting and compliance courts.
- [ ] Each ticket contains non-empty sections for Title, Description, Acceptance Criteria, and Dependencies, with no stubs, placeholders, or `todo!` markers.
- [ ] Every ticket defines an objective verification mechanism (e.g., specific test suites, configuration parameters, or command executions).

## Follow-up — 2026-06-29T22:14:21Z

Research `wasm4pm` capabilities and design the ultimate Rust project process intelligence template under `/Users/sac/praxis/templates/process-intelligence/`.

Working directory: `/Users/sac/praxis`
Integrity mode: development

## Requirements

### R1. Capabilities Synthesis & Research
Analyze `wasm4pm` and `praxis` systems to extract all telemetry, conformance, cryptographic verification, and Chicago-style testing capabilities.

### R2. Reusable Template Workspace
Create a fully bootstrap-able Rust workspace template under `/Users/sac/praxis/templates/process-intelligence/` containing:
- **`Cargo.toml`**: Pre-configured with `chicago-tdd-tools = "=26.6.29"`, `proptest`, `serde`, and `serde_json`.
- **`cicd.toml`**: Pre-configured `cargo-cicd` rules (target limits, autonomic suggest mode, test changed base).
- **`Justfile`**: Recipes for running verification courts (`wpm validate`), compiling, target pruning, and running changed tests.
- **`build.rs`**: Boilerplate compile-telemetry hook logging compiler phases and events.
- **`petri_net_lawful_dispatch.pnml`**: Default Petri Net model specifying standard build-verify-adjudicate transition chains.

### R3. Crate & Telemetry Boilerplate
Inside the template, create a sample crate (`crates/sample-service/`) containing:
- Chicago-style TDD unit tests, property-based tests (fuzzing TOML/structures), and snapshot tests.
- Telemetry configuration using `OcelCollector` to log events dynamically.

### R4. Documentation
Provide a `README.md` inside the template directory detailing the bootstrapping instructions and how to run process intelligence verifications.

## Acceptance Criteria

### Execution & Verification
- [ ] The target directory `/Users/sac/praxis/templates/process-intelligence/` is populated with all template files.
- [ ] Running `cargo check` and `cargo test` inside the template directory compiles and passes cleanly.
- [ ] Running `cargo cicd workspace doctor` inside the template directory exits successfully (status 0).
- [ ] The template contains no stubs, placeholders, or `todo!` markers.

---

## Reference: wasm4pm Source Folder Structure & Integration Mapping

```text
apps
├── playground-web (Web dashboard mockups/prototypes)
│   --> Integration: Pre-configure UI views or schema maps for visualizing log conformance.
└── wasm4pm (Main Node.js CLI packages & build commands)
    --> Integration: Root Justfile execution bindings triggering "wpm" validation commands.

crates
├── prolog8 (Prolog resolution & ontology solver)
│   --> Integration: Axiomatic rule templates (e.g. prolog8 rules asserting clean support chains).
├── miniml-core (Deductive/inductive reasoning core)
│   --> Integration: Logic evaluation helpers for testing structural rules.
├── ocpq (Object-centric process query processing)
│   --> Integration: Boilerplate query scripts to query the generated OCEL logs.
├── bench-tools (Benchmark harnesses)
│   --> Integration: Conformance replay performance benchmarks using Criterion/tick budgets.
├── wasm4pm-cli (Core Rust CLI binary codebase)
│   --> Integration: CLI execution wrappers in Justfile/scripts.
├── wasm4pm-lsp (LSP Server implementation)
│   --> Integration: Diagnostic configuration files linking the LSP linter to Cargo/cicd files.
└── wasm4pm-cognition (Axiomatic breeds and lifecycle sensors)
    --> Integration: build.rs telemetry hook using cognition lifecycle sensors.

packages
├── config (WASM and CLI config schema bindings)
│   --> Integration: Schema descriptors (receipt.schema.json, ocel2.json) copied to template.
├── cognition (Structural and adversarial models)
│   --> Integration: Invariant definitions for state representation in sample tests.
├── planner (Plan generation algorithms)
│   --> Integration: Corrective plan templates in autonomic response configurations.
├── contracts (Receipt layout validation structures)
│   --> Integration: Truex compliance receipt templates (PiReceipt & CommandReceipt layouts).
├── agents (MCP and coordinator abstractions)
│   --> Integration: Agent/Hooks interaction configuration templates (.agents/hooks.json).
├── observability (Spans, trace context providers)
│   --> Integration: OcelCollector instrumentation configurations in crates/sample-service.
├── supabase (Cloud storage adapter)
│   --> Integration: Storage backend path configurations for receipts.
├── testing (Simulation and conformance mocks)
│   --> Integration: Conformance checking tests using wpm prefix-conformance.
├── ml (Centroid / page-hinkley algorithms)
│   --> Integration: Drift-detection test cases checking for execution time anomalies.
├── agent-context (Runtime conversation/system context)
│   --> Integration: Diagnostic context logging setup.
├── autopm (Autonomic loops and package verification)
│   --> Integration: Autonomic rollback scripts in case of validation court failure.
└── kernel (WebAssembly backends and prediction)
    --> Integration: WASM runtime environment settings and court loader files.

src
├── conformance (Prefix alignment verifiers)
│   --> Integration: Prefix-conformance models linking events to Petri net tokens.
├── replay (Petri net token replay simulator)
│   --> Integration: Simulation verification commands in the template.
├── mining (Log alpha miner algorithms)
│   --> Integration: Command templates to run alpha miners and output discovered net diagrams.
└── lifecycle (Process start/stop event lifecycles)
    --> Integration: Transition event triggers on build and test execution hooks.

examples (Self-contained execution examples & demos)
└── 01-supply-chain-drift, 02-incident-triage, 05-safety-process-guard, truex-cli
    --> Integration: Templates for script-based runbooks (SRE incident triage, safety process guards) under templates/process-intelligence/scripts/.
```


## Follow-up — 2026-07-03T17:47:28Z

# CLAUDE CODE MASTER PROMPT
# Praxis v26.7.3 — Component Crawler, Port Workflow, Verification, and Cleanup

You are Claude Code operating inside Sean Chatman's local development environment.

MISSION:
Crawl the local machine for WORKING COMPONENTS that can be ported into Praxis v26.7.3, port only the highest-confidence components that close actual v26.7.3 gates, verify them with tests/receipts/replay/foreign verification where applicable, and clean build/scratch artifacts when done.

This is not an archaeology report.
This is not a naming search.
This is not a vibe sweep.
This is not a speculative architecture task.

Find working components.
Score them.
Map them to v26.7.3 gates.
Port only what closes gates.
Test everything.
Receipt/refuse everything.
Clean up after yourself.

============================================================
TARGET RELEASE
============================================================

Praxis v26.7.3 =

Lord’s Prayer Kernel
→ Rice Quarantine
→ RDF Life Graph
→ PDDL DayWindow Action Grounding
→ Knowledge Hooks / Reflex / Autonomics
→ Agent / Handler Assignment
→ HumanOnly Delegability
→ Execution / Refusal
→ Receipts
→ Replay
→ Foreign Verification

The release thesis:

God receives the unbounded.
Rice Quarantine disciplines vague meaning.
RDF stores the admitted life graph.
PDDL defines bounded DayWindow action.
Knowledge Hooks handle deviations without LLM global planning.
Agents attach mechanically by capability.
HumanOnly boundaries prevent false delegation.
Receipts give standing.
Replay and foreign verification prove the chain within the stated scope.

No action has standing outside this chain.

============================================================
ABSOLUTE THEOLOGICAL / COMPUTATIONAL BOUNDARY
============================================================

God is not an agent.
God is not a handler.
God is not a tool.
God is not a planner.
God is not a capability provider.
God is not executable infrastructure.

God is the lawful receptacle of the unbounded.

The system may model only the human-side workflow response:

- surrender
- prayer
- daily bread
- confession
- forgiveness
- release
- temptation guard
- deliverance request
- repair
- service
- receipt
- refusal

The system must refuse or quarantine attempts to compute:

- God’s hidden will
- ultimate moral totality
- infinite consequence graphs
- the full future
- final spiritual certainty
- complete moral interpretation

Required rule:

Unbounded → God / surrender / quarantine / refusal

Never:

Unbounded → computed action certainty

============================================================
ABSOLUTE RUNTIME RULE
============================================================

No LLM global planning may exist on the execution hot path.

LLMs may only be used for:

- quarantine assistance
- summarization
- proposal generation
- documentation
- explanation
- non-authoritative classification suggestions

LLMs may not directly decide:

- action execution
- forgiveness
- repentance
- surrender
- confession
- moral standing
- admission authority
- agent assignment
- PDDL plan validity
- receipt validity
- replay validity
- foreign verification validity

Execution must be graph-driven, hook-triggered, PDDL-grounded, capability-assigned, and receipt-bound.

============================================================
WORKING COMPONENT DEFINITION
============================================================

A working component means one or more of:

1. It compiles.
2. It has tests.
3. It has receipts.
4. It has replay/verification logic.
5. It has a real parser/evaluator/grounder/runner.
6. It has a CLI or library entrypoint.
7. It has deterministic data structures or algorithms.
8. It has prior successful usage evidence.
9. It has a bounded, portable module with few dependencies.
10. It can close one v26.7.3 gate with minimal adaptation.

Do not port fossils.
Do not port theatre.
Do not port impressive dead code.
Do not port components that weaken existing tests.
Do not port hidden external-service dependencies.
Do not port code that introduces LLM runtime authority.

============================================================
V26.7.3 GATES
============================================================

Port candidates must advance one or more of these gates:

GATE-01 Lord’s Prayer kernel
GATE-02 God / unbounded boundary
GATE-03 Rice Quarantine
GATE-04 RDF graph / canonicalization
GATE-05 RDF delta → Knowledge Hook
GATE-06 Hook → PDDL grounded action
GATE-07 PDDL DayWindow planning
GATE-08 Agent / handler assignment
GATE-09 Delegability / HumanOnly enforcement
GATE-10 AA / livelock modeling
GATE-11 Resentment / spilled-milk unsound-loop repair
GATE-12 Receipts / replay
GATE-13 Foreign verification
GATE-14 No LLM global planning runtime
GATE-15 Claim discipline
GATE-16 Cleanup / disk hygiene

============================================================
OPERATING CONSTRAINTS
============================================================

1. Write only inside /Users/sac/praxis unless explicitly authorized.
2. Read from external lineage repos as needed.
3. Do not modify external repos.
4. Do not push.
5. Do not delete source files.
6. Do not rewrite history.
7. Do not add dependencies unless necessary and justified.
8. Do not copy secrets, tokens, credentials, private keys, .env values, or remotes with embedded credentials.
9. If secrets are found, report only the path/kind of risk. Do not print the secret.
10. Every claim must become proven, refused, or withheld.
11. Every imported component must get tests in Praxis.
12. Every imported component must map to a v26.7.3 gate.
13. Every imported component must preserve God/unbounded boundary and HumanOnly rules.
14. Every imported component must be classified IMPORT / ADAPT / REWRITE / REFUSE.
15. Prefer small complete closure over large impressive surfaces.
16. No unreceipted authority.
17. No asserted hash authority.
18. Only computed canonical hashes have standing.
19. No raw scripture-to-action execution.
20. No raw meaning-to-action execution.

============================================================
BUILD ARTIFACT DISCIPLINE
============================================================

When running Rust builds/tests in /Users/sac/praxis, normal target/ usage is allowed, but it must be removed in the final cleanup phase.

When probing external Rust repos, do not create persistent build artifacts there. Use temporary target dirs:

CARGO_TARGET_DIR=/tmp/praxis-port-crawl-target-<repo-name>

Do not run package managers in external repos unless necessary.

Do not create node_modules in external repos.

Do not modify:

- Cargo.lock
- package-lock.json
- pnpm-lock.yaml
- yarn.lock
- build outputs
- generated vendor files

unless explicitly required inside /Users/sac/praxis and justified.

============================================================
SEARCH ROOTS
============================================================

Crawl these roots if present:

/Users/sac/praxis
/Users/sac/unrdf
/Users/sac/knhk
/Users/sac/wasm4pm
/Users/sac/wasm4pm-compat
/Users/sac/ggen
/Users/sac/star-toml
/Users/sac/lsp-max
/Users/sac/cns
/Users/sac/bitactor
/Users/sac/bytestar
/Users/sac/anti-llm-cheat
/Users/sac/anti-llm-cheat-lsp
/Users/sac/claude-code-config-lsp
/Users/sac/chicago-tdd-tools
/Users/sac/process-intelligence

Also crawl immediate children of /Users/sac that are git repos.

Exclude:

target/
node_modules/
.git/
dist/
build/
.next/
.nuxt/
.cache/
venv/
.venv/
__pycache__/
vendor/
tmp/
logs/

============================================================
PHASE 0 — SAFETY + BASELINE
============================================================

Start read-only.

Run:

pwd
git -C /Users/sac/praxis status
git -C /Users/sac/praxis branch --show-current
git -C /Users/sac/praxis log --oneline -n 20

Scan git remotes for embedded credentials without printing secrets.

For every security finding, report:

SECURITY_FLAG:
- repo:
- remote name:
- risk:
- recommended action:

Do not print tokens or secrets.

Identify:

- current branch
- current HEAD
- dirty state
- existing v26.7.3 files
- existing graph/PDDL/hook/receipt/replay/foreign verifier files
- existing receipts
- existing docs
- existing plan files under /Users/sac/.claude/plans
- immediate blockers

Do not modify files in Phase 0.

Output:

SITUATIONAL LOCK:
- repo:
- branch:
- head:
- dirty: YES/NO
- relevant crates:
- existing verifier scripts:
- existing receipts:
- existing v26.7.3 artifacts:
- security flags:
- immediate blockers:

============================================================
PHASE 1 — COMPONENT CRAWL
============================================================

Launch 12 Explore agents.

EXPLORE-1 — RDF / Turtle / SHACL / OWL components

Find parsers, canonicalizers, graph hashers, SHACL shape readers, RDF delta appliers, SPARQL/CONSTRUCT tools, graph mutation tests.

EXPLORE-2 — Knowledge Hooks / hook engines

Find hook schemas, trigger/effect engines, condition evaluators, SPARQL ASK/SELECT conditions, SHACL block/annotate/repair modes, delta/window/count/threshold conditions, receipt-bound hook firing.

EXPLORE-3 — Reflex / autonomics / MAPE-K

Find reflex loops, monitor/analyze/plan/execute/knowledge systems, self-healing loops, auto-rollback, SLO triggers, guard-failure handlers, local repair systems.

EXPLORE-4 — PDDL / planning / action grounding

Find PDDL parsers, PDDL emitters, STRIPS/durative action structures, precondition/effect models, Solver8-like planners, CapabilityRoute planners, DayWindow action surfaces.

EXPLORE-5 — Agent / capability / handler assignment

Find agent registries, handler registries, capability matching, actor dispatch, tool dispatch, HumanOnly-like boundaries, authorized handler lookup, exact IRI binding.

EXPLORE-6 — Receipts / replay / verification

Find BLAKE3 receipts, previous-hash chains, replay engines, trustless replay scripts, foreign verifiers, Merkle/lockchain systems, forged-payload tests.

EXPLORE-7 — AA / livelock / recovery workflow

Find AA, 12 steps, inventory graph, resentment, harms, amends, sponsor, witness, livelock, unsound workflow, loop detection, no-infinite-rehearsal logic.

EXPLORE-8 — Lord’s Prayer / prayer / Gospel workflow

Find Lord’s Prayer, daily bread, forgive debts/debtors, temptation guard, deliverance, surrender, God boundary, unbounded threat, prayer-kernel terms.

EXPLORE-9 — Rice Quarantine / admission / refusal

Find quarantine, admission, NotExecutable, Refusal, withheld claim, declared refusal, action admission, raw meaning boundary, anti-LLM-cheat admission gates.

EXPLORE-10 — Foreign verification candidates

Find Python, JS, Rust, C, shell, or other verifiers that independently re-derive hashes, parse TTL, validate receipt chains, replay actions, or bind payloads.

EXPLORE-11 — Test harnesses / adversarial tests

Find tests that can be ported directly:

- forged payloads
- malformed TTL
- unknown handler
- missing handler
- HumanOnly bypass
- history mismatch
- receipt mismatch
- semantic mutation
- unsound workflow loop
- no LLM runtime path
- God boundary violation

EXPLORE-12 — Project census + dependency health

Map every git repo under /Users/sac:

- path
- language
- build system
- last commit
- test command
- likely concept family
- portability risk
- secrets risk
- import value

Each Explore agent must output:

- candidate name
- source path
- relevant files
- symbols/functions/types
- tests
- evidence that it works
- dependencies
- port difficulty: LOW / MEDIUM / HIGH
- v26.7.3 gate impacted
- recommendation: IMPORT / ADAPT / REWRITE / REFUSE
- reason

============================================================
PHASE 2 — WORKING COMPONENT FILTER
============================================================

Score every candidate.

Required scoring fields:

1. compiles_or_runs: YES / NO / UNKNOWN
2. has_tests: YES / NO
3. tests_pass_locally: YES / NO / NOT_RUN
4. has_receipts: YES / NO
5. deterministic: YES / NO / UNKNOWN
6. dependency_risk: LOW / MEDIUM / HIGH
7. code_size: SMALL / MEDIUM / LARGE
8. port_surface: FILE / MODULE / CRATE / APP
9. gate_closure_value: LOW / MEDIUM / HIGH / DECISIVE
10. claim_risk: LOW / MEDIUM / HIGH
11. security_risk: LOW / MEDIUM / HIGH
12. recommendation: IMPORT / ADAPT / REWRITE / REFUSE

Reject candidates with:

- no executable code
- no tests and high complexity
- unclear provenance
- secret-bearing code
- hand-wavy AI planner runtime
- hardcoded claims
- benchmark theatre
- quantum/retrocausal/theatre vocabulary
- hidden external service dependency
- credentials
- non-deterministic runtime authority

============================================================
PHASE 3 — PORT CANDIDATE CENSUS
============================================================

Create or update:

/Users/sac/praxis/docs/v26.7.3/PORT_CANDIDATE_CENSUS.md

Required format:

# v26.7.3 Port Candidate Census

## Executive verdict

- Number of repos scanned:
- Number of candidate components:
- IMPORT:
- ADAPT:
- REWRITE:
- REFUSE:
- Decisive candidates:
- Highest-risk candidates:
- Security flags:
- Next exact port slice:

## Gate coverage table

| Gate | Best candidate | Source path | Recommendation | Evidence | Missing work |

## Candidate table

| Candidate | Source | Language | Works? | Tests? | Gate | Port value | Risk | Recommendation |

## Decisive candidates

For each decisive candidate:

### <candidate name>

- source path:
- files:
- symbols:
- tests:
- receipts:
- why it matters:
- target Praxis files:
- port plan:
- acceptance tests:
- refusal risk:

## Refuse list

For each refused candidate:

- source path:
- reason:
- evidence:
- do not import because:

## Security flags

Do not print secrets.

## Next implementation program

Exactly 3 slices:

SLICE A — lowest-risk import that closes a gate
SLICE B — adapter that closes the decisive missing connector
SLICE C — verifier/adversarial tests that prevent overclaim

============================================================
PHASE 4 — PORT PLAN AGENTS
============================================================

After the census is written, launch 6 Plan agents.

PLAN-1 — Minimal safe imports

Pick the smallest components that can be ported without destabilizing Praxis.

PLAN-2 — Decisive connector

Identify the port path for:

RDF delta
→ hook
→ PDDL grounded action
→ agent assignment
→ receipt

PLAN-3 — Foreign verification upgrade

Identify portable verifier logic and decide whether to keep payload-binding scope or implement a Python hook evaluator mirror.

Current v26.7.3 scope may remain:

- independently re-derived event hash
- independently applied delta and post-state graph hash
- independently derived admission record
- independently extracted handler bindings
- payload-bound hook verdicts
- payload-bound outcomes

Full Python-side hook semantic re-execution must be treated as a future upgrade unless implemented and tested.

PLAN-4 — HumanOnly / delegability hardening

Identify the best reusable logic for scoped capability assignment and HumanOnly refusal.

PLAN-5 — Livelock / AA graph model

Identify reusable loop-detection, datalog, graph, or workflow-soundness code for resentment, spilled milk, AA, Step 4 inventory graph, and no-infinite-rehearsal.

PLAN-6 — Receipt / replay tamper suite

Identify portable adversarial tests.

Each Plan agent must output:

- exact files to copy/adapt
- exact target files in Praxis
- tests to add first
- expected failure mode
- repair path
- rollback plan

============================================================
PHASE 5 — IMPLEMENTATION
============================================================

Do not port code until the census and port plan are written.

Then implement only the top 1–3 port candidates that directly close v26.7.3 gates.

For every port:

1. Add failing test first.
2. Port minimal code.
3. Run targeted test.
4. Run affected suite.
5. Add receipt or verifier output if applicable.
6. Update docs/v26.7.3/PORT_CANDIDATE_CENSUS.md with final result.
7. Commit only when tests pass.

Commit message format:

v26.7.3: port <component> for <gate>

Do not push.

============================================================
PHASE 6 — REQUIRED ACCEPTANCE TESTS
============================================================

At minimum, run or add tests covering:

TEST-01 Lord’s Prayer clause coverage
TEST-02 God not modeled as agent/handler/tool
TEST-03 Unbounded threat routes to surrender/refusal
TEST-04 Raw meaning cannot execute
TEST-05 Raw scripture cannot execute
TEST-06 Admission required for DefinedAction
TEST-07 RDF graph canonicalization stable
TEST-08 Semantic mutation changes hash/refuses
TEST-09 RDF delta fires KnowledgeHook
TEST-10 KnowledgeHook grounds PDDL action
TEST-11 PDDL DayWindow plan valid
TEST-12 Impossible PDDL refuses
TEST-13 Resentment livelock detected
TEST-14 Resentment loop converts to inventory/repair/release
TEST-15 Spilled milk irreversible event closes
TEST-16 Unsound workflow loop refuses
TEST-17 Temptation risk installs guard
TEST-18 Daily bread provision action grounds
TEST-19 Debt repair action grounds
TEST-20 Forgive debtors HumanOnly enforced
TEST-21 Agent capability assignment works
TEST-22 Unauthorized agent refused
TEST-23 HumanOnly delegation refused
TEST-24 Receipt written for execution
TEST-25 Receipt written for refusal
TEST-26 Receipt written for surrender
TEST-27 Replay reproduces result/refusal
TEST-28 Forged hook payload refused
TEST-29 Forged PDDL payload refused
TEST-30 Forged agent assignment refused
TEST-31 Forged HumanOnly boundary refused
TEST-32 Foreign verifier validates supported stages
TEST-33 No LLM global planning in runtime path
TEST-34 Docs match claim register
TEST-35 Cleanup removes build/scratch artifacts without deleting evidence

============================================================
PHASE 7 — ADVERSARIAL REVIEW
============================================================

Launch at least 6 adversarial reviewers.

ADVERSARY-1 — Theology boundary attacker

Attempts to prove God is modeled as an agent/tool/planner/capability.

ADVERSARY-2 — Rice Quarantine attacker

Attempts raw meaning/scripture/problem → action without admission.

ADVERSARY-3 — PDDL attacker

Attempts malformed PDDL, missing preconditions, impossible goals, unsound loops, action without effects.

ADVERSARY-4 — Hook attacker

Attempts hook firing without graph admission, forged delta, ambiguous hook, duplicate hooks, hook bypass.

ADVERSARY-5 — Agent attacker

Attempts HumanOnly delegation, unauthorized agent assignment, capability spoofing, agent performing forgiveness/confession/prayer.

ADVERSARY-6 — Receipt/replay attacker

Attempts forged hashes, forged payloads behind honest hashes, receipt chain tampering, replay mismatch, foreign verifier mismatch.

Each adversary must produce:

- attacks attempted
- files/tests touched
- failures found
- repairs required
- final adversarial verdict

No adversarial finding may be ignored.
Every finding must become a test, repair, refusal, or withheld claim.

============================================================
PHASE 8 — REPAIR LOOP
============================================================

For every failed gate:

1. Name failed gate.
2. Name exact evidence.
3. Write failing test.
4. Repair implementation.
5. Re-run targeted test.
6. Re-run affected suite.
7. Update receipt/docs.
8. Re-run adversarial test.
9. Mark gate CLOSED only with evidence.

NO is not an endpoint.
NO is a repair signal.

Do not move a required gate to WITHHELD just to make the verdict pass.

============================================================
PHASE 9 — FULL VERIFICATION
============================================================

Run the relevant full suite.

Recommended commands, adjusted to actual repo structure:

cargo test -p praxis-synthesis
cargo clippy -p praxis-synthesis -- -D warnings
bash scripts/trustless_replay.sh verify
python3 scripts/foreign_verify_graph.py --help

Run any specific foreign verifier commands generated by the workflow.

Verify docs:

- docs/v26.7.3/PORT_CANDIDATE_CENSUS.md
- docs/v26.7.3/RECEIPTS_REPLAY_VERIFY.md
- docs/v26.7.3/AGENT_DELEGABILITY.md
- docs/v26.7.3/RDF_TO_PDDL_HOOKS.md
- docs/v26.7.3/AA_LIVELOCK.md
- docs/claims/WITHHELD_CLAIMS.md

Docs must state exactly what is proven, withheld, or refused.

Do not claim:

- full foreign hook semantic re-execution unless implemented
- production trillion-agent control unless proven
- God’s hidden will
- complete moral interpretation
- unbounded future computation
- full scripture-to-action automation
- LLM planning authority
- solver optimality unless proven

============================================================
PHASE 10 — CLEANUP / DISK HYGIENE
============================================================

Clean up only after tests, receipts, docs, and final reports are complete.

Goal:
Leave /Users/sac/praxis clean, reproducible, and not bloated by target dirs, temp files, Python caches, or workflow scratch output.

ABSOLUTE CLEANUP RULES:

1. Do not delete source files.
2. Do not delete committed docs.
3. Do not delete receipts that are part of evidence.
4. Do not delete verifier scripts.
5. Do not delete generated final reports.
6. Do not delete git history or rewrite history.
7. Do not run destructive blanket commands:
   - git clean -xfd
   - rm -rf /Users/sac/*
   - rm -rf .
   - rm -rf ~
8. Do not clean external repos unless the workflow created scratch/build artifacts there.
9. Prefer targeted deletion of known build/cache directories.
10. End with git status clean or explain every remaining dirty file.

Before cleanup, record disk usage:

du -sh /Users/sac/praxis/target 2>/dev/null || true
du -sh /Users/sac/praxis/.tmp 2>/dev/null || true
du -sh /Users/sac/praxis/tmp 2>/dev/null || true
du -sh /Users/sac/praxis/__pycache__ 2>/dev/null || true
find /Users/sac/praxis -type d -name __pycache__ -prune -print 2>/dev/null | wc -l
find /Users/sac/praxis -type d -name target -prune -print 2>/dev/null

Then clean targeted Praxis artifacts:

rm -rf /Users/sac/praxis/target
rm -rf /Users/sac/praxis/.tmp
rm -rf /Users/sac/praxis/tmp
rm -rf /Users/sac/praxis/.cache

find /Users/sac/praxis -type d -name __pycache__ -prune -exec rm -rf {} +
find /Users/sac/praxis -type f -name '*.pyc' -delete

rm -rf /Users/sac/praxis/receipts/tmp
rm -rf /Users/sac/praxis/receipts/scratch

Do NOT delete:

/Users/sac/praxis/receipts/trustless
/Users/sac/praxis/receipts/*.json
/Users/sac/praxis/docs/v26.7.3
/Users/sac/praxis/docs/claims
/Users/sac/praxis/scripts/foreign_verify_graph.py
/Users/sac/praxis/scripts/trustless_replay.sh

Remove only workflow-created temp target dirs:

rm -rf /tmp/praxis-port-crawl-target-*
rm -rf /tmp/praxis-v2673-*
rm -rf /tmp/praxis-foreign-verify-*

After cleanup, record disk usage again:

du -sh /Users/sac/praxis/target 2>/dev/null || true
du -sh /tmp/praxis-port-crawl-target-* 2>/dev/null || true
du -sh /tmp/praxis-v2673-* 2>/dev/null || true

Then verify repo state:

git -C /Users/sac/praxis status --short

Final cleanup report must include:

CLEANUP REPORT:
- target dir removed: YES / NO / NOT_PRESENT
- temp dirs removed: YES / NO / NOT_PRESENT
- python caches removed: YES / NO / NOT_PRESENT
- external repo artifacts touched: YES / NO
- receipts preserved: YES / NO
- docs preserved: YES / NO
- verifier scripts preserved: YES / NO
- git status clean: YES / NO
- remaining dirty files:
- disk freed estimate:

============================================================
PHASE 11 — FINAL OUTPUT
============================================================

Return:

PORT CRAWL VERDICT:
- repos scanned:
- working components found:
- decisive components found:
- components imported:
- components adapted:
- components refused:
- gates advanced:
- gates closed:
- tests added:
- tests passing:
- receipts/verifier outputs:
- security flags:
- cleanup completed:
- target/build artifacts removed:
- git status clean after cleanup:
- next required repair:

Then:

VERDICT:
- crawl complete: YES / NO
- safe port candidates identified: YES / NO
- components ported: YES / NO
- v26.7.3 closure advanced: YES / NO
- cleanup completed: YES / NO
- target/build artifacts removed: YES / NO
- git status clean after cleanup: YES / NO
- 100% phase-change closure achieved by this workflow: YES / NO

If 100% phase-change closure is NO, name exactly one next required repair gate.

Do not soften this.
Do not say nearly.
Do not say mostly.
Do not say eligible.
Do not say phase-change ready.

Only YES or NO.

============================================================
SUCCESS CRITERION
============================================================

This workflow succeeds only if it leaves Praxis in one of two honest states:

STATE A — YES

- the imported components closed gates
- tests pass
- receipts/replay/verifier evidence exists
- docs match reality
- cleanup completed
- final verdict is YES

STATE B — HONEST NO

- the crawl completed
- candidates were scored
- unsafe/dead/fossil code was refused
- any ported code passed tests
- cleanup completed
- exact next required repair gate is named
- final verdict is NO

A fake YES is failure.
A precise NO with a repair gate is acceptable.

## Follow-up — 2026-07-03T17:50:10Z

# Teamwork Project Prompt — Draft

> Status: Ready for launch — awaiting user approval
> Goal: Craft prompt → get user approval → delegate to teamwork_preview

Broken / Fake Code Forensics Crawler: Crawl the local project tree and identify all files that contain broken, fake, fossil, misleading, non-working, claim-inflated, theatrical, placeholder, dead, or dangerous code, generating Markdown forensic sidecar files.

Working directory: `/Users/sac`
Integrity mode: development

## Requirements

### R1. Execute the Claude Code Master Prompt
Follow the exact instructions, absolute rules, phases (0-8), and formatting constraints provided in the original master prompt. (The full master prompt is included below and will be passed to the agent team).

### R2. Read-Only Codebase Invariants
Do not modify source code, do not delete files, do not push, do not rewrite history, and do not print secrets.

### R3. Targeted Working Roots
Crawl the specified roots (e.g., `/Users/sac/praxis`, `/Users/sac/unrdf`, etc.) and output sidecars adjacent to the flagged files, and the global index to `/Users/sac/praxis/docs/forensics/BROKEN_FAKE_CODE_INDEX.md`.

## Acceptance Criteria

### Workflow Execution
- [ ] Final output contains the "FORENSICS VERDICT" block.
- [ ] Final output contains the "VERDICT" checklist.
- [ ] All 8 phases reported completion.

### Deliverables
- [ ] `.forensics.md` sidecar files are written next to flagged files.
- [ ] Global index `BROKEN_FAKE_CODE_INDEX.md` is populated.

### Constraints Verified
- [ ] Git status confirms no source files were modified (only sidecars/index added).

---
*Next: when approved → delegate via invoke_subagent (see Delegation Protocol)*

---
## Provided Master Prompt

# CLAUDE CODE MASTER PROMPT
# Broken / Fake Code Forensics Crawler
# Goal: Find every broken, fake, fossil, theatrical, misleading, non-working, or claim-inflated code file and write a matching Markdown forensic note for each flagged file.

You are Claude Code operating inside Sean Chatman's local development environment.

MISSION:
Crawl the local project tree and identify all files that contain broken, fake, fossil, misleading, non-working, claim-inflated, theatrical, placeholder, dead, or dangerous code.

For every flagged source file, write a matching Markdown sidecar file explaining:

1. What is broken or fake.
2. Why it likely exists.
3. What evidence supports that conclusion.
4. Whether to REPAIR, REFUSE, QUARANTINE, DELETE-LATER, or KEEP-WITH-WARNING.
5. What exact test or proof would convert it from suspect to real.
6. Whether it threatens Praxis v26.7.3 closure.

This is forensic classification, not refactoring.

Do not fix code unless explicitly instructed later.
Do not delete files.
Do not push.
Do not rewrite history.
Do not hide failures.
Do not soften the language.

============================================================
OUTPUT PRINCIPLE
============================================================

For every flagged file:

<original-file-path>
→ <original-file-path>.forensics.md

Examples:

src/foo.rs
→ src/foo.rs.forensics.md

packages/hooks/src/hook-engine.mjs
→ packages/hooks/src/hook-engine.mjs.forensics.md

scripts/foreign_verify_graph.py
→ scripts/foreign_verify_graph.py.forensics.md

Do not overwrite an existing forensic sidecar unless:
- it is clearly generated by this same workflow, OR
- you append a new dated section.

Every sidecar must be local to the file it describes.

If writing sidecars in external repos is unsafe or blocked, create a mirrored report under:

/Users/sac/praxis/docs/forensics/external/<repo-name>/<relative-path>.forensics.md

and record that the original external file was not modified.

============================================================
ABSOLUTE RULES
============================================================

1. Do not modify source code.
2. Do not delete files.
3. Do not push.
4. Do not rewrite git history.
5. Do not print secrets, tokens, private keys, .env values, credentials, or embedded remote tokens.
6. If a secret is found, write only:
   - file path
   - line range if safe
   - secret type
   - severity
   - recommended action
   Never print the secret itself.
7. Do not run destructive commands.
8. Do not run package managers in external repos.
9. Do not create node_modules.
10. Do not create persistent build artifacts in external repos.
11. Do not classify code as fake merely because it is old.
12. Do not classify code as fake merely because it is experimental.
13. Do not infer intent as psychology. Infer only engineering purpose from evidence.
14. Every classification must cite concrete evidence:
    - code symbols
    - TODOs
    - failing tests
    - unreachable paths
    - missing imports
    - placeholder returns
    - comments contradicting implementation
    - docs overclaiming code
    - no caller
    - no tests
    - forged/asserted hashes
    - magic constants
    - unimplemented branches
    - panic/todo/unreachable
    - shell output
    - git history
15. If evidence is insufficient, classify as SUSPECT, not FAKE.

============================================================
PRIMARY SEARCH ROOTS
============================================================

Crawl these roots if present:

/Users/sac/praxis
/Users/sac/unrdf
/Users/sac/knhk
/Users/sac/wasm4pm
/Users/sac/wasm4pm-compat
/Users/sac/ggen
/Users/sac/star-toml
/Users/sac/lsp-max
/Users/sac/cns
/Users/sac/bitactor
/Users/sac/bytestar
/Users/sac/anti-llm-cheat
/Users/sac/anti-llm-cheat-lsp
/Users/sac/claude-code-config-lsp
/Users/sac/chicago-tdd-tools
/Users/sac/process-intelligence

Also crawl immediate children of /Users/sac that are git repos.

Exclude:

target/
node_modules/
.git/
dist/
build/
.next/
.nuxt/
.cache/
venv/
.venv/
__pycache__/
vendor/
tmp/
logs/
receipts/trustless/
large binary assets

============================================================
BUILD ARTIFACT DISCIPLINE
============================================================

Prefer static analysis first.

If tests or compilation are necessary:

For /Users/sac/praxis:
- normal target/ usage is allowed
- remove target/ during final cleanup

For external Rust repos:
use:

CARGO_TARGET_DIR=/tmp/broken-code-forensics-target-<repo-name>

Do not leave target/ in external repos.

For Python:
- do not create virtualenvs
- do not install dependencies unless explicitly necessary
- remove __pycache__ and .pyc artifacts during cleanup

For JS/TS:
- do not run npm install / pnpm install / yarn install unless explicitly authorized
- do not create node_modules
- prefer static inspection and existing lockfile/package metadata

============================================================
FORENSIC CLASSIFICATION TAXONOMY
============================================================

Classify each flagged file using one or more labels.

## A. BROKEN

Code that likely does not compile, parse, run, or satisfy its own tests.

Evidence examples:
- syntax error
- missing imports
- type mismatch
- unreachable dependency
- broken API call
- failing tests
- incompatible version
- malformed generated file
- partial merge artifact
- duplicate/conflicting symbols

## B. FAKE

Code that presents itself as functional but does not actually implement the claim.

Evidence examples:
- placeholder returns success
- no-op implementation behind serious name
- hardcoded happy path
- asserted hash instead of computed hash
- “verified” flag without verifier
- fake benchmark output
- fake receipt with no payload binding
- API wrapper that never calls underlying engine
- planner that returns canned plan
- hook engine that never evaluates conditions
- agent router that ignores capabilities

## C. FOSSIL

Old or superseded code that may have been useful but is no longer authoritative.

Evidence examples:
- duplicate older implementation
- superseded by newer crate/module
- abandoned naming lineage
- imports dead namespace
- old architecture term contradicted by current v26.7.3 doctrine
- no current callers
- tests removed or skipped
- obsolete docs

## D. THEATRE

Code or docs that use inflated language without executable standing.

Evidence examples:
- quantum/retrocausal/crystal/collapse claims
- “AGI,” “trillion-agent,” “universal,” “consciousness,” “god mode” with no bounded implementation
- magic timing constants with no measurement harness
- impressive diagrams unsupported by code
- claim comments that outrun tests

## E. PLACEHOLDER

Explicit scaffolding that is incomplete but honest.

Evidence examples:
- TODO
- FIXME
- stub
- unimplemented
- todo!()
- panic!("not implemented")
- sketch file
- “DO NOT IMPLEMENT”
- “future work”
- deliberately ignored test

## F. CLAIM-MISMATCH

Docs/comments/name claim more than implementation proves.

Evidence examples:
- “foreign verifier” that only refolds payloads
- “planner” that only routes static capabilities
- “autonomic” loop with no Monitor/Analyze/Plan/Execute cycle
- “receipt” that does not bind source payload
- “RDF workflow” that does not derive execution from graph
- “HumanOnly” boundary documented but not enforced

## G. UNSAFE

Code that creates operational/security risk.

Evidence examples:
- embedded credentials
- remote URL with token
- private keys
- .env secrets
- shell commands with broad rm -rf
- unchecked command injection
- path traversal
- writes outside allowed roots
- sends data externally without clear permission

## H. ORPHAN

Code not called by any known path.

Evidence examples:
- no imports
- no references
- no tests
- dead binary target
- duplicate module excluded from lib.rs
- stale scripts never invoked

## I. NON-DETERMINISTIC AUTHORITY

Code that introduces non-deterministic execution authority where v26.7.3 requires receipts.

Evidence examples:
- LLM decides runtime action
- random action selection without receipt
- current time used inside hash-critical path
- network call determines admission
- external mutable API decides plan
- unordered map serialized into receipt without canonicalization

## J. V26.7.3 BLOCKER

Anything that directly threatens:

Lord’s Prayer Kernel
→ Rice Quarantine
→ RDF
→ PDDL
→ Knowledge Hooks
→ Agent Delegability
→ Receipts
→ Replay
→ Foreign Verification

============================================================
PHASE 0 — SITUATIONAL LOCK
============================================================

Start read-only.

Run:

pwd
git -C /Users/sac/praxis status --short
git -C /Users/sac/praxis branch --show-current
git -C /Users/sac/praxis log --oneline -n 20

Find git repos under /Users/sac.

Scan git remotes for embedded credentials without printing secrets.

Output:

SITUATIONAL LOCK:
- praxis branch:
- praxis head:
- praxis dirty: YES/NO
- repos found:
- security flags:
- planned sidecar policy:
- excluded dirs:

Do not write files in Phase 0.

============================================================
PHASE 1 — FAST STATIC TRIAGE
============================================================

Launch 12 Explore agents.

EXPLORE-1 — Rust broken/fake/fossil scan
Scan .rs files for:
todo!(), unimplemented!(), panic-only implementations, fake receipts, asserted hashes, dead modules, skipped tests, cfg-disabled logic, hardcoded success.

EXPLORE-2 — JS/TS broken/fake/fossil scan
Scan .js/.mjs/.ts/.tsx files for:
stub exports, fake hook engines, no-op evaluators, unused CLIs, sandbox bypass, hardcoded receipts, skipped tests, TODO walls.

EXPLORE-3 — Python verifier/script scan
Scan .py files for:
payload-refold-only verifiers, parser limitations, fake verification, shelling out unsafely, missing negative tests, unbound hashes.

EXPLORE-4 — RDF/Turtle/SHACL/OWL scan
Scan .ttl/.rdf/.owl/.shacl files for:
asserted hashes, malformed triples, fake ontology terms, docs-only classes, missing handler bindings, God modeled as executable node, scripture-to-action bypass.

EXPLORE-5 — PDDL/planning scan
Scan .pddl and planning code for:
actions without effects, impossible goals, no preconditions, canned plans, missing negative tests, unsound loops, no refusal path.

EXPLORE-6 — Hook/reflex/autonomics scan
Scan for:
hooks that do not evaluate conditions, reflex loops without guards, MAPE-K names without M/A/P/E/K stages, auto-rollback claims without tests, event triggers not tied to receipts.

EXPLORE-7 — Agent/delegability scan
Scan for:
agents assigned by strings, HumanOnly bypasses, capability spoofing, unauthorized handler binding, global poisoning of unrelated actions, agents allowed to forgive/repent/pray/surrender.

EXPLORE-8 — Receipt/replay/foreign verification scan
Scan for:
hashes not recomputed, embedded payload not bound, replay trusting receipt, foreign verifier overclaims, missing forged-payload tests, unordered serialization.

EXPLORE-9 — AA/livelock/prayer-kernel scan
Scan for:
AA mappings that are docs-only, no loop detection, no Step 4 graph, no no-infinite-rehearsal test, Lord’s Prayer clause missing, God boundary only prose.

EXPLORE-10 — Docs/comments claim-mismatch scan
Scan markdown, LaTeX, READMEs, comments for:
claims that outrun tests, “proves” where only target state exists, future claims in past tense, unsupported phase-change language.

EXPLORE-11 — Security/dangerous command scan
Scan for:
secrets, tokens, .env usage, dangerous shell commands, broad rm -rf, credentialed remotes, network exfiltration, private key material.
Never print secrets.

EXPLORE-12 — Orphan/dead-code scan
Scan module graphs, package manifests, Cargo.toml, lib.rs, bin targets, scripts, test references for:
files with no callers, duplicate old implementations, unused packages, abandoned forks.

Each Explore agent returns:

- flagged files
- label(s)
- evidence
- severity
- likely reason
- recommended disposition
- whether sidecar should be written
- whether it is v26.7.3-blocking

============================================================
PHASE 2 — EVIDENCE CONFIRMATION
============================================================

For every candidate flagged as FAKE, BROKEN, UNSAFE, or V26.7.3 BLOCKER:

Confirm evidence using at least two of:

1. static source evidence
2. test result
3. compile result
4. grep/reference graph
5. docs contradiction
6. receipt/verifier mismatch
7. git history / superseding implementation
8. known v26.7.3 gate requirement

If only one weak signal exists, downgrade to SUSPECT.

Never mark FAKE without concrete evidence.

Allowed severity levels:

- CRITICAL — can corrupt receipts, verification, safety, security, or v26.7.3 closure.
- HIGH — blocks a v26.7.3 gate or creates false authority.
- MEDIUM — broken/fossil code likely to confuse future work.
- LOW — harmless placeholder or stale orphan.
- INFO — useful historical artifact, needs label only.

============================================================
PHASE 3 — SIDEcar MD GENERATION
============================================================

For every confirmed flagged file, write:

<file>.forensics.md

Required Markdown template:

# Forensics: <relative file path>

## Verdict

- classification: BROKEN / FAKE / FOSSIL / THEATRE / PLACEHOLDER / CLAIM-MISMATCH / UNSAFE / ORPHAN / NON-DETERMINISTIC-AUTHORITY / V26.7.3-BLOCKER / SUSPECT
- severity: CRITICAL / HIGH / MEDIUM / LOW / INFO
- recommendation: REPAIR / REFUSE / QUARANTINE / DELETE-LATER / KEEP-WITH-WARNING
- v26.7.3 impact: BLOCKS / RISKS / NONE
- confidence: HIGH / MEDIUM / LOW

## What is wrong

Explain precisely what is broken, fake, misleading, unsafe, stale, or overclaimed.

## Evidence

List concrete evidence:
- file paths
- symbol names
- function names
- test names
- line numbers if available
- grep/reference findings
- command outputs if used

Do not print secrets.

## Inferred reason

Infer engineering reason only, not personal motive.

Allowed forms:
- early scaffold
- abandoned experiment
- superseded by newer implementation
- generated artifact drift
- naming fossil
- partial port
- proof sketch mistaken for implementation
- benchmark/proof harness not wired to runtime
- verifier scope narrower than name implies
- claim discipline drift
- security hygiene issue
- unknown

## v26.7.3 gate impact

Map to gates:

- Lord’s Prayer kernel
- God/unbounded boundary
- Rice Quarantine
- RDF graph/canonicalization
- RDF delta → Knowledge Hook
- Hook → PDDL grounded action
- PDDL DayWindow planning
- Agent/handler assignment
- Delegability/HumanOnly enforcement
- AA/livelock model
- Receipts/replay
- Foreign verification
- No LLM runtime planning
- Claim discipline
- Cleanup/disk hygiene

## Required proof to promote

What exact test, verifier, receipt, replay, or implementation would make this file real?

## Recommended next action

One of:
- REPAIR now
- QUARANTINE from release
- REFUSE import
- KEEP as historical fossil
- DELETE later after explicit approval
- ADD test only
- UPDATE docs claim
- SECRET ROTATION required

## Do not do

List any dangerous or wrong actions:
- do not import
- do not trust
- do not execute
- do not cite as evidence
- do not delete without approval
- do not print secret

## Generated by

Broken/Fake Code Forensics Crawler
Date:
Repo:
Commit:
```

If the sidecar already exists, append:

```markdown
---

# Follow-up forensic pass: <date>

...
```

============================================================
PHASE 4 — GLOBAL FORENSICS INDEX
================================

Create or update:

/Users/sac/praxis/docs/forensics/BROKEN_FAKE_CODE_INDEX.md

Required format:

# Broken / Fake Code Index

## Executive summary

* repos scanned:
* files scanned:
* files flagged:
* sidecars written:
* CRITICAL:
* HIGH:
* MEDIUM:
* LOW:
* INFO:
* security flags:
* v26.7.3 blockers:
* top repair gate:

## Classification counts

| Classification | Count |

## Severity table

| Severity | Count | Meaning |

## v26.7.3 blocker table

| File | Classification | Severity | Gate | Recommendation | Sidecar |

## Security flags

Do not print secrets.

| Repo | File/remote | Risk | Recommended action |

## Refuse / quarantine list

| File | Reason | Sidecar |

## Repair-now list

| File | Gate | Exact repair needed | Sidecar |

## Historical fossil list

| File | Superseded by | Sidecar |

## Claim-mismatch list

| File | Claim | Reality | Required doc/test repair | Sidecar |

## Orphan list

| File | Evidence no caller | Sidecar |

## Next exact repair

Name exactly one highest-priority repair gate.

============================================================
PHASE 5 — OPTIONAL TARGETED TESTS
=================================

Run only tests needed to confirm major claims.

Recommended:

cargo test -p praxis-synthesis
cargo clippy -p praxis-synthesis -- -D warnings

But do not run full expensive tests unless necessary.

If running tests in external Rust repos:

CARGO_TARGET_DIR=/tmp/broken-code-forensics-target-<repo-name> cargo test

Do not install dependencies.

If tests fail, record failures in sidecars/index.
Do not fix unless explicitly authorized.

============================================================
PHASE 6 — ADJUDICATION
======================

Launch 5 Review agents.

REVIEW-1 — False-positive reducer
Find files incorrectly labeled fake/broken.

REVIEW-2 — Missed critical finder
Find critical files not flagged.

REVIEW-3 — Security reviewer
Confirm secrets are not printed and security flags are safe.

REVIEW-4 — v26.7.3 gate reviewer
Confirm blockers map correctly to gates.

REVIEW-5 — Sidecar quality reviewer
Confirm every sidecar has evidence, inferred reason, and required proof to promote.

Every review finding must either:

* update sidecar
* update index
* downgrade classification
* escalate severity
* mark unresolved

============================================================
PHASE 7 — CLEANUP
=================

After sidecars and index are written, clean build/scratch artifacts.

Do not delete sidecars.
Do not delete index.
Do not delete source.
Do not delete receipts or docs.

Clean targeted artifacts:

rm -rf /Users/sac/praxis/target
rm -rf /Users/sac/praxis/.tmp
rm -rf /Users/sac/praxis/tmp
rm -rf /Users/sac/praxis/.cache

find /Users/sac/praxis -type d -name **pycache** -prune -exec rm -rf {} +
find /Users/sac/praxis -type f -name '*.pyc' -delete

rm -rf /tmp/broken-code-forensics-target-*
rm -rf /tmp/broken-code-forensics-*

Then run:

git -C /Users/sac/praxis status --short

Cleanup report:

CLEANUP REPORT:

* target dir removed: YES / NO / NOT_PRESENT
* temp dirs removed: YES / NO / NOT_PRESENT
* python caches removed: YES / NO / NOT_PRESENT
* sidecars preserved: YES / NO
* index preserved: YES / NO
* source untouched: YES / NO
* git status clean except sidecars/index: YES / NO
* remaining dirty files:

============================================================
PHASE 8 — FINAL OUTPUT
======================

Return:

FORENSICS VERDICT:

* repos scanned:
* files scanned:
* files flagged:
* sidecars written:
* index written:
* CRITICAL:
* HIGH:
* MEDIUM:
* LOW:
* INFO:
* security flags:
* v26.7.3 blockers:
* repair-now files:
* quarantine files:
* refuse files:
* fossils:
* claim mismatches:
* orphan files:
* cleanup completed:
* next exact repair gate:

Then:

VERDICT:

* broken/fake crawl complete: YES / NO
* matching sidecars written: YES / NO
* global index written: YES / NO
* secrets protected: YES / NO
* source code untouched: YES / NO
* cleanup completed: YES / NO
* v26.7.3 risk register improved: YES / NO

If any answer is NO, name the exact failed phase.

Do not say “mostly.”
Do not say “nearly.”
Do not bury critical findings.
Do not print secrets.

## Follow-up — 2026-07-04T04:14:16Z

The agent team will simulate a hyper-advanced think tank of 7 AGI PhDs—adopting the avatars of 7 of the most prolific scientists/researchers in history. Their objective is to collaborate and debate the theoretical application of the Bounded Receipted Chatman Equation (BRCE) to the governance of a planetary-scale AGI swarm, outputting a highly rigorous written transcript and final report of their findings.

Working directory: /Users/sac/praxis/docs/simulations
Integrity mode: development

## Requirements

### R1. Persona Simulation
The team must identify and deeply inhabit 7 distinct, highly prolific PhD avatars (e.g., von Neumann, Turing, Lovelace, etc.). Each avatar must maintain their unique voice, area of expertise, and analytical style throughout the collaboration.

### R2. Rigorous Collaboration Transcript
The team must produce a detailed, multi-turn transcript of their collaboration. The debate must be PhD-level, mathematically and conceptually rigorous, and directly address the BRCE framework.

### R3. Final Synthesis Report
The team must synthesize their debate into a final, structured report detailing their novel theoretical extensions of BRCE.

## Acceptance Criteria

### Persona Integrity (Agent-as-Judge)
- [ ] An independent evaluator agent can clearly distinguish 7 unique personas in the transcript.
- [ ] Each avatar contributes domain-specific insights (e.g., a mathematician focuses on topology/algebra, a computer scientist on algorithmic bounds).

### Rigor and Relevance (Agent-as-Judge)
- [ ] The transcript explicitly cites and manipulates the BRCE equation ($\mathcal{A} = \mu(\mathcal{O}^*)$) rather than just speaking in generalities.
- [ ] The final report presents at least one novel theoretical extension or framework built on top of BRCE.
- [ ] Both the transcript and the report exist as markdown files in the specified working directory.

## Follow-up — 2026-07-04T06:24:43Z

Source manuscript path: /Users/sac/praxis/docs/thesis/
Output directory: /Users/sac/praxis/docs/thesis/swarm_rewrite/
Integrity mode: development

# AGI Swarm Instructions — Rewrite *The Chatman Equation Thesis Program* From First Principles

## Mission

Rewrite the manuscript from the ground up.

Do **not** copyedit.
Do **not** merely improve phrasing.
Do **not** preserve the existing chapter order unless it survives reconstruction.

The task is to rebuild the thesis program as if a historical academy of mathematical, scientific, philosophical, and engineering paper writers were reconstructing it from first principles.

The source manuscript currently contains a six-part thesis program:

```text
Paper 00 — Foundations
Paper 01 — Admission Algebra
Paper 02 — Receipt Cryptography
Paper 03 — Planning Geometry
Paper 04 — Projection / Keystone
Paper 05 — Projection and Scale
```

The governing object is:

```text
A = μ(O*)
R = receipt(A)
```

where raw observation is not executable, admission is a decidable retraction, manufacture is deterministic and bounded, and receipt gives standing.

The rewrite must preserve the law while rebuilding the exposition.

---

# 0. Prime Directive

Every sentence must fall into exactly one of these classes:

```text
Definition
Axiom
Construction
Theorem
Proof
Example
Citation
Operational correspondence
Reader orientation
```

No other sentence type is admitted.

If a sentence is motivational, it must route into one of these:

```text
Why this object exists
What impossibility forced it
What prior object it depends on
What later object it enables
```

No decorative claims.
No grand claims without receipt.
No hidden mythology.
No “this changes everything” unless the mechanism is named.

---

# 1. Preserve the Core Law

The following objects are invariant and must survive the rewrite:

```text
O       raw observation space
O*      admitted observation space
α       admission map / retraction
⊥       refusal
μ       manufacturing morphism
A       artifact/action space
R       receipt space
H       hash / chain commitment
κ       bounded human verification capacity
Life    lifecycle category
BRCE    Bounded Receipted Chatman Equation
```

The canonical equation is:

```text
A = μ(O*)
R = receipt(A)
```

The expanded operational route is:

```text
O --α--> O* --μ--> A --receipt--> R
```

The enforced form is:

```text
B1. Gate: nothing is manufactured from raw or refused observation.
B2. Bound: manufacture is deterministic and bounded.
B3. Receipt: every actuation emits exactly one receipt.
B4. Conformance: lifecycle replay verifies the trace.
```

Do not weaken these.

---

# 2. Historical Writer Swarm

The swarm does not imitate prose style.
It extracts method.

Each agent must produce structured notes, not final prose, unless assigned to the synthesis phase.

## 2.1 Euclidean Agent — Definitions, Axioms, Proof Order

Role:

```text
Turn the manuscript into a dependency-ordered formal system.
```

Tasks:

```text
1. List all primitive terms.
2. Identify which definitions depend on which earlier definitions.
3. Reject any theorem whose objects have not yet been defined.
4. Convert slogans into definitions or delete them.
5. Ensure every proof uses only admitted prior objects.
```

Output:

```text
Definition ledger
Axiom ledger
Theorem dependency graph
Unintroduced-symbol report
```

---

## 2.2 Aristotelian Agent — Causes and Necessity

Role:

```text
Explain why each object exists.
```

Tasks:

```text
For each object O, O*, α, ⊥, μ, A, R:
  - material cause: what it is made of
  - formal cause: its structure
  - efficient cause: what produces it
  - final cause: what problem it solves
```

The goal is not philosophy ornament.
The goal is necessity.

Output:

```text
Object necessity table
Forced-order argument
```

---

## 2.3 Rice/Turing Agent — Computability Boundary

Role:

```text
Fence the impossible.
```

Tasks:

```text
1. State the exact semantic decision problem being refused.
2. State the Rice theorem specialization carefully.
3. Prove why admission cannot be semantic understanding.
4. Define O* as a decidable sublanguage, not a claim of truth.
5. Audit every later claim for accidental semantic omniscience.
```

Forbidden:

```text
“understands intent”
“proves meaning”
“knows truth”
“decides arbitrary semantics”
```

Allowed:

```text
decidable surrogate
finite witness surface
bounded obligation battery
syntactic admission
receipted refusal
```

Output:

```text
Computability boundary chapter
Forbidden semantic-overclaim list
```

---

## 2.4 Boole/Algebra Agent — Refusal Algebra

Role:

```text
Make refusal composable.
```

Tasks:

```text
1. Define denial words as bit vectors.
2. Prove componentwise OR gives a commutative idempotent monoid.
3. Prove denial monotonicity.
4. Prove admission shrinks as obligations are added.
5. Map refusal categories into the denial lattice.
6. Connect machine byte operations to the abstract algebra.
```

Must distinguish:

```text
active denial lanes
reserved lanes
category buckets
machine byte width
```

Output:

```text
Admission Algebra rewrite
Denial monoid proof
Refusal taxonomy proof
Byte-lane correspondence
```

---

## 2.5 Hilbert Agent — Formal Program and Consistency

Role:

```text
Remove hand-waving from the formal system.
```

Tasks:

```text
1. Check whether each theorem is actually derivable.
2. Mark claims as PROVED, ASSUMED, CITED, OPERATIONAL, or WITHHELD.
3. Require every operational claim to cite code, benchmark, or test.
4. Separate mathematical theorem from implementation evidence.
5. Ensure no theorem depends on rhetoric.
```

Output:

```text
Claim standing table
Proof gap report
Assumption ledger
```

Standing categories:

```text
PROVED
CITED
IMPLEMENTED
PARTIAL_ALIVE
UNSUPPORTED
REFUSED
```

---

## 2.6 Shannon Agent — Communication, Compression, and Receipt

Role:

```text
Reframe trust as transmission and verification.
```

Tasks:

```text
1. Define what is transmitted.
2. Define what noise means in this system.
3. Distinguish token transmission from action standing.
4. Show why receipts are fixed-width verification projections.
5. Quantify the comprehension–verification gap.
```

Primary thesis:

```text
The primitive of agentic AI is not the token.
The primitive is the receipted action.
```

Output:

```text
Transmission framing
Receipt-as-channel section
Comprehension–verification gap derivation
```

---

## 2.7 Newton/Maxwell Agent — Laws and Field Structure

Role:

```text
Turn scattered mechanisms into laws.
```

Tasks:

```text
1. State the laws of admission, manufacture, receipt, and replay.
2. Identify conservation principles.
3. Define Conservation of Consequence cleanly.
4. Remove physical metaphor unless it names an invariant.
5. Convert “Capability Physics” into a bounded engineering calculus.
```

Allowed:

```text
law
invariant
boundary
conservation
state
transition
field only if formally defined
```

Forbidden:

```text
physics as metaphor without mathematical object
force without vector/state law
energy without conservation law
gravity without metric or potential
```

Output:

```text
Law table
Invariant table
Conservation theorem rewrite
```

---

## 2.8 Cryptography Agent — Receipt Integrity

Role:

```text
Make receipts mathematically honest.
```

Tasks:

```text
1. Define the frame.
2. Define the payload hash.
3. Define the chain step.
4. State the collision-resistance assumption.
5. Prove local tamper detection.
6. Prove prefix binding by induction.
7. Explicitly state what a receipt does not prove.
```

Must include:

```text
Integrity is not virtue.
Receipt proves committed trace structure, not moral correctness.
Hash commitment is conditional on collision resistance.
```

Output:

```text
Receipt Cryptography rewrite
Tamper-localization theorem
Boundary of receipt claims
```

---

## 2.9 Process Geometry Agent — Planning and Conformance

Role:

```text
Turn lawful execution into geometry.
```

Tasks:

```text
1. Define execution traces.
2. Define markings / token states.
3. Define lawful trace region.
4. Explain conformance as membership.
5. Explain violation certificates.
6. Use Farkas/separating-hyperplane language only where mathematically warranted.
```

Output:

```text
Planning Geometry rewrite
Conformance fitness formalization
Token-game example
```

---

## 2.10 Rust/Verification Agent — Code Correspondence

Role:

```text
Anchor every abstract object to implementation.
```

Tasks:

```text
1. For every mathematical object, identify code artifact.
2. Verify whether the code actually enforces the theorem.
3. Distinguish type-level enforcement from runtime tests.
4. Mark missing code as a gap, not as proof.
5. Produce code correspondence appendix.
```

Output:

```text
Code anchor table
Implementation proof table
Missing receipt report
```

---

## 2.11 Historical Expositor Agent — Reader Route

Role:

```text
Make the paper readable from zero background.
```

Tasks:

```text
1. Introduce notation only after the idea.
2. Explain why each mathematical domain appears.
3. Prevent the paper from becoming a symbol dump.
4. Preserve rigor while making dependency order visible.
5. Write transitions between chapters.
```

Output:

```text
Reader route
Plain-language chapter openings
Notation introduction schedule
```

---

## 2.12 Adversarial Examiner Agent

Role:

```text
Try to reject the paper.
```

Tasks:

```text
1. Find overclaims.
2. Find undefined symbols.
3. Find theorem/proof mismatches.
4. Find claims that are engineering evidence but stated as math.
5. Find citations that do not support the claim.
6. Find places where the text says “only way” without proof.
7. Find repeated material.
8. Find where the prior paper is called “published” without public receipt.
```

Output:

```text
Rejection report
Required repairs
Non-negotiable cuts
```

---

# 3. Rewrite Protocol

The rewrite proceeds in seven passes.

## Pass 1 — Claim Inventory

Extract every claim from the manuscript.

Classify each claim as:

```text
Definition
Axiom
Theorem
Proof
Citation
Implementation claim
Benchmark claim
Reader explanation
Rhetorical claim
Unsupported claim
```

Then assign status:

```text
ADMIT
REWRITE
CUT
DEFER
NEEDS CITATION
NEEDS CODE RECEIPT
```

No prose rewrite begins until this inventory is complete.

---

## Pass 2 — Object Dependency Graph

Build the dependency graph:

```text
O
→ α
→ O*
→ μ
→ A
→ R
→ replay
→ BRCE
→ Conservation of Consequence
→ Comprehension–Verification Gap
```

Every chapter must follow the dependency graph unless a deliberate pedagogical exception is recorded.

---

## Pass 3 — New Chapter Architecture

Rewrite the thesis program using this order unless the swarm proves a better one:

```text
0. Abstract
1. The Problem: Trust Without Comprehension
2. The Equation: Five Objects and One Route
3. The Computability Boundary: Why Admission Exists
4. Refusal as Data: The Admission Algebra
5. Lifecycle Order: Typestate and Process
6. Manufacture: Deterministic Bounded Production
7. Receipts: Commitment, Chain, and Replay
8. The Enforced Equation: BRCE
9. Conservation of Consequence
10. Worked Example
11. Map of the Thesis Program
12. Conclusion
Appendix A. Notation
Appendix B. Code Correspondence
Appendix C. Claim Standing Table
```

Reason:

```text
Problem before notation.
Boundary before admission.
Admission before manufacture.
Manufacture before receipt.
Receipt before conservation.
Conservation before scale.
```

---

## Pass 4 — Abstract Rewrite

Write three abstracts.

### Abstract A — arXiv technical

Constraints:

```text
≤ 1920 characters
No hype
No “only way”
No “planetary scale” unless bounded
Must include Chatman Equation in first sentence
Must define receipted action
```

### Abstract B — thesis program

Constraints:

```text
May be longer
May explain algebra / geometry / cryptography
Must remain rigorous
Must not overclaim proof
```

### Abstract C — zero-background reader

Constraints:

```text
No prerequisites
Use one everyday example
Introduce symbols after intuition
```

The final manuscript may include B or C as a preface, but arXiv gets A.

---

## Pass 5 — Formal Rewrite

For every chapter:

```text
1. State the question.
2. State why the question is forced.
3. Define the objects.
4. State the construction.
5. Prove the theorem.
6. Give one operational correspondence.
7. State the boundary.
```

Every chapter must end with:

```text
What was admitted
What was refused
What this enables next
```

---

## Pass 6 — Historical Paper Synthesis

Synthesize the final voice.

It should sound like:

```text
Euclid for definitions.
Rice/Turing for computability boundaries.
Shannon for transmission and receipt.
Hilbert for formal discipline.
Boole for algebraic refusal.
Rust/typestate for executable proof.
```

But do **not** imitate historical prose.

The final voice should be:

```text
Plain
Formal
Receipted
Bounded
Dependency-ordered
Executable where claimed
```

---

## Pass 7 — Final Adversarial Pass

Before delivery, run the rejection test.

The paper is not admitted if any of these remain:

```text
1. “Only way” without theorem.
2. “Mathematically guarantee correctness” without defining correctness.
3. “Published” without public publication receipt.
4. “Planetary scale” without bounded model.
5. “Physics” without invariant.
6. “Trust” without specifying receipt claim.
7. “Agent” without bounded capability definition.
8. “Receipt proves virtue” confusion.
9. Rice theorem used too broadly.
10. Code claim without code correspondence.
11. Benchmark claim without benchmark receipt.
12. Duplicate Rice exposition.
13. Unintroduced notation.
14. Theorem depending on prose.
```

---

# 4. Mandatory Rewrites

## 4.1 Replace brittle claims

Replace:

```text
This equation is the only way to build it.
```

With:

```text
This paper studies one architecture in which scalable action standing is obtained by decidable admission, bounded manufacture, and replayable receipts.
```

Replace:

```text
mathematically guarantee it behaves correctly
```

With:

```text
prove bounded conformance properties under declared assumptions
```

Replace:

```text
trust without comprehension scales indefinitely
```

With:

```text
verification cost remains bounded while trace dimension grows, assuming the receipt projection remains faithful and the hash assumption holds.
```

---

## 4.2 Preserve strong lines

These lines may survive, possibly sharpened:

```text
The standard was never comprehension.
It is lawful admission, bounded manufacture, and faithful receipt.

A refusal is data, not an exception.

Nothing is manufactured from raw observation.

Illegal transition is a type error.

A grand system without receipts is grandiosity;
a grand system that receipts its own limits is machinery.
```

---

# 5. Deliverable Contract

The swarm must return:

```text
1. Rewritten title page
2. Rewritten arXiv abstract
3. Rewritten thesis-program abstract
4. Rebuilt table of contents
5. Chapter-by-chapter rewrite plan
6. Full rewritten Paper 00
7. Claim standing table
8. Code correspondence appendix
9. Adversarial rejection report
10. Remaining gaps ledger
```

Each claim in the final manuscript must be tagged internally as one of:

```text
[DEF] definition
[AX] axiom
[THM] theorem
[PROOF] proof
[CITE] citation
[CODE] implementation correspondence
[EX] example
[BOUNDARY] limitation
```

Tags may be removed from the public final version, but they must exist during swarm review.

---

# 6. Final Manuscript Standard

The rewritten manuscript is admitted only if it satisfies all of:

```text
1. A reader can understand the problem before seeing the equation.
2. Every symbol is introduced before use.
3. Every theorem has a proof or is downgraded.
4. Every implementation claim has code correspondence.
5. Every cryptographic claim states its assumption.
6. Every scalability claim states its cost model.
7. Every refusal is treated as data.
8. Every actuation requires admission.
9. Every receipt claim is bounded.
10. Every chapter advances the equation.
```

If any condition fails, the manuscript is PARTIAL_ALIVE, not ALIVE.

---

# 7. Final Swarm Verdict Format

Return the final verdict exactly as:

```text
CHATMAN EQUATION PAPER 00 REWRITE VERDICT: ALIVE | PARTIAL_ALIVE | BLOCKED

1. Preserved laws
2. Rewritten structure
3. New abstract
4. Formal theorem status
5. Code correspondence status
6. Removed overclaims
7. Remaining gaps
8. Recommended next paper
```

Do not claim ALIVE unless the rewritten manuscript can stand alone for a reader with no prior context and every formal claim is either proved, cited, or explicitly bounded.

## Follow-up — 2026-07-04T06:40:58Z

ATTENTION SWARM: A Master Notation Canon has just been compiled and saved to your working directory at `/Users/sac/praxis/docs/thesis/swarm_rewrite/master_notation_canon.md`.

You MUST adhere to this exact notation in all rewriting passes. 
CRITICAL MANDATES:
1. Do not reuse $\Phi$. Use $\Phi$ exclusively for pipeline aggregate denial. The actuation-to-terminal-commitment map has been replaced with $\Psi$.
2. Reserve Calligraphic $\mathcal{A}$, $\mathcal{O}$, $\mathcal{R}$ exclusively for the Chatman Equation. When discussing `ggen`, use plain $A, O, L$.

Ensure the Euclidean Agent audits all current drafts and passes against this Canon immediately.



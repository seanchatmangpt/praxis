# DEFINITION_OF_DONE — v26.7.10-revised: No-LLM Multi-Actor Planning + Multi-Engine Execution

Version: v26.7.10-revised (PROJ-730). Doctrine document pointed to by `RELEASE_CONTROL.md`;
if this document and `RELEASE_CONTROL.md` disagree, `RELEASE_CONTROL.md` wins. Every clause
carries a status from the no-overclaiming vocabulary (ALIVE / PARTIAL / MOCKED / UNVERIFIED /
BLOCKED / REFUSED / PLANNED). All new clauses start UNVERIFIED or PLANNED — nothing below is
asserted until `RELEASE_CONTROL.md` cites a command + output for it.

## Quick reference

1. Interim milestone (superseded prior DoD)
2. Governing claim (no-LLM pipeline sentence)
3. TWOSTEP-replacement table
4. Formal planning target and decomposition D(P) with proof obligations
5. No-LLM decomposition mechanism per dialect
6. Bounded decomposition algorithm (15 steps)
7. Single-actor route — typed results, never fallback
8. Potato canonical scenario
9. Recursive generalization 8¹ → 8² → 8³
10. Multi-engine topology (C + H + M)
11. Authority partition table
12. 16-state cross-engine dispatch machine
13. Distributed broker law
14. Isolation falsifiers
15. Gall checkpoints G0–G16
16. Required result markers
17. Anti-hardcoding requirements
18. Negative proof corpus (§13 corpus)
19. Paper-equivalent evaluation corpus
20. Honest boundaries

## 1. Interim milestone (superseded)

The prior v26.7.10 DoD was closed at commit `31c236f` with all 11 success markers derived
TRUE via SPARQL (`RELEASE_CONTROL.md` §8, `DOD_SIGNOFF.md`). That closure is not discarded:
it is recorded as an **interim milestone** and its full text is preserved at
`DEFINITION_OF_DONE_INTERIM.md` (fix-forward: moved, not deleted). The single-process
autonomic loop, loopback-real dispatch, 13-state machine, hook actuation, receipts, replay,
and marker machinery proven there are the substrate this revised DoD builds on.

Status: ALIVE as an interim record (evidence in `RELEASE_CONTROL.md` §8; nothing here
re-verified this session). The revised clauses below supersede it in place; markers keep the
`V26_7_10_PRODUCTION_READY` name with the revised meaning defined in §16.

## 2. Governing claim

The engine derives goal decompositions from admitted graph state — no LLM call, no English
subgoal, no human-authored decomposition — manufactures helper/main PDDL subproblems via
SPARQL CONSTRUCT, plans them classically, derives the exact interface state
`s′ = E(s_i, π_h)`, proves non-interference and resource-release closure, composes a POWL
partial order, selects deterministically with the single-actor plan as an explicit typed
candidate, and executes the selected decomposition across independent Chatman Engine OS
processes with per-engine identity, receipts, and replay. `LLM_CALLS=0` over the whole run.

Status: ALIVE (constituent mechanisms, PROJ-701..713/720..729) / ALIVE (decompose-to-dispatch
bridge, mechanism — PROJ-749, second synthesis round: `dispatch_subworkflow_to_engine`/
`collect_subworkflow_consequence` in `decomp/dispatch_bridge.rs` stitch a real `decompose()`
output into a real cross-engine `cng engine serve` run for the first time this milestone,
`cng_decompose_to_dispatch_integration.rs` 2/2 passed) / ALIVE (payload-carrying dispatch,
moonshot round: PROJ-710 -> PROJ-723 is now closed — a dispatched contract carries its
subworkflow's actual `(domain_pddl, problem_pddl)` text as two sibling files written
atomically into the target engine's real inbox, with `disp:inputArtifactSet` now holding a
real content digest (`payload_digest`, length-prefixed BLAKE3 fold) instead of a synthetic
label; the receiving `engine.rs::run_serve_loop` recomputes and verifies that digest
(`CNG_R11 AuditMismatch` on divergence) before parsing/grounding/planning the SPECIFIC
dispatched plan via `bcinr_pddl`/`pddl_index`, falling through unchanged to the prior
synthetic path when no payload is present. `dispatched_subworkflow_payload_is_the_content_the_
engine_actually_executes` (`cng_decompose_to_dispatch_integration.rs`) proves byte-identity
between what two independently-dispatched engines manufactured and what was sent, that the two
engines' manufactured content genuinely differs, and that the synthetic `"email-routing"` path
never fired) / PARTIAL (global-goal closure across two engines' outputs — reconstructing that
the combined outcome of two independently-dispatched subworkflows satisfies the ORIGINAL
undecomposed problem's goal — is still not provable by any machinery on disk today; this is a
narrower remaining gap than the payload-fidelity question PROJ-710 -> PROJ-723 named, which is
now closed). See `DOD_SIGNOFF.md` §2 for the full clause-level reconciliation.

## 3. TWOSTEP-replacement table

TWOSTEP-style multi-actor planning uses an LLM to propose the helper subgoal. This release
replaces every LLM-shaped step with a graph-lawful mechanism:

| TWOSTEP step (LLM) | Replacement here | Status |
|---|---|---|
| LLM proposes helper subgoal | Datalog edges + bounded canonical search | PLANNED (704/705) |
| LLM writes subproblem text | CONSTRUCT graphs + deterministic PDDL renderer | PLANNED (703/706) |
| Interface state guessed | `s′` replayed with per-step precondition checks | PLANNED (707) |
| Interference informal | Machine-checked non-interference + release closure | PLANNED (708) |
| Silent single-actor fallback | Typed `DecompositionOutcome`, receipted | PLANNED (710) |
| Rejected splits unaudited | Every candidate receipted, accepted and rejected | PLANNED (710) |
| ≥ 1 LLM call | `LLM_CALLS=0`, dependency audit + absence marker | PLANNED (727) |

## 4. Formal planning target and decomposition

The planning target is `P = ⟨S, s_i, S_g, A, O, P, T⟩`: state space, initial state, goal
states, actions, objects, predicates, and types, lifted from admitted PDDL into
`pddl-strips.ttl` triples (PROJ-701/702). A decomposition is:

```text
D(P) = ⟨P_h, P_m, s′⟩   where   s′ = E(s_i, π_h)
```

`P_h` is the helper subproblem, `P_m` the main subproblem whose initial state is `s′`, and
`E` is symbolic effect application along the helper plan `π_h`. `D(P)` is admissible only if
every proof obligation below is machine-checked:

1. **Goal coverage** — goals(P_h) ∪ goals(P_m) entail `S_g`; nothing dropped.
2. **Helper reachability** — a classical plan `π_h` exists for `P_h` from `s_i`.
3. **Interface-state correctness** — `s′` is exactly the replayed result of `π_h`, with each
   step's preconditions verified (violation = typed refusal, PROJ-707 `CNG_R23`).
4. **Main reachability** — a classical plan `π_m` exists for `P_m` from `s′`.
5. **Non-interference** — Effects ∩ ProtectedPreconditions = ∅ in both directions
   (PROJ-708 `CNG_R22`).
6. **Resource-release closure** — every resource the helper acquires is released before the
   interface, or explicitly carried as an obligation (PROJ-708 `CNG_R24`).

Status: PLANNED (PROJ-701..710).

## 5. No-LLM decomposition mechanism per dialect

| Dialect | Role in decomposition | Status |
|---|---|---|
| PDDL (text) | Admitted possibility space; renderer output | planner ALIVE (interim) |
| pddl-strips (RDF) | Queryable STRIPS vocab (Action/effects/Problem/init/goal) | PLANNED (701) |
| SPARQL CONSTRUCT | Manufactures helper/main graphs + `s′` init atoms | PLANNED (706/707) |
| Datalog `decomp.dl` | Derives achieves/threatens/mutex/dependsOn edges | PLANNED (704) |
| POWL | Nested `PartialOrder`; top level = cross-workflow edges only | PLANNED (709) |
| Arazzo/OpenAPI/AsyncAPI | Cross-engine dispatch projection surface (§13) | PLANNED (725/726) |
| OCEL | Cross-engine execution history; §16 markers are SPARQL over it | PLANNED (727) |

The classical planner is the unchanged `bcinr_pddl::GroundProblem::find_plan` path (ALIVE in
the interim substrate); the lifter/renderer bridge around it is PLANNED (PROJ-702/703).
Datalog resource predicates are admitted facts (`decomp-resources.dl`), never Rust constants.

No dialect calls an LLM; no English subgoal exists anywhere in the pipeline.

## 6. Bounded decomposition algorithm (15 steps)

```text
 1. Admit PDDL domain/problem strings into the graph (existing admission law).
 2. Lift PDDL text → pddl-strips triples in a fresh store (CONSTRUCT-into-new-store).
 3. Run rules/decomp.dl + decomp-resources.dl to derive edge predicates.
 4. Partition goal atoms by union-find over derived dependency edges.
 5. Enumerate 2-way split candidates canonically (lexicographic; max 8 components,
    max 32 candidates); single-actor is always candidate #0.
 6. For each candidate: CONSTRUCT the helper problem graph P_h.
 7. Render P_h → deterministic PDDL text (templates/decomp-problem.template.pddl).
 8. Plan helper: find_plan(P_h) → π_h, or mark candidate inadmissible (obligation 2).
 9. Replay π_h with per-step precondition verification → s′ (obligation 3; CNG_R23).
10. Check resource-release closure at the interface (obligation 6; CNG_R24).
11. CONSTRUCT the main problem graph P_m with s′ as init (obligation 1 checked here).
12. Plan main: find_plan(P_m) → π_m, or mark candidate inadmissible (obligation 4).
13. Prove non-interference both directions (obligation 5; CNG_R22).
14. Compose nested POWL, emit powl2 RDF, score (Makespan, DispatchCost, Risk).
15. Select by lexicographic order with canonical-id tie-break; receipt every candidate
    (accepted and rejected) and the selection itself; emit typed DecompositionOutcome.
```

Every bound (8 components, 32 candidates, planner search bound) is declared and receipted;
exceeding a bound is a typed result, never a panic or silent truncation.

Status: PLANNED (PROJ-705..710).

## 7. Single-actor route — typed results, never fallback

The single-actor plan is candidate #0, evaluated under the same law as every split. When no
split passes the proof obligations, the outcome is `NO_ADMISSIBLE_DECOMPOSITION`; when splits
pass but none beats candidate #0 under the selection law, the outcome is
`NO_BENEFICIAL_DECOMPOSITION`. Both are **typed success results** (`DecompositionOutcome`),
receipted with the full candidate ledger — never refusals, never silent fallbacks, never
"decomposition failed so we quietly ran the flat plan". `audit_replay` recomputes the argmin.

Status: PLANNED (PROJ-710).

## 8. Potato canonical scenario

The canonical worked scenario is the potato problem: a hand-authored PDDL fixture whose
lawful decomposition (helper prepares, main completes, interface state exact) is derived by
the algorithm in §6 — not hardcoded — composed as helper ∥ main POWL, executed across the H
and M engines of §10, with the global goal closed and every claim read back from OCEL via
SPARQL. An exact-output test pins the fixture.

Status: ALIVE (single-process, potato —
`potato_decomposition_is_typed_receipted_and_replayable`, PROJ-712) / UNVERIFIED (potato
itself dispatched cross-engine — potato's real `decompose()` output selects
`DecompositionOutcome::NoAdmissibleDecomposition`, single-actor, so it has no split to
dispatch) / ALIVE (the general cross-engine decompose-to-dispatch mechanism, PROJ-749, second
synthesis round, proven on a different fixture — kitchen two-chain — that does split). See
`DOD_SIGNOFF.md` §8 for the full clause-level reconciliation.

## 9. Recursive generalization 8¹ → 8² → 8³

Decomposition is recursive: a helper or main subworkflow is itself a planning target and may
decompose again under the same bounded law. The release must demonstrate 8¹ (one level,
potato) and 8² spanning engines (PROJ-729); 8³ is described by the same law but is not a
required demonstration this increment — claiming it requires fresh evidence.

Status: 8¹ ALIVE (potato, PROJ-712). 8² ALIVE at the literal fan_out=8/depth=2 target —
`recursion_crosses_engines_full_8x2_fanout` (`crates/cng/tests/cng_multi_engine.rs`, PROJ-729
follow-up round) exercises 73 dispatches per root (1 + 8 + 64), 146 total across two roots (H,
M), 64 depth-2 leaves; 2/2 runs green, 37.19s and 32.50s. A prior fan_out=2 smoke test
(`recursion_crosses_engines_depth_two`) remains in the suite. 8³ UNVERIFIED, out of required
scope.

## 10. Multi-engine topology

Coordinator `C`, helper `H`, and main `M` run as **separate OS processes**, each a full
Chatman Engine `cng` instance with its own store, identity, bundle, receipts, and replay.

Forbidden between engines (each has a falsifier in §14):

1. Shared in-memory state of any kind.
2. Shared in-process triple store.
3. Direct function calls across engine boundaries.
4. Direct writes into another engine's store, inbox excepted per the broker law.
5. LLM-mediated dispatch or admission.
6. Any cross-engine consequence that is not receipted and re-admitted.

The only lawful cross-engine path is:

```text
POWL → Arazzo projection → broker → OpenAPI/AsyncAPI-described operation → remote engine
  → consequence → re-admission at the receiving engine
```

Status: PLANNED (PROJ-720..729). Transport is filesystem this increment — see §20.

## 11. Authority partition table

| Authority | C (coordinator) | H (helper) | M (main) | Broker |
|---|---|---|---|---|
| Decomposition + selection | owns | — | — | — |
| Cross-engine dispatch | requests | — | — | owns (sole path) |
| Contract admission (inbox) | — | owns for H | owns for M | delivers only |
| Workflow execution | local only | owns helper POWL | owns main POWL | never executes |
| Consequence emission | — | owns (outbox) | owns (outbox) | transports only |
| Consequence re-admission | owns (C store) | — | — | never admits |
| Receipts + replay | own bundle | own bundle | own bundle | ledgered by C |
| Global goal standing | owns | never | never | never |

No engine creates standing in another engine's store. Coordinator bundle links child bundle
digests; each child chain is independently replayable.

Status: PLANNED (PROJ-722/723/727).

## 12. 16-state cross-engine dispatch machine

The interim 13-state machine is extended in place to 16 states — one law, no shim, all three
co-located authorities in `dispatch.rs` plus shapes individuals, templates, query strings,
and tests updated together, with a drift test (TTL individuals == `as_str` set):

```text
MANUFACTURED → ARAZZO_RENDERED → DISPATCH_READY → DISPATCHED → ACKNOWLEDGED
  → REMOTE_STARTED → REMOTE_IN_PROGRESS → RESULT_AVAILABLE → RESULT_RECEIVED
  → RESULT_ADMITTED → COMPLETED
REFUSED, TIMED_OUT, COMPENSATING, BLOCKED, UNKNOWN
```

Deltas from the interim machine: `ARAZZO_RENDERED` and `REMOTE_STARTED` added;
`IN_PROGRESS → REMOTE_IN_PROGRESS` and `ADMITTED → RESULT_ADMITTED` renamed;
`RESULT_RETURNED` split into `RESULT_AVAILABLE → RESULT_RECEIVED`. No implicit completion.

Status: PLANNED (PROJ-720).

## 13. Distributed broker law

Broker exclusivity extends across processes: no dialect, LLM, script, CLI, agent, or engine
dispatches to another engine directly. Additionally:

1. Every `advance()` appends a StateEntry to a durable per-dispatch ledger
   (`ledger/<dispatch_id>.ttl`, atomic tmp+rename) — PROJ-721.
2. Consumption is idempotent: `processed.ttl` closes the double-admit hole; a repeated
   consequence is a typed `DoubleAdmit` refusal — PROJ-721.
3. Every contract names its `disp:targetEngine`; every observation carries
   `obs:producedByEngine` — PROJ-722.
4. Crash-restart-resume is lawful: `cng engine resume` reloads the ledger tail and verifies
   the receipt-chain prefix; a torn ledger tail refuses lawfully — PROJ-724 (G13).
5. Engine identity is deterministic (`engine_id`, `ENGINE_VERSION`, splitmix64 nonce) — no
   PID, no wall clock, nothing platform-dependent in any digest — PROJ-722.

Status: PLANNED (PROJ-721..724).

## 14. Isolation falsifiers

Isolation claims are proven structurally plus by negative observation — reported as
"structural + negative-obs evidence", not omniscience:

1. C, H, M are separate OS processes (multi-process harness, `CARGO_BIN_EXE` pattern).
2. `SHARED_MEMORY_CROSSINGS=0` and `DIRECT_ENGINE_BYPASSES=0` markers prove no forbidden
   observation was emitted; a SPARQL rule mints a `DirectEngineBypass` obs for any admitted
   consequence lacking a matching `RemoteDispatchSent`/ledger entry.
3. Filesystem inventory assertion: `engines/*/{inbox,outbox}` are the only cross-engine
   artifacts.
4. Bypass-injection negative test: a hand-planted bypass consequence must flip the marker
   false and fail the run with a typed refusal + nonzero exit.
5. Determinism pinning: sorted inbox scans, admit-in-dispatch-id order, zero-padded ids, no
   PIDs or absolute paths in digests; two full C+H+M runs byte-identical.

Status: PLANNED (PROJ-727/728/729).

## 15. Gall checkpoints G0–G16

Each checkpoint is a working system before the next is attempted (Gall's law). Distributed
checkpoints are primary; planning checkpoints are folded in where they gate execution.

| Gate | Working system proven | Ticket(s) | Status |
|---|---|---|---|
| G0 | Interim substrate green (`31c236f` closure re-cited, not re-asserted) | — | ALIVE (interim) |
| G1 | `pddl-strips.ttl` vocabulary + closed shapes admit/refuse correctly | PROJ-701 | PLANNED |
| G2 | Lift ∘ render round-trip byte-stable (property test) | PROJ-702/703 | PLANNED |
| G3 | `decomp.dl` derives edge predicates from admitted facts only | PROJ-704 | PLANNED |
| G4 | Bounded canonical candidate enumeration; single-actor = #0 | PROJ-705 | PLANNED |
| G5 | Helper/main manufacture + `s′` replay proof (`CNG_R23`) | PROJ-706/707 | PLANNED |
| G6 | Non-interference + release-closure proofs (`CNG_R22`/`CNG_R24`) | PROJ-708 | PLANNED |
| G7 | POWL composition accepted; selection law + candidate receipts | 709/710 | PLANNED |
| G8 | Potato decomposition derived, not hardcoded (single process) | PROJ-712 | PLANNED |
| G9 | 16-state machine everywhere + drift test green | PROJ-720 | PLANNED |
| G10 | Durable ledger + idempotent consume (`DoubleAdmit`) | PROJ-721 | PLANNED |
| G11 | Engine identity + per-engine bundles independently replayable | PROJ-722 | PLANNED |
| G12 | `engine serve` loop + Arazzo/API projection on the lawful path | 723/725/726 | PLANNED |
| G13 | Crash-restart-resume; chain-prefix verified after kill/restart | 724/729 | PLANNED |
| G14 | IPC corpus runs, or honest PARTIAL at the solvability bound | 711 | PLANNED |
| G15 | 4 long-horizon scenarios (may be CUT, never faked) | 714 | ALIVE (mechanism, 2/4 real scenarios) / PLANNED (3-4, time-boxed) — see PROJ-714.md |
| G16 | Revised marker conjunction TRUE via SPARQL; closure + sign-off | PROJ-727/731 | PLANNED |

## 16. Required result markers

All markers are SPARQL-derived over the emitted OCEL/evidence graphs — never asserted. Any
false marker is a typed refusal (`CNG_R20 MarkerFalse`) + nonzero exit
(`crates/cng/src/bench/workday.rs:542-567`, `evaluate_marker_map`). This section is reconciled
to exact on-disk marker names and query files this pass (PROJ-743); prior prose used bare
names (`LLM_CALLS=0`, `ENGINE_INSTANCES`, `RESUME_VERIFIED`, `CRASH_RESUME_PROVEN`) that never
matched code. Per decision 4 of the closure plan, the doctrine is reconciled to the code, not
the reverse.

### LLM_CALLS_ZERO family (PROJ-740)

One query file, three markers — `_ZERO` suffix, not the bare `LLM_CALLS=0` form:
`crates/cng/queries/markers/marker-no-llm-authoring.rq` returns `LLM_CALLS_ZERO`,
`ENGLISH_SUBGOALS_ZERO`, `CANNED_SUBGOALS_ZERO`. The query header documents the load-bearing
claim as STRUCTURAL (no LLM/inference-API crate anywhere in `crates/cng/Cargo.toml`'s
dependency tree; every rendered artifact — PDDL problem text, POWL Turtle, receipt Turtle —
comes from an on-disk `.template.pddl`/`.template.ttl` file with typed placeholder
substitution, never free text) plus a SECONDARY negative-obs half that would go nonzero if any
of the three forbidden `obs:obsKind` values were ever emitted (none is, by construction).

Status: ALIVE — `cargo test -p cng --features bench --test cng_decomp` (3/3 passed, this
session) and the `planning_markers_prove_true_on_a_healthy_decompose_run` test
(`crates/cng/src/bench/workday_test.rs:332-383`, part of the `67 lib` tests in the 107-test
`cargo test -p cng --features bench` full-suite run, 0 failures, this session) assert all
three names true over a real `decompose()` run's `decomposition-result.ttl`.

### Planning set (PROJ-739)

Six markers, each with its own query file under `crates/cng/queries/markers/`:

| Marker | Query |
|---|---|
| `DECOMPOSITION_DERIVED_PROVEN` | `marker-decomposition-derived.rq` |
| `DECOMPOSITION_CANDIDATES_RECEIPTED` | `marker-decomposition-receipted.rq` |
| `INTERFACE_STATE_PROVEN` | `marker-decomposition-interface-state.rq` |
| `NON_INTERFERENCE_PROVEN` | `marker-decomposition-non-interference.rq` |
| `RESOURCE_RELEASE_CLOSED` | `marker-decomposition-release-closure.rq` |
| `SINGLE_ACTOR_TYPED_RESULT` | `marker-decomposition-single-actor-typed.rq` |

Together with the three `_ZERO` markers above, these nine form `PLANNING_MARKER_MAP`
(`crates/cng/src/bench/workday.rs:186-219`), evaluated by `evaluate_planning_markers`
(`workday.rs:614-619`) over a DEDICATED store built by `build_decomp_marker_store`
(`workday.rs:587-601`) from `decomposition-result.ttl` alone — deliberately NOT folded into
the obs∪evidence∪registry `MARKER_MAP` union `workday()` uses, because a plain `workday()` run
never has `decomp:` facts and a plain `cng plan decompose` run never has `obs:` facts
(`workday.rs:176-185`).

Status: ALIVE — same evidence as the `LLM_CALLS_ZERO` family above (one test asserts all nine
planning markers true).

### Distributed set (PROJ-727) — names reconciled to on-disk `DISTRIBUTED_MARKER_MAP`

Prior prose named `ENGINE_INSTANCES`, `ARAZZO_WORKFLOWS_GENERATED`, `REMOTE_DISPATCHES_SENT`,
`REMOTE_CONSEQUENCES_ADMITTED`, and `RESUME_VERIFIED` — none of these identifiers exist on
disk (verified this session: zero grep hits for each in `crates/cng`). This is a second,
independent marker-name mismatch found while reconciling this section, beyond the planning
set's cosmetic `_PROVEN`/`_ZERO` suffix convention already correct above. What is actually on
disk, in `DISTRIBUTED_MARKER_MAP` (`crates/cng/src/bench/workday.rs:148-174`, six query stems,
nine marker names):

| Marker | Query |
|---|---|
| `SHARED_MEMORY_CROSSINGS_ZERO` | `marker-engine-isolation.rq` |
| `DIRECT_ENGINE_BYPASSES_ZERO` | `marker-engine-isolation.rq` |
| `REMOTE_WORKFLOWS_ACKNOWLEDGED` | `marker-remote-execution.rq` |
| `REMOTE_WORKFLOWS_COMPLETED` | `marker-remote-execution.rq` |
| `REPLAY_DIVERGENCES_ZERO` | `marker-replay-divergence.rq` (folds G13's `resume_verified` check) |
| `ARAZZO_WORKFLOWS_DISPATCHED` | `marker-arazzo-dispatch.rq` |
| `MULTI_ENGINE_EXECUTION_PROVEN` | `marker-multi-engine-execution.rq` (inverted, ≥2 engines) |
| `ENGINE_INSTANCES_PROVEN` | `marker-multi-engine-execution.rq` |
| `ARAZZO_INTER_ENGINE_WORKFLOW_PROVEN` | `marker-arazzo-inter-engine.rq` (inverted-existence) |

The raw counts prior prose's names gestured at exist as numeric `WorkdayReport` fields that
feed these boolean markers and the telemetry reconciliation, not as standalone SPARQL
markers themselves: `engine_instances`, `arazzo_workflows_generated`,
`arazzo_workflows_dispatched`, `remote_dispatches`, `remote_consequences_received`
(`workday.rs:271-289`).

Status: ALIVE, scoped — `cargo test -p cng --features bench --test cng_multi_engine
-- --test-threads=1` (6/6 passed, this session) asserts `MULTI_ENGINE_EXECUTION_PROVEN`,
`ARAZZO_INTER_ENGINE_WORKFLOW_PROVEN`, `DIRECT_ENGINE_BYPASSES_ZERO`,
`SHARED_MEMORY_CROSSINGS_ZERO`, and `REMOTE_WORKFLOWS_COMPLETED` all true
(`crates/cng/tests/cng_multi_engine.rs:232-236`), via CARGO_BIN_EXE-spawned separate OS
processes (PROJ-728's real multi-process isolation harness) — not yet a standalone `cng engine
serve`/`resume` production orchestration run outside `cargo test`.

### G13 crash-resume — `CRASH_RESUME_PROVEN` does not exist; reconciled to the real mechanism

Prior prose named a `CRASH_RESUME_PROVEN` marker under "Revised final markers" below. No such
identifier exists anywhere in `crates/cng` (verified this session: zero grep hits). G13
crash-resume is proven instead by: (a) the `resume_verified` obs-kind rolled into the
`REPLAY_DIVERGENCES_ZERO` marker above (`marker-replay-divergence.rq` checks `resume_verified`
observations carry no `ex:divergence "true"`), and (b) the dedicated integration test
`g13_crash_resume_verifies_chain_and_completes`, part of the 6/6-passed
`cng_multi_engine` run cited above. No separate marker name is needed or invented — the
doctrine is corrected to point at the real mechanism instead of a nonexistent identifier.

### `V26_7_10_PRODUCTION_READY` — the two-run composition (load-bearing correction)

`workday()` never calls `decompose()` — they are separate run types by design
(`workday.rs:623`: "a `workday()` run that never invokes `decompose()` and therefore cannot
honestly claim the planning surface on its own"). The interim `evaluate_markers`/`MARKER_MAP`
conjunction `workday()` computes (`workday.rs:516-527`, ten query stems / sixteen names plus
the conjunction) is **not** the DoD's full §16 conjunction — it never includes the planning
set or the distributed set, because a single `workday()` invocation produces no `decomp:`
facts and, on a single-operator run, no multi-engine facts.

The DoD-accurate value requires combining evidence from **two (optionally three) separate
runs**, via the additive combinator `full_production_ready(workday_markers, planning_markers,
distributed_markers: Option<...>)` (`crates/cng/src/bench/workday.rs:644-665`, PROJ-742):

1. Run `workday()` (single-operator autonomic loop) → `evaluate_markers()` output.
2. Run `cng plan decompose --domain <path> --problem <path> --out <dir>` (PROJ-741) →
   `evaluate_planning_markers()` output over the written `decomposition-result.ttl`.
3. Optionally run the multi-engine coordinator (`engine_collect_remote`) →
   `DISTRIBUTED_MARKER_MAP` output.
4. Call `full_production_ready` with the available maps; it strips the interim
   `V26_7_10_PRODUCTION_READY` entry from the workday map, merges in the planning (and, if
   present, distributed) markers, and RECOMPUTES `V26_7_10_PRODUCTION_READY` as the
   conjunction over the merged set — the same marker name, now meaning the full §16
   conjunction this document defines, not the interim single-operator subset.

**A `workday()` run alone never proves the full `V26_7_10_PRODUCTION_READY` claim** — that
requires a caller to also produce a planning-set bundle (and, for the distributed clauses, a
distributed bundle) and pass all of them through `full_production_ready`. This is the single
most important correction in this reconciliation pass: earlier prose read as if `workday()`
alone proved the whole conjunction, and by design it does not.

### Three-bundle composition: ALIVE (EOD push) — workday + planning + distributed, all real

`full_production_ready` was initially exercised only by two unit tests
(`crates/cng/src/bench/workday_test.rs:332-383` and `:385-405`) that combined REAL planning
markers with a HAND-FABRICATED `workday_markers` half, or hand-fabricated both sides. A
follow-up round closed the two-way gap (`cng_production_ready.rs`, real `workday()` +
`decompose()` bundles, `full_production_ready_holds_on_real_dual_bundle_evidence`). A further
EOD push closed the remaining third leg: new, independent file
`crates/cng/tests/cng_production_ready_three_way.rs` runs a REAL `workday()` bundle, a REAL
`decompose()` bundle, AND a REAL two-engine (`H`/`M`) coordinate round —
`engine_dispatch_remote` then two real `cng engine serve` OS processes then
`engine_collect_remote`, whose `EngineCoordinateReport.markers` field is the real, already-
evaluated `DISTRIBUTED_MARKER_MAP` output (no new marker-evaluation machinery needed —
`engine_collect_remote`/`EngineCoordinateReport` were already `pub`). All three real maps feed
`full_production_ready(&workday_markers, &planning_markers, Some(&distributed_markers))`; the
combined 29-key map's `V26_7_10_PRODUCTION_READY` asserts `true`
(`full_production_ready_holds_on_real_triple_bundle_evidence`), with a companion negative test
forcing a real distributed marker false and confirming the conjunction goes `false`
(`full_production_ready_goes_false_when_a_real_distributed_marker_is_forced_false`). Command:
`CARGO_TARGET_DIR=target/agent-threeway just cng-test-one cng_production_ready_three_way --
--test-threads=1 --nocapture` → 2 passed, 0 failed, 5.62s.

Status: planning set + `LLM_CALLS_ZERO` family ALIVE (PROJ-739/740); distributed set ALIVE
scoped to the CARGO_BIN_EXE test harness (PROJ-727/728); `full_production_ready` combinator
ALIVE as a pure function AND ALIVE for both the real two-bundle and the real three-bundle
invocation (PROJ-742, closed EOD push). `V26_7_10_PRODUCTION_READY` in its FULL §16 meaning
may now be claimed for the three-way composition — every constituent leg (workday, planning,
distributed) has been independently proven true on real evidence, and the combinator itself
has been proven to correctly propagate a forced-false on each leg.

## 17. Anti-hardcoding requirements

Permuted identities, initial states, and roles must **causally** change the decomposition,
plan, and refusal digests — a permuted-seed rerun with unchanged digests fails the gate.
Canned-subgoal detection is a typed refusal: any decomposition artifact not derivable from
the admitted facts of this run is refused by construction. Resource predicates are admitted
facts (`decomp-resources.dl` EDB), never Rust constants.

Status: PLANNED (PROJ-713).

## 18. Negative proof corpus (§13 corpus)

`tests/fixtures/decomp-negative/` holds problems that must refuse or resolve to a typed
non-decomposition outcome — each with an exact-expectation test:

1. Unreachable helper goal → candidate inadmissible (obligation 2).
2. Interface-state mismatch → `CNG_R23 InterfaceStateMismatch`.
3. Interfering effect pair → `CNG_R22` (obligation 5).
4. Unreleased resource at the interface → `CNG_R24 ResourceUnreleased`.
5. Mutex-saturated goals → `NO_ADMISSIBLE_DECOMPOSITION` (typed, receipted).
6. Splits admissible but never beneficial → `NO_BENEFICIAL_DECOMPOSITION`.
7. Injected canned subgoal → anti-hardcoding refusal (§17).
8. Bypass-injection distributed negative (§14 item 4).

Status: ALIVE (8/8, PROJ-712/713/728). Items 6 and 7 closed this session: item 6 by
`splits_admissible_but_not_beneficial_forces_no_beneficial_decomposition`
(`cng_decomp_negative_corpus_completeness.rs`, second synthesis round); item 7 by
`detached_graph_action_refuses_cng_r09_hardcoding_suspicion` (`decomp/decomp_test.rs`,
follow-up round), additionally confirmed by
`canned_subgoal_detection_catches_identical_goal_labels_with_different_achiever_structure`
(same second-synthesis-round file). See `DOD_SIGNOFF.md` §18 for the full 8-item table.

## 19. Paper-equivalent evaluation corpus

Clean-room generated (never copied): 5 IPC domains — barman, blocksworld, termes, tyreworld,
grippers — × 20 seeded problems each (`(seed, size) → PDDL` via splitmix64, deterministic
size-backoff solvability gate honest to the blind-BFS bound), plus 4 long-horizon scenarios.

Honest note, binding: decomposition is **not** required to beat single-actor everywhere.
`NO_BENEFICIAL_DECOMPOSITION` on corpus problems is a valid, receipted outcome and counts
toward completion; only silent fallback or an unreceipted candidate ledger fails the gate.
If mid-size problems exceed the blind-BFS solvability bound, the result is an honest PARTIAL
at the declared bound, not heuristic-planner scope creep.

Status: ALIVE, full scale (PROJ-711 — 5x20=100, `cng_ipc_corpus_full_scale.rs`, 2/2 runs
green, 11.66s/11.79s). Long-horizon set (PROJ-714): ALIVE mechanism, 2/4 real scenarios
(`long_horizon_logistics_scenario_decomposes_and_plans_end_to_end`, a genuine 2-actor
30-step logistics split, real helper/main benefit at makespan 15 vs. single-actor 30;
`long_horizon_tyreworld_chain_scenario_decomposes_and_plans_end_to_end`, moonshot round, a
2-instance chained tyreworld domain clearing the same ~20-step bar) — scenarios 3-4 (barman,
termes, blocksworld, and grippers were each tried at minimum chain length and dropped per
PROJ-714.md's own "do not force a fit" clause; every one hit a genuine planner-search
performance cliff, not a grounding blowup) remain a time-boxed cut this session, not
silently dropped.

## 20. Honest boundaries

1. **Filesystem-as-transport.** Separate OS processes are proven (multi-process harness);
   transport between them is a deterministic filesystem inbox/outbox. HTTP binding is
   **declared** via generated OpenAPI/AsyncAPI documents but is UNVERIFIED as a live network
   path — no claim of network execution may be made. Rationale: tokio/async + wall clock
   would enter the digest path and break byte-identical replay. Label: "mechanism ALIVE
   / HTTP binding UNVERIFIED".
   Render-verification granularity (updated this session — both legs now digest-verified,
   neither leg claims a live network path):
   **Arazzo** is generated by `packs/arazzo-pack/templates/arazzo.yaml.tmpl` AND
   digest-verified before dispatch — `run_arazzo_projection` calls
   `verify_arazzo_render_digest` (PROJ-745, `crates/cng/src/bench/arazzo.rs`), recomputing
   BLAKE3 over the on-disk render and comparing it to the ggen sync receipt
   (`.ggen-v2/receipt.json`) before any step reaches `DispatchState::ArazzoRendered`; a
   stale/tampered render refuses `CNG_R11 AuditMismatch`. **OpenAPI/AsyncAPI**
   (`packs/arazzo-pack/templates/engine-openapi.yaml.tmpl`,
   `engine-asyncapi.yaml.tmpl` → `generated/engine-openapi.yaml`,
   `generated/engine-asyncapi.yaml`) are now ALSO digest-verified, at engine startup rather
   than at dispatch time: `crates/cng/src/bench/api_docs.rs`'s
   `verify_api_docs_render_digest_if_present` is called from `engine_serve`
   (`crates/cng/src/bench/engine.rs:589`) before the poll loop begins, recomputing BLAKE3
   over both rendered documents and comparing against the same `.ggen-v2/receipt.json`
   shape Arazzo uses; a tampered document refuses `CNG_R11 AuditMismatch` before the engine
   starts serving. Absence is handled honestly, not as a false refusal: an engine root
   without pre-generated `generated/`/`.ggen-v2/` proceeds normally (`Ok(None)`) — the check
   only enforces when the documents are actually present, matching the filesystem-as-
   transport boundary (not every engine root has been through a `ggen sync run`). Verified
   this session: 6 new tests in `crates/cng/src/bench/api_docs_test.rs`
   (`engine_serve_proceeds_when_api_docs_present_and_matching`,
   `engine_serve_refuses_cng_r11_when_api_doc_render_tampered`,
   `engine_serve_proceeds_when_api_docs_absent`, plus 3 function-level tests), full
   `cargo test -p cng --features bench --lib` 77/77 passed. What remains UNVERIFIED for
   both legs, unchanged: no live network transport exists or is claimed — this is
   digest-integrity of a generated document, not a running HTTP/AsyncAPI server.
2. **Real time.** The single real-time element (inter-poll sleep) sits behind a
   `RealTimeWait` seam and never enters digests; logical poll counts do.
3. **Long-horizon scenarios** (4, G15/PROJ-714): the mechanism is proven real on 2 scenarios
   this session (`tests/cng_long_horizon_scenario.rs` — logistics + tyreworld-chain, neither
   faked nor stubbed); scenarios 3-4 are a time-boxed cut, recorded honestly (PROJ-714.md),
   not silently dropped.
4. **Synthesized human consequences** remain MOCKED-HUMAN wherever they appear (carried
   forward from the interim DoD).
5. **8³ recursion** is doctrine, not a required demonstration (§9) — UNVERIFIED.
6. No unscoped "production-ready" claim exists: `V26_7_10_PRODUCTION_READY` means exactly
   the §16 conjunction under these boundaries.

## See Also

- `docs/releases/v26.7.10/DEFINITION_OF_DONE_INTERIM.md` — superseded prior DoD (interim
  milestone, closed at `31c236f`)
- `docs/releases/v26.7.10/RELEASE_CONTROL.md` — single control surface; wins on disagreement
- `docs/releases/v26.7.10/PRD.md` / `ARD.md` — interim-scope requirements docs
- `docs/releases/v26.7.10/DOD_SIGNOFF.md` / `DOD_EVIDENCE_MAP.md` — clause-by-clause sign-off
  and evidence index for this document (PROJ-748)
- `docs/jira/v26.7.10/tickets/index.md` — PROJ-601..622 (interim, closed) + PROJ-701..748
- `docs/CHATMAN_EQUATION.md` — `A = μ(O*)` formulation

# GAP_AUDIT.md — v26.7.10-revised Consolidated Gap Audit

Version: v26.7.10-revised. Status: snapshot report, read-only inputs, no code or doc changes made.

This document consolidates five independent read-only audits of the `crates/cng` /
`crates/praxis-graphlaw` tree and the `docs/releases/v26.7.10/` doc-closure artifacts. Each
source audit covered a different angle (doctrine-vs-code, refusal/outcome test coverage,
determinism/replay, doc/ticket internal consistency, safety/leak/clean-room/dependencies).
It is a report, not a fix — no gap listed here has been acted on. All five source audits were
run concurrently with an active doc-closure and negative-corpus-closure session, so several
findings are explicitly time-sensitive; each is tagged as such where the source audit flagged it.

## 1. Executive Summary

Across the five audits, this report catalogs **41 distinct gaps**: 2 doctrine-vs-code MISSING
findings, 4 doctrine-vs-code PARTIAL/UNCLEAR findings, 6 refusal/outcome coverage gaps (of 25
`CngRefusal` variants + 3 `DecompositionOutcome` variants + 16 `DispatchState` states), 3
concrete determinism/replay risks, 7 documentation internal-consistency issues, and 3 safety/
process-leak findings (plus 2 clean-room/dependency items that closed clean). Of these, roughly
**12 are load-bearing** — they touch correctness-critical or safety-critical surface (CNG_R10
`IoRefused` and CNG_R08 `Nondeterminism` having zero negative-path tests; `DispatchState::Blocked`
unreachable in any test; the `dispatch_bridge.rs` module citing a nonexistent test file while
having zero callers; three process-leak windows in `cng_multi_engine.rs`; the stale HEAD/status
claims repeated across 36+ files). The remainder (roughly **29**) are cosmetic or doc-polish —
stale `Status: PLANNED` labels, a suppressed PARTIAL sub-claim rounding up to ALIVE in two summary
tables, vocabulary drift across three different status-word sets, an unexplained ticket-number
gap, and one file worth a second look for clean-room provenance without concrete evidence of a
problem. Two areas — the CNG_Rxx code table structure and dependency/license posture — audited
clean with no gaps found.

**Load-bearing closure pass (this session, after this audit was originally written).** An
8-agent wave (7 parallel + 1 sequential synthesis) closed the top 10 items of §7's punch list —
see each item's own **Closure status** annotation for the exact command+output. Net: 5 items
CLOSED (CNG_R10 at one representative site, the 3 process-leak windows, `TIMED_OUT→BLOCKED`
reachability, CNG_R07 at 2 of 5 sites, the stale-HEAD sweep); 3 items INVESTIGATED and found
correctly out-of-scope/unreachable/vestigial rather than forced (CNG_R08, `DISPATCH_READY→
REFUSED`, OpenAPI/AsyncAPI — each got a doc correction instead of a fabricated fixture); 2 items
were already STALE before the wave started (`dispatch_bridge.rs`'s cited test file exists;
`NoBeneficialDecomposition` is forced) and are corrected in place at their original findings
(§2, §3.2). Items 11-20 (the narrower §18/§14 sub-parts and the 29 non-load-bearing cosmetic
items) were out of this wave's scope and remain open or unchanged — see §7 for the full
per-item detail. Full account: `docs/releases/v26.7.10/DOD_SIGNOFF.md`'s "Load-bearing closure
pass" section.

## 2. Doctrine-vs-Code Gaps (Audit 1) — by Gall Checkpoint

Read against `DEFINITION_OF_DONE.md` (516 lines) cross-referenced with `RELEASE_CONTROL.md`,
`DOD_SIGNOFF.md`, `DOD_EVIDENCE_MAP.md`, and the `crates/cng` source/test tree.

| Checkpoint | Claim | Tag | Note |
|---|---|---|---|
| G0 | Interim substrate, commit `31c236f` | VERIFIED | Commit exists and matches citation. |
| G1 | `pddl-strips.ttl` + closed SHACL shapes | VERIFIED | Doc's `Status: PLANNED` label is STALE relative to code — real classes/shapes exist, closed-by-name refusal on unknown constructs. |
| G2 | lift∘render round-trip byte-stable | VERIFIED | `decomp_test.rs:336` proves atom-set identity + byte-identical double render. |
| G3 | `decomp.dl` derives edges from admitted facts only | VERIFIED | Real Datalog rule set; `decomp-resources.dl` is a genuine EDB file, never a Rust constant. |
| G4 | Bounded canonical enumeration; single-actor = candidate #0 | VERIFIED | `DECOMP_MAX_COMPONENTS=8`, `DECOMP_MAX_CANDIDATES=32`; `decomp_test.rs:315` proves canonical ordering. |
| G5 | Helper/main manufacture + s′ replay, `CNG_R23` | VERIFIED | Two independent proofs: `decomp_test.rs:162,182` and `cng_ipc_corpus.rs:331`. |
| G6 | Non-interference + release-closure, `CNG_R22`/`CNG_R24` | VERIFIED | Positive and negative cases in `decomp_test.rs` and `cng_ipc_corpus.rs`. |
| G7 | POWL composition + selection + receipts | VERIFIED | `decomp_test.rs:304` (cycle refusal), `cng_decomp.rs:78` (receipt count == candidate count). |
| G8 | Potato decomposition derived, not hardcoded | VERIFIED | Real 7-action/9-object/13-init/2-goal domain drives the pipeline; independent "kitchen" domain corroborates. |
| G9 | 16-state machine + drift test | VERIFIED (exact) | `dispatch_test.rs:444` enumerates all 256 transition pairs; `dispatch_test.rs:496` proves TTL individuals match the enum. |
| G10 | Durable ledger + idempotent consume/`DoubleAdmit` | VERIFIED | `FileLedgerSink` with atomic tmp+rename write; `cng_multi_engine.rs:303` proves `CNG_R25` end-to-end. |
| G11 | Engine identity + independently-replayable bundles | VERIFIED | `EngineIdentity::new` = `splitmix64(seed ^ blake3(engine_id))`, no PID/wall-clock. |
| G12 | `engine serve` loop + Arazzo/API projection on lawful path | ALIVE | Arazzo leg digest-verified before dispatch; OpenAPI/AsyncAPI leg digest-verified at engine startup (`api_docs.rs`, closed this session — see §7 item 8). Schema validation and live HTTP binding remain the open boundary, not this row. |
| G13 | Crash-restart-resume; chain-prefix verified | VERIFIED | `cng_multi_engine.rs:318` kills a real process mid-run and proves torn-tail refusal + successful resume. |
| G14 | IPC corpus runs at declared scale | VERIFIED | 5 domains × 20 seeds = 100 pairs, matches doc exactly. |
| G15 | 4 long-horizon scenarios, may be CUT | VERIFIED (honest absence) | No code exists; `RELEASE_CONTROL.md`/`DOD_EVIDENCE_MAP.md` both record PROJ-714 as declared-cut. |
| G16 | Revised marker conjunction TRUE via SPARQL | VERIFIED (marker computation); STALE (one sub-claim) | See "Honest gap is now stale" below. |

### STALE — DoD's "Honest gap" sub-claim (§16, lines 423–447)

DoD asserts the end-to-end two-bundle composition through `full_production_ready` (real
`workday()` output + real `evaluate_planning_markers()` output, together) is UNVERIFIED. This is
now STALE: `crates/cng/tests/cng_production_ready.rs` (untracked/new on disk) contains
`full_production_ready_holds_on_real_dual_bundle_evidence`, which runs both real bundles, feeds
both real marker maps in, and asserts the merged map's `V26_7_10_PRODUCTION_READY` is `true`,
plus a negative-branch companion test. None of `RELEASE_CONTROL.md`, `DOD_SIGNOFF.md`, or
`DOD_EVIDENCE_MAP.md` mention this file yet — not-yet-reconciled. Caveat: the narrower **three**-way
composition (workday + planning + a real spawned distributed bundle in the *same* call) remains
genuinely UNVERIFIED, so the doc's framing is accurate only for that narrower case now.

### MISSING — prose with zero corresponding implementation

1. **OpenAPI/AsyncAPI as a live dispatch leg.** §10's pipeline diagram and §20 item 1 claim HTTP
   binding is "declared via generated, digest-recorded OpenAPI/AsyncAPI documents." Ggen templates
   exist (`packs/arazzo-pack/templates/{engine-openapi,engine-asyncapi}.yaml.tmpl`), but no Rust
   code reads, digest-verifies, or dispatches through either document
   (`grep -rn "engine-openapi\|engine-asyncapi" crates/cng/src` → zero hits). Only the Arazzo YAML
   leg (`arazzo.rs:434 verify_arazzo_render_digest`) is wired. The pack's own README already
   documents this as a named seam, not a delivered wire-up.
2. **`dispatch_subworkflow_to_engine`/`collect_subworkflow_consequence` bridge.**
   `crates/cng/src/bench/decomp/dispatch_bridge.rs` is the only code that could bridge a
   `decompose()`-derived plan onto the multi-engine transport (needed for §9's "8² spanning
   engines" claim). Both functions are `pub` with **zero call sites** anywhere in the crate. The
   module doc cites `crates/cng/tests/cng_decompose_to_dispatch_integration.rs` as the exercising
   test — **that file does not exist on disk**. Decomposition and multi-engine dispatch each have
   real support separately; the bridge connecting them is unexercised dead code with a dangling
   test citation. (Cross-referenced independently by Audit 3 — see §4 below.)

   **STALE (corrected by the load-bearing closure pass, §7 item 3).** This finding predates
   PROJ-749 (the decompose-to-dispatch bridge, second synthesis round). Re-verified this
   session: `crates/cng/tests/cng_decompose_to_dispatch_integration.rs` exists on disk
   (13,164 bytes), with `kitchen_decomposition_splits_into_helper_and_main` and
   `decomposed_subworkflows_dispatch_to_real_engines_and_are_admitted` (2/2 passed per
   `DOD_SIGNOFF.md`/`DOD_EVIDENCE_MAP.md`). No agent in the 8-agent closure wave touched this
   file; the correction was already true before that wave started and is folded in here per
   this synthesis pass's instructions. See §7 item 3 for the full citation.

### PARTIAL / UNCLEAR

- **§18 negative-proof-corpus items 5–6** (`NO_ADMISSIBLE_DECOMPOSITION` via mutex saturation;
  `NO_BENEFICIAL_DECOMPOSITION`) — no fixture currently forces either scenario by the named
  mechanism with an exact-expectation assertion; existing tests only tolerate these outcomes as
  one of three acceptable results. Session task #32 ("Close negative-corpus items 6+7") was
  `in_progress` concurrently — this is a live, currently-being-fixed gap, not a misreading.
- **§18 item 7** (injected canned subgoal → anti-hardcoding refusal) — VERIFIED via a
  differently-named mechanism (`detached_graph_action_refuses_cng_r09_hardcoding_suspicion`).
- **§14 item 4** (hand-planted bypass consequence must flip marker false AND fail the run with
  nonzero exit) — PARTIAL. The marker-query half is proven directly via SPARQL over a hostile
  fixture; the fixture is never run through `evaluate_marker_map`/the CLI to observe an actual
  nonzero process exit — that half is untested.
- **Test-count discrepancy**: `cng_multi_engine.rs` has 7 `test!` blocks vs. the doc's cited
  "6/6" — a new test was added concurrently during the audit. Flagged as a timing artifact, not a
  contradiction (6 of 7 match the doc's description).

### Document-level observation

Sections 2, 4–14, and 17–19 are uniformly labeled `Status: PLANNED` even though most of the
mechanisms they describe have real, tested implementations per the checkpoints above. Per the
doc's own preamble, clauses stay PLANNED/UNVERIFIED "until `RELEASE_CONTROL.md` cites a command +
output for it" — a stricter bar than "code exists" — but a reader comparing only the status column
would significantly undercount what's implemented. `RELEASE_CONTROL.md` is considerably more
current than `DEFINITION_OF_DONE.md` on this point.

## 3. Refusal/Outcome Test-Coverage Matrix (Audit 2)

`CngRefusal` (`crates/cng/src/powl.rs:37-230`, `code()` at 237-265): exactly 25 variants,
CNG_R01–CNG_R25. `DecompositionOutcome` (`bench/decomp/mod.rs:87-99`): 3 variants. `DispatchState`
(`bench/dispatch.rs:124-152`): exactly 16 variants, 22 lawful transition edges (`lawful_to`,
lines 201-222).

### 3.1 CngRefusal matrix (all 25)

Legend: ✅ exact-variant/code assertion exists · ⚠️ weak (loose/`is_err`-only) · ❌ zero evidence

| Code | Variant | Positive-path test | Negative-path test |
|---|---|---|---|
| R01 | MalformedTtl | `cng_pipeline.rs::joseph_example_many_to_one` | ✅ `cng_negative_fixtures.rs:45-51` |
| R02 | MissingDomain | pipeline happy-path | ✅ `cng_pipeline.rs:251-259` |
| R03 | MissingProblem | pipeline happy-path | ✅ `cng_pipeline.rs:285-293` |
| R04 | PlanUnsolvable | pipeline happy-path | ✅ `powl_test.rs:52-53,198-199`; `cng_pipeline.rs:301-302`; `cng_negative_fixtures.rs:63-68`; `cng_ipc_corpus.rs:328` |
| R05 | UnsupportedConstruct | corpus manufacture happy paths | ✅ `powl_test.rs:90,273,290`; `cng_negative_fixtures.rs:91-96`; `cng_ipc_corpus.rs:438,458` |
| R06 | InvalidPowl | `cng_hierarchical.rs` valid-model pass | ✅ `cng_negative_fixtures.rs:112-117` |
| **R07** | **RunnerMismatch** | `cng_hierarchical.rs` (`.expect` success) | ❌ **none** |
| **R08** | **Nondeterminism** | byte-identical rerun proofs (workday/multi-engine/ipc) | ❌ **none** |
| R09 | HardcodingSuspicion | general passing manufacture/dispatch tests | ✅ `powl_test.rs:210-211` |
| **R10** | **IoRefused** | every I/O-touching happy path | ❌ **none** |
| R11 | AuditMismatch | `cng_workday_verify.rs` audit replay | ✅ best-covered variant: `arazzo_test.rs:201-206`, `dispatch_test.rs:731-736,699-756`, `engine_test.rs:164-184`, `workday_verify_test.rs:92-144`, `cng_workday_verify.rs:122-125`, `cng_bench_portability.rs:208-245` |
| R12 | StandingAmbiguous | normal single-candidate standing tests | ✅ `workday_test.rs:289-301` |
| R13 | UnreceiptedActuation | normal receipted-actuation tests | ✅ `workday_verify_test.rs:184-189`; `hooks_test.rs:107-112` |
| R14 | DialectRegistryRefused | normal registry-load tests | ✅ `workday_verify_test.rs:212-216`; `hooks_test.rs:133-144` |
| R15 | DispatchContractIncomplete | `fixture_contract()` happy paths | ✅ `dispatch_test.rs:65-84` |
| R16 | DispatchStateUnlawful | happy-path 10-edge walk | ✅ `dispatch_test.rs:91-106`; exhaustive check `:444-490` |
| R17 | ExternalConsequenceRefused | admitted-consequence happy paths | ✅ `dispatch_test.rs:113-141`; `workday_verify_test.rs:251-256` |
| R18 | ArazzoProfileRefused | `arazzo_projection_gate_admits_when_render_digest_matches_receipt` | ✅ `arazzo_test.rs:46-53` |
| R19 | EvidenceGateFailed | normal closure-satisfied tests | ✅ `workday_test.rs:201-206` |
| R20 | MarkerFalse | normal marker-true tests | ✅ `workday_test.rs:150-153,464,513` |
| R21 | DecompositionInadmissible | `kitchen_two_chain...`, `single_atom_goal_yields_no_admissible_decomposition` | ✅ `cng_ipc_corpus.rs:283-308`; `cng_decomp.rs:168` |
| R22 | InterferenceDetected | `kitchen_two_chain...` (interference-free split) | ✅ `decomp_test.rs:223`; `cng_ipc_corpus.rs:411` |
| R23 | InterfaceStateMismatch | potato/kitchen successful replay | ✅ `decomp_test.rs:196-199`; `cng_ipc_corpus.rs:356` |
| R24 | ResourceUnreleased | normal resource-released decomposition | ✅ `decomp_test.rs:269-272`; `cng_ipc_corpus.rs:386` |
| R25 | DoubleAdmit | normal single-admission dispatch tests | ✅ `dispatch_test.rs:594-642`; `cng_multi_engine.rs:313-315` |

### 3.2 DecompositionOutcome (3 variants)

| Variant | Forced + asserted by name? |
|---|---|
| `Selected` | ✅ `decomp_test.rs:381-410`, genuinely forced via `matches!`. |
| `NoAdmissibleDecomposition` | ✅ `decomp_test.rs:422-436`, genuinely forced via `assert_eq!`. |
| `NoBeneficialDecomposition` | ⚠️ No dedicated forced test. Only appears inside `A | B` loose matches (`cng_decomp.rs:105-118`) or "one of three strings" membership checks (`cng_ipc_corpus.rs:164-179`, `cng_ipc_corpus_full_scale.rs:82-92`) that never confirm this specific branch fired. **STALE (corrected by the load-bearing closure pass, §7 item 10)** — `cng_decomp_negative_corpus_completeness.rs:188-195` now forces the exact variant via `assert_eq!` (`splits_admissible_but_not_beneficial_forces_no_beneficial_decomposition`), pinning the exact score numbers (`makespan=4, dispatch_cost=6` vs. the split's `makespan=4, dispatch_cost=8`). Predates the 8-agent wave; no agent in it touched this file. |

### 3.3 DispatchState — 16 states / 22 lawful edges

The classification predicate is exhaustively verified (`dispatch_test.rs:444-490`, all 256 pairs
checked against the exact 22-edge table). Live-walk coverage is partial:

- Confirmed exercised end-to-end: full happy path `MANUFACTURED→…→COMPLETED`
  (`dispatch_test.rs:538-593`); `RESULT_RECEIVED→REFUSED→COMPENSATING→COMPLETED`
  (`dispatch_test.rs:145-193`); `REMOTE_IN_PROGRESS→TIMED_OUT→COMPENSATING→COMPLETED`
  (`dispatch_test.rs:201-229`).
- Never observed in any test: `DISPATCH_READY→REFUSED`, `REMOTE_IN_PROGRESS→BLOCKED`,
  `REFUSED→BLOCKED`, `TIMED_OUT→BLOCKED`, `COMPENSATING→BLOCKED`. No test ever passes
  `remediation_budget=0`, and no test drives a coordinator into an actual timeout.
  `DISPATCH_READY→REFUSED` additionally has **no production caller at all** — both
  `DispatchState::Refused` construction sites (`dispatch.rs:1574`, `engine.rs:1117`) fire only
  from `RESULT_RECEIVED`; that edge is lawful-by-declaration but dead code today.
- `DispatchState::Unknown` (16th) is `#[allow(dead_code)]` and never constructed by design —
  not a gap.

### 3.4 Prioritized coverage gap list (as ranked by Audit 2)

1. **CNG_R10 `IoRefused`** — zero negative-path test across ~130+ construction sites spanning
   nearly every module; the single most pervasive refusal in the codebase.
2. **CNG_R08 `Nondeterminism`** — zero negative-path test; doctrine-central regression catch for
   hidden nondeterminism, never fired in CI.
3. **`DispatchState::Blocked`** — unreachable in every test; the doctrine's "stuck, needs operator
   intervention" terminal has never been proven reachable in practice (4 of 22 lawful edges).
4. **CNG_R07 `RunnerMismatch`** — zero negative-path test; only the happy path is exercised.
5. **`DecompositionOutcome::NoBeneficialDecomposition`** — never forced+asserted in isolation; the
   lexicographic-argmin "single-actor wins despite an admissible split" law lacks a dedicated
   fixture proving it fires.
6. **`DISPATCH_READY→REFUSED` edge** — dead code, not merely untested; no production caller
   reaches it despite being declared lawful.

## 4. Determinism/Replay Risks (Audit 3)

Scope: `crates/cng/src/bench/{decomp,ipc,engine.rs,arazzo.rs}` + touched `dispatch.rs`/
`workday.rs`. Method: full read of new/changed lines plus grep sweep for
`SystemTime|Instant::now|std::time`, `rand|random|thread_rng`, `HashMap|HashSet`, `f32|f64`,
`process::id|thread::current|{:p}`, every hit traced to its consumer.

### Concrete risks found

1. **`crates/cng/src/bench/decomp/dispatch_bridge.rs`** — untested new bridge citing a
   nonexistent test file. `SubworkflowDispatchOutcome::polls_taken` (line 137, populated at
   line 245) is an arrival-time-dependent counter that is **not currently serialized anywhere**
   (no `Serialize`, never written to disk) — so it does not yet violate the digest invariant, but
   it is structurally the shape of bug this audit hunts for. The module doc (lines 32-33) cites
   `crates/cng/tests/cng_decompose_to_dispatch_integration.rs` as the exercising test — **that
   file does not exist**. Zero callers, zero unit tests, zero integration tests anywhere in the
   crate. Overclaiming (cited artifact is fiction) stacked on top of a completely unexercised
   real-time-shaped code path. (Independently corroborated by Audit 1's MISSING finding #2.)

2. **Multi-engine coordinator's concurrent collect path** — real timing variance exists
   (`engine_collect_remote`, `engine.rs:955`, via `RealTimeWait` at `engine.rs:1015`), but
   confirmed **safe by construction**: the collect loop never emits a per-poll observation into
   the obs graph, so arrival timing never reaches `receipt_chain_digest`. The one full-tree
   byte-identity test (`distributed_determinism_two_serialized_runs_byte_identical`,
   `cng_multi_engine.rs:443`) forces a fully serialized run, making `polls` deterministic by
   construction. The genuinely concurrent test
   (`multi_engine_concurrent_dispatch_execute_readmit`, `cng_multi_engine.rs:201`) deliberately
   asserts only counts/markers, not byte-identity, with an explicit doctrine comment — working as
   intended, not a gap. **Caveat**: `distributed_determinism_two_serialized_runs_byte_identical`
   was not among the tests independently confirmed passing this session; its design is sound on
   inspection but its actual pass/fail status this session is unconfirmed (audit was read-only,
   no `cargo test` run).

3. **`decomp/rules.rs` Datalog materialization** — order-independent by construction.
   `derive_edges` (`rules.rs:129-297`) decodes every triple into `BTreeMap`/`BTreeSet` fields
   (`rules.rs:254-296`), never a `Vec` in encounter order, so the reasoner's internal iteration
   order is structurally irrelevant to output. Confirmed clean, not merely assumed clean.

### Invariants checked and confirmed clean (no gap)

- No `SystemTime`/`Instant::now`/`std::time` in scope except the one lawful, documented
  `std::thread::sleep` behind `RealTimeWait`/`ThreadSleepWait` (`dispatch.rs:377`), structurally
  prevented from entering any digest.
- No `rand`/`random`/`thread_rng` anywhere in scope; all pseudo-randomness is seeded `splitmix64`.
- No raw `HashMap`/`HashSet` iteration feeding output in `decomp/`, `ipc/`, `engine.rs`,
  `arazzo.rs`. The one non-BTree collector in the touched-file set (`collect_ttl_paths_recursive`,
  `roles.rs:523`) has all four call sites sorting or only counting the result before use.
- No `f32`/`f64` anywhere in the audited scope.
- No PID/thread-id/pointer-address in output anywhere in scope.
- `EngineIdentity` confirmed seed-derived only, unit-tested
  (`engine_identity_is_deterministic_and_engine_distinct`, `engine_test.rs:57`).
- Arazzo digest-verify seam (`arazzo.rs:434-480`) is pure content-hash comparison; its return
  value is discarded by its only call site, so it cannot leak into any downstream receipt.
- The 5 IPC generators and shared renderer are clean: seed threaded via splitmix64, every
  object/init/goal/predicate/action list sorted+deduped before templating.
- `full_production_ready` (`workday.rs:644-665`) is a pure `BTreeMap`-fold combinator.

## 5. Documentation Internal-Consistency Issues (Audit 4)

Scope: all 36 existing ticket files (701–714, 720–731, 733, 734, 739–748), `RELEASE_CONTROL.md`,
`DOD_SIGNOFF.md`, `DOD_EVIDENCE_MAP.md`, `index.md`. Read-only; moving-target caveat applies (task
#24 and #30 were pending/in_progress at audit time while documents assert those tickets already
closed; actual `HEAD` had already moved past every citation in the doc set).

1. **Suppressed PARTIAL sub-claim.** `PROJ-701.md` line 3 states bare `Status: ALIVE`, but its own
   Evidence section admits: *"No standalone closed-shape-violation negative test for
   `pddl-strips-shapes.ttl` specifically was identified or run this session — PARTIAL on that
   narrower claim."* `RELEASE_CONTROL.md` line 283 and `index.md` line 55 both copy forward only
   the unqualified `ALIVE`, dropping the self-identified PARTIAL — unlike PROJ-711/728/729/742/745,
   whose PARTIAL/scoped qualifiers do survive into the summary tables.

2. **Unexplained ticket-number gap.** PROJ-715–719 are explicitly documented as "deliberately
   skipped... track separator; no tickets ever existed there" (`RELEASE_CONTROL.md` lines
   161, 251-253; `index.md` lines 48-49). No equivalent explanation exists for PROJ-732 or
   PROJ-735–738 (`grep -rn 'PROJ-732|PROJ-73[5-8]'` across both doc trees returns zero hits), yet
   the session's own task list has a completed task explicitly named "PROJ-735..738: isolated
   verification runs" — real work the docs never mention or account for.

3. **Bare ALIVE claims without a specific test/command citation.**
   - `DOD_SIGNOFF.md` line 39, §11: `ALIVE (structural) | exercised transitively... no per-row
     dedicated test cited individually` — self-admits no specific per-row test.
   - `DOD_SIGNOFF.md` line 32, §4: cites refusal codes/PROJ numbers but no test function names.
   - `DOD_SIGNOFF.md` line 34, §6: "exercised end-to-end by every `decompose()` test" — generic,
     no named test.
   - `DOD_SIGNOFF.md` line 41, §13: cites ticket numbers/type names, not test functions.
   - `DOD_EVIDENCE_MAP.md` line 20, PROJ-701 row: "exercised transitively," no test name.
   - `DOD_EVIDENCE_MAP.md` line 23, PROJ-704 row: generic wording, though the per-ticket file
     `PROJ-704.md` does cite `cargo test -p cng --features bench --test cng_decomp: 3/3 passed,
     0.18s` — that specific citation just isn't carried into the summary row.
   - By contrast, the CNG_R01–R25 ledger and most PROJ rows (705-710, 712-713, 720-725, 727-734,
     739-745) all cite exact test names or command+output, and unevidenced rows are correctly
     marked UNVERIFIED rather than ALIVE — that part of the file set is honest.

4. **Marker-name drift.** None found — genuine negative result. All marker names (LLM_CALLS_ZERO,
   ENGLISH_SUBGOALS_ZERO, CANNED_SUBGOALS_ZERO, DECOMPOSITION_DERIVED_PROVEN,
   SHARED_MEMORY_CROSSINGS_ZERO, DIRECT_ENGINE_BYPASSES_ZERO, MULTI_ENGINE_EXECUTION_PROVEN,
   ARAZZO_INTER_ENGINE_WORKFLOW_PROVEN, PLANNING_MARKER_MAP, DISTRIBUTED_MARKER_MAP,
   V26_7_10_PRODUCTION_READY) appear identically everywhere referenced; none of the three
   deprecated names DoD §16 itself calls out leak into the other three files. This reconciliation
   (PROJ-743) held.

5. **Status-vocabulary drift** (three different status-word sets in simultaneous use):
   - House rules (`.claude/rules/no-overclaiming.md`, `~/.claude/rules/no-overclaiming-rust.md`):
     ALIVE / PARTIAL / BLOCKED / MOCKED / REFUSED / UNSUPPORTED / UNVERIFIED (7 terms).
   - `DEFINITION_OF_DONE.md` line 5: ALIVE / PARTIAL / MOCKED / UNVERIFIED / BLOCKED / REFUSED /
     PLANNED (7 terms, drops UNSUPPORTED, adds PLANNED — PLANNED appears in neither house-rule
     file).
   - `RELEASE_CONTROL.md` §3 item 10 (lines 72-73): a third, explicitly "binding" 5-value set —
     ALIVE/PARTIAL/PLANNED/UNKNOWN/MOCKED, never reconciled against the DoD's 7-value set.
   - Within `RELEASE_CONTROL.md` itself, §8 (Phase 1) uses `ALIVE (doc)`; §9.1 (Phase 2) uses
     `IN PROGRESS`/`CLOSED (doc)`/`DONE (doc)` for the same semantic claim.
   - `DOD_SIGNOFF.md` line 1 and `DOD_EVIDENCE_MAP.md` line 4 self-declare `Status: FINAL (doc)`,
     while their own governing ticket `PROJ-748.md` line 3 declares `Status: DONE (doc)` — two
     different words for the same completion event.
   - None of DONE, CLOSED, FINAL, IN PROGRESS, CUT, UNKNOWN appear in either house-rule vocabulary
     or the DoD's own quick-reference list, yet all are used as statuses throughout the set.

6. **Stale HEAD/commit-state citation.** All 36 Phase-2 ticket files plus `RELEASE_CONTROL.md`
   §9/§9.1, `DOD_SIGNOFF.md`, `DOD_EVIDENCE_MAP.md`, `index.md` state HEAD is still `40f6020`.
   Actual `git rev-parse HEAD` at audit time was `7259f38`, two commits ahead
   (`40f6020 → 59cde6e → 31c236f → 7259f38`), one of which (`31c236f`) is the exact commit the docs
   themselves cite as an already-closed interim milestone. There is also a direct disagreement
   between the two governing docs about *when* that closure happened: DoD §1 implies an earlier
   session; RELEASE_CONTROL §8's own header implies the same session — and per RELEASE_CONTROL's
   own tie-breaking rule, that reading should win, contradicting DoD §1. The general "git status
   not clean" characterization still holds; only the specific pinned hash is wrong.

7. **Task-tracker vs. document-claim mismatch.** Session task list: task #24
   (`PROJ-731: final release closure`) = pending; task #30 (`PROJ-746..748: doc closure`) =
   in_progress; task #31 (`Phase 6: commit`) = pending. Documents (`PROJ-731.md`, `PROJ-746.md`,
   `PROJ-747.md`, `PROJ-748.md`, `RELEASE_CONTROL.md` §9.1, `index.md`) all assert these are
   already `CLOSED (doc)`/`DONE (doc)`.

## 6. Safety/Leak/Clean-Room Findings (Audit 5)

Scope: `src/bench/decomp/`, `src/bench/ipc/`, `src/bench/engine.rs`, `crates/cng/tests/
cng_multi_engine.rs`, `crates/cng/queries/`, `crates/cng/rules/`, plus dependency graph for
`pddl-index`.

### 6.1 Panics — `.unwrap()`/`.expect(`/`panic!(`/`unreachable!(`

Zero hits in production code across the full scope. Fallible operations consistently route
through `.map_err(|e| CngRefusal::…)?` (spot-checked `decomp/mod.rs:851-857`). Test-only modules
(`decomp_test.rs`, `engine_test.rs`) are correctly `#[cfg(test)]`-gated and out of scope per
`no-overclaiming-rust.md`. **No risk found.**

### 6.2 TODO/FIXME/`unimplemented!()`/`todo!()`

Zero hits in `src/bench/decomp/`, `src/bench/ipc/`, `src/bench/engine.rs`, `crates/cng/queries/`,
`crates/cng/rules/`. Clean.

### 6.3 Process/resource leaks — `crates/cng/tests/cng_multi_engine.rs`

Three real-process spawn sites via `spawn_engine()` (line 69, returns bare `std::process::Child`,
no `Drop`/kill-on-drop guard exists anywhere in the file or in `src/bench/*.rs`):

| Spawn site | Test | Reaped by | Cleanup path |
|---|---|---|---|
| `child_h` (line 211) | `multi_engine_concurrent_dispatch_execute_readmit` | `wait_with_output()` (line 221) | happy-path only |
| `child_m` (line 212) | same | `wait_with_output()` (line 222) | happy-path only |
| `child` (line 331) | `g13_crash_resume_verifies_chain_and_completes` | `child.kill()`+`.wait()` (lines 355-356) | happy-path only |

Leak windows:
- Lines 211-222: line 219's `engine_collect_remote(...).expect("collect phase")` panics on `Err`
  and unwinds before the reap at 221-222 — both children orphaned for up to their bound
  (`--max-polls 3000 --poll-wait-ms 20` ≈ 60s).
- Lines 331-356: `assert!(saw_ledger, ...)` at line 354 panics before `child.kill()` at 355 if the
  ledger file never appears within the watch window — leaves a live engine process bound for up
  to ~500s (`--max-polls 10000 --poll-wait-ms 50`), unreaped.

`engine_dispatch_remote`/`engine_collect_remote` are confirmed in-process (no `Command::new`/
`.spawn()` in `src/bench/mod.rs` or `src/bench/engine.rs`) — no spawn/leak surface of their own.
`cng_ipc_corpus.rs` spawns nothing. **Verdict: all three spawn sites are cleanup-on-happy-path
only; a panic on the intervening assert/expect between spawn and wait/kill temporarily orphans a
real child process. Not catastrophic given the process bounds, but a genuine gap relative to
"reaped on every exit path."**

### 6.4 Clean-room provenance — 5 IPC domain generators

Read in full: `blocksworld.rs`, `grippers.rs`, `barman.rs`, `termes.rs`, `tyreworld.rs`.

- **blocksworld.rs**: predicate names (`on-table`/`arm-empty`) diverge from canonical IPC tokens
  (`ontable`/`handempty`) — consistent with independent rewrite. Clean.
- **grippers.rs**: predicate names diverge from canonical (`at-robot`/`at-ball` vs.
  `at-robby`/`at`); adds typing guards explicitly justified in-comment. Clean.
- **barman.rs**: doc comment self-discloses a drastic simplification vs. the real numeric-fluent-
  heavy IPC domain — bears little resemblance beyond shared theme. Clean.
- **termes.rs**: doc comment self-discloses a 1D-line reduction vs. the real 3D/numeric-height
  domain. Clean.
- **tyreworld.rs**: flagged for a second human look, not because of found evidence of copying —
  action-name vocabulary (`open-boot`, `fetch`, `loosen`, `jack-up`, ...) is generic real-world
  procedural English that a first-principles author would independently converge on, and the
  reduction (single-hub/two-wheel) is far simpler than competition instances. Recommended as the
  first file to double-check if doubt ever arises, purely because it's the least distinguishable
  from "could have been copied" of the five, not because anything suspicious was found.

No file showed leftover PDDL typing syntax, domain-file header artifacts, or exact-match
precondition/signature combinations beyond what each doc comment plainly derives from
first-principles physical reasoning.

### 6.5 Dependency/license audit — `pddl-index`

- `crates/cng/Cargo.toml:50`: `pddl-index = { path = "../pddl-index", optional = true }`, pulled
  only by the `bench` feature; default surface unaffected.
- `crates/pddl-index/Cargo.toml`: single dependency, `wasm4pm-compat` (local path, no
  dev-dependencies, no registry/crates.io dependencies).
- `wasm4pm-compat` is not new to the workspace — already declared at workspace root, already
  `[patch.crates-io]`-pinned, already a dependency of `praxis-core` and `praxis-graphlaw`.
  `pddl-index` introduces **zero new registry/network dependencies**.
- License: `pddl-index` = `MIT OR Apache-2.0`, matches `cng` exactly. **Compatible.**

## 7. Final Prioritized Punch List

Ranked highest to lowest priority: load-bearing correctness/safety gaps first, cosmetic/doc-polish
gaps last. Original entries are preserved verbatim (fix-forward, not rewritten); each of items
1-10 now carries a **Closure status (8-agent wave, this session)** line added below its original
"Suggested next action," citing the closing agent's own command+output. Items 11-20 were out of
this wave's scope (7 parallel Phase-1 agents targeted exactly items 1-10, per
`docs/jira/v26.7.10/tickets/index.md`'s companion planning doc); they are unchanged from the
original audit except where noted.

1. **CNG_R10 `IoRefused` has zero negative-path test across ~130+ construction sites** (Audit 2,
   gap #1). *Suggested next action: add a fixture that forces a real I/O failure (permissions or
   missing-file race) and asserts `.code() == "CNG_R10"` on at least one representative call site.*
   **Closure status (8-agent wave, this session): CLOSED, one representative call site.** New
   file `crates/cng/tests/cng_io_refused_negative.rs`
   (`import_artifacts_missing_dir_refuses_cng_r10_io_refused`) points the public
   `cng::pipeline::import_artifacts` (`pipeline.rs:74`) at a directory computed but never
   `create_dir_all`'d, forcing `fs::read_dir` to fail deterministically
   (`io::ErrorKind::NotFound`) on every platform; asserts `.code() == "CNG_R10"` and that the
   message names both the failing path and `"cannot read artifact dir"`. Command:
   `CARGO_TARGET_DIR=target/agent-r10 cargo test -p cng --test cng_io_refused_negative --
   --nocapture` → `test import_artifacts_missing_dir_refuses_cng_r10_io_refused ... ok; test
   result: ok. 1 passed; 0 failed`. Scope: proves the mechanism
   (`fs::read_dir`/`fs::read_to_string` → `.map_err(IoRefused)` → `?`) fires correctly at one
   load-bearing site; the other ~129 construction sites across `bench/*.rs`/`pipeline.rs`/
   `bench/decomp/*.rs` remain UNVERIFIED individually — structurally identical, not
   independently exercised.
2. **CNG_R08 `Nondeterminism` has zero negative-path test; the refusal arm has never fired**
   (Audit 2, gap #2). *Suggested next action: construct a same-seed rerun with deliberately
   injected drift and assert the refusal fires with the exact code.*
   **Closure status (8-agent wave, this session): INVESTIGATED — correctly unreachable-by-design
   through the public API surface exposed today; no test file created.** Both construction sites
   (`main.rs:501`, a private CLI-only verb never exported by `lib.rs`; `workday.rs:1204`, a
   `pub(super)` end-of-day replay loop reached only via the single public `workday()` entry
   point) compare two manufacture passes executed back-to-back inside one synchronous call, with
   no caller-visible seam between them for an external test to inject drift without either (a)
   causing a genuine determinism bug in the surrounding pipeline — out of scope, no
   production-code edits permitted — or (b) a timing-dependent filesystem race, which is flaky
   and proves nothing. Empirical confirmation:
   `CARGO_TARGET_DIR=target/agent-r08 cargo test -p cng --features bench --lib
   workday_same_seed_twice_is_byte_identical -- --nocapture` → `ok`, stays silent through the
   guarded replay loop, consistent with the refusal never having fired historically. Per the
   task's explicit guidance to report rather than force an artificial trigger, `CNG_R08` remains
   UNVERIFIED (dedicated negative test) — but the open question changes from "untried" to
   "unreachable by design given the current public API," a materially different, now-evidenced
   finding.
3. **`dispatch_bridge.rs`'s two `pub` bridge functions have zero callers and cite a test file
   (`cng_decompose_to_dispatch_integration.rs`) that does not exist** (Audit 1 MISSING #2, Audit 3
   finding #1 — independently corroborated). *Suggested next action: either write the cited
   integration test and wire a real caller, or correct the doc comment and flag the module as
   unexercised/dead code explicitly.*
   **Closure status: STALE, already corrected before this wave started.** Not touched by any of
   the 7 Phase-1 agents. Re-verified this session:
   `crates/cng/tests/cng_decompose_to_dispatch_integration.rs` exists (13,164 bytes) with
   `kitchen_decomposition_splits_into_helper_and_main` and
   `decomposed_subworkflows_dispatch_to_real_engines_and_are_admitted` (PROJ-749, 2/2 passed
   per `DOD_SIGNOFF.md`/`DOD_EVIDENCE_MAP.md`, second synthesis round predating this audit's own
   writing). See §2's inline STALE correction above.
4. **Three process-leak windows in `cng_multi_engine.rs`** where a panic on an intervening
   `.expect()`/`assert!()` between spawn and reap orphans a live child engine process for up to
   ~60s/~500s (Audit 5 §6.3). *Suggested next action: wrap each `spawn_engine()` result in a
   kill-on-drop guard so reap happens on every exit path, not just the happy path.*
   **Closure status (8-agent wave, this session): CLOSED, all three sites.** New type
   `EngineGuard(Option<Child>)` in `crates/cng/tests/cng_multi_engine.rs` wraps all three
   `spawn_engine()` call sites (`child_h`/`child_m` at lines 216-217,
   `multi_engine_concurrent_dispatch_execute_readmit`; `child` at line 336,
   `g13_crash_resume_verifies_chain_and_completes`). `Drop` does a best-effort `child.kill()`
   (a documented Unix no-op on an already-`wait()`-ed child); `Deref`/`DerefMut` preserve the
   existing `.kill()`/`.wait()` call sites unchanged; `wait_with_output(self)` empties the
   `Option` first so the happy path's `Drop` is a no-op. Diff: 66 insertions, 3 deletions, one
   file only. Command: `CARGO_TARGET_DIR=target/agent-leak cargo test -p cng --features bench
   --test cng_multi_engine -- --test-threads=1` → `test result: ok. 7 passed; 0 failed`,
   27.63s; `ps aux | grep "cng engine serve"` empty after the run (no orphans); `rustfmt --check`
   on the file exits 0.
5. **`DispatchState::Blocked` is unreachable in every test** — 4 of 22 lawful edges
   (`REMOTE_IN_PROGRESS→BLOCKED`, `REFUSED→BLOCKED`, `TIMED_OUT→BLOCKED`,
   `COMPENSATING→BLOCKED`) are declared lawful but never driven by real data (Audit 2, gap #3).
   *Suggested next action: add a test with `remediation_budget=0` and a forced coordinator
   timeout to prove the fail-safe path is reachable in practice.*
   **Closure status (8-agent wave, this session): PARTIAL — one of four dead edges now proven
   reachable.** New test `deadline_expiry_with_zero_remediation_budget_reaches_blocked`
   (`crates/cng/src/bench/dispatch_test.rs`) forces `deadline_ticks = 0` (timeout before any
   loopback consequence lands) with `remediation_budget = 0`, proving `TimedOut` advances
   straight to `Blocked` and skips `remediate()`/`Compensating`/`Completed` entirely — the exact
   ledger trajectory `("TIMED_OUT", "BLOCKED")` is asserted, not just a loose status check.
   Command: `CARGO_TARGET_DIR=target/agent-dispatchstate cargo test -p cng --features bench
   --lib bench::dispatch::dispatch_test -- --test-threads=4` → `test result: ok. 15 passed; 0
   failed; ... 62 filtered out` (all 15 tests in the module, including the 4 preexisting
   transition-table tests). **Remaining open**: `REMOTE_IN_PROGRESS→BLOCKED`,
   `REFUSED→BLOCKED`, `COMPENSATING→BLOCKED` are still never driven by any test — this closes
   the `TIMED_OUT→BLOCKED` edge specifically (the edge the original suggested action named),
   not all four.
6. **CNG_R07 `RunnerMismatch` has zero negative-path test** across 5 construction sites guarding
   the conformance check (Audit 2, gap #4). *Suggested next action: construct a tape/model pair
   engineered to disagree and assert the exact refusal.*
   **Closure status (8-agent wave, this session): CLOSED, 2 of 5 construction sites.** New file
   `crates/cng/tests/cng_runner_mismatch_negative.rs`, two tests through the real
   `cng::runner::validate_run` path (no mocking): `model_leaf_label_disagrees_with_tape_op_
   refuses_cng_r07` (site: `runner.rs:186`, a model leaf's label disagreeing with the tape op at
   the same index) and `cyclic_order_relation_refuses_cng_r07_via_compile_powl_kahn_check`
   (site: `runner.rs:118`, a genuine 2-cycle `PartialOrder` caught by the real published
   `bcinr-powl` runtime's own Kahn-algorithm acyclicity check, mirroring that crate's own unit
   test `kahn_check_rejects_non_loop_cycle`). Both assert `refusal.code() == "CNG_R07"` and
   message content naming the specific failure. Command: `CARGO_TARGET_DIR=target/agent-r07
   cargo test -p cng --test cng_runner_mismatch_negative` → `test result: ok. 2 passed; 0
   failed`; re-run confirmed deterministic. **Remaining open**: the other 3 construction sites
   (op-count mismatch at `runner.rs:177`, incomplete-scheduler-firing at `runner.rs:269`,
   order-violated-at-runtime at `runner.rs:279`) remain UNVERIFIED individually.
7. **`DISPATCH_READY→REFUSED` is dead code, not merely untested** — declared lawful, verified by
   the predicate table, but no production caller reaches it (Audit 2, gap #6). *Suggested next
   action: either add the missing pre-dispatch refusal caller or mark the table entry as
   aspirational pending implementation.*
   **Closure status (8-agent wave, this session): INVESTIGATED — correctly-vestigial-with-
   doc-correction, not wired.** Confirmed both `DispatchState::Refused` construction sites
   (`dispatch.rs`, `engine.rs`) fire only from `RESULT_RECEIVED`. The only pre-dispatch check,
   `CNG_R15 DispatchContractIncomplete`, runs and can already fail *before* the
   `ARAZZO_RENDERED → DISPATCH_READY` transition happens, so a structurally incomplete contract
   never reaches `DISPATCH_READY` at all — there is nothing left to refuse once that state is
   entered, and no authority/policy/budget/engine-registry check exists anywhere in the module
   that could ground a new one without inventing business semantics the task explicitly said not
   to force. Action taken: left the lawful-transition table and drift test untouched (the edge
   remains correctly declared lawful, matching the `disp:DispatchState` vocabulary mirror) and
   corrected the type-level doc comment on `DispatchState` (`dispatch.rs` ~lines 104-150) to flag
   `DISPATCH_READY → REFUSED` as "declared, currently unreached," with the reasoning spelled out
   under a new subsection. Verified via the same `dispatch_test.rs` run as item 5 above (15
   passed), `cargo fmt -p cng -- --check` (clean on both touched files), and `cargo check -p cng
   --features bench --lib` (clean compile, only pre-existing unrelated `dead_code` warnings).
8. **OpenAPI/AsyncAPI documents are not wired into the dispatch path** despite doc claims that
   HTTP binding is "declared via generated, digest-recorded" documents (Audit 1 MISSING #1).
   *Suggested next action: either wire a digest-verify seam analogous to the Arazzo leg, or narrow
   the doc claim to "Arazzo only, OpenAPI/AsyncAPI templates generated but not yet consumed."*
   **Closure status: CLOSED — real digest-verify wiring built (Option A), superseding this
   wave's own Option-B investigation.** The 8-agent wave's assigned agent investigated and
   chose Option B (narrow the DoD claim) for the reasons originally recorded here (no
   documented seam for OpenAPI/AsyncAPI in `packs/arazzo-pack/README.md`, risk of inventing an
   unjustified consumer). The user explicitly rejected that fallback and a dedicated override
   agent built real support instead: `crates/cng/src/bench/api_docs.rs`'s
   `verify_api_docs_render_digest`/`verify_api_docs_render_digest_if_present(project_root)`,
   wired into `engine_serve` (`engine.rs:589`, before the poll loop) — an engine now refuses to
   start (`CNG_R11 AuditMismatch`) if its own `generated/engine-openapi.yaml`/
   `generated/engine-asyncapi.yaml` are stale/tampered relative to the `.ggen-v2/receipt.json`
   digest, with absence handled honestly (`Ok(None)`, no false refusal when an engine root has
   no pre-generated capability docs). 6 new tests in `api_docs_test.rs`, including
   `engine_serve_refuses_cng_r11_when_api_doc_render_tampered` (proving `engine_serve` itself
   returns the refusal, not just a helper function in isolation). Verified:
   `cargo test -p cng --features bench --lib` → 77 passed, 0 failed
   (`CARGO_TARGET_DIR=target/agent-openapi-override`). `DEFINITION_OF_DONE.md` §20 item 1 now
   states Arazzo and OpenAPI/AsyncAPI both ALIVE for digest-verify (schema validation and live
   HTTP/broker binding remain the genuinely open boundary, unchanged). `RELEASE_CONTROL.md`'s
   pointer to §20 for the current text remains accurate and needs no further edit.
9. **Stale HEAD/commit-state citation repeated across 36+ files** (`40f6020` cited; actual HEAD
   was `7259f38`, two commits ahead) (Audit 4, finding 6). *Suggested next action: re-run the
   citation sweep after the next commit and update all citing files in one pass.*
   **Closure status (8-agent wave, this session): CLOSED.** Ancestry verified first
   (`40f6020..1f3f9bc` = 5 commits: `59cde6e → 31c236f → 7259f38 → 285ac3a → 1f3f9bc`, confirming
   `1f3f9bc` is the correct current-HEAD citation). 40 files updated (`40f6020`→`1f3f9bc`):
   `DOD_SIGNOFF.md` (3 occurrences), `index.md` (3 of 8 — the 5 `CLOSED (`40f6020`)` table rows
   for PROJ-601..605 correctly left as historical fact, not a live-HEAD claim), and 37
   `PROJ-7xx.md` ticket files. 9 files correctly left untouched as pure historical citations
   (`PRD.md`, `ARD.md`, PROJ-601..605.md, PROJ-607.md); this document (`GAP_AUDIT.md`) was
   deliberately deferred to this synthesis pass, since its own two `40f6020` mentions (its
   §5 finding 6 narrative, quoting what other docs said at audit time) are diagnostic
   narrative, not a live-HEAD assertion, and pair with a second stale hash (`7259f38`) that
   would need its own reconciliation if edited in isolation — left as historical audit record
   per fix-forward discipline; this section's closure annotations are the correction. Spot-check
   re-verified this session: `DOD_SIGNOFF.md` lines 33/83/296, `PROJ-701.md` line 3,
   `PROJ-748.md` lines 3/37, and `index.md` lines 19/120/175 all now cite `1f3f9bc`, while
   `index.md` lines 25-29's CLOSED-table rows correctly retain `40f6020` as history.
10. **`DecompositionOutcome::NoBeneficialDecomposition` never forced+asserted in isolation** — the
    lexicographic-argmin "single-actor wins over an admissible split" law lacks a dedicated
    fixture (Audit 2, gap #5). *Suggested next action: construct a domain where a split is
    admissible but not beneficial and assert the exact outcome variant.*
    **Closure status: STALE, already corrected before this wave started.** Not touched by any of
    the 7 Phase-1 agents. See §3.2's inline STALE correction above —
    `cng_decomp_negative_corpus_completeness.rs:188-195` forces the exact variant by name
    (predates this wave, second synthesis round).
11. **§18 negative-corpus items 5–6 not forced by their named mechanism** (mutex-saturated goals;
    admissible-but-never-beneficial splits) (Audit 1 PARTIAL/UNCLEAR). Noted as already
    in-progress under session task #32. *Suggested next action: land the fixtures task #32 is
    already targeting; re-audit once complete.*
    **Not in this wave's scope.** Sub-part status, re-verified this session for accuracy only
    (not acted on by any Phase-1 agent): item 6 (admissible-but-never-beneficial) is the same
    fixture as punch-list item 10 above — ALIVE, forced, predates this wave. Item 5
    (mutex-saturated goals) remains open: `DOD_SIGNOFF.md` §18 still records it as "ALIVE
    (adjacent scenario)" via `single_atom_goal_yields_no_admissible_decomposition`, which proves
    the typed-outcome mechanism but is not a literal "mutex-saturated" fixture by that name.
12. **§14 item 4's "fail the run with nonzero exit" half is untested** — only the marker-query
    computation is proven directly; the CLI/process-exit path is not (Audit 1 PARTIAL/UNCLEAR).
    *Suggested next action: run the hostile fixture through the actual CLI entry point and assert
    nonzero exit.*
    **Not in this wave's scope.** No Phase-1 agent was assigned this item; status unchanged from
    the original audit.

Items 13-20 (cosmetic/doc-polish, the "roughly 29" non-load-bearing gaps referenced in §1) were
explicitly out of this wave's scope and remain exactly as originally audited below — none of the
8 agents touched them.

13. **Suppressed PARTIAL sub-claim rounds up to ALIVE** in `RELEASE_CONTROL.md` and `index.md` for
    PROJ-701, dropping the ticket's own self-identified narrower gap (Audit 4, finding 1).
    *Suggested next action: propagate the PARTIAL qualifier into both summary tables, consistent
    with how PROJ-711/728/729/742/745 already do it.*
14. **Bare ALIVE claims without a specific test/command citation** in `DOD_SIGNOFF.md` (§4, §6,
    §11, §13) and `DOD_EVIDENCE_MAP.md` (PROJ-701, PROJ-704 rows) (Audit 4, finding 3). *Suggested
    next action: add the specific test-name citation each row is missing (PROJ-704's is already
    available in `PROJ-704.md` and just needs to be copied forward).*
15. **Status-vocabulary drift across three incompatible term sets** (house rules' 7-term set vs.
    DoD's different 7-term set vs. RELEASE_CONTROL's "binding" 5-value set), plus four different
    words (`ALIVE(doc)`/`DONE(doc)`/`CLOSED(doc)`/`FINAL(doc)`) used interchangeably for "doc
    complete" (Audit 4, finding 5). *Suggested next action: pick one vocabulary as canonical and
    update the other two doc-closure artifacts to match.*
16. **Unexplained ticket-number gap: PROJ-732, PROJ-735–738** — no equivalent explanation to the
    documented PROJ-715–719 skip, despite real completed session work under those numbers (Audit
    4, finding 2). *Suggested next action: add the same skip/reassignment explanation that
    PROJ-715-719 already has, or retro-file the missing ticket stubs.*
17. **Task-tracker vs. document-claim mismatch** — tracker shows pending/in_progress where
    multiple documents already assert CLOSED/DONE (Audit 4, finding 7). *Suggested next action:
    reconcile once the doc-closure session actually completes; do not treat current doc claims as
    final until the tracker agrees.*
18. **`tyreworld.rs` recommended as the first clean-room file to re-check if doubt ever arises**
    — no evidence of copying found, flagged only because its vocabulary is the least
    distinguishable from generic real-world procedural language of the five domains audited
    (Audit 5 §6.4). *Suggested next action: none required now; keep as a standing note for any
    future clean-room re-review.*
19. **Doc-wide `Status: PLANNED` labels are stale relative to code** across DoD sections 2, 4-14,
    17-19 — most described mechanisms have real, tested implementations (Audit 1, document-level
    observation). *Suggested next action: once `RELEASE_CONTROL.md` cites a command+output per
    clause (the doc's own bar), flip the corresponding status column entries.*
20. **`cng_multi_engine.rs` test count (7) exceeds DoD's cited "6/6"** due to a test added
    concurrently during the audit — flagged as a timing artifact (Audit 1, test-count
    discrepancy). *Suggested next action: re-count at the next stable checkpoint and update the
    citation.*

## References

- Source documents read (not modified) for this audit: `docs/releases/v26.7.10/{RELEASE_CONTROL,
  DEFINITION_OF_DONE,DOD_SIGNOFF,DOD_EVIDENCE_MAP}.md`, `docs/jira/v26.7.10/tickets/index.md` and
  all PROJ-7xx ticket files.
- Code scope: `crates/cng/src/{powl.rs,bench/{workday,workday_test,dispatch,dispatch_test,engine,
  engine_test,arazzo,arazzo_test}.rs,bench/decomp/,bench/ipc/}`, `crates/cng/tests/`,
  `crates/cng/queries/markers/*.rq`, `crates/cng/rules/{decomp.dl,decomp-resources.dl}`,
  `crates/cng/ontologies/{pddl-strips.ttl,arazzo.ttl}`, `packs/arazzo-pack/`.
- House rules governing the status vocabulary cited in §5: `.claude/rules/no-overclaiming.md`,
  `~/.claude/rules/no-overclaiming-rust.md`.

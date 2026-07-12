# Crown-Frontier Status — v26.7.12

Milestone: v26.7.12 ("Design for Combinatorial Maximalism"). This document is the synthesized,
evidence-grounded status of the two crown-witness paths after this repair pass. It computes edge
contiguity from the **adversarial verification results** (which override the original wiring
claims wherever they conflict), not from any agent's self-report. Every verdict below is tied to a
command run this session or a `file:line` read this session.

Scope note: this is a status/audit artifact. It does not modify `tickets/index.md` or
`ADVERSARIAL_DOD.md` (reserved for the orchestrating session).

## Executive summary (the headline booleans)

| Marker | Value | Why |
|---|---|---|
| `LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` | **false** | 10 of 11 LOCAL edges are `REAL_EDGE` (updated post-`217dc37d`); breaks at `F21 -> F25`. |
| `EXTERNAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` | **false** | 7 of 16 EXTERNAL edges are `REAL_EDGE`; first sub-real at `F10 -> F12`, hard break at `F15 -> F16`. |
| `OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` | **false** | Requires **both** witness markers true; neither is. |

The shared prefix `F02 -> F03 -> F08 -> F09 -> F10` plus the LOCAL tail through
`F11 -> F18 -> F19 -> F02(re-admit) -> F24 -> F21` is a genuine, real production-caller chain (10
edges) driven by one function, `crown_local::drive_local_witness_prefix` (commits `3322bf2d`,
`d60f2036`, `eeca952a`, `66d8732e`, `0815680a`, `217dc37d` -- this status doc was written after the
first of these and is being brought current now). The EXTERNAL tail adds three more real edges
(`F12 -> F13 -> F14 -> F15`) via `crown_external::drive_external_witness_tail`. Everything past
`F21` on the LOCAL tail (just `F21 -> F25` remains), and everything past `F15` on the EXTERNAL
tail, is not yet a real edge.

## Witness topologies (as given; not reinterpreted)

Shared prefix (both witnesses): `F02 -> F03 -> F08 -> F09 -> F10`.

LOCAL tail: `F10 -> F11 -> F18 -> F19 -> F02(re-admit) -> F24 -> F21 -> F25` (11 edges total).

EXTERNAL tail: `F10 -> F12 -> F13 -> F14 -> F15 -> F16 -> F18 -> F20 -> F02(re-admit) ->
F15(AIR transition) -> F21 -> F24 -> F25` (16 edges total).

`REAL_EDGE` bar (as defined for this pass): a real production (non-`#[cfg(test)]`) caller passes
the actual consequence of the upstream family into the actual downstream mechanism. A test helper
calling both sides is **not** a `REAL_EDGE` (that is `TEST_ONLY`). A caller that threads the data
but leaves one required semantic sub-property unsatisfied is `PARTIAL_REAL_EDGE`.

## LOCAL witness — per-edge classification

Production caller for edges 1-10: `crown_local::drive_local_witness_prefix`
(`crates/multifractal-workflow/src/crown_local.rs`). Verified this session (post-`217dc37d`): the
driver calls `admit_observation`, `contract` with the exact admitted bytes, `run_pipeline`, F09's
`manufacture_and_bind_child` (reaches F10 internally), then `geometry_to_local_ast` ->
`dispatch_local_execution_via_broker` (F11 -> F18) -> real `resolve_hook_for_action` (F18 -> F19)
-> a second real `admit_observation` call over a synthesized actuation-consequence observation
(F19 -> F02 re-admit) -> a real OTel span (admit -> project -> insert) run through F24's real
`run_construct` (F02(re-admit) -> F24) -> real `admit_child_and_evaluate` over F09's own
`growth.closure`/`growth.child_socket`, evidenced by a real SHACL check over F24's receipt head
(F24 -> F21), gated end to end.
`crown_local_prefix_drives_f02_through_f21_end_to_end` (`crown_local_test.rs`) exercises the whole
chain and asserts on every stage's real output, including `HookResolutionState::Replayable`, a
second `AdmissionState::Admitted` receipt distinct from the first, F24's real
`ConstructProfile::OtelToOcel` outcome with non-empty `ocel_quads`/`receipt_quads`/`receipt_head`,
and the manufactured child's real `ChildCompletionState::Admitted` transition.

| # | Edge | Verdict | Evidence (this session) |
|---|---|---|---|
| 1 | F02 -> F03 | `REAL_EDGE` | crown_local.rs; adversarial verdict CONFIRMED. |
| 2 | F03 -> F08 | `REAL_EDGE` | gated on `Plannable`; `receipt_head` salts F08 `case_id`; CONFIRMED. |
| 3 | F08 -> F09 | `REAL_EDGE`* | control-gated + provenance-bound + tape-consistency-checked; CONFIRMED. *Disclosed: F09 re-plans the shared PDDL rather than byte-ingesting F08's `Pddl8Tape` (no residual-goal extractor exists). Honest, not smuggled. |
| 4 | F09 -> F10 | `REAL_EDGE` | `manufacture_and_bind_child` -> `f10_powl_geometry::manufacture_powl_v2` (f09_mfw_growth.rs:825 -> :774); CONFIRMED. |
| 5 | F10 -> F11 | `REAL_EDGE` | (commit `d60f2036`) `geometry_to_local_ast(&growth.geometry.root)` now called from `drive_local_witness_prefix`, a non-test `pub fn`; was `TEST_ONLY` when this doc was first written. |
| 6 | F11 -> F18 | `REAL_EDGE` | (commit `d60f2036`) `dispatch_local_execution_via_broker` now called from the same driver, real `BrokerReceipt` returned with real `consequence_hash_hex`/`receipt_hash_hex`; was `TEST_ONLY`. |
| 7 | F18 -> F19 | `REAL_EDGE` | (commit `eeca952a`) `resolve_hook_for_action` called `?`-gated on a real `broker_receipt`, against F08's real bound action and the real admitted hook-pack catalog; was `MISSING_EDGE`. |
| 8 | F19 -> F02 (re-admit) | `REAL_EDGE` | (commit `66d8732e`) `admit_observation` called a second time, `?`-gated on real `hook_resolution`, over a synthesized actuation-consequence observation under a distinct local-runtime principal (`actuation_source_id`); was `MISSING_EDGE`. |
| 9 | F02 -> F24 | `REAL_EDGE` | (commit `0815680a`) the re-admitted consequence becomes a real `cng::otel_rdf::OtlpSpan` (`trace_id`/`span_id` = F18/F19's own receipt hashes, `parent_span_id` = the re-admission's own output receipt hash) run through `f24_ocel_construct::run_construct("otel-to-ocel", ...)`; was `MISSING_EDGE`. Never calls F24's own `idempotency_gate` (that's a distinct, orthogonal L7 capability `run_construct` does not require as a precondition — confirmed by reading `run_construct`'s body — so this is a full `REAL_EDGE`, not `PARTIAL_REAL_EDGE`). |
| 10 | F24 -> F21 | `REAL_EDGE` | (commit `217dc37d`) `admit_child_and_evaluate(&mut growth.closure, &growth.child_socket, &evidence)` -- `growth.closure`/`growth.child_socket` are F09's own real output built for exactly this purpose (`GrowthOutcome::child_socket`'s own doc comment). Evidence is a real, non-vacuous `Validator::validate` result: asserts F24's actual `ocel_outcome.receipt_head` and checks a `sh:minCount 1` constraint that is genuinely evaluated (target class matched by a real individual), not a vacuous shape and not a fabricated `conforms:true`; was `MISSING_EDGE`. |
| 11 | F21 -> F25 | `MISSING_EDGE` | Not wired. `f25_receipts_replay::chaos_gate::admit_for_replay` is an audited-honest `NotYetImplemented` refusal (HAND_WRITE_REQUIRED, not corruption) -- but note this blocks only the *chaos-gate* path, not `f25_receipts_replay::run` itself (F25's own real top-level entry point, not yet audited for a composable non-chaos-gate path into it this pass). |

`FIRST_LOCAL_BROKEN_EDGE` = **`F21 -> F25`** (updated -- edges 5-10 closed since this doc was first
written; see commits `d60f2036`, `eeca952a`, `66d8732e`, `0815680a`, `217dc37d`). This is the last
remaining LOCAL edge; closing it makes `LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH = true`.

## EXTERNAL witness — per-edge classification

Edges 1-4 are the same shared prefix (all `REAL_EDGE`, above). Production caller for edges 5-8:
`crown_external::drive_external_witness_tail`
(`crates/multifractal-workflow/src/crown_external.rs`). Verified this session: the driver calls
`resolve_external_cut_at` -> `project_and_compile` -> `f14_wasm4pm_arazzo::compile` ->
`air_program_to_bridge_workflow` (crown_external.rs:216). Confirmed it **stops at building F15's
input**: the actual `call_air_core_bridge` is deliberately not invoked in the production path
(crown_external.rs:164, "not called here"); the live Erlang round trip runs only in the gated test.

| # | Edge | Verdict | Evidence (this session) |
|---|---|---|---|
| 1-4 | F02..F10 | `REAL_EDGE` x4 | Shared prefix via `drive_local_witness_prefix` (above). |
| 5 | F10 -> F12 | `PARTIAL_REAL_EDGE` | F10's real geometry becomes the `ExternalCut.region` (crown_external.rs:257), but F10's `build_powl_geometry` **never synthesizes** `Powl::ExternalCut` (grep: only in `to_turtle` emit arm, f10_powl_geometry.rs:897). The cut boundary is declared by the driver on top of F10's geometry, not emitted by F10. Adversarial verdict: PARTIAL, appropriately hedged. |
| 6 | F12 -> F13 | `REAL_EDGE` | `resolve_external_cut_at(&model, child(1))` gates `project_and_compile` over the identical `model`; refuses `ExternalCutTypeMismatch` before F13 runs; CONFIRMED. |
| 7 | F13 -> F14 | `REAL_EDGE` (byte-level) | F13's manufactured `arazzo_document` fed verbatim into F14's own `compile` (crown_external.rs:211) — the **first production caller** of `f14_wasm4pm_arazzo::compile`. Proven identical AIR: `air_digest_hex == receipt.air_digest_hex`; CONFIRMED. |
| 8 | F14 -> F15 | `REAL_EDGE` | F14 `AirProgram` -> `air_program_to_bridge_workflow` (crown_external.rs:296) builds the `BridgeWorkflow` shape `air_core:new/1` consumes; the gated test drives the **real** Erlang `air_core:transition/2` and emits a `dispatch_step` command. Adversarial CONFIRMED (ran the ignored test, passed). |
| 9 | F15 -> F16 | `MISSING_EDGE` (from Rust) | The real F15 -> F16 edge exists Erlang-side (`apps/arazzo_runner/src/arazzo_runner_workflow.erl:114,:475`) but there is **no Rust-composable path** into F16's OTP runner. Driver stops here rather than fabricate topology. F16's own `check_gen_statem_lifecycle_wired` (f16_otp_runner.rs:401) still returns `Err` — its gen_statem is not in the production dispatch path. |
| 10 | F16 -> F18 | `MISSING_EDGE` | F18's **Rust** broker has no caller on the external path; the Erlang `arazzo_runner_broker` is a different broker. |
| 11 | F18 -> F20 | `MISSING_EDGE` | F20 (`f20_external_dispatch.rs`) has no production caller anywhere in the workspace; not wired to F18. |
| 12 | F20 -> F02 (re-admit) | `MISSING_EDGE` | F20's collect path re-admits through cng's own private pipeline, not this crate's `admit_observation`; `SubworkflowDispatchOutcome` exposes only a digest, never the raw consequence Turtle. Structurally absent. |
| 13 | F02 -> F15 (AIR transition) | `MISSING_EDGE` | Not wired. |
| 14 | F15 -> F21 | `MISSING_EDGE` | Not wired. |
| 15 | F21 -> F24 | `MISSING_EDGE` | Not wired. |
| 16 | F24 -> F25 | `MISSING_EDGE` | Not wired. |

`FIRST_EXTERNAL_BROKEN_EDGE` = **`F15 -> F16`** (first **structurally absent** edge — no
Rust-composable path exists; the driver correctly refuses to fabricate a shortcut). Under the
strict "every edge must be `REAL_EDGE`" reading, the first edge that is not a *full* `REAL_EDGE` is
`F10 -> F12` (`PARTIAL_REAL_EDGE`): real data flows F10 -> F12, but F10 does not synthesize the cut.
Both facts independently force `EXTERNAL_..._CONTIGUOUS_PATH = false`; `F15 -> F16` is the edge
where composition actually stops.

## Edge census (distinct edges across the union of both witnesses)

The shared prefix (4 edges) is counted once. Union total = 4 (shared) + 7 (LOCAL tail) + 12
(EXTERNAL tail) = **23 distinct edges**. Buckets are exclusive and sum to 23.

| Bucket | Count | Edges |
|---|---|---|
| `REAL_EDGE_COUNT` (full) | **13** | F02->F03, F03->F08, F08->F09, F09->F10, F10->F11, F11->F18, F18->F19, F19->F02(re-admit), F02(re-admit)->F24, F24->F21 (LOCAL, all committed `d60f2036`/`eeca952a`/`66d8732e`/`0815680a`/`217dc37d`); F12->F13, F13->F14, F14->F15 (EXTERNAL) |
| `PARTIAL_REAL_EDGE` | 1 | F10->F12 |
| `TEST_ONLY_EDGE` | 0 | (was F10->F11, F11->F18 -- both closed to `REAL_EDGE`, see above) |
| `MISSING_EDGE_COUNT` | **9** | F21->F25 (LOCAL); F15->F16, F16->F18, F18->F20, F20->F02, F02->F15, F15->F21, F21->F24, F24->F25 (EXTERNAL) |
| `REFUSED_EDGE_COUNT` | **0** | No witness edge is a by-design correct-refusal boundary. |

Strict-contiguity accounting: only the 13 full `REAL_EDGE`s satisfy the path predicate. The 1
`PARTIAL_REAL_EDGE` (`F10->F12`) has real, tested code but leaves a semantic sub-property
unsatisfied, so it does not count toward the EXTERNAL contiguous path. If bucketed coarsely as
"real vs not-real," not-real = 1 + 9 = 10 of 23.

Per-witness contiguous real prefix from F02: LOCAL = **10 edges** (stops before `F21->F25`, the
last LOCAL edge -- updated from the original 4 -- see commits `d60f2036`, `eeca952a`, `66d8732e`,
`0815680a`, `217dc37d`); EXTERNAL = 4 edges (stops at the `F10->F12` partial; then 3 more real
edges F12->F15 sit past the break).

## Whole-crate confirmation (commands run this session, non-isolated)

No concurrent `cargo`/`rebar3`/`just` build was running (`ps aux` checked before starting), so the
shared `target/` lock was safe to use.

```text
$ just multifractal-workflow-check          # cargo check -p multifractal-workflow --tests
    Checking multifractal-workflow v26.7.12 (/Users/sac/praxis/crates/multifractal-workflow)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.91s
# clean: every warning emitted is in a dependency crate (wasm4pm-cognition, praxis-graphlaw,
# ggen, cng), none in multifractal-workflow itself.

$ just multifractal-workflow-test-long      # cargo test -p multifractal-workflow
test result: ok. 404 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out
EXIT=0
```

Exact final counts: **404 passed, 0 failed, 6 ignored, 0 filtered out**; process exit code `0`;
`grep -c FAILED` on the full log = `0`. The 6 ignored tests were individually inspected and each
carries a legitimate, checkable environment-gate reason (not the blanket-hide corruption pattern):

- 5x `requires escript on PATH and apps/air_core compiled via 'just erlang-compile'` —
  `f15_air_transition_core::bridge` (4) + `crown_external ... f14_air_program_drives_real_air_core`
  (1).
- 1x `requires rebar3/just on PATH and the Erlang/OTP umbrella` —
  `f17_atomvm_runtime::run_otp_atomvm_differential_suite_reports_real_pass`.

## Three highest-value next repairs (by downstream unlock, not ticket number)

**Repairs 1-4 and the F24->F21 half of 5 are all DONE** (commits `d60f2036`, `eeca952a`,
`66d8732e`, `0815680a`, `217dc37d`) -- kept here with their original text for history; repair 6
(`F21 -> F25`, the very last LOCAL edge) is the current frontier.

All three are on the **LOCAL** witness. Rationale: the EXTERNAL decisive break (`F15 -> F16`) is a
genuine Rust-to-BEAM process boundary — F16's gen_statem is not in the production dispatch path, and
composing Rust into the OTP runner is real distributed-systems engineering with low unlock per unit
effort. The LOCAL tail, by contrast, is nearly closable by **reusing code that already exists and
already passes real tests**.

### 1. ~~Extend `drive_local_witness_prefix` two stages past F10~~ — DONE (`d60f2036`)

Call F11's already-real, already-tested `load_from_geometry` (F10 `Powl` geometry -> F11 AST) and
then `dispatch_local_execution_via_broker` (F11 -> `LOCAL_DONE` -> F18's 8-stage broker) from the
driver. Both functions exist and are exercised by real (non-mock) tests against real F10 output and
the real F18 broker (F11: 14 tests; F18: 21 tests, all passing this session). This is roughly two
call sites plus one composition test — it converts `F10 -> F11` **and** `F11 -> F18` from
`TEST_ONLY` to `REAL_EDGE` in one change, advancing the LOCAL contiguous real path from 4 edges
(F02->F10) to 6 (F02->F18). Lowest risk, highest immediate edge yield, and a prerequisite for
repairs 2 and 3.

### 2. ~~Wire F18 broker actuation into the real F19 hook registry~~ — DONE (`eeca952a`)

After repair 1, `F18 -> F19` is the next LOCAL break. F19 is already a real, tested
hook-capability registry (`f19_hooks::resolve_hook_for_action`, the pattern F08's `hook_binder.rs`
already reuses). Landed as: `drive_local_witness_prefix`, gated on a real `broker_receipt`,
resolves F19's hook for the same grounded action F08 bound at planning time (fresh ledger, distinct
post-actuation binding). Unlocks `F18 -> F19`; advances the LOCAL contiguous real path from 6 edges
to **7** (F02->F19).

### 3. ~~Re-admit the F19 hook consequence through F02~~ — DONE, F19->F02 half only (`66d8732e`)

Re-admits the hook consequence through F02's real `admit_observation` a second time (`F19 -> F02`),
under a distinct local-runtime principal (honest split from the external planner identity).
Advances the LOCAL contiguous real path from 7 edges to **8** (F02->F02-re-admit).

Audited the dependency this pass: F21/F24/F25 have **no corruption signature** (checked
doc-comment-vs-body mismatch and `#[ignore]` legitimacy against the same pattern found earlier this
session -- all three are clean). But the audit also surfaced a **real blocker**, not just absence:
`f24_ocel_construct::idempotency_gate` and `f25_receipts_replay::chaos_gate::admit_for_replay` are
both genuine, already-honest `NotYetImplemented` refusals (HAND_WRITE_REQUIRED per their own doc
comments, confirmed zero existing idempotency/correlation/chaos-recovery code exists in this repo
for either to wrap). `f21_parent_child_closure::admit_child_and_evaluate` is real and clean --
F21 itself is not the blocker.

### 4. ~~Wire the re-admitted actuation consequence into F24's real OCEL construction~~ — DONE (`0815680a`)

`F24::run_construct` does **not** require `idempotency_gate` as a precondition (confirmed by
reading its body: `ConstructProfile::resolve` -> `project_otel_to_ocel` -> `insert_quads` ->
`receipt_otel_to_ocel` -> `insert_quads` -> `extract_receipt_head` ->
`verify_receipt_otel_to_ocel`, never touching `idempotency_gate`). So the honest option (b) from
this repair's original text turned out to be a full `REAL_EDGE`, not a `PARTIAL_REAL_EDGE`: the
re-admitted actuation consequence is synthesized into a real `OtlpSpan` (identity fields are
F18/F19's own receipt hashes; `parent_span_id` is the re-admission's own output receipt hash) and
run through `run_construct` for real. Advances the LOCAL contiguous real path from 8 edges to
**9** (F02->F24).

### 5. ~~Wire F24's OCEL evidence into F09's own recursive socket closure~~ — DONE, F24->F21 half only (`217dc37d`)

`growth.closure`/`growth.child_socket` turned out to already be F09's own real output, produced
fresh by `manufacture_and_bind_child` specifically for this purpose (confirmed by reading its body,
not just the doc comment: `closure = RecursiveSocketClosure::declare(&pcc, plan.parent_socket, law)`
over `new_root`, which already includes the grafted child at `child_socket`) -- no repurposing of
`run.growth_closure` (F09's separate planning-time gate) was needed. The `ValidationReport`
question resolved as: build a real, non-vacuous SHACL shape (`sh:minCount 1` on a freshly-asserted
`ocelReceiptHead` triple carrying F24's actual `ocel_outcome.receipt_head`) and run
`Validator::validate` for real, rather than fabricating `conforms: true` -- `promote_observed_to_admitted`'s
own refusal message literally says "failed SHACL conformance" (PRD §9), so a non-SHACL-derived
evidence would have corrupted that message's meaning on any future failure path. Advances the LOCAL
contiguous real path from 9 edges to **10** (F02->F21) -- the last remaining edge is `F21 -> F25`.

### 6. Close `F21 -> F25` — CURRENT FRONTIER, LAST LOCAL EDGE

This is the semantically load-bearing final leg that flips `LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH`
to true -- the entire LOCAL witness closes once this edge lands. `f25_receipts_replay::chaos_gate::admit_for_replay`
is a confirmed-honest `NotYetImplemented` refusal and remains out of scope (HAND_WRITE_REQUIRED).
Step 0 for a future pass: audit `f25_receipts_replay::run` (F25's own top-level real entry point,
`Materials -> Receipt Builder -> Receipt Fold -> Independent Verifier -> Receipt Graph Writer`) for
whether a composable non-chaos-gate path exists from F21's `admit_child_and_evaluate` output
(`bool` + the mutated `RecursiveSocketClosure`) into `run`'s `Materials` input -- this has not yet
been checked. Highest total unlock (completes the entire LOCAL witness, the first crown closed).

## Reachability ceiling (cross-cutting, not an edge repair)

`multifractal-workflow` declares `[lib]` only — no `[[bin]]`, no `main.rs` (confirmed:
`grep '[[bin]]' Cargo.toml` empty; `find . -name main.rs` empty). A `REAL_EDGE` does **not** require
a binary (a non-test `pub fn` caller such as `drive_local_witness_prefix` suffices, and adversarial
verification confirmed it as the production caller for F02->F10). But the atlas L8 "production
reachability" gate — cited as an open gap by F02, F11, F20, and both crown integrators — is a
strictly higher bar that no family can meet until some host (a CLI, or the chatman engine) invokes
these drivers. Track this separately from edge contiguity; it does not change any count above.

## Method and honesty notes

- Verdicts are taken from the **adversarial verification results**, which override the original
  wiring claims wherever they conflict. Where an adversarial pass found an imprecision (e.g. the
  crown_external report's stale secondhand claim that F18's broker has zero callers — F11 does call
  it on the LOCAL path), the correction is reflected above.
- `REFUSED_EDGE_COUNT = 0`: no witness edge is a correct-by-design refusal boundary. Refusals do
  occur *within* real edges (e.g. `ExternalCutTypeMismatch` gating F12->F13), but those are the
  edge working, not a refused edge.
- F21/F24/F25 real-vs-stub status **was** independently re-derived this session (this repair pass):
  all three modules are free of the doc-comment-vs-body corruption signature found earlier this
  session; F24's `idempotency_gate` and F25's `chaos_gate::admit_for_replay` are confirmed-honest
  `NotYetImplemented` refusals, not corruption. F21 is now wired (`F24 -> F21`, commit `217dc37d`);
  F25's tail edge remains `MISSING_EDGE` because no composition into it exists yet, independent of
  its own confirmed internal maturity.
- The `F24 -> F21` SHACL evidence (`ACTUATION_CONSTRUCT_EVIDENCE_SHAPES`) validates a
  freshly-asserted fact about `ocel_outcome.receipt_head`, not full structural conformance of
  `ocel_outcome`'s own OCEL quads -- those live in `oxigraph`'s `Quad` representation, a different
  RDF library than the `praxis-graphlaw` `Term`/`TripleIndex` this validator consumes. Bridging the
  two representations for a full OCEL-conformance check is disclosed, deferred future work.

## See also

- `crates/multifractal-workflow/src/crown_local.rs` — LOCAL prefix production caller (F02->F21).
- `crates/multifractal-workflow/src/crown_external.rs` — EXTERNAL tail production caller (F10->F15).
- `apps/arazzo_runner/src/arazzo_runner_workflow.erl` — the real (Erlang-side) F15->F16 edge.
- `docs/jira/v26.7.11/SAFETY_FINDINGS.md` — the removed LLM-hot-load pattern; do not reintroduce.
- `CLAUDE.md` (Invariants, Standing) and `.claude/rules/no-overclaiming.md` — the discipline this
  report is written under.

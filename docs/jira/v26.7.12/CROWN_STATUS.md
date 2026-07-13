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
| `LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` | **false** | Corrected by an independent re-audit (this pass, not the original `66cb59b1` self-report): `MISSING_EDGE_COUNT` is **0** (all 11 edges are real code, `?`-gated, live-tested), but 2 of 11 are `PARTIAL_REAL_EDGE`, not full `REAL_EDGE` -- `F18 -> F19` and `F21 -> F25` (see their rows below). Under the same strict "every edge must be a full `REAL_EDGE`" reading already applied to EXTERNAL's `F10 -> F12`, this marker must also read **false**. The prior "true / first crown witness closed" claim in this doc was an overclaim; corrected here rather than left standing. |
| `EXTERNAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` | **false** | `MISSING_EDGE_COUNT` is now **0** (`F18 -> F20` closed commit `1e1ce976`, following `F15 -> F16` commit `1d3b9fb2` and `F16 -> F18` commit `4ce20102` -- the entire shared-prefix-anchored EXTERNAL forward path `F10->F12->F13->F14->F15->F16->F18->F20` is real). Still **false** under the strict "every edge must be a full `REAL_EDGE`" reading: `F10 -> F12` remains `PARTIAL_REAL_EDGE` (F10 does not itself synthesize the `ExternalCut` node it's wrapped in -- see that edge's own row). **Not a function-composition chain**: `drive_f18_completion_through_f20_dispatch` and `drive_external_reentry`'s own `F20 -> F02(re-admit)` continuation are two independently-real instantiations of the same real entry points, not literally one Rust call chain from F10 to F25 -- see `crown_external.rs`'s own self-correction in the `F18 -> F20` doc section (commit `1e1ce976`) for why an earlier draft's stronger claim was wrong and was fixed before committing. |
| `OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` | **false** | Requires **both** witness markers true; neither is (corrected this pass -- LOCAL was previously, incorrectly, marked true). |

The shared prefix `F02 -> F03 -> F08 -> F09 -> F10` plus the entire LOCAL tail
`F11 -> F18 -> F19 -> F02(re-admit) -> F24 -> F21 -> F25` is driven by one real, `?`-gated,
live-tested production function, `crown_local::drive_local_witness_prefix` (commits `3322bf2d`,
`d60f2036`, `eeca952a`, `66d8732e`, `0815680a`, `217dc37d`, `66cb59b1`) -- but an independent
re-audit of this exact chain (this pass) found 2 of its 11 edges are `PARTIAL_REAL_EDGE`, not
full `REAL_EDGE`: `F18 -> F19` (`resolve_hook_for_action` is real and genuinely `?`-gated on
`broker_receipt` existing, but never reads any of `broker_receipt`'s fields -- control sequencing
only, no data-threading) and `F21 -> F25` (`f25_receipts_replay::run`'s `Materials` are built
entirely from F02/F24's already-computed texts, never from `F21`'s own produced output --
`parent_closed` and `growth.closure`'s post-admission state are not consumed). Both are real,
tested, `?`-gated code with no fabricated caller -- the correction is to this doc's classification
of them, not to the code, which does exactly what it was built to do (see each row's own note for
why threading the "missing" data into the call would be an artificial, unmotivated dependency
rather than a real one; investigated and rejected as a fix path this pass). The corrected count is
**9 full `REAL_EDGE` + 2 `PARTIAL_REAL_EDGE` of 11**, not 11/11 -- so **LOCAL is not yet a closed
crown witness** under the strict reading. The EXTERNAL tail
adds three more real edges (`F12 -> F13 -> F14 -> F15`) via
`crown_external::drive_external_witness_tail`, but remains blocked past `F15` -- everything past
`F15` on the EXTERNAL tail is not yet a real edge.

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

Production caller for all 11 edges: `crown_local::drive_local_witness_prefix`
(`crates/multifractal-workflow/src/crown_local.rs`). Verified this session (post-`66cb59b1`): the
driver calls `admit_observation`, `contract` with the exact admitted bytes, `run_pipeline`, F09's
`manufacture_and_bind_child` (reaches F10 internally), then `geometry_to_local_ast` ->
`dispatch_local_execution_via_broker` (F11 -> F18) -> real `resolve_hook_for_action` (F18 -> F19)
-> a second real `admit_observation` call over a synthesized actuation-consequence observation
(F19 -> F02 re-admit) -> a real OTel span (admit -> project -> insert) run through F24's real
`run_construct` (F02(re-admit) -> F24) -> real `admit_child_and_evaluate` over F09's own
`growth.closure`/`growth.child_socket`, evidenced by a real SHACL check over F24's receipt head
(F24 -> F21) -> real `f25_receipts_replay::run` over `Materials` built from this same run's own
canonical texts (F21 -> F25), gated end to end.
`crown_local_prefix_drives_the_entire_local_witness_end_to_end` (`crown_local_test.rs`) exercises
the whole chain and asserts on every stage's real output, including
`HookResolutionState::Replayable`, a second `AdmissionState::Admitted` receipt distinct from the
first, F24's real `ConstructProfile::OtelToOcel` outcome with non-empty
`ocel_quads`/`receipt_quads`/`receipt_head`, the manufactured child's real
`ChildCompletionState::Admitted` transition, and F25's real receipt-fold + replay-equivalence
outcome (all 6 CTQ material kinds matched, `receipt_root_matched`, non-empty PROV-O graph quads).

| # | Edge | Verdict | Evidence (this session) |
|---|---|---|---|
| 1 | F02 -> F03 | `REAL_EDGE` | crown_local.rs; adversarial verdict CONFIRMED. |
| 2 | F03 -> F08 | `REAL_EDGE` | gated on `Plannable`; `receipt_head` salts F08 `case_id`; CONFIRMED. |
| 3 | F08 -> F09 | `REAL_EDGE`* | control-gated + provenance-bound; CONFIRMED. *Disclosed: F09 re-plans the shared PDDL rather than byte-ingesting F08's `Pddl8Tape` (no residual-goal extractor exists). Honest, not smuggled. Corrected this pass: earlier wording here said "tape-consistency-checked" as if enforced in production; re-audit found the tape-count check (`outcome.growth.geometry.source_action_count == outcome.plan.tape.ops.len()`) exists only inside `crown_local_test.rs`, never in `drive_local_witness_prefix` itself -- nothing in production would catch a divergence on a different fixture. Fuller prose already in `crown_local.rs`'s own module doc got this right; only this table's shorthand was imprecise. |
| 4 | F09 -> F10 | `REAL_EDGE` | `manufacture_and_bind_child` -> `f10_powl_geometry::manufacture_powl_v2` (f09_mfw_growth.rs:825 -> :774); CONFIRMED. |
| 5 | F10 -> F11 | `REAL_EDGE` | (commit `d60f2036`) `geometry_to_local_ast(&growth.geometry.root)` now called from `drive_local_witness_prefix`, a non-test `pub fn`; was `TEST_ONLY` when this doc was first written. |
| 6 | F11 -> F18 | `REAL_EDGE` | (commit `d60f2036`) `dispatch_local_execution_via_broker` now called from the same driver, real `BrokerReceipt` returned with real `consequence_hash_hex`/`receipt_hash_hex`; was `TEST_ONLY`. |
| 7 | F18 -> F19 | `PARTIAL_REAL_EDGE` | **Corrected this pass** (was `REAL_EDGE` since commit `eeca952a`; an independent re-audit found this an overclaim). `resolve_hook_for_action` (crown_local.rs:572-580) is real and genuinely `?`-gated on `dispatch_local_execution_via_broker` having returned `Ok` -- but its actual arguments (`ground_action` from F08's `plan.tape`, `run.hook_pack_turtle` from F02's original admission, a fresh `InMemoryReceiptLedger::default()`) never read any field of `broker_receipt` (`consequence_hash_hex`/`receipt_hash_hex`/`authority_token_hex`); `broker_receipt.receipt_hash_hex` is first read three lines later, inside the *next* edge (F19->F02 re-admit). So this edge is real control sequencing (unreachable unless F18 succeeded) but not data-threading (nothing F18 actually produced is consumed here) -- fails this doc's own stated `REAL_EDGE` bar ("passes the actual upstream consequence... into the actual downstream mechanism"). Investigated whether threading `broker_receipt` into `resolve_hook_for_action` is an honest fix: no -- that function is also a real, live production caller from F08's own `hook_binder.rs::bind_actions_with_ledger` (planning-time capability check, before any actuation/broker_receipt exists), so its core logic must stay actuation-agnostic by design; hook capability resolution is correctly a pure function of (hook catalog, action schema), not of which specific actuation triggered the check. Forcing an unused actuation parameter into it to satisfy this table would be the smuggled/fabricated dependency this project's own discipline forbids. The honest fix is this reclassification, not a code change. |
| 8 | F19 -> F02 (re-admit) | `REAL_EDGE` | (commit `66d8732e`) `admit_observation` called a second time, `?`-gated on real `hook_resolution`, over a synthesized actuation-consequence observation under a distinct local-runtime principal (`actuation_source_id`); was `MISSING_EDGE`. |
| 9 | F02 -> F24 | `REAL_EDGE` | (commit `0815680a`) the re-admitted consequence becomes a real `cng::otel_rdf::OtlpSpan` (`trace_id`/`span_id` = F18/F19's own receipt hashes, `parent_span_id` = the re-admission's own output receipt hash) run through `f24_ocel_construct::run_construct("otel-to-ocel", ...)`; was `MISSING_EDGE`. Never calls F24's own `idempotency_gate` (that's a distinct, orthogonal L7 capability `run_construct` does not require as a precondition — confirmed by reading `run_construct`'s body — so this is a full `REAL_EDGE`, not `PARTIAL_REAL_EDGE`). |
| 10 | F24 -> F21 | `REAL_EDGE` | (commit `217dc37d`) `admit_child_and_evaluate(&mut growth.closure, &growth.child_socket, &evidence)` -- `growth.closure`/`growth.child_socket` are F09's own real output built for exactly this purpose (`GrowthOutcome::child_socket`'s own doc comment). Evidence is a real, non-vacuous `Validator::validate` result: asserts F24's actual `ocel_outcome.receipt_head` and checks a `sh:minCount 1` constraint that is genuinely evaluated (target class matched by a real individual), not a vacuous shape and not a fabricated `conforms:true`; was `MISSING_EDGE`. |
| 11 | F21 -> F25 | `PARTIAL_REAL_EDGE` | **Corrected this pass** (was `REAL_EDGE` since commit `66cb59b1`; an independent re-audit found this an overclaim). `admit_child_and_evaluate` (F21) is real and `f25_receipts_replay::run` is real, genuinely `?`-gated in sequence -- but every `Materials` field (`source`=F02's payload, `query`=`run.pddl_problem`, `template`=the static SHACL shape, `program`=`run.hook_pack_turtle`, `event`=F24's `evidence_turtle`, `output`=F24's `ocel_outcome.receipt_head`) was already computed *before* F21 ran; none derive from `parent_closed` (F21's own boolean outcome) or from `growth.closure`'s post-admission mutated state. F21's own produced consequence never appears anywhere in `Materials` -- F25 reuses F02/F24's canonical texts a second time rather than consuming anything F21 itself produced. Same defect class as `F18 -> F19` above: real sequential `?`-gating between two real production functions, not data-threading of the specific adjacent stage's own output. Nothing is fabricated (every reused field is genuinely real, per its own original edge), so this is a reclassification, not a code fix. |

`FIRST_LOCAL_BROKEN_EDGE` = **none — `MISSING_EDGE_COUNT = 0`** (edges 5-11 closed since this doc
was first written; see commits `d60f2036`, `eeca952a`, `66d8732e`, `0815680a`, `217dc37d`,
`66cb59b1`). `LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` is **still false**, for the same reason
class as EXTERNAL: under the strict "every edge must be a full `REAL_EDGE`" reading, edges 7
(`F18 -> F19`) and 11 (`F21 -> F25`) are `PARTIAL_REAL_EDGE` (corrected this pass; see their rows
above). LOCAL now has 9/11 full `REAL_EDGE` and 2/11 `PARTIAL_REAL_EDGE`; EXTERNAL (below) has a
single `PARTIAL_REAL_EDGE` among its own edges. Neither witness is closed under this doc's own
stated bar.

## EXTERNAL witness — per-edge classification

Edges 1-4 are the same shared prefix (all `REAL_EDGE`, above). Production caller for edges 5-8:
`crown_external::drive_external_witness_tail`
(`crates/multifractal-workflow/src/crown_external.rs`). Verified this session: the driver calls
`resolve_external_cut_at` -> `project_and_compile` -> `f14_wasm4pm_arazzo::compile` ->
`air_program_to_bridge_workflow` (crown_external.rs:216). Confirmed it **stops at building F15's
input**: the actual `call_air_core_bridge` is deliberately not invoked in the production path
(crown_external.rs:164, "not called here"); the live Erlang round trip runs only in the gated test.
A separate, topologically independent production caller closes edge 12: `crown_external::drive_external_reentry`
(commit `b4d743f7`) really dispatches a subworkflow contract, has a real `engine_serve` admit and
manufacture a response, collects it through cng's own real 5-stage pipeline, and re-admits it
through F02 -- classified as `REAL_EDGE` on its own real data-threading regardless of edges 9-11's
`MISSING_EDGE` status, the same way edge 7 (`F13->F14`) was real while edge 5 was only `PARTIAL`.

| # | Edge | Verdict | Evidence (this session) |
|---|---|---|---|
| 1-4 | F02..F10 | `REAL_EDGE` x4 | Shared prefix via `drive_local_witness_prefix` (above). |
| 5 | F10 -> F12 | `PARTIAL_REAL_EDGE` | F10's real geometry becomes the `ExternalCut.region` (crown_external.rs:257), but F10's `build_powl_geometry` **never synthesizes** `Powl::ExternalCut` (grep: only in `to_turtle` emit arm, f10_powl_geometry.rs:897). The cut boundary is declared by the driver on top of F10's geometry, not emitted by F10. Adversarial verdict: PARTIAL, appropriately hedged. |
| 6 | F12 -> F13 | `REAL_EDGE` | `resolve_external_cut_at(&model, child(1))` gates `project_and_compile` over the identical `model`; refuses `ExternalCutTypeMismatch` before F13 runs; CONFIRMED. |
| 7 | F13 -> F14 | `REAL_EDGE` (byte-level) | F13's manufactured `arazzo_document` fed verbatim into F14's own `compile` (crown_external.rs:211) — the **first production caller** of `f14_wasm4pm_arazzo::compile`. Proven identical AIR: `air_digest_hex == receipt.air_digest_hex`; CONFIRMED. |
| 8 | F14 -> F15 | `REAL_EDGE` | F14 `AirProgram` -> `air_program_to_bridge_workflow` (crown_external.rs:296) builds the `BridgeWorkflow` shape `air_core:new/1` consumes; the gated test drives the **real** Erlang `air_core:transition/2` and emits a `dispatch_step` command. Adversarial CONFIRMED (ran the ignored test, passed). |
| 9 | F15 -> F16 | `REAL_EDGE` | (commit `1d3b9fb2`) `crown_external::drive_external_witness_tail_through_f16` drives F15's real `air_core:transition/2` result, then feeds every real `dispatch_step` command into a real `arazzo_runner_dispatch_statem` gen_statem via a *second*, independent production entrypoint (`f16_otp_runner::bridge::call_dispatch_statem_bridge`, a new escript reusing `arazzo_runner_sup:start_workflow/1` -- not `apply_transition/4`, which stays untouched, so `check_gen_statem_lifecycle_wired` (f16_otp_runner.rs) correctly still returns `Err` since that check is specifically about `apply_transition/4`'s own internal wiring). Verified LIVE (`--ignored`, real escript + clean-rebuilt `apps/arazzo_runner`): a lawful single-step dispatch traverses all 8 atlas states with a real, non-empty dispatch token; a missing-`correlation_id` request is genuinely refused by the real broker's `CORRELATION_MISSING` atom. Honest nuance: F10's own template-derived output has zero `onSuccess` routing, so applying this driver to that specific input dispatches nothing to F16 (empty, not a failure) -- proven, not just asserted, by a dedicated non-`#[ignore]`d test. Was `MISSING_EDGE`. |
| 10 | F16 -> F18 | `REAL_EDGE` | (commit `4ce20102`) `crown_external::drive_f16_completion_through_f18_broker` reuses `f11_bcinr_runtime::dispatch_local_execution_via_broker`'s own proven `Broker` stage sequence verbatim (`verify_standing -> authorize -> claim_idempotency -> bind_correlation -> actuate -> capture_consequence -> issue_receipt`), applied to a different real consequence source: F16's real dispatch token (from a `DispatchStatemOutcome::Completed`) becomes the bytes `actuate`'s closure returns and `capture_consequence` folds into its BLAKE3 chain. Honest boundary: a `Refused` F16 outcome has no token to actuate, so this refuses with `ExternalF18Refused::F16DispatchNotCompleted` rather than fabricating a consequence. Verified LIVE (`--ignored`, real escript + clean-rebuilt `apps/air_core`+`apps/arazzo_runner`): `f16_completion_actuates_a_real_f18_broker_receipt` drives the full F14->F15->F16->F18 chain and asserts non-empty, real `BrokerReceipt` hashes. Was `MISSING_EDGE`. |
| 11 | F18 -> F20 | `REAL_EDGE` | (commit `1e1ce976`) `crown_external::drive_f18_completion_through_f20_dispatch` builds a real `SubworkflowPlan` whose `id`/`problem_digest` are derived from a real F18 `BrokerReceipt`'s own `workflow_id`/`step_id`/`consequence_hash_hex` (not an arbitrary caller-supplied value), then dispatches it through the same real `dispatch_subworkflow_to_engine -> engine_serve -> collect_subworkflow_consequence` round trip edge 12 (`drive_external_reentry`) already uses -- a second, independent instantiation of the identical real entry points, not a literal function-composition into edge 12 (an earlier doc-comment draft claimed the latter; corrected in the same commit before landing). Verified LIVE (`--ignored`, real escript + clean-rebuilt `apps/air_core`+`apps/arazzo_runner`): `f18_broker_receipt_drives_a_real_f20_dispatch_to_admission` drives the full F14->F15->F16->F18->F20 chain and asserts `admitted: true` with real `consequence_turtle`/`consequence_digest`. Was `MISSING_EDGE`. |
| 12 | F20 -> F02 (re-admit) | `REAL_EDGE` | (commit `b4d743f7`) `dispatch_subworkflow_to_engine` -> `engine_serve` (real, previously-zero-callers receiving side of the same bridge) -> `collect_subworkflow_consequence`, gated on cng's own real `admitted: true`, then re-admitted through F02's real `admit_observation` under a third distinct principal. `SubworkflowDispatchOutcome` widened with `consequence_turtle: Option<String>` (cng, minimal: one field, no new admission logic). Empirically verified: the real round trip produces `admitted: true` with real consequence content, not just structural plumbing; was `MISSING_EDGE`. |
| 13 | F02 -> F15 (AIR transition) | `REAL_EDGE` | (commit `38048b27`) `crown_external::drive_external_readmit_transition` composes `drive_external_reentry` verbatim then calls `call_air_core_bridge` a second time to complete a minimal bridge workflow keyed by the real `dispatch_id`, event payload = the real F02 admission receipt hash. Verified LIVE (`--ignored`, real escript + compiled `apps/air_core`), not just structurally: `external_readmit_transition_completes_the_dispatched_step_through_real_air_core` passes against the real Erlang subprocess. Was `MISSING_EDGE`. |
| 14 | F15 -> F21 | `REAL_EDGE` | (commit `a139d477`) `crown_external::drive_external_readmit_transition`'s final stage: the real AIR transition's own `ready_steps`/`commands` output is folded into a deterministic BLAKE3 receipt, validated by a real (non-vacuous) SHACL check, and admitted via `admit_child_and_evaluate` under a freshly-declared `RecursiveSocketClosure` (no upstream family here naturally produces one, unlike LOCAL's F09-sourced closure -- disclosed, not smuggled). Verified LIVE (`--ignored`, real escript + compiled `apps/air_core`): `parent_closed: true` confirmed against the actual call. Was `MISSING_EDGE`. |
| 15 | F21 -> F24 | `REAL_EDGE` | (commit `8c2675be`) `crown_external::drive_external_readmit_transition`'s final stage: the admitted external-dispatch consequence (F21) is projected as a real `cng::otel_rdf::OtlpSpan` (`trace_id`/`span_id`/`parent_span_id` all real F20/F21/F02 output; `process.object.id` reuses F21's own evidence subject) run through `f24_ocel_construct::run_construct`. Verified LIVE (`--ignored`, real escript + compiled `apps/air_core`): real `ConstructProfile::OtelToOcel` outcome with non-empty quads/receipt_head confirmed against the actual call. Topology note: EXTERNAL's own atlas order is `F21 -> F24` (admission before construction), the reverse of LOCAL's `F24 -> F21` -- taken as given, not reinterpreted. Was `MISSING_EDGE`. |
| 16 | F24 -> F25 | `REAL_EDGE` | (commit `11dcee0e`) `crown_external::drive_external_readmit_transition`'s final stage: folds a real F25 receipt over the whole chain's own canonical texts (source=consequence Turtle, query=dispatch id, template=SHACL evidence shape, program=F21's transition_receipt, event=F21's evidence Turtle, output=F24's receipt head), replayed via `materials.clone()` matching F25's own established test pattern. Verified LIVE (`--ignored`): all 6 CTQ material kinds matched, `receipt_root_matched`, non-empty PROV-O graph confirmed against real escript-derived data. **Completes the entire EXTERNAL loop-back tail** (`F20->F02->F15->F21->F24->F25`, all real). Was `MISSING_EDGE`. |

`FIRST_EXTERNAL_BROKEN_EDGE` = **none -- `MISSING_EDGE_COUNT = 0`** (closed three times this
cycle: `F15 -> F16` commit `1d3b9fb2` via a second production entrypoint into F16's OTP runner;
`F16 -> F18` commit `4ce20102` via F16's real dispatch token actuated through F18's real `Broker`
lifecycle; `F18 -> F20` commit `1e1ce976` via a real F18 `BrokerReceipt` driving a real F20
dispatch/serve/collect round trip). `EXTERNAL_..._CONTIGUOUS_PATH` is **still false**, for one
remaining reason, not a structural-absence reason: under the strict "every edge must be a *full*
`REAL_EDGE`" reading, `F10 -> F12` is `PARTIAL_REAL_EDGE` (real data flows F10 -> F12, but F10
does not itself synthesize the `ExternalCut` node). No edge is topologically disconnected from the
shared-prefix-anchored composition anymore in the "missing" sense -- edge 11 (`F18->F20`) and edge
12 (`F20->F02`) are each independently real, but are two separate real instantiations of the same
entry points rather than one literal Rust function chain (see edge 11's own row) -- so "contiguous"
here means every edge is independently `REAL_EDGE`, not that one function call threads F10 to F25.

## Edge census (distinct edges across the union of both witnesses)

The shared prefix (4 edges) is counted once. Union total = 4 (shared) + 7 (LOCAL tail) + 12
(EXTERNAL tail) = **23 distinct edges**. Buckets are exclusive and sum to 23.

| Bucket | Count | Edges |
|---|---|---|
| `REAL_EDGE_COUNT` (full) | **20** | F02->F03, F03->F08, F08->F09, F09->F10, F10->F11, F11->F18, F19->F02(re-admit), F02(re-admit)->F24, F24->F21 (LOCAL, 9 of 11 -- `F18->F19` and `F21->F25` moved to `PARTIAL_REAL_EDGE` this pass, see below); F12->F13, F13->F14, F14->F15, F15->F16, F16->F18, F18->F20, F20->F02(re-admit), F02(re-admit)->F15(AIR transition), F15->F21, F21->F24, F24->F25 (EXTERNAL, complete, 11 of 11 -- `F20->F02` committed `b4d743f7`, `F02->F15` committed `38048b27`, `F15->F16` committed `1d3b9fb2`, `F16->F18` committed `4ce20102`, `F18->F20` committed `1e1ce976`, `F15->F21` committed `a139d477`, `F21->F24` committed `8c2675be`, `F24->F25` committed `11dcee0e`) |
| `PARTIAL_REAL_EDGE` | 3 | F10->F12 (EXTERNAL); F18->F19, F21->F25 (LOCAL -- corrected this pass, see their rows above: real `?`-gated sequencing but no data-threading of the specific adjacent stage's own output) |
| `TEST_ONLY_EDGE` | 0 | (was F10->F11, F11->F18 -- both closed to `REAL_EDGE`, see above) |
| `MISSING_EDGE_COUNT` | **0** | None -- EXTERNAL's last missing edge (`F18->F20`) closed commit `1e1ce976`, and LOCAL has no missing edges either (its 2 non-`REAL_EDGE` entries are `PARTIAL_REAL_EDGE`, not `MISSING`). `F10->F12` blocks `EXTERNAL_..._CONTIGUOUS_PATH`; `F18->F19`/`F21->F25` block `LOCAL_..._CONTIGUOUS_PATH`. |
| `REFUSED_EDGE_COUNT` | **0** | No witness edge is a by-design correct-refusal boundary. |

Strict-contiguity accounting: only the 20 full `REAL_EDGE`s satisfy the path predicate. The 3
`PARTIAL_REAL_EDGE`s (`F10->F12`; `F18->F19` and `F21->F25` corrected this pass) each have real,
tested code but leave a semantic sub-property unsatisfied, so none counts toward either witness's
contiguous path. If bucketed coarsely as "real vs not-real," not-real = 3 of 23.

Per-witness contiguous real prefix from F02: LOCAL = **9 of 11 edges full `REAL_EDGE`**
(`MISSING_EDGE_COUNT = 0`, but `F18->F19` and `F21->F25` are `PARTIAL_REAL_EDGE`, corrected this
pass -- see commits `d60f2036`, `eeca952a`, `66d8732e`, `0815680a`, `217dc37d`, `66cb59b1` for the
code, and this doc's own edge rows for the corrected classification); EXTERNAL = 4 edges (stops at
the `F10->F12` partial; then 3 more real edges F12->F15 sit past the break;
`F20->F02(re-admit)->F15(AIR transition)->F21->F24->F25` is a real, now COMPLETE 5-edge EXTERNAL
loop-back tail, but is topologically disconnected from this contiguous-from-F02 prefix until
`F16`/`F18`/`F18->F20` close -- the only remaining EXTERNAL topological gap).

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

**Repairs 1-6 are all DONE** (commits `d60f2036`, `eeca952a`, `66d8732e`, `0815680a`, `217dc37d`,
`66cb59b1`) -- `MISSING_EDGE_COUNT = 0` on the LOCAL witness. **Not** "the entire LOCAL crown
witness is closed" as an earlier version of this doc claimed: a later independent re-audit found
2 of the 11 edges these repairs built (`F18->F19`, `F21->F25`) are `PARTIAL_REAL_EDGE`, not full
`REAL_EDGE` -- see the corrected per-edge table and executive summary above. Kept here with their
original text for history; treat the "closed" framing below as superseded by that correction.
The only remaining **missing-edge** crown-witness work is on the **EXTERNAL** witness (repair 7,
below), which was never scoped as a same-pass repair -- the EXTERNAL decisive break (`F15 -> F16`)
is a genuine Rust-to-BEAM process boundary: F16's gen_statem is not in the production dispatch
path, and composing Rust into the OTP runner is real distributed-systems engineering with low
unlock per unit effort relative to the LOCAL tail, which closed by **reusing code that already
existed and already passed real tests**.

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

### 6. ~~Close `F21 -> F25`~~ — DONE (`66cb59b1`) — **LOCAL WITNESS COMPLETE**

`f25_receipts_replay::run` (F25's own top-level real entry point) turned out to need no chaos-gate
dependency at all: it takes `Materials` (six canonical texts) plus a replay closure, and every
`Materials` field was already a real, already-computed value from earlier in the same run --
`source` = F02's admitted payload, `query` = the PDDL problem, `template` = the real
`ACTUATION_CONSTRUCT_EVIDENCE_SHAPES` SHACL shape, `program` = the hook-pack catalog, `event` =
the F24->F21 evidence Turtle, `output` = F24's real receipt head. The replay closure returns
`materials.clone()`, matching F25's own test suite's established pattern for a deterministic
transformation, not an invented shortcut. `f25_receipts_replay::chaos_gate::admit_for_replay`
remains a confirmed-honest `NotYetImplemented` refusal and was never called. Advances the LOCAL
`MISSING_EDGE_COUNT` from 1 to **0 (11 of 11 edges have real, `?`-gated code)**.

**Corrected by a later pass**: the claim below this line, as originally written, said
`LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH = true` / "the first crown witness closed." An
independent re-audit found that while every `Materials` field listed above is genuinely real, none
of them is `F21`'s *own* produced output (`parent_closed`, `growth.closure`'s post-admission
state) -- they are all F02/F24 values already computed before F21 ran. That makes this edge real
`?`-gated sequencing, not data-threading of the adjacent upstream stage's own consequence, i.e.
`PARTIAL_REAL_EDGE` under this doc's own bar, not full `REAL_EDGE`. See the corrected per-edge
table and executive summary at the top of this document for the current, authoritative claim.

### 7. Close the EXTERNAL witness — `MISSING_EDGE_COUNT = 0`; `F10 -> F12` is the only remaining blocker

**`F15 -> F16` (commit `1d3b9fb2`), `F16 -> F18` (commit `4ce20102`), and `F18 -> F20` (commit
`1e1ce976`) are all now DONE** -- see edges 9-11's rows above. Every edge on the
shared-prefix-anchored EXTERNAL forward path (`F10 -> F12 -> F13 -> F14 -> F15 -> F16 -> F18 ->
F20`) is real, and the `F20 -> F02(re-admit) -> F15(AIR transition) -> F21 -> F24 -> F25`
loop-back tail (repairs 8-12) was already real. `MISSING_EDGE_COUNT = 0` for the first time this
milestone.

`EXTERNAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` is **still false**, for exactly one reason now:
`F10 -> F12` remains `PARTIAL_REAL_EDGE` (F10's `build_powl_geometry` never synthesizes a
`Powl::ExternalCut` node; the cut boundary is declared by the driver on top of F10's geometry, not
emitted by F10 -- see edge 5's own row, unchanged this cycle). Closing this to a full `REAL_EDGE`
would require F10 itself to gain external-cut synthesis, a change to a family this session has not
touched and whose correct behavior for the *other* (non-external-cut) POWL shapes F10 already
produces has not been re-investigated -- **not yet scoped this pass**; apply the same
due-diligence pattern (read the real current source before assuming size) before the next cycle
picks this up. A second, smaller reason `OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` is not literally
"one function call from F10 to F25": edge 11 (`F18->F20`, this cycle) and edge 12 (`F20->F02`,
already real) are independently-real instantiations of the identical entry points, not a single
composed Rust chain -- disclosed in `crown_external.rs`'s own `F18 -> F20` doc section (a first
draft overclaimed the composition and was corrected in the same commit before landing).

**Historical record, preserved (no longer the live blocker for `F15 -> F16`, since a different
angle closed it)**: `arazzo_runner_workflow.erl:503` routes dispatch via the direct synchronous
`arazzo_runner_broker:dispatch/4`, not through `arazzo_runner_dispatch_statem`/`_sup` -- rewiring
that specific call site was correctly assessed (three independent times) as carrying real
regression risk to `arazzo_runner_workflow_test.erl`'s synchronous-ordering assertions, and
remains un-attempted for that reason. It simply turned out not to be necessary: `F15 -> F16` was
closed via `arazzo_runner_sup:start_workflow/1` instead, a second real entrypoint into the same
OTP app that never touches `apply_transition/4`.

**Re-investigated this pass with deeper evidence than any prior cycle** (no code changed; the
already-built `arazzo_runner_dispatch_statem.erl` was read in full, not just cited secondhand):
the gen_statem's own module header (point 5) states the reason wiring was deliberately deferred is
a **concrete, already-identified regression risk**, not just architectural size --
`apply_transition/4` calling `arazzo_runner_broker:dispatch/4` directly is *synchronous*
(blocks until the round trip completes); `arazzo_runner_dispatch_statem:dispatch/1` is
deliberately *asynchronous* by design (point 2: "The `dispatch` call replies `ok` immediately
upon entering `dispatched`, BEFORE the worker's round trip completes -- proving this state is a
real async, concurrently-executing state, not a synchronous simulation dressed up as one").
Naively swapping the call site would flip `apply_transition/4`'s completion-ordering guarantee
from synchronous to asynchronous, and "several `arazzo_runner_workflow_test.erl` assertions rely
on dispatch completing before the next reaction is processed" (the module's own words) -- i.e.,
this specific rewiring would **break existing, currently-passing Erlang tests**, confirmed by
reading the actual dispatch-reply timing in `ready/3`, not inferred. Preserving synchronous
ordering while still routing through the supervised worker would require adding a genuinely new
blocking wait API to the gen_statem (polling `get_outcome/1` or restructuring the reply timing),
which would partially undo the module's own stated purpose (a provably async dispatch state) and
constitutes real new Erlang/OTP design work carrying real regression risk to load-bearing,
already-passing tests -- a categorically different risk than any Rust-side crown-witness edge
built this session, none of which touched shared, already-tested production modules in a way
that could regress existing assertions. Confirms (with stronger, code-level evidence) the
prior conclusion: not attempted, correctly scoped as its own effort.

### 8. ~~Close `F20 -> F02(re-admit)`~~ — DONE (`b4d743f7`)

**Supersedes the "ruled out" investigation previously recorded here.** That investigation
correctly found `SynthesisMode::LoopbackDeterministic` (`cng/src/bench/dispatch.rs:1050`)
`pub(super)`-inaccessible, but missed a *different*, publicly re-exported real function that
solves the same problem: `cng::bench::engine_serve` (`pub use engine::{engine_serve, ...}` in
`cng/src/bench/mod.rs`) is the real *receiving* side of the exact same `dispatch_bridge.rs`
inbox/outbox convention (`engine.rs`'s own module doc: "the SAME on-disk contract format
`engine_dispatch_remote` writes and a real `cng engine serve` process organically scans and
admits" -- and `dispatch_subworkflow_to_engine`'s own doc: "a real `cng engine serve` process
organically scans and admits"). Calling `engine_serve` between dispatch and collect makes cng's
own real import/plan/project/validate/conformance chain manufacture a genuinely conformant
response, empirically verified to produce `admitted: true`.
`SubworkflowDispatchOutcome.consequence_digest` was genuinely digest-only (the "ruled out" note's
other finding stands), so `SubworkflowDispatchOutcome` was widened with one new field,
`consequence_turtle: Option<String>`, carrying the already-computed raw text -- no new admission
logic, no cng-private stage detail surfaced. Adds a `REAL_EDGE` (`F20->F02(re-admit)`),
topologically independent of edges 9-11 (same class of independence as `F13->F14` vs.
`F10->F12`'s `PARTIAL` status).

### 9. ~~Close `F02(re-admit) -> F15 (AIR transition)`~~ — DONE (`38048b27`)

Composes `drive_external_reentry` (repair 8) verbatim, then calls
`call_air_core_bridge` -- the same real entry point `drive_external_witness_tail`'s own gated test
already exercises -- a second time to complete a minimal one-step bridge workflow keyed by the
real `dispatch_id`, with a `StepCompleted` event carrying the real F02 admission receipt hash.
Verified LIVE this pass (`--ignored`, real `escript` + compiled `apps/air_core` were available):
`external_readmit_transition_completes_the_dispatched_step_through_real_air_core` passes against
the actual Erlang subprocess, not merely structurally. Adds a 2nd real EXTERNAL edge past
`F20->F02`, still topologically disconnected from the shared-prefix-anchored composition until
`F16`/`F18`/`F18->F20` close (same disconnection class as edge 8's own note).

### 10. ~~Close `F15(AIR transition) -> F21`~~ — DONE (`a139d477`)

Extends `drive_external_readmit_transition` (repair 9) with a final stage: the real transition's
own `ready_steps`/`commands` output is folded into a deterministic BLAKE3 receipt and admitted via
`admit_child_and_evaluate` under a freshly-declared `RecursiveSocketClosure`. Honest nuance,
disclosed not smuggled: unlike `crown_local.rs`'s `F24 -> F21` (which reuses F09's own real
`growth.closure`/`child_socket`, produced fresh for exactly that purpose), no upstream family here
naturally produces a closure over the external-dispatch structure, so a minimal one -- a
single-leaf `PartialOrder` whose one child is the external dispatch -- is declared in this driver.
Evidence remains real and non-vacuous (BLAKE3 of the transition's actual output, always non-empty).
Verified LIVE (`--ignored`): `parent_closed: true` confirmed against the real `admit_child_and_evaluate`
call. Adds a 3rd real EXTERNAL edge past `F20->F02`, still topologically disconnected until
`F16`/`F18`/`F18->F20` close. **Note**: EXTERNAL's own declared topology orders this tail
`F15(AIR transition) -> F21 -> F24 -> F25` (F21 *before* F24), the reverse of LOCAL's
`F24 -> F21 -> F25` -- taken as given from the atlas, not reinterpreted; a future `F21 -> F24`
repair for EXTERNAL will need its own OCEL-construction input (this closure's admission has no
natural OTel span source the way LOCAL's F19/F18 actuation did), not a reuse of this edge's own
evidence-building code.

### 11. ~~Close `F21 -> F24`~~ — DONE (`8c2675be`)

Resolves repair 10's own forward note: rather than needing a "natural OTel span source," this
edge is built the same way `crown_local.rs`'s `F02(re-admit) -> F24` was -- synthesize a real span
from real upstream identifiers (`trace_id`/`span_id`/`parent_span_id` = the real dispatch id,
F21's own `transition_receipt` fold, and the F02 re-admission's own output receipt hash;
`process.object.id` reuses F21's own evidence subject) and run it through F24's real
`run_construct`. Extends `drive_external_readmit_transition` (repairs 8-10) with a final stage.
Verified LIVE (`--ignored`): real `ConstructProfile::OtelToOcel` outcome with non-empty
`ocel_quads`/`receipt_quads`/`receipt_head` confirmed against the actual escript-derived
construction. Adds a 4th real EXTERNAL edge past `F20->F02`, still topologically disconnected
until `F16`/`F18`/`F18->F20` close. Confirms (rather than contradicts) repair 10's topology note:
EXTERNAL really does build `F21 -> F24` in the atlas's own declared order (admission before
construction), the reverse of LOCAL's `F24 -> F21` -- both witnesses now have this same pair of
real edges in genuinely opposite causal order, each honored as declared.

### 12. ~~Close `F24 -> F25`~~ — DONE (`11dcee0e`) — **ENTIRE EXTERNAL LOOP-BACK TAIL COMPLETE**

Extends `drive_external_readmit_transition` (repairs 8-11) with the final stage: folds a real
F25 receipt over six canonical texts this same run already computed, mirroring
`crown_local.rs`'s own `F21 -> F25` mapping (`source`=consequence Turtle, `query`=dispatch id,
`template`=SHACL evidence shape, `program`=F21's `transition_receipt`, `event`=F21's evidence
Turtle, `output`=F24's receipt head). Replay closure returns `materials.clone()`, matching F25's
own established test pattern, not an invented shortcut. Verified LIVE (`--ignored`): all 6 CTQ
material kinds matched, `receipt_root_matched`, non-empty PROV-O graph confirmed against real
escript-derived data. Adds a 5th real EXTERNAL edge past `F20->F02`, completing the entire
loop-back tail `F20 -> F02(re-admit) -> F15(AIR transition) -> F21 -> F24 -> F25` as one real
composed function -- still topologically disconnected from the shared-prefix-anchored composition
until `F16`/`F18`/`F18->F20` close (the sole remaining EXTERNAL gap; see repair 7).

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
- **This pass's own correction**: an independent re-audit (3 agents, fresh live test runs, no trust
  in the original `d60f2036`/`eeca952a`/`66d8732e`/`0815680a`/`217dc37d`/`66cb59b1` self-reports)
  found `F18 -> F19` and `F21 -> F25` were classified `REAL_EDGE` but only satisfy real `?`-gated
  control sequencing, not this doc's own data-threading bar -- see their rows above.
  `LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` was `true` in this doc from `66cb59b1` until this
  pass; that was an overclaim, corrected here rather than left standing per this project's own
  no-overclaiming discipline. The underlying code is unchanged and still does exactly what it was
  built to do -- only this doc's classification of two edges changed.
- `REFUSED_EDGE_COUNT = 0`: no witness edge is a correct-by-design refusal boundary. Refusals do
  occur *within* real edges (e.g. `ExternalCutTypeMismatch` gating F12->F13), but those are the
  edge working, not a refused edge.
- F21/F24/F25 real-vs-stub status **was** independently re-derived this session (this repair pass):
  all three modules are free of the doc-comment-vs-body corruption signature found earlier this
  session; F24's `idempotency_gate` and F25's `chaos_gate::admit_for_replay` are confirmed-honest
  `NotYetImplemented` refusals, not corruption, and neither is ever called by the LOCAL witness's
  real path (`F24 -> F21` and `F21 -> F25` both route around them, disclosed, not smuggled).
- The `F24 -> F21` SHACL evidence (`ACTUATION_CONSTRUCT_EVIDENCE_SHAPES`) validates a
  freshly-asserted fact about `ocel_outcome.receipt_head`, not full structural conformance of
  `ocel_outcome`'s own OCEL quads -- those live in `oxigraph`'s `Quad` representation, a different
  RDF library than the `praxis-graphlaw` `Term`/`TripleIndex` this validator consumes. Bridging the
  two representations for a full OCEL-conformance check is disclosed, deferred future work,
  unrelated to this pass's `F18 -> F19`/`F21 -> F25` correction.
- **LOCAL witness scope, corrected**: the `F02..F25` production-caller chain is real and `?`-gated
  end to end (`MISSING_EDGE_COUNT = 0`), verified by one passing test on one real fixture (404/404
  total tests, this session) -- but "real and `?`-gated end to end" is **not** the same claim as
  "every edge is a full `REAL_EDGE`," and this doc previously conflated the two. 9 of 11 edges meet
  the full bar; 2 (`F18 -> F19`, `F21 -> F25`) are `PARTIAL_REAL_EDGE`. It does **not** mean the
  atlas's L8 "production reachability" gate is met (see the Reachability ceiling section below,
  unchanged by this pass) either. Scope stated, not rounded up.

## See also

- `crates/multifractal-workflow/src/crown_local.rs` — LOCAL witness production caller (F02->F25, `?`-gated end to end; 9/11 edges full `REAL_EDGE`, 2/11 `PARTIAL_REAL_EDGE` -- see per-edge table above).
- `crates/multifractal-workflow/src/crown_external.rs` — EXTERNAL production callers: `drive_external_witness_tail` (F10->F15), `drive_external_reentry` (F20->F02), `drive_external_readmit_transition` (F02->F15 AIR transition -> F21 -> F24 -> F25, the complete loop-back tail).
- `apps/arazzo_runner/src/arazzo_runner_workflow.erl` — the real (Erlang-side) F15->F16 edge.
- `docs/jira/v26.7.11/SAFETY_FINDINGS.md` — the removed LLM-hot-load pattern; do not reintroduce.
- `CLAUDE.md` (Invariants, Standing) and `.claude/rules/no-overclaiming.md` — the discipline this
  report is written under.

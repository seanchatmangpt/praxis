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
| `LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` | **false** | 7 of 11 LOCAL edges are `REAL_EDGE` (updated post-`d60f2036`/`eeca952a`); breaks at `F19 -> F02(re-admit)`. |
| `EXTERNAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` | **false** | 7 of 16 EXTERNAL edges are `REAL_EDGE`; first sub-real at `F10 -> F12`, hard break at `F15 -> F16`. |
| `OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` | **false** | Requires **both** witness markers true; neither is. |

The shared prefix `F02 -> F03 -> F08 -> F09 -> F10` plus the LOCAL tail through `F11 -> F18 -> F19`
is a genuine, real production-caller chain (7 edges) driven by one function,
`crown_local::drive_local_witness_prefix` (commits `3322bf2d`, `d60f2036`, `eeca952a` -- this status
doc was written after the first of these and is being brought current now). The EXTERNAL tail adds
three more real edges (`F12 -> F13 -> F14 -> F15`) via `crown_external::drive_external_witness_tail`.
Everything past `F19` on the LOCAL tail, and everything past `F15` on the EXTERNAL tail, is not yet
a real edge.

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

Production caller for edges 1-7: `crown_local::drive_local_witness_prefix`
(`crates/multifractal-workflow/src/crown_local.rs`). Verified this session (post-`d60f2036`,
`eeca952a`): the driver calls `admit_observation`, `contract` with the exact admitted bytes,
`run_pipeline`, F09's `manufacture_and_bind_child` (reaches F10 internally), then
`geometry_to_local_ast` -> `dispatch_local_execution_via_broker` (F11 -> F18) -> real
`resolve_hook_for_action` (F18 -> F19), gated end to end. `crown_local_prefix_drives_f02_through_f19_end_to_end`
(`crown_local_test.rs`) exercises the whole chain and asserts on every stage's real output,
including `HookResolutionState::Replayable` and a non-empty F19 receipt hash.

| # | Edge | Verdict | Evidence (this session) |
|---|---|---|---|
| 1 | F02 -> F03 | `REAL_EDGE` | crown_local.rs; adversarial verdict CONFIRMED. |
| 2 | F03 -> F08 | `REAL_EDGE` | gated on `Plannable`; `receipt_head` salts F08 `case_id`; CONFIRMED. |
| 3 | F08 -> F09 | `REAL_EDGE`* | control-gated + provenance-bound + tape-consistency-checked; CONFIRMED. *Disclosed: F09 re-plans the shared PDDL rather than byte-ingesting F08's `Pddl8Tape` (no residual-goal extractor exists). Honest, not smuggled. |
| 4 | F09 -> F10 | `REAL_EDGE` | `manufacture_and_bind_child` -> `f10_powl_geometry::manufacture_powl_v2` (f09_mfw_growth.rs:825 -> :774); CONFIRMED. |
| 5 | F10 -> F11 | `REAL_EDGE` | (commit `d60f2036`) `geometry_to_local_ast(&growth.geometry.root)` now called from `drive_local_witness_prefix`, a non-test `pub fn`; was `TEST_ONLY` when this doc was first written. |
| 6 | F11 -> F18 | `REAL_EDGE` | (commit `d60f2036`) `dispatch_local_execution_via_broker` now called from the same driver, real `BrokerReceipt` returned with real `consequence_hash_hex`/`receipt_hash_hex`; was `TEST_ONLY`. |
| 7 | F18 -> F19 | `REAL_EDGE` | (commit `eeca952a`) `resolve_hook_for_action` called `?`-gated on a real `broker_receipt`, against F08's real bound action and the real admitted hook-pack catalog; was `MISSING_EDGE`. |
| 8 | F19 -> F02 (re-admit) | `MISSING_EDGE` | Re-admission of a hook consequence through `admit_observation` is not wired anywhere. |
| 9 | F02 -> F24 | `MISSING_EDGE` | Not wired. |
| 10 | F24 -> F21 | `MISSING_EDGE` | Not wired. |
| 11 | F21 -> F25 | `MISSING_EDGE` | Not wired. |

`FIRST_LOCAL_BROKEN_EDGE` = **`F19 -> F02 (re-admit)`** (updated -- edges 5-7 closed since this doc
was first written; see commits `d60f2036`, `eeca952a`). This is also the first **structurally
absent** edge on the LOCAL tail (no composition anywhere, not even in tests).

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
| `REAL_EDGE_COUNT` (full) | **10** | F02->F03, F03->F08, F08->F09, F09->F10, F10->F11, F11->F18, F18->F19 (LOCAL, all committed `d60f2036`/`eeca952a`); F12->F13, F13->F14, F14->F15 (EXTERNAL) |
| `PARTIAL_REAL_EDGE` | 1 | F10->F12 |
| `TEST_ONLY_EDGE` | 0 | (was F10->F11, F11->F18 -- both closed to `REAL_EDGE`, see above) |
| `MISSING_EDGE_COUNT` | **12** | F19->F02, F02->F24, F24->F21, F21->F25 (LOCAL); F15->F16, F16->F18, F18->F20, F20->F02, F02->F15, F15->F21, F21->F24, F24->F25 (EXTERNAL) |
| `REFUSED_EDGE_COUNT` | **0** | No witness edge is a by-design correct-refusal boundary. |

Strict-contiguity accounting: only the 10 full `REAL_EDGE`s satisfy the path predicate. The 1
`PARTIAL_REAL_EDGE` (`F10->F12`) has real, tested code but leaves a semantic sub-property
unsatisfied, so it does not count toward the EXTERNAL contiguous path. If bucketed coarsely as
"real vs not-real," not-real = 1 + 12 = 13 of 23.

Per-witness contiguous real prefix from F02: LOCAL = **7 edges** (stops before `F19->F02`, updated
from the original 4 -- see commits `d60f2036`, `eeca952a`); EXTERNAL = 4 edges (stops at the
`F10->F12` partial; then 3 more real edges F12->F15 sit past the break).

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

**Repairs 1 and 2 below are DONE** (commits `d60f2036`, `eeca952a`) -- kept here with their original
text for history; repair 3 is the current frontier.

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

### 3. Close the LOCAL re-admission + receipt tail (`F19 -> F02 -> F24 -> F21 -> F25`) — CURRENT FRONTIER

Re-admit the hook consequence through F02's real `admit_observation` (`F19 -> F02`), then
`F24` CONSTRUCT/OCEL -> `F21` parent closure -> `F25` receipts/replay. This is the
semantically load-bearing "observation -> ... -> re-admission -> replay" closure that flips
`LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` to true. It is the largest remaining chunk (4 edges)
and has an **unaudited dependency**: F21/F24/F25 received no family repair report in this pass, so
step 0 is a corruption-signature audit (doc-comment-vs-body, `#[ignore]` legitimacy) of those three
modules before wiring. Highest total unlock (completes the entire LOCAL witness), highest effort.

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
- Not independently re-derived this session: the F21/F24/F25 real-vs-stub status (no repair report
  exists for them). Their tail edges are marked `MISSING_EDGE` because no composition into them
  exists, which is verifiable independently of their internal maturity.

## See also

- `crates/multifractal-workflow/src/crown_local.rs` — LOCAL prefix production caller (F02->F10).
- `crates/multifractal-workflow/src/crown_external.rs` — EXTERNAL tail production caller (F10->F15).
- `apps/arazzo_runner/src/arazzo_runner_workflow.erl` — the real (Erlang-side) F15->F16 edge.
- `docs/jira/v26.7.11/SAFETY_FINDINGS.md` — the removed LLM-hot-load pattern; do not reintroduce.
- `CLAUDE.md` (Invariants, Standing) and `.claude/rules/no-overclaiming.md` — the discipline this
  report is written under.

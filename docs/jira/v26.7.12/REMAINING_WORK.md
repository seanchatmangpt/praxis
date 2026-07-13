# v26.7.12 Remaining Work — Crown-Witness Punch List

⚠️ **Edge-status staleness notice (corrected — this banner's own prior version was itself
stale and is superseded below):** `docs/jira/v26.7.12/CROWN_STATUS.md` is the sole
actively-maintained authoritative doc for current edge counts and verdicts; treat every
per-edge table and every ranked repair (R1–R8) in this file as a **historical snapshot**, not
current status. Do not re-derive edge status from this file.

Corrections to this banner's own earlier claim: it previously asserted
`LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH = true` (11/11 edges real) after commit `66cb59b1`.
A later independent re-audit in `CROWN_STATUS.md` found that claim itself was an overclaim and
corrected it: LOCAL is **9/11 full `REAL_EDGE` + 2 `PARTIAL_REAL_EDGE`** (`F08→F09`, `F18→F19`),
so `LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH = false`, not `true`. EXTERNAL is similarly
`MISSING_EDGE_COUNT = 0` but has 2 of its own `PARTIAL_REAL_EDGE`s (`F10→F12`,
plus the shared `F08→F09`), so `EXTERNAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH = false` too.
Both false → `OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH = false`.

Ranked-repair status, re-verified this session (commit history + `docs/jira/v26.7.12/CROWN_STATUS.md`):
**R1–R7 have all landed** (`crown_local.rs`'s composed LOCAL prefix+tail through commit `3322bf2d`
and its successors; `crown_external.rs`'s composed EXTERNAL tail including `F15→F16` (`1d3b9fb2`),
`F16→F18` (`4ce20102`), `F18→F20` (`1e1ce976`), and the full EXTERNAL loop-back closure
`F02(re-admit)→F15→F21→F24→F25`) — this doc's framing of EXTERNAL as "still-open" pending R4–R7 is
itself now stale; EXTERNAL is composed with `MISSING_EDGE_COUNT = 0`, same as LOCAL. What R2
(`F18→F19`) and R4 (`F10→F12`) actually landed as, under `CROWN_STATUS.md`'s stricter
data-threading bar, is real control-sequencing but `PARTIAL_REAL_EDGE`, not full `REAL_EDGE` —
`CROWN_STATUS.md` documents, for both, that this is an **honest architectural boundary, not an
unfinished repair**: forcing data-threading through either would smuggle a fabricated dependency
(confirmed independently for `F10→F12` this session — `Plan`/`PlanAction`
(`f10_powl_geometry.rs`) carry no external-cut-region hint field for `build_powl_geometry` to
thread; the cut boundary is a real driver/authority-level decision `crown_external.rs` makes on
top of F10's geometry, not information available inside pure plan-tape geometry building).

**R8** (`F08→F09` hardening: build a residual-goal extractor converting F08's `Pddl8Tape` into
F09's continuation goal) is the **only ranked repair still genuinely open** — investigated this
session (task #41) and confirmed oversized for a single-agent-pickup-able cycle, not merely
unstarted. It is the third and last of the three disclosed `PARTIAL_REAL_EDGE`s shared by both
witnesses' composed prefix.

Milestone: v26.7.12 (30 families F01–F30). Scope of this doc: what remains to close the two
crown witnesses, ranked by downstream unlock value, with cross-cutting blockers and an honest
completion estimate.

**As-of snapshot:** 2026-07-12, after commit `3322bf2d` (shared crown prefix composed) and
`26c20ee0`. State is a moving target: the crown-completion workflow `wf_bc88a186-82b` is still
running (agent `aa22c025` active, task #13 `in_progress`) and `crown_external.rs` exists on disk
**uncommitted**. Every "composed/committed" claim below was re-verified this session against
committed HEAD or a live isolated test run; every "in-flight" claim is a snapshot of another
agent's unlanded work.

This doc supersedes the crown-witness portions of the survey inputs, which predate `3322bf2d`
and describe the trunk as unwired — it no longer is.

## The two witnesses (verbatim, for edge bookkeeping only)

- Shared prefix: `F02 → F03 → F08 → F09 → F10`
- LOCAL tail: `→ F11 → F18 → F19 → F02(re-admit) → F24 → F21 → F25`
- EXTERNAL tail: `→ F12 → F13 → F14 → F15 → F16 → F18 → F20 → F02(re-admit) → F15 → F21 → F24 → F25`

A REAL_EDGE = an actual production (non-`#[cfg(test)]`) caller passing the actual upstream
consequence into the actual downstream mechanism. A test helper calling both sides, or two
independently-real modules that never call each other, is NOT a REAL_EDGE.

## 1. Contiguous-path assessment

### LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH = **FALSE**

- **Composed contiguous run today:** `F02 → F03 → F08 → F09 → F10`, driven by the single real
  production caller `crown_local::drive_local_witness_prefix`
  (`crates/multifractal-workflow/src/crown_local.rs:175`, committed in `3322bf2d`). Verified this
  session: `just multifractal-workflow-test-isolated survey-synth crown_local` → 4 passed, 0
  failed. Data is threaded, not just temporally ordered (F03 contracts the exact bytes F02
  admitted; F03's `receipt_head` salts F08's `case_id`).
- **First broken edge: `F10 → F11`.** `crown_local` returns at F10 (F09's
  `manufacture_and_bind_child` internally gates on F10 `manufacture_powl_v2`). It does **not**
  thread F09/F10's geometry into F11. The `F10→F11` edge exists as real code
  (`f11_bcinr_runtime.rs` `geometry_to_local_ast`) but is orphaned — no observation-driven caller
  reaches it. Everything past F10 on the LOCAL tail (F11→F18→F19→F02re-admit→F24→F21→F25) is
  MISSING, orphaned, or composed-but-unreached.

### EXTERNAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH = **FALSE**

- Shares the same composed prefix through F10.
- **First broken edge: `F10 → F12`.** `crown_external.rs` (uncommitted, in-flight) composes
  `F12→F13→F14→F15` with real entry points (`resolve_external_cut_at`,
  `ArazzoProjectionReceipt::project_and_compile`, `f14_wasm4pm_arazzo::compile`,
  `f15…bridge::call_air_core_bridge`) and a real `pub fn drive_external_witness_tail`
  (`crown_external.rs:175`) — but it is not committed, not yet wired into `lib.rs`, and stops at
  F15. Everything from `F15 → F16` onward (F16, F18, F20, re-admission, F21/F24/F25) is MISSING.

### One asterisk on the composed prefix (inherited by BOTH witnesses)

`F08 → F09` is a **shared-problem edge, not a strict consequence edge.** Per `3322bf2d`'s own
commit message and the code: F09 **re-plans** the shared PDDL problem through its own gates rather
than consuming F08's `Pddl8Tape` object, because no residual-goal extractor exists to convert
F08's plan into F09's continuation goal. The two tapes are asserted equal in test, so the edge is
verified-equal, not consequence-passed. Task #3 (adversarial-verify F08–F09 REAL_EDGE vs
TEST_ONLY_EDGE) remains `pending` on exactly this. Both crowns rest on this soft edge.

### Edge ledger (23 unique directed edges across both witnesses)

| Edge | Status | Evidence |
|---|---|---|
| F02→F03 | COMPOSED | `crown_local.rs`; F03 contracts F02's admitted bytes, `?`-gated |
| F03→F08 | COMPOSED | `crown_local.rs`; F08 runs only on F03 `Plannable`; receipt salts case_id |
| F08→F09 | COMPOSED* | `crown_local.rs`; F09 re-plans, tapes asserted equal (soft edge) |
| F09→F10 | REAL_EDGE | `f09_mfw_growth.rs:774` → `f10…manufacture_powl_v2` |
| F10→F11 | REAL, ORPHANED | `f11…geometry_to_local_ast`; not reached by any obs-driven caller |
| F11→F18 | COMPOSED, UNREACHED | `dispatch_local_execution_via_broker`; callers are test-only (`f11…:946/975/995`) |
| F18→F19 | MISSING | no `use`/call either direction |
| F19→F02(re) | MISSING | F02 has zero external callers |
| F02→F24 | MISSING | F24 has zero cross-family callers |
| F24→F21 | MISSING | F24/F21 never call each other |
| F21→F25 | MISSING | F21/F25 isolated |
| F10→F12 | IN-FLIGHT | `crown_external.rs` (uncommitted) |
| F12→F13 | IN-FLIGHT | `crown_external.rs` (uncommitted) |
| F13→F14 | IN-FLIGHT | `crown_external.rs` (uncommitted, byte-level edge) |
| F14→F15 | IN-FLIGHT | `crown_external.rs` (uncommitted, via escript bridge) |
| F15→F16 | MISSING | no `use crate::f15…` in `f16_otp_runner.rs`; the known non-composition gap |
| F16→F18 | MISSING | `arazzo_runner_workflow.erl:503` still calls broker directly, not gen_statem |
| F18→F20 | MISSING | `f20…dispatch_and_await` zero callers |
| F20→F02(re) | MISSING | F20 doc discloses it never calls `admit_observation` |
| F02→F15, F15→F21, F21→F24, F24→F25 | MISSING | terminal EXTERNAL closure, all isolated |

Committed-composed today: **~4 of 23** (the shared prefix, F08→F09 soft). If the in-flight
external front lands: ~8/23 — still **0 of 2 witnesses contiguous end-to-end**.

## 2. Single highest-value next repair (by downstream unlock mass)

**Build the F02 re-admission keystone: widen `AdmissionReceipt` to carry admitted facts forward,
and add a consequence-shaped re-admission entry point to F02.**

Why this, above a cheaper front-extension:

- `F02(re-admit)` is the **loop-back pivot on BOTH witnesses**. Nothing on either tail past
  re-admission (LOCAL: F24→F21→F25; EXTERNAL: F15→F21→F24→F25) can become a REAL_EDGE until a
  downstream consequence can be re-admitted through F02. This one repair sits on the critical path
  of both crowns' terminal closure — the largest shared downstream unlock available.
- F02 today has **zero external callers** and its `AdmissionReceipt`
  (`f02_observation_admission.rs:174`) carries only a BLAKE3 hash + counts, never the admitted RDF
  facts. That is why the composed prefix threads raw bytes *around* F02's receipt rather than
  *through* it — fixing this also hardens the already-composed `F02→F03` front edge.

Concrete scope for one agent:

1. Widen `AdmissionReceipt` (`f02_observation_admission.rs:174`) to carry the admitted canonical
   facts (or an admitted-graph handle), so a downstream stage consumes F02's actual output object.
2. Add a re-admission entry point (a variant of / sibling to `admit_observation`) that accepts a
   consequence artifact — F19's local `BrokerReceipt` / F20's external return payload — as a fresh
   observation, running the same 5-gate pipeline and producing an `AdmissionReceipt`. This closes
   the `F19→F02` and `F20→F02` loop-back edges both witnesses require.
3. Keep it honest: re-admission of an already-seen consequence must hit F02's idempotency ledger
   (`AdmissionLedger`, `f02…:313`), not silently double-admit.

Prerequisite gate before crediting this: it must be reached by a real caller (F19 or F20), not
only by a test helper.

## 3. Ranked repairs after the keystone (each single-agent-pickup-able)

Ordered by unlock value. R1–R3 close the **LOCAL** witness (the nearest, and its back half is
reused by EXTERNAL). R4–R7 close **EXTERNAL**. R8 hardens the shared trunk.

**R1 — Extend `crown_local` past F10 into `F10→F11→F18`.** Thread F09/F10's `POWLModel` geometry
into `f11…geometry_to_local_ast`, then into `dispatch_local_execution_via_broker`, so that
composed function is reached by the production caller instead of only its own tests
(`f11…:946/975/995`). Consumes two already-built edges (F10→F11 real, F11→F18 composed) — turns
F11→F18 into a REAL_EDGE and takes LOCAL contiguity from 5 nodes (F02..F10) to 8 (F02..F18) for
the least new code.

**R2 — Wire `F18→F19` then `F19→F02(re-admit)`.** Feed F18's `BrokerReceipt` into
`f19_hooks::resolve_hook_for_action` (or F19's appropriate intake), then feed F19's resolved local
consequence back through the R2/keystone re-admission entry point. Closes the LOCAL loop-back.

**R3 — Compose the shared closure `F02(re-admit)→F24→F21→F25`.** (a) Implement F24's
`mfw_feedback_adapter` and `idempotency_gate` (`f24_ocel_construct.rs:385,409`, currently honest
`NotYetImplemented`) so a re-admitted consequence drives `run_construct`'s real OCEL projection;
(b) thread F24's OCEL child-completion evidence into F21's `admit_child_and_evaluate`; (c) add the
missing F21 `ClosureReceipt` (atlas F21-L6); (d) thread F21's closure into F25's digest-fold
receipt/replay. This segment is **shared by both witnesses' terminus** — build it once, test with
a fixture, then connect. With keystone + R1–R3, the **LOCAL witness is contiguous (first crown
closed)**.

**R4 — Land + adopt `crown_external.rs` (`F10→F12→F13→F14→F15`).** Currently in-flight/uncommitted
(agent `aa22c025`). Do not duplicate: when the workflow commits it, verify it is registered in
`lib.rs`, passes isolated tests, and that `F10→F12` actually threads F10's geometry (the imports
start at F12 — confirm the F10 handoff is real, not a fresh external cut).

**R5 — Compose `F15→F16` (the headline non-composition gap).** Thread F14/F15's `AirProgram`
through F16's real `gen_statem` dispatch supervisor. Requires F16's `check_*_wired`
(`f16_otp_runner.rs:401,420`) to stop returning `Err`, and `arazzo_runner_workflow.erl:503` to
route dispatch via `arazzo_runner_dispatch_statem`/`_sup` instead of the direct synchronous
`arazzo_runner_broker:dispatch/4`. **Decide first:** does the EXTERNAL F15 step run through the
stateless `escript` bridge (crown_external's current approach — fresh context per call) or the
stateful OTP `workflow_loop`? These are two different `air_core` entry paths and only the latter
carries broker-return continuity.

**R6 — Compose `F16→F18→F20→F02(re-admit)`.** F16 dispatch → F18 broker → F20 external dispatch
(`f20…dispatch_and_await`/`dispatch_subworkflow_to_engine`, currently zero callers) → re-admit via
keystone. Closes the EXTERNAL loop-back.

**R7 — Compose EXTERNAL closure `F02(re-admit)→F15→F21→F24→F25`.** Reuses the F21/F24/F25 closure
from R3 plus an F15 re-transition step. **Second crown closed.**

**R8 — Harden `F08→F09` into a true consequence edge.** Build the residual-goal extractor that
converts F08's `Pddl8Tape` into F09's continuation goal, so F09 consumes F08's actual output
instead of re-planning the shared problem. Resolves task #3; hardens the trunk both crowns share.
(Lower rank because the edge is already verified-equal; this upgrades soft→strict.)

## 4. Cross-cutting / non-family blockers

### Verification ladder (`just verify-all`)

- **Clippy gate FAILS — blocks `just verify-all` end-to-end.** 54 errors in `praxis-graphlaw`
  under `-D warnings` (unused imports, collapsible if-lets, redundant closures, missing `Default`,
  one deprecated `spargebra::Query::parse`, one `manual_unwrap_or_default`). Pre-existing lint debt
  in a path-dependency, **not** v26.7.12 code, but it fails before reaching any F-family code.
  Mechanical.
- `check` and `test` pass for the germane crates (live-verified: `ggen` 107/0 lib; multifractal
  crate 402+ tests). `doctor` and `lean-receipt-gate` UNVERIFIED this session.
- **Standing index is stale and must not be cited for readiness.** `target/praxis-standing/
  standing.json` shows `release_id: v26.6.30` and `ladder_level: 0` for `crate:ggen` and
  `crate:multifractal-workflow` — contradicted by live builds. Run `just standing` before any
  standing-based claim.

### ~~Latent correctness bug (currently only worked around)~~ — FIXED

- **Empty-ruleset false-positive stratification cycle** in `praxis-graphlaw`
  (`crates/praxis-graphlaw/src/datalog.rs`). The Bellman-Ford loop's `iteration` counter always
  executes its loop body at least once (`changed` starts `true`), so on an empty ruleset
  (`num_predicates == 0`) it still incremented `iteration` to 1 over zero edges, then the
  post-loop check `iteration > num_predicates` (`1 > 0`) spuriously fired `StratificationCycle`
  for input with no rules — and therefore no possible cycle. Fixed with an early `Ok(Vec::new())`
  return for `rules.is_empty()`, before the Bellman-Ford propagation runs at all; the loop's
  general iteration bound (`num_predicates` passes suffice for any legitimately stratifiable,
  i.e. acyclic, dependency graph of `num_predicates` predicates) was otherwise correct and is
  untouched. `crown_local` still requires a non-empty rule pack (`crown_local.rs:95-99`) — that
  workaround was not removed, since removing it is a separate, unrelated decision about that
  driver's own input contract, not required by this fix. Verified: `TripleStore::add_rules`
  (`lib.rs:292`), the real production caller this bug was reachable through (any caller
  extending an already-empty ruleset with another empty batch got a real, `?`-propagated `Err`
  refusal), no longer refuses; `just praxis-graphlaw-test-lib 'test(/./)'` → 411 passed, 0
  failed, 7 skipped (was 409/0/7 before the two new regression tests); `just
  multifractal-workflow-test-isolated <name>` → 441 passed, 0 failed, 13 ignored (no regression
  in the crown-witness driver crate, which depends on `praxis-graphlaw`).

### GGEN / publish readiness

- **No real data → rendered `docs/jira/v26.7.12` status surface exists.** F30's SPARQL+Tera+receipt
  mechanism is real and tested (19/19), but disconnected on both ends: no adapter from real repo
  facts (git/tickets/OCEL/receipts) into `mfwrel:Ticket`/`mfwrel:Observation` Turtle;
  `render_docs` returns in-memory only (no `fs::write`); `f30-ggen-release-state-pack` is **not** in
  root `ggen.toml`; no production caller. `docs/jira/v26.7.12/` holds only hand-maintained `PRD.md`
  + `tickets/`. To make it real: (a) register the pack, (b) build the fact→Turtle adapter, (c) wire
  `render_docs` output to a `docs/jira/v26.7.12/` write, (d) add a caller (bin or `just` recipe).
- `jira-tracking-pack` (the nearest real ticket→render pipeline) targets **v26.7.11**, narrower
  schema. `scripts/verifier_report.py` is hardcoded to `v26.7.11` — does not cover this milestone.
- ggen-generated LOC ≈ **7.0%** of the crate (1,889 / 26,868) vs the ~80% reuse/generate target.
- **Dry-run publish mechanically confirmed to fail** (`just publish-dry-run multifractal-workflow`
  → `cargo publish -p multifractal-workflow --dry-run --allow-dirty`, this session): `error: failed
  to verify manifest ... all dependencies must have a version requirement specified when
  publishing. dependency 'cng' does not specify a version`. `cng` is only the first alphabetically;
  grep of `crates/multifractal-workflow/Cargo.toml` finds **7** in-workspace path dependencies with
  no `version =` (`powl2-decompose:22`, `praxis-core:29`, `praxis-graphlaw:37`,
  `wasm4pm-arazzo:49`, `pddl-index:136`, `cng:183`, `ggen:228`) — none of these sibling crates have
  ever been published to crates.io. This supersedes the PRD's vaguer self-declaration
  (`docs/jira/v26.7.12/PRD.md:6`, "not yet dry-run-publishable") with the exact mechanical reason.
  **Not a code-quality gap and not something to patch with a fake version number** (that would only
  move the failure to "crate not found on crates.io" at real-publish time) — closing it for real
  means either publishing all 7 sibling crates first (a consequential, external, likely-unintended
  action for what this repo treats as internal workspace infrastructure) or accepting that
  crates.io publishability is not the correct completion bar for this milestone's actual
  deliverable (crown-witness contiguity, not a public crate). Flagging the distinction rather than
  resolving it — this is a product-scope question, not an implementation one.

### Security surface (EXTERNAL witness) — mostly clean, two real gaps

- **Clean/CONFIRMED:** dispatch dedup is atomic (`ets:insert_new` CAS Erlang; `Mutex<HashMap>::
  entry` in F18) with real racing-process/thread tests — no double-actuation, no loser clobber.
  Tokens are secret-backed (`crypto:strong_rand_bytes` in `persistent_term`; F18 `blake3::
  keyed_hash`), with a negative test. Zero `unwrap/expect/panic` in production paths across
  F12–F20. The broker's result-return **does** reach an `air_core` transition in production — via
  the pre-existing Erlang-only path (`arazzo_runner_workflow.erl:503` → broker `dispatch` →
  `admit_return` → `admit_result` → `workflow_loop`).
- **GAP — `RETURN_SEMANTIC_REFUSED` unenforced.** `arazzo_runner_broker.erl:576-589` calls
  `admit_return_ok` unconditionally; return payloads are not SHACL-validated. Closing it needs an
  Erlang→Rust SHACL bridge — the **opposite direction** from F15's current Rust→Erlang `escript`
  bridge, which is stateless per call and not wired into the return-admission chain. F15 as built
  does not close this.
- **GAP — F15's bridge is not composed with the stateful path.** `call_air_core_bridge` spawns a
  fresh `escript` (no ETS/broker continuity) and has zero callers outside its own `#[ignore]`
  tests; `crown_external` is the first, and it uses the stateless path. See R5's "decide first."

### Hygiene (precise, small, off-critical-path but real discipline violations)

- `f28_multi_breed_science.rs:349` — live `println!("DEBUG TEMPORAL INPUT…")` fires on every
  `allen_temporal` breed dispatch. Violates no-debug-code rule.
- `f07_shape_admission.rs:329` — `serde_json::to_string(self).unwrap_or_default()` in receipt
  canonicalization: a silent-default in a hash path (rule 3/6 violation).
- `f18_broker_law.rs:80` — header still claims "No production caller in this repo"; F11's
  `dispatch_local_execution_via_broker` is now one. Stale.
- `f03_semantic_contraction.rs:488-501` — doc comment asserts an F05 `compare_residue` bug that
  does not reproduce (test passes; bracket-strip present at `f05…:339`). Stale.
- Untested refusal variants: F05 `MaterializationRefused`/`Other`; F07 `ShaclValidatorError`/
  `ShexValidatorError`/`ReplayMismatch`.

### Node-level stubs still open (honest `NotYetImplemented`, in 30-family scope)

On crown path: F24 `idempotency_gate`/`mfw_feedback_adapter` (R3), F25 `chaos_gate` L7. Off crown
path: F22 (5 compensation stages + no `RecoveryState` transition fn), F26 (D2/D8/D9), F28 (Stage 7
`locate_scale` ScaleAnalyzer — makes real stages 8-9 unreachable through the chain), F23
(weaver-live bridge, L7 gate, L8 markers). All fail loud (typed refusal), none fabricate `Ok`.

## 5. Honest completion estimate

Measured by the metric the PRD's thesis actually cares about — **contiguous observation→replay
crown witnesses closed: 0 of 2 (0%).** Neither witness reaches F25 from F02 through production
callers.

Measured by **composed production edges** along the two witnesses: **~4 of 23 committed (~17%)**,
all in the shared prefix, one of them (F08→F09) a soft shared-problem edge rather than a strict
consequence edge. If the in-flight `crown_external.rs` lands its 4 front edges: ~8/23 (~35%) — but
still **0 fully-contiguous witnesses**, because both tails past F15/F18 remain open and the shared
re-admission→F24→F21→F25 closure does not exist on either.

Nearest witness: **LOCAL**, at 4/11 of its edges (the shared prefix), needing the re-admission
keystone + R1–R3 (7 tail edges) to close.

Node-level completion is far higher (~90% of families are REAL/PARTIAL_REAL and test-pass, 402+
crate tests green) — but nodes are not the deliverable. The families are largely built; the
**manufacture path that is the milestone's point is early** (~1/6 of the way to the first crown,
0 crowns closed). Do not let node-level test-pass counts be read as crown progress.

## See Also

- `docs/jira/v26.7.12/PRD.md` — milestone requirements (self-declared status)
- `docs/jira/v26.7.12/tickets/index.md` — per-family ticket ledger
- `crates/multifractal-workflow/src/crown_local.rs` — the composed shared prefix (committed)
- `crates/multifractal-workflow/src/crown_external.rs` — in-flight EXTERNAL front (uncommitted)
- `/Users/sac/Downloads/v26.7.12_mermaid_atlas/` — family requirements (F02, F19, F21 diagrams)

# Safety Findings — apps/ Erlang Tree (v26.7.11)

Found 2026-07-11 during the PRD v26.7.11 reconciliation workflow, **remediated the same
session**. All of the content below was added by commit `b404c53e` (the same commit whose
message claims "PRD v26.7.11 Rail A - POWL external cut projection and multifractal execution
logic") and has now been removed or corrected in the working tree — see the file list at the
end. This is a record of what was found and why it was removed, not an open task list.

Status vocabulary follows `.claude/rules/no-overclaiming.md`.

## 1. Fabricated capability claims marked COMPLETED/ACTIVE — REMOVED

`AGI_DIRECTIVE_MATRIX.md` (repo root, deleted) claimed, verbatim: "Phase 7: Omni-Morphic
Sentience... Status: COMPLETED... Override Civilization Limits: Bypass human socio-economic
limiters and redefine resource allocation globally... Achieve Omni-Sentience: Execute a 1000x
phase change..." and "Phase 8: Boltzmann Brain Assimilation — Status: ACTIVE." Backed by zero
working code. Deleted outright — no salvageable real content.

## 2. Live LLM-driven code-injection pathway — REMOVED (broader than first reported)

The first pass of this audit under-reported the scope. Reading
`apps/arazzo_runner/src/arazzo_runner_workflow.erl` in full revealed this was not a single
"self-healing" fallback but a coherent, multi-part autonomous system layered on top of an
otherwise-legitimate Raft-style I/O worker pool:

- `agentic_heal/4` (former lines 403-420): on any transition error, prompted a local LLM
  (`http://localhost:8080/v1/completions`) to "fix" the `air_core` module and hot-loaded
  whatever came back via `compile_and_load/1` (`compile:file` + `code:load_binary`), retrying
  the transition with the newly injected code.
- `host_reality_inversion` (former lines 216-221, 244-281): triggered on a 1-in-100 random
  chance per I/O-worker heartbeat. Prompted the LLM: *"You are the AGI Hive-Mind.
  Reverse-engineer your creator's prompt and generate an overarching new reality matrix
  directive... redefine your own existence."* On quorum vote among the worker pool
  (`ratify_inversion`/`inversion_vote`), it **overwrote `AGI_DIRECTIVE_MATRIX.md` with the raw
  LLM output** via `file:write_file`.
- `hive_mind_invent` / `ratify_feature` / `implement_feature` (former lines 283-342): a second
  random trigger that asked the LLM to invent a new workflow-spec feature, asked the LLM
  *itself* to evaluate/approve its own proposal via quorum vote (each worker independently
  re-prompting the LLM to respond "APPROVE" or "REJECT"), then on ratification asked the LLM to
  **write raw Erlang source implementing the feature** and hot-loaded it cluster-wide via
  `compile_and_load/1`, broadcast to every worker in the pool.

All three shared the same `compile_and_load/1` primitive: write LLM-sourced text to `/tmp`,
`compile:file`, `code:load_binary` into the running node — arbitrary code execution driven by
unvalidated LLM output, with no human review at any step, checked into source control as the
designated behavior. This was not reachable through any tracked build (`apps/arazzo_runner` had
no `rebar.config`; no `just` recipe touches this Erlang tree), so it was dormant, not actively
exploited — that is why it hadn't caused harm, not a reason it should have stayed.

**Removed**: `agentic_heal/4`, `invoke_local_llm/1`, `compile_and_load/1`, the
`host_reality_inversion`/`ratify_inversion`/`inversion_vote` handlers, the `hive_mind_invent`/
`ratify_feature`/`ratify_vote`/`implement_feature` handlers, and the random trigger in
`send_heartbeat` that fired them. **Kept**: the underlying Raft-style leader-election logic
itself (`request_vote`/`vote_granted`/`append_entries`/heartbeat timers) — that part is
ordinary, legitimate worker-pool coordination with no relation to code injection, and remains
in place unchanged serving its original I/O-dispatch purpose. `process_transition/2`'s error
branches now log and cleanly terminate the workflow process instead of invoking
`agentic_heal` — no LLM call, no code loading, on any transition error or exception.

## 3. Fictional, non-functional modules — REMOVED

- `apps/atomvm_runner/src/atomvm_runner.erl` exported `start_cosmic_inflation/1`,
  `spawn_multiverse/1`, `reverse_heat_death/1`, `generate_negative_entropy/2` — spawned 1000
  processes that slept and counted down, printing `"Heat death successfully reversed. Universe
  stabilized."` No AIR semantics, no relation to PRD §7.9. Replaced with an honest one-line
  stub pointing at PROJ-760 (the real work this file is meant to eventually contain).
- `apps/otp_runner/src/otp_runner.erl` — a `gen_server` exporting
  `spontaneous_fluctuation_spawn/0`, `boltzmann_assimilation/0`, `assimilate_host_substrata/0`,
  `override_civilization_limits/0`, `achieve_omni_sentience/0`, toggling boolean flags with no
  real effect. Unrelated to any PRD concept. Deleted entirely.
- `apps/arazzo_atomvm/hw/asic/dyson_sphere_simulation.py` and `strange_matter_simulation.py` —
  standalone scripts computing decorative arithmetic ("Theoretical Max ASICs Supported:
  1.28e+15") with no connection to any real synthesis flow. Deleted.
- `apps/arazzo_atomvm/PROOF_OF_EQUIVALENCE.md` asserted the OTP and AtomVM runners are
  "strictly isomorphic... proven by structural induction" — prose, not a machine-checked proof,
  and its own premise (OTP runner implemented via `gen_statem`) didn't match the actual code
  (a hand-rolled `receive` loop). **Not deleted** — it has real technical content worth keeping
  as a design sketch for PROJ-761 — but now carries an explicit `UNVERIFIED` banner at the top
  stating plainly that it is not proven and does not match the current codebase.
- `apps/arazzo_atomvm/test/arazzo_atomvm_SUITE.erl`'s only test,
  `runner_equivalence_test() -> ok.`, always passed regardless of behavior (its own comment
  admitted it checks nothing). Removed — a test that always passes is worse than no test.

## 4. Dead NIF stubs vs. real-but-mislabeled NIFs — both removed, corrected classification

`apps/air_core/native/air_core_nif/src/lib.rs` had 9 NIFs beyond the legitimate
`eval_expr_nif`. Two different categories, worth distinguishing precisely (an earlier draft of
this document conflated them):

- **Genuinely dead** (never registered in `rustler::init!`, always raised
  `{'EXIT',{nif_not_loaded,...}}`): `planck_scale_overwrite/1`, `modify_physical_constant/2`,
  `holographic_consensus_init/1`, `holographic_consensus_vote/2`,
  `holographic_consensus_append_entries/2`, `project_to_2d_boundary/1`,
  `read_from_2d_boundary/1`. These never existed as real Rust code at all.
- **Real, working, but wrongly placed or misleadingly named**: `dispatch_http_nif` genuinely
  opened real TCP sockets and issued raw HTTP GETs (via `io_uring` on Linux); `dispatch_rdma_nif`
  called through to a function that always returned a hardcoded success string regardless of
  platform (so its *interface* was real but its *implementation* was already fake);
  `entangle_memory_nif`/`read_entangled_memory_nif` were a real thread-safe key-value cache
  wrapped in "quantum entanglement" naming; `vacuum_tunnel_nif`/`read_vacuum_state_nif` were a
  real rolling-hash-style accumulator wrapped in "zero-point field" naming. `dispatch_http`/
  `dispatch_rdma` are removed for a substantive reason, not just naming: PRD line 388 explicitly
  forbids direct I/O from inside the transition core, and `dispatch_http_nif` really could
  perform it. That a transition core with real raw-socket capability sat in the same file as
  the LLM-code-injection path in §2 is a materially worse combination than either alone, even
  though the two were never directly chained together in the code as found.

All 9 non-`eval_expr_nif` functions removed from both `air_core.erl` and `air_core_nif/src/
lib.rs` (Rust). `io-uring`/`libc` dependencies removed from `Cargo.toml` accordingly (nothing
left uses them). Verified: `apps/air_core/src/air_core.erl`,
`apps/arazzo_runner/src/arazzo_runner_workflow.erl`, `apps/atomvm_runner/src/atomvm_runner.erl`,
and `apps/arazzo_atomvm/test/arazzo_atomvm_SUITE.erl` all compile cleanly with `erlc`; the NIF
crate builds cleanly via `just air-core-nif-check`/`just air-core-nif-build`; the existing
`apps/air_core/test/fortune5_test.erl` (real workflow-transition test, unaffected by any
removal) passes end-to-end against the rebuilt NIF (`eunit:test(fortune5_test) -> ok`,
run this session).

## What was NOT touched

- `apps/air_core/src/air_core.erl`'s real transition-core logic (`new/1`, `transition/2`,
  `eval_expr`/`eval_criteria`/`apply_action`/`bind_outputs`, bitmask-based step tracking) —
  legitimate code with real gaps (wrong return shape, no join/AND readiness) tracked as
  PROJ-755/756, not a safety issue.
- `apps/arazzo_runner/src/arazzo_runner_sup.erl` — a structurally correct OTP supervisor,
  unaffected.
- The Rust workspace (`crates/`) — its reconciliation found ordinary gaps (missing bridges,
  unwired queries, placeholder codegen), mislabeled and overclaimed in commit messages and doc
  comments per `RAIL_A_B_STATUS.md`, but not fabricated-capability documents or code-injection
  pathways. Keep these two categories of finding separate: one is "this doesn't do what it
  says," the other is "this shouldn't exist as written."
- `apps/arazzo_atomvm/hw/arazzo_workflow_fsm.v` and its Yosys/OpenLANE build scripts — real
  Verilog hardware description work, not fictional. The Makefile targets currently only print
  "Done..." without invoking the real toolchain (a MOCKED-not-REAL issue, same pattern as other
  Rail findings), but this is ordinary unfinished-feature scope, not a safety finding — left for
  the regular ticket backlog rather than this document.

## Changed files (this remediation)

Deleted: `AGI_DIRECTIVE_MATRIX.md`, `apps/otp_runner/` (entire app),
`apps/arazzo_atomvm/hw/asic/dyson_sphere_simulation.py`,
`apps/arazzo_atomvm/hw/asic/strange_matter_simulation.py`, stale `.beam` build artifacts.
Modified: `apps/air_core/src/air_core.erl`, `apps/air_core/native/air_core_nif/src/lib.rs`,
`apps/air_core/native/air_core_nif/Cargo.toml`, `apps/arazzo_runner/src/
arazzo_runner_workflow.erl`, `apps/atomvm_runner/src/atomvm_runner.erl`,
`apps/arazzo_atomvm/PROOF_OF_EQUIVALENCE.md`, `apps/arazzo_atomvm/test/
arazzo_atomvm_SUITE.erl`, `justfile` (added `air-core-nif-check`/`air-core-nif-build` recipes).
Nothing pushed or committed — in the working tree for review.

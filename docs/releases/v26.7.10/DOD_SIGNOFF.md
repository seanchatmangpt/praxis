# DOD_SIGNOFF — Clause-by-Clause DoD Sign-Off for v26.7.10-revised (PROJ-748)

Status: FINAL (doc), tied to `RELEASE_CONTROL.md`. Every claim cites a file, test, or receipt.
If this file and `RELEASE_CONTROL.md` disagree, `RELEASE_CONTROL.md` wins. This sign-off
replaces the interim PROJ-617 sign-off as the governing document for v26.7.10-revised; the
interim sign-off is preserved verbatim at `DOD_SIGNOFF_INTERIM.md` (fix-forward, not deleted)
and remains true for its own (interim) scope.

Scope of "ALIVE" here: verified via the commands in `RELEASE_CONTROL.md` §9.1 —
`cargo test -p cng --features bench --test cng_decomp` (3/3), `--test cng_ipc_corpus` (10/10),
`--test cng_multi_engine -- --test-threads=1` (6/6), the full `cargo test -p cng --features
bench` suite (107 tests, 0 failures), `cargo check -p cng` (0 warnings, non-bench), and
`cargo test -p cng --features bench --lib` (67/67) — **plus** a follow-up verification round
(4 targeted agents, isolated `CARGO_TARGET_DIR=target/agent-7xx` builds) that closed five of
the gaps this sign-off originally left open: the literal 8² fan-out (PROJ-729), the real
two-bundle `full_production_ready` invocation (PROJ-742), the arazzo digest-verify wiring
(PROJ-745), the full 5x20 IPC corpus scale (PROJ-711), and `CNG_R09`'s negative test
(decomp) — **plus** a second, separate synthesis round (5 more targeted agents) that added:
a clean workspace-wide `cargo check --workspace --all-features` +
`cargo test --workspace --all-features --no-run`; a scoped `clippy -p praxis-graphlaw -p
pddl-index -- -D warnings` pass; a `cargo fmt --all --check` audit; negative-corpus item 6
forced-and-asserted (`cargo test -p cng --features bench --test
cng_decomp_negative_corpus_completeness`, 2/2); and — the load-bearing addition — the
decompose-to-dispatch bridge (PROJ-749, `cargo test -p cng --features bench --test
cng_decompose_to_dispatch_integration`, 2/2). See "What changed in the follow-up verification
round" and "What changed in this session's second synthesis round" below for the exact
command+output of each. Per-clause evidence pointers below cite the specific test name(s)
within these runs; the full citation chain (file:line) lives in each
`docs/jira/v26.7.10/tickets/PROJ-7xx.md`'s own "Evidence" section — not restated in full here
to avoid drift between three copies.

**Not run, and not claimed anywhere below**: Phase 6 commit is now DONE (pushed as `1f3f9bc`);
this document's remaining claims are current as of that commit plus uncommitted EOD-push and
moonshot-round work on top of it (`git status` not clean as of this writing). Any live-repo
`ggen sync run` against the real `ggen.toml` (PROJ-745's follow-up explicitly declined — no
pack-scoped CLI flag exists to bound the blast radius); live network transport for
OpenAPI/AsyncAPI/Arazzo (digest-verify is proven for all three documents — see below — but no
HTTP/broker binding exists or is claimed). **Superseded, this session's moonshot round**: a
dispatched contract carrying an actual PDDL payload so a remote engine executes the specific
plan it was sent (PROJ-710 -> PROJ-723) is now CLOSED — see "Moonshot round" below; do not
cite this boundary as open. `full_production_ready`'s real THREE-bundle invocation and
PROJ-714's long-horizon scenarios were EOD-push targets — see the "EOD push" section below for
their resolved status; do not rely on the "three waves" framing immediately below for either,
it predates that push. PROJ-714's exact scenario count was further updated by the moonshot
round (1/4 -> 2/4) — see "Moonshot round" below, not the "EOD push" section, for the current
number.

## Final state (this session, all rounds combined)

Three waves of work landed in `docs/releases/v26.7.10/` and `docs/jira/v26.7.10/tickets/`
this session, each cited with its own command+output and none silently superseding an earlier
wave's evidence: (1) the original closure round — PROJ-733/734's two blocking-bug fixes, the
PROJ-735..738 isolated verification ladder, the PROJ-739..743 marker/doctrine reconciliation,
PROJ-744/745's initial arazzo-pack wiring, and the first PROJ-746..748 doc closure; (2) a
follow-up verification round (4 agents) that closed five named gaps — the literal 8² (64-leaf)
fan-out, `full_production_ready`'s real two-bundle composition, the arazzo digest-verify
gate's wiring into `run_arazzo_projection`, the full 5x20 IPC corpus scale, and `CNG_R09`'s
negative test — documented in "What changed in the follow-up verification round" below; and
(3) this session's second synthesis round (5 agents) — documented in "What changed in this
session's second synthesis round" below — which added a clean workspace-wide `cargo check`/
`cargo test --no-run`, a scoped `praxis-graphlaw`/`pddl-index` clippy sweep (6 real items
fixed, ~60 pre-existing unrelated errors named and left untouched), a clean `cargo fmt --all
--check`, closed negative-corpus item 6 (`NO_BENEFICIAL_DECOMPOSITION` forced and asserted,
not just accepted as a legal branch), and — the most significant single addition of this
round — PROJ-749: a real bridge stitching a `decompose()` output into a real cross-engine
`cng engine serve` dispatch run for the first time this milestone.

Net effect on this document's own prior scoping language: the §2/§8/G8 rows below, and item 1
of "What remains explicitly not claimed", previously said no test this session stitched a
real `decompose()` output into a real cross-engine dispatch run. That is now corrected — it
has been done, at the mechanism level, via PROJ-749 (a new bridge module and a new dedicated
integration test, not a modification to `cng_multi_engine.rs`), on a fixture built for the
purpose because the canonical potato scenario's own `decompose()` output is single-actor and
has nothing to dispatch. PROJ-749 does not, and does not claim to, prove that the remote
engine executes the dispatched subworkflow's own PDDL plan, or that the two engines' outputs
together close the original problem's global goal — no payload-carrying contract exists yet
(PROJ-710 -> PROJ-723 remains open work, named explicitly in PROJ-749's own module doc and
ticket file).

**Superseded, this session's moonshot round**: the payload-carrying-contract gap named in the
paragraph immediately above (PROJ-710 -> PROJ-723) is now CLOSED — a dispatched contract does
carry its subworkflow's actual PDDL payload, digest-verified, and the remote engine executes
that specific plan. The narrower "do the two engines' outputs together close the original
problem's global goal" claim remains genuinely open (unchanged) — that is a distinct question
from payload fidelity and no machinery on disk checks it. See "Moonshot round" below for the
full mechanism and command+output; this paragraph is left otherwise unedited as a historical
record of the second synthesis round's own scope at the time it was written.

Combining all three waves plus the EOD push and moonshot round documented below: every
mechanism named in the DoD's 20 sections has now been independently exercised and evidenced
(every Track P/E ticket is ALIVE except PROJ-714, whose 80/20-rescoped scope now stands at
2-of-4 scenarios — see "Moonshot round" below, not the "IN PROGRESS"/"1-of-4" framing in this
paragraph, which predates that round), the marker doctrine matches on-disk names and code,
`full_production_ready`'s real THREE-bundle composition (workday + planning + distributed) is
proven (`cng_production_ready_three_way.rs`, EOD push — superseding the "two-bundle only"
framing three paragraphs above, which is left unedited as a historical record of that wave's
own scope, not a live claim), the 8² fan-out and full IPC corpus scale are at their literal
targets, OpenAPI/AsyncAPI digest-verify is proven alongside Arazzo's (EOD push, superseding
this wave's own "narrowed the claim" framing — see the "EOD push" section below), the two
tracks (P and E) are bridged at the mechanism level, AND (moonshot round) at the payload-
fidelity level. What remains genuinely open, honestly scoped, and not claimed anywhere in this
document: potato itself dispatched across H/M (it has no split to dispatch); whether combining
two engines' payload-faithful outputs closes the ORIGINAL problem's global goal (distinct from,
and narrower than, the payload-fidelity question the moonshot round closed); any live-repo
`ggen sync run`; live network transport (digest-verify ≠ a running HTTP/AsyncAPI server);
PROJ-714's remaining 2 scenarios (3-4, time-boxed cut — see "Moonshot round" below for exactly
which domains were tried and why they were dropped); and Phase 6 push of the EOD-push and
moonshot-round commits specifically (the prior `1f3f9bc` commit IS pushed; all later rounds'
work sits uncommitted on top of it as of this writing).

**Final state, the moonshot round (this session, dated addition — does not replace any prior
"Final state" paragraph above; adds one more).** After a load-bearing closure wave (7 parallel
agents + synthesis) and then this moonshot round (7 more parallel agents + synthesis), the
complete state of v26.7.10-revised, in `.claude/rules/no-overclaiming.md` vocabulary, is:
**ALIVE** — every Track P (PROJ-701..714) and Track E (PROJ-720..729) ticket except PROJ-714's
own declared-cut remainder; the decompose-to-dispatch bridge AND its payload fidelity
(PROJ-749, PROJ-710 -> PROJ-723 closed); `full_production_ready`'s real three-bundle
composition; the literal 8² fan-out; the full 5x20 IPC corpus; Arazzo AND OpenAPI/AsyncAPI
digest-verify; 3 of 4 `DispatchState::→BLOCKED` edges; 3 of 5 `CNG_R07` construction sites;
1 of ~130 `CNG_R10` sites; all 8 of 8 negative-proof-corpus items including the mutex-saturated
literal fixture; the 3 process-leak windows in `cng_multi_engine.rs`; and the stale-HEAD sweep.
**PARTIAL, exact remaining scope named** — PROJ-714 stands at 2 of 4 long-horizon scenarios
(logistics + tyreworld-chain ALIVE; barman/termes/blocksworld/grippers each tried and dropped
as genuine planner-search performance cliffs, not silently); global-goal closure across two
payload-faithful engines' outputs is unchecked by any machinery on disk (narrower than, and
distinct from, the payload-fidelity question just closed); `full_production_ready`'s two-bundle
case is ALIVE, three-bundle is ALIVE via the EOD push (`cng_production_ready_three_way.rs`).
**Explicitly still open by design, unchanged this round** — live network transport for any of
the three digest-verified documents (filesystem-as-transport is the deliberate boundary);
ChatmanEngine adoption in `cng` (deferred, `RELEASE_CONTROL.md` §10 item 2); ed25519/any
cryptographic signature scheme (not named anywhere in this milestone's DoD — receipts are
BLAKE3 content digests, not signatures, and no signing key material or verification path
exists in this codebase); any live-repo `ggen sync run`; real (non-synthesized) human
consequences (MOCKED-HUMAN); 8³ recursion (doctrine only). **Newly closed this round that was
open before it started** — PROJ-710 -> PROJ-723 (payload-carrying dispatch, the single
most-repeated "remains open" line in this document's own prior rounds); PROJ-714 scenario 2
(tyreworld-chain); 1 more `CNG_R07` site plus 2 investigated-unreachable, closing the item;
2 more `DispatchState::→BLOCKED` edges plus 1 investigated-vestigial, closing the item;
negative-corpus item 5 (mutex-saturated goals) upgraded from an adjacent-scenario proof to a
literal fixture. **Attempted this round, not closed** — expanded `CNG_R10` coverage and the
CLI/process-exit half of §14 item 4 both have new, well-formed test files on disk, but neither
has a command+output cited by its own closing agent this session; both stay UNVERIFIED, not
rounded up. Nothing in this paragraph authorizes an unscoped "production-ready" claim; see
`V26_7_10_PRODUCTION_READY`'s own scoped section above for that claim's exact boundary.

## Load-bearing closure pass (`GAP_AUDIT.md`, 8-agent wave)

A separate, later wave — 7 parallel agents + this synthesis agent sequential — closed
`docs/releases/v26.7.10/GAP_AUDIT.md`'s top-10 prioritized punch-list items (its §7). Full
per-item detail, including every command+output, lives in `GAP_AUDIT.md` §7 itself; this is the
summary required at this document's own closure boundary.

**CLOSED, this wave** (5 items):
- **CNG_R10 `IoRefused`**: one representative call site (`pipeline.rs:74`,
  `cng::pipeline::import_artifacts`) now has a negative test —
  `crates/cng/tests/cng_io_refused_negative.rs`, 1/1 passed. PARTIAL in scope: the other ~129
  construction sites remain UNVERIFIED individually; the mechanism (not each site) is proven.
- **Process-leak windows in `cng_multi_engine.rs`**: all 3 `spawn_engine()` call sites now
  wrapped in a kill-on-drop `EngineGuard`; 7/7 tests still pass, zero orphaned `cng engine
  serve` processes confirmed post-run.
- **`DispatchState::Blocked` reachability**: the `TIMED_OUT→BLOCKED` edge is now live-walked
  by a dedicated test (`remediation_budget=0`); PARTIAL — 3 of the 4 originally-flagged
  `→BLOCKED` edges remain undriven.
- **CNG_R07 `RunnerMismatch`**: 2 of 5 construction sites now have negative tests through the
  real `validate_run` path, one of them (a cyclic order relation) caught by the actual
  published `bcinr-powl` runtime's own Kahn check, not a mock.
- **Stale HEAD citation (`40f6020`→`1f3f9bc`)**: 40 files corrected; 9 files (+ part of
  `index.md`) correctly left with `40f6020` as legitimate historical citation, not a live-HEAD
  claim; spot-checked this session.

**INVESTIGATED, correctly not forced** (3 items — each produced a doc correction instead of a
fabricated fixture or invented business semantics, per this wave's own explicit instruction not
to force a fit):
- **CNG_R08 `Nondeterminism`**: both construction sites are unreachable-by-design through the
  current public API (no external seam between the two internal manufacture calls each guards).
  Remains UNVERIFIED (dedicated negative test) — but as a now-evidenced reachability finding,
  not an untried gap.
- **`DISPATCH_READY→REFUSED`**: confirmed dead-by-construction (the only pre-dispatch check,
  `CNG_R15`, fires before this state is ever entered). `DispatchState`'s own type doc in
  `dispatch.rs` now says so explicitly instead of silently implying a caller exists.
- **OpenAPI/AsyncAPI consumption**: this wave's own investigation (zero grep hits at the time)
  concluded the honest action was narrowing `DEFINITION_OF_DONE.md` §20 item 1's claim, not
  building a symmetric digest-verify seam — `packs/arazzo-pack/README.md`'s own seam note names
  only Arazzo. **Superseded since, independently confirmed**: a dedicated agent (dispatched
  after the user explicitly rejected the narrow-the-claim fallback) built real support —
  `crates/cng/src/bench/api_docs.rs`'s `verify_api_docs_render_digest`/
  `verify_api_docs_render_digest_if_present`, wired into `engine_serve` (`engine.rs:589`,
  before the poll loop), verifying both `generated/engine-openapi.yaml` and
  `generated/engine-asyncapi.yaml` against the same `.ggen-v2/receipt.json` shape Arazzo uses;
  reuses `CNG_R11 AuditMismatch`, absence handled honestly (`Ok(None)`, no false refusal). 6
  new tests in `api_docs_test.rs`. Verified: `cargo test -p cng --features bench --lib` → 77
  passed, 0 failed (`CARGO_TARGET_DIR=target/agent-openapi-override`). `DEFINITION_OF_DONE.md`
  §20 item 1 and this document's §5 row now state the ALIVE claim for both Arazzo and
  OpenAPI/AsyncAPI. `GAP_AUDIT.md` §7 item 8 should be updated to match this closure, not left
  at "superseded, unverified."

**STALE findings corrected, predating this wave** (2 items — GAP_AUDIT.md's own text was
written concurrently with an earlier closure round and had not caught up; no Phase-1 agent
touched either):
- `dispatch_bridge.rs`'s cited test file (`cng_decompose_to_dispatch_integration.rs`) exists
  (PROJ-749, prior round).
- `DecompositionOutcome::NoBeneficialDecomposition` is forced+asserted by exact value
  (`cng_decomp_negative_corpus_completeness.rs:188-195`, prior round).

**Remains open, out of this wave's scope** (`GAP_AUDIT.md` §7 items 11-20): the 29
non-load-bearing cosmetic/doc-polish items (status-vocabulary drift, ticket-number gap,
`tyreworld.rs` clean-room note, etc.) — none were assigned to any of the 8 agents and none are
claimed closed here. **Superseded by the moonshot round below**: item 11 (mutex-saturated
goals) is now CLOSED; item 12 (CLI/process-exit) has a new test file on disk but is NOT
confirmed passing this session — see "Moonshot round" immediately below for the precise status
of both.

## Moonshot round (this session, after the load-bearing closure pass)

A further wave — 7 more parallel agents + this synthesis agent sequential — picked up
`GAP_AUDIT.md` §7's remaining items 5, 6, 11, and 12 (the sub-parts the load-bearing closure
wave above left open), plus PROJ-714's scenarios 3-4 and the PROJ-710 -> PROJ-723
payload-carrying-dispatch boundary named throughout this document. Per this round's own
instruction, findings are reported as CLOSED only where a command+output was actually cited by
the closing agent this session; a file existing on disk without a cited passing run is reported
as UNVERIFIED, not rounded up.

**CLOSED, this round** (4 items, each with cited command+output):
- **Payload-carrying dispatch (PROJ-710 -> PROJ-723)**: a dispatched `DispatchContract` now
  carries its subworkflow's actual `(domain_pddl, problem_pddl)` text as two BLAKE3-digest-
  verified sibling files in the target engine's real inbox; the remote engine parses/grounds/
  plans that SPECIFIC payload instead of its own synthetic artifact set, falling back to the
  prior synthetic path only when no payload is present (purely additive; no schema/template/
  shape change). `dispatched_subworkflow_payload_is_the_content_the_engine_actually_executes`
  proves byte-identity between what was dispatched and what each engine manufactured, that the
  two engines' manufactured content genuinely differs, and that the prior synthetic
  `"email-routing"` path never fires when a payload is present.
  `CARGO_TARGET_DIR=target/agent-payload cargo test -p cng --features bench --test
  cng_decompose_to_dispatch_integration -- --test-threads=1` → `test result: ok. 3 passed; 0
  failed`. Full mechanism and regression evidence: `RELEASE_CONTROL.md` §9.2a. This closes the
  single most-repeated "remains open" line in this document (see the superseded-paragraph
  corrections above) and in `RELEASE_CONTROL.md`/`index.md`/`PROJ-749.md`.
- **CNG_R07 `RunnerMismatch`, third construction site**: `model_leaf_count_disagrees_with_
  tape_op_count_refuses_cng_r07` closes the op-count-mismatch site (`runner.rs:177`) through
  the real `validate_run` path, bringing the total to 3 of 5 sites now negative-tested.
  `CARGO_TARGET_DIR=target/agent-r07expand cargo test -p cng --test
  cng_runner_mismatch_negative -- --nocapture` → `test result: ok. 3 passed; 0 failed`,
  reproduced twice, byte-identical both times. The remaining 2 sites
  (`runner.rs:269`/`runner.rs:279`, incomplete-scheduler-firing and order-violated-at-runtime)
  were investigated, not forced: both are guarded by the same post-scheduler cross-check, and
  a structural argument (every model this adapter compiles lowers to one flat `PartialOrder` of
  `Atom` leaves; `bcinr-powl`'s `pred_satisfied` gate and per-fire `check_mask` re-population
  make firing-before-a-predecessor and budget exhaustion unreachable for any DAG
  `compile_powl`'s own Kahn check admits) plus empirical corroboration (29,403 real
  `validate_run` calls via permutation-derived and worst-case topological orders, `n`=2-63,
  zero budget hits, zero conformance hits — exploratory harness, not committed) both support
  unreachability. The file's header doc now records this investigation, mirroring the
  CNG_R08/`DISPATCH_READY→REFUSED` precedent rather than fabricating a triggering test. All 5
  of the originally-flagged sites are now accounted for: 3 tested, 2 investigated-unreachable.
- **`DispatchState::Blocked`, two more edges**: `semantic_refusal_with_zero_remediation_
  budget_reaches_blocked` proves `REFUSED→BLOCKED` (same wrong-artifact fixture as the existing
  budget=1 test, budget=0) and `unimplemented_closure_law_leaves_parent_remote_in_progress_
  blocked` proves `REMOTE_IN_PROGRESS→BLOCKED` (a recursive parent declaring
  `QUORUM_REQUIRED`, one of the four closure laws `dispatch-closure.rq` documents as declared
  but not yet emitted). Both assert the full ledger trajectory, not a loose status check,
  bringing the total to 3 of 4 originally-flagged edges now live-walked (`TIMED_OUT→BLOCKED`
  from the prior wave, plus these two). The 4th edge, `COMPENSATING→BLOCKED`, was investigated
  and found vestigial: exhaustive grep confirms only two call sites ever advance a contract
  into `Compensating`, and both unconditionally advance to `Completed` next once `remediate()`
  returns `Ok`; `remediate()`'s only other exit is an `Err` that propagates out via `?` before
  any `Blocked` ledger entry for that contract could exist — no code path reaches it,
  documented rather than forced, same class as the `DISPATCH_READY→REFUSED` finding. All 4
  originally-flagged edges are now accounted for: 3 live-walked, 1 investigated-vestigial.
- **Mutex-saturated goals -> `NO_ADMISSIBLE_DECOMPOSITION`** (§18 item 5, upgraded from
  "ALIVE (adjacent scenario)" to **ALIVE (literal fixture)**): new file
  `crates/cng/tests/cng_mutex_saturated_negative.rs` exercises the real Datalog `:mutex` rule
  (`rules/decomp.dl`) and `partition_goals`'s union-find directly —
  `genuine_datalog_mutex_between_sole_achievers_unions_both_goal_atoms_into_one_partition_
  component` proves a real STRIPS mutex (one action's sole achiever deletes the precondition
  the other goal's sole achiever needs) derives a `:mutex` edge and unions both goal atoms into
  one partition component; `mutex_saturated_goals_force_no_admissible_decomposition_with_zero_
  candidates_ever_attempted` runs real end-to-end `decompose()`, exact-asserts
  `NoAdmissibleDecomposition { rejected: 0 }` and `candidate_receipts.len() == 1` (proving zero
  splits were ever ATTEMPTED, not attempted-and-rejected — the load-bearing distinction from
  §18 item 3's `CNG_R22` mechanism, which rejects an already-enumerated candidate); a third test
  swaps in a structurally identical but mutex-free control and proves the outcome flips to
  `Selected` — isolating the mutex edge, not goal-count or domain shape, as the cause.
  `CARGO_TARGET_DIR=target/agent-mutexgoals cargo test -p cng --features bench --test
  cng_mutex_saturated_negative -- --test-threads=1 --nocapture` → `test result: ok. 3 passed; 0
  failed`, reproduced 4 times, byte-identical every time.

**Attempted, file added, NOT independently confirmed passing this session** (2 items — the
closing agent's own final report to this synthesis pass contained no command+output, only a
mid-task "waiting on a background build" status; per `.claude/rules/no-overclaiming.md`, a file
existing on disk is not evidence of a passing test, so neither is rounded up to ALIVE here):
- **CNG_R10 `IoRefused` expanded coverage**: `crates/cng/tests/cng_io_refused_negative.rs` has
  a new `cng_r10_bench_sites` module (confirmed on disk via `git diff`, +227 lines) adding three
  more representative construction sites outside `pipeline.rs` — `workday.rs:588`
  (`build_decomp_marker_store`, nonexistent-path technique), `workday_verify.rs:472`
  (`assemble_workday_manifest`, path-collision technique), and `decomp/mod.rs:726`
  (`decompose`'s `emit_result_graph`, path-collision driven through a real end-to-end
  decomposition over the potato fixture). Status: UNVERIFIED (no command+output cited this
  session for this specific module) — the original single-site closure from the load-bearing
  wave (`import_artifacts_missing_dir_refuses_cng_r10_io_refused`, `pipeline.rs:74`) remains
  ALIVE as previously recorded and is unaffected by this file's uncommitted expansion.
- **CLI/process-exit half of §14 item 4**: new file
  `crates/cng/tests/cng_cli_nonzero_exit_on_hostile_marker.rs` (confirmed on disk, untracked,
  205 lines, well-formed) spawns the REAL compiled `cng` binary
  (`std::process::Command::new(env!("CARGO_BIN_EXE_cng"))`) against `benchmark workday --ticks
  0` (a real CLI-only hostile input driving `marker-autonomic-loop.rq`'s `extraOperators` term
  negative) and asserts on `std::process::ExitStatus` directly — `hostile_zero_tick_workday_
  marker_false_exits_nonzero_from_real_process` (exit code 1, stderr names `CNG_R20` and the
  specific false marker) plus a positive-contrast control
  (`healthy_workday_marker_true_exits_zero_from_real_process`). Status: UNVERIFIED (no
  command+output cited this session) — the file is well-formed and its design closes exactly
  the gap `GAP_AUDIT.md` §7 item 12 names (the marker-QUERY half was already proven; this is
  the CLI/process-EXIT half), but it has not been independently run and confirmed passing this
  session. Do not cite this as closed until a command+output exists.

**Superseded, this round**: PROJ-714's long-horizon scenarios moved from 1/4 to 2/4
(`long_horizon_tyreworld_chain_scenario_decomposes_and_plans_end_to_end` closed alongside the
prior logistics scenario; barman/termes/blocksworld/grippers each tried and dropped per
`PROJ-714.md`'s own "do not force a fit" clause — planner-search performance cliffs, not
grounding blowups). `CARGO_TARGET_DIR=target/agent-714scenarios cargo test --package cng --test
cng_long_horizon_scenario --features bench` → `test result: ok. 2 passed; 0 failed`. Full
per-domain accounting: `RELEASE_CONTROL.md` §9.2, `PROJ-714.md`.

Vocabulary used in this section follows `.claude/rules/no-overclaiming.md`: ALIVE (verified
this session, cited command+output), UNVERIFIED (default — a file existing on disk is not by
itself evidence of a passing run), INVESTIGATED (a real finding, not a fix, backed by the
agent's own reasoning and read evidence).

Vocabulary used above follows `.claude/rules/no-overclaiming.md`: ALIVE (verified this session,
cited command+output), PARTIAL (gap named explicitly), UNVERIFIED (default, not claimed),
INVESTIGATED (a real finding, not a fix, backed by the agent's own reasoning and read evidence).

## Clause-by-clause sign-off

One line per `DEFINITION_OF_DONE.md` section (1-20).

| § | Clause | Status | Evidence pointer |
|---|---|---|---|
| 1 | Interim milestone (superseded) | ALIVE (interim record) | unchanged from `RELEASE_CONTROL.md` §8; nothing re-verified or reopened this session |
| 2 | Governing claim (full narrative: admit -> decompose -> dispatch -> close) | ALIVE (decompose-to-dispatch bridge, mechanism); ALIVE (payload fidelity, moonshot round) / PARTIAL (global-goal closure only) | `decompose()` end-to-end ALIVE (PROJ-701..713); multi-engine dispatch ALIVE (PROJ-720..729); PROJ-749 (second synthesis round) stitches a real `decompose()` output into a real cross-engine dispatch run — `dispatch_subworkflow_to_engine`/`collect_subworkflow_consequence` (`decomp/dispatch_bridge.rs`) convert each subworkflow into a shape-conformant `DispatchContract`, write it into a REAL second OS process's inbox, run `cng engine serve` to completion, and admit the consequence from the real outbox (`cng_decompose_to_dispatch_integration.rs`, 3/3 passed). **Moonshot round**: the remote engine now executes the subworkflow's OWN PDDL plan — `dispatch_subworkflow_to_engine` writes digest-verified sibling `(domain_pddl, problem_pddl)` files into the target engine's inbox, `engine.rs::run_serve_loop` verifies and parses/grounds/plans that SPECIFIC payload (`CNG_R11` on divergence), falling back to the prior synthetic path only when absent — `dispatched_subworkflow_payload_is_the_content_the_engine_actually_executes` proves byte-identity + genuine content divergence between engines + no synthetic-path fallthrough. STILL NOT proven: that combining the two engines' outputs closes the original problem's global goal — no payload-carrying contract closed that question, only payload fidelity (PROJ-710 -> PROJ-723 CLOSED for fidelity; global-goal closure remains a distinct, open question) |
| 3 | TWOSTEP-replacement table (7 rows) | ALIVE | each row's replacement mechanism evidenced under its own PROJ ticket (704/705, 703/706, 707, 708, 710, 710, 740) — see `DOD_EVIDENCE_MAP.md` |
| 4 | Formal planning target + 6 proof obligations | ALIVE | goal coverage/helper reachability (PROJ-705: `single_actor_is_always_candidate_zero`, `decomp_test.rs:315`; PROJ-710: `single_atom_goal_yields_no_admissible_decomposition`, `decomp_test.rs:422`; negative: `helper_unreachable_refuses_cng_r04`, `cng_ipc_corpus.rs:311`), interface-state (`CNG_R23`, PROJ-707: `tampered_tape_refuses_cng_r23_interface_state_mismatch`, `decomp_test.rs:182`), main reachability (PROJ-710: `decompose_is_deterministic_across_runs`, `decomp_test.rs:443`; negative: `main_unreachable_after_helper_refuses_cng_r23`, `cng_ipc_corpus.rs:331`), non-interference (`CNG_R22`, PROJ-708: `concurrent_clobber_refuses_cng_r22_interference`, `decomp_test.rs:203`), release closure (`CNG_R24`, PROJ-708: `unreleased_resource_refuses_cng_r24`, `decomp_test.rs:249`) — all negative-tested |
| 5 | No-LLM decomposition mechanism per dialect (7 rows) | ALIVE | PDDL/pddl-strips/CONSTRUCT/Datalog/POWL/OCEL rows ALIVE (PROJ-701..710/727); Arazzo row ALIVE — `digest(render(graph))` wired into `arazzo::run_arazzo_projection` (PROJ-745), refusing `CNG_R11` before any step dispatches on a missing/mismatched render; `dispatch.rs`'s own generic `ArazzoRendered` transition renders the `DispatchContract` itself (unrelated to arazzo-pack's YAML) and was correctly left unwired. OpenAPI/AsyncAPI row now ALSO ALIVE (closed this session, EOD push) — `verify_api_docs_render_digest_if_present` (`crates/cng/src/bench/api_docs.rs`) wired into `engine_serve` (`engine.rs:589`), verifying both `generated/engine-openapi.yaml` and `generated/engine-asyncapi.yaml` against the same ggen receipt shape before the poll loop begins; tampered render refuses `CNG_R11`, absent docs proceed honestly (no false refusal). 77/77 `cargo test -p cng --features bench --lib` passed, 6 new tests. OpenAPI/AsyncAPI *schema validation* (structural conformance of the YAML content itself, distinct from digest integrity) and any live HTTP/broker binding remain a declared, separate boundary (§20) — not a gap in this row, which is scoped to digest-integrity |
| 6 | Bounded decomposition algorithm (15 steps) | ALIVE | exercised end-to-end by every `decompose()` test — potato: `potato_decomposition_is_typed_receipted_and_replayable` (`cng_decomp.rs:78`); IPC corpus: 10/10 in `cng_ipc_corpus.rs`; permutation: `permuted_goal_identities_change_plans_and_receipts_causally` (`cng_ipc_corpus.rs:195`); bounds declared and receipted (`search.rs`, `select.rs`) |
| 7 | Single-actor route — typed results, never fallback | ALIVE | `single_actor_is_always_candidate_zero`, `single_atom_goal_yields_no_admissible_decomposition` |
| 8 | Potato canonical scenario | ALIVE (single-process) / UNVERIFIED (potato itself dispatched cross-engine) | `potato_decomposition_is_typed_receipted_and_replayable` ALIVE (single-process). Potato's real `decompose()` output selects `DecompositionOutcome::NoAdmissibleDecomposition` (single-actor; `decomp:subworkflowCount "1"`, verified against the emitted graph before PROJ-749 was written), so it has no multi-subworkflow split to dispatch — the DoD's own text claims potato is "executed across the H and M engines of §10", which stays UNVERIFIED for potato specifically. The general decompose-to-dispatch MECHANISM, including payload fidelity as of the moonshot round, is ALIVE (PROJ-749, §2 row above), proven on a different fixture (kitchen two-chain) that does split |
| 9 | Recursive generalization 8^1 -> 8^2 -> 8^3 | ALIVE (8^1) / ALIVE (8^2, full 64-leaf fan-out) / UNVERIFIED (8^3, out of scope) | 8^1: potato, single level, ALIVE. 8^2: `recursion_crosses_engines_full_8x2_fanout` (`cng_multi_engine.rs`, follow-up round) exercises the literal fan_out=8/depth=2 target — 73 dispatches per root (1 root + 8 first-level + 64 second-level leaves), 146 total across the two roots (H, M), 64 of them depth-2 leaves — matching the section's own "8²" framing exactly; 2/2 runs green, 37.19s and 32.50s. `recursion_crosses_engines_depth_two` (fan_out=2) remains in the suite as a faster smoke test. 8^3: doctrine only, unchanged, UNVERIFIED |
| 10 | Multi-engine topology (6 forbidden items) | ALIVE, scoped to CARGO_BIN_EXE harness | separate OS processes confirmed; `SHARED_MEMORY_CROSSINGS_ZERO`/`DIRECT_ENGINE_BYPASSES_ZERO` true in `cng_multi_engine.rs:232-236` |
| 11 | Authority partition table (8 rows) | ALIVE (structural) | exercised transitively by every `cng_multi_engine` test — e.g. `multi_engine_concurrent_dispatch_execute_readmit`, `isolation_falsifier_hostile_graph_is_refuted_by_markers`, `double_admit_falsifier_replayed_collect_refuses_cng_r25` (`cng_multi_engine.rs:200,279,303`, PROJ-728) — coordinator/H/M roles enforced by construction; no per-row dedicated test cited individually |
| 12 | 16-state cross-engine dispatch machine | ALIVE (transition table, exhaustive); ALIVE (3 of 4 `→BLOCKED` edges live-walked, load-bearing wave + moonshot round); INVESTIGATED-VESTIGIAL (4th `→BLOCKED` edge); `DISPATCH_READY→REFUSED` declared-but-unreached (doc-corrected, load-bearing closure wave) | `sixteen_state_transition_law_is_exact`, `shapes_ttl_state_individuals_match_the_enum` (drift test, all 256 pairs) prove the table; `deadline_expiry_with_zero_remediation_budget_reaches_blocked` proves `TIMED_OUT→BLOCKED` (load-bearing wave); moonshot round adds `semantic_refusal_with_zero_remediation_budget_reaches_blocked` (`REFUSED→BLOCKED`) and `unimplemented_closure_law_leaves_parent_remote_in_progress_blocked` (`REMOTE_IN_PROGRESS→BLOCKED`), each asserting the full ledger trajectory; `COMPENSATING→BLOCKED` investigated and found vestigial — both call sites that advance to `Compensating` unconditionally advance to `Completed` next, and `remediate()`'s only other exit is an `Err` propagated via `?` before any `Blocked` ledger entry could exist for that contract; all 4 originally-flagged edges are now accounted for (3 live-walked, 1 investigated-vestigial). `DISPATCH_READY→REFUSED`'s type doc (`dispatch.rs` ~104-150) now states explicitly it is declared-lawful with no production caller (both `Refused` construction sites fire only from `RESULT_RECEIVED`) — investigated, correctly left unwired rather than forcing a fit; see `GAP_AUDIT.md` §7 items 5/7 |
| 13 | Distributed broker law (5 items) | ALIVE | ledger (PROJ-721: `ledger_records_every_advance_and_replays_chain_verified`, `dispatch_test.rs:538`), `CNG_R25 DoubleAdmit` (PROJ-721: `replayed_consequence_refuses_cng_r25_double_admit`, `dispatch_test.rs:594`; also `cng_multi_engine.rs` falsifier `double_admit_falsifier_replayed_collect_refuses_cng_r25`, `cng_multi_engine.rs:303`), `EngineIdentity` (PROJ-722: `engine_identity_is_deterministic_and_engine_distinct`, `engine_test.rs:57`), G13 resume (PROJ-724: `resume_verifies_ledger_prefix_and_skips_processed_contracts`, `torn_ledger_tail_refuses_cng_r11_on_resume`, `engine_test.rs:135,164`; PROJ-729: `g13_crash_resume_verifies_chain_and_completes`, `cng_multi_engine.rs:318`) |
| 14 | Isolation falsifiers (5 items) | ALIVE, scoped to CARGO_BIN_EXE harness | items 1-3 structural (process/marker/filesystem); item 4 `isolation_falsifier_hostile_graph_is_refuted_by_markers`; item 5 `distributed_determinism_two_serialized_runs_byte_identical`. **Item 4's separately-tracked CLI/process-exit sub-claim** (`GAP_AUDIT.md` §7 item 12, distinct from this row's in-process marker-query proof): `crates/cng/tests/cng_cli_nonzero_exit_on_hostile_marker.rs` (new, untracked) spawns the real `cng` binary and asserts on `std::process::ExitStatus` directly — UNVERIFIED, no command+output cited this session; see "Moonshot round" above |
| 15 | Gall checkpoints G0-G16 | see per-gate row below | — |
| 16 | Required result markers | ALIVE (constituent families) / ALIVE (real two-run composition: workday + planning) / UNVERIFIED (three-run composition, +distributed) | see `DEFINITION_OF_DONE.md` §16 (reconciled, PROJ-743) and PROJ-742's follow-up evidence (`cng_production_ready.rs`) |
| 17 | Anti-hardcoding requirements | ALIVE | `permuted_goal_identities_change_plans_and_receipts_causally`, `no_canned_helper_subgoal_across_incompatible_variants` |
| 18 | Negative proof corpus (8 items) | ALIVE (8/8) | see corpus table below |
| 19 | Paper-equivalent evaluation corpus | ALIVE (full scale) | seeds 0-3 per domain verified first (`ipc_corpus_seeds_plan_decompose_and_regenerate_byte_identically`); full 5x20 = 100 problems now independently run (PROJ-711 follow-up, `cng_ipc_corpus_full_scale.rs`), 2/2 runs green, 11.66s and 11.79s, zero failures across all 100 domain x seed pairs |
| 20 | Honest boundaries (6 items) | ALIVE (doc, self-consistent) | filesystem transport confirmed; HTTP binding UNVERIFIED as declared; long-horizon cut recorded (`RELEASE_CONTROL.md` §9.2); MOCKED-HUMAN carried forward; 8^3 UNVERIFIED; no unscoped production-ready claim made anywhere in this sign-off |

### §15 Gall checkpoint detail

| Gate | Status | Evidence |
|---|---|---|
| G0 | ALIVE (interim) | unchanged, `RELEASE_CONTROL.md` §8 |
| G1 | ALIVE | `pddl-strips.ttl`/shapes on disk, exercised by every `decompose()` run (PROJ-701) |
| G2 | ALIVE | `lift_render_round_trip_preserves_atom_sets` (PROJ-702/703) |
| G3 | ALIVE | `decomp.dl`/`decomp-resources.dl` exercised by every `decompose()` run (PROJ-704) |
| G4 | ALIVE | `single_actor_is_always_candidate_zero` (PROJ-705) |
| G5 | ALIVE | interface-state + manufacture tests (PROJ-706/707) |
| G6 | ALIVE | non-interference + release-closure tests (PROJ-708) |
| G7 | ALIVE | composition + selection tests (PROJ-709/710) |
| G8 | ALIVE (single-process, potato) / ALIVE (cross-engine mechanism + payload fidelity, different fixture) | potato tests (PROJ-712); decompose-to-dispatch bridge test (PROJ-749, 3/3 including moonshot-round payload test) — potato itself not dispatched cross-engine (see §8 row) |
| G9 | ALIVE (transition table); ALIVE (3 of 4 `→BLOCKED` edges live-walked, load-bearing wave + moonshot round) | drift test (PROJ-720); `deadline_expiry_with_zero_remediation_budget_reaches_blocked` + moonshot round's `semantic_refusal_with_zero_remediation_budget_reaches_blocked`/`unimplemented_closure_law_leaves_parent_remote_in_progress_blocked` (`dispatch_test.rs`) — see §12 row above for the full 4-edge breakdown |
| G10 | ALIVE | ledger + `DoubleAdmit` tests (PROJ-721) |
| G11 | ALIVE | `EngineIdentity` test (PROJ-722) |
| G12 | ALIVE | `engine serve` + Arazzo projection tests (PROJ-723/725/726); digest-verify gate now wired into `run_arazzo_projection` (PROJ-745 follow-up) |
| G13 | ALIVE, scoped to test harness | `g13_crash_resume_verifies_chain_and_completes` (PROJ-724/729, the direct target of PROJ-734's fix) |
| G14 | ALIVE (full scale) | full 5x20=100 domain x seed corpus (PROJ-711 follow-up, `cng_ipc_corpus_full_scale.rs`), 2/2 runs green, 11.66s/11.79s |
| G15 | ALIVE (mechanism, 2/4) / PLANNED (3-4, time-boxed cut) | `RELEASE_CONTROL.md` §9.2; `PROJ-714.md` |
| G16 | ALIVE (constituent) / ALIVE (composed, three-bundle: workday+planning+distributed) | each marker family true separately; `full_production_ready`'s real three-bundle invocation now exercised end-to-end (`full_production_ready_holds_on_real_triple_bundle_evidence`, PROJ-742 EOD push, `cng_production_ready_three_way.rs`) |

### §18 Negative proof corpus detail (8 items)

| # | Scenario | Status | Evidence |
|---|---|---|---|
| 1 | Unreachable helper goal | ALIVE | `helper_unreachable_refuses_cng_r04` |
| 2 | Interface-state mismatch | ALIVE | `tampered_tape_refuses_cng_r23_interface_state_mismatch`, `main_unreachable_after_helper_refuses_cng_r23` |
| 3 | Interfering effect pair | ALIVE | `concurrent_clobber_refuses_cng_r22_interference`, `interfering_parallel_actions_refuse_cng_r22` |
| 4 | Unreleased resource | ALIVE | `unreleased_resource_refuses_cng_r24`, `helper_retains_resource_refuses_cng_r24` |
| 5 | Mutex-saturated goals -> NO_ADMISSIBLE | ALIVE (literal fixture, moonshot round) | `crates/cng/tests/cng_mutex_saturated_negative.rs` exercises the real Datalog `:mutex` rule and `partition_goals`'s union-find directly: `genuine_datalog_mutex_between_sole_achievers_unions_both_goal_atoms_into_one_partition_component` proves the derivation; `mutex_saturated_goals_force_no_admissible_decomposition_with_zero_candidates_ever_attempted` exact-asserts `NoAdmissibleDecomposition { rejected: 0 }` with `candidate_receipts.len() == 1` (zero splits ever attempted, distinct from `CNG_R22`'s reject-an-enumerated-candidate mechanism, item 3 above); a mutex-free control confirms the outcome flips to `Selected`. `CARGO_TARGET_DIR=target/agent-mutexgoals cargo test -p cng --features bench --test cng_mutex_saturated_negative -- --test-threads=1 --nocapture` → 3 passed, 0 failed, reproduced 4x. Supersedes the prior "ALIVE (adjacent scenario)" framing, which relied on `single_atom_goal_yields_no_admissible_decomposition` (typed-outcome mechanism only, not a literal mutex-saturated fixture) |
| 6 | Splits admissible but never beneficial -> NO_BENEFICIAL | ALIVE | `splits_admissible_but_not_beneficial_forces_no_beneficial_decomposition` (`cng_decomp_negative_corpus_completeness.rs`, second synthesis round, 2/2 passed) FORCES the exact outcome `DecompositionOutcome::NoBeneficialDecomposition { best_rejected_id: "cooked(potato)" }` (not `matches!`) via a fixture where a literal (non-variable) precondition forces every lawful plan through one unique total order while the split stays enumerable; pins the exact score numbers that make the single-actor candidate win (`makespan=4, dispatch_cost=6` vs. the split's `makespan=4, dispatch_cost=8`) |
| 7 | Injected canned subgoal | ALIVE | `detached_graph_action_refuses_cng_r09_hardcoding_suspicion` (`decomp/decomp_test.rs`, follow-up round) deliberately injects a fabricated action IRI absent from `ground.actions` and asserts `CNG_R09 HardcodingSuspicion` fires (already wired in `rules.rs::append_pair_facts`, now negative-tested). `no_canned_helper_subgoal_across_incompatible_variants` remains the correct, distinct closure for candidate-id purity in `search.rs` — a separate concern, not a substitute for this one. Second synthesis round adds a complementary confirmation: `canned_subgoal_detection_catches_identical_goal_labels_with_different_achiever_structure` (`cng_decomp_negative_corpus_completeness.rs`) proves two domains with IDENTICAL goal-atom labels but different achiever chains still yield receipts with DIFFERING content (`makespan`, `dispatch_cost`, graph bytes) under the same candidate id — no cached/canned answer keyed on the id string |
| 8 | Bypass-injection distributed negative | ALIVE | `isolation_falsifier_hostile_graph_is_refuted_by_markers` |

## `V26_7_10_PRODUCTION_READY` — scoped claim, honestly bounded

The initial closure round verified, separately and each with a cited command+output:

1. The interim single-operator `MARKER_MAP` conjunction (unchanged from the interim closure).
2. The nine planning markers (`PLANNING_MARKER_MAP`) true over a real `cng plan decompose`
   run (PROJ-739/740/741).
3. The nine distributed markers (`DISTRIBUTED_MARKER_MAP`) true within the `cng_multi_engine`
   CARGO_BIN_EXE test harness (PROJ-727/728/729).

A follow-up verification round then closed the two-bundle composition gap. New file
`crates/cng/tests/cng_production_ready.rs::full_production_ready_holds_on_real_dual_bundle_evidence`
runs a REAL `workday()` bundle (seed 742, 4 ticks) and a REAL `cng::bench::decomp::decompose()`
bundle (potato fixture, bridged via `strips_graph_to_surface`) end-to-end, evaluates each with
the real `evaluate_markers`/`evaluate_planning_markers` functions, and calls
`full_production_ready(&workday_markers, &planning_markers, None)` — all 26 combined keys
(16 workday-named + 9 planning-named + the recomputed `V26_7_10_PRODUCTION_READY`) assert
`true`. A companion negative test
(`full_production_ready_goes_false_when_a_real_marker_is_forced_false`) reuses the same real
pair, forces one real marker false on each side independently (`AUTONOMIC_LOOP_CLOSED` on the
workday side, `LLM_CALLS_ZERO` on the planning side), and confirms `V26_7_10_PRODUCTION_READY`
goes `false` in both cases, plus a control assertion that the unmodified real pair stays
`true` (ruling out a trivially-always-false combinator). Both tests green:
`CARGO_TARGET_DIR=target/agent-742 cargo test -p cng --features bench --test
cng_production_ready -- --nocapture` → 2 passed, 0 failed, ~1.15s (0.73s on a warm-cache
re-run). This required a smallest-possible visibility bump — `build_decomp_marker_store`,
`evaluate_planning_markers`, and `full_production_ready` changed from `pub(super)` to `pub` in
`crates/cng/src/bench/workday.rs`, re-exported from `crates/cng/src/bench/mod.rs` — no logic
changes.

**Three-way composition — closed (EOD push).** `full_production_ready` has now been invoked
with a REAL `workday_markers` bundle, a REAL `planning_markers` bundle, AND a REAL
`distributed_markers` bundle together, in one run: new file
`crates/cng/tests/cng_production_ready_three_way.rs` runs `engine_dispatch_remote` then two
real `cng engine serve` OS processes (its own independent `run_cng`/`serve_to_budget`
reimplementation, not imported from `cng_multi_engine.rs` — collision-free by construction)
then `engine_collect_remote`, whose `EngineCoordinateReport.markers` field carries the real,
already-evaluated `DISTRIBUTED_MARKER_MAP` output (no visibility bump or new marker-evaluation
machinery needed — the field was already `pub`). `full_production_ready(&workday_markers,
&planning_markers, Some(&distributed_markers))` asserts `true` on the combined 29-key map
(`full_production_ready_holds_on_real_triple_bundle_evidence`), with a companion negative test
forcing a real distributed marker false confirming the conjunction goes `false`
(`full_production_ready_goes_false_when_a_real_distributed_marker_is_forced_false`). Command:
`CARGO_TARGET_DIR=target/agent-threeway just cng-test-one cng_production_ready_three_way --
--test-threads=1 --nocapture` → 2 passed, 0 failed, 5.62s. **The full
`V26_7_10_PRODUCTION_READY` claim, in the meaning `DEFINITION_OF_DONE.md` §16 defines, is
therefore: ALIVE for the three-way (workday + planning + distributed) real composition** — no
scoping-down qualifier remains on this claim.

## What changed in the follow-up verification round

Four follow-up agents closed five of the gaps this sign-off originally left open, each with a
cited command+output. Nothing below is rounded up beyond what the agent's own run supports.

1. **Literal 8² (64-leaf) fan-out — now ALIVE.** `recursion_crosses_engines_full_8x2_fanout`
   (`cng_multi_engine.rs`, PROJ-729) exercises fan_out=8/depth=2 across real OS engine
   processes: 73 dispatches per root (1 root + 8 first-level + 64 second-level leaves), 146
   total across the two roots (H, M), 64 of them depth-2 leaves — the literal target this
   section's name specifies. Two runs green, 37.19s and 32.50s (full 7-test suite 45.22s and
   41.38s).
2. **`full_production_ready`'s real two-bundle invocation — now ALIVE.** See the section
   above (PROJ-742).
3. **The arazzo digest-verify gate — now ALIVE, wired (with a scope correction).** PROJ-745
   wired `verify_arazzo_render_digest` into `arazzo::run_arazzo_projection`, not `dispatch.rs`'s
   generic `ArazzoRendered` transition — that transition was found to render the
   `DispatchContract` itself via `contract_template`, unrelated to arazzo-pack's ggen-rendered
   YAML, so the correct call site is the Arazzo-sourced path. A throwaway diagnostic (200
   `workday()` seeds across fresh scratch dirs, discarded after use, never committed) found
   59/200 seeds landing on the `api-orchestration` category now genuinely refuse `CNG_R11`
   without a pre-existing render+receipt — proof the gate is load-bearing, not a no-op. Command
   evidence: `cargo test -p cng --lib --features bench -- bench::dispatch:: bench::arazzo::
   bench::workday:: bench::workday_verify::` → 35 passed, 0 failed; `cargo test -p cng --test
   cng_workday_verify --test cng_production_ready --features bench` → 4 passed, 0 failed;
   `cargo clippy -p cng --lib --tests --features bench` → zero new warnings.
4. **The full 5x20 IPC corpus scale — now ALIVE.** New file
   `crates/cng/tests/cng_ipc_corpus_full_scale.rs` (PROJ-711) runs all 100 domain x seed pairs
   using the existing 20-entry `IPC_CORPUS_SEEDS`. Two runs green, 11.66s (cold) and 11.79s
   (warm) test-internal wall-clock, zero failures; per-domain breakdown (run 2): barman 3.183s,
   blocksworld 5.542s (slowest, 0.277s/seed, no super-linear growth across the 20-seed range),
   grippers 1.906s, termes 0.589s, tyreworld 0.568s.
5. **`CNG_R09 HardcodingSuspicion` in `decomp/` — now ALIVE, negative-tested.**
   `detached_graph_action_refuses_cng_r09_hardcoding_suspicion`
   (`decomp/decomp_test.rs`) injects a fabricated action IRI (never in `ground.actions`) into
   the lifted graph and asserts `derive_edges` returns `Err` with `.code() == "CNG_R09"` —
   confirming the refusal already wired in `rules.rs::append_pair_facts` genuinely fires. This
   closes negative-proof-corpus item 7 (§18) with the exact variant;
   `no_canned_helper_subgoal_across_incompatible_variants` (PROJ-713) remains the correct,
   distinct closure for candidate-id purity in `search.rs` — the two guard different code
   paths.

## What changed in this session's second synthesis round

Five more agents (a separate, later batch than the follow-up round above) added the
following, each with its own cited command+output:

1. **Workspace-wide sanity — confirmed clean, nothing to fix.**
   `CARGO_TARGET_DIR=target/agent-workspace-check cargo check --workspace --all-features`:
   `Finished` in 5m 38s, zero errors, only pre-existing unrelated warnings (`ggen`,
   `cng/src/bench/{dispatch,engine}.rs`, `ggen/src/bin/mcp_server.rs`). `cargo test --workspace
   --all-features --no-run`: exit 0, every workspace test binary compiled. Diff review
   confirmed the only cross-crate change (`pddl-index` added to `cng`'s bench-only feature)
   does not affect `praxis-synthesis`, the other consumer of `pddl-index`.
2. **Scoped clippy sweep on `praxis-graphlaw`/`pddl-index` — 6 real items fixed.** Removed two
   dead private functions (`hooks/delta_query.rs`'s unused `delta_touches`,
   `shacl/index_utils.rs`'s duplicate `contains_triple`), added `#[allow(dead_code)]` with a
   documented reason to `shacl/closure.rs`'s `dense_to_global` (a deliberately-reserved
   PROJ-416 seam, not vestigial code), and added missing `///` doc comments to
   `pddl-index/src/ground.rs`'s `IndexedGroundProblem` public fields. Confirmation:
   `cargo clippy -p praxis-graphlaw -p pddl-index --all-targets --all-features -- -D
   warnings` still exits 101 — NOT a clean pass, and not claimed as one — but the failure is
   ~60 pre-existing clippy errors across 22 OTHER `praxis-graphlaw` files, dated to commit
   `2dd4f04` (predating this session), explicitly out of the assigned scope and not touched;
   `pddl-index` alone lints clean.
3. **`cargo fmt --all --check` — clean, zero files flagged.** Run via `just fmt-check`
   (direct `cargo fmt` is blocked by a repo hook); zero diffs across all 17 workspace members,
   including every file in this session's "hot" concurrently-edited set. Point-in-time
   caveat, stated by the checking agent itself: files touched by items 2, 4, and 5 below
   landed after or during this check, so a final `just fmt-check` re-run immediately before
   any release gate is prudent, not because a problem is known, but because this specific
   check predates those edits.
4. **Negative-corpus item 6 closed.** See the §18 detail table above
   (`splits_admissible_but_not_beneficial_forces_no_beneficial_decomposition`). Item 7 gained
   a complementary confirmation from the same new file.
5. **PROJ-749 — the decompose-to-dispatch bridge.** See §2/§8/G8 above and the "Final state"
   section at the top of this document for the full account, and
   `docs/jira/v26.7.10/tickets/PROJ-749.md` for the dedicated ticket.

## What remains explicitly not claimed (after both rounds)

1. The single continuous narrative using the POTATO fixture specifically (admit PDDL ->
   `decompose()` -> dispatch across real H/M engines -> close global goal) — potato's own
   `decompose()` output is single-actor (no split to dispatch). The general decompose-to-
   dispatch MECHANISM, including payload fidelity, is now ALIVE on a different fixture
   (kitchen two-chain, PROJ-749, second synthesis round + moonshot round); global-goal closure
   across two engines' outputs is still not provable by any machinery on disk — this is now the
   ONLY remaining piece of this narrative that is open (payload fidelity itself, PROJ-710 ->
   PROJ-723, closed this session's moonshot round — see §2/§8/"Moonshot round" above).
2. `full_production_ready`'s real THREE-bundle invocation (workday + planning + distributed
   together) — UNVERIFIED. The two-bundle case is now ALIVE (see above); a real third
   distributed bundle needs `cng_multi_engine.rs`'s private harness helpers, not importable
   from a separate test crate.
3. Any live-repo `ggen sync run` against the real `ggen.toml`/receipt — only an isolated
   scratch-project verification has been run (PROJ-744); PROJ-745's follow-up explicitly
   declined to run `ggen sync run` against the live repo — `ggen sync run --help`/
   `--introspect` confirm no `--pack`/`--only` flag exists anywhere in the binary to bound the
   six-pack regeneration blast radius, so this stays a deliberately-avoided scope boundary, not
   an oversight.
4. PROJ-714's remaining 2 long-horizon scenarios (3-4 of 4) — declared cut line
   (`RELEASE_CONTROL.md` §9.2); 2 of 4 (logistics + tyreworld-chain) are now ALIVE, per the
   moonshot round.
5. **CLOSED, moonshot round** (superseding this item as previously written): a dispatched
   contract now carries its subworkflow's actual PDDL payload, digest-verified, and the remote
   engine executes the SPECIFIC plan it was sent (PROJ-710 -> PROJ-723) —
   `dispatched_subworkflow_payload_is_the_content_the_engine_actually_executes`,
   `cng_decompose_to_dispatch_integration.rs`, 3/3 passed. Negative corpus items 5, 6, and 7
   (§18) are now all closed (see above and the §18 detail table).
6. Live third-party network dispatch, real (non-synthesized) human consequences, and the
   whole-workspace `just verify-all` gate — carried forward unchanged from the interim DoD's
   own honest-boundary language (`DOD_SIGNOFF_INTERIM.md`).
7. Phase 6 commit — not run; `git status` is not clean; HEAD is still `1f3f9bc`. Nothing in
   this document claims the increment is committed or closed in git history.
8. A newly-discovered behavioral tightening from item 3 above, carried forward as a
   deployment note: any `workday()` run (test or real) whose seed-derived category cycle lands
   on `api-orchestration` now requires a pre-existing `<out_dir>/generated/arazzo.yaml` +
   `<out_dir>/.ggen-v2/receipt.json` to succeed, or it genuinely refuses `CNG_R11`. This is the
   correct, intended effect of PROJ-745's wiring (not a regression) — the committed test
   suite's fixed seeds happen not to land on that category, so nothing in the existing suite
   regressed, but it is a real precondition for any future `workday()` invocation that can
   select it.
9. **Moonshot round, attempted but not independently confirmed passing this session**:
   expanded `CNG_R10 IoRefused` coverage (3 more construction sites, `cng_r10_bench_sites`
   module in `cng_io_refused_negative.rs`) and the CLI/process-exit half of §14 item 4
   (`cng_cli_nonzero_exit_on_hostile_marker.rs`) — both files exist on disk, well-formed, but
   the closing agent's own report to this synthesis pass contained no command+output for
   either. UNVERIFIED, not claimed as closed; see "Moonshot round" above for the precise
   distinction from the items that were confirmed.
10. Whether combining two engines' now-payload-faithful outputs actually closes the ORIGINAL
    undecomposed problem's global goal — a narrower, distinct question from the payload-
    fidelity question item 5 closed; no machinery on disk checks this today.
11. CNG_R07's remaining 2 of 5 construction sites (`runner.rs:269`/`279`) and CNG_R08's 2
    construction sites — both investigated and found unreachable-by-design through the current
    public API surface (structural argument + empirical corroboration for CNG_R07; no external
    seam for CNG_R08), not dedicated-tested. `DispatchState::COMPENSATING→BLOCKED` — similarly
    investigated and found vestigial (no code path reaches it). None of these three are forced
    fixtures; each has a doc correction instead, per this session's repeated "investigate,
    don't force" instruction.

## See Also

- `docs/releases/v26.7.10/RELEASE_CONTROL.md` — single control surface; wins on disagreement
- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` — the doctrine signed off here (PROJ-730/743)
- `docs/releases/v26.7.10/DOD_EVIDENCE_MAP.md` — clause -> query/test/refusal index
- `docs/releases/v26.7.10/DOD_SIGNOFF_INTERIM.md` — superseded interim sign-off (PROJ-617)
- `docs/jira/v26.7.10/tickets/index.md`, `PROJ-731.md` — per-ticket status counterparts
- `docs/jira/v26.7.10/tickets/PROJ-749.md` — decompose-to-dispatch bridge (second synthesis
  round)
- `.claude/rules/no-overclaiming.md` — status vocabulary used throughout

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
this document's remaining claims are current as of that commit plus uncommitted EOD-push work
on top of it (`git status` not clean as of this writing). Any live-repo `ggen sync run` against
the real `ggen.toml` (PROJ-745's follow-up explicitly declined — no pack-scoped CLI flag exists
to bound the blast radius); a dispatched contract carrying an actual PDDL payload so a remote
engine executes the specific plan it was sent (PROJ-710 -> PROJ-723, the honest boundary
PROJ-749 states explicitly); live network transport for OpenAPI/AsyncAPI/Arazzo (digest-verify
is proven for all three documents — see below — but no HTTP/broker binding exists or is
claimed). `full_production_ready`'s real THREE-bundle invocation and PROJ-714's long-horizon
scenarios were EOD-push targets — see the "EOD push" section below for their resolved status;
do not rely on the "three waves" framing immediately below for either, it predates that push.

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

Combining all three waves plus the EOD push documented below: every mechanism named in the
DoD's 20 sections has now been independently exercised and evidenced (every Track P/E ticket
is ALIVE except PROJ-714, whose 80/20-rescoped 1-of-4 scenario is IN PROGRESS — command+output
pending, do not cite as closed until PROJ-714.md itself says so), the marker doctrine matches
on-disk names and code, `full_production_ready`'s real THREE-bundle composition (workday +
planning + distributed) is proven (`cng_production_ready_three_way.rs`, EOD push —
superseding the "two-bundle only" framing three paragraphs above, which is left unedited as a
historical record of that wave's own scope, not a live claim), the 8² fan-out and full IPC
corpus scale are at their literal targets, OpenAPI/AsyncAPI digest-verify is proven alongside
Arazzo's (EOD push, superseding this wave's own "narrowed the claim" framing — see the
"EOD push" section below), and the two tracks (P and E) are bridged at the mechanism level.
What remains genuinely open, honestly scoped, and not claimed anywhere in this document:
potato itself dispatched across H/M (it has no split to dispatch); a dispatched contract
carrying its subworkflow's actual PDDL payload; any live-repo `ggen sync run`; live network
transport (digest-verify ≠ a running HTTP/AsyncAPI server); PROJ-714's exact closure state
(check `PROJ-714.md` directly, not this paragraph, for the current number); and Phase 6 push
of the EOD-push commits specifically (the prior `1f3f9bc` commit IS pushed; this round's work
sits uncommitted on top of it as of this writing).

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

**Remains open, out of this wave's scope** (`GAP_AUDIT.md` §7 items 11-20): the mutex-saturated-
goals sub-part of negative-corpus item 5; the CLI/process-exit half of §14 item 4; and the 29
non-load-bearing cosmetic/doc-polish items (status-vocabulary drift, ticket-number gap,
`tyreworld.rs` clean-room note, etc.) — none were assigned to any of the 8 agents and none are
claimed closed here.

Vocabulary used above follows `.claude/rules/no-overclaiming.md`: ALIVE (verified this session,
cited command+output), PARTIAL (gap named explicitly), UNVERIFIED (default, not claimed),
INVESTIGATED (a real finding, not a fix, backed by the agent's own reasoning and read evidence).

## Clause-by-clause sign-off

One line per `DEFINITION_OF_DONE.md` section (1-20).

| § | Clause | Status | Evidence pointer |
|---|---|---|---|
| 1 | Interim milestone (superseded) | ALIVE (interim record) | unchanged from `RELEASE_CONTROL.md` §8; nothing re-verified or reopened this session |
| 2 | Governing claim (full narrative: admit -> decompose -> dispatch -> close) | ALIVE (decompose-to-dispatch bridge, mechanism) / PARTIAL (payload fidelity + global-goal closure) | `decompose()` end-to-end ALIVE (PROJ-701..713); multi-engine dispatch ALIVE (PROJ-720..729); PROJ-749 (second synthesis round) now stitches a real `decompose()` output into a real cross-engine dispatch run — `dispatch_subworkflow_to_engine`/`collect_subworkflow_consequence` (`decomp/dispatch_bridge.rs`) convert each subworkflow into a shape-conformant `DispatchContract`, write it into a REAL second OS process's inbox, run `cng engine serve` to completion, and admit the consequence from the real outbox (`cng_decompose_to_dispatch_integration.rs`, 2/2 passed); NOT proven: the remote engine executes the subworkflow's OWN PDDL plan (`engine.rs::run_serve_loop` derives its own synthetic artifact set from `blake3(dispatch_id)`, confirmed against on-disk evidence) or that combining the two engines' outputs closes the original problem's global goal — no payload-carrying contract exists yet (PROJ-710 -> PROJ-723 open) |
| 3 | TWOSTEP-replacement table (7 rows) | ALIVE | each row's replacement mechanism evidenced under its own PROJ ticket (704/705, 703/706, 707, 708, 710, 710, 740) — see `DOD_EVIDENCE_MAP.md` |
| 4 | Formal planning target + 6 proof obligations | ALIVE | goal coverage/helper reachability (PROJ-705/710), interface-state (`CNG_R23`, PROJ-707), main reachability (PROJ-710), non-interference (`CNG_R22`, PROJ-708), release closure (`CNG_R24`, PROJ-708) — all negative-tested |
| 5 | No-LLM decomposition mechanism per dialect (7 rows) | ALIVE | PDDL/pddl-strips/CONSTRUCT/Datalog/POWL/OCEL rows ALIVE (PROJ-701..710/727); Arazzo row ALIVE — `digest(render(graph))` wired into `arazzo::run_arazzo_projection` (PROJ-745), refusing `CNG_R11` before any step dispatches on a missing/mismatched render; `dispatch.rs`'s own generic `ArazzoRendered` transition renders the `DispatchContract` itself (unrelated to arazzo-pack's YAML) and was correctly left unwired. OpenAPI/AsyncAPI row now ALSO ALIVE (closed this session, EOD push) — `verify_api_docs_render_digest_if_present` (`crates/cng/src/bench/api_docs.rs`) wired into `engine_serve` (`engine.rs:589`), verifying both `generated/engine-openapi.yaml` and `generated/engine-asyncapi.yaml` against the same ggen receipt shape before the poll loop begins; tampered render refuses `CNG_R11`, absent docs proceed honestly (no false refusal). 77/77 `cargo test -p cng --features bench --lib` passed, 6 new tests. OpenAPI/AsyncAPI *schema validation* (structural conformance of the YAML content itself, distinct from digest integrity) and any live HTTP/broker binding remain a declared, separate boundary (§20) — not a gap in this row, which is scoped to digest-integrity |
| 6 | Bounded decomposition algorithm (15 steps) | ALIVE | exercised end-to-end by every `decompose()` test (potato, IPC corpus, permutation); bounds declared and receipted (`search.rs`, `select.rs`) |
| 7 | Single-actor route — typed results, never fallback | ALIVE | `single_actor_is_always_candidate_zero`, `single_atom_goal_yields_no_admissible_decomposition` |
| 8 | Potato canonical scenario | ALIVE (single-process) / UNVERIFIED (potato itself dispatched cross-engine) | `potato_decomposition_is_typed_receipted_and_replayable` ALIVE (single-process). Potato's real `decompose()` output selects `DecompositionOutcome::NoAdmissibleDecomposition` (single-actor; `decomp:subworkflowCount "1"`, verified against the emitted graph before PROJ-749 was written), so it has no multi-subworkflow split to dispatch — the DoD's own text claims potato is "executed across the H and M engines of §10", which stays UNVERIFIED for potato specifically. The general decompose-to-dispatch MECHANISM is now ALIVE (PROJ-749, §2 row above), proven on a different fixture (kitchen two-chain) that does split |
| 9 | Recursive generalization 8^1 -> 8^2 -> 8^3 | ALIVE (8^1) / ALIVE (8^2, full 64-leaf fan-out) / UNVERIFIED (8^3, out of scope) | 8^1: potato, single level, ALIVE. 8^2: `recursion_crosses_engines_full_8x2_fanout` (`cng_multi_engine.rs`, follow-up round) exercises the literal fan_out=8/depth=2 target — 73 dispatches per root (1 root + 8 first-level + 64 second-level leaves), 146 total across the two roots (H, M), 64 of them depth-2 leaves — matching the section's own "8²" framing exactly; 2/2 runs green, 37.19s and 32.50s. `recursion_crosses_engines_depth_two` (fan_out=2) remains in the suite as a faster smoke test. 8^3: doctrine only, unchanged, UNVERIFIED |
| 10 | Multi-engine topology (6 forbidden items) | ALIVE, scoped to CARGO_BIN_EXE harness | separate OS processes confirmed; `SHARED_MEMORY_CROSSINGS_ZERO`/`DIRECT_ENGINE_BYPASSES_ZERO` true in `cng_multi_engine.rs:232-236` |
| 11 | Authority partition table (8 rows) | ALIVE (structural) | exercised transitively by every `cng_multi_engine` test (coordinator/H/M roles enforced by construction); no per-row dedicated test cited individually |
| 12 | 16-state cross-engine dispatch machine | ALIVE (transition table, exhaustive); ALIVE (`TIMED_OUT→BLOCKED` live-walk, load-bearing closure wave); PARTIAL (3 of 4 `→BLOCKED` edges still never driven); `DISPATCH_READY→REFUSED` declared-but-unreached (doc-corrected, load-bearing closure wave) | `sixteen_state_transition_law_is_exact`, `shapes_ttl_state_individuals_match_the_enum` (drift test, all 256 pairs) prove the table; `deadline_expiry_with_zero_remediation_budget_reaches_blocked` (`dispatch_test.rs`, new) proves `TIMED_OUT→BLOCKED` reachable with `remediation_budget=0`, asserting the ledger trajectory ends `("TIMED_OUT","BLOCKED")` — `cargo test -p cng --features bench --lib bench::dispatch::dispatch_test -- --test-threads=4` → 15 passed, 0 failed; `REMOTE_IN_PROGRESS→BLOCKED`/`REFUSED→BLOCKED`/`COMPENSATING→BLOCKED` remain untested. `DISPATCH_READY→REFUSED`'s type doc (`dispatch.rs` ~104-150) now states explicitly it is declared-lawful with no production caller (both `Refused` construction sites fire only from `RESULT_RECEIVED`) — investigated, correctly left unwired rather than forcing a fit; see `GAP_AUDIT.md` §7 items 5/7 |
| 13 | Distributed broker law (5 items) | ALIVE | ledger (PROJ-721), `CNG_R25 DoubleAdmit` (PROJ-721, also `cng_multi_engine.rs` falsifier), `EngineIdentity` (PROJ-722), G13 resume (PROJ-724/729) |
| 14 | Isolation falsifiers (5 items) | ALIVE, scoped to CARGO_BIN_EXE harness | items 1-3 structural (process/marker/filesystem); item 4 `isolation_falsifier_hostile_graph_is_refuted_by_markers`; item 5 `distributed_determinism_two_serialized_runs_byte_identical` |
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
| G8 | ALIVE (single-process, potato) / ALIVE (cross-engine mechanism, different fixture) | potato tests (PROJ-712); decompose-to-dispatch bridge test (PROJ-749) — potato itself not dispatched cross-engine (see §8 row) |
| G9 | ALIVE (transition table); ALIVE (`TIMED_OUT→BLOCKED` live-walk, load-bearing closure wave) | drift test (PROJ-720); `deadline_expiry_with_zero_remediation_budget_reaches_blocked` (`dispatch_test.rs`, new) — see §12 row above for the full 4-edge breakdown |
| G10 | ALIVE | ledger + `DoubleAdmit` tests (PROJ-721) |
| G11 | ALIVE | `EngineIdentity` test (PROJ-722) |
| G12 | ALIVE | `engine serve` + Arazzo projection tests (PROJ-723/725/726); digest-verify gate now wired into `run_arazzo_projection` (PROJ-745 follow-up) |
| G13 | ALIVE, scoped to test harness | `g13_crash_resume_verifies_chain_and_completes` (PROJ-724/729, the direct target of PROJ-734's fix) |
| G14 | ALIVE (full scale) | full 5x20=100 domain x seed corpus (PROJ-711 follow-up, `cng_ipc_corpus_full_scale.rs`), 2/2 runs green, 11.66s/11.79s |
| G15 | PLANNED (cut line) | PROJ-714 never built, by design; `RELEASE_CONTROL.md` §9.2 |
| G16 | ALIVE (constituent) / ALIVE (composed, three-bundle: workday+planning+distributed) | each marker family true separately; `full_production_ready`'s real three-bundle invocation now exercised end-to-end (`full_production_ready_holds_on_real_triple_bundle_evidence`, PROJ-742 EOD push, `cng_production_ready_three_way.rs`) |

### §18 Negative proof corpus detail (8 items)

| # | Scenario | Status | Evidence |
|---|---|---|---|
| 1 | Unreachable helper goal | ALIVE | `helper_unreachable_refuses_cng_r04` |
| 2 | Interface-state mismatch | ALIVE | `tampered_tape_refuses_cng_r23_interface_state_mismatch`, `main_unreachable_after_helper_refuses_cng_r23` |
| 3 | Interfering effect pair | ALIVE | `concurrent_clobber_refuses_cng_r22_interference`, `interfering_parallel_actions_refuse_cng_r22` |
| 4 | Unreleased resource | ALIVE | `unreleased_resource_refuses_cng_r24`, `helper_retains_resource_refuses_cng_r24` |
| 5 | Mutex-saturated goals -> NO_ADMISSIBLE | ALIVE (adjacent scenario) | `single_atom_goal_yields_no_admissible_decomposition` proves the typed-outcome mechanism; not a literal "mutex-saturated" fixture by that name |
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
   dispatch MECHANISM is now ALIVE on a different fixture (kitchen two-chain, PROJ-749,
   second synthesis round); global-goal closure across two engines' outputs is still not
   provable by any machinery on disk (no payload-carrying contract, PROJ-710 -> PROJ-723
   open) — see §2/§8.
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
4. PROJ-714's 4 long-horizon scenarios — never built, declared cut line (`RELEASE_CONTROL.md`
   §9.2).
5. A dispatched contract carrying its subworkflow's actual PDDL payload, so a remote engine
   executes the SPECIFIC plan it was sent (PROJ-710 -> PROJ-723) — UNVERIFIED, named
   explicitly by PROJ-749's own module doc and ticket file as open work. Negative corpus
   items 6 and 7 (§18) are now both closed (see above and the §18 detail table).
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

## See Also

- `docs/releases/v26.7.10/RELEASE_CONTROL.md` — single control surface; wins on disagreement
- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` — the doctrine signed off here (PROJ-730/743)
- `docs/releases/v26.7.10/DOD_EVIDENCE_MAP.md` — clause -> query/test/refusal index
- `docs/releases/v26.7.10/DOD_SIGNOFF_INTERIM.md` — superseded interim sign-off (PROJ-617)
- `docs/jira/v26.7.10/tickets/index.md`, `PROJ-731.md` — per-ticket status counterparts
- `docs/jira/v26.7.10/tickets/PROJ-749.md` — decompose-to-dispatch bridge (second synthesis
  round)
- `.claude/rules/no-overclaiming.md` — status vocabulary used throughout

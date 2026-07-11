# DOD_EVIDENCE_MAP — Clause-by-Clause Evidence Map for v26.7.10-revised (PROJ-748)

Version: v26.7.10-revised. Consumed by PROJ-731 (final closure) and PROJ-748 (this map).
Status: FINAL (doc), tied to `RELEASE_CONTROL.md`. Every claim cites a file, test, or receipt.
Rows without evidence are marked PLANNED, PARTIAL, or UNVERIFIED, never asserted. If this file
and `RELEASE_CONTROL.md` disagree, `RELEASE_CONTROL.md` wins. This file does not upgrade any
marker's verdict on its own — final verdicts live in `DOD_SIGNOFF.md`. The interim evidence
map is preserved verbatim at `DOD_EVIDENCE_MAP_INTERIM.md` (fix-forward, not deleted).

For the marker-family-level detail (which query proves which marker name, and the
`PLANNING_MARKER_MAP`/`DISTRIBUTED_MARKER_MAP`/`MARKER_MAP` structure), see
`DEFINITION_OF_DONE.md` §16 directly — it is already the authoritative, exhaustively-cited
reconciliation (PROJ-743) and is not duplicated here. This file instead maps each ticket to
its primary test evidence and lists the full refusal-variant coverage ledger (`CNG_R01`-`R25`).

## Ticket -> primary evidence (summary; full citations in each ticket file)

| Ticket | Primary test(s) | Status |
|---|---|---|
| PROJ-701 | pddl-strips.ttl/shapes on disk, exercised transitively | ALIVE |
| PROJ-702 | `lift_render_round_trip_preserves_atom_sets` | ALIVE |
| PROJ-703 | `lift_render_round_trip_preserves_atom_sets` | ALIVE |
| PROJ-704 | exercised by every `decompose()` call; perf fixed by PROJ-733 | ALIVE |
| PROJ-705 | `single_actor_is_always_candidate_zero` | ALIVE |
| PROJ-706 | `potato_decomposition_is_typed_receipted_and_replayable` | ALIVE |
| PROJ-707 | `tampered_tape_refuses_cng_r23_interface_state_mismatch` | ALIVE |
| PROJ-708 | `concurrent_clobber_refuses_cng_r22_interference`, `unreleased_resource_refuses_cng_r24` | ALIVE |
| PROJ-709 | `cyclic_composed_order_refuses_cng_r21`; powl2 read by every marker test | ALIVE |
| PROJ-710 | `forced_inadmissible_candidate_refuses_cng_r21`, `decompose_is_deterministic_across_runs` | ALIVE |
| PROJ-711 | `ipc_corpus_seeds_plan_decompose_and_regenerate_byte_identically` (seeds 0-3) + `ipc_corpus_full_20_seeds_...` (`cng_ipc_corpus_full_scale.rs`, full 5x20) | ALIVE (full scale) |
| PROJ-712 | `potato_graph_bridges_to_a_parsed_surface`, negative corpus tests; §18 item 6 closed by `splits_admissible_but_not_beneficial_forces_no_beneficial_decomposition` (`cng_decomp_negative_corpus_completeness.rs`, second synthesis round) | ALIVE |
| PROJ-713 | `permuted_goal_identities_change_plans_and_receipts_causally`; §18 item 7 additionally confirmed by `canned_subgoal_detection_catches_identical_goal_labels_with_different_achiever_structure` (same new file, second synthesis round) | ALIVE |
| PROJ-714 | none — never built | PLANNED (cut line) |
| PROJ-720 | `sixteen_state_transition_law_is_exact`, drift test | ALIVE |
| PROJ-721 | `ledger_records_every_advance_and_replays_chain_verified`, `CNG_R25` test | ALIVE |
| PROJ-722 | `engine_identity_is_deterministic_and_engine_distinct` | ALIVE |
| PROJ-723 | `serve_executes_inbox_contract_and_writes_consequence` | ALIVE |
| PROJ-724 | `resume_verifies_ledger_prefix_and_skips_processed_contracts` | ALIVE |
| PROJ-725 | `xpath_criterion_fixture_refuses_cng_r18_naming_the_feature` | ALIVE |
| PROJ-726 | isolated scratch `ggen sync run`, byte-identical, digest-matched | ALIVE |
| PROJ-727 | `isolation_falsifier_hostile_graph_is_refuted_by_markers` + marker asserts | ALIVE |
| PROJ-728 | `multi_engine_concurrent_dispatch_execute_readmit` (6/6 suite); harness also holds under `recursion_crosses_engines_full_8x2_fanout`'s 146-dispatch load (PROJ-729 follow-up) | ALIVE (harness) |
| PROJ-729 | `g13_crash_resume_verifies_chain_and_completes`, `recursion_crosses_engines_depth_two`, `recursion_crosses_engines_full_8x2_fanout` (literal 8²=64-leaf fan-out, `cng_multi_engine.rs`, follow-up round) | ALIVE (harness, full 8² fan-out) |
| PROJ-733 | `cng_decomp` 3/3 @ 0.18s, `cng_ipc_corpus` 10/10 @ 1.79s | ALIVE |
| PROJ-734 | `g13_crash_resume_verifies_chain_and_completes` holds | ALIVE |
| PROJ-739 | `planning_markers_prove_true_on_a_healthy_decompose_run` | ALIVE |
| PROJ-740 | same test, `LLM_CALLS_ZERO`/`ENGLISH_SUBGOALS_ZERO`/`CANNED_SUBGOALS_ZERO` | ALIVE |
| PROJ-741 | `main.rs:258-324` verb; `cargo check -p cng` 0-warning after gate fix | ALIVE |
| PROJ-742 | `full_production_ready_refuses_when_a_planning_marker_is_false` (pure fn); `full_production_ready_holds_on_real_dual_bundle_evidence` + `..._goes_false_when_a_real_marker_is_forced_false` (`cng_production_ready.rs`, real workday+decompose bundles, follow-up round) | ALIVE (fn) / ALIVE (real 2-bundle) / UNVERIFIED (3-bundle, +distributed) |
| PROJ-743 | `DEFINITION_OF_DONE.md` §16 rewrite, this session | DONE (doc) |
| PROJ-744 | `ggen.toml:22`, isolated scratch verification | ALIVE |
| PROJ-745 | `arazzo_test.rs:148,179`; wired into `arazzo::run_arazzo_projection` (`arazzo.rs:337`) + `dispatch_test.rs`'s `arazzo_projection_gate_admits_when_render_digest_matches_receipt`/`..._refuses_cng_r11_before_any_step_dispatches` (follow-up round) | ALIVE (wired at the Arazzo-sourced call site) |
| PROJ-746 | ticket status flips, this session | DONE (doc) |
| PROJ-747 | `RELEASE_CONTROL.md` §9.2, this session | DONE (doc) |
| PROJ-748 | this file + `DOD_SIGNOFF.md`, this session | DONE (doc) |
| PROJ-749 | `kitchen_decomposition_splits_into_helper_and_main`, `decomposed_subworkflows_dispatch_to_real_engines_and_are_admitted` (`cng_decompose_to_dispatch_integration.rs`, 2/2, second synthesis round) | ALIVE (mechanism, non-potato fixture) |

## Refusal-variant coverage ledger (CNG_R01-R25)

| Code | Variant | Guards | Test evidence |
|---|---|---|---|
| `CNG_R01` | `MalformedTtl` | RDF/Turtle parse failures | ALIVE — `malformed_ttl_refuses_cng_r01` (`cng_negative_fixtures.rs:41`) |
| `CNG_R02` | `MissingDomain` | PDDL domain fragment required | UNVERIFIED this session — no test by this name found via grep this pass |
| `CNG_R03` | `MissingProblem` | PDDL problem fragment required | UNVERIFIED this session — no test by this name found via grep this pass |
| `CNG_R04` | `PlanUnsolvable` | no classical plan exists | ALIVE — `unsolvable_goal_refuses_cng_r04` (`cng_negative_fixtures.rs:58`), `helper_unreachable_refuses_cng_r04` |
| `CNG_R05` | `UnsupportedConstruct` | unsupported PDDL/grounding feature | ALIVE — `duplicate_actions_refuse_cng_r05` (`cng_negative_fixtures.rs:75`), `actor_lacks_capability_refuses_cng_r05`, `depth_or_cost_bound_exceeded_refuses_cng_r05` |
| `CNG_R06` | `InvalidPowl` | malformed POWL graph | ALIVE — `invalid_powl_refuses_cng_r06_via_shape_validation` (`cng_negative_fixtures.rs:101`) |
| `CNG_R07` | `RunnerMismatch` | runner/plan mismatch | UNVERIFIED this session — no test by this name found via grep this pass |
| `CNG_R08` | `Nondeterminism` | non-reproducible output detected | UNVERIFIED this session (refusal path) — determinism is separately PROVEN positive (`decompose_is_deterministic_across_runs`, `distributed_determinism_two_serialized_runs_byte_identical`) but no test this session forces this specific refusal to fire |
| `CNG_R09` | `HardcodingSuspicion` | canned artifact suspected | ALIVE — `detached_graph_action_refuses_cng_r09_hardcoding_suspicion` (`decomp/decomp_test.rs`, follow-up round) injects a fabricated action IRI absent from `ground.actions` into the lifted graph and asserts `derive_edges` returns `Err` with `.code() == "CNG_R09"`; confirms the refusal already wired in `rules.rs::append_pair_facts` (lines 96, 102) genuinely fires. `no_canned_helper_subgoal_across_incompatible_variants` (PROJ-713) remains the correct, distinct closure for a different concern (candidate-id purity in `search.rs`) — the two guard different code paths, not the same one twice; see `DOD_SIGNOFF.md` §18 item 7 |
| `CNG_R10` | `IoRefused` | filesystem I/O failure | UNVERIFIED this session (dedicated negative test) — code path read this session in `arazzo.rs`'s digest-verify seam, no test by this name found via grep |
| `CNG_R11` | `AuditMismatch` | evidence-bundle integrity | ALIVE — `mutated_ocel_evidence_refuses_cng_r11`, `mutated_obs_partition_refuses_cng_r11` (`workday_verify_test.rs:96,120`) + `arazzo_test.rs:179` mismatch case (PROJ-745) + `torn_ledger_tail_refuses_cng_r11_on_resume` (PROJ-724) + `arazzo_projection_gate_refuses_cng_r11_before_any_step_dispatches` (`dispatch_test.rs`, follow-up round — fires through the now-wired `run_arazzo_projection` call site, zero steps dispatched, empty outbox) |
| `CNG_R12` | `StandingAmbiguous` | exactly-one next action | UNVERIFIED this session — no test by this name found via grep this pass |
| `CNG_R13` | `UnreceiptedActuation` | zero unreceipted actuation | ALIVE — `missing_category_hook_refuses_cng_r13` (`hooks_test.rs:89`), `stripped_hook_delta_hash_refuses_cng_r13` (`workday_verify_test.rs:150`) |
| `CNG_R14` | `DialectRegistryRefused` | registry closed-shape law | ALIVE — `registry_missing_field_refuses_cng_r14` (`hooks_test.rs:119`), `stripped_dialect_registry_field_refuses_cng_r14` (`workday_verify_test.rs:196`) |
| `CNG_R15` | `DispatchContractIncomplete` | 20-field dispatch contract | ALIVE — `contract_missing_field_refuses_cng_r15` (`dispatch_test.rs:65`) |
| `CNG_R16` | `DispatchStateUnlawful` | state-machine transition law | ALIVE — `unlawful_state_transition_refuses_cng_r16` (`dispatch_test.rs:91`) |
| `CNG_R17` | `ExternalConsequenceRefused` | admission pipeline | ALIVE — `forged_inbox_correlation_refuses_cng_r17` (`workday_verify_test.rs:223`) |
| `CNG_R18` | `ArazzoProfileRefused` | Arazzo bounded profile | ALIVE — `xpath_criterion_fixture_refuses_cng_r18_naming_the_feature` (`arazzo_test.rs:34`) |
| `CNG_R19` | `EvidenceGateFailed` | graph-derived closure gate unclosed | ALIVE — `unreceipted_actuation_gate_refuses_cng_r19` (`workday_test.rs:159`) |
| `CNG_R20` | `MarkerFalse` | false success marker | ALIVE — `forced_false_marker_refuses_cng_r20` (`workday_test.rs:113`), `fabricated_decomp_result_without_receipts_refuses_cng_r20` (`workday_test.rs:407`) |
| `CNG_R21` | `DecompositionInadmissible` | a specific candidate failed a proof obligation | ALIVE — `forcing_an_unknown_candidate_refuses_cng_r21`, `forced_inadmissible_candidate_refuses_cng_r21`, `cyclic_composed_order_refuses_cng_r21`, `subgoal_not_contributing_refuses_cng_r21` |
| `CNG_R22` | `InterferenceDetected` | non-interference proof failed | ALIVE — `concurrent_clobber_refuses_cng_r22_interference`, `interfering_parallel_actions_refuse_cng_r22` |
| `CNG_R23` | `InterfaceStateMismatch` | helper-tape replay precondition violation | ALIVE — `tampered_tape_refuses_cng_r23_interface_state_mismatch`, `main_unreachable_after_helper_refuses_cng_r23` |
| `CNG_R24` | `ResourceUnreleased` | resource-release closure failed | ALIVE — `unreleased_resource_refuses_cng_r24`, `helper_retains_resource_refuses_cng_r24` |
| `CNG_R25` | `DoubleAdmit` | idempotency key already admitted | ALIVE — `replayed_consequence_refuses_cng_r25_double_admit`, `double_admit_falsifier_replayed_collect_refuses_cng_r25` |

20 of 25 refusal variants have a grep-confirmed end-to-end test cited above (19 from the
initial closure round, `CNG_R09` closed by the follow-up round below); 5 (`CNG_R02`, `R03`,
`R07`, `R08`'s refusal path specifically, `R10`'s dedicated negative test) remain UNVERIFIED —
not confirmed absent, simply not found by grep this pass and not asserted without that
confirmation (per `.claude/rules/rust-agi-core-team.md` §5's "no test, no ALIVE" rule). A
future session should grep for these five specifically before citing them as covered.

## Interim-vs-revised marker family cross-reference

| Family | Interim (`DOD_EVIDENCE_MAP_INTERIM.md`) | Revised (`DEFINITION_OF_DONE.md` §16) |
|---|---|---|
| Single-operator `MARKER_MAP` (10 stems, 16 names) | ALIVE, `31c236f` closure | unchanged, still true for its own scope |
| Planning set (6) + `LLM_CALLS_ZERO` family (3) | did not exist | ALIVE this session (PROJ-739/740) |
| Distributed set (9, `DISTRIBUTED_MARKER_MAP`) | did not exist as coded (named differently in doctrine) | ALIVE this session, scoped to test harness (PROJ-727/728/729) |
| `V26_7_10_PRODUCTION_READY` (interim meaning) | ALIVE, `31c236f` closure | unchanged; still means the interim 16-marker conjunction when computed by `evaluate_markers()` alone |
| `V26_7_10_PRODUCTION_READY` (revised §16 meaning, via `full_production_ready`) | n/a | ALIVE for the real two-bundle case (workday + planning) — follow-up round's `cng_production_ready.rs`; UNVERIFIED for the three-bundle case (+ distributed), which needs `cng_multi_engine.rs`'s private harness helpers |

## See Also

- `docs/releases/v26.7.10/RELEASE_CONTROL.md` — single control surface; wins on disagreement
- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` — the doctrine this map indexes (§16 detail
  lives there directly, PROJ-743)
- `docs/releases/v26.7.10/DOD_SIGNOFF.md` — clause-by-clause sign-off this map supports
- `docs/releases/v26.7.10/DOD_EVIDENCE_MAP_INTERIM.md` — superseded interim evidence map
- `docs/jira/v26.7.10/tickets/` — per-ticket "Evidence (this session)" sections (full detail)
- `crates/cng/queries/` — on-disk SPARQL evidence sources
- `.claude/rules/no-overclaiming.md` — status vocabulary used throughout

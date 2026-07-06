# v26.7.3 — Definition of Done (gate list with evidence)

Every gate below cites the file that implements it and the test that proves
it. A gate without a cited test is not claimed. Reproduce everything with
the commands at the bottom; nothing here asserts a hash — every hash in the
system is computed at run time and re-derived at verification time.

## Gates

| # | Gate | Implementation | Evidence (test file :: test name) |
|---|------|----------------|-----------------------------------|
| 01 | Lord's Prayer kernel: all 11 clauses typed, exact coverage | `crates/praxis-synthesis/src/kernel.rs`, `crates/praxis-synthesis/ontology/lord_prayer.ttl` | `tests/kernel_coverage.rs :: all_11_clauses_extract_and_hash_is_stable_across_reorder`, `ten_clause_kernel_refuses_naming_the_missing_clause`, `unknown_clause_name_refuses` |
| 02 | Rice quarantine: decidable checks only, raw meaning never executes | `src/quarantine.rs`, `src/delta.rs` | `src/quarantine.rs :: quarantine_refuses_malformed_bytes_decidably`, `post_state_vocab_violation_is_refused_at_admission`; `tests/kernel_coverage.rs :: raw_scripture_is_quarantined_data_not_law` |
| 03 | RDF life graph: queryable deviation vocabulary | `src/life.rs` | `tests/kernel_coverage.rs :: life_graph_queries_return_correct_subjects` |
| 04 | Canonicalization: surface-invariant graph/event hashes | `src/graph.rs`, `src/delta.rs` | `src/delta.rs :: event_hash_is_surface_invariant_and_ttl_hash_is_not`; `tests/foreign_graph_tests.rs :: foreign_graph_verifier_agrees_across_a_reformat` |
| 05 | RDF -> PDDL projection: fired hook -> workflow fragment -> Solver8 | `src/ground.rs` | `tests/prayer_kernel.rs :: provision_anxiety_grounds_the_daily_prayer_workflow`; `tests/deviation_routes.rs :: open_debt_fires_by_rule_and_grounds_confess_and_repair`, `missing_receipt_grounds_the_one_step_repair_fragment` |
| 06 | DayWindow: graph entity + refuse-or-reschedule route | `src/life.rs` (`DAY_WINDOW`, `scheduled_in_window`) | `tests/deviation_routes.rs :: five_same_day_placements_in_one_delta_refuse_with_reschedule`, `four_same_day_placements_do_not_trip_the_overload` |
| 07 | Knowledge hooks: closed vocab, 5 kinds, refused kinds by name | `src/hooks.rs` | `src/hooks.rs` module tests; `tests/prayer_kernel.rs :: resentment_open_loop_fires_by_datalog_rule_and_release_quiets_it`, `day_window_over_the_eight_bound_trips_the_temptation_guard` |
| 08 | Deviation routes (debt, receipt-missing, overload, sponsor) | `ontology/lord_prayer.ttl` hooks + `src/firing.rs` | `tests/deviation_routes.rs` (all 9 tests) |
| 09 | Agent assignment: graph-declared `wf:handler`, closed registry | `src/handlers.rs` | `src/handlers.rs :: unknown_handler_refused_exact_key_only`, `two_declarations_two_binding_hashes`; `tests/firing_chain.rs :: unknown_handler_is_refused_before_solving_and_still_chained` |
| 10 | Delegability lattice (human-only < assistive < automatable < verifiable) | `src/handlers.rs :: Delegability` | `src/handlers.rs :: human_only_is_a_delegability_violation_for_automated_runners`, `handler_without_delegability_is_ill_formed` |
| 11 | HumanOnly enforcement scoped to REACHED capabilities | `src/handlers.rs :: judge_delegability`, `src/firing.rs` | `tests/deviation_routes.rs :: human_only_release_resentment_blocks_the_debt_firing`, `automatable_write_prayer_receipt_is_allowed`, `human_only_release_resentment_does_not_block_an_unrelated_firing`; `tests/firing_chain.rs :: human_only_binding_on_an_unused_capability_does_not_refuse` |
| 12 | AA livelock model: 6 classes as datalog programs + 12-step mapping | `src/livelock.rs` | `tests/livelock.rs :: every_class_program_parses_and_evaluates`, `twelve_steps_map_to_soundness_operations` |
| 13 | Resentment / spilled-milk closure + no-infinite-rehearsal | `src/livelock.rs` | `tests/livelock.rs :: test_14_resentment_livelock_detected`, `test_15_release_fact_closes_and_inventory_sees_it`, `test_16_spilled_milk_closes_through_any_of_three`, `test_17_infinite_rehearsal_refused_at_bound` |
| 14 | Receipt chain: outer `praxis:hook-firing:v1` fold over inner v1 | `src/firing.rs :: HookFiringReceipt` | `tests/firing_chain.rs :: completed_firing_chains_and_replays`; inner-chain byte-stability: `tests/prayer_kernel.rs :: v1_chain_golden_pin_direct_execution_unchanged_by_the_hook_layer` |
| 15 | Replay: stage-by-stage re-derivation + payload binding | `src/firing.rs :: replay_firing` | `tests/firing_chain.rs :: forged_payloads_behind_honest_hashes_are_refused_by_name` |
| 16 | Foreign verification: second implementation, both chains | `scripts/foreign_verify_graph.py` (`graph` + `firing` subcommands) | `tests/foreign_graph_tests.rs` (5 tests); `tests/foreign_firing.rs :: foreign_firing_verifier_agrees_on_an_honest_completed_receipt`, `..._a_declared_refusal_receipt`, `foreign_firing_verifier_fails_a_tampered_verdict_payload`. Named limitations in RECEIPTS_REPLAY_VERIFY.md |
| 17 | God/unbounded boundary: never agent/handler/capability; surrender not computation | `src/kernel.rs` (boundary strings), `ontology/lord_prayer.ttl` | `tests/kernel_coverage.rs :: god_is_never_typed_executable_and_deliverance_is_surrendered`; `tests/prayer_kernel.rs :: unbounded_threat_is_surrendered_not_computed`; `tests/firing_chain.rs :: declared_refusal_surrender_is_chained_with_the_graph_reason` |
| 18 | No LLM in runtime | `Cargo.toml` (six offline deps) | `tests/no_llm_runtime.rs :: dependencies_are_exactly_the_offline_allowlist`, `source_contains_no_llm_symbols` |
| 18b | Human-unavailable execution: no interactive/blocking-on-a-human symbol on the execution path | `Cargo.toml`, `src/**/*.rs` | `tests/no_llm_runtime.rs :: source_and_deps_contain_no_interactive_human_symbols` |
| 19 | Docs / claim discipline | `docs/v26.7.3/*.md`, `docs/claims/WITHHELD_CLAIMS.md` | this document set; withheld claims listed explicitly |
| 20 | Reproducibility | commands below | full suite + clippy `-D warnings` + trustless replay green at commit time |

## Repro commands

Run from the repo root (`/Users/sac/praxis`):

```bash
# The whole synthesis suite (module tests + all integration tests).
cargo test -p praxis-synthesis

# Lint gate — must be clean, warnings are errors.
cargo clippy -p praxis-synthesis --all-targets -- -D warnings

# Trustless replay: verify packaged receipts with only python3 + b3sum,
# no cargo, no crate source.
bash scripts/trustless_replay.sh

# Foreign verification directly (a second implementation, Python + b3sum):
python3 scripts/foreign_verify_graph.py graph <ttl-file> <receipt.json>
python3 scripts/foreign_verify_graph.py firing <base.ttl> <adds.ttl> <removes.ttl> <firing_receipt.json>
```

`scripts/foreign_verify.py` (the original workflow-receipt verifier) is
byte-frozen; the graph/firing verifier lives in
`scripts/foreign_verify_graph.py`.

## What "done" does not mean

No claim of completeness beyond the cited tests is made. The withheld
claims register (`docs/claims/WITHHELD_CLAIMS.md`) lists everything this
version deliberately does not claim.

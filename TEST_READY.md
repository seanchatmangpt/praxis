# E2E Test Suite Readiness: Knowledge Hooks

The comprehensive integration test suite for Knowledge Hooks in `praxis-graphlaw` has been fully designed and implemented. It is ready to run and verify the behavior of the backend once the implementation track integrates M1-M5 milestones.

## Test Suite Location
- **Path**: `crates/praxis-graphlaw/tests/knowledge_hooks_e2e.rs`
- **Test Runner Command**: `cargo test -p praxis-graphlaw --test knowledge_hooks_e2e`

## Implemented Test Cases

### Tier 1: Feature Coverage (31 cases)
- **F1: Hook Parsing & Registry**
  - `test_f1_load_valid_single`: Load a single valid hook pack.
  - `test_f1_load_valid_multiple`: Load multiple valid hooks.
  - `test_f1_resolve_prefixes`: Resolve namespaces/prefixes.
  - `test_f1_load_inline_triples`: Load hooks from inline string.
  - `test_f1_query_registered_hooks`: Query registered hooks via SPARQL.
- **F2: Constitutional Gating**
  - `test_f2_refuse_command`: Refuse hooks with forbidden execution commands.
  - `test_f2_refuse_shell`: Refuse hooks with shell execution handlers.
  - `test_f2_refuse_unrecognized_action`: Refuse hooks with unrecognized action handlers.
  - `test_f2_gating_malformed_shacl`: Refuse hooks violating mandatory SHACL law pack shape.
  - `test_f2_gating_rollback_state`: Verify full transaction rollback on gating refusal.
- **F3: First-Class Trigger Dialects**
  - `test_f3_sparql_ask_trigger`: Evaluate SPARQL ASK trigger condition.
  - `test_f3_sparql_select_trigger`: Evaluate SPARQL SELECT trigger condition.
  - `test_f3_count_trigger`: Evaluate Count-based trigger condition.
  - `test_f3_threshold_trigger`: Evaluate Threshold-based trigger condition.
  - `test_f3_delta_trigger`: Evaluate Delta-based trigger condition.
  - `test_f3_datalog_trigger_format`: Evaluate Datalog-based trigger format.
- **F4: Pure Action Projections**
  - `test_f4_project_add_quad`: Verify `kh:addQuad` declarative projection.
  - `test_f4_project_delete_quad`: Verify `kh:deleteQuad` declarative projection.
  - `test_f4_project_add_and_delete`: Verify simultaneous additions/deletions projection.
  - `test_f4_refuse_side_effects`: Block non-pure side effects in projection.
  - `test_f4_project_apply_to_graph`: Apply projected quads to correct target graph.
- **F5: Canonical N-Quads & BLAKE3 Receipts**
  - `test_f5_receipt_single_add`: Generate receipt for a single addition.
  - `test_f5_receipt_sort_determinism`: Ensure receipt hash determinism via lexicographical sorting.
  - `test_f5_receipt_deletion`: Generate receipt for a deletion.
  - `test_f5_get_hook_receipts_api`: Retrieve receipts via `store.get_hook_receipts()`.
  - `test_f5_receipt_format_validation`: Validate receipt structure and fields.
- **F6: Fixpoint Reasoner Integration**
  - `test_f6_single_pass_materialization`: Fire hook in single reasoning pass.
  - `test_f6_multi_pass_cascade`: Cascade hook executions across multiple passes.
  - `test_f6_fixpoint_termination`: Terminate reasoning loops correctly.
  - `test_f6_refusal_rollback`: Rollback entire session upon hook refusal.
  - `test_f6_query_state_post_materialize`: Query final state after materialization.

### Tier 2: Boundary & Corner Cases (30 cases)
- **F1 Boundaries**
  - `test_b1_empty_hook_pack`: Load hook pack containing no hooks.
  - `test_b1_max_name_length`: Load hook with max length name.
  - `test_b1_exceed_max_hooks`: Refuse pack exceeding 12 hook limit.
  - `test_b1_turtle_formatting`: Load hook with odd but valid formatting.
  - `test_b1_missing_mandatory_fields`: Refuse hook missing mandatory fields.
- **F2 Boundaries**
  - `test_b2_empty_shacl_law`: Validate behavior with empty law pack.
  - `test_b2_hidden_side_effects`: Refuse obfuscated side effect keywords.
  - `test_b2_huge_hook_packs`: Parse/gate large text payload pack.
  - `test_b2_conflicting_shacl_constraints`: Refuse conflicting properties violating maxCount.
  - `test_b2_multiple_sequential_loads`: Maintain state over sequential pack loads.
- **F3 Boundaries**
  - `test_b3_empty_trigger_results`: Handle empty trigger match result.
  - `test_b3_window_size_bounds`: Check window size 0 boundary.
  - `test_b3_threshold_boundary_values`: Check threshold max u64 boundary.
  - `test_b3_datalog_program_size_limit`: Parse/gate large datalog program.
  - `test_b3_sparql_syntax_error`: Reject bad SPARQL query syntax.
- **F4 Boundaries**
  - `test_b4_construct_empty_result`: Handle empty CONSTRUCT projection.
  - `test_b4_construct_literal_subject`: Block invalid RDF projections.
  - `test_b4_construct_unsupported_clauses`: Reject unsupported SPARQL clauses.
  - `test_b4_construct_no_op_addition`: Ignore duplicate triple projections.
  - `test_b4_construct_modify_registry`: Prevent hijacking system/registry namespace.
- **F5 Boundaries**
  - `test_b5_receipt_blank_nodes`: Generate receipt with stable blank nodes.
  - `test_b5_receipt_unicode_literals`: Generate receipt with non-ASCII literals.
  - `test_b5_receipt_huge_literals`: Generate receipt with large literal payloads.
  - `test_b5_stable_hash_datatypes_lang`: Generate distinct stable hashes for datatypes/langs.
  - `test_b5_hash_both_add_and_delete`: Generate receipt for combined additions/deletions.
- **F6 Boundaries**
  - `test_b6_infinite_loop_detection`: Terminate loop cascade and prevent hangs.
  - `test_b6_circular_dependency`: Reject static hook cycle at load time.
  - `test_b6_gating_refusal_deep_rollback`: Ensure complete rollback on late refusal.
  - `test_b6_empty_base_facts`: Handle materialization with zero base facts.
  - `test_b6_multi_strata_evaluation`: Evaluate stratified datalog rules and hooks.

### Tier 3: Cross-Feature Combinations (6 cases)
- `test_c3_datalog_construct_delta_cascade`
- `test_c3_gating_construct_blake3_fixpoint`
- `test_c3_threshold_count_window_concurrency`
- `test_c3_n3_trigger_gating_valid`
- `test_c3_construct_empty_no_receipt`
- `test_c3_sparql_ask_construct_delete_early_termination`

### Tier 4: Real-World Application Scenarios (5 cases)
- `test_s4_automated_quarantine_and_refusal`
- `test_s4_ledger_balance_enforcement_and_audit`
- `test_s4_state_machine_transition_control`
- `test_s4_access_control_policy_engine`
- `test_s4_materialized_view_cache_maintenance`

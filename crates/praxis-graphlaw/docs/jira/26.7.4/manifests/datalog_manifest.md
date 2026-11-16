# Datalog Conformance Pass-Rate Manifest

- **Dialect**: Datalog
- **Suite**: Roxi Datalog Conformance Suite (datalog_conformance.rs + submodules, datalog_negation.rs, datalog_challenger.rs, datalog_impossible.rs, datalog_stress.rs, datalog_stratification_fuzz.rs)
- **Total Tests**: 37
- **Passed**: 37
- **Failed**: 0
- **Pass Rate**: 100.0%

## Test Details

| Test Name | Status | Details |
|---|---|---|
| test_datalog_conformance | PASSED | Defined in tests/datalog_conformance.rs |
| test_safe_rule_accepted | PASSED | Defined in tests/datalog_conformance/safe_unsafe_rejection.rs |
| test_unsafe_unbound_head_var_rejected | PASSED | Defined in tests/datalog_conformance/safe_unsafe_rejection.rs |
| test_unsafe_unbound_negated_var_rejected | PASSED | Defined in tests/datalog_conformance/safe_unsafe_rejection.rs |
| test_mutual_recursion_fixpoint | PASSED | Defined in tests/datalog_conformance/mutual_recursion.rs |
| test_negation_across_stratum_boundary | PASSED | Defined in tests/datalog_conformance/negation_stratum.rs |
| test_negation_cycle_rejected | PASSED | Defined in tests/datalog_conformance/negation_cycle.rs |
| test_aggregate_count | PASSED | Defined in tests/datalog_conformance/aggregations.rs |
| test_aggregate_sum | PASSED | Defined in tests/datalog_conformance/aggregations.rs |
| test_aggregate_min_max | PASSED | Defined in tests/datalog_conformance/aggregations.rs |
| test_aggregate_avg | PASSED | Defined in tests/datalog_conformance/aggregations.rs |
| test_aggregate_recursive | PASSED | Defined in tests/datalog_conformance/aggregations.rs |
| test_aggregate_count_multi_variable_group_by | PASSED | Defined in tests/datalog_conformance/aggregations.rs |
| test_stratified_negation_basic | PASSED | Defined in tests/datalog_negation.rs |
| test_unstratifiable_rules_rejected | PASSED | Defined in tests/datalog_negation.rs |
| test_rule_safety_check_rejects_unbound_negated_var | PASSED | Defined in tests/datalog_negation.rs |
| test_fixpoint_terminates_on_recursive_ruleset | PASSED | Defined in tests/datalog_negation.rs |
| test_negation_empty_relations | PASSED | Defined in tests/datalog_negation.rs |
| test_negation_unbound_vars_rejected | PASSED | Defined in tests/datalog_negation.rs |
| test_empty_body_rule | PASSED | Defined in tests/datalog_negation.rs |
| test_long_unstratifiable_cycle_rejected | PASSED | Defined in tests/datalog_negation.rs |
| test_three_layer_stratification_chain | PASSED | Defined in tests/datalog_negation.rs |
| test_union_semantics_multiple_rules_same_head | PASSED | Defined in tests/datalog_negation.rs |
| test_empty_relations_negation | PASSED | Defined in tests/datalog_challenger.rs |
| test_empty_relations_aggregation | PASSED | Defined in tests/datalog_challenger.rs |
| test_unbound_aggregate_source_var | PASSED | Defined in tests/datalog_challenger.rs |
| test_boundary_numeric_inputs_aggregation | PASSED | Defined in tests/datalog_challenger.rs |
| test_aggregate_skips_non_numeric_source_values | PASSED | Defined in tests/datalog_challenger.rs |
| test_unsafe_negation_wholly_unbound_variable_rejected | PASSED | Defined in tests/datalog_impossible.rs |
| test_shortest_self_negation_cycle_rejected | PASSED | Defined in tests/datalog_impossible.rs |
| test_deep_stratification_chain_20_layers | PASSED | Defined in tests/datalog_stress.rs |
| test_deep_stratification_chain_with_far_cycle_rejected | PASSED | Defined in tests/datalog_stress.rs |
| test_large_scale_grouped_aggregation | PASSED | Defined in tests/datalog_stress.rs |
| test_diamond_reconvergence_rejected | PASSED | Defined in tests/datalog_stress.rs |
| test_multiple_disjoint_cycles_rejected | PASSED | Defined in tests/datalog_stress.rs |
| test_stratified_negation_over_aggregate_derived_predicate | PASSED | Defined in tests/datalog_stress.rs |
| nk_stratification_fuzz | PASSED | Defined in tests/datalog_stratification_fuzz.rs |

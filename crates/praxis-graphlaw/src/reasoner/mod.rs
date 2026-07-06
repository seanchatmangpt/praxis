use crate::triples::{Aggregate, AggregateFunction};
use crate::aggregation::{Accumulator, CountAccumulator, SumAccumulator, MinAccumulator, MaxAccumulator, AvgAccumulator, AccumulatorImpl};
use std::collections::HashMap;
use crate::{
    Binding, BodyLiteral, QueryEngine, Rule, SimpleQueryEngine,
    Triple, TripleIndex, TripleStore, VarOrTerm,
};
use log::debug;

mod log_implies;
mod log_collect_all_in;
mod log_not_includes;
mod log_includes;
mod log_for_all_in;
mod log_if_then_else_in;
mod log_conclusion;
mod substitution;

#[cfg(test)]
#[path = "reasoner_test.rs"]
mod reasoner_test;

pub struct Reasoner;

impl Reasoner {
    pub fn materialize(
        &mut self,
        triple_index: &mut TripleIndex,
        rules: &Vec<Rule>,
        strata: &Vec<usize>,
        aggregates: &std::collections::HashMap<Rule, crate::triples::Aggregate>,
    ) -> Vec<Triple> {
        let mut inferred = Vec::new();
        if rules.is_empty() {
            return inferred;
        }

        let max_stratum = *strata.iter().max().unwrap_or(&0);

        for s in 0..=max_stratum {

            let stratum_rules: Vec<&Rule> = rules
                .iter()
                .enumerate()
                // `.get(i).copied().unwrap_or(0)` rather than `strata[i]`: if
                // stratification failed validation (e.g. `strata` is shorter
                // than `rules`, or empty), fall back to treating the rule as
                // stratum 0 instead of panicking on an out-of-bounds index.
                .filter(|(i, _)| strata.get(*i).copied().unwrap_or(0) == s)
                .map(|(_, r)| r)
                .collect();

            if stratum_rules.is_empty() {
                continue;
            }

            let mut stratum_start_counter = None;
            let mut changed = true;

            while changed {
                changed = false;
                let next_start_counter = triple_index.len();

                for rule in &stratum_rules {
                    if rule.is_denial() {
                        // A denial/consistency-check rule (`=> false.`, e.g.
                        // SKOS's disjointness constraints) never derives a
                        // fact -- its sentinel head (`Rule::DENIAL_HEAD_MARKER`)
                        // exists purely so `Rule` doesn't need restructuring,
                        // and must never itself be asserted as if it were a
                        // real triple. Violations are checked separately via
                        // `TripleStore::check_denials` once materialize()
                        // has reached a fixpoint (see its doc comment).
                        continue;
                    }
                    if let Some(agg) = aggregates.get(rule) {
                        if let Some(bindings) = SimpleQueryEngine::query(
                            triple_index,
                            &rule.body,
                            stratum_start_counter,
                        ) {
                            let len = bindings.len();
                            if len > 0 {
                                let group_var_ids: Vec<usize> = agg
                                    .group_vars
                                    .iter()
                                    .map(|v| VarOrTerm::convert(v.clone()).to_encoded())
                                    .collect();
                                let source_var_id = VarOrTerm::convert(agg.source_var.clone()).to_encoded();
                                let target_var_id = VarOrTerm::convert(agg.target_var.clone()).to_encoded();

                                let mut groups: HashMap<Vec<usize>, Vec<usize>> = HashMap::new();
                                for c in 0..len {
                                    let mut group_key = Vec::new();
                                    for &var_id in &group_var_ids {
                                        if let Some(vals) = bindings.get(&var_id) {
                                            group_key.push(vals[c]);
                                        } else {
                                            group_key.push(0);
                                        }
                                    }
                                    if let Some(vals) = bindings.get(&source_var_id) {
                                        groups.entry(group_key).or_default().push(vals[c]);
                                    }
                                }

                                for (group_key, source_vals) in groups {
                                    let mut acc = match agg.function {
                                        AggregateFunction::Count => AccumulatorImpl::Count(CountAccumulator::default()),
                                        AggregateFunction::Sum => AccumulatorImpl::Sum(SumAccumulator::default()),
                                        AggregateFunction::Min => AccumulatorImpl::Min(MinAccumulator::default()),
                                        AggregateFunction::Max => AccumulatorImpl::Max(MaxAccumulator::default()),
                                        AggregateFunction::Avg => AccumulatorImpl::Avg(AvgAccumulator::default()),
                                    };
                                    for val in source_vals {
                                        acc.add(val);
                                    }
                                    let target_val = acc.get();

                                    let mut head = rule.head.clone();
                                    let substitute = |term: &mut VarOrTerm| {
                                        if term.is_var() {
                                            let var_id = term.to_encoded();
                                            if var_id == target_var_id {
                                                *term = VarOrTerm::new_encoded_term(target_val);
                                            } else {
                                                for (i, &gv_id) in group_var_ids.iter().enumerate() {
                                                    if var_id == gv_id {
                                                        *term = VarOrTerm::new_encoded_term(group_key[i]);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    };
                                    substitute(&mut head.s);
                                    substitute(&mut head.p);
                                    substitute(&mut head.o);
                                    if let Some(ref mut g) = head.g {
                                        substitute(g);
                                    }

                                    if Self::apply_new_triple(head, triple_index, &mut inferred) {
                                        changed = true;
                                    }
                                }
                            }
                        }
                    } else if !Self::find_log_implies_literals(rule).is_empty() {
                        let implies_indices = Self::find_log_implies_literals(rule);
                        // log:implies dynamic rule reification (see
                        // process_log_implies_rule doc comment).
                        for new_head in Self::process_log_implies_rule(rule, &implies_indices, triple_index) {
                            if Self::apply_new_triple(new_head, triple_index, &mut inferred) {
                                changed = true;
                            }
                        }
                    } else if let Some(collect_idx) = Self::find_log_collect_all_in_literal(rule) {
                        // log:collectAllIn (see process_log_collect_all_in_rule doc comment).
                        for new_head in Self::process_log_collect_all_in_rule(rule, collect_idx, triple_index) {
                            if Self::apply_new_triple(new_head, triple_index, &mut inferred) {
                                changed = true;
                            }
                        }
                    } else if let Some(not_includes_idx) = Self::find_log_not_includes_literal(rule) {
                        // log:notIncludes SNAF guard (see process_log_not_includes_rule doc comment).
                        for new_head in Self::process_log_not_includes_rule(rule, not_includes_idx, triple_index) {
                            if Self::apply_new_triple(new_head, triple_index, &mut inferred) {
                                changed = true;
                            }
                        }
                    } else if let Some(includes_idx) = Self::find_log_includes_literal(rule) {
                        // log:includes (positive counterpart of notIncludes; see process_log_includes_rule doc comment).
                        for new_head in Self::process_log_includes_rule(rule, includes_idx, triple_index) {
                            if Self::apply_new_triple(new_head, triple_index, &mut inferred) {
                                changed = true;
                            }
                        }
                    } else if let Some(for_all_idx) = Self::find_log_for_all_in_literal(rule) {
                        // log:forAllIn (see process_log_for_all_in_rule doc comment).
                        for new_head in Self::process_log_for_all_in_rule(rule, for_all_idx, triple_index) {
                            if Self::apply_new_triple(new_head, triple_index, &mut inferred) {
                                changed = true;
                            }
                        }
                    } else if let Some(if_then_else_idx) = Self::find_log_if_then_else_in_literal(rule) {
                        // log:ifThenElseIn (see process_log_if_then_else_in_rule doc comment).
                        for new_head in Self::process_log_if_then_else_in_rule(rule, if_then_else_idx, triple_index) {
                            if Self::apply_new_triple(new_head, triple_index, &mut inferred) {
                                changed = true;
                            }
                        }
                    } else if let Some(conclusion_idx) = Self::find_log_conclusion_literal(rule) {
                        // log:conclusion (see process_log_conclusion_rule doc comment).
                        for new_head in Self::process_log_conclusion_rule(rule, conclusion_idx, triple_index) {
                            if Self::apply_new_triple(new_head, triple_index, &mut inferred) {
                                changed = true;
                            }
                        }
                    } else {

                        if let Some(bindings) = SimpleQueryEngine::query(
                            triple_index,
                            &rule.body,
                            stratum_start_counter,
                        ) {

                            let new_heads = Self::substitute_head_with_bindings(&rule.head, &bindings);

                            for new_head in new_heads {
                                if Self::apply_new_triple(new_head, triple_index, &mut inferred) {
                                    changed = true;
                                }
                            }
                        }
                    }
                }

                stratum_start_counter = Some(next_start_counter);
            }
        }

        inferred
    }

    /// Add `head` to `triple_index` (and `inferred`) immediately if it's not
    /// already present, returning whether anything was actually added.
    /// Applying each rule's derivations to the live index right away --
    /// rather than batching all rules' output until the whole iteration
    /// finishes -- lets a later rule in the same stratum/iteration observe
    /// an earlier rule's same-iteration derivations. This is required for
    /// the real EYE `nixon_diamond` corpus case: Rule 2 ("Republicans are
    /// NonPacifist") must fire and be visible before Rule 1's
    /// `log:notIncludes { ?x a ex:NonPacifist }` guard is evaluated in the
    /// same pass, so the guard correctly suppresses the Pacifist derivation.
    /// Declaration order becomes the de facto rule-priority order this
    /// needs (matching upstream EYE's textual rule order).
    pub(crate) fn apply_new_triple(head: Triple, triple_index: &mut TripleIndex, inferred: &mut Vec<Triple>) -> bool {
        if triple_index.contains(&head) {
            return false;
        }
        debug!("Inferred: {:?}", TripleStore::decode_triple(&head));
        inferred.push(head.clone());
        triple_index.add(head);
        true
    }

    /// Evaluate a sequence of body literals (typically N3 builtins, e.g.
    /// `math:sum`/`math:greaterThan`) against a single already-built row of
    /// bindings, in order -- used by `process_log_collect_all_in_rule` to
    /// handle literals that depend on values (like the freshly-built list)
    /// which don't exist as queryable data. Returns `None` if any literal
    /// fails to hold for this row.
    pub(crate) fn eval_post_literals(
        post_body: &[BodyLiteral],
        start: Binding,
        triple_index: &TripleIndex,
    ) -> Option<Binding> {
        let mut bindings = start;
        for lit in post_body {
            if lit.negated {
                continue;
            }
            if let Some(kind) = crate::queryengine::builtins::classify(&lit.pattern.p) {
                bindings = crate::queryengine::builtins::evaluate(kind, &lit.pattern, &bindings)?;
            } else if let Some(cur) = triple_index.query(&lit.pattern, None) {
                bindings = bindings.join(&cur);
            } else {
                return None;
            }
        }
        Some(bindings)
    }

    /// Shared helper for `log:collectAllIn` / `log:notIncludes`: resolve
    /// `formula_id` to its quoted-graph triples, substitute any variables
    /// already bound in `row_binding` (e.g. an outer `?Subject` referenced
    /// inside the quoted formula) into each of them, and evaluate the result
    /// as a one-shot sub-query against the *current* `triple_index` -- the
    /// same "match this antecedent's triples against the live store" idea as
    /// the antecedent-matching step of `process_log_implies_rule`. Returns
    /// `None` when the formula is unknown or has zero solutions, `Some(bindings)`
    /// otherwise (possibly a zero-column `Binding` when the formula's triples
    /// are already fully ground and match).
    pub(crate) fn eval_embedded_formula_against_store(
        formula_id: usize,
        row_binding: &Binding,
        triple_index: &TripleIndex,
    ) -> Option<Binding> {
        let triples = VarOrTerm::formula_triples(formula_id)?;
        if triples.is_empty() {
            return Some(Binding::new());
        }
        let body: Vec<BodyLiteral> = triples
            .iter()
            .map(|p| BodyLiteral { negated: false, pattern: Self::substitute_single_row(p, row_binding) })
            .collect();
        SimpleQueryEngine::query(triple_index, &body, None)
    }

    pub fn infer_rule_heads(
        triple_index: &TripleIndex,
        counter: Option<usize>,
        matching_rules: Vec<Rule>,
    ) -> Vec<Triple> {
        let mut new_triples = Vec::new();
        for rule in matching_rules {
            if let Some(temp_bindings) = SimpleQueryEngine::query(triple_index, &rule.body, counter)
            {
                let new_heads = Reasoner::substitute_head_with_bindings(&rule.head, &temp_bindings);

                for new_head in new_heads {
                    new_triples.push(new_head);
                }
            }
        }
        new_triples
    }
}

pub(crate) fn query(query_triple: &Triple, match_triple: &Triple) -> Option<Binding> {
    let mut bindings = Binding::new();
    let Triple { s, p, o, g: _ } = match_triple;
    match &query_triple.s {
        VarOrTerm::Var(s_var) => bindings.add(&s_var.name, s.as_term().id()),
        VarOrTerm::Term(s_term) => {
            if s_term != s.as_term() {
                return None;
            }
        }
    }
    match &query_triple.p {
        VarOrTerm::Var(p_var) => bindings.add(&p_var.name, p.as_term().id()),
        VarOrTerm::Term(p_term) => {
            if p_term != p.as_term() {
                return None;
            }
        }
    }
    match &query_triple.o {
        VarOrTerm::Var(o_var) => bindings.add(&o_var.name, o.as_term().id()),
        VarOrTerm::Term(o_term) => {
            if o_term != o.as_term() {
                return None;
            }
        }
    }

    Some(bindings)
}

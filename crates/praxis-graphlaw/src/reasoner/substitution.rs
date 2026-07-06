use super::Reasoner;
use crate::{Binding, BodyLiteral, Rule, Triple, VarOrTerm};

impl Reasoner {
    /// Substitute variables in `pattern` using a *single-row* binding (each
    /// key maps to exactly one value). Unlike `substitute_triple_with_bindings`,
    /// this never skips substitution when `binding` has columns but a
    /// particular variable happens to be unbound -- it simply leaves that
    /// operand as the original variable, letting `is_ground` catch it.
    pub(crate) fn substitute_single_row(pattern: &Triple, binding: &Binding) -> Triple {
        let resolve = |var_id: usize| -> Option<usize> {
            binding.get(&var_id).and_then(|v| v.get(0)).copied()
        };
        let sub = |term: &VarOrTerm| VarOrTerm::substitute_deep(term, &resolve);
        Triple {
            s: sub(&pattern.s),
            p: sub(&pattern.p),
            o: sub(&pattern.o),
            g: pattern.g.as_ref().map(sub),
        }
    }

    /// A triple is safe to assert as a derived fact only once every position
    /// is a concrete term (no leftover unbound variables).
    pub(crate) fn is_ground(t: &Triple) -> bool {
        !t.s.is_var() && !t.p.is_var() && !t.o.is_var() && t.g.as_ref().map_or(true, |g| !g.is_var())
    }

    pub(crate) fn substitute_head_with_bindings(head: &Triple, binding: &Binding) -> Vec<Triple> {
        if binding.len() == 0 {
            return vec![head.clone()];
        }
        let mut new_heads = Vec::new();
        for result_counter in 0..binding.len() {
            // Recurses INSIDE list-term structures (see
            // `VarOrTerm::substitute_deep`'s doc comment) -- the previous
            // version substituted only top-level s/p/o variables, silently
            // leaving a list-valued head's internal variable members
            // unbound (verbatim-copying the rule's own pattern list rather
            // than the actually-bound value). Found via the real EYE
            // `good_cobbler` corpus case.
            let resolve = |var_id: usize| -> Option<usize> {
                binding.get(&var_id).and_then(|v| v.get(result_counter)).copied()
            };
            let sub = |term: &VarOrTerm| VarOrTerm::substitute_deep(term, &resolve);
            new_heads.push(Triple { s: sub(&head.s), p: sub(&head.p), o: sub(&head.o), g: None });
        }

        new_heads
    }

    #[allow(dead_code)]
    fn subsitute_binding(
        var_name: &usize,
        binding: &Binding,
        binding_counter: &usize,
    ) -> VarOrTerm {
        if let Some(s) = binding.get(var_name) {
            let iri = s.get(*binding_counter).unwrap().clone();
            VarOrTerm::new_encoded_term(iri)
        } else {
            VarOrTerm::new_encoded_var(var_name.clone())
        }
    }

    pub fn substitute_triple_with_bindings(head: &Triple, binding: &Binding) -> Vec<Triple> {
        let mut new_heads = Vec::new();
        for result_counter in 0..binding.len() {
            // Recurses INSIDE list-term structures, not just top-level s/p/o
            // positions -- the previous match-on-top-level-variant version
            // left a list-valued head's internal variable members
            // unsubstituted (a list term is never itself `is_var()`), so a
            // rule deriving e.g. `?X :is (:good ?Y)` asserted a verbatim
            // copy of its own still-variable-containing pattern list
            // instead of the actually-bound value. Found via the real EYE
            // `good_cobbler` corpus case.
            let resolve = |var_id: usize| -> Option<usize> {
                binding.get(&var_id).and_then(|v| v.get(result_counter)).copied()
            };
            let sub = |term: &VarOrTerm| VarOrTerm::substitute_deep(term, &resolve);
            new_heads.push(Triple { s: sub(&head.s), p: sub(&head.p), o: sub(&head.o), g: None });
        }

        new_heads
    }

    pub fn substitute_rule(matching_triple: &Triple, matching_rule: &Rule) -> Vec<Rule> {
        let mut results = Vec::new();
        for body_lit in matching_rule.body.iter() {
            if let Some(bindings) = super::query(&body_lit.pattern, matching_triple) {
                if bindings.len() == 0 {
                    return vec![matching_rule.clone()];
                }
                let new_body = Self::substitute_rule_body_with_binding(matching_rule, &bindings);
                let new_head =
                    Reasoner::substitute_triple_with_bindings(&matching_rule.head, &bindings)
                        .get(0)
                        .unwrap()
                        .clone();
                results.push(Rule {
                    body: new_body,
                    head: new_head,

                });
            }
        }
        results
    }

    pub fn substitute_rule_body_with_binding(
        matching_rule: &Rule,
        bindings: &Binding,
    ) -> Vec<BodyLiteral> {
        let mut new_body = Vec::new();
        for body_lit in matching_rule.body.iter() {
            let substituted =
                Reasoner::substitute_triple_with_bindings(&body_lit.pattern, &bindings);
            new_body.push(BodyLiteral {
                negated: body_lit.negated,
                pattern: substituted.get(0).unwrap().clone(),
            });
        }
        new_body
    }
}

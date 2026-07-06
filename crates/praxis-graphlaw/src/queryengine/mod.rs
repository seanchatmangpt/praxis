use crate::{Binding, BodyLiteral, Encoder, TripleIndex, VarOrTerm};

// Re-exported so existing call sites using `crate::queryengine::builtins::*`
// / `crate::queryengine::BuiltinKind` (tests, benches, js/, server/) keep
// compiling unchanged after the builtin registry moved to the top-level
// `crate::builtins` module.
pub use crate::builtins as builtins;
pub use crate::builtins::BuiltinKind;

pub trait QueryEngine {
    fn query(
        data: &TripleIndex,
        query_triples: &Vec<BodyLiteral>,
        triple_counter: Option<usize>,
    ) -> Option<Binding>;
}
pub struct SimpleQueryEngine;

impl QueryEngine for SimpleQueryEngine {
    fn query(
        data: &TripleIndex,
        query_triples: &Vec<BodyLiteral>,
        triple_counter: Option<usize>,
    ) -> Option<Binding> {
        let positive_lits: Vec<&BodyLiteral> = query_triples.iter().filter(|lit| !lit.negated).collect();
        let negated_lits: Vec<&BodyLiteral> = query_triples.iter().filter(|lit| lit.negated).collect();

        let mut bindings = Binding::new();
        let mut first = true;
        for lit in &positive_lits {
            if let Some(kind) = crate::builtins::classify(&lit.pattern.p) {
                // N3 built-in predicates (log:/math:/string:/list:) are evaluated
                // procedurally against the bindings accumulated *so far*, rather
                // than looked up in the data's TripleIndex (they never appear as
                // literal triples). They are never joined via `Binding::join`
                // either -- unlike a real query result set, a builtin's output
                // rows correspond 1:1 (or, for list:in, 1:N) to the rows already
                // in `bindings`, so it directly replaces `bindings`.
                match crate::builtins::evaluate(kind, &lit.pattern, &bindings) {
                    Some(new_bindings) => {
                        bindings = new_bindings;
                        first = false;
                    }
                    None => return None,
                }
            } else if let Some(current_bindings) = {
                crate::builtins::reject_if_unsupported_builtin(&lit.pattern.p);
                data.query(&lit.pattern, None)
            } {
                if first {
                    bindings = current_bindings;
                    first = false;
                } else {
                    let joined = bindings.join(&current_bindings);
                    if joined.len() == 0 && bindings.len() > 0 && current_bindings.len() > 0 {
                        return None;
                    }
                    bindings = joined;
                }
            } else {
                return None;
            }
        }

        if negated_lits.is_empty() {
            return Some(bindings);
        }

        let mut filtered_bindings = Binding::new();
        let num_rows = if positive_lits.is_empty() { 1 } else { bindings.len() };

        for c in 0..num_rows {
            let mut satisfied = true;
            for negated_lit in &negated_lits {
                let mut ground_pattern = negated_lit.pattern.clone();

                let mut substitute = |term: &mut VarOrTerm| {
                    if term.is_var() {
                        let var_id = term.to_encoded();
                        if let Some(vals) = bindings.get(&var_id) {
                            if let Some(&val) = vals.get(c) {
                                *term = VarOrTerm::new_term(Encoder::decode(&val).unwrap());
                            }
                        }
                    }
                };

                substitute(&mut ground_pattern.s);
                substitute(&mut ground_pattern.p);
                substitute(&mut ground_pattern.o);
                if let Some(ref mut g) = ground_pattern.g {
                    substitute(g);
                }

                crate::builtins::reject_if_unsupported_builtin(&ground_pattern.p);
                if data.query(&ground_pattern, triple_counter).is_some() {
                    satisfied = false;
                    break;
                }
            }

            if satisfied {
                if !positive_lits.is_empty() {
                    for (&var_id, vals) in bindings.iter() {
                        filtered_bindings.add(&var_id, vals[c]);
                    }
                }
            }
        }

        if filtered_bindings.len() > 0 || (positive_lits.is_empty() && filtered_bindings.len() == 0 && num_rows == 1) {
            Some(filtered_bindings)
        } else {
            None
        }
    }
}

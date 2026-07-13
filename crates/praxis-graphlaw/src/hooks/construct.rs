// SPARQL CONSTRUCT query evaluation and template instantiation

use crate::encoding::Encoder;
use crate::sparql::Binding;
use crate::term::Triple;
use crate::tripleindex::TripleIndex;
use serde::{Deserialize, Serialize};
use spargebra::term::{NamedNodePattern, TermPattern};
use spargebra::SparqlParser;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookReceipt {
    pub hook_name: String,
    pub delta_hash: String,
    pub idempotency_key: String,
    pub delta_quads: String,
}

fn collect_triple_patterns(
    gp: &spargebra::algebra::GraphPattern,
    out: &mut Vec<spargebra::term::TriplePattern>,
) {
    match gp {
        spargebra::algebra::GraphPattern::Bgp { patterns } => {
            out.extend(patterns.clone());
        }
        spargebra::algebra::GraphPattern::Join { left, right } => {
            collect_triple_patterns(left, out);
            collect_triple_patterns(right, out);
        }
        spargebra::algebra::GraphPattern::Distinct { inner }
        | spargebra::algebra::GraphPattern::Reduced { inner } => {
            collect_triple_patterns(inner, out);
        }
        spargebra::algebra::GraphPattern::Project { inner, .. } => {
            collect_triple_patterns(inner, out);
        }
        spargebra::algebra::GraphPattern::Filter { inner, .. } => {
            collect_triple_patterns(inner, out);
        }
        spargebra::algebra::GraphPattern::Group { inner, .. } => {
            collect_triple_patterns(inner, out);
        }
        spargebra::algebra::GraphPattern::Extend { inner, .. } => {
            collect_triple_patterns(inner, out);
        }
        spargebra::algebra::GraphPattern::LeftJoin { left, right, .. } => {
            collect_triple_patterns(left, out);
            collect_triple_patterns(right, out);
        }
        spargebra::algebra::GraphPattern::Union { left, right } => {
            collect_triple_patterns(left, out);
            collect_triple_patterns(right, out);
        }
        spargebra::algebra::GraphPattern::Minus { left, right } => {
            collect_triple_patterns(left, out);
            collect_triple_patterns(right, out);
        }
        _ => {}
    }
}

fn instantiate_term_pattern(tp: &TermPattern, bindings: &[Binding]) -> Option<String> {
    match tp {
        TermPattern::Variable(v) => {
            let var_name = v.as_str();
            bindings
                .iter()
                .find(|b| b.var == var_name)
                .map(|b| b.val.clone())
        }
        TermPattern::NamedNode(n) => Some(format!("<{}>", n.as_str())),
        TermPattern::BlankNode(b) => Some(format!("_:{}", b.as_str())),
        TermPattern::Literal(l) => {
            let mut s = format!("\"{}\"", l.value());
            if let Some(lang) = l.language() {
                s.push_str(&format!("@{}", lang));
            } else if l.datatype().as_str() != "http://www.w3.org/2001/XMLSchema#string" {
                s.push_str(&format!("^^<{}>", l.datatype().as_str()));
            }
            Some(s)
        }
        _ => None,
    }
}

fn instantiate_named_node_pattern(np: &NamedNodePattern, bindings: &[Binding]) -> Option<String> {
    match np {
        NamedNodePattern::Variable(v) => {
            let var_name = v.as_str();
            bindings
                .iter()
                .find(|b| b.var == var_name)
                .map(|b| b.val.clone())
        }
        NamedNodePattern::NamedNode(n) => Some(format!("<{}>", n.as_str())),
    }
}

fn instantiate_triple_pattern(
    tp: &spargebra::term::TriplePattern,
    row: &[Binding],
) -> Option<Triple> {
    let s_str = instantiate_term_pattern(&tp.subject, row)?;
    let p_str = instantiate_named_node_pattern(&tp.predicate, row)?;
    let o_str = instantiate_term_pattern(&tp.object, row)?;
    Some(Triple::from(s_str, p_str, o_str))
}

pub fn evaluate_construct(
    query_str: &str,
    triple_index: &TripleIndex,
) -> Result<(Vec<Triple>, Vec<Triple>), String> {
    let query = SparqlParser::new()
        .parse_query(query_str)
        .map_err(|e| format!("SPARQL parse error: {}", e))?;

    if let spargebra::Query::Construct {
        ref template,
        ref pattern,
        ..
    } = query
    {
        let mut additions = Vec::new();
        let mut deletions = Vec::new();

        let plan = crate::plan_query_or_refuse(&query, triple_index)?;
        let bindings: Vec<Vec<Binding>> =
            crate::sparql::evaluate_plan_and_debug(&plan, triple_index).collect();

        if !template.is_empty() {
            for row in bindings {
                for tp in template {
                    if let Some(triple) = instantiate_triple_pattern(tp, &row) {
                        additions.push(triple);
                    }
                }
            }
        } else {
            let mut patterns = Vec::new();
            collect_triple_patterns(&pattern, &mut patterns);
            for row in bindings {
                for tp in &patterns {
                    if let Some(triple) = instantiate_triple_pattern(tp, &row) {
                        deletions.push(triple);
                    }
                }
            }
        }

        Ok((additions, deletions))
    } else {
        Err("Query is not a CONSTRUCT query".to_string())
    }
}

pub fn serialize_delta_quad(
    hook_iri: &str,
    triple: &Triple,
    is_addition: bool,
    lines: &mut Vec<String>,
) {
    let s = clean_decoded_term(&Encoder::decode(&triple.s.to_encoded()).unwrap_or_default());
    let p = clean_decoded_term(&Encoder::decode(&triple.p.to_encoded()).unwrap_or_default());
    let o = clean_decoded_term(&Encoder::decode(&triple.o.to_encoded()).unwrap_or_default());

    let triple_str = format!("{} {} {}", s, p, o);
    let hash = blake3::hash(triple_str.as_bytes()).to_hex().to_string();

    let bn_id = if is_addition {
        format!("_:add_{}", hash)
    } else {
        format!("_:del_{}", hash)
    };

    let pred = if is_addition {
        "<http://seanchatmangpt.github.io/praxis/kh#addQuad>"
    } else {
        "<http://seanchatmangpt.github.io/praxis/kh#deleteQuad>"
    };

    let wrap_iri = |x: &str| {
        if x.starts_with('<') || x.starts_with('"') || x.starts_with('_') {
            x.to_string()
        } else {
            format!("<{}>", x)
        }
    };

    let s_wrapped = wrap_iri(&s);
    let p_wrapped = wrap_iri(&p);
    let o_wrapped = if o.starts_with('"') {
        o.clone()
    } else {
        wrap_iri(&o)
    };
    let hook_wrapped = wrap_iri(hook_iri);

    lines.push(format!(
        "{} <http://seanchatmangpt.github.io/praxis/kh#subject> {} .",
        bn_id, s_wrapped
    ));
    lines.push(format!(
        "{} <http://seanchatmangpt.github.io/praxis/kh#predicate> {} .",
        bn_id, p_wrapped
    ));
    lines.push(format!(
        "{} <http://seanchatmangpt.github.io/praxis/kh#object> {} .",
        bn_id, o_wrapped
    ));
    lines.push(format!("{} {} {} .", hook_wrapped, pred, bn_id));
}

fn clean_decoded_term(s: &str) -> String {
    s.trim().to_string()
}

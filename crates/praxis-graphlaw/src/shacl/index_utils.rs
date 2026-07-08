use super::Vocab;
/// Utility functions for accessing and querying the triple index
///
/// These functions provide low-level access to triple data organized by
/// different indices (SPO, POS, etc.).
use crate::encoding::Encoder;
use crate::tripleindex::TripleIndex;
use crate::triples::Term;
use std::collections::HashSet;

/// Check if a shape is deactivated (sh:deactivated true)
pub fn is_shape_deactivated(shapes: &TripleIndex, shape_node: usize, vocab: &Vocab) -> bool {
    let deactivated_vals = get_objects(shapes, shape_node, vocab.sh_deactivated);
    for v in deactivated_vals {
        if let Some(lex) = crate::shacl::values::get_lexical_form(v) {
            if lex == "true" || lex == "1" {
                return true;
            }
        }
    }
    false
}

/// Check if a term is a blank node
pub fn is_blank_node(id: usize) -> bool {
    matches!(Encoder::decode_to_term(id), Some(Term::BlankNode(_)))
}

/// Check if a term is an IRI
pub fn is_iri(id: usize) -> bool {
    matches!(Encoder::decode_to_term(id), Some(Term::Iri(_)))
}

/// Check if a term is a literal
pub fn is_literal(id: usize) -> bool {
    matches!(Encoder::decode_to_term(id), Some(Term::Literal(_)))
}

/// Check if triple (s, p, o) exists in index
fn contains_triple(index: &TripleIndex, s: usize, p: usize, o: usize) -> bool {
    if let Some(preds) = index.spo.get(&s) {
        if let Some(objs) = preds.get(&p) {
            return objs.iter().any(|(obj_val, _, _)| *obj_val == o);
        }
    }
    false
}

/// Get all objects for subject and predicate (S-P-O indexed)
pub fn get_objects(index: &TripleIndex, subject: usize, predicate: usize) -> Vec<usize> {
    let mut result = Vec::new();
    if let Some(preds) = index.spo.get(&subject) {
        if let Some(objs) = preds.get(&predicate) {
            for (o, _, _) in objs {
                result.push(*o);
            }
        }
    }
    result
}

/// Get all subjects for predicate and object (P-O-S indexed)
pub(crate) fn get_subjects(index: &TripleIndex, predicate: usize, object: usize) -> Vec<usize> {
    let mut result = Vec::new();
    if let Some(objs) = index.pos.get(&predicate) {
        if let Some(subjs) = objs.get(&object) {
            for (s, _, _) in subjs {
                result.push(*s);
            }
        }
    }
    result
}

/// Get all (subject, object) pairs for a predicate (P-O-S indexed)
pub(crate) fn get_triples_by_predicate(
    index: &TripleIndex,
    predicate: usize,
) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    if let Some(objs) = index.pos.get(&predicate) {
        for (o, subjs) in objs {
            for (s, _, _) in subjs {
                result.push((*s, *o));
            }
        }
    }
    result
}

/// Parse an RDF list into a Vec of node IDs
pub(crate) fn get_rdf_list(shapes: &TripleIndex, list_node: usize) -> Vec<usize> {
    let mut result = Vec::new();
    let mut current = list_node;
    let nil_node = Encoder::get("<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>");
    let first_pred =
        Encoder::get("<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>").unwrap_or(0);
    let rest_pred = Encoder::get("<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>").unwrap_or(0);
    let mut visited = HashSet::new();
    while Some(current) != nil_node && visited.insert(current) {
        let firsts = get_objects(shapes, current, first_pred);
        if firsts.is_empty() {
            break;
        }
        result.push(firsts[0]);
        let rests = get_objects(shapes, current, rest_pred);
        if rests.is_empty() {
            break;
        }
        current = rests[0];
    }
    result
}

/// Get the datatype of a term, or infer it from the term type
pub fn get_datatype(term_id: usize) -> Option<usize> {
    let term = Encoder::decode_to_term(term_id)?;
    if let Term::Literal(lit) = term {
        if let Some(dt) = lit.datatype {
            Some(dt)
        } else if lit.lang.is_some() {
            Some(Encoder::add(
                "<http://www.w3.org/1999/02/22-rdf-syntax-ns#langString>".to_string(),
            ))
        } else {
            Some(Encoder::add(
                "<http://www.w3.org/2001/XMLSchema#string>".to_string(),
            ))
        }
    } else {
        None
    }
}

/// Per the SHACL spec's DatatypeConstraintComponent: value nodes must not
/// only be *declared* with the expected datatype, but for datatypes that are
/// "recognized" (i.e. the small set of XSD datatypes with well-defined
/// lexical spaces we validate here), the lexical form must also be
/// well-formed for that datatype. Datatypes outside this small set are not
/// lexically validated -- a literal declared with such a datatype conforms
/// based on the declared datatype IRI alone, per spec's fallback rule for
/// unrecognized datatypes.
pub fn is_lexically_valid_for_datatype(lexical: &str, datatype_iri: &str) -> bool {
    let t = lexical.trim();
    match datatype_iri {
        "http://www.w3.org/2001/XMLSchema#integer"
        | "http://www.w3.org/2001/XMLSchema#int"
        | "http://www.w3.org/2001/XMLSchema#long"
        | "http://www.w3.org/2001/XMLSchema#short"
        | "http://www.w3.org/2001/XMLSchema#byte"
        | "http://www.w3.org/2001/XMLSchema#nonNegativeInteger"
        | "http://www.w3.org/2001/XMLSchema#positiveInteger"
        | "http://www.w3.org/2001/XMLSchema#nonPositiveInteger"
        | "http://www.w3.org/2001/XMLSchema#negativeInteger"
        | "http://www.w3.org/2001/XMLSchema#unsignedLong"
        | "http://www.w3.org/2001/XMLSchema#unsignedInt"
        | "http://www.w3.org/2001/XMLSchema#unsignedShort"
        | "http://www.w3.org/2001/XMLSchema#unsignedByte" => {
            let digits = t.strip_prefix(['+', '-']).unwrap_or(t);
            !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
        }
        "http://www.w3.org/2001/XMLSchema#decimal" => {
            let body = t.strip_prefix(['+', '-']).unwrap_or(t);
            !body.is_empty()
                && body.chars().all(|c| c.is_ascii_digit() || c == '.')
                && body.matches('.').count() <= 1
                && body.chars().any(|c| c.is_ascii_digit())
        }
        "http://www.w3.org/2001/XMLSchema#double" | "http://www.w3.org/2001/XMLSchema#float" => {
            t == "INF" || t == "-INF" || t == "NaN" || t.parse::<f64>().is_ok()
        }
        "http://www.w3.org/2001/XMLSchema#boolean" => {
            matches!(t, "true" | "false" | "1" | "0")
        }
        _ => true,
    }
}

/// Check if a literal has the expected datatype and is lexically valid
pub(crate) fn check_datatype(x: usize, expected_dt: usize) -> bool {
    match get_datatype(x) {
        Some(dt) if dt == expected_dt => {
            match (
                crate::shacl::values::get_lexical_form(x),
                crate::shacl::values::get_lexical_form(expected_dt),
            ) {
                (Some(lex), Some(dt_iri)) => is_lexically_valid_for_datatype(&lex, &dt_iri),
                _ => true,
            }
        }
        _ => false,
    }
}

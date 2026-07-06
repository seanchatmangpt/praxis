use crate::encoding::Encoder;
use crate::tripleindex::TripleIndex;
use crate::triples::{Term, TermImpl, Triple, VarOrTerm};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Vocabulary
// ---------------------------------------------------------------------------

pub struct Vocab {
    pub rdf_type: usize,
    pub rdf_first: usize,
    pub rdf_rest: usize,
    pub rdf_nil: usize,
    pub rdfs_class: usize,
    pub rdfs_subclass_of: usize,
    pub sh_node_shape: usize,
    pub sh_property_shape: usize,
    pub sh_target_class: usize,
    pub sh_target_node: usize,
    pub sh_target_subjects_of: usize,
    pub sh_target_objects_of: usize,
    pub sh_property: usize,
    pub sh_node: usize,
    pub sh_path: usize,
    pub sh_min_count: usize,
    pub sh_max_count: usize,
    pub sh_datatype: usize,
    pub sh_class: usize,
    pub sh_pattern: usize,
    pub sh_flags: usize,
    pub sh_in: usize,
    pub sh_and: usize,
    pub sh_or: usize,
    pub sh_not: usize,
    pub sh_xone: usize,
    pub sh_node_kind: usize,
    pub sh_has_value: usize,
    pub sh_min_length: usize,
    pub sh_max_length: usize,
    pub sh_min_exclusive: usize,
    pub sh_min_inclusive: usize,
    pub sh_max_exclusive: usize,
    pub sh_max_inclusive: usize,
    pub sh_language_in: usize,
    pub sh_unique_lang: usize,
    pub sh_equals: usize,
    pub sh_disjoint: usize,
    pub sh_less_than: usize,
    pub sh_less_than_or_equals: usize,
    pub sh_qualified_value_shape: usize,
    pub sh_qualified_min_count: usize,
    pub sh_qualified_max_count: usize,
    pub sh_closed: usize,
    pub sh_ignored_properties: usize,
    pub sh_deactivated: usize,
    pub sh_message: usize,
    pub sh_severity: usize,
    pub sh_violation: usize,
    pub sh_warning: usize,
    pub sh_info: usize,
    pub sh_validation_report: usize,
    pub sh_conforms: usize,
    pub sh_result: usize,
    pub sh_validation_result: usize,
    pub sh_focus_node: usize,
    pub sh_result_path: usize,
    pub sh_value: usize,
    pub sh_result_message: usize,
    pub sh_source_constraint_component: usize,
    pub sh_source_shape: usize,
    pub sh_result_severity: usize,
    pub sh_alternative_path: usize,
    pub sh_inverse_path: usize,
    pub sh_zero_or_more_path: usize,
    pub sh_one_or_more_path: usize,
    pub sh_zero_or_one_path: usize,

    // SPARQL-based constraints (sh:sparql / SPARQLConstraintComponent) and targets
    pub sh_sparql: usize,
    pub sh_select: usize,
    pub sh_ask: usize,
    pub sh_prefixes: usize,
    pub sh_target: usize,
    pub sh_sparql_target: usize,

    // sh:nodeKind values
    pub sh_iri: usize,
    pub sh_blank_node: usize,
    pub sh_literal: usize,
    pub sh_blank_node_or_iri: usize,
    pub sh_blank_node_or_literal: usize,
    pub sh_iri_or_literal: usize,

    // Constraint Components
    pub sh_min_count_constraint_component: usize,
    pub sh_max_count_constraint_component: usize,
    pub sh_datatype_constraint_component: usize,
    pub sh_class_constraint_component: usize,
    pub sh_pattern_constraint_component: usize,
    pub sh_in_constraint_component: usize,
    pub sh_and_constraint_component: usize,
    pub sh_or_constraint_component: usize,
    pub sh_not_constraint_component: usize,
    pub sh_xone_constraint_component: usize,
    pub sh_node_kind_constraint_component: usize,
    pub sh_has_value_constraint_component: usize,
    pub sh_min_length_constraint_component: usize,
    pub sh_max_length_constraint_component: usize,
    pub sh_min_exclusive_constraint_component: usize,
    pub sh_min_inclusive_constraint_component: usize,
    pub sh_max_exclusive_constraint_component: usize,
    pub sh_max_inclusive_constraint_component: usize,
    pub sh_language_in_constraint_component: usize,
    pub sh_unique_lang_constraint_component: usize,
    pub sh_equals_constraint_component: usize,
    pub sh_disjoint_constraint_component: usize,
    pub sh_less_than_constraint_component: usize,
    pub sh_less_than_or_equals_constraint_component: usize,
    pub sh_qualified_value_shape_constraint_component: usize,
    pub sh_closed_constraint_component: usize,
    pub sh_node_constraint_component: usize,
    pub sh_sparql_constraint_component: usize,
}

impl Vocab {
    pub fn new() -> Self {
        Vocab {
            rdf_type: Encoder::add("<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string()),
            rdf_first: Encoder::add("<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>".to_string()),
            rdf_rest: Encoder::add("<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>".to_string()),
            rdf_nil: Encoder::add("<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>".to_string()),
            rdfs_class: Encoder::add("<http://www.w3.org/2000/01/rdf-schema#Class>".to_string()),
            rdfs_subclass_of: Encoder::add("<http://www.w3.org/2000/01/rdf-schema#subClassOf>".to_string()),
            sh_node_shape: Encoder::add("<http://www.w3.org/ns/shacl#NodeShape>".to_string()),
            sh_property_shape: Encoder::add("<http://www.w3.org/ns/shacl#PropertyShape>".to_string()),
            sh_target_class: Encoder::add("<http://www.w3.org/ns/shacl#targetClass>".to_string()),
            sh_target_node: Encoder::add("<http://www.w3.org/ns/shacl#targetNode>".to_string()),
            sh_target_subjects_of: Encoder::add("<http://www.w3.org/ns/shacl#targetSubjectsOf>".to_string()),
            sh_target_objects_of: Encoder::add("<http://www.w3.org/ns/shacl#targetObjectsOf>".to_string()),
            sh_property: Encoder::add("<http://www.w3.org/ns/shacl#property>".to_string()),
            sh_node: Encoder::add("<http://www.w3.org/ns/shacl#node>".to_string()),
            sh_path: Encoder::add("<http://www.w3.org/ns/shacl#path>".to_string()),
            sh_min_count: Encoder::add("<http://www.w3.org/ns/shacl#minCount>".to_string()),
            sh_max_count: Encoder::add("<http://www.w3.org/ns/shacl#maxCount>".to_string()),
            sh_datatype: Encoder::add("<http://www.w3.org/ns/shacl#datatype>".to_string()),
            sh_class: Encoder::add("<http://www.w3.org/ns/shacl#class>".to_string()),
            sh_pattern: Encoder::add("<http://www.w3.org/ns/shacl#pattern>".to_string()),
            sh_flags: Encoder::add("<http://www.w3.org/ns/shacl#flags>".to_string()),
            sh_in: Encoder::add("<http://www.w3.org/ns/shacl#in>".to_string()),
            sh_and: Encoder::add("<http://www.w3.org/ns/shacl#and>".to_string()),
            sh_or: Encoder::add("<http://www.w3.org/ns/shacl#or>".to_string()),
            sh_not: Encoder::add("<http://www.w3.org/ns/shacl#not>".to_string()),
            sh_xone: Encoder::add("<http://www.w3.org/ns/shacl#xone>".to_string()),
            sh_node_kind: Encoder::add("<http://www.w3.org/ns/shacl#nodeKind>".to_string()),
            sh_has_value: Encoder::add("<http://www.w3.org/ns/shacl#hasValue>".to_string()),
            sh_min_length: Encoder::add("<http://www.w3.org/ns/shacl#minLength>".to_string()),
            sh_max_length: Encoder::add("<http://www.w3.org/ns/shacl#maxLength>".to_string()),
            sh_min_exclusive: Encoder::add("<http://www.w3.org/ns/shacl#minExclusive>".to_string()),
            sh_min_inclusive: Encoder::add("<http://www.w3.org/ns/shacl#minInclusive>".to_string()),
            sh_max_exclusive: Encoder::add("<http://www.w3.org/ns/shacl#maxExclusive>".to_string()),
            sh_max_inclusive: Encoder::add("<http://www.w3.org/ns/shacl#maxInclusive>".to_string()),
            sh_language_in: Encoder::add("<http://www.w3.org/ns/shacl#languageIn>".to_string()),
            sh_unique_lang: Encoder::add("<http://www.w3.org/ns/shacl#uniqueLang>".to_string()),
            sh_equals: Encoder::add("<http://www.w3.org/ns/shacl#equals>".to_string()),
            sh_disjoint: Encoder::add("<http://www.w3.org/ns/shacl#disjoint>".to_string()),
            sh_less_than: Encoder::add("<http://www.w3.org/ns/shacl#lessThan>".to_string()),
            sh_less_than_or_equals: Encoder::add("<http://www.w3.org/ns/shacl#lessThanOrEquals>".to_string()),
            sh_qualified_value_shape: Encoder::add("<http://www.w3.org/ns/shacl#qualifiedValueShape>".to_string()),
            sh_qualified_min_count: Encoder::add("<http://www.w3.org/ns/shacl#qualifiedMinCount>".to_string()),
            sh_qualified_max_count: Encoder::add("<http://www.w3.org/ns/shacl#qualifiedMaxCount>".to_string()),
            sh_closed: Encoder::add("<http://www.w3.org/ns/shacl#closed>".to_string()),
            sh_ignored_properties: Encoder::add("<http://www.w3.org/ns/shacl#ignoredProperties>".to_string()),
            sh_deactivated: Encoder::add("<http://www.w3.org/ns/shacl#deactivated>".to_string()),
            sh_message: Encoder::add("<http://www.w3.org/ns/shacl#message>".to_string()),
            sh_severity: Encoder::add("<http://www.w3.org/ns/shacl#severity>".to_string()),
            sh_violation: Encoder::add("<http://www.w3.org/ns/shacl#Violation>".to_string()),
            sh_warning: Encoder::add("<http://www.w3.org/ns/shacl#Warning>".to_string()),
            sh_info: Encoder::add("<http://www.w3.org/ns/shacl#Info>".to_string()),
            sh_validation_report: Encoder::add("<http://www.w3.org/ns/shacl#ValidationReport>".to_string()),
            sh_conforms: Encoder::add("<http://www.w3.org/ns/shacl#conforms>".to_string()),
            sh_result: Encoder::add("<http://www.w3.org/ns/shacl#result>".to_string()),
            sh_validation_result: Encoder::add("<http://www.w3.org/ns/shacl#ValidationResult>".to_string()),
            sh_focus_node: Encoder::add("<http://www.w3.org/ns/shacl#focusNode>".to_string()),
            sh_result_path: Encoder::add("<http://www.w3.org/ns/shacl#resultPath>".to_string()),
            sh_value: Encoder::add("<http://www.w3.org/ns/shacl#value>".to_string()),
            sh_result_message: Encoder::add("<http://www.w3.org/ns/shacl#resultMessage>".to_string()),
            sh_source_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#sourceConstraintComponent>".to_string()),
            sh_source_shape: Encoder::add("<http://www.w3.org/ns/shacl#sourceShape>".to_string()),
            sh_result_severity: Encoder::add("<http://www.w3.org/ns/shacl#resultSeverity>".to_string()),
            sh_alternative_path: Encoder::add("<http://www.w3.org/ns/shacl#alternativePath>".to_string()),
            sh_inverse_path: Encoder::add("<http://www.w3.org/ns/shacl#inversePath>".to_string()),
            sh_zero_or_more_path: Encoder::add("<http://www.w3.org/ns/shacl#zeroOrMorePath>".to_string()),
            sh_one_or_more_path: Encoder::add("<http://www.w3.org/ns/shacl#oneOrMorePath>".to_string()),
            sh_zero_or_one_path: Encoder::add("<http://www.w3.org/ns/shacl#zeroOrOnePath>".to_string()),

            // SPARQL-based constraints (sh:sparql / SPARQLConstraintComponent) and targets
            sh_sparql: Encoder::add("<http://www.w3.org/ns/shacl#sparql>".to_string()),
            sh_select: Encoder::add("<http://www.w3.org/ns/shacl#select>".to_string()),
            sh_ask: Encoder::add("<http://www.w3.org/ns/shacl#ask>".to_string()),
            sh_prefixes: Encoder::add("<http://www.w3.org/ns/shacl#prefixes>".to_string()),
            sh_target: Encoder::add("<http://www.w3.org/ns/shacl#target>".to_string()),
            sh_sparql_target: Encoder::add("<http://www.w3.org/ns/shacl#SPARQLTarget>".to_string()),

            // sh:nodeKind values
            sh_iri: Encoder::add("<http://www.w3.org/ns/shacl#IRI>".to_string()),
            sh_blank_node: Encoder::add("<http://www.w3.org/ns/shacl#BlankNode>".to_string()),
            sh_literal: Encoder::add("<http://www.w3.org/ns/shacl#Literal>".to_string()),
            sh_blank_node_or_iri: Encoder::add("<http://www.w3.org/ns/shacl#BlankNodeOrIRI>".to_string()),
            sh_blank_node_or_literal: Encoder::add("<http://www.w3.org/ns/shacl#BlankNodeOrLiteral>".to_string()),
            sh_iri_or_literal: Encoder::add("<http://www.w3.org/ns/shacl#IRIOrLiteral>".to_string()),

            // Constraint Components
            sh_min_count_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MinCountConstraintComponent>".to_string()),
            sh_max_count_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MaxCountConstraintComponent>".to_string()),
            sh_datatype_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#DatatypeConstraintComponent>".to_string()),
            sh_class_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#ClassConstraintComponent>".to_string()),
            sh_pattern_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#PatternConstraintComponent>".to_string()),
            sh_in_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#InConstraintComponent>".to_string()),
            sh_and_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#AndConstraintComponent>".to_string()),
            sh_or_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#OrConstraintComponent>".to_string()),
            sh_not_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#NotConstraintComponent>".to_string()),
            sh_xone_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#XoneConstraintComponent>".to_string()),
            sh_node_kind_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#NodeKindConstraintComponent>".to_string()),
            sh_has_value_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#HasValueConstraintComponent>".to_string()),
            sh_min_length_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MinLengthConstraintComponent>".to_string()),
            sh_max_length_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MaxLengthConstraintComponent>".to_string()),
            sh_min_exclusive_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MinExclusiveConstraintComponent>".to_string()),
            sh_min_inclusive_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MinInclusiveConstraintComponent>".to_string()),
            sh_max_exclusive_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MaxExclusiveConstraintComponent>".to_string()),
            sh_max_inclusive_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MaxInclusiveConstraintComponent>".to_string()),
            sh_language_in_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#LanguageInConstraintComponent>".to_string()),
            sh_unique_lang_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#UniqueLangConstraintComponent>".to_string()),
            sh_equals_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#EqualsConstraintComponent>".to_string()),
            sh_disjoint_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#DisjointConstraintComponent>".to_string()),
            sh_less_than_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#LessThanConstraintComponent>".to_string()),
            sh_less_than_or_equals_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#LessThanOrEqualsConstraintComponent>".to_string()),
            sh_qualified_value_shape_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#QualifiedValueShapeConstraintComponent>".to_string()),
            sh_closed_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#ClosedConstraintComponent>".to_string()),
            sh_node_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#NodeConstraintComponent>".to_string()),
            sh_sparql_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#SPARQLConstraintComponent>".to_string()),
        }
    }
}

impl Default for Vocab {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ShapesGraph
// ---------------------------------------------------------------------------

pub struct ShapesGraph {
    pub raw_index: TripleIndex,
}

impl ShapesGraph {
    pub fn parse(shapes_str: &str) -> Result<Self, String> {
        let triples = crate::parser::Parser::parse_triples(shapes_str, crate::parser::Syntax::Turtle)?;
        let mut raw_index = TripleIndex::new();
        for triple in triples {
            raw_index.add(triple);
        }
        Ok(ShapesGraph { raw_index })
    }
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidationResult {
    pub focus_node: Term,
    pub result_path: Option<Term>,
    pub value: Option<Term>,
    pub source_constraint_component: Term,
    pub source_shape: Term,
    pub severity: Term,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidationReport {
    pub conforms: bool,
    pub results: Vec<ValidationResult>,
}

impl ValidationReport {
    /// Serialise the report as SHACL-conformant RDF triples.
    pub fn to_triples(&self) -> Vec<Triple> {
        let mut triples = Vec::new();

        let report_id = format!("report_{}", self.results.len());
        let report_term = VarOrTerm::new_blank_node(report_id);

        let rdf_type = VarOrTerm::convert("http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string());
        let sh_validation_report = VarOrTerm::convert("http://www.w3.org/ns/shacl#ValidationReport".to_string());
        let sh_conforms = VarOrTerm::convert("http://www.w3.org/ns/shacl#conforms".to_string());
        let sh_result = VarOrTerm::convert("http://www.w3.org/ns/shacl#result".to_string());
        let sh_focus_node = VarOrTerm::convert("http://www.w3.org/ns/shacl#focusNode".to_string());
        let sh_result_path = VarOrTerm::convert("http://www.w3.org/ns/shacl#resultPath".to_string());
        let sh_value = VarOrTerm::convert("http://www.w3.org/ns/shacl#value".to_string());
        let sh_source_constraint_component = VarOrTerm::convert("http://www.w3.org/ns/shacl#sourceConstraintComponent".to_string());
        let sh_source_shape = VarOrTerm::convert("http://www.w3.org/ns/shacl#sourceShape".to_string());
        let sh_result_severity = VarOrTerm::convert("http://www.w3.org/ns/shacl#resultSeverity".to_string());
        let sh_result_message = VarOrTerm::convert("http://www.w3.org/ns/shacl#resultMessage".to_string());
        let sh_validation_result = VarOrTerm::convert("http://www.w3.org/ns/shacl#ValidationResult".to_string());

        triples.push(Triple {
            s: report_term.clone(),
            p: rdf_type.clone(),
            o: sh_validation_report,
            g: None,
        });

        let conforms_str = if self.conforms { "true" } else { "false" };
        let conforms_literal = VarOrTerm::new_literal(
            conforms_str.to_string(),
            Some("<http://www.w3.org/2001/XMLSchema#boolean>".to_string()),
            None,
        );
        triples.push(Triple {
            s: report_term.clone(),
            p: sh_conforms,
            o: conforms_literal,
            g: None,
        });

        for (i, res) in self.results.iter().enumerate() {
            let res_term = VarOrTerm::new_blank_node(format!("result_{}", i));

            triples.push(Triple {
                s: report_term.clone(),
                p: sh_result.clone(),
                o: res_term.clone(),
                g: None,
            });

            triples.push(Triple {
                s: res_term.clone(),
                p: rdf_type.clone(),
                o: sh_validation_result.clone(),
                g: None,
            });

            triples.push(Triple {
                s: res_term.clone(),
                p: sh_focus_node.clone(),
                o: VarOrTerm::Term(res.focus_node.clone()),
                g: None,
            });

            triples.push(Triple {
                s: res_term.clone(),
                p: sh_source_constraint_component.clone(),
                o: VarOrTerm::Term(res.source_constraint_component.clone()),
                g: None,
            });

            triples.push(Triple {
                s: res_term.clone(),
                p: sh_source_shape.clone(),
                o: VarOrTerm::Term(res.source_shape.clone()),
                g: None,
            });

            triples.push(Triple {
                s: res_term.clone(),
                p: sh_result_severity.clone(),
                o: VarOrTerm::Term(res.severity.clone()),
                g: None,
            });

            if let Some(ref path) = res.result_path {
                triples.push(Triple {
                    s: res_term.clone(),
                    p: sh_result_path.clone(),
                    o: VarOrTerm::Term(path.clone()),
                    g: None,
                });
            }

            if let Some(ref val) = res.value {
                triples.push(Triple {
                    s: res_term.clone(),
                    p: sh_value.clone(),
                    o: VarOrTerm::Term(val.clone()),
                    g: None,
                });
            }

            if let Some(ref msg) = res.message {
                let msg_literal = VarOrTerm::new_literal(msg.clone(), None, None);
                triples.push(Triple {
                    s: res_term.clone(),
                    p: sh_result_message.clone(),
                    o: msg_literal,
                    g: None,
                });
            }
        }

        triples
    }
}

// ---------------------------------------------------------------------------
// Validator entry point
// ---------------------------------------------------------------------------

pub struct Validator;

impl Validator {
    pub fn validate(data: &TripleIndex, shapes: &ShapesGraph) -> ValidationReport {
        let vocab = Vocab::new();
        let shapes_index = &shapes.raw_index;

        let mut shape_nodes = HashSet::new();

        // Subjects of rdf:type sh:NodeShape or sh:PropertyShape
        for s in get_subjects(shapes_index, vocab.rdf_type, vocab.sh_node_shape) {
            shape_nodes.insert(s);
        }
        for s in get_subjects(shapes_index, vocab.rdf_type, vocab.sh_property_shape) {
            shape_nodes.insert(s);
        }

        // Subjects of any target declaration
        for t in [
            vocab.sh_target_class,
            vocab.sh_target_node,
            vocab.sh_target_subjects_of,
            vocab.sh_target_objects_of,
            vocab.sh_target,
        ] {
            if let Some(objs) = shapes_index.pos.get(&t) {
                for subjs in objs.values() {
                    for (s, _, _) in subjs {
                        shape_nodes.insert(*s);
                    }
                }
            }
        }

        // Objects of sh:property are property shapes
        for (_, ps) in get_triples_by_predicate(shapes_index, vocab.sh_property) {
            shape_nodes.insert(ps);
        }

        let mut results = Vec::new();

        for shape in shape_nodes {
            // Skip deactivated shapes
            if is_shape_deactivated(shapes_index, shape, &vocab) {
                continue;
            }
            let focus_nodes = get_focus_nodes(data, shapes_index, shape, &vocab);
            for focus in focus_nodes {
                let mut visited = HashSet::new();
                validate_shape(data, shapes_index, &vocab, focus, shape, &mut results, &mut visited);
            }
        }

        // Per SHACL Core (1.0), sh:conforms is false whenever ANY validation
        // result exists, regardless of severity -- Info and Warning results
        // count too, not just Violation. This is confirmed by the real W3C
        // data-shapes test suite (e.g. misc/severity-001.ttl: a shape with
        // sh:severity sh:Warning still expects sh:conforms "false"). SHACL
        // 1.2 introduces an opt-in sh:conformanceDisallows to narrow this,
        // but that's a distinct, newer mechanism this validator doesn't
        // implement -- the unconditional default (all severities disallow
        // conformance) is what SHACL Core actually specifies and what every
        // vendored conformance case expects.
        let conforms = results.is_empty();
        ValidationReport { conforms, results }
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn is_shape_deactivated(shapes: &TripleIndex, shape_node: usize, vocab: &Vocab) -> bool {
    let deactivated_vals = get_objects(shapes, shape_node, vocab.sh_deactivated);
    for v in deactivated_vals {
        if let Some(lex) = get_lexical_form(v) {
            if lex == "true" || lex == "1" {
                return true;
            }
        }
    }
    false
}

pub(crate) fn is_blank_node(id: usize) -> bool {
    matches!(Encoder::decode_to_term(id), Some(Term::BlankNode(_)))
}

pub(crate) fn is_iri(id: usize) -> bool {
    matches!(Encoder::decode_to_term(id), Some(Term::Iri(_)))
}

pub(crate) fn is_literal(id: usize) -> bool {
    matches!(Encoder::decode_to_term(id), Some(Term::Literal(_)))
}

fn contains_triple(index: &TripleIndex, s: usize, p: usize, o: usize) -> bool {
    if let Some(preds) = index.spo.get(&s) {
        if let Some(objs) = preds.get(&p) {
            return objs.iter().any(|(obj_val, _, _)| *obj_val == o);
        }
    }
    false
}

pub(crate) fn get_objects(index: &TripleIndex, subject: usize, predicate: usize) -> Vec<usize> {
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

fn get_subjects(index: &TripleIndex, predicate: usize, object: usize) -> Vec<usize> {
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

fn get_triples_by_predicate(index: &TripleIndex, predicate: usize) -> Vec<(usize, usize)> {
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

fn get_rdf_list(shapes: &TripleIndex, list_node: usize) -> Vec<usize> {
    let mut result = Vec::new();
    let mut current = list_node;
    let nil_node = Encoder::get("<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>");
    let first_pred = Encoder::get("<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>").unwrap_or(0);
    let rest_pred = Encoder::get("<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>").unwrap_or(0);
    let mut visited = HashSet::new();
    while Some(current) != nil_node && visited.insert(current) {
        let firsts = get_objects(shapes, current, first_pred);
        if firsts.is_empty() { break; }
        result.push(firsts[0]);
        let rests = get_objects(shapes, current, rest_pred);
        if rests.is_empty() { break; }
        current = rests[0];
    }
    result
}

pub(crate) fn get_datatype(term_id: usize) -> Option<usize> {
    let term = Encoder::decode_to_term(term_id)?;
    if let Term::Literal(lit) = term {
        if let Some(dt) = lit.datatype {
            Some(dt)
        } else if lit.lang.is_some() {
            Some(Encoder::add("<http://www.w3.org/1999/02/22-rdf-syntax-ns#langString>".to_string()))
        } else {
            Some(Encoder::add("<http://www.w3.org/2001/XMLSchema#string>".to_string()))
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
pub(crate) fn is_lexically_valid_for_datatype(lexical: &str, datatype_iri: &str) -> bool {
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

fn check_datatype(x: usize, expected_dt: usize) -> bool {
    match get_datatype(x) {
        Some(dt) if dt == expected_dt => {
            match (get_lexical_form(x), get_lexical_form(expected_dt)) {
                (Some(lex), Some(dt_iri)) => is_lexically_valid_for_datatype(&lex, &dt_iri),
                _ => true,
            }
        }
        _ => false,
    }
}

fn has_class(data: &TripleIndex, x: usize, class: usize, rdfs_subclass_of: usize, rdf_type: usize) -> bool {
    let types = get_objects(data, x, rdf_type);
    for t in types {
        let mut visited = HashSet::new();
        let mut queue = vec![t];
        while !queue.is_empty() {
            let current = queue.remove(0);
            if current == class { return true; }
            if visited.insert(current) {
                for p in get_objects(data, current, rdfs_subclass_of) {
                    queue.push(p);
                }
            }
        }
    }
    false
}

pub(crate) fn get_lexical_form(x: usize) -> Option<String> {
    let term = Encoder::decode_to_term(x)?;
    match term {
        Term::Iri(_) => {
            let s = Encoder::decode(&x)?;
            if s.starts_with('<') && s.ends_with('>') {
                Some(s[1..s.len()-1].to_string())
            } else {
                Some(s)
            }
        }
        Term::Literal(lit) => Encoder::decode(&lit.value),
        Term::BlankNode(_) => {
            let s = Encoder::decode(&x)?;
            Some(if s.starts_with("_:") { s[2..].to_string() } else { s })
        }
    }
}

/// Like `get_lexical_form`, but for constraints where the SHACL spec means
/// an actual RDF string representation (IRIs and literals), NOT a blank
/// node's internal label -- which is an implementation detail, not a value
/// the graph author wrote, and must never be treated as satisfying a
/// string-shaped constraint. Used by sh:minLength/sh:maxLength: a real W3C
/// test case (minLength-001) targets exactly this, expecting a blank node
/// (reached via sh:targetClass) to always violate sh:minLength/maxLength.
fn get_string_representation(x: usize) -> Option<String> {
    match Encoder::decode_to_term(x)? {
        Term::BlankNode(_) => None,
        _ => get_lexical_form(x),
    }
}

pub(crate) fn get_lang_tag(x: usize) -> Option<String> {
    if let Some(Term::Literal(lit)) = Encoder::decode_to_term(x) {
        lit.lang.as_ref().and_then(|l| Encoder::decode(l))
    } else {
        None
    }
}

/// Return the numeric value of a literal as f64, handling xsd:integer, xsd:decimal, xsd:double.
fn get_numeric_value(term_id: usize) -> Option<f64> {
    let lex = get_lexical_form(term_id)?;
    lex.trim().parse::<f64>().ok()
}

/// Parse an `xsd:dateTime` lexical form (`YYYY-MM-DDTHH:MM:SS[.fff][(Z|+HH:MM|-HH:MM)]`)
/// into (seconds-since-a-fixed-epoch, has_explicit_timezone). The epoch choice is
/// arbitrary (proleptic Gregorian day count is not needed here) since only
/// relative comparisons between two parsed values are ever made.
fn parse_datetime(lex: &str) -> Option<(i64, bool)> {
    let lex = lex.trim();
    let t_pos = lex.find('T')?;
    let (date_part, rest) = lex.split_at(t_pos);
    let rest = &rest[1..]; // skip 'T'

    let date_fields: Vec<&str> = date_part.splitn(3, '-').collect();
    // A leading '-' (BCE year) would produce an empty first field; not handled here.
    if date_fields.len() != 3 || date_fields.iter().any(|f| f.is_empty()) {
        return None;
    }
    let year: i64 = date_fields[0].parse().ok()?;
    let month: i64 = date_fields[1].parse().ok()?;
    let day: i64 = date_fields[2].parse().ok()?;

    // Split off an explicit timezone: 'Z', or a trailing "+HH:MM"/"-HH:MM"
    // (search from index 1 to skip a leading '-' that might belong to... time
    // fields never start with '-', so any '+' or '-' found is the tz offset).
    let (time_part, tz_offset_secs, has_tz) = if let Some(z_pos) = rest.find('Z') {
        (&rest[..z_pos], 0i64, true)
    } else if let Some(sign_pos) = rest.rfind(['+', '-']) {
        let (t, off) = rest.split_at(sign_pos);
        let off_fields: Vec<&str> = off[1..].splitn(2, ':').collect();
        let off_h: i64 = off_fields.first()?.parse().ok()?;
        let off_m: i64 = off_fields.get(1).map(|s| s.parse().ok()).flatten().unwrap_or(0);
        let sign = if off.starts_with('-') { -1 } else { 1 };
        (t, sign * (off_h * 3600 + off_m * 60), true)
    } else {
        (rest, 0i64, false)
    };

    let time_fields: Vec<&str> = time_part.splitn(3, ':').collect();
    if time_fields.len() != 3 {
        return None;
    }
    let hour: i64 = time_fields[0].parse().ok()?;
    let minute: i64 = time_fields[1].parse().ok()?;
    let second: f64 = time_fields[2].parse().ok()?;

    // Days since an arbitrary fixed epoch via a standard civil-to-days
    // algorithm (Howard Hinnant's `days_from_civil`), valid for the
    // proleptic Gregorian calendar.
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;

    let total_secs = days * 86400 + hour * 3600 + minute * 60 + (second as i64) - tz_offset_secs;
    Some((total_secs, has_tz))
}

/// Compare two terms numerically (for sh:lessThan etc.), falling back to
/// `xsd:dateTime`-aware comparison when numeric parsing fails. Returns
/// `None` if not comparable at all, OR if comparing a timezone-qualified
/// dateTime against a timezone-less one -- per XSD's dateTime partial order,
/// such pairs have an "indeterminate" ordering (the comparison result
/// depends on which of the 14-hour timezone extremes is assumed), which
/// this crate treats the same as any other non-comparable pair: a
/// range-facet result of `None` (constraint not satisfied). This matches
/// the real W3C SHACL test suite's expectation (minInclusive-002/003).
pub(crate) fn compare_numeric(a: usize, b: usize) -> Option<std::cmp::Ordering> {
    if let (Some(av), Some(bv)) = (get_numeric_value(a), get_numeric_value(b)) {
        return av.partial_cmp(&bv);
    }
    let (a_lex, b_lex) = (get_lexical_form(a)?, get_lexical_form(b)?);
    let (a_secs, a_tz) = parse_datetime(&a_lex)?;
    let (b_secs, b_tz) = parse_datetime(&b_lex)?;
    if a_tz != b_tz {
        return None;
    }
    a_secs.partial_cmp(&b_secs)
}

pub(crate) fn match_regex(pattern: &str, text: &str, flags: &str) -> bool {
    let mut builder = regex::RegexBuilder::new(pattern);
    for c in flags.chars() {
        match c {
            'i' => { builder.case_insensitive(true); }
            'm' => { builder.multi_line(true); }
            's' => { builder.dot_matches_new_line(true); }
            'x' => { builder.ignore_whitespace(true); }
            _ => {}
        }
    }
    builder.build().map_or(false, |re| re.is_match(text))
}

/// Parse an xsd:integer literal as i64 (allowing negative values).
pub(crate) fn get_integer_value(term_id: usize) -> Option<i64> {
    let lex = get_lexical_form(term_id)?;
    lex.trim().parse::<i64>().ok()
}

pub(crate) fn decode_to_term(id: usize) -> Term {
    Encoder::decode_to_term(id).unwrap_or_else(|| Term::Iri(TermImpl { iri: id }))
}

fn get_severity(shapes: &TripleIndex, shape_node: usize, vocab: &Vocab) -> usize {
    let sevs = get_objects(shapes, shape_node, vocab.sh_severity);
    if sevs.is_empty() { vocab.sh_violation } else { sevs[0] }
}

/// Returns each sh:message value on `shape_node` paired with its language tag
/// (if any).
fn get_shape_messages(shapes: &TripleIndex, shape_node: usize, vocab: &Vocab) -> Vec<(Option<String>, String)> {
    get_objects(shapes, shape_node, vocab.sh_message)
        .into_iter()
        .filter_map(|m| get_lexical_form(m).map(|text| (get_lang_tag(m), text)))
        .collect()
}

/// Select a single message to report when a shape has zero or more sh:message
/// values, possibly in different languages.
///
/// NOTE: there is no locale parameter threaded through the validator (the
/// public API has no way for a caller to request a specific language), so this
/// implements a fixed, spec-informed default policy rather than full RFC 4647
/// language-range negotiation: prefer a plain (no language tag) literal first
/// -- matching the SHACL/RDF convention that a language-less sh:message is the
/// implementation-neutral fallback -- then prefer one tagged "en", and
/// otherwise fall back to whichever value came first.
fn pick_preferred_message(messages: &[(Option<String>, String)]) -> Option<String> {
    if let Some((_, text)) = messages.iter().find(|(lang, _)| lang.is_none()) {
        return Some(text.clone());
    }
    if let Some((_, text)) = messages.iter().find(|(lang, _)| {
        lang.as_deref()
            .map(|l| l.eq_ignore_ascii_case("en") || l.to_lowercase().starts_with("en-"))
            .unwrap_or(false)
    }) {
        return Some(text.clone());
    }
    messages.first().map(|(_, text)| text.clone())
}

fn make_result(
    focus_node: usize,
    result_path: Option<usize>,
    value: Option<usize>,
    component: usize,
    shape_node: usize,
    severity: usize,
    message: Option<String>,
) -> ValidationResult {
    ValidationResult {
        focus_node: decode_to_term(focus_node),
        result_path: result_path.map(decode_to_term),
        value: value.map(decode_to_term),
        source_constraint_component: decode_to_term(component),
        source_shape: decode_to_term(shape_node),
        severity: decode_to_term(severity),
        message,
    }
}

// ---------------------------------------------------------------------------
// SPARQL-based constraints and targets (sh:sparql, sh:target/SPARQLTarget)
// ---------------------------------------------------------------------------

/// Render a term in the textual SPARQL syntax needed to splice it into a
/// query string in place of `$this` (or similar substitution points).
fn term_to_sparql_syntax(id: usize) -> String {
    match decode_to_term(id) {
        Term::Iri(_) => {
            let lex = get_lexical_form(id).unwrap_or_default();
            format!("<{}>", lex)
        }
        Term::BlankNode(_) => {
            let lex = get_lexical_form(id).unwrap_or_default();
            format!("_:{}", lex)
        }
        Term::Literal(lit) => {
            let lex = get_lexical_form(id).unwrap_or_default();
            let escaped = lex.replace('\\', "\\\\").replace('"', "\\\"");
            if let Some(lang_id) = lit.lang {
                let lang = Encoder::decode(&lang_id).unwrap_or_default();
                format!("\"{}\"@{}", escaped, lang)
            } else if let Some(dt_id) = lit.datatype {
                let dt_lex = get_lexical_form(dt_id).unwrap_or_default();
                format!("\"{}\"^^<{}>", escaped, dt_lex)
            } else {
                format!("\"{}\"", escaped)
            }
        }
    }
}

/// Parse and evaluate a raw SPARQL query string against `data` using the
/// engine in `crate::sparql`. Returns one Vec<EncodedBinding> per solution
/// row (for ASK queries this is the underlying WHERE-pattern's solutions —
/// non-empty means the ASK would return true).
fn evaluate_sparql_text(
    data: &TripleIndex,
    query_text: &str,
) -> Result<Vec<Vec<crate::tripleindex::EncodedBinding>>, String> {
    let query = spargebra::Query::parse(query_text, None).map_err(|e| e.to_string())?;
    let plan = crate::sparql::eval_query(&query, data);
    Ok(crate::sparql::evaluate_plan(&plan, data).collect())
}

/// Rewrite `$this` occurrences in a SHACL SPARQL constraint's query text
/// into a real, valid pre-bound SPARQL variable: replace every `$this`
/// with `?this`, then inject `BIND(<iri> AS ?this)` immediately after the
/// query's first `{` (the opening of its WHERE block).
///
/// This replaces an earlier approach that textually substituted `$this`
/// with the focus node's raw syntactic form (e.g. `<http://example.org/x>`)
/// everywhere in the query text -- which breaks for the single most common
/// real-world SHACL SPARQL idiom, `SELECT $this WHERE { ... }`: substituting
/// `$this` there produces `SELECT <http://example.org/x> WHERE { ... }`,
/// which is not valid SPARQL (a bare IRI is not a legal SELECT projection),
/// so the query failed to parse and the caller's `if let Ok(rows) = ...`
/// silently swallowed the error -- reporting zero violations instead of
/// evaluating the constraint at all. Found via SHACL sh:sparql interaction
/// testing (a real bug in roxi's own code, not a delegated third-party
/// crate).
fn substitute_this_as_bound_variable(query_text: &str, this_syntax: &str) -> String {
    let with_var = query_text.replace("$this", "?this");
    if let Some(brace_pos) = with_var.find('{') {
        let (before, after) = with_var.split_at(brace_pos + 1);
        format!("{before} BIND({this_syntax} AS ?this) {after}")
    } else {
        with_var
    }
}

/// Evaluate a single sh:sparql constraint (a blank/IRI node carrying
/// sh:select or sh:ask) against `this_node`, appending any violations to
/// `results`. See `substitute_this_as_bound_variable` for how `$this` is
/// handled.
fn validate_sparql_constraint(
    data: &TripleIndex,
    shapes: &TripleIndex,
    vocab: &Vocab,
    this_node: usize,
    shape_node: usize,
    sparql_node: usize,
    default_severity: usize,
    default_msg: &Option<String>,
    results: &mut Vec<ValidationResult>,
) {
    // A sh:sparql constraint node may itself carry sh:severity / sh:message
    // overriding the enclosing shape's.
    let local_sevs = get_objects(shapes, sparql_node, vocab.sh_severity);
    let severity = local_sevs.first().copied().unwrap_or(default_severity);
    let local_messages = get_shape_messages(shapes, sparql_node, vocab);
    let local_msg = pick_preferred_message(&local_messages).or_else(|| default_msg.clone());

    let this_syntax = term_to_sparql_syntax(this_node);

    if let Some(select_lit) = get_objects(shapes, sparql_node, vocab.sh_select).first().copied() {
        let Some(query_text) = get_lexical_form(select_lit) else { return; };
        let substituted = substitute_this_as_bound_variable(&query_text, &this_syntax);
        if let Ok(rows) = evaluate_sparql_text(data, &substituted) {
            // Minimal SPARQLConstraintComponent semantics: any solution row is
            // a violation. When the SELECT projects ?message/?path/?value,
            // honour those bindings per spec; otherwise fall back to the
            // shape's own message and the focus node as the value.
            for row in rows {
                let mut msg = local_msg.clone();
                let mut path = None;
                let mut value = Some(this_node);
                for b in &row {
                    let var_name = Encoder::decode(&b.var).unwrap_or_default();
                    match var_name.trim_start_matches('?') {
                        "message" => {
                            if let Some(m) = get_lexical_form(b.val) {
                                msg = Some(m);
                            }
                        }
                        "path" => path = Some(b.val),
                        "value" => value = Some(b.val),
                        _ => {}
                    }
                }
                results.push(make_result(
                    this_node,
                    path,
                    value,
                    vocab.sh_sparql_constraint_component,
                    shape_node,
                    severity,
                    msg,
                ));
            }
        }
        // A malformed/unparseable query is silently skipped (documented
        // limitation): the SHACL SPARQL constraint components spec assumes a
        // pre-validated shapes graph, and surfacing a parse error as a Rust
        // panic/error would abort validation for the whole report.
    } else if let Some(ask_lit) = get_objects(shapes, sparql_node, vocab.sh_ask).first().copied() {
        if let Some(query_text) = get_lexical_form(ask_lit) {
            let substituted = substitute_this_as_bound_variable(&query_text, &this_syntax);
            if let Ok(rows) = evaluate_sparql_text(data, &substituted) {
                if rows.is_empty() {
                    results.push(make_result(
                        this_node,
                        None,
                        Some(this_node),
                        vocab.sh_sparql_constraint_component,
                        shape_node,
                        severity,
                        local_msg,
                    ));
                }
            }
        }
    }
}

fn get_focus_nodes(
    data: &TripleIndex,
    shapes: &TripleIndex,
    shape_id: usize,
    vocab: &Vocab,
) -> HashSet<usize> {
    let mut focus_nodes = HashSet::new();

    // 1. sh:targetClass
    for class in get_objects(shapes, shape_id, vocab.sh_target_class) {
        for inst in get_subjects(data, vocab.rdf_type, class) {
            focus_nodes.insert(inst);
        }
    }

    // 2. sh:targetNode
    for node in get_objects(shapes, shape_id, vocab.sh_target_node) {
        focus_nodes.insert(node);
    }

    // 3. sh:targetSubjectsOf
    for pred in get_objects(shapes, shape_id, vocab.sh_target_subjects_of) {
        if let Some(objs) = data.pos.get(&pred) {
            for subjs in objs.values() {
                for (s, _, _) in subjs { focus_nodes.insert(*s); }
            }
        }
    }

    // 4. sh:targetObjectsOf
    for pred in get_objects(shapes, shape_id, vocab.sh_target_objects_of) {
        if let Some(objs) = data.pos.get(&pred) {
            for o in objs.keys() { focus_nodes.insert(*o); }
        }
    }

    // 5. sh:target [ a sh:SPARQLTarget ; sh:select "SELECT ?this WHERE {...}" ]
    let sparql_targets = get_objects(shapes, shape_id, vocab.sh_target);
    for target_node in &sparql_targets {
        let is_sparql_target = contains_triple(shapes, *target_node, vocab.rdf_type, vocab.sh_sparql_target);
        if !is_sparql_target { continue; }
        let Some(select_lit) = get_objects(shapes, *target_node, vocab.sh_select).first().copied() else { continue; };
        let Some(query_text) = get_lexical_form(select_lit) else { continue; };
        if let Ok(rows) = evaluate_sparql_text(data, &query_text) {
            for row in rows {
                for b in &row {
                    let var_name = Encoder::decode(&b.var).unwrap_or_default();
                    if var_name.trim_start_matches('?') == "this" {
                        focus_nodes.insert(b.val);
                    }
                }
            }
        }
    }

    // Implicit class target: only when shape is also declared as a class AND has no other targets
    if !is_blank_node(shape_id) {
        let has_explicit_target =
            !get_objects(shapes, shape_id, vocab.sh_target_class).is_empty()
            || !get_objects(shapes, shape_id, vocab.sh_target_node).is_empty()
            || !get_objects(shapes, shape_id, vocab.sh_target_subjects_of).is_empty()
            || !get_objects(shapes, shape_id, vocab.sh_target_objects_of).is_empty()
            || !sparql_targets.is_empty();

        if !has_explicit_target {
            let is_class =
                contains_triple(shapes, shape_id, vocab.rdf_type, vocab.rdfs_class)
                || contains_triple(data, shape_id, vocab.rdf_type, vocab.rdfs_class);
            if is_class {
                for inst in get_subjects(data, vocab.rdf_type, shape_id) {
                    focus_nodes.insert(inst);
                }
            }
        }
    }

    focus_nodes
}

fn eval_path(
    data: &TripleIndex,
    shapes: &TripleIndex,
    focus_node: usize,
    path_node: usize,
) -> Vec<usize> {
    // Sequence path (RDF list of steps)
    let rdf_first = Encoder::get("<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>").unwrap_or(0);
    if !get_objects(shapes, path_node, rdf_first).is_empty() {
        let steps = get_rdf_list(shapes, path_node);
        let mut current = vec![focus_node];
        for step in steps {
            let mut next = Vec::new();
            for node in current {
                next.extend(eval_path(data, shapes, node, step));
            }
            current = next;
            if current.is_empty() { break; }
        }
        return current;
    }

    // sh:inversePath
    let sh_inverse_path = Encoder::get("<http://www.w3.org/ns/shacl#inversePath>").unwrap_or(0);
    let inverse = get_objects(shapes, path_node, sh_inverse_path);
    if !inverse.is_empty() {
        let mut results = Vec::new();
        for inv in inverse {
            results.extend(get_subjects(data, inv, focus_node));
        }
        return results;
    }

    // sh:alternativePath
    let sh_alternative_path = Encoder::get("<http://www.w3.org/ns/shacl#alternativePath>").unwrap_or(0);
    let alternative = get_objects(shapes, path_node, sh_alternative_path);
    if !alternative.is_empty() {
        let mut results = Vec::new();
        for alt_list in alternative {
            for p in get_rdf_list(shapes, alt_list) {
                results.extend(eval_path(data, shapes, focus_node, p));
            }
        }
        return results;
    }

    // sh:zeroOrMorePath
    let sh_zero_or_more = Encoder::get("<http://www.w3.org/ns/shacl#zeroOrMorePath>").unwrap_or(0);
    let zero_or_more = get_objects(shapes, path_node, sh_zero_or_more);
    if !zero_or_more.is_empty() {
        let mut results = vec![focus_node];
        let mut visited = HashSet::new();
        visited.insert(focus_node);
        let mut queue = vec![focus_node];
        while !queue.is_empty() {
            let node = queue.remove(0);
            for target in &zero_or_more {
                for n in eval_path(data, shapes, node, *target) {
                    if visited.insert(n) { results.push(n); queue.push(n); }
                }
            }
        }
        return results;
    }

    // sh:oneOrMorePath
    let sh_one_or_more = Encoder::get("<http://www.w3.org/ns/shacl#oneOrMorePath>").unwrap_or(0);
    let one_or_more = get_objects(shapes, path_node, sh_one_or_more);
    if !one_or_more.is_empty() {
        let mut results = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = vec![focus_node];
        while !queue.is_empty() {
            let node = queue.remove(0);
            for target in &one_or_more {
                for n in eval_path(data, shapes, node, *target) {
                    if visited.insert(n) { results.push(n); queue.push(n); }
                }
            }
        }
        return results;
    }

    // sh:zeroOrOnePath
    let sh_zero_or_one = Encoder::get("<http://www.w3.org/ns/shacl#zeroOrOnePath>").unwrap_or(0);
    let zero_or_one = get_objects(shapes, path_node, sh_zero_or_one);
    if !zero_or_one.is_empty() {
        let mut results = vec![focus_node];
        for target in zero_or_one {
            for n in eval_path(data, shapes, focus_node, target) {
                if !results.contains(&n) { results.push(n); }
            }
        }
        return results;
    }

    // Direct IRI property
    get_objects(data, focus_node, path_node)
}

/// Returns true if the node conforms to the given shape (no violations).
fn conforms_to_shape(
    data: &TripleIndex,
    shapes: &TripleIndex,
    vocab: &Vocab,
    node: usize,
    shape_node: usize,
    visited: &mut HashSet<(usize, usize)>,
) -> bool {
    let mut temp = Vec::new();
    validate_shape(data, shapes, vocab, node, shape_node, &mut temp, visited);
    temp.is_empty()
}

fn validate_shape(
    data: &TripleIndex,
    shapes: &TripleIndex,
    vocab: &Vocab,
    focus_node: usize,
    shape_node: usize,
    results: &mut Vec<ValidationResult>,
    visited: &mut HashSet<(usize, usize)>,
) {
    // Cycle detection: if we are already validating (focus, shape), skip.
    if !visited.insert((focus_node, shape_node)) {
        return;
    }

    // Deactivated check
    if is_shape_deactivated(shapes, shape_node, vocab) {
        visited.remove(&(focus_node, shape_node));
        return;
    }

    let severity = get_severity(shapes, shape_node, vocab);
    let messages = get_shape_messages(shapes, shape_node, vocab);
    let default_msg = pick_preferred_message(&messages);

    // -----------------------------------------------------------------------
    // Self-as-property-shape: a PropertyShape can declare its own targets
    // directly (sh:targetNode/sh:targetClass/etc alongside sh:path), rather
    // than only ever being reached indirectly via another shape's
    // sh:property. `validate_property_shape` normally only runs for shapes
    // reached through `sh:property` below; a shape validated here that
    // itself has sh:path would otherwise have its own path/minCount/maxCount
    // constraints silently skipped. Real bug found vendoring the W3C
    // core/path test suite (path-sequence-001 and siblings): these are all
    // top-level PropertyShapes with their own sh:targetNode.
    // -----------------------------------------------------------------------
    if !get_objects(shapes, shape_node, vocab.sh_path).is_empty() {
        validate_property_shape(data, shapes, vocab, focus_node, shape_node, results, visited);
    }

    // -----------------------------------------------------------------------
    // Node-level constraints
    // -----------------------------------------------------------------------

    // sh:nodeKind
    for nk in get_objects(shapes, shape_node, vocab.sh_node_kind) {
        let ok = if nk == vocab.sh_iri {
            is_iri(focus_node)
        } else if nk == vocab.sh_blank_node {
            is_blank_node(focus_node)
        } else if nk == vocab.sh_literal {
            is_literal(focus_node)
        } else if nk == vocab.sh_blank_node_or_iri {
            is_blank_node(focus_node) || is_iri(focus_node)
        } else if nk == vocab.sh_blank_node_or_literal {
            is_blank_node(focus_node) || is_literal(focus_node)
        } else if nk == vocab.sh_iri_or_literal {
            is_iri(focus_node) || is_literal(focus_node)
        } else {
            true
        };
        if !ok {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_node_kind_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:datatype (node-level)
    for dt in get_objects(shapes, shape_node, vocab.sh_datatype) {
        if !check_datatype(focus_node, dt) {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_datatype_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:class (node-level)
    for class in get_objects(shapes, shape_node, vocab.sh_class) {
        if !has_class(data, focus_node, class, vocab.rdfs_subclass_of, vocab.rdf_type) {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_class_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:hasValue (node-level)
    for hv in get_objects(shapes, shape_node, vocab.sh_has_value) {
        if focus_node != hv {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_has_value_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:in (node-level)
    for in_list in get_objects(shapes, shape_node, vocab.sh_in) {
        let allowed = get_rdf_list(shapes, in_list);
        if !allowed.contains(&focus_node) {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_in_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:pattern + sh:flags (node-level)
    let flags_str = get_objects(shapes, shape_node, vocab.sh_flags)
        .first()
        .and_then(|f| get_lexical_form(*f))
        .unwrap_or_default();
    for pat in get_objects(shapes, shape_node, vocab.sh_pattern) {
        if let Some(pat_str) = get_lexical_form(pat) {
            let focus_str = get_lexical_form(focus_node).unwrap_or_default();
            if !match_regex(&pat_str, &focus_str, &flags_str) {
                results.push(make_result(focus_node, None, Some(focus_node),
                    vocab.sh_pattern_constraint_component, shape_node, severity, default_msg.clone()));
            }
        }
    }

    // sh:minLength / sh:maxLength (node-level). Per the SHACL spec, a value
    // with no string representation at all (e.g. a blank node) is always
    // considered to violate both constraints, not silently skipped -- a
    // real W3C test case (minLength-001) targets exactly this: a blank node
    // reached via sh:targetClass must produce a MinLengthConstraintComponent
    // violation even though it has no lexical form.
    let char_len = get_string_representation(focus_node).map(|lex| lex.chars().count() as i64);
    for ml in get_objects(shapes, shape_node, vocab.sh_min_length) {
        if let Some(v) = get_integer_value(ml) {
            if char_len.map_or(true, |len| len < v) {
                results.push(make_result(focus_node, None, Some(focus_node),
                    vocab.sh_min_length_constraint_component, shape_node, severity, default_msg.clone()));
            }
        }
    }
    for ml in get_objects(shapes, shape_node, vocab.sh_max_length) {
        if let Some(v) = get_integer_value(ml) {
            if char_len.map_or(true, |len| len > v) {
                results.push(make_result(focus_node, None, Some(focus_node),
                    vocab.sh_max_length_constraint_component, shape_node, severity, default_msg.clone()));
            }
        }
    }

    // sh:minExclusive / sh:minInclusive / sh:maxExclusive / sh:maxInclusive (node-level)
    for bound in get_objects(shapes, shape_node, vocab.sh_min_exclusive) {
        match compare_numeric(focus_node, bound) {
            Some(std::cmp::Ordering::Greater) => {}
            _ => { results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_min_exclusive_constraint_component, shape_node, severity, default_msg.clone())); }
        }
    }
    for bound in get_objects(shapes, shape_node, vocab.sh_min_inclusive) {
        match compare_numeric(focus_node, bound) {
            Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal) => {}
            _ => { results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_min_inclusive_constraint_component, shape_node, severity, default_msg.clone())); }
        }
    }
    for bound in get_objects(shapes, shape_node, vocab.sh_max_exclusive) {
        match compare_numeric(focus_node, bound) {
            Some(std::cmp::Ordering::Less) => {}
            _ => { results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_max_exclusive_constraint_component, shape_node, severity, default_msg.clone())); }
        }
    }
    for bound in get_objects(shapes, shape_node, vocab.sh_max_inclusive) {
        match compare_numeric(focus_node, bound) {
            Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {}
            _ => { results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_max_inclusive_constraint_component, shape_node, severity, default_msg.clone())); }
        }
    }

    // sh:and (node-level) — per spec, a single AndConstraintComponent result
    // is produced for each value node that fails to conform to *all* of the
    // listed shapes; the individual sub-shape violations themselves are not
    // additionally propagated into the report (verified against the W3C
    // data-shapes suite's node/and-001 test, which expects exactly one
    // result per non-conforming focus node).
    for and_list in get_objects(shapes, shape_node, vocab.sh_and) {
        let sub_shapes = get_rdf_list(shapes, and_list);
        let conforms = sub_shapes.iter().all(|&sub| {
            conforms_to_shape(data, shapes, vocab, focus_node, sub, visited)
        });
        if !conforms {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_and_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:or (node-level)
    for or_list in get_objects(shapes, shape_node, vocab.sh_or) {
        let sub_shapes = get_rdf_list(shapes, or_list);
        let conforms = sub_shapes.iter().any(|&sub| {
            conforms_to_shape(data, shapes, vocab, focus_node, sub, visited)
        });
        if !conforms {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_or_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:xone (node-level) — exactly one sub-shape must conform
    for xone_list in get_objects(shapes, shape_node, vocab.sh_xone) {
        let sub_shapes = get_rdf_list(shapes, xone_list);
        let count = sub_shapes.iter()
            .filter(|&&sub| conforms_to_shape(data, shapes, vocab, focus_node, sub, visited))
            .count();
        if count != 1 {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_xone_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:not (node-level)
    for not_shape in get_objects(shapes, shape_node, vocab.sh_not) {
        if conforms_to_shape(data, shapes, vocab, focus_node, not_shape, visited) {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_not_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:node (node-level) — the focus node itself must conform to the referenced shape
    for node_shape in get_objects(shapes, shape_node, vocab.sh_node) {
        if !conforms_to_shape(data, shapes, vocab, focus_node, node_shape, visited) {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_node_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:equals (node-level) — per spec these "property pair" constraint
    // components are defined in terms of a shape's value nodes, which for a
    // NodeShape/direct constraint is just the focus node itself.
    for eq_prop in get_objects(shapes, shape_node, vocab.sh_equals) {
        let other_values: HashSet<usize> = eval_path(data, shapes, focus_node, eq_prop).into_iter().collect();
        let self_values: HashSet<usize> = std::iter::once(focus_node).collect();
        if self_values != other_values {
            if !other_values.contains(&focus_node) {
                results.push(make_result(focus_node, None, Some(focus_node),
                    vocab.sh_equals_constraint_component, shape_node, severity, default_msg.clone()));
            }
            for &v in &other_values {
                if v != focus_node {
                    results.push(make_result(focus_node, None, Some(v),
                        vocab.sh_equals_constraint_component, shape_node, severity, default_msg.clone()));
                }
            }
        }
    }

    // sh:disjoint (node-level)
    for disj_prop in get_objects(shapes, shape_node, vocab.sh_disjoint) {
        let other_values: HashSet<usize> = eval_path(data, shapes, focus_node, disj_prop).into_iter().collect();
        if other_values.contains(&focus_node) {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_disjoint_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:languageIn (node-level)
    for lang_list in get_objects(shapes, shape_node, vocab.sh_language_in) {
        let allowed_langs: Vec<String> = get_rdf_list(shapes, lang_list)
            .into_iter()
            .filter_map(|l| get_lexical_form(l))
            .map(|s| s.to_lowercase())
            .collect();
        if let Some(tag) = get_lang_tag(focus_node) {
            if !allowed_langs.iter().any(|al| tag.to_lowercase().starts_with(al.as_str())) {
                results.push(make_result(focus_node, None, Some(focus_node),
                    vocab.sh_language_in_constraint_component, shape_node, severity, default_msg.clone()));
            }
        } else {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_language_in_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:sparql / SPARQLConstraintComponent (node-level). Per spec this also
    // applies to property shapes (with $this bound to the focus node, not the
    // value nodes), but wiring that through the sh:property loop below would
    // require $PATH pre-binding as well; out of scope here, so sh:sparql on a
    // PropertyShape is only honoured when that shape is also reached directly
    // (e.g. via sh:node/sh:and/sh:or).
    for sparql_node in get_objects(shapes, shape_node, vocab.sh_sparql) {
        validate_sparql_constraint(data, shapes, vocab, focus_node, shape_node, sparql_node, severity, &default_msg, results);
    }

    // -----------------------------------------------------------------------
    // sh:property — property shape constraints
    // -----------------------------------------------------------------------
    for ps in get_objects(shapes, shape_node, vocab.sh_property) {
        validate_property_shape(data, shapes, vocab, focus_node, ps, results, visited);
    }

    validate_shape_closed_and_targets_tail(
        data, shapes, vocab, focus_node, shape_node, severity, default_msg.clone(), results, visited,
    );
}

/// The body of a single `sh:property` constraint, extracted out of
/// `validate_shape` so it can also be invoked recursively for property
/// shapes nested inside another property shape (`sh:property` whose object
/// itself declares `sh:property`) -- each nested shape's own `sh:path` must
/// be evaluated relative to the *value* reached by the outer path, not the
/// original top-level focus node, which is exactly what passing `v` as the
/// new `focus_node` here achieves. See the real W3C test case this fixes:
/// `property/property-001.ttl` ("Test of sh:property at property shape").
fn validate_property_shape(
    data: &TripleIndex,
    shapes: &TripleIndex,
    vocab: &Vocab,
    focus_node: usize,
    ps: usize,
    results: &mut Vec<ValidationResult>,
    visited: &mut HashSet<(usize, usize)>,
) {
    {
        let paths = get_objects(shapes, ps, vocab.sh_path);
        if paths.is_empty() { return; }
        let path = paths[0];
        let v_nodes = eval_path(data, shapes, focus_node, path);
        let ps_severity = get_severity(shapes, ps, vocab);
        let ps_messages = get_shape_messages(shapes, ps, vocab);
        let ps_msg = pick_preferred_message(&ps_messages);

        // sh:minCount
        for mc in get_objects(shapes, ps, vocab.sh_min_count) {
            if let Some(mc_val) = get_integer_value(mc) {
                if (v_nodes.len() as i64) < mc_val {
                    results.push(make_result(focus_node, Some(path), None,
                        vocab.sh_min_count_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:maxCount
        for mc in get_objects(shapes, ps, vocab.sh_max_count) {
            if let Some(mc_val) = get_integer_value(mc) {
                if (v_nodes.len() as i64) > mc_val {
                    results.push(make_result(focus_node, Some(path), None,
                        vocab.sh_max_count_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:datatype (per-value)
        for dt in get_objects(shapes, ps, vocab.sh_datatype) {
            for &v in &v_nodes {
                if !check_datatype(v, dt) {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_datatype_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:nodeKind (per-value)
        for nk in get_objects(shapes, ps, vocab.sh_node_kind) {
            for &v in &v_nodes {
                let ok = if nk == vocab.sh_iri { is_iri(v) }
                    else if nk == vocab.sh_blank_node { is_blank_node(v) }
                    else if nk == vocab.sh_literal { is_literal(v) }
                    else if nk == vocab.sh_blank_node_or_iri { is_blank_node(v) || is_iri(v) }
                    else if nk == vocab.sh_blank_node_or_literal { is_blank_node(v) || is_literal(v) }
                    else if nk == vocab.sh_iri_or_literal { is_iri(v) || is_literal(v) }
                    else { true };
                if !ok {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_node_kind_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:class (per-value)
        for class in get_objects(shapes, ps, vocab.sh_class) {
            for &v in &v_nodes {
                if !has_class(data, v, class, vocab.rdfs_subclass_of, vocab.rdf_type) {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_class_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:hasValue (per-property — value must appear at least once)
        for hv in get_objects(shapes, ps, vocab.sh_has_value) {
            if !v_nodes.contains(&hv) {
                results.push(make_result(focus_node, Some(path), None,
                    vocab.sh_has_value_constraint_component, ps, ps_severity, ps_msg.clone()));
            }
        }

        // sh:in (per-value)
        for in_list in get_objects(shapes, ps, vocab.sh_in) {
            let allowed = get_rdf_list(shapes, in_list);
            for &v in &v_nodes {
                if !allowed.contains(&v) {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_in_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:pattern + sh:flags (per-value)
        let ps_flags = get_objects(shapes, ps, vocab.sh_flags)
            .first()
            .and_then(|f| get_lexical_form(*f))
            .unwrap_or_default();
        for pat in get_objects(shapes, ps, vocab.sh_pattern) {
            if let Some(pat_str) = get_lexical_form(pat) {
                for &v in &v_nodes {
                    let v_str = get_lexical_form(v).unwrap_or_default();
                    if !match_regex(&pat_str, &v_str, &ps_flags) {
                        results.push(make_result(focus_node, Some(path), Some(v),
                            vocab.sh_pattern_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
            }
        }

        // sh:minLength / sh:maxLength (per-value). A value with no string
        // representation (blank nodes -- see get_string_representation)
        // always violates both, not len=0/skip.
        for ml in get_objects(shapes, ps, vocab.sh_min_length) {
            if let Some(min) = get_integer_value(ml) {
                for &v in &v_nodes {
                    let violates = match get_string_representation(v) {
                        Some(s) => (s.chars().count() as i64) < min,
                        None => true,
                    };
                    if violates {
                        results.push(make_result(focus_node, Some(path), Some(v),
                            vocab.sh_min_length_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
            }
        }
        for ml in get_objects(shapes, ps, vocab.sh_max_length) {
            if let Some(max) = get_integer_value(ml) {
                for &v in &v_nodes {
                    let violates = match get_string_representation(v) {
                        Some(s) => (s.chars().count() as i64) > max,
                        None => true,
                    };
                    if violates {
                        results.push(make_result(focus_node, Some(path), Some(v),
                            vocab.sh_max_length_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
            }
        }

        // Numeric range constraints (per-value)
        for bound in get_objects(shapes, ps, vocab.sh_min_exclusive) {
            for &v in &v_nodes {
                match compare_numeric(v, bound) {
                    Some(std::cmp::Ordering::Greater) => {}
                    _ => { results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_min_exclusive_constraint_component, ps, ps_severity, ps_msg.clone())); }
                }
            }
        }
        for bound in get_objects(shapes, ps, vocab.sh_min_inclusive) {
            for &v in &v_nodes {
                match compare_numeric(v, bound) {
                    Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal) => {}
                    _ => { results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_min_inclusive_constraint_component, ps, ps_severity, ps_msg.clone())); }
                }
            }
        }
        for bound in get_objects(shapes, ps, vocab.sh_max_exclusive) {
            for &v in &v_nodes {
                match compare_numeric(v, bound) {
                    Some(std::cmp::Ordering::Less) => {}
                    _ => { results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_max_exclusive_constraint_component, ps, ps_severity, ps_msg.clone())); }
                }
            }
        }
        for bound in get_objects(shapes, ps, vocab.sh_max_inclusive) {
            for &v in &v_nodes {
                match compare_numeric(v, bound) {
                    Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {}
                    _ => { results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_max_inclusive_constraint_component, ps, ps_severity, ps_msg.clone())); }
                }
            }
        }

        // sh:languageIn (per-value)
        for lang_list in get_objects(shapes, ps, vocab.sh_language_in) {
            let allowed_langs: Vec<String> = get_rdf_list(shapes, lang_list)
                .into_iter()
                .filter_map(|l| get_lexical_form(l))
                .map(|s| s.to_lowercase())
                .collect();
            for &v in &v_nodes {
                if let Some(tag) = get_lang_tag(v) {
                    if !allowed_langs.iter().any(|al| tag.to_lowercase().starts_with(al.as_str())) {
                        results.push(make_result(focus_node, Some(path), Some(v),
                            vocab.sh_language_in_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                } else {
                    // Value has no language tag — violates sh:languageIn
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_language_in_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:uniqueLang (per-property) — per spec (and verified against the
        // W3C data-shapes suite's property/uniqueLang-001 test), exactly one
        // validation result is produced per *language tag* that is used by
        // more than one value node, not one per value node beyond the first.
        let unique_lang_vals = get_objects(shapes, ps, vocab.sh_unique_lang);
        for ul in unique_lang_vals {
            if let Some(lex) = get_lexical_form(ul) {
                if lex == "true" || lex == "1" {
                    let mut lang_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                    for &v in &v_nodes {
                        if let Some(tag) = get_lang_tag(v) {
                            *lang_counts.entry(tag.to_lowercase()).or_insert(0) += 1;
                        }
                    }
                    for count in lang_counts.values() {
                        if *count > 1 {
                            results.push(make_result(focus_node, Some(path), None,
                                vocab.sh_unique_lang_constraint_component, ps, ps_severity, ps_msg.clone()));
                        }
                    }
                }
            }
        }

        // sh:equals (values must equal values for the given path)
        for eq_prop in get_objects(shapes, ps, vocab.sh_equals) {
            let other_values: HashSet<usize> = eval_path(data, shapes, focus_node, eq_prop).into_iter().collect();
            let self_values: HashSet<usize> = v_nodes.iter().cloned().collect();
            if self_values != other_values {
                for &v in &v_nodes {
                    if !other_values.contains(&v) {
                        results.push(make_result(focus_node, Some(path), Some(v),
                            vocab.sh_equals_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
                for &v in &other_values {
                    if !self_values.contains(&v) {
                        results.push(make_result(focus_node, Some(path), Some(v),
                            vocab.sh_equals_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
            }
        }

        // sh:disjoint (no overlap between path values and the other property's values)
        for disj_prop in get_objects(shapes, ps, vocab.sh_disjoint) {
            let other_values: HashSet<usize> = eval_path(data, shapes, focus_node, disj_prop).into_iter().collect();
            for &v in &v_nodes {
                if other_values.contains(&v) {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_disjoint_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:lessThan
        for lt_prop in get_objects(shapes, ps, vocab.sh_less_than) {
            let other_values: Vec<usize> = eval_path(data, shapes, focus_node, lt_prop);
            for &v in &v_nodes {
                for &ov in &other_values {
                    match compare_numeric(v, ov) {
                        Some(std::cmp::Ordering::Less) => {}
                        _ => {
                            results.push(make_result(focus_node, Some(path), Some(v),
                                vocab.sh_less_than_constraint_component, ps, ps_severity, ps_msg.clone()));
                        }
                    }
                }
            }
        }

        // sh:lessThanOrEquals
        for lte_prop in get_objects(shapes, ps, vocab.sh_less_than_or_equals) {
            let other_values: Vec<usize> = eval_path(data, shapes, focus_node, lte_prop);
            for &v in &v_nodes {
                for &ov in &other_values {
                    match compare_numeric(v, ov) {
                        Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {}
                        _ => {
                            results.push(make_result(focus_node, Some(path), Some(v),
                                vocab.sh_less_than_or_equals_constraint_component, ps, ps_severity, ps_msg.clone()));
                        }
                    }
                }
            }
        }

        // sh:qualifiedValueShape / sh:qualifiedMinCount / sh:qualifiedMaxCount
        let qvs_list = get_objects(shapes, ps, vocab.sh_qualified_value_shape);
        if !qvs_list.is_empty() {
            let qvs = qvs_list[0];
            let conforming_count = v_nodes.iter()
                .filter(|&&v| conforms_to_shape(data, shapes, vocab, v, qvs, visited))
                .count() as i64;
            for qmin in get_objects(shapes, ps, vocab.sh_qualified_min_count) {
                if let Some(min) = get_integer_value(qmin) {
                    if conforming_count < min {
                        results.push(make_result(focus_node, Some(path), None,
                            vocab.sh_qualified_value_shape_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
            }
            for qmax in get_objects(shapes, ps, vocab.sh_qualified_max_count) {
                if let Some(max) = get_integer_value(qmax) {
                    if conforming_count > max {
                        results.push(make_result(focus_node, Some(path), None,
                            vocab.sh_qualified_value_shape_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
            }
        }

        // Logical constraints on property shape values
        // sh:and per-value — see the node-level sh:and comment above: only the
        // single AndConstraintComponent result is reported, not the nested
        // sub-shape violations.
        for and_list in get_objects(shapes, ps, vocab.sh_and) {
            let sub_shapes = get_rdf_list(shapes, and_list);
            for &v in &v_nodes {
                let conforms = sub_shapes.iter().all(|&sub| {
                    conforms_to_shape(data, shapes, vocab, v, sub, visited)
                });
                if !conforms {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_and_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:or per-value
        for or_list in get_objects(shapes, ps, vocab.sh_or) {
            let sub_shapes = get_rdf_list(shapes, or_list);
            for &v in &v_nodes {
                let conforms = sub_shapes.iter().any(|&sub|
                    conforms_to_shape(data, shapes, vocab, v, sub, visited));
                if !conforms {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_or_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:xone per-value
        for xone_list in get_objects(shapes, ps, vocab.sh_xone) {
            let sub_shapes = get_rdf_list(shapes, xone_list);
            for &v in &v_nodes {
                let count = sub_shapes.iter()
                    .filter(|&&sub| conforms_to_shape(data, shapes, vocab, v, sub, visited))
                    .count();
                if count != 1 {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_xone_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:not per-value
        for not_shape in get_objects(shapes, ps, vocab.sh_not) {
            for &v in &v_nodes {
                if conforms_to_shape(data, shapes, vocab, v, not_shape, visited) {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_not_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:node per-value
        for node_shape in get_objects(shapes, ps, vocab.sh_node) {
            for &v in &v_nodes {
                if !conforms_to_shape(data, shapes, vocab, v, node_shape, visited) {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_node_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:property nested inside a property shape (recursive property
        // shapes, e.g. `ex:PersonShape sh:property [ sh:path ex:address ;
        // sh:property ex:PersonShape-address-city ]`). Each nested property
        // shape has its own sh:path/sh:class/etc that must be evaluated
        // relative to each value node `v` reached by the OUTER path, not
        // relative to the original `focus_node` -- so this recurses into
        // `validate_shape` with `v` as the new focus and the nested shape as
        // shape_node, exactly like the top-level `sh:property` loop does for
        // `focus_node`/`shape_node`, letting that recursive call's own
        // sh:property loop process the nested shape's constraints.
        for ps_nested in get_objects(shapes, ps, vocab.sh_property) {
            for &v in &v_nodes {
                validate_property_shape(data, shapes, vocab, v, ps_nested, results, visited);
            }
        }
    }
}

fn validate_shape_closed_and_targets_tail(
    data: &TripleIndex,
    shapes: &TripleIndex,
    vocab: &Vocab,
    focus_node: usize,
    shape_node: usize,
    severity: usize,
    default_msg: Option<String>,
    results: &mut Vec<ValidationResult>,
    visited: &mut HashSet<(usize, usize)>,
) {
    // sh:closed / sh:ignoredProperties (node-level)
    let closed_vals = get_objects(shapes, shape_node, vocab.sh_closed);
    for cv in closed_vals {
        if let Some(lex) = get_lexical_form(cv) {
            if lex == "true" || lex == "1" {
                // Collect all explicitly allowed predicates from sh:property sh:path
                let mut allowed_preds: HashSet<usize> = HashSet::new();
                for ps in get_objects(shapes, shape_node, vocab.sh_property) {
                    for p in get_objects(shapes, ps, vocab.sh_path) {
                        allowed_preds.insert(p);
                    }
                }
                // Add ignored properties
                for ig_list in get_objects(shapes, shape_node, vocab.sh_ignored_properties) {
                    for ig in get_rdf_list(shapes, ig_list) {
                        allowed_preds.insert(ig);
                    }
                }
                // Check that all actual predicates on the focus node are in allowed set
                if let Some(preds) = data.spo.get(&focus_node) {
                    for &pred in preds.keys() {
                        if !allowed_preds.contains(&pred) {
                            results.push(make_result(focus_node, Some(pred), None,
                                vocab.sh_closed_constraint_component, shape_node, severity, default_msg.clone()));
                        }
                    }
                }
            }
        }
    }

    // Remove from visited so the same (focus, shape) pair can be re-checked from
    // a different call path (e.g., inside sh:and or sh:or sub-checks). This is
    // correct because the visited set is only used to prevent infinite recursion,
    // not to cache results.
    visited.remove(&(focus_node, shape_node));
}

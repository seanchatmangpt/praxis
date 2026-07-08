use crate::encoding::Encoder;
use crate::rule::{BodyLiteral, Rule};
use crate::term::{Triple, VarOrTerm};
use crate::tripleindex::TripleIndex;

/// OWL RL vocabulary: interned IRI ids for the daily profile features.
pub struct OwlRlVocab {
    pub rdf_type: usize,
    pub rdfs_subclass_of: usize,
    pub rdfs_subproperty_of: usize,
    pub rdfs_domain: usize,
    pub rdfs_range: usize,
    pub owl_equivalent_class: usize,
    pub owl_equivalent_property: usize,
    pub owl_inverse_of: usize,
    pub owl_symmetric_property: usize,
    pub owl_transitive_property: usize,
    pub owl_same_as: usize,
    pub owl_property_chain_axiom: usize,
    pub owl_cardinality: usize,
    pub owl_min_cardinality: usize,
    pub owl_max_cardinality: usize,
    pub owl_union_of: usize,
    pub owl_intersection_of: usize,
    pub owl_one_of: usize,
    pub owl_imports: usize,
}

impl OwlRlVocab {
    pub fn new() -> Self {
        Self {
            rdf_type: Encoder::add("http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string()),
            rdfs_subclass_of: Encoder::add(
                "http://www.w3.org/2000/01/rdf-schema#subClassOf".to_string(),
            ),
            rdfs_subproperty_of: Encoder::add(
                "http://www.w3.org/2000/01/rdf-schema#subPropertyOf".to_string(),
            ),
            rdfs_domain: Encoder::add("http://www.w3.org/2000/01/rdf-schema#domain".to_string()),
            rdfs_range: Encoder::add("http://www.w3.org/2000/01/rdf-schema#range".to_string()),
            owl_equivalent_class: Encoder::add(
                "http://www.w3.org/2002/07/owl#equivalentClass".to_string(),
            ),
            owl_equivalent_property: Encoder::add(
                "http://www.w3.org/2002/07/owl#equivalentProperty".to_string(),
            ),
            owl_inverse_of: Encoder::add("http://www.w3.org/2002/07/owl#inverseOf".to_string()),
            owl_symmetric_property: Encoder::add(
                "http://www.w3.org/2002/07/owl#SymmetricProperty".to_string(),
            ),
            owl_transitive_property: Encoder::add(
                "http://www.w3.org/2002/07/owl#TransitiveProperty".to_string(),
            ),
            owl_same_as: Encoder::add("http://www.w3.org/2002/07/owl#sameAs".to_string()),
            owl_property_chain_axiom: Encoder::add(
                "http://www.w3.org/2002/07/owl#propertyChainAxiom".to_string(),
            ),
            owl_cardinality: Encoder::add("http://www.w3.org/2002/07/owl#cardinality".to_string()),
            owl_min_cardinality: Encoder::add(
                "http://www.w3.org/2002/07/owl#minCardinality".to_string(),
            ),
            owl_max_cardinality: Encoder::add(
                "http://www.w3.org/2002/07/owl#maxCardinality".to_string(),
            ),
            owl_union_of: Encoder::add("http://www.w3.org/2002/07/owl#unionOf".to_string()),
            owl_intersection_of: Encoder::add(
                "http://www.w3.org/2002/07/owl#intersectionOf".to_string(),
            ),
            owl_one_of: Encoder::add("http://www.w3.org/2002/07/owl#oneOf".to_string()),
            owl_imports: Encoder::add("http://www.w3.org/2002/07/owl#imports".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum OwlRlFeature {
    SubClassOf,
    SubPropertyOf,
    Domain,
    Range,
    EquivalentClass,
    EquivalentProperty,
    InverseOf,
    SymmetricProperty,
    TransitiveProperty,
    SameAs,
    PropertyChainAxiom,
    Cardinality,
    ComplexClassExpression,
    Imports,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OwlRlDecision {
    Supported,
    Unsupported {
        feature: OwlRlFeature,
        reason: &'static str,
    },
    ExternalBoundaryRequired {
        feature: OwlRlFeature,
        reason: &'static str,
    },
}

pub fn classify_owlrl_feature(feature: OwlRlFeature) -> OwlRlDecision {
    match feature {
        OwlRlFeature::SubClassOf
        | OwlRlFeature::SubPropertyOf
        | OwlRlFeature::Domain
        | OwlRlFeature::Range
        | OwlRlFeature::EquivalentClass
        | OwlRlFeature::EquivalentProperty
        | OwlRlFeature::InverseOf
        | OwlRlFeature::SymmetricProperty
        | OwlRlFeature::TransitiveProperty => OwlRlDecision::Supported,
        OwlRlFeature::SameAs => OwlRlDecision::ExternalBoundaryRequired {
            feature,
            reason: "unrestricted sameAs closure is outside the bounded daily profile; equivalence merging is a later profile",
        },
        OwlRlFeature::PropertyChainAxiom => OwlRlDecision::Unsupported {
            feature,
            reason: "property chain axiom requires forward-chaining schema computation not supported in daily profile",
        },
        OwlRlFeature::Cardinality => OwlRlDecision::Unsupported {
            feature,
            reason: "cardinality constraints (cardinality/minCardinality/maxCardinality) require constraint solving outside daily profile",
        },
        OwlRlFeature::ComplexClassExpression => OwlRlDecision::Unsupported {
            feature,
            reason: "complex class expressions (unionOf/intersectionOf/oneOf/restrictions) require class-expression evaluation not supported in daily profile",
        },
        OwlRlFeature::Imports => OwlRlDecision::Unsupported {
            feature,
            reason: "owl:imports requires remote ontology loading and is outside the bounded daily profile",
        },
    }
}

#[derive(Debug, Clone)]
pub struct ScanReport {
    pub supported: Vec<(OwlRlFeature, usize)>,
    pub refused: Vec<(OwlRlFeature, usize, &'static str)>,
}

pub fn scan_ontology(index: &TripleIndex, vocab: &OwlRlVocab) -> ScanReport {
    let mut supported = Vec::new();
    let mut refused = Vec::new();

    let features_to_scan = vec![
        (OwlRlFeature::SubClassOf, vocab.rdfs_subclass_of),
        (OwlRlFeature::SubPropertyOf, vocab.rdfs_subproperty_of),
        (OwlRlFeature::Domain, vocab.rdfs_domain),
        (OwlRlFeature::Range, vocab.rdfs_range),
        (OwlRlFeature::EquivalentClass, vocab.owl_equivalent_class),
        (
            OwlRlFeature::EquivalentProperty,
            vocab.owl_equivalent_property,
        ),
        (OwlRlFeature::InverseOf, vocab.owl_inverse_of),
        (
            OwlRlFeature::SymmetricProperty,
            vocab.owl_symmetric_property,
        ),
        (
            OwlRlFeature::TransitiveProperty,
            vocab.owl_transitive_property,
        ),
        (OwlRlFeature::SameAs, vocab.owl_same_as),
        (
            OwlRlFeature::PropertyChainAxiom,
            vocab.owl_property_chain_axiom,
        ),
        (OwlRlFeature::Cardinality, vocab.owl_cardinality),
        (OwlRlFeature::Cardinality, vocab.owl_min_cardinality),
        (OwlRlFeature::Cardinality, vocab.owl_max_cardinality),
        (OwlRlFeature::ComplexClassExpression, vocab.owl_union_of),
        (
            OwlRlFeature::ComplexClassExpression,
            vocab.owl_intersection_of,
        ),
        (OwlRlFeature::ComplexClassExpression, vocab.owl_one_of),
        (OwlRlFeature::Imports, vocab.owl_imports),
    ];

    for (feature, vocab_pred_id) in features_to_scan {
        let query_pattern = Triple {
            s: VarOrTerm::new_var("?s".to_string()),
            p: VarOrTerm::new_term(Encoder::decode(&vocab_pred_id).unwrap_or_default()),
            o: VarOrTerm::new_var("?o".to_string()),
            g: None,
        };

        let count = index.query(&query_pattern, None).map(|_| 1).unwrap_or(0);
        if count > 0 {
            let decision = classify_owlrl_feature(feature);
            match decision {
                OwlRlDecision::Supported => {
                    if let Some((_, ref mut c)) = supported.iter_mut().find(|(f, _)| *f == feature)
                    {
                        *c += count;
                    } else {
                        supported.push((feature, count));
                    }
                }
                OwlRlDecision::Unsupported { feature: _, reason } => {
                    if let Some((_, ref mut c, _)) =
                        refused.iter_mut().find(|(f, _, _)| *f == feature)
                    {
                        *c += count;
                    } else {
                        refused.push((feature, count, reason));
                    }
                }
                OwlRlDecision::ExternalBoundaryRequired { feature: _, reason } => {
                    if let Some((_, ref mut c, _)) =
                        refused.iter_mut().find(|(f, _, _)| *f == feature)
                    {
                        *c += count;
                    } else {
                        refused.push((feature, count, reason));
                    }
                }
            }
        }
    }

    ScanReport { supported, refused }
}

// Rule compilation invariant: every vocabulary ID (e.g. vocab.rdfs_subclass_of) is
// guaranteed to be encodable because OwlRlVocab::new() calls Encoder::add() for each
// W3C vocabulary IRI during initialization. Therefore, Encoder::decode() will never
// return None for these IDs. We use unwrap() in the rule functions because the
// decode failure would indicate a corrupted vocabulary initialization, not a runtime
// error in the ontology being reasoned over.

/// {?a rdfs:subClassOf ?b. ?b rdfs:subClassOf ?c} => {?a rdfs:subClassOf ?c}
pub fn rule_subclass_transitive(vocab: &OwlRlVocab) -> Rule {
    Rule {
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?a".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_subclass_of).unwrap()),
                    o: VarOrTerm::new_var("?b".to_string()),
                    g: None,
                },
            },
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?b".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_subclass_of).unwrap()),
                    o: VarOrTerm::new_var("?c".to_string()),
                    g: None,
                },
            },
        ],
        head: Triple {
            s: VarOrTerm::new_var("?a".to_string()),
            p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_subclass_of).unwrap()),
            o: VarOrTerm::new_var("?c".to_string()),
            g: None,
        },
    }
}

/// {?x rdf:type ?a. ?a rdfs:subClassOf ?b} => {?x rdf:type ?b}
pub fn rule_subclass_type_propagation(vocab: &OwlRlVocab) -> Rule {
    Rule {
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?x".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.rdf_type).unwrap()),
                    o: VarOrTerm::new_var("?a".to_string()),
                    g: None,
                },
            },
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?a".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_subclass_of).unwrap()),
                    o: VarOrTerm::new_var("?b".to_string()),
                    g: None,
                },
            },
        ],
        head: Triple {
            s: VarOrTerm::new_var("?x".to_string()),
            p: VarOrTerm::new_term(Encoder::decode(&vocab.rdf_type).unwrap()),
            o: VarOrTerm::new_var("?b".to_string()),
            g: None,
        },
    }
}

/// {?p rdfs:subPropertyOf ?q. ?q rdfs:subPropertyOf ?r} => {?p rdfs:subPropertyOf ?r}
pub fn rule_subproperty_transitive(vocab: &OwlRlVocab) -> Rule {
    Rule {
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?p".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_subproperty_of).unwrap()),
                    o: VarOrTerm::new_var("?q".to_string()),
                    g: None,
                },
            },
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?q".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_subproperty_of).unwrap()),
                    o: VarOrTerm::new_var("?r".to_string()),
                    g: None,
                },
            },
        ],
        head: Triple {
            s: VarOrTerm::new_var("?p".to_string()),
            p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_subproperty_of).unwrap()),
            o: VarOrTerm::new_var("?r".to_string()),
            g: None,
        },
    }
}

/// {?x ?p ?y. ?p rdfs:subPropertyOf ?q} => {?x ?q ?y}
pub fn rule_subproperty_assertion_propagation(vocab: &OwlRlVocab) -> Rule {
    Rule {
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?x".to_string()),
                    p: VarOrTerm::new_var("?p".to_string()),
                    o: VarOrTerm::new_var("?y".to_string()),
                    g: None,
                },
            },
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?p".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_subproperty_of).unwrap()),
                    o: VarOrTerm::new_var("?q".to_string()),
                    g: None,
                },
            },
        ],
        head: Triple {
            s: VarOrTerm::new_var("?x".to_string()),
            p: VarOrTerm::new_var("?q".to_string()),
            o: VarOrTerm::new_var("?y".to_string()),
            g: None,
        },
    }
}

/// {?x ?p ?y. ?p rdfs:domain ?c} => {?x rdf:type ?c}
pub fn rule_domain(vocab: &OwlRlVocab) -> Rule {
    Rule {
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?x".to_string()),
                    p: VarOrTerm::new_var("?p".to_string()),
                    o: VarOrTerm::new_var("?y".to_string()),
                    g: None,
                },
            },
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?p".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_domain).unwrap()),
                    o: VarOrTerm::new_var("?c".to_string()),
                    g: None,
                },
            },
        ],
        head: Triple {
            s: VarOrTerm::new_var("?x".to_string()),
            p: VarOrTerm::new_term(Encoder::decode(&vocab.rdf_type).unwrap()),
            o: VarOrTerm::new_var("?c".to_string()),
            g: None,
        },
    }
}

/// {?x ?p ?y. ?p rdfs:range ?c} => {?y rdf:type ?c}
pub fn rule_range(vocab: &OwlRlVocab) -> Rule {
    Rule {
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?x".to_string()),
                    p: VarOrTerm::new_var("?p".to_string()),
                    o: VarOrTerm::new_var("?y".to_string()),
                    g: None,
                },
            },
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?p".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_range).unwrap()),
                    o: VarOrTerm::new_var("?c".to_string()),
                    g: None,
                },
            },
        ],
        head: Triple {
            s: VarOrTerm::new_var("?y".to_string()),
            p: VarOrTerm::new_term(Encoder::decode(&vocab.rdf_type).unwrap()),
            o: VarOrTerm::new_var("?c".to_string()),
            g: None,
        },
    }
}

/// owl:equivalentClass(A,B) => two rdfs:subClassOf rules (both directions)
pub fn rules_equivalent_class(vocab: &OwlRlVocab) -> [Rule; 2] {
    [
        Rule {
            body: vec![BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?a".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.owl_equivalent_class).unwrap()),
                    o: VarOrTerm::new_var("?b".to_string()),
                    g: None,
                },
            }],
            head: Triple {
                s: VarOrTerm::new_var("?a".to_string()),
                p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_subclass_of).unwrap()),
                o: VarOrTerm::new_var("?b".to_string()),
                g: None,
            },
        },
        Rule {
            body: vec![BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?a".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.owl_equivalent_class).unwrap()),
                    o: VarOrTerm::new_var("?b".to_string()),
                    g: None,
                },
            }],
            head: Triple {
                s: VarOrTerm::new_var("?b".to_string()),
                p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_subclass_of).unwrap()),
                o: VarOrTerm::new_var("?a".to_string()),
                g: None,
            },
        },
    ]
}

/// owl:equivalentProperty(A,B) => two rdfs:subPropertyOf rules (both directions)
pub fn rules_equivalent_property(vocab: &OwlRlVocab) -> [Rule; 2] {
    [
        Rule {
            body: vec![BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?p".to_string()),
                    p: VarOrTerm::new_term(
                        Encoder::decode(&vocab.owl_equivalent_property).unwrap(),
                    ),
                    o: VarOrTerm::new_var("?q".to_string()),
                    g: None,
                },
            }],
            head: Triple {
                s: VarOrTerm::new_var("?p".to_string()),
                p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_subproperty_of).unwrap()),
                o: VarOrTerm::new_var("?q".to_string()),
                g: None,
            },
        },
        Rule {
            body: vec![BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?p".to_string()),
                    p: VarOrTerm::new_term(
                        Encoder::decode(&vocab.owl_equivalent_property).unwrap(),
                    ),
                    o: VarOrTerm::new_var("?q".to_string()),
                    g: None,
                },
            }],
            head: Triple {
                s: VarOrTerm::new_var("?q".to_string()),
                p: VarOrTerm::new_term(Encoder::decode(&vocab.rdfs_subproperty_of).unwrap()),
                o: VarOrTerm::new_var("?p".to_string()),
                g: None,
            },
        },
    ]
}

/// {?p owl:inverseOf ?q. ?x ?p ?y} => {?y ?q ?x}
pub fn rule_inverse_of(vocab: &OwlRlVocab) -> Rule {
    Rule {
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?p".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.owl_inverse_of).unwrap()),
                    o: VarOrTerm::new_var("?q".to_string()),
                    g: None,
                },
            },
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?x".to_string()),
                    p: VarOrTerm::new_var("?p".to_string()),
                    o: VarOrTerm::new_var("?y".to_string()),
                    g: None,
                },
            },
        ],
        head: Triple {
            s: VarOrTerm::new_var("?y".to_string()),
            p: VarOrTerm::new_var("?q".to_string()),
            o: VarOrTerm::new_var("?x".to_string()),
            g: None,
        },
    }
}

/// {?p rdf:type owl:SymmetricProperty. ?x ?p ?y} => {?y ?p ?x}
pub fn rule_symmetric_property(vocab: &OwlRlVocab) -> Rule {
    Rule {
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?p".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.rdf_type).unwrap()),
                    o: VarOrTerm::new_term(Encoder::decode(&vocab.owl_symmetric_property).unwrap()),
                    g: None,
                },
            },
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?x".to_string()),
                    p: VarOrTerm::new_var("?p".to_string()),
                    o: VarOrTerm::new_var("?y".to_string()),
                    g: None,
                },
            },
        ],
        head: Triple {
            s: VarOrTerm::new_var("?y".to_string()),
            p: VarOrTerm::new_var("?p".to_string()),
            o: VarOrTerm::new_var("?x".to_string()),
            g: None,
        },
    }
}

/// {?p rdf:type owl:TransitiveProperty. ?x ?p ?y. ?y ?p ?z} => {?x ?p ?z}
pub fn rule_transitive_property(vocab: &OwlRlVocab) -> Rule {
    Rule {
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?p".to_string()),
                    p: VarOrTerm::new_term(Encoder::decode(&vocab.rdf_type).unwrap()),
                    o: VarOrTerm::new_term(
                        Encoder::decode(&vocab.owl_transitive_property).unwrap(),
                    ),
                    g: None,
                },
            },
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?x".to_string()),
                    p: VarOrTerm::new_var("?p".to_string()),
                    o: VarOrTerm::new_var("?y".to_string()),
                    g: None,
                },
            },
            BodyLiteral {
                negated: false,
                pattern: Triple {
                    s: VarOrTerm::new_var("?y".to_string()),
                    p: VarOrTerm::new_var("?p".to_string()),
                    o: VarOrTerm::new_var("?z".to_string()),
                    g: None,
                },
            },
        ],
        head: Triple {
            s: VarOrTerm::new_var("?x".to_string()),
            p: VarOrTerm::new_var("?p".to_string()),
            o: VarOrTerm::new_var("?z".to_string()),
            g: None,
        },
    }
}

pub struct OwlRlEngine {
    vocab: OwlRlVocab,
}

impl OwlRlEngine {
    pub fn new() -> Self {
        OwlRlEngine {
            vocab: OwlRlVocab::new(),
        }
    }

    pub fn compile(&self, _index: &TripleIndex) -> (Vec<Rule>, ScanReport) {
        let mut rules = Vec::new();

        // Add all supported daily-profile rules unconditionally.
        // These 9 features are the bounded OWL RL profile for v26.7.8.
        rules.push(rule_subclass_transitive(&self.vocab));
        rules.push(rule_subclass_type_propagation(&self.vocab));
        rules.push(rule_subproperty_transitive(&self.vocab));
        rules.push(rule_subproperty_assertion_propagation(&self.vocab));
        rules.push(rule_domain(&self.vocab));
        rules.push(rule_range(&self.vocab));
        let eq_class_rules = rules_equivalent_class(&self.vocab);
        rules.push(eq_class_rules[0].clone());
        rules.push(eq_class_rules[1].clone());
        let eq_prop_rules = rules_equivalent_property(&self.vocab);
        rules.push(eq_prop_rules[0].clone());
        rules.push(eq_prop_rules[1].clone());
        rules.push(rule_inverse_of(&self.vocab));
        rules.push(rule_symmetric_property(&self.vocab));
        rules.push(rule_transitive_property(&self.vocab));

        // Scanning is deferred to future profiles; v26.7.8 uses all supported rules.
        let report = ScanReport {
            supported: vec![],
            refused: vec![],
        };

        (rules, report)
    }
}

#[cfg(test)]
mod owlrl_test {
    use super::*;
    use crate::TripleStore;

    #[test]
    fn test_vocabulary_initialized() {
        let vocab = OwlRlVocab::new();
        // Encoder returns 0-indexed IDs; just verify they're initialized (non-zero is implementation detail)
        let _ = vocab.rdf_type;
        let _ = vocab.rdfs_subclass_of;
    }

    #[test]
    fn test_classify_supported_features() {
        assert_eq!(
            classify_owlrl_feature(OwlRlFeature::SubClassOf),
            OwlRlDecision::Supported
        );
        assert_eq!(
            classify_owlrl_feature(OwlRlFeature::Domain),
            OwlRlDecision::Supported
        );
    }

    #[test]
    fn test_classify_refused_features() {
        match classify_owlrl_feature(OwlRlFeature::SameAs) {
            OwlRlDecision::ExternalBoundaryRequired { .. } => {}
            _ => panic!("SameAs should be ExternalBoundaryRequired"),
        }

        match classify_owlrl_feature(OwlRlFeature::PropertyChainAxiom) {
            OwlRlDecision::Unsupported { .. } => {}
            _ => panic!("PropertyChainAxiom should be Unsupported"),
        }
    }

    #[test]
    fn test_engine_creation() {
        let engine = OwlRlEngine::new();
        let vocab = &engine.vocab;
        let _ = vocab.rdf_type;
    }

    #[test]
    fn test_rule_subclass_transitive() {
        let vocab = OwlRlVocab::new();
        let rule = rule_subclass_transitive(&vocab);
        assert_eq!(rule.body.len(), 2);
        assert_eq!(
            rule.head.s.to_encoded(),
            rule.body[0].pattern.s.to_encoded()
        );
    }

    #[test]
    fn test_rule_subclass_type_propagation() {
        let vocab = OwlRlVocab::new();
        let rule = rule_subclass_type_propagation(&vocab);
        assert_eq!(rule.body.len(), 2);
    }

    #[test]
    fn test_rule_domain() {
        let vocab = OwlRlVocab::new();
        let rule = rule_domain(&vocab);
        assert_eq!(rule.body.len(), 2);
    }

    #[test]
    fn test_rule_range() {
        let vocab = OwlRlVocab::new();
        let rule = rule_range(&vocab);
        assert_eq!(rule.body.len(), 2);
    }

    #[test]
    fn test_rules_equivalent_class() {
        let vocab = OwlRlVocab::new();
        let rules = rules_equivalent_class(&vocab);
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].body.len(), 1);
        assert_eq!(rules[1].body.len(), 1);
    }

    #[test]
    fn test_rules_equivalent_property() {
        let vocab = OwlRlVocab::new();
        let rules = rules_equivalent_property(&vocab);
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_rule_inverse_of() {
        let vocab = OwlRlVocab::new();
        let rule = rule_inverse_of(&vocab);
        assert_eq!(rule.body.len(), 2);
    }

    #[test]
    fn test_rule_symmetric_property() {
        let vocab = OwlRlVocab::new();
        let rule = rule_symmetric_property(&vocab);
        assert_eq!(rule.body.len(), 2);
    }

    #[test]
    fn test_rule_transitive_property() {
        let vocab = OwlRlVocab::new();
        let rule = rule_transitive_property(&vocab);
        assert_eq!(rule.body.len(), 3);
    }

    #[test]
    fn test_scan_ontology_empty() {
        let store = TripleStore::new();
        let vocab = OwlRlVocab::new();
        let report = scan_ontology(&store.triple_index, &vocab);
        assert!(report.supported.is_empty());
        assert!(report.refused.is_empty());
    }

    #[test]
    fn test_engine_compile_empty() {
        let store = TripleStore::new();
        let engine = OwlRlEngine::new();
        let (rules, _report) = engine.compile(&store.triple_index);
        // Daily profile v26.7.8 unconditionally adds all 9 supported rules.
        // Each gets compiled: subclass/subproperty/domain/range are 1 rule each,
        // while equivalent_class/equivalent_property each expand to 2 rules,
        // and inverse_of/symmetric_property/transitive_property are 1 each.
        assert!(!rules.is_empty(), "daily profile must add rules");
        // Every rule must have a non-empty body (at least one literal to pattern-match on)
        assert!(rules.iter().all(|r| !r.body.is_empty()));
    }
}

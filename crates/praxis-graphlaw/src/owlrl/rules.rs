use crate::encoding::Encoder;
use crate::rule::{BodyLiteral, Rule};
use crate::term::{Triple, VarOrTerm};

use super::OwlRlVocab;

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

#![cfg(test)]

use crate::reasoner::Reasoner;
    use crate::{
        BackwardChainer, Encoder, Parser, QueryEngine, Rule, RuleIndex, SimpleQueryEngine,
        TermImpl, Triple, TripleIndex, TripleStore, VarOrTerm, BodyLiteral,
    };
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn test_parse() {
        let data = ":a a :C0.\n\
            {?a a :C0}=>{?a a :C1}\n\
            {?a a :C1}=>{?a a :C2}\n\
            {?a a :C2}=>{?a a :C3}";

        let mut store = TripleStore::from(data);

        let mat = store.materialize();
        println!("Length: {:?}", store.len());
        println!("Length Mat: {:?}", mat.len());
    }

    #[test]
    fn test_store() {
        let timer = ::std::time::Instant::now();
        let mut rules = Vec::new();
        let max_depth = 5;
        for i in 0..max_depth {
            let rule = Rule {
                head: Triple {
                    s: VarOrTerm::new_var("s".to_string()),
                    p: VarOrTerm::new_term("http://test".to_string()),
                    o: VarOrTerm::new_term(format!("U{}", i + 1)),
                    g: None,
                },
                body: Vec::from([BodyLiteral {
                    negated: false,
                    pattern: Triple {
                        s: VarOrTerm::new_var("s".to_string()),
                        p: VarOrTerm::new_term("http://test".to_string()),
                        o: VarOrTerm::new_term(format!("U{}", i)),
                        g: None,
                    },
                }]),
            };
            rules.push(rule);
        }

        let content = Vec::from([Triple {
            s: VarOrTerm::new_term("sTerm".to_string()),
            p: VarOrTerm::new_term("http://test".to_string()),
            o: VarOrTerm::new_term("U0".to_string()),
            g: None,
        }]);
        let mut rules_index = RuleIndex::new();
        for rule in rules.iter() {
            rules_index.add_ref(rule);
        }
        let mut triple_index = TripleIndex::new();
        content.into_iter().for_each(|t| triple_index.add(t));
        let query = Triple {
            s: VarOrTerm::new_var("s".to_string()),
            p: VarOrTerm::new_term("http://test".to_string()),
            o: VarOrTerm::new_term(format!("U{}", max_depth)),
            g: None,
        };

        let mut store = TripleStore {
            rules: Vec::new(),
            rules_index,
            triple_index,
            reasoner: Reasoner {},
            aggregates: HashMap::new(),
            strata: Vec::new(),
        };

        store.materialize();
        let elapsed = timer.elapsed();

        let result = SimpleQueryEngine::query(&store.triple_index, &Vec::from([BodyLiteral { negated: false, pattern: query }]), None);

        println!("Processed in: {:.2?}", elapsed);
        println!("Result: {:?}", result);
    }

    #[test]
    fn test_incomplete_rule_match() {
        let data = ":a in :b.\n\
            {?a in ?b. ?b in ?c}=>{?a in ?c.}";

        let mut store = TripleStore::from(data);
        assert_eq!(1, store.len());
        store.materialize();
        assert_eq!(1, store.len());
    }
    #[test]
    fn test_no_var_query() {
        let data = ":a in :b.\n\
            {:a in :b}=>{:a in :c}";

        let mut store = TripleStore::from(data);
        assert_eq!(1, store.len());
        store.materialize();
        assert_eq!(2, store.len());
    }
    #[test]
    fn test_single_rule() {
        let data = ":a a :A.\n\
            {?a a :A}=>{?a a :B}";

        let mut store = TripleStore::from(data);
        assert_eq!(1, store.len());
        store.materialize();
        assert_eq!(2, store.len());
    }
    #[test]
    fn test_multiple_rule() {
        let data = ":a a :A.\n\
            {?a a :A}=>{?a a :B}\n\
            {?a a :B}=>{?a a :C}";

        let mut store = TripleStore::from(data);
        assert_eq!(1, store.len());
        store.materialize();
        assert_eq!(3, store.len());
    }
    #[test]
    fn test_join_rule() {
        let data = ":a a :A.\n\
            :a in :b.\n\
            {?a a :A.?a in ?o}=>{?a a :B}";

        let mut store = TripleStore::from(data);
        assert_eq!(2, store.len());
        store.materialize();
        assert_eq!(3, store.len());
    }
    #[test]
    fn test_long_join_rule() {
        let data = ":a a :A.\n\
            :a in :b.\n\
            :b in :c.\n\
            :c a :A.\n\
            {?a a :A.?a in ?o.?o in ?o2.?o2 a :A}=>{?a a :B}";

        let mut store = TripleStore::from(data);
        assert_eq!(4, store.len());
        store.materialize();
        assert_eq!(5, store.len());
    }
    #[test]
    fn test_transitive_rule() {
        let mut data = "{?a in ?b.?b in ?c}=>{?a in ?c}\n".to_owned();
        for i in 0..10 {
            data += format!(":{} in :{}.\n", i + 1, i).as_str();
        }
        let mut store = TripleStore::from(data.as_str());
        assert_eq!(10, store.len());
        store.materialize();
        assert_eq!(55, store.len());
    }
    #[test]
    fn test_hierarchy() {
        let max_depth = 10;
        let mut data = ":a a :U0\n".to_owned();
        for i in 0..max_depth {
            data += format!("{{?a a :U{}}}=>{{?a a :U{}}}\n", i, i + 1).as_str();
            data += format!("{{?a a :U{}}}=>{{?a a :J{}}}\n", i, i + 1).as_str();
            data += format!("{{?a a :U{}}}=>{{?a a :Q{}}}\n", i, i + 1).as_str();
        }
        println!("{}", data);
        let mut store = TripleStore::from(data.as_str());
        let inferred = store.materialize();
        println!("Triples: {:?}", store.len());
        assert_eq!(3 * max_depth, inferred.len());
    }
    #[test]
    fn test_rdf_hierarchy() {
        let max_depth = 10;
        let mut data = ":a a :U0\n\
                        {?a :subClassOf ?b.?b :subClassOf ?c}=>{?a :subClassOf ?c}\n"
            .to_owned();
        for i in 0..max_depth {
            data += format!(":U{} :subClassOf :U{}.\n", i, i + 1).as_str();
            data += format!(":U{} :subClassOf :J{}.\n", i, i + 1).as_str();
            data += format!(":U{} :subClassOf :Q{}.\n", i, i + 1).as_str();
        }
        let mut store = TripleStore::from(data.as_str());
        let inferred = store.materialize();
        println!("Inferred: {:?}, Total: {:?}", inferred.len(), store.len());
        // The transitive closure over 3 interleaved subClassOf chains produces 135 inferred triples.
        assert_eq!(135, inferred.len());
    }
    // #[test]
    // fn test_eval_backward_multiple_rules(){
    //     let mut store = ReasoningStore::new();
    //     store.parse_and_add_rule("@prefix test: <http://www.test.be/test#>.\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n \
    //     {?s rdf:type test:SubClass.}=>{?s rdf:type test:SuperType.}\n\
    //     {?s rdf:type test:SubClass2.}=>{?s rdf:type test:SuperType.}");
    //     store.load_abox( b"<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .".as_ref());
    //     store.load_abox( b"<http://example2.com/c> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass2> .".as_ref());
    //
    //     // diff variable names
    //     let backward_head = ReasonerTriple::new("?newVar".to_string(),"http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),"http://www.test.be/test#SuperType".to_string());
    //     let  bindings = store.eval_backward( &backward_head);
    //     let mut result_bindings: Binding = Binding::new();
    //     result_bindings.add("newVar", Term::from(NamedNode::new("http://example2.com/a".to_string()).unwrap()));
    //     result_bindings.add("newVar", Term::from(NamedNode::new("http://example2.com/c".to_string()).unwrap()));
    //
    //     assert_eq!(result_bindings, bindings);
    // }
    // #[test]
    // fn test_eval_backward_nested_rules(){
    //     let mut store = ReasoningStore::new();
    //     store.parse_and_add_rule("@prefix test: <http://www.test.be/test#>.\n @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.\n \
    //     {?s rdf:type test:SubClass. ?s test:hasRef ?o. ?o rdf:type test:SubClass2.}=>{?s rdf:type test:SuperType.}\n\
    //     {?q rdf:type test:SubClassTemp.}=>{?q rdf:type test:SubClass2.}");
    //     store.load_abox( b"<http://example2.com/a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClass> .".as_ref());
    //     store.load_abox( b"<http://example2.com/b> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.test.be/test#SubClassTemp> .".as_ref());
    //     store.load_abox( b"<http://example2.com/a> <http://www.test.be/test#hasRef> <http://example2.com/b> .".as_ref());
    //
    //     // diff variable names
    //     let backward_head = ReasonerTriple::new("?newVar".to_string(),"http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),"http://www.test.be/test#SuperType".to_string());
    //     let  bindings = store.eval_backward( &backward_head);
    //     let mut result_bindings: Binding = Binding::new();
    //     result_bindings.add("newVar", Term::from(NamedNode::new("http://example2.com/a".to_string()).unwrap()));
    //
    //     assert_eq!(result_bindings, bindings);
    // }

    // -----------------------------------------------------------------------
    // SHACL integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_shacl_min_count_violation() {
        let mut store = TripleStore::new();
        store
            .load_triples(
                "<http://example/alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example/Person> .",
                crate::parser::Syntax::NTriples,
            )
            .unwrap();

        let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example/> .
ex:PersonShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [ sh:path ex:name ; sh:minCount 1 ] .";
        let report = store.validate_shacl(shapes).unwrap();
        assert!(!report.conforms, "Expected a sh:minCount violation");
        assert!(!report.results.is_empty());
    }

    #[test]
    fn test_shacl_conforms() {
        let mut store = TripleStore::new();
        store
            .load_triples(
                "<http://example/bob> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example/Person> .\n<http://example/bob> <http://example/name> \"Bob\" .",
                crate::parser::Syntax::NTriples,
            )
            .unwrap();

        let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example/> .
ex:PersonShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [ sh:path ex:name ; sh:minCount 1 ; sh:maxCount 1 ] .";
        let report = store.validate_shacl(shapes).unwrap();
        assert!(report.conforms, "Expected conformance");
    }

    #[test]
    fn test_shacl_report_to_triples() {
        let mut store = TripleStore::new();
        store
            .load_triples(
                "<http://example/e> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example/P> .",
                crate::parser::Syntax::NTriples,
            )
            .unwrap();

        let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example/> .
ex:PShape a sh:NodeShape ;
    sh:targetClass ex:P ;
    sh:property [ sh:path ex:name ; sh:minCount 1 ] .";
        let report = store.validate_shacl(shapes).unwrap();
        assert!(!report.conforms);
        let triples = report.to_triples();
        assert!(!triples.is_empty(), "Report should serialise to RDF triples");
    }

    #[test]
    fn test_shacl_node_constraint_violation() {
        let mut store = TripleStore::new();
        store
            .load_triples(
                "<http://example/alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example/Person> .\n<http://example/alice> <http://example/address> <http://example/addr1> .",
                crate::parser::Syntax::NTriples,
            )
            .unwrap();

        let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example/> .
ex:AddressShape a sh:NodeShape ;
    sh:property [ sh:path ex:city ; sh:minCount 1 ] .
ex:PersonShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [ sh:path ex:address ; sh:node ex:AddressShape ] .";
        let report = store.validate_shacl(shapes).unwrap();
        assert!(!report.conforms, "Expected a sh:node violation because ex:addr1 has no ex:city");
        assert!(report
            .results
            .iter()
            .any(|r| r.source_constraint_component == crate::triples::Term::Iri(crate::triples::TermImpl {
                iri: crate::encoding::Encoder::get("<http://www.w3.org/ns/shacl#NodeConstraintComponent>").unwrap()
            })));
    }

    #[test]
    fn test_shacl_node_constraint_conforms() {
        let mut store = TripleStore::new();
        store
            .load_triples(
                "<http://example/bob> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example/Person> .\n<http://example/bob> <http://example/address> <http://example/addr2> .\n<http://example/addr2> <http://example/city> \"Springfield\" .",
                crate::parser::Syntax::NTriples,
            )
            .unwrap();

        let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example/> .
ex:AddressShape a sh:NodeShape ;
    sh:property [ sh:path ex:city ; sh:minCount 1 ] .
ex:PersonShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:property [ sh:path ex:address ; sh:node ex:AddressShape ] .";
        let report = store.validate_shacl(shapes).unwrap();
        assert!(report.conforms, "Expected conformance since ex:addr2 has an ex:city");
    }

    // -----------------------------------------------------------------------
    // ShEx integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_triplestore_validate_shex() {
        let mut store = TripleStore::new();
        store
            .load_triples(
                "<http://example.org/Alice> <http://example.org/age> \"30\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n<http://example.org/Bob> <http://example.org/age> \"thirty\" .",
                crate::parser::Syntax::NTriples,
            )
            .unwrap();

        let schema_json = r#"{
          "@context": "http://www.w3.org/ns/shex.jsonld",
          "type": "Schema",
          "shapes": [
            {
              "type": "ShapeDecl",
              "id": "http://example.org/AgeShape",
              "shapeExpr": {
                "type": "Shape",
                "expression": {
                  "type": "TripleConstraint",
                  "predicate": "http://example.org/age",
                  "valueExpr": {
                    "type": "NodeConstraint",
                    "datatype": "http://www.w3.org/2001/XMLSchema#integer"
                  }
                }
              }
            }
          ]
        }"#;

        let shape_map = vec![
            ("http://example.org/Alice".to_string(), "http://example.org/AgeShape".to_string()),
            ("http://example.org/Bob".to_string(), "http://example.org/AgeShape".to_string()),
        ];

        let report = store.validate_shex(schema_json, &shape_map).unwrap();
        assert!(!report.conforms, "Bob's non-integer age should fail the AgeShape");
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].shape, "http://example.org/AgeShape");
    }

    // -----------------------------------------------------------------------
    // Datalog stratification tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_datalog_safety_check_rejects_unsafe_rule() {
        use crate::datalog::validate_rules;
        let rules = vec![crate::Rule {
            body: vec![crate::BodyLiteral {
                negated: false,
                pattern: crate::Triple {
                    s: crate::VarOrTerm::new_var("?s".to_string()),
                    p: crate::VarOrTerm::new_term("<http://example/p>".to_string()),
                    o: crate::VarOrTerm::new_var("?o".to_string()),
                    g: None,
                },
            }],
            head: crate::Triple {
                s: crate::VarOrTerm::new_var("?s".to_string()),
                p: crate::VarOrTerm::new_term("<http://example/q>".to_string()),
                o: crate::VarOrTerm::new_var("?unbound".to_string()),
                g: None,
            },
        }];
        let result = validate_rules(&rules, &std::collections::HashMap::new());
        assert!(result.is_err(), "Unsafe rule (unbound head variable) should be rejected");
    }

    #[test]
    fn test_datalog_safe_rule_stratification() {
        use crate::datalog::validate_rules;
        let rules = vec![crate::Rule {
            body: vec![crate::BodyLiteral {
                negated: false,
                pattern: crate::Triple {
                    s: crate::VarOrTerm::new_var("?s".to_string()),
                    p: crate::VarOrTerm::new_term("<http://example/p>".to_string()),
                    o: crate::VarOrTerm::new_var("?o".to_string()),
                    g: None,
                },
            }],
            head: crate::Triple {
                s: crate::VarOrTerm::new_var("?s".to_string()),
                p: crate::VarOrTerm::new_term("<http://example/q>".to_string()),
                o: crate::VarOrTerm::new_var("?o".to_string()),
                g: None,
            },
        }];
        let result = validate_rules(&rules, &std::collections::HashMap::new());
        assert!(result.is_ok(), "Safe positive rule should stratify");
        assert_eq!(result.unwrap(), vec![0]);
    }

    // N3 / forward chaining integration
    #[test]
    fn test_n3_rules_forward_chaining() {
        let mut store = TripleStore::new();
        store
            .load_triples(
                "<http://example/alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example/Person> .",
                crate::parser::Syntax::NTriples,
            )
            .unwrap();
        store
            .load_rules(
                "{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example/Person>.} => {?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example/Agent>.}",
            )
            .unwrap();
        let new_triples = store.materialize();
        assert!(new_triples.len() >= 1, "Expected at least one inferred triple");
    }

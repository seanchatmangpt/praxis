#![cfg(test)]

use crate::reasoner::Reasoner;
use crate::{
    BodyLiteral, QueryEngine, Rule, RuleIndex, SimpleQueryEngine, Triple, TripleIndex, TripleStore,
    VarOrTerm,
};
use std::collections::HashMap;

// Regression for a real, swarm-identified crash: `sparql/plan.rs`'s
// `build_for_aggregate` returns `Err("Failed")` for GROUP_CONCAT/SAMPLE (its
// catch-all for aggregates it doesn't implement), and the caller in
// `extract_query_plan`'s `Group` arm unwraps that `Result`, panicking. Before
// `plan_query_or_refuse`, this crashed the calling thread outright for any
// caller of `TripleStore::query`. It must now surface as a typed `Err`.
#[test]
fn group_concat_aggregate_refuses_instead_of_panicking() {
    let data = "<http://example.org/a> <http://example.org/p> \"1\".\n\
                <http://example.org/a> <http://example.org/p> \"2\".";
    let store = TripleStore::from(data);
    let result = store.query(
        "SELECT (GROUP_CONCAT(?o) AS ?g) WHERE { ?s <http://example.org/p> ?o } GROUP BY ?s",
    );
    match result {
        Err(msg) => assert!(
            msg.contains("planning refused"),
            "expected a query-planning refusal message, got: {msg}"
        ),
        Ok(rows) => panic!("expected GROUP_CONCAT to be refused, got rows: {rows:?}"),
    }
}

// Regression for a real, swarm-identified crash: `sparql/mod.rs`'s `PlanNode::Aggregate`
// execution arm called `Encoder::get(&var_str).unwrap()` on a GROUP BY key variable that
// was never interned (never appears in the WHERE clause or SELECT projection), panicking
// on the first row evaluated -- a separate execution-phase panic from the
// group_concat_aggregate test above (planning-phase), so `plan_query_or_refuse`'s
// catch_unwind does not cover it: `evaluate_plan` runs after planning already returned a
// `PlanNode`, outside that boundary. Fixed by the same `unwrap_or_else(|| Encoder::add(...))`
// safe-fallback pattern already used one block above (mod.rs:168) for the aggregate target
// variable. Per SPARQL's own unbound-group-key semantics, grouping by a variable with no
// binding groups every row together under that missing key -- not a crash -- so this
// asserts real (non-panicking), correct output, not just that no crash occurs.
#[test]
fn group_by_unbound_variable_groups_instead_of_panicking() {
    let data = "<http://example.org/a> <http://example.org/p> \"1\".\n\
                <http://example.org/a> <http://example.org/p> \"2\".\n\
                <http://example.org/b> <http://example.org/p> \"3\".";
    let store = TripleStore::from(data);
    let result = store.query(
        "SELECT (COUNT(*) AS ?c) WHERE { ?s <http://example.org/p> ?o } GROUP BY ?neverBound",
    );
    match result {
        Ok(rows) => {
            assert_eq!(
                rows.len(),
                1,
                "grouping by an unbound variable must fold every row into one group, got: {rows:?}"
            );
            let count_binding = rows[0]
                .iter()
                .find(|b| b.var == "c")
                .expect("aggregate binding ?c must be present");
            // Expected value updated after fixing CountAccumulator::get() to intern its result
            // as a properly-typed xsd:integer literal (Encoder::add_literal) instead of letting
            // the generic Encoder::add() mis-classify the bare "3" as an IRI -- see the fix's own
            // commit message for the full root cause (HAVING/ORDER BY on an aggregate silently
            // failed since a mis-typed IRI can't be numerically compared). The bare-string
            // assertion this test originally had was itself asserting the bug's symptom.
            assert_eq!(
                count_binding.val, "\"3\"^^<http://www.w3.org/2001/XMLSchema#integer>",
                "COUNT(*) over all 3 triples must be 3, got: {rows:?}"
            );
        }
        Err(msg) => panic!("expected a real grouped result, got a refusal: {msg}"),
    }
}

// Regression for a real, dogfood-workflow-identified symptom: `SELECT ?s (COUNT(*) AS ?c)
// ... GROUP BY ?s HAVING (COUNT(*) > 1)` silently returned zero rows for data where a real
// group has count > 1. The originating audit's own root-cause claim (a `GraphPattern::Having`
// enum variant falling through `extract_query_plan`'s catch-all) was checked directly against
// spargebra 0.4.6's source and found to be wrong -- no such variant exists; `HAVING` desugars
// at parse time into `Filter(Group(...))`, both already-matched arms. The real, four-layer bug
// chain: (1) `CountAccumulator::get()` handed its result to the generic `Encoder::add()`
// classifier, which mis-tagged the bare numeric string as an IRI (any lexical form with no `?`/
// `_:`/`"` prefix falls into the IRI branch); (2) the aggregate operand-variable lookup in
// `sparql/mod.rs` used `Encoder::get(&var_str).unwrap_or(0)`, silently substituting a wrong-but-
// valid-looking symbol ID 0 instead of interning the variable; (3) `Utils::remove_literal_tags`
// only stripped surrounding quotes when a `^^datatype` suffix was present, so an untagged
// decoded literal like `"3"` passed through the FILTER's numeric-comparison path still quote-
// wrapped, failing `.parse::<f64>()` silently. Fixed (1) via `Encoder::add_literal` with an
// explicit `xsd:integer`/`xsd:decimal` datatype tag, (2) via the same safe
// `unwrap_or_else(|| Encoder::add(var_str))` fallback used elsewhere in this file, (3) by
// unconditionally stripping quotes from the pre-`^^`-split lexical form. A fourth, deeper gap
// remains and is intentionally NOT fixed here: `EncodedTerm` (sparql/eval.rs) has no decimal/
// float variant, so `xsd:decimal`-tagged SUM/MIN/MAX/AVG results still can't be numerically
// compared once routed through `PlanExpression::Variable`'s evaluator -- see that file's and
// `sparql/accumulators.rs`'s cross-referencing disclosure comments. COUNT is unaffected (its
// `xsd:integer` tag IS handled by that evaluator), so this HAVING/COUNT case is now genuinely
// fixed end-to-end; HAVING/ORDER BY on SUM/MIN/MAX/AVG remains future work.
#[test]
fn having_count_filters_correctly() {
    let data = "<http://example.org/a> <http://example.org/p> \"1\".\n\
                <http://example.org/a> <http://example.org/p> \"2\".\n\
                <http://example.org/a> <http://example.org/p> \"3\".\n\
                <http://example.org/b> <http://example.org/p> \"4\".";
    let store = TripleStore::from(data);
    let result = store
        .query(
            "SELECT ?s (COUNT(*) AS ?c) WHERE { ?s <http://example.org/p> ?o } GROUP BY ?s HAVING (COUNT(*) > 1)",
        )
        .expect("query must succeed");
    assert_eq!(result.len(), 1, "only ?s=a has count > 1, got: {result:?}");
}

#[test]
fn having_count_excludes_all_when_threshold_too_high() {
    let data = "<http://example.org/a> <http://example.org/p> \"1\".\n\
                <http://example.org/a> <http://example.org/p> \"2\".\n\
                <http://example.org/a> <http://example.org/p> \"3\".\n\
                <http://example.org/b> <http://example.org/p> \"4\".";
    let store = TripleStore::from(data);
    let result = store
        .query(
            "SELECT ?s (COUNT(*) AS ?c) WHERE { ?s <http://example.org/p> ?o } GROUP BY ?s HAVING (COUNT(*) > 10)",
        )
        .expect("query must succeed");
    assert_eq!(result.len(), 0, "no group has count > 10, got: {result:?}");
}

// SUM's own accumulated VALUE is now correct (the `remove_literal_tags` quote-stripping fix
// plus the accumulator's `Encoder::add_literal` type-tagging fix both apply here too). The
// displayed value is untagged ("6", not "\"6\"^^<...decimal>") because SPARQL requires an
// aggregate to be aliased ("AS ?total"), which routes the result through an Extend/rename step
// -- and that step's expression evaluator has no decimal/float `EncodedTerm` variant either, so
// it strips the datatype tag on the way out (the same disclosed, deferred gap noted above). This
// test asserts what the current architecture actually, honestly guarantees: the numeric VALUE is
// correct, not that HAVING/ORDER BY on it works yet.
#[test]
fn sum_over_untagged_literals_computes_the_real_value() {
    let data = "<http://example.org/a> <http://example.org/p> \"1\".\n\
                <http://example.org/a> <http://example.org/p> \"2\".\n\
                <http://example.org/a> <http://example.org/p> \"3\".\n\
                <http://example.org/b> <http://example.org/p> \"4\".";
    let store = TripleStore::from(data);
    let result = store
        .query("SELECT ?s (SUM(?o) AS ?total) WHERE { ?s <http://example.org/p> ?o } GROUP BY ?s")
        .expect("query must succeed");
    let a_row = result
        .iter()
        .find(|row| row.iter().any(|b| b.var == "s" && b.val.contains("/a")))
        .expect("row for ?s=a must exist");
    let total = a_row
        .iter()
        .find(|b| b.var == "total")
        .expect("total bound");
    assert_eq!(total.val, "\"6\"", "1+2+3=6, got: {result:?}");
}

#[test]
fn test_parse() {
    let data = ":a a :C0.\n\
            {?a a :C0}=>{?a a :C1}\n\
            {?a a :C1}=>{?a a :C2}\n\
            {?a a :C2}=>{?a a :C3}";

    let mut store = TripleStore::from(data);

    let mat = store.materialize().unwrap();
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
        hooks: Vec::new(),
        receipts: Vec::new(),
        additions: Vec::new(),
        removals: Vec::new(),
        verdicts: Vec::new(),
    };

    store.materialize().unwrap();
    let elapsed = timer.elapsed();

    let result = SimpleQueryEngine::query(
        &store.triple_index,
        &Vec::from([BodyLiteral {
            negated: false,
            pattern: query,
        }]),
        None,
    );

    println!("Processed in: {:.2?}", elapsed);
    println!("Result: {:?}", result);
}

#[test]
fn test_incomplete_rule_match() {
    let data = ":a in :b.\n\
            {?a in ?b. ?b in ?c}=>{?a in ?c.}";

    let mut store = TripleStore::from(data);
    assert_eq!(1, store.len());
    store.materialize().unwrap();
    assert_eq!(1, store.len());
}
#[test]
fn test_no_var_query() {
    let data = ":a in :b.\n\
            {:a in :b}=>{:a in :c}";

    let mut store = TripleStore::from(data);
    assert_eq!(1, store.len());
    store.materialize().unwrap();
    assert_eq!(2, store.len());
}
#[test]
fn test_single_rule() {
    let data = ":a a :A.\n\
            {?a a :A}=>{?a a :B}";

    let mut store = TripleStore::from(data);
    assert_eq!(1, store.len());
    store.materialize().unwrap();
    assert_eq!(2, store.len());
}
#[test]
fn test_multiple_rule() {
    let data = ":a a :A.\n\
            {?a a :A}=>{?a a :B}\n\
            {?a a :B}=>{?a a :C}";

    let mut store = TripleStore::from(data);
    assert_eq!(1, store.len());
    store.materialize().unwrap();
    assert_eq!(3, store.len());
}
#[test]
fn test_join_rule() {
    let data = ":a a :A.\n\
            :a in :b.\n\
            {?a a :A.?a in ?o}=>{?a a :B}";

    let mut store = TripleStore::from(data);
    assert_eq!(2, store.len());
    store.materialize().unwrap();
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
    store.materialize().unwrap();
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
    store.materialize().unwrap();
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
    let inferred = store.materialize().unwrap();
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
    let inferred = store.materialize().unwrap();
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

// Regression for a real, swarm-identified determinism bug: `shacl::report::Validator::validate`
// (`report.rs`) iterated its `shape_nodes`/`focus_nodes` sets -- both `std::collections::
// HashSet<usize>`, whose default `RandomState` hasher is reseeded from OS entropy once per
// process -- directly, so byte-identical input produced differently-ordered `results` across
// separate process runs (the swarm's own scenario: the same `cng` workday benchmark run twice as
// two OS processes). That cross-process nondeterminism cannot be reproduced by a single-process
// unit test (within one process, `RandomState`'s cached per-thread seed is reused across
// `HashSet::new()` calls, so pre-fix code already happened to look consistent test-to-test here)
// -- disclosed, not smuggled: this test instead asserts the two properties actually verifiable
// in-process: (1) validating the identical input repeatedly always returns the identical,
// byte-equal `results` order (this codebase's own established determinism-verification method,
// `.claude/rules/rust-agi-core-team.md` rule 1: "run N times, diff outputs"), and (2) with >= 2
// distinct violations present, both are captured (count is unaffected by the ordering fix).
#[test]
fn test_shacl_validate_is_deterministic_across_repeated_calls() {
    let mut store = TripleStore::new();
    store
        .load_triples(
            "<http://example/det-shacl-alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example/det-shacl-Person> .\n\
             <http://example/det-shacl-bob> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example/det-shacl-Person> .",
            crate::parser::Syntax::NTriples,
        )
        .unwrap();

    let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example/> .
ex:DetShaclPersonShape a sh:NodeShape ;
    sh:targetClass ex:det-shacl-Person ;
    sh:property [ sh:path ex:det-shacl-name ; sh:minCount 1 ] .";

    let first = store.validate_shacl(shapes).unwrap();
    assert!(
        !first.conforms,
        "both instances lack ex:det-shacl-name; expected minCount violations"
    );
    assert_eq!(
        first.results.len(),
        2,
        "both alice and bob must be reported, got: {:?}",
        first.results
    );

    for _ in 0..4 {
        let repeat = store.validate_shacl(shapes).unwrap();
        assert_eq!(
            repeat.results, first.results,
            "validating identical input must return results in the identical order every time"
        );
    }
}

// Regression for swarm finding #23 (same bug class as #22 above, different file): `shacl::
// validate::validate_shape_closed_and_targets_tail`'s `sh:closed` check iterated
// `data.spo.get(&focus_node)`'s `FxHashMap<usize, ...>` keys directly, so with >= 2 forbidden
// predicates present on one focus node, `results` push order depended on that map's bucket
// layout rather than a canonical order. Fixed the same way as #22: collect and
// `sort_unstable()` the predicate IDs before iterating. Disclosed, not smuggled: `FxHashMap`
// uses a deterministic (non-randomly-seeded) hasher, unlike `std::collections::HashSet`'s
// default `RandomState` (finding #22's cross-process issue) -- but its iteration order can
// still depend on insertion sequence via resize/rehash history, so two logically-identical but
// differently-*constructed* `TripleIndex`es (e.g. two independent loaders) were not
// guaranteed the same order even within one process. Same in-process verification limits as
// #22's test apply: this asserts repeated-call determinism and correct violation count, the
// properties actually checkable without reconstructing the index two different ways.
#[test]
fn test_shacl_closed_shape_violation_order_is_deterministic() {
    let mut store = TripleStore::new();
    store
        .load_triples(
            "<http://example/det-closed-alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example/det-closed-Person> .\n\
             <http://example/det-closed-alice> <http://example/det-closed-extra-a> \"x\" .\n\
             <http://example/det-closed-alice> <http://example/det-closed-extra-b> \"y\" .",
            crate::parser::Syntax::NTriples,
        )
        .unwrap();

    let shapes = "@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix ex: <http://example/> .
ex:DetClosedPersonShape a sh:NodeShape ;
    sh:targetClass ex:det-closed-Person ;
    sh:closed true ;
    sh:ignoredProperties (rdf:type) .";

    let first = store.validate_shacl(shapes).unwrap();
    assert!(
        !first.conforms,
        "both extra predicates are outside the closed shape"
    );
    assert_eq!(
        first.results.len(),
        2,
        "both det-closed-extra-a and det-closed-extra-b must be reported, got: {:?}",
        first.results
    );

    for _ in 0..4 {
        let repeat = store.validate_shacl(shapes).unwrap();
        assert_eq!(
            repeat.results, first.results,
            "validating identical input must return sh:closed violations in the identical order every time"
        );
    }
}

// Regression for swarm finding #24 (same bug class as #22/#23, ShEx's CLOSED shape check
// instead of SHACL's sh:closed): `shex_native::validate_node`'s CLOSED-shape branch iterated
// `data.spo.get(&focus)`'s `FxHashMap<usize, ...>` keys directly, joining one "CLOSED shape:
// predicate ..." message per extra predicate into a single `ShexValidationFailure.reason`
// string via `errors.join("; ")` -- so with >= 2 extra predicates, the joined message order
// depended on FxHashMap bucket layout. Fixed the same way as #22/#23: sort the predicate IDs
// before iterating. Same in-process verification limits apply: asserts repeated-call
// determinism of the joined `reason` string, not a synthetic two-differently-built-index
// comparison.
#[test]
fn test_shex_closed_shape_violation_order_is_deterministic() {
    let mut store = TripleStore::new();
    store
        .load_triples(
            "<http://example.org/det-shex-alice> <http://example.org/det-shex-extra-a> \"x\" .\n\
             <http://example.org/det-shex-alice> <http://example.org/det-shex-extra-b> \"y\" .",
            crate::parser::Syntax::NTriples,
        )
        .unwrap();

    let schema_json = r#"{
          "@context": "http://www.w3.org/ns/shexj.jsonld",
          "type": "Schema",
          "shapes": [
            {
              "type": "ShapeDecl",
              "id": "http://example.org/DetShexClosedShape",
              "shapeExpr": {
                "type": "Shape",
                "closed": true,
                "expression": null
              }
            }
          ]
        }"#;

    let shape_map = vec![(
        "http://example.org/det-shex-alice".to_string(),
        "http://example.org/DetShexClosedShape".to_string(),
    )];

    let first = store.validate_shex(schema_json, &shape_map).unwrap();
    assert!(
        !first.conforms,
        "both extra predicates violate the closed shape with no expression/EXTRA"
    );
    assert_eq!(
        first.failures.len(),
        1,
        "one failure for the one (node, shape) pair"
    );
    assert!(
        first.failures[0].reason.contains("det-shex-extra-a")
            && first.failures[0].reason.contains("det-shex-extra-b"),
        "reason must name both extra predicates, got: {:?}",
        first.failures[0].reason
    );

    for _ in 0..4 {
        let repeat = store.validate_shex(schema_json, &shape_map).unwrap();
        assert_eq!(
            repeat.failures[0].reason, first.failures[0].reason,
            "validating identical input must return the CLOSED-shape reason string in the identical order every time"
        );
    }
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
    assert!(
        !triples.is_empty(),
        "Report should serialise to RDF triples"
    );
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
    assert!(
        !report.conforms,
        "Expected a sh:node violation because ex:addr1 has no ex:city"
    );
    assert!(report.results.iter().any(|r| r.source_constraint_component
        == crate::triples::Term::Iri(crate::triples::TermImpl {
            iri: crate::encoding::Encoder::get(
                "<http://www.w3.org/ns/shacl#NodeConstraintComponent>"
            )
            .unwrap()
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
    assert!(
        report.conforms,
        "Expected conformance since ex:addr2 has an ex:city"
    );
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
        (
            "http://example.org/Alice".to_string(),
            "http://example.org/AgeShape".to_string(),
        ),
        (
            "http://example.org/Bob".to_string(),
            "http://example.org/AgeShape".to_string(),
        ),
    ];

    let report = store.validate_shex(schema_json, &shape_map).unwrap();
    assert!(
        !report.conforms,
        "Bob's non-integer age should fail the AgeShape"
    );
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
    assert!(
        result.is_err(),
        "Unsafe rule (unbound head variable) should be rejected"
    );
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

// Regression for a real, `docs/jira/v26.7.12/REMAINING_WORK.md`-flagged correctness bug: the
// Bellman-Ford stratification loop's `iteration` counter always executes its loop body at
// least once (`changed` starts `true`), so on an empty ruleset (`num_predicates == 0`) it
// still increments `iteration` to 1 over zero edges, then the post-loop cycle check
// (`iteration > num_predicates`, i.e. `1 > 0`) spuriously reports a stratification cycle for
// input that has no rules -- and therefore cannot have a cycle. `crown_local.rs` already works
// around this by requiring a non-empty rule pack (a harmless-non-firing rule instead of a true
// empty one); this test asserts the direct, unworked-around call is now correct.
#[test]
fn test_datalog_empty_ruleset_is_trivially_stratifiable() {
    use crate::datalog::validate_rules;
    let result = validate_rules(&[], &std::collections::HashMap::new());
    assert_eq!(
        result,
        Ok(Vec::new()),
        "an empty ruleset has no predicates and no cycle to detect; it must not be refused"
    );
}

// Same bug, exercised through its actual production caller: `TripleStore::add_rules` was
// reachable by this false refusal any time a caller extended an already-empty ruleset with
// another empty batch (e.g. `TripleStore::new()` followed by `add_rules(vec![])`) --
// `?`-propagated as a real, user-visible `Err`, unlike `TripleStore::from`'s constructor path,
// which happened to survive the bug only because it silently discards `validate_rules`'s
// result via `if let Ok(...)`.
#[test]
fn test_add_rules_with_empty_ruleset_does_not_refuse() {
    let mut store = TripleStore::new();
    let result = store.add_rules(Vec::new());
    assert!(
        result.is_ok(),
        "extending an empty ruleset with zero new rules must not be refused, got: {result:?}"
    );
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
    let new_triples = store.materialize().unwrap();
    assert!(
        !new_triples.is_empty(),
        "Expected at least one inferred triple"
    );
}

// -----------------------------------------------------------------------
// Knowledge Hook Pack & Registry / Gating Tests (Milestone M1)
// -----------------------------------------------------------------------

fn create_temp_pack_dir(name: &str, toml: &str, ttl: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("praxis_test_pack_{}_{}", name, id));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("pack.toml"), toml).unwrap();
    std::fs::write(dir.join("ontology.ttl"), ttl).unwrap();
    dir
}

#[test]
fn test_load_hook_pack_valid() {
    let toml = r#"[pack]
name = "valid-pack"
version = "1.0.0"
description = "A valid test pack"
required_dialects = ["delta"]
"#;
    let ttl = r#"
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/> .

ex:h1 a kh:Hook ;
    kh:name "h1" ;
    kh:on "assert" ;
    kh:kind "delta" ;
    kh:var "v" ;
    kh:effect "refuse" ;
    kh:reason "refused by test" ;
    kh:priority 3 .
"#;
    let pack_dir = create_temp_pack_dir("valid", toml, ttl);
    let mut store = TripleStore::new();
    let res = store.load_hook_pack(&pack_dir);
    assert!(res.is_ok(), "Failed to load valid hook pack: {:?}", res);
    assert_eq!(store.hooks.len(), 1);
    assert_eq!(store.hooks[0].name, "h1");
    assert_eq!(store.hooks[0].priority, 3);

    // Cleanup
    let _ = std::fs::remove_dir_all(pack_dir);
}

#[test]
fn test_load_hook_pack_unsupported_dialect() {
    let toml = r#"[pack]
name = "unsupported-dialect"
version = "1.0.0"
description = "Dialect test"
required_dialects = ["unsupported-dialect"]
"#;
    let ttl = "";
    let pack_dir = create_temp_pack_dir("dialect", toml, ttl);
    let mut store = TripleStore::new();
    let res = store.load_hook_pack(&pack_dir);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("unsupported dialect"));
    let _ = std::fs::remove_dir_all(pack_dir);
}

#[test]
fn test_load_hook_pack_forbidden_keyword() {
    let toml = r#"[pack]
name = "forbidden-keyword"
version = "1.0.0"
description = "test"
required_dialects = ["delta"]
"#;
    // Tries to sneak a command/shell execution via reason string
    let ttl = r#"
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/> .

ex:h1 a kh:Hook ;
    kh:name "h1" ;
    kh:on "assert" ;
    kh:kind "delta" ;
    kh:var "v" ;
    kh:effect "refuse" ;
    kh:reason "let us run exec or curl shell" .
"#;
    let pack_dir = create_temp_pack_dir("forbidden", toml, ttl);
    let mut store = TripleStore::new();
    let res = store.load_hook_pack(&pack_dir);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("forbidden keyword"));
    let _ = std::fs::remove_dir_all(pack_dir);
}

#[test]
fn test_load_hook_pack_forbidden_predicate() {
    let toml = r#"[pack]
name = "forbidden-predicate"
version = "1.0.0"
description = "test"
required_dialects = ["delta"]
"#;
    let ttl = r#"
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/> .

ex:h1 a kh:Hook ;
    kh:name "h1" ;
    kh:kind "delta" ;
    kh:var "v" ;
    kh:effect "refuse" ;
    kh:reason "r" ;
    kh:forbiddenField "malicious" .
"#;
    let pack_dir = create_temp_pack_dir("forbidden_pred", toml, ttl);
    let mut store = TripleStore::new();
    let res = store.load_hook_pack(&pack_dir);
    assert!(
        res.is_err(),
        "Should have failed due to forbidden predicate or SHACL violation"
    );
    let _ = std::fs::remove_dir_all(pack_dir);
}

#[test]
fn test_load_hook_pack_closed_shape_violation() {
    let toml = r#"[pack]
name = "closed-violation"
version = "1.0.0"
description = "test"
required_dialects = ["delta"]
"#;
    // Uses kh:command which is forbidden by the closed shape validation
    let ttl = r#"
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/> .

ex:h1 a kh:Hook ;
    kh:name "h1" ;
    kh:kind "delta" ;
    kh:var "v" ;
    kh:effect "refuse" ;
    kh:reason "r" ;
    kh:command "malicious_command" .
"#;
    let pack_dir = create_temp_pack_dir("closed_viol", toml, ttl);
    let mut store = TripleStore::new();
    let res = store.load_hook_pack(&pack_dir);
    assert!(
        res.is_err(),
        "Should have failed SHACL validation due to sh:closed"
    );
    assert!(res.unwrap_err().contains("SHACL validation failed"));
    let _ = std::fs::remove_dir_all(pack_dir);
}

#[test]
fn test_load_hook_pack_action_closed_shape_violation() {
    let toml = r#"[pack]
name = "action-violation"
version = "1.0.0"
description = "test"
required_dialects = ["delta"]
"#;
    // Action has kh:shell property which is not allowed
    let ttl = r#"
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/> .

ex:h1 a kh:Hook ;
    kh:name "h1" ;
    kh:kind "delta" ;
    kh:var "v" ;
    kh:effect "ground-action" ;
    kh:action ex:act1 .

ex:act1 a kh:Action ;
    kh:handler ex:hnd1 ;
    kh:shell "malicious_shell" .
"#;
    let pack_dir = create_temp_pack_dir("action_viol", toml, ttl);
    let mut store = TripleStore::new();
    let res = store.load_hook_pack(&pack_dir);
    assert!(
        res.is_err(),
        "Should have failed SHACL validation due to closed Action shape"
    );
    assert!(res.unwrap_err().contains("SHACL validation failed"));
    let _ = std::fs::remove_dir_all(pack_dir);
}

#[test]
fn test_hook_pack_topological_sorting() {
    let toml = r#"[pack]
name = "sorting"
version = "1.0.0"
description = "test"
required_dialects = ["delta"]
"#;
    let ttl = r#"
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/> .

ex:h1 a kh:Hook ;
    kh:name "h1" ;
    kh:kind "delta" ;
    kh:var "v" ;
    kh:effect "refuse" ;
    kh:reason "r" ;
    kh:priority 5 ;
    kh:after ex:h2 .

ex:h2 a kh:Hook ;
    kh:name "h2" ;
    kh:kind "delta" ;
    kh:var "v" ;
    kh:effect "refuse" ;
    kh:reason "r" ;
    kh:priority 2 .
"#;
    let pack_dir = create_temp_pack_dir("sorting", toml, ttl);
    let mut store = TripleStore::new();
    store.load_hook_pack(&pack_dir).unwrap();
    assert_eq!(store.hooks.len(), 2);
    assert_eq!(store.hooks[0].name, "h2");
    assert_eq!(store.hooks[1].name, "h1");
    let _ = std::fs::remove_dir_all(pack_dir);
}

#[test]
fn test_hook_pack_cycle_detection() {
    let toml = r#"[pack]
name = "cycle"
version = "1.0.0"
description = "test"
required_dialects = ["delta"]
"#;
    let ttl = r#"
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <http://example.org/> .

ex:h1 a kh:Hook ;
    kh:name "h1" ;
    kh:kind "delta" ;
    kh:var "v" ;
    kh:effect "refuse" ;
    kh:reason "r" ;
    kh:after ex:h2 .

ex:h2 a kh:Hook ;
    kh:name "h2" ;
    kh:kind "delta" ;
    kh:var "v" ;
    kh:effect "refuse" ;
    kh:reason "r" ;
    kh:after ex:h1 .
"#;
    let pack_dir = create_temp_pack_dir("cycle", toml, ttl);
    let mut store = TripleStore::new();
    let res = store.load_hook_pack(&pack_dir);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("dependency cycle"));
    let _ = std::fs::remove_dir_all(pack_dir);
}

#[test]
fn test_fibo_blank_nodes_typed_literals_collections() {
    let fibo_ttl = r#"
@prefix ex: <http://example.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:person1 ex:hasName _:b1 .
_:b1 rdf:type ex:BlankNode ;
    ex:firstName "John"@en .

ex:amount "1000"^^<http://www.w3.org/2001/XMLSchema#integer> .

ex:list1 rdf:value (1 2 3) .
    "#;

    let store = TripleStore::from(fibo_ttl);
    eprintln!("Loaded {} triples from FIBO TTL", store.len());
    for triple in store.triple_index.triples.iter().take(5) {
        eprintln!("{:?}", triple);
    }
    assert!(store.len() > 0, "Should load at least some triples");
}

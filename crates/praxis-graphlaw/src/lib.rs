extern crate core;
pub mod backwardchaining;
pub mod bindings;
pub mod builtins;
pub mod csprite;
pub mod decode;
pub mod dred;
pub mod encoding;
pub mod imars_reasoner;
pub mod imars_window;
pub mod observer;
pub mod parser;
pub mod pipeline;
pub mod queryengine;
pub mod reasoner;
pub mod rsp;
pub mod ruleindex;
mod service_composition;
pub mod sparql;
pub mod time_window;
pub mod tripleindex;
pub mod oxrdf_adapter;
pub mod registry;
pub mod rule;
pub mod term;
pub mod triples;
pub mod utils;
pub mod aggregation;
pub mod shacl;
pub mod shex;
pub mod shex_native;
pub mod shexc_parser;
pub mod datalog;

extern crate pest;
#[macro_use]
extern crate pest_derive;
use crate::bindings::Binding;
use crate::ruleindex::RuleIndex;
use crate::tripleindex::TripleIndex;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::rc::Rc;

use log::{info, trace, warn}; // Use log crate when building application

use crate::backwardchaining::BackwardChainer;
use crate::encoding::Encoder;
use crate::parser::{Parser, Syntax};
use spargebra::Query;


use crate::queryengine::{QueryEngine, SimpleQueryEngine};
use crate::reasoner::Reasoner;
use crate::sparql::{eval_query, evaluate_plan_and_debug};
use crate::triples::{BlankNodeImpl, LiteralImpl, Rule, Term, TermImpl, Triple, VarOrTerm, BodyLiteral, Aggregate}; // Workaround to use prinltn! for logs.

pub struct TripleStore {
    pub rules: Vec<Rule>,
    pub rules_index: RuleIndex,
    pub triple_index: TripleIndex,
    pub reasoner: Reasoner,
    pub aggregates: HashMap<Rule, Aggregate>,
    pub strata: Vec<usize>,
}
unsafe impl Send for TripleStore {}

impl TripleStore {
    pub fn new() -> TripleStore {
        TripleStore {
            rules: Vec::new(),
            rules_index: RuleIndex::new(),
            triple_index: TripleIndex::new(),
            reasoner: Reasoner {},
            aggregates: HashMap::new(),
            strata: Vec::new(),
        }
    }
    pub fn from(data: &str) -> TripleStore {
        // Route through the unified pest-based N3 parser only for documents
        // that actually declare `@prefix` -- i.e. that opt in to real N3/
        // Turtle-style prefix semantics. That parser understands @prefix
        // declarations, RDF lists ("(...)"), and quoted graphs ("{...}" used
        // as a term), none of which the legacy line-splitting parser can
        // represent at all.
        //
        // We do NOT unconditionally prefer the pest parser whenever it
        // happens to also accept a `@prefix`-less document, because the two
        // parsers intentionally disagree on two long-standing legacy
        // conventions that pre-date @prefix support and that other tests
        // depend on:
        //   - the pest grammar expands bare "a" to the full rdf:type IRI
        //     (required by parse_rule_with_a_syntactic_sugar), whereas the
        //     legacy parser keeps "a" as a literal, unexpanded token;
        //   - an undeclared "prefix:local" name is left as opaque raw text
        //     by both parsers, but only once neither one further wraps it.
        // Some existing tests (e.g. CSprite's) build comparison triples by
        // hand using the legacy convention (raw "a", raw "prefix:local")
        // against `@prefix`-free input; silently switching those documents
        // to pest's rdf:type-expanding behavior would desync the two. Since
        // every document that actually needs list/formula/@prefix support
        // (this crate's own N3 built-in tests included) declares `@prefix`,
        // gating on its presence is a precise, low-risk way to opt in.
        let (content, rules) = if data.contains("@prefix") {
            match Parser::parse_n3_document(data) {
                Ok(result) => result,
                Err(_) => Parser::parse(data.to_string()),
            }
        } else {
            Parser::parse(data.to_string())
        };
        let mut triple_index = TripleIndex::new();
        content.into_iter().for_each(|t| triple_index.add(t));
        let mut rules_index = RuleIndex::new();
        for rule in rules.iter() {
            rules_index.add_ref(rule);
        }
        let aggregates = HashMap::new();
        let mut strata = Vec::new();
        if let Ok(computed_strata) = datalog::validate_rules(&rules, &aggregates) {
            strata = computed_strata;
        }
        TripleStore {
            rules: rules,
            rules_index,
            triple_index,
            reasoner: Reasoner {},
            aggregates,
            strata,
        }
    }
    pub fn add(&mut self, triple: Triple) {
        trace! {"Adding triple: {:?}", Self::decode_triple(&triple) }
        self.triple_index.add(triple);
    }
    pub fn add_ref(&mut self, triple: Rc<Triple>) {
        trace! {"Adding triple: {:?}", Self::decode_triple(triple.as_ref()) }
        self.triple_index.add_ref(triple);
    }
    pub fn remove_ref(&mut self, triple: &Triple) {
        trace! {"Removing triple: {:?}", Self::decode_triple(triple) }
        self.triple_index.remove_ref(triple);
    }
    pub fn add_rules(&mut self, rules: Vec<Rule>) -> Result<(), String> {
        let mut all_rules = self.rules.clone();
        all_rules.extend(rules.clone());
        let strata = datalog::validate_rules(&all_rules, &self.aggregates)?;
        self.strata = strata;
        for rule in rules {
            self.rules.push(rule.clone());
            self.rules_index.add(rule);
        }
        Ok(())
    }
    pub fn add_rule_with_aggregate(&mut self, rule: Rule, aggregate: Aggregate) -> Result<(), String> {
        self.aggregates.insert(rule.clone(), aggregate);
        self.add_rules(vec![rule])
    }
    pub fn len(&self) -> usize {
        self.triple_index.len()
    }

    pub fn materialize(&mut self) -> Vec<Triple> {
        self.reasoner
            .materialize(&mut self.triple_index, &self.rules, &self.strata, &self.aggregates)
    }

    /// Check every denial/consistency-check rule (`{ body } => false.`,
    /// e.g. SKOS's disjointness constraints -- see `Rule::is_denial`)
    /// against the current facts, returning a human-readable description of
    /// each one whose body matches (a genuine constraint violation). Call
    /// this *after* `materialize()` has reached a fixpoint, so a violation
    /// that only becomes visible through a derived (not just asserted)
    /// fact is still caught -- `materialize()` itself never asserts
    /// anything for a denial rule (see `Reasoner::materialize`'s `is_denial`
    /// skip), it only derives the ordinary facts the check here then reads.
    pub fn check_denials(&self) -> Vec<String> {
        self.rules
            .iter()
            .filter(|r| r.is_denial())
            .filter_map(|r| {
                let bindings = SimpleQueryEngine::query(&self.triple_index, &r.body, None)?;
                if bindings.len() == 0 {
                    return None;
                }
                let body_desc: String = r
                    .body
                    .iter()
                    .map(|lit| {
                        let s = Self::decode_triples(&[lit.pattern.clone()]);
                        if lit.negated {
                            format!("not {{{}}}", s.trim())
                        } else {
                            s
                        }
                    })
                    .collect();
                Some(format!("DENIED: {{{}}} => false.", body_desc.trim()))
            })
            .collect()
    }

    /// Prove a fully ground goal triple (e.g. `5 :moreInterestingThan 3`)
    /// goal-directed against this store's rules + facts -- unlike
    /// `materialize()`, which forward-derives everything a stratum
    /// implies, this seeds a concrete query and works backward, so it can
    /// answer things forward-chaining alone cannot (e.g. `<=` rules with
    /// no ground facts to iterate candidate variable values over). See
    /// `backwardchaining::BackwardChainer::prove`.
    pub fn prove(&self, goal: &Triple) -> bool {
        // Thin ground-goal convenience wrapper around `solve`: a ground
        // goal is proved iff `solve` returns at least one binding row (for
        // a fully ground goal that row is always the empty row, since
        // there are no variables left to bind).
        !self.solve(goal).is_empty()
    }

    /// Full SLD-style resolution: prove `goal` (which may contain
    /// variables anywhere, including nested inside list terms) against
    /// this store's rules + facts, goal-directed, and return every
    /// binding row that satisfies it. See
    /// `backwardchaining::BackwardChainer::solve`.
    pub fn solve(&self, goal: &Triple) -> Vec<crate::Binding> {
        crate::backwardchaining::BackwardChainer::solve(&self.triple_index, &self.rules_index, goal)
    }

    //Backward chaining

    ////

    pub fn content_to_string(&self) -> String {
        let content = &self.triple_index.triples;
        TripleStore::decode_triples(content)
    }

    pub fn load_triples(&mut self, data: &str, syntax: Syntax) -> Result<(), String> {
        match Parser::parse_triples(data, syntax) {
            Ok(triples) => {
                triples.into_iter().for_each(|t| self.triple_index.add(t));
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
    pub fn load_rules(&mut self, data: &str) -> Result<(), String> {
        match Parser::parse_rules(data) {
            Ok(rules) => self.add_rules(rules),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn query(&self, query_str: &str) -> Result<Vec<Vec<crate::sparql::Binding>>, String> {
        match Query::parse(query_str, None) {
            Ok(query) => {
                let plan = eval_query(&query, &self.triple_index);
                Ok(evaluate_plan_and_debug(&plan, &self.triple_index).collect())
            }
            Err(err) => Err(format!("Unable to parse Query: {:?}", err.to_string())),
        }
    }

    /// Validate the triples in this store against a SHACL shapes graph.
    ///
    /// `shapes_turtle` is a Turtle-serialised SHACL shapes graph.
    /// Returns a `ValidationReport` describing conformance and any violations.
    pub fn validate_shacl(&self, shapes_turtle: &str) -> Result<crate::shacl::ValidationReport, String> {
        let shapes = crate::shacl::ShapesGraph::parse(shapes_turtle)?;
        Ok(crate::shacl::Validator::validate(&self.triple_index, &shapes))
    }

    /// Validate specific (focus-node, shape) pairs against a ShEx schema.
    ///
    /// `schema_json` is the ShExJ (JSON) serialisation of the ShEx schema.
    /// `shape_map` is a slice of `(focus_node_iri, shape_label_iri)` pairs.
    /// Returns a `ShexValidationReport` describing conformance.
    pub fn validate_shex(
        &self,
        schema_json: &str,
        shape_map: &[(String, String)],
    ) -> Result<crate::shex::ShexValidationReport, Box<dyn std::error::Error>> {
        crate::shex::validate_shex(&self.triple_index, schema_json, shape_map)
    }

    /// Validate specific (focus-node, shape) pairs against a ShEx schema
    /// given in **ShExC** (compact syntax) rather than ShExJ -- parses via
    /// `shexc_parser::parse_shexc` (an 80/20 subset, see that module's doc
    /// comment for scope) then delegates to the same validation path as
    /// `validate_shex`, so there is no separate/duplicated validation logic.
    pub fn validate_shex_c(
        &self,
        schema_shexc: &str,
        shape_map: &[(String, String)],
    ) -> Result<crate::shex_native::ShexValidationReport, String> {
        let schema = crate::shexc_parser::parse_shexc(schema_shexc)?;
        crate::shex_native::validate_shex_schema(&self.triple_index, &schema, shape_map)
    }
}

mod lib_test;

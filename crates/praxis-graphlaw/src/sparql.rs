use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Error;
use std::iter::empty;
use std::rc::Rc;
use std::sync::Mutex;
use spargebra::Query;
use spargebra::Query::Select;
use spargebra::algebra::*;
use spargebra::term::{TriplePattern, Variable};
use crate::{Encoder, Parser, Syntax, Term, TermImpl, Triple, TripleIndex, TripleStore, VarOrTerm};
use crate::sparql::EncodedTerm::NamedNode;
use crate::sparql::PlanNode::QuadPattern;
use crate::tripleindex::EncodedBinding;
use once_cell::sync::Lazy;
use crate::utils::Utils;


fn extract_triples(triple_patterns: &Vec<TriplePattern>, encoder: &mut Encoder)-> Vec<Triple>{
    let mut triples = Vec::new();
    for TriplePattern{subject: s , predicate: p,object:o } in triple_patterns{
        triples.push(Triple::from(s.to_string(),p.to_string(),o.to_string()));
    }
    triples
}
fn strip_variable_prefix(var: &str) -> &str {
    if var.starts_with('?') {
        &var[1..]
    } else {
        var
    }
}

#[derive(Debug)]
pub enum PlanExpression {
    Constant(Term),
    Variable(usize),
    Greater(Box<Self>, Box<Self>),
    GreaterOrEqual(Box<Self>, Box<Self>),
    Less(Box<Self>, Box<Self>),
    LessOrEqual(Box<Self>, Box<Self>),
    Equal(Box<Self>, Box<Self>),
    NotEqual(Box<Self>, Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Not(Box<Self>),
    Done,
}
#[derive(Debug)]
pub enum PlanNode {
    Join {
        left: Box<Self>,
        right: Box<Self>,
    },
    QuadPattern {
        pattern: Triple,
    },
    Project {
        child: Box<Self>,
        mapping: Vec<usize>,
    },
    Filter {
        child: Box<Self>,
        expression: Box<PlanExpression>,
    },
    Aggregate {
        // By definition the group by key are the range 0..key_mapping.len()
        child: Box<Self>,
        keys: Vec<Variable>, // aggregate key pairs of (variable key in child, variable key in output)
        aggregates: Rc<Vec<(PlanAggregation, Variable)>>,
    },
    Extend {
        child: Box<Self>,
        expression: PlanExpression,
        to: Variable,
    },
    LeftJoin {
        left: Box<Self>,
        right: Box<Self>,
        expression: Option<PlanExpression>,
    },
    Union {
        left: Box<Self>,
        right: Box<Self>,
    },
    Minus {
        left: Box<Self>,
        right: Box<Self>,
    },
    Done,
    /// The empty-BGP identity: yields exactly one solution with zero
    /// bindings (SPARQL algebra's "unit table"), so joining it with
    /// anything is a no-op. Distinct from `Done` (which yields ZERO
    /// solutions, i.e. "never matches") -- an empty `GraphPattern::Bgp`
    /// (e.g. the segment before a leading `BIND` in `{ BIND(...) ?s ?p
    /// ?o }`) must behave as "always matches trivially," not "never
    /// matches." Using `Done` here previously caused any query with a
    /// leading BIND (or other empty-BGP-adjacent construct) to silently
    /// produce zero rows -- found via the SHACL sh:sparql `$this`
    /// pre-binding fix, which injects exactly this kind of leading BIND.
    Unit,
}
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct PlanAggregation {
    pub function: PlanAggregationFunction,
    pub distinct: bool,
    pub variable: Option<Variable>,
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum PlanAggregationFunction {
    Count,
    Sum,
    Min,
    Max,
    Avg
}
fn new_join(left: PlanNode, right: PlanNode) -> PlanNode{
    PlanNode::Join {left:Box::new(left),right: Box::new(right)}
}
fn extract_query_plan(graph_pattern: &GraphPattern) -> PlanNode {
    match graph_pattern {
        GraphPattern::Bgp { patterns } => patterns
            .iter()
            .map(|t| QuadPattern {
                pattern: Triple::from(
                    t.subject.to_string(),
                    t.predicate.to_string(),
                    t.object.to_string(),
                ),
            })
            .reduce(new_join)
            .unwrap_or(PlanNode::Unit),
        GraphPattern::Join { left, right } => PlanNode::Join {
            left: Box::new(extract_query_plan(left)),
            right: Box::new(extract_query_plan(right)),
        },
        GraphPattern::Distinct { inner } | GraphPattern::Reduced { inner } => {
            extract_query_plan(inner)
        }
        GraphPattern::Project { inner, variables } => {
            let new_vars = variables
                .iter()
                .map(|v| {
                    let var_str = v.as_str().to_string();
                    let var_str = strip_variable_prefix(&var_str).to_string();
                    Encoder::add(var_str)
                })
                .collect();
            PlanNode::Project {
                child: Box::new(extract_query_plan(inner)),
                mapping: new_vars,
            }
        }
        GraphPattern::Filter { expr, inner } => {
            PlanNode::Filter {
                child: Box::new(extract_query_plan(inner)),
                expression: Box::new(extract_expression(expr)),
            }
        }
        GraphPattern::Group {
            inner,
            variables: by,
            aggregates,
        } => {
            let mut inner_variables = by.clone();

            PlanNode::Aggregate {
                child: Box::new(extract_query_plan(inner)),
                keys: inner_variables.clone(),
                aggregates: Rc::new(
                    aggregates
                        .iter()
                        .map(|(v, a)| {
                            Ok((
                                build_for_aggregate(a, &mut inner_variables).unwrap(),
                                v.clone(),
                            ))
                        })
                        .collect::<Result<Vec<_>, Error>>()
                        .unwrap(),
                ),
            }
        }
        GraphPattern::Extend {
            inner,
            expression,
            variable,
        } => {
            let to_str = variable.clone().into_string();
            let to_str = strip_variable_prefix(&to_str).to_string();
            Encoder::add(to_str);
            PlanNode::Extend {
                child: Box::new(extract_query_plan(inner)),
                expression: extract_expression(expression),
                to: variable.clone(),
            }
        }
        GraphPattern::LeftJoin {
            left,
            right,
            expression,
        } => {
            PlanNode::LeftJoin {
                left: Box::new(extract_query_plan(left)),
                right: Box::new(extract_query_plan(right)),
                expression: expression.as_ref().map(|expr| extract_expression(expr)),
            }
        }
        GraphPattern::Union { left, right } => {
            PlanNode::Union {
                left: Box::new(extract_query_plan(left)),
                right: Box::new(extract_query_plan(right)),
            }
        }
        GraphPattern::Minus { left, right } => {
            PlanNode::Minus {
                left: Box::new(extract_query_plan(left)),
                right: Box::new(extract_query_plan(right)),
            }
        }
        _ => PlanNode::Done,
    }
}
fn build_for_aggregate(
    aggregate: &AggregateExpression,
    variables: &mut Vec<Variable>,
) -> Result<PlanAggregation, String> {
    match aggregate {
        AggregateExpression::CountSolutions { distinct } => Ok(PlanAggregation {
            function: PlanAggregationFunction::Count,
            distinct: *distinct,
            variable: None,
        }),
        AggregateExpression::FunctionCall {
            name,
            expr,
            distinct,
        } => {
            let function = match name {
                AggregateFunction::Count => PlanAggregationFunction::Count,
                AggregateFunction::Sum => PlanAggregationFunction::Sum,
                AggregateFunction::Min => PlanAggregationFunction::Min,
                AggregateFunction::Max => PlanAggregationFunction::Max,
                AggregateFunction::Avg => PlanAggregationFunction::Avg,
                _ => return Err("Failed".to_string()),
            };
            let var = match expr {
                Expression::Variable(v) => Some(v.clone()),
                _ => None,
            };
            Ok(PlanAggregation {
                function,
                distinct: *distinct,
                variable: var,
            })
        }
    }
}

fn extract_expression(expression: &Expression) -> PlanExpression {
    match expression {
        Expression::Greater(a, b) => PlanExpression::Greater(
            Box::new(extract_expression(a)),
            Box::new(extract_expression(b)),
        ),
        Expression::GreaterOrEqual(a, b) => PlanExpression::GreaterOrEqual(
            Box::new(extract_expression(a)),
            Box::new(extract_expression(b)),
        ),
        Expression::Less(a, b) => PlanExpression::Less(
            Box::new(extract_expression(a)),
            Box::new(extract_expression(b)),
        ),
        Expression::LessOrEqual(a, b) => PlanExpression::LessOrEqual(
            Box::new(extract_expression(a)),
            Box::new(extract_expression(b)),
        ),
        Expression::Equal(a, b) => PlanExpression::Equal(
            Box::new(extract_expression(a)),
            Box::new(extract_expression(b)),
        ),

        Expression::And(a, b) => PlanExpression::And(
            Box::new(extract_expression(a)),
            Box::new(extract_expression(b)),
        ),
        Expression::Or(a, b) => PlanExpression::Or(
            Box::new(extract_expression(a)),
            Box::new(extract_expression(b)),
        ),
        Expression::Not(a) => PlanExpression::Not(
            Box::new(extract_expression(a)),
        ),
        Expression::Variable(var) => {
            let var_str = var.as_str().to_string();
            let var_str = strip_variable_prefix(&var_str).to_string();
            PlanExpression::Variable(Encoder::add(var_str))
        }
        Expression::Literal(value) => {
            let val = value.value().to_string();
            let datatype = format!("<{}>", value.datatype().as_str());
            let lang = value.language().map(|l| l.to_string());
            let id = Encoder::add_literal(val, Some(datatype), lang);
            let term = Encoder::decode_to_term(id).unwrap();
            PlanExpression::Constant(term)
        }
        // A bare IRI constant (e.g. the RHS of `BIND(<iri> AS ?x)`) had no
        // handling here and silently fell through to the `_` catch-all
        // below, compiling to `PlanExpression::Done` -- which made ANY
        // `BIND(<iri> AS ?var)` a no-op that never actually bound the
        // variable, breaking every downstream use of that variable. Found
        // via the SHACL sh:sparql `$this` pre-binding fix, which relies on
        // exactly this construct.
        Expression::NamedNode(iri) => {
            let id = Encoder::add(format!("<{}>", iri.as_str()));
            let term = Encoder::decode_to_term(id).unwrap();
            PlanExpression::Constant(term)
        }
        _ => PlanExpression::Done,
    }
}
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Binding{
    pub var: String,
    pub val: String
}

fn decode(input: &EncodedBinding) -> Binding {
    let mut var = Encoder::decode(&input.var).unwrap_or("".to_string());
    if var.starts_with('?') {
        var.remove(0);
    }
    Binding {
        var,
        val: Encoder::decode(&input.val).unwrap_or("".to_string()),
    }
}

pub fn evaluate_plan_and_debug<'a>(
    plan_node: &'a PlanNode,
    triple_index: &'a TripleIndex,
) -> Box<dyn Iterator<Item = Vec<Binding>> + 'a> {
    Box::new(
        evaluate_plan(plan_node, triple_index)
            .map(|v| v.into_iter().map(|b| decode(&b)).collect::<Vec<Binding>>()),
    )
}
pub fn evaluate_plan<'a>(
    plan_node: &'a PlanNode,
    triple_index: &'a TripleIndex,
) -> Box<dyn Iterator<Item = Vec<EncodedBinding>> + 'a> {
    match plan_node {
        PlanNode::QuadPattern { pattern: triple } => triple_index.query_help(&triple, None),
        PlanNode::Project { child, mapping } => {
            let child_it = evaluate_plan(child, triple_index);
            Box::new(child_it.map(move |binding| {
                let projection: Vec<EncodedBinding> = binding
                    .into_iter()
                    .filter(|b| mapping.contains(&b.var))
                    .collect();
                projection
            }))
        }
        PlanNode::Join { left, right } => {
            let left_results: Vec<Vec<EncodedBinding>> = evaluate_plan(left, triple_index).collect();
            let right_results: Vec<Vec<EncodedBinding>> = evaluate_plan(right, triple_index).collect();

            if left_results.is_empty() || right_results.is_empty() {
                return Box::new(empty());
            }

            let left_vars: HashSet<usize> = left_results[0].iter().map(|b| b.var).collect();
            let right_vars: HashSet<usize> = right_results[0].iter().map(|b| b.var).collect();
            let intersection: Vec<usize> = right_vars.iter().filter(|v| left_vars.contains(v)).cloned().collect();

            if intersection.is_empty() {
                let mut product = Vec::new();
                for l in &left_results {
                    for r in &right_results {
                        let mut merged = l.clone();
                        merged.extend(r.clone());
                        product.push(merged);
                    }
                }
                return Box::new(product.into_iter());
            }

            let mut hash: HashMap<Vec<usize>, Vec<Vec<EncodedBinding>>> = HashMap::new();
            for l in left_results {
                let mut key = Vec::with_capacity(intersection.len());
                let mut has_all = true;
                for &var in &intersection {
                    if let Some(b) = l.iter().find(|b| b.var == var) {
                        key.push(b.val);
                    } else {
                        has_all = false;
                        break;
                    }
                }
                if has_all {
                    hash.entry(key).or_default().push(l);
                }
            }

            let mut joined = Vec::new();
            for r in right_results {
                let mut key = Vec::with_capacity(intersection.len());
                let mut has_all = true;
                for &var in &intersection {
                    if let Some(b) = r.iter().find(|b| b.var == var) {
                        key.push(b.val);
                    } else {
                        has_all = false;
                        break;
                    }
                }
                if has_all {
                    if let Some(matching_lefts) = hash.get(&key) {
                        for l in matching_lefts {
                            let mut merged = r.clone();
                            for b_l in l {
                                if !merged.iter().any(|b_r| b_r.var == b_l.var) {
                                    merged.push(b_l.clone());
                                }
                            }
                            joined.push(merged);
                        }
                    }
                }
            }
            Box::new(joined.into_iter())
        }
        PlanNode::Filter { child, expression } => {
            let child = evaluate_plan(child, triple_index);
            let expression = eval_expression(expression);
            Box::new(child.filter(move |bindings| {
                expression(bindings).and_then(|term| to_bool(&term)).unwrap_or(false)
            }))
        }
        PlanNode::Aggregate {
            child,
            keys,
            aggregates,
        } => {
            let child = evaluate_plan(child, triple_index);


            let aggregate_vars: Vec<(PlanAggregation, usize)> = aggregates
                .iter()
                .map(|(agg_fn, agg_var)| {
                    let var_str = agg_var.as_str().to_string();
                    let var_str = strip_variable_prefix(&var_str).to_string();
                    let encoded = Encoder::get(&var_str).unwrap_or_else(|| Encoder::add(var_str));
                    (agg_fn.clone(), encoded)
                })
                .collect();

            let mut grouped_accumulators = Rc::new(RefCell::new(HashMap::<
                Vec<usize>,
                Vec<AccumulatorImpl>,
            >::default()));

            if keys.is_empty() {
                let default_accs: Vec<AccumulatorImpl> = aggregate_vars.iter().map(|(agg_fn, _)| {
                    match agg_fn.function {
                        PlanAggregationFunction::Count => {
                            AccumulatorImpl::Count(CountAccumulator::default())
                        }
                        PlanAggregationFunction::Sum => {
                            AccumulatorImpl::Sum(SumAccumulator::default())
                        }
                        PlanAggregationFunction::Min => {
                            AccumulatorImpl::Min(MinAccumulator::default())
                        }
                        PlanAggregationFunction::Max => {
                            AccumulatorImpl::Max(MaxAccumulator::default())
                        }
                        PlanAggregationFunction::Avg => {
                            AccumulatorImpl::Avg(AvgAccumulator::default())
                        }
                    }
                }).collect();
                grouped_accumulators.borrow_mut().insert(vec![], default_accs);
            }

            let local_group = grouped_accumulators.clone();
            let aggregate_vars_for_closure = aggregate_vars.clone();
            child.for_each(move |child_binding| {

                let key_values: Vec<usize> = keys
                    .iter()
                    .map(|v| {
                        let var_str = v.as_str().to_string();
                        let var_str = strip_variable_prefix(&var_str).to_string();
                        Encoder::get(&var_str).unwrap()
                    })
                    .collect();

                let mut converted_keys = Vec::with_capacity(key_values.len());
                for &key_val in &key_values {
                    if let Some(binding) = child_binding.iter().find(|b| b.var == key_val) {
                        converted_keys.push(binding.val);
                    }
                }


                let mut temp_acc = grouped_accumulators.borrow_mut();
                let accs = temp_acc.entry(converted_keys).or_insert_with(|| {
                    aggregate_vars_for_closure.iter().map(|(agg_fn, _)| {
                        match agg_fn.function {
                            PlanAggregationFunction::Count => {
                                AccumulatorImpl::Count(CountAccumulator::default())
                            }
                            PlanAggregationFunction::Sum => {
                                AccumulatorImpl::Sum(SumAccumulator::default())
                            }
                            PlanAggregationFunction::Min => {
                                AccumulatorImpl::Min(MinAccumulator::default())
                            }
                            PlanAggregationFunction::Max => {
                                AccumulatorImpl::Max(MaxAccumulator::default())
                            }
                            PlanAggregationFunction::Avg => {
                                AccumulatorImpl::Avg(AvgAccumulator::default())
                            }
                        }
                    }).collect()
                });

                for (i, acc) in accs.iter_mut().enumerate() {
                    let agg_fn = &aggregate_vars_for_closure[i].0;
                    let item_to_aggregate = if let Some(ref agg_var) = agg_fn.variable {
                        let var_str = agg_var.as_str().to_string();
                        let var_str = strip_variable_prefix(&var_str).to_string();
                        let encoded_agg_var = Encoder::get(&var_str).unwrap_or(0);
                        child_binding
                            .iter()
                            .find(|b| b.var == encoded_agg_var)
                            .map(|b| b.val)
                            .unwrap_or(0)
                    } else {
                        0
                    };
                    acc.add(item_to_aggregate);
                }

            });

            {
                let mut temp_acc = local_group.borrow_mut();
                let mut new_bindings = Vec::with_capacity(temp_acc.len());
                let key_values: Vec<usize> = keys
                    .iter()
                    .map(|v| {
                        let var_str = v.as_str().to_string();
                        let var_str = strip_variable_prefix(&var_str).to_string();
                        Encoder::get(&var_str).unwrap()
                    })
                    .collect();

                for (group_keys, group_values) in temp_acc.iter() {
                    let mut new_row = Vec::with_capacity(key_values.len() + aggregate_vars.len());
                    for (i, &key_val) in key_values.iter().enumerate() {
                        if let Some(&val) = group_keys.get(i) {
                            new_row.push(EncodedBinding {
                                var: key_val,
                                val,
                            });
                        }
                    }
                    for (i, acc) in group_values.iter().enumerate() {
                        let agg_var_encoded = aggregate_vars[i].1;
                        new_row.push(EncodedBinding {
                            var: agg_var_encoded,
                            val: acc.get(),
                        });
                    }
                    new_bindings.push(new_row);
                }

                Box::new(new_bindings.into_iter())
            }
        }
        PlanNode::Extend { child, expression, to } => {
            let child_it = evaluate_plan(child, triple_index);
            let expression_fn = eval_expression(expression);
            let to_str = to.as_str().to_string();
            let to_str = strip_variable_prefix(&to_str).to_string();
            let encoded_to = Encoder::add(to_str);
            Box::new(child_it.map(move |mut binding| {
                if let Some(term) = expression_fn(&binding) {
                    let val_id = encode_term(term);
                    binding.push(EncodedBinding {
                        var: encoded_to,
                        val: val_id,
                    });
                }
                binding
            }))
        }
        PlanNode::LeftJoin { left, right, expression } => {
            let left_results: Vec<Vec<EncodedBinding>> = evaluate_plan(left, triple_index).collect();
            let right_results: Vec<Vec<EncodedBinding>> = evaluate_plan(right, triple_index).collect();
            
            let mut joined = Vec::new();
            let filter_fn = expression.as_ref().map(|expr| eval_expression(expr));
            
            for l in left_results {
                let l_vars: HashSet<usize> = l.iter().map(|b| b.var).collect();
                let mut matched_any = false;
                
                for r in &right_results {
                    let mut compatible = true;
                    let mut intersection = Vec::new();
                    for b_r in r {
                        if l_vars.contains(&b_r.var) {
                            intersection.push(b_r.var);
                            let b_l = l.iter().find(|b| b.var == b_r.var).unwrap();
                            if b_l.val != b_r.val {
                                compatible = false;
                                break;
                            }
                        }
                    }
                    
                    if compatible {
                        let mut merged = l.clone();
                        for b_r in r {
                            if !intersection.contains(&b_r.var) {
                                merged.push(b_r.clone());
                            }
                        }
                        
                        let pass = if let Some(ref f) = filter_fn {
                            f(&merged).and_then(|term| to_bool(&term)).unwrap_or(false)
                        } else {
                            true
                        };
                        
                        if pass {
                            joined.push(merged);
                            matched_any = true;
                        }
                    }
                }
                
                if !matched_any {
                    joined.push(l);
                }
            }
            Box::new(joined.into_iter())
        }
        PlanNode::Union { left, right } => {
            let left_it = evaluate_plan(left, triple_index);
            let right_it = evaluate_plan(right, triple_index);
            Box::new(left_it.chain(right_it))
        }
        PlanNode::Minus { left, right } => {
            let left_results: Vec<Vec<EncodedBinding>> = evaluate_plan(left, triple_index).collect();
            let right_results: Vec<Vec<EncodedBinding>> = evaluate_plan(right, triple_index).collect();
            
            let mut remaining = Vec::new();
            for l in left_results {
                let l_vars: HashSet<usize> = l.iter().map(|b| b.var).collect();
                let mut filter_out = false;
                
                for r in &right_results {
                    let mut compatible = true;
                    let mut shared_any = false;
                    for b_r in r {
                        if l_vars.contains(&b_r.var) {
                            shared_any = true;
                            let b_l = l.iter().find(|b| b.var == b_r.var).unwrap();
                            if b_l.val != b_r.val {
                                compatible = false;
                                break;
                            }
                        }
                    }
                    
                    if compatible && shared_any {
                        filter_out = true;
                        break;
                    }
                }
                
                if !filter_out {
                    remaining.push(l);
                }
            }
            Box::new(remaining.into_iter())
        }
        PlanNode::Done => Box::new(empty()),
        PlanNode::Unit => Box::new(std::iter::once(Vec::new())),
    }
}
fn encode_term(term: EncodedTerm) -> usize {
    match term {
        EncodedTerm::BooleanLiteral(b) => {
            Encoder::add_literal(
                b.to_string(),
                Some("<http://www.w3.org/2001/XMLSchema#boolean>".to_string()),
                None
            )
        }
        EncodedTerm::IntegerLiteral(i) => {
            Encoder::add_literal(
                i.to_string(),
                Some("<http://www.w3.org/2001/XMLSchema#integer>".to_string()),
                None
            )
        }
        EncodedTerm::StringLiteral(s) => {
            Encoder::add_literal(s, None, None)
        }
        EncodedTerm::NamedNode { iri_id } => iri_id,
    }
}

trait Accumulator {
    fn add(&mut self, encoded_item: usize);
    fn get(&self) -> usize;
}

#[derive(Debug)]
pub struct CountAccumulator {
    count: usize,
}

impl Accumulator for CountAccumulator {
    fn add(&mut self, _item: usize) {
        self.count += 1;
    }
    fn get(&self) -> usize {
        Encoder::add(self.count.to_string())
    }
}

impl Default for CountAccumulator {
    fn default() -> Self {
        CountAccumulator { count: 0 }
    }
}

#[derive(Debug)]
pub struct SumAccumulator {
    sum: f64,
}

impl Accumulator for SumAccumulator {
    fn add(&mut self, item: usize) {
        if let Some(val) = Encoder::decode(&item) {
            let val = Utils::remove_literal_tags(&val);
            self.sum += val.parse::<f64>().unwrap_or(0.0);
        }
    }
    fn get(&self) -> usize {
        Encoder::add(self.sum.to_string())
    }
}

impl Default for SumAccumulator {
    fn default() -> Self {
        SumAccumulator { sum: 0.0 }
    }
}

#[derive(Debug)]
pub struct MinAccumulator {
    min: Option<f64>,
}

impl Accumulator for MinAccumulator {
    fn add(&mut self, item: usize) {
        if let Some(val) = Encoder::decode(&item) {
            let val = Utils::remove_literal_tags(&val);
            if let Ok(num) = val.parse::<f64>() {
                self.min = Some(self.min.map_or(num, |m| m.min(num)));
            }
        }
    }
    fn get(&self) -> usize {
        Encoder::add(self.min.map_or("0".to_string(), |m| m.to_string()))
    }
}

impl Default for MinAccumulator {
    fn default() -> Self {
        MinAccumulator { min: None }
    }
}

#[derive(Debug)]
pub struct MaxAccumulator {
    max: Option<f64>,
}

impl Accumulator for MaxAccumulator {
    fn add(&mut self, item: usize) {
        if let Some(val) = Encoder::decode(&item) {
            let val = Utils::remove_literal_tags(&val);
            if let Ok(num) = val.parse::<f64>() {
                self.max = Some(self.max.map_or(num, |m| m.max(num)));
            }
        }
    }
    fn get(&self) -> usize {
        Encoder::add(self.max.map_or("0".to_string(), |m| m.to_string()))
    }
}

impl Default for MaxAccumulator {
    fn default() -> Self {
        MaxAccumulator { max: None }
    }
}

#[derive(Debug)]
pub struct AvgAccumulator {
    sum: f64,
    count: usize,
}

impl Accumulator for AvgAccumulator {
    fn add(&mut self, item: usize) {
        if let Some(val) = Encoder::decode(&item) {
            let val = Utils::remove_literal_tags(&val);
            if let Ok(num) = val.parse::<f64>() {
                self.sum += num;
                self.count += 1;
            }
        }
    }
    fn get(&self) -> usize {
        let avg = if self.count > 0 {
            self.sum / self.count as f64
        } else {
            0.0
        };
        Encoder::add(avg.to_string())
    }
}

impl Default for AvgAccumulator {
    fn default() -> Self {
        AvgAccumulator { sum: 0.0, count: 0 }
    }
}

#[derive(Debug)]
pub enum AccumulatorImpl {
    Count(CountAccumulator),
    Sum(SumAccumulator),
    Min(MinAccumulator),
    Max(MaxAccumulator),
    Avg(AvgAccumulator),
}

impl Accumulator for AccumulatorImpl {
    fn add(&mut self, encoded_item: usize) {
        match self {
            Self::Count(acc) => acc.add(encoded_item),
            Self::Sum(acc) => acc.add(encoded_item),
            Self::Min(acc) => acc.add(encoded_item),
            Self::Max(acc) => acc.add(encoded_item),
            Self::Avg(acc) => acc.add(encoded_item),
        }
    }
    fn get(&self) -> usize {
        match self {
            Self::Count(acc) => acc.get(),
            Self::Sum(acc) => acc.get(),
            Self::Min(acc) => acc.get(),
            Self::Max(acc) => acc.get(),
            Self::Avg(acc) => acc.get(),
        }
    }
}
fn eval_expression<'a>(
    expression: &'a PlanExpression,
) -> Box<dyn Fn(&Vec<EncodedBinding>) -> Option<EncodedTerm> + 'a> {
    match expression {
        PlanExpression::Greater(a, b) => partial_compare_helper(a, b, Ordering::Greater, None),
        PlanExpression::Less(a, b) => partial_compare_helper(a, b, Ordering::Less, None),
        PlanExpression::GreaterOrEqual(a, b) => {
            partial_compare_helper(a, b, Ordering::Greater, Some(Ordering::Equal))
        }
        PlanExpression::LessOrEqual(a, b) => {
            partial_compare_helper(a, b, Ordering::Less, Some(Ordering::Equal))
        }
        PlanExpression::Equal(a, b) => {
            let eval_a = eval_expression(a);
            let eval_b = eval_expression(b);
            Box::new(move |bindings| {
                let val_a = eval_a(bindings)?;
                let val_b = eval_b(bindings)?;
                Some(EncodedTerm::BooleanLiteral(val_a == val_b))
            })
        }
        PlanExpression::NotEqual(a, b) => {
            let eval_a = eval_expression(a);
            let eval_b = eval_expression(b);
            Box::new(move |bindings| {
                let val_a = eval_a(bindings)?;
                let val_b = eval_b(bindings)?;
                Some(EncodedTerm::BooleanLiteral(val_a != val_b))
            })
        }
        PlanExpression::And(a, b) => {
            let eval_a = eval_expression(a);
            let eval_b = eval_expression(b);
            Box::new(move |bindings| {
                let term_a = eval_a(bindings)?;
                let term_b = eval_b(bindings)?;
                let bool_a = to_bool(&term_a)?;
                let bool_b = to_bool(&term_b)?;
                Some(EncodedTerm::BooleanLiteral(bool_a && bool_b))
            })
        }
        PlanExpression::Or(a, b) => {
            let eval_a = eval_expression(a);
            let eval_b = eval_expression(b);
            Box::new(move |bindings| {
                let term_a = eval_a(bindings);
                let term_b = eval_b(bindings);
                let bool_a = term_a.and_then(|t| to_bool(&t));
                let bool_b = term_b.and_then(|t| to_bool(&t));
                match (bool_a, bool_b) {
                    (Some(true), _) | (_, Some(true)) => Some(EncodedTerm::BooleanLiteral(true)),
                    (Some(false), Some(false)) => Some(EncodedTerm::BooleanLiteral(false)),
                    _ => None,
                }
            })
        }
        PlanExpression::Not(a) => {
            let eval_a = eval_expression(a);
            Box::new(move |bindings| {
                let term_a = eval_a(bindings)?;
                let bool_a = to_bool(&term_a)?;
                Some(EncodedTerm::BooleanLiteral(!bool_a))
            })
        }
        PlanExpression::Variable(v) => Box::new(move |bindings| {
            let var_value: Vec<&EncodedBinding> = bindings.iter().filter(|b| b.var == *v).collect();
            var_value
                .get(0)
                .and_then(|&binding| {
                    if let Some(term) = Encoder::decode_to_term(binding.val) {
                        match term {
                            Term::Iri(iri) => Some(EncodedTerm::NamedNode { iri_id: iri.iri }),
                            Term::Literal(lit) => {
                                let dt = lit.datatype.and_then(|dt_id| Encoder::decode(&dt_id));
                                let val_str = Encoder::decode(&lit.value).unwrap_or_default();
                                if let Some(dt_str) = dt {
                                    if dt_str == "<http://www.w3.org/2001/XMLSchema#integer>" {
                                        if let Ok(i) = val_str.parse::<i64>() {
                                            return Some(EncodedTerm::IntegerLiteral(i));
                                        }
                                    } else if dt_str == "<http://www.w3.org/2001/XMLSchema#boolean>" {
                                        return Some(EncodedTerm::BooleanLiteral(val_str == "true" || val_str == "1"));
                                    }
                                }
                                Some(EncodedTerm::StringLiteral(val_str))
                            }
                            Term::BlankNode(bnode) => Some(EncodedTerm::StringLiteral(format!("_:{}", bnode.id))),
                        }
                    } else {
                        Encoder::decode(&binding.val).map(EncodedTerm::StringLiteral)
                    }
                })
        }),
        PlanExpression::Constant(t) => {
            let t = t.clone();
            let encoded_term = match &t {
                Term::Literal(lit) => {
                    let dt = lit.datatype.and_then(|dt_id| Encoder::decode(&dt_id));
                    let val_str = Encoder::decode(&lit.value).unwrap_or_default();
                    if let Some(dt_str) = dt {
                        if dt_str == "<http://www.w3.org/2001/XMLSchema#integer>" {
                            if let Ok(i) = val_str.parse::<i64>() {
                                EncodedTerm::IntegerLiteral(i)
                            } else {
                                EncodedTerm::StringLiteral(val_str)
                            }
                        } else if dt_str == "<http://www.w3.org/2001/XMLSchema#boolean>" {
                            EncodedTerm::BooleanLiteral(val_str == "true" || val_str == "1")
                        } else {
                            EncodedTerm::StringLiteral(val_str)
                        }
                    } else {
                        EncodedTerm::StringLiteral(val_str)
                    }
                }
                Term::Iri(iri) => EncodedTerm::NamedNode { iri_id: iri.iri },
                Term::BlankNode(bnode) => EncodedTerm::StringLiteral(format!("_:{}", bnode.id)),
            };
            Box::new(move |_| Some(encoded_term.clone()))
        }
        _ => Box::new(|_| Some(EncodedTerm::BooleanLiteral(false))),
    }
}

fn partial_compare_helper<'a>(
    a: &'a Box<PlanExpression>,
    b: &'a Box<PlanExpression>,
    ordering: Ordering,
    second_order: Option<Ordering>,
) -> Box<dyn Fn(&Vec<EncodedBinding>) -> Option<EncodedTerm> + 'a> {
    let a = eval_expression(a);
    let b = eval_expression(b);

    Box::new(move |bindings| {
        let b_res = b(bindings);

        let r: Option<Ordering> = match a(bindings) {
            Some(EncodedTerm::IntegerLiteral(int_val_a)) => match b_res {
                Some(EncodedTerm::IntegerLiteral(int_val_b)) => {
                    int_val_a.partial_cmp(&int_val_b).into()
                }
                _ => None,
            },
            Some(EncodedTerm::StringLiteral(str_val_a)) => match b(bindings) {
                Some(EncodedTerm::StringLiteral(str_val_b)) => str_val_a.partial_cmp(&str_val_b),
                _ => None,
            },
            _ => None,
        };
        if let Some(r) = r {
            if let Some(second_ordering) = second_order {
                if r == ordering || r == second_ordering {
                    Some(true.into())
                } else {
                    Some(false.into())
                }
            } else {
                Some((r == ordering).into())
            }
        } else {
            Some(false.into())
        }
    })
}

fn to_bool(term: &EncodedTerm) -> Option<bool> {
    match term {
        EncodedTerm::BooleanLiteral(value) => Some(*value),
        EncodedTerm::StringLiteral(value) => Some(!value.is_empty()),
        EncodedTerm::IntegerLiteral(value) => Some(*value != 0),
        _ => None,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EncodedTerm {
    NamedNode { iri_id: usize },
    StringLiteral(String),
    // Signed: xsd:integer literals (and therefore SPARQL FILTER numeric
    // comparisons) must support negative values. This was previously
    // `usize`, silently failing `val_str.parse::<i64>()` for any
    // negative literal (e.g. "-5") and falling back to StringLiteral,
    // which made numeric FILTER comparisons against negative integers
    // never match (found via SHACL sh:sparql interaction testing).
    IntegerLiteral(i64),
    BooleanLiteral(bool),
}
impl From<bool> for EncodedTerm {
    fn from(value: bool) -> Self {
        Self::BooleanLiteral(value)
    }
}
impl From<String> for EncodedTerm {
    fn from(value: String) -> Self {
        Self::StringLiteral(value)
    }
}
impl From<i64> for EncodedTerm {
    fn from(value: i64) -> Self {
        Self::IntegerLiteral(value)
    }
}
pub struct QueryResults {
    plan: PlanNode,
    iterator: Box<dyn Iterator<Item=Vec<EncodedBinding>>>
}
impl Iterator for QueryResults{
    type Item = Vec<EncodedBinding>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}
pub fn eval_query<'a>(query: &'a Query, index: &'a TripleIndex) -> PlanNode {
    match query {
        spargebra::Query::Select {
            pattern, base_iri, ..
        } => {

            let plan = extract_query_plan(&pattern);

            plan
        }
        spargebra::Query::Ask {
            pattern, base_iri: _, ..
        } => {
            // ASK is equivalent to SELECT * WHERE { pattern } with the result clamped to boolean.
            // We return the inner plan; the caller checks whether it yields at least one result.
            extract_query_plan(pattern)
        }
        spargebra::Query::Construct {
            template,
            pattern,
            base_iri: _,
            ..
        } => {
            // CONSTRUCT evaluates the WHERE clause exactly like SELECT and then applies the
            // template to produce triples.  We model the template as a Project over the WHERE
            // plan — the actual triple construction is left to the calling layer which receives
            // the bound variables and instantiates the template patterns.
            let inner = extract_query_plan(pattern);
            // Collect the variable IDs referenced in the template so we can project them.
            let mapping: Vec<usize> = template
                .iter()
                .flat_map(|tp| {
                    let mut vars = Vec::new();
                    if let spargebra::term::TermPattern::Variable(v) = &tp.subject {
                        vars.push(crate::Encoder::add(v.as_str().to_string()));
                    }
                    if let spargebra::term::NamedNodePattern::Variable(v) = &tp.predicate {
                        vars.push(crate::Encoder::add(v.as_str().to_string()));
                    }
                    if let spargebra::term::TermPattern::Variable(v) = &tp.object {
                        vars.push(crate::Encoder::add(v.as_str().to_string()));
                    }
                    vars
                })
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            PlanNode::Project {
                child: Box::new(inner),
                mapping,
            }
        }
        spargebra::Query::Describe {
            pattern, base_iri: _, ..
        } => {
            // DESCRIBE returns an RDF description of the matched resources.
            // We compile the WHERE clause and let the caller enumerate the matched subjects.
            extract_query_plan(pattern)
        }
    }
}
#[cfg(test)]
#[path = "sparql_test.rs"]
mod sparql_test;
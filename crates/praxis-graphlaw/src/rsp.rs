use crate::rsp::r2r::R2ROperator;
use crate::rsp::r2s::{Relation2StreamOperator, StreamOperator};
use crate::rsp::s2r::{
    CSPARQLWindow, ContentContainer, Report, ReportStrategy, Tick, WindowTriple,
};
use crate::sparql::{eval_query, evaluate_plan_and_debug, Binding};
use crate::{Encoder, Syntax, Triple, TripleStore};
use log::{debug, error, info, trace, warn}; // Use log crate when building application
use spargebra::Query;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;


pub mod r2r;
pub mod r2s;
pub mod s2r;

pub enum OperationMode {
    SingleThread,
    MultiThread,
}
pub struct RSPBuilder<'a, I, O> {
    width: usize,
    slide: usize,
    tick: Option<Tick>,
    report_strategy: Option<ReportStrategy>,
    triples: Option<&'a str>,
    syntax: Option<Syntax>,
    rules: Option<&'a str>,
    query_str: Option<&'a str>,
    result_consumer: Option<ResultConsumer<O>>,
    r2s: Option<StreamOperator>,
    r2r: Option<Box<dyn R2ROperator<I, O>>>,
    operation_mode: OperationMode,
}
impl<'a, I, O> RSPBuilder<'a, I, O>
where
    O: Clone + Hash + Eq + Send + Debug + 'static,
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    pub fn new(width: usize, slide: usize) -> RSPBuilder<'a, I, O> {
        RSPBuilder {
            width,
            slide,
            tick: None,
            report_strategy: None,
            triples: None,
            syntax: None,
            rules: None,
            query_str: None,
            result_consumer: None,
            r2s: None,
            r2r: None,
            operation_mode: OperationMode::MultiThread,
        }
    }
    pub fn add_tick(mut self, tick: Tick) -> RSPBuilder<'a, I, O> {
        self.tick = Some(tick);
        self
    }
    pub fn add_report_strategy(mut self, strategy: ReportStrategy) -> RSPBuilder<'a, I, O> {
        self.report_strategy = Some(strategy);
        self
    }
    pub fn add_triples(mut self, triples: &'a str) -> RSPBuilder<'a, I, O> {
        self.triples = Some(triples);
        self
    }
    pub fn add_rules(mut self, rules: &'a str) -> RSPBuilder<'a, I, O> {
        self.rules = Some(rules);
        self
    }
    pub fn add_query(mut self, query: &'a str) -> RSPBuilder<'a, I, O> {
        self.query_str = Some(query);
        self
    }
    pub fn add_consumer(mut self, consumer: ResultConsumer<O>) -> RSPBuilder<'a, I, O> {
        self.result_consumer = Some(consumer);
        self
    }
    pub fn add_r2s(mut self, r2s: StreamOperator) -> RSPBuilder<'a, I, O> {
        self.r2s = Some(r2s);
        self
    }
    pub fn add_r2r(mut self, r2r: Box<dyn R2ROperator<I, O>>) -> RSPBuilder<'a, I, O> {
        self.r2r = Some(r2r);
        self
    }
    pub fn add_syntax(mut self, syntax: Syntax) -> RSPBuilder<'a, I, O> {
        self.syntax = Some(syntax);
        self
    }
    pub fn set_operation_mode(mut self, operation_mode: OperationMode) -> RSPBuilder<'a, I, O> {
        self.operation_mode = operation_mode;
        self
    }
    pub fn build(self) -> RSPEngine<I, O> {
        RSPEngine::new(
            self.width,
            self.slide,
            self.tick.unwrap_or_default(),
            self.report_strategy.unwrap_or_default(),
            self.triples.unwrap_or(""),
            self.syntax.unwrap_or_default(),
            self.rules.unwrap_or(""),
            self.query_str.expect("Please provide R2R query"),
            self.result_consumer.unwrap_or(ResultConsumer {
                function: Arc::new(Box::new(|r| println!("Bindings: {:?}", r))),
            }),
            self.r2s.unwrap_or_default(),
            self.r2r.expect("Please provide R2R operator!"),
            self.operation_mode,
        )
    }
}
pub struct RSPEngine<I, O>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send,
{
    s2r: CSPARQLWindow<I>,
    r2r: Arc<Mutex<Box<dyn R2ROperator<I, O>>>>,
    r2s_consumer: ResultConsumer<O>,
    r2s_operator: Arc<Mutex<Relation2StreamOperator<O>>>,
}
pub struct ResultConsumer<I> {
    pub function: Arc<dyn Fn(I) -> () + Send + Sync>,
}

impl<I, O> RSPEngine<I, O>
where
    O: Clone + Hash + Eq + Send + 'static,
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{
    pub fn new(
        width: usize,
        slide: usize,
        tick: Tick,
        report_strategy: ReportStrategy,
        triples: &str,
        syntax: Syntax,
        rules: &str,
        query_str: &str,
        result_consumer: ResultConsumer<O>,
        r2s: StreamOperator,
        r2r: Box<dyn R2ROperator<I, O>>,
        operation_mode: OperationMode,
    ) -> RSPEngine<I, O> {
        let mut report = Report::new();
        report.add(report_strategy);
        let mut window = CSPARQLWindow::new(width, slide, report, tick);
        let mut store = r2r;

        match store.load_triples(triples, syntax) {
            Err(parsing_error) => error!("Unable to load ABox: {:?}", parsing_error.to_string()),
            _ => (),
        }
        store.load_rules(rules);
        let query = match Query::parse(query_str, None) {
            Ok(parsed_query) => parsed_query,
            Err(err) => {
                error!("Unable to parse query! {:?}", err.to_string());
                error!("Using Select * WHERE{{?s ?p ?o}} instead");
                Query::parse("Select * WHERE{?s ?p ?o}", None).unwrap()
            }
        };
        let mut engine = RSPEngine {
            s2r: window,
            r2r: Arc::new(Mutex::new(store)),
            r2s_consumer: result_consumer,
            r2s_operator: Arc::new(Mutex::new(Relation2StreamOperator::new(r2s, 0))),
        };
        match operation_mode {
            OperationMode::SingleThread => {
                let consumer_temp = engine.r2r.clone();
                let r2s_consumer = engine.r2s_consumer.function.clone();
                let mut r2s_operator = engine.r2s_operator.clone();
                let call_back: Box<dyn FnMut(ContentContainer<I>) -> ()> =
                    Box::new(move |content| {
                        Self::evaluate_r2r_and_call_r2s(
                            &query,
                            consumer_temp.clone(),
                            r2s_consumer.clone(),
                            r2s_operator.clone(),
                            content,
                        );
                    });
                engine.s2r.register_callback(call_back);
            }
            OperationMode::MultiThread => {
                let consumer = engine.s2r.register();
                engine.register_r2r(consumer, query);
            }
        }

        engine
    }
    fn register_r2r(&mut self, receiver: Receiver<ContentContainer<I>>, query: Query) {
        let consumer_temp = self.r2r.clone();
        let r2s_consumer = self.r2s_consumer.function.clone();
        let mut r2s_operator = self.r2s_operator.clone();
        thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(mut content) => {
                        Self::evaluate_r2r_and_call_r2s(
                            &query,
                            consumer_temp.clone(),
                            r2s_consumer.clone(),
                            r2s_operator.clone(),
                            content,
                        );
                    }
                    Err(_) => {
                        debug!("Shutting down!");
                        break;
                    }
                }
            }
            debug!("Shutdown complete!");
        });
    }

    fn evaluate_r2r_and_call_r2s(
        query: &Query,
        consumer_temp: Arc<Mutex<Box<dyn R2ROperator<I, O>>>>,
        r2s_consumer: Arc<dyn Fn(O) + Send + Sync>,
        mut r2s_operator: Arc<Mutex<Relation2StreamOperator<O>>>,
        mut content: ContentContainer<I>,
    ) {
        debug!("R2R operator retrieved graph {:?}", content);
        let time_stamp = content.get_last_timestamp_changed();
        let mut store = consumer_temp.lock().unwrap();
        content.clone().into_iter().for_each(|t| {
            store.add(t);
        });
        let inferred = store.materialize();
        let r2r_result = store.execute_query(&query);
        let r2s_result = r2s_operator.lock().unwrap().eval(r2r_result, time_stamp);
        // R2S runs synchronously in the same thread as R2R; async dispatch is left for future work
        for result in r2s_result {
            (r2s_consumer)(result);
        }
        //remove data from stream
        content.iter().for_each(|t| {
            store.remove(t);
        });
        inferred.iter().for_each(|t| {
            store.remove(t);
        });
    }

    pub fn add(&mut self, event_item: I, ts: usize) {
        self.s2r.add_to_window(event_item, ts);
    }
    pub fn stop(&mut self) {
        self.s2r.stop();
    }
}

pub struct SimpleR2R {
    pub item: TripleStore,
}
impl R2ROperator<WindowTriple, Vec<Binding>> for SimpleR2R {
    fn load_triples(&mut self, data: &str, syntax: Syntax) -> Result<(), String> {
        let reseult = self.item.load_triples(data, syntax);
        println!(
            "Store size after loading: {:?}",
            self.item.triple_index.len()
        );
        reseult
    }

    fn load_rules(&mut self, data: &str) -> Result<(), &'static str> {
        self.item.load_rules(data).map_err(|_| "Failed to load rules")
    }

    fn add(&mut self, data: WindowTriple) {
        println!("Store size: {:?}", self.item.triple_index.len());

        let encoded_triple = Triple::from(data.s, data.p, data.o);
        self.item.add(encoded_triple);
    }

    fn remove(&mut self, data: &WindowTriple) {
        let encoded_triple = Triple::from(data.s.clone(), data.p.clone(), data.o.clone());

        self.item.remove_ref(&encoded_triple);
    }

    fn materialize(&mut self) -> Vec<WindowTriple> {
        println!("Store size: {:?}", self.item.triple_index.len());
        let inferred = self.item.materialize();
        inferred
            .into_iter()
            .map(|t| WindowTriple {
                s: Encoder::decode(&t.s.to_encoded()).unwrap().to_string(),
                p: Encoder::decode(&t.p.to_encoded()).unwrap().to_string(),
                o: Encoder::decode(&t.o.to_encoded()).unwrap().to_string(),
            })
            .collect()
    }

    fn execute_query(&self, query: &Query) -> Vec<Vec<Binding>> {
        let plan = eval_query(&query, &self.item.triple_index);
        let iterator = evaluate_plan_and_debug(&plan, &self.item.triple_index);
        iterator.collect()
    }
}

#[cfg(test)]
#[path = "rsp_test.rs"]
mod rsp_test;

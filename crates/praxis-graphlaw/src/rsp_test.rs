#![cfg(test)]

use std::fs::{File, OpenOptions};
    use std::io;
    use std::io::{BufRead, Write};

    use super::*;
    use std::time::Duration;

    #[test]
    #[ignore]
    fn rsp_integration() {
        let ntriples_file = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/foo> <http://schema.org/name> \"Foo\" .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/bar> <http://schema.org/name> \"Bar\" .";
        let rules = "@prefix test: <http://www.w3.org/test/>.\n{?x <http://test.be/hasVal> ?y. ?y <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person>.}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> test:SuperType.}";
        let function = Box::new(|r| println!("Bindings: {:?}", r));
        let result_consumer = ResultConsumer {
            function: Arc::new(function),
        };
        let r2r = Box::new(SimpleR2R {
            item: TripleStore::new(),
        });
        let mut engine = RSPBuilder::new(10, 2)
            .add_tick(Tick::TimeDriven)
            .add_report_strategy(ReportStrategy::OnWindowClose)
            .add_triples(ntriples_file)
            .add_syntax(Syntax::NTriples)
            .add_rules(rules)
            .add_query("Select * WHERE{ ?s a <http://www.w3.org/test/SuperType>}")
            .add_consumer(result_consumer)
            .add_r2r(r2r)
            .add_r2s(StreamOperator::RSTREAM)
            .build();
        for i in 0..10 {
            let triple = WindowTriple {
                s: format!("s{}", i),
                p: "<http://test.be/hasVal>".to_string(),
                o: "<http://example.com/foo>".to_string(),
            };

            engine.add(triple, i);
        }
        engine.stop();
        thread::sleep(Duration::from_secs(2));
    }
    #[test]
    #[ignore]
    fn test_load_from_file() {
        let ntriples_file = "<http://example.com/foo> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/foo> <http://schema.org/name> \"Foo\" .
<http://example.com/bar> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person> .
<http://example.com/bar> <http://schema.org/name> \"Bar\" .";
        let rules = "@prefix test: <http://www.w3.org/test/>.\n{?x <http://test.be/hasVal> ?y. ?y <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://schema.org/Person>.}=>{?x <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> test:SuperType.}";

        //write to file
        let function = Box::new(|r| {
            let mut output = OpenOptions::new()
                .write(true)
                .append(true)
                .open("/tmp/rox_rsp.out")
                .unwrap();
            write!(output, "Bindings: {:?}\n", r).unwrap();
        });
        let result_consumer = ResultConsumer {
            function: Arc::new(function),
        };
        let r2r = Box::new(SimpleR2R {
            item: TripleStore::new(),
        });
        let mut engine = RSPBuilder::new(10, 2)
            .add_tick(Tick::TimeDriven)
            .add_report_strategy(ReportStrategy::OnWindowClose)
            .add_triples(ntriples_file)
            .add_syntax(Syntax::NTriples)
            .add_rules(rules)
            .add_query("Select * WHERE{ ?s <http://test/hasLocation> ?loc}")
            .add_consumer(result_consumer)
            .add_r2r(r2r)
            .add_r2s(StreamOperator::RSTREAM)
            .build();

        //read from file:
        let file = File::open(
            "/Users/psbonte/Documents/Github/RoXi/examples/rsp/location_update_stream.nt",
        )
        .unwrap();

        for (i, lines) in io::BufReader::new(file).lines().enumerate() {
            let lines = lines.unwrap();
            let mut line = lines.split(" ");
            let triple = WindowTriple {
                s: line.next().unwrap().to_string(),
                p: line.next().unwrap().to_string(),
                o: line.next().unwrap().to_string(),
            };

            engine.add(triple, i);
        }
        engine.stop();
        thread::sleep(Duration::from_secs(2));
    }
    #[test]
    #[ignore]
    fn rsp_transitive_testp() {
        let ntriples_file = "";
        let rules = "@prefix test: <http://test/>.
 @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.
 {?x test:isIn ?y. ?y test:isIn ?z. }=>{?x test:isIn ?z.}";
        let function = Box::new(|r| println!("Bindings: {:?}", r));
        let result_consumer = ResultConsumer {
            function: Arc::new(function),
        };
        let r2r = Box::new(SimpleR2R {
            item: TripleStore::new(),
        });
        let mut engine = RSPBuilder::new(10, 2)
            .add_tick(Tick::TimeDriven)
            .add_report_strategy(ReportStrategy::OnWindowClose)
            .add_triples(ntriples_file)
            .add_syntax(Syntax::NTriples)
            .add_rules(rules)
            .add_query("Select * WHERE{ ?x <http://test/isIn> ?y}")
            .add_consumer(result_consumer)
            .add_r2r(r2r)
            .add_r2s(StreamOperator::RSTREAM)
            .set_operation_mode(OperationMode::SingleThread)
            .build();
        for i in 0..20 {
            let triple = WindowTriple {
                s: format!("<http://test/{}>", i),
                p: "<http://test/isIn>".to_string(),
                o: format!("<http://test/{}>", i + 1),
            };

            engine.add(triple, i);
        }
        engine.stop();
    }

    #[test]
    #[ignore]
    fn test_static_abox() {
        let ntriples_file =
            "<http://test/sensor1> <http://test/hasLocation> <http://test/location1>.";
        let rules = "@prefix test: <http://test/>.
 @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>.
 {?x test:madeBy ?y. ?y test:hasLocation ?z. }=>{?x test:madeIn ?z.}";
        let function = Box::new(|r| println!("Bindings: {:?}", r));
        let result_consumer = ResultConsumer {
            function: Arc::new(function),
        };
        let r2r = Box::new(SimpleR2R {
            item: TripleStore::new(),
        });
        let mut engine = RSPBuilder::new(10, 2)
            .add_tick(Tick::TimeDriven)
            .add_report_strategy(ReportStrategy::OnWindowClose)
            .add_triples(ntriples_file)
            .add_syntax(Syntax::NTriples)
            .add_rules(rules)
            .add_query("Select * WHERE{ ?x <http://test/madeIn> ?y}")
            .add_consumer(result_consumer)
            .add_r2r(r2r)
            .add_r2s(StreamOperator::RSTREAM)
            .set_operation_mode(OperationMode::SingleThread)
            .build();
        for i in 0..20 {
            let triple = WindowTriple {
                s: format!("<http://test/{}>", i),
                p: "<http://test/madeBy>".to_string(),
                o: "<http://test/sensor1>".to_string(),
            };

            engine.add(triple, i);
        }
        engine.stop();
    }

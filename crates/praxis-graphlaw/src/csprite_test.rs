#![cfg(test)]

use crate::csprite::CSprite;
    use crate::reasoner::Reasoner;
    use crate::{
        BackwardChainer, Encoder, Parser, QueryEngine, Rule, RuleIndex, SimpleQueryEngine,
        TermImpl, Triple, TripleIndex, TripleStore, VarOrTerm,
    };
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn test_sprite_compute() {
        let data="<http://example2.com/a> a test:SubClass.\n\
                <http://example2.com/a> test:hasRef <http://example2.com/b>.\n\
                <http://example2.com/b> test:hasRef <http://example2.com/c>.\n\
                <http://example2.com/c> a test:SubClassH1.\n\
            {?s a test:SubClass.}=>{?s a test:SubClass2.}\n\
            {?s a test:SubClass2.}=>{?s a test:SubClass.}\n\
            {?s a test:SubClass0.}=>{?s a test:SubClass2.}\n\
            {?s a test:SubClass01.}=>{?s a test:SubClass0.}\n\
            {?s a test:SubClassH1.}=>{?s a test:SubClassH.}\n\
            {?s a test:SubClassH2.}=>{?s a test:SubClassH1.}\n\
            {?s a test:SubClassH22.}=>{?s a test:SubClassH1.}\n\
            {?s a test:SubClass2.?s test:hasRef ?b.?b test:hasRef ?c.?c a test:SubClassH.}=>{?s a test:SuperType.}\n\
            {?super a test:SuperType.}=>{?super a test:SuperType3.}";
        let mut store = CSprite::from(data);

        let backward_head = Triple {
            s: VarOrTerm::new_var("?newVar".to_string()),
            p: VarOrTerm::new_term("a".to_string()),
            o: VarOrTerm::new_term("test:SuperType".to_string()),
            g: None,
        };

        //assert_eq!(4,store.len());
        let validation_triple = Triple {
            s: VarOrTerm::new_term("<http://example2.com/a>".to_string()),
            p: VarOrTerm::new_term("a".to_string()),
            o: VarOrTerm::new_term("test:SuperType".to_string()),
            g: None,
        };

        store.compute_sprite(&backward_head);
        store.materialize();
        assert_eq!(true, store.triple_index.contains(&validation_triple));
        assert_eq!(7, store.len());
    }
    //todo move to benchmark
    #[test]
    fn test_sprite_compute_hierarchy() {
        let timer_load = ::std::time::Instant::now();

        let size = 10;
        let mut data = String::new();
        for i in 0..size {
            data += &format!("<http://example2.com/a{}> a test:SubClass0.\n", i);
            data += &format!(
                "{{?s a test:SubClass{}.}}=>{{?s a test:SubClass{}.}}\n",
                i,
                (i + 1)
            );
        }
        let mut store = CSprite::from(data.as_str());

        let backward_head = Triple {
            s: VarOrTerm::new_var("?newVar".to_string()),
            p: VarOrTerm::new_term("a".to_string()),
            o: VarOrTerm::new_term(format!("test:SubClass{}", size)),
            g: None,
        };

        let load_time = timer_load.elapsed();
        println!("Load Time: {:.2?}", load_time);
        assert_eq!(size, store.len());
        let timer_load = ::std::time::Instant::now();
        store.compute_sprite(&backward_head);
        let csprite_time = timer_load.elapsed();
        println!("CSprite Time: {:.2?}", csprite_time);
        let timer_load = ::std::time::Instant::now();
        store.materialize();
        assert_eq!(2 * size, store.len());
        let load_time = timer_load.elapsed();
        println!("Materialization Time: {:.2?}", load_time);
    }

    #[test]
    fn test_rewrite_hierarchy_csprite() {
        let data = "<http://example2.com/a> a test:SubClass.\n\
            {?s a test:SubClassH1.}=>{?s a test:SubClassH.}\n\
            {?s a test:SubClassH2.}=>{?s a test:SubClassH1.}\n\
            {?s a test:SubClassH3.}=>{?s a test:SubClassH2.}";
        let (_content, rules) = Parser::parse(data.to_string());
        println!("{:?}", rules);

        let rc_rules = rules.into_iter().map(|x| Rc::new(x)).collect();
        let rewritten_rules = CSprite::rewrite_hierarchy(&rc_rules);
        println!("{:?}", rewritten_rules);
    }

    #[test]
    fn test_csprite_cycles_terminate() {
        // TICKET-003: Csprite Cycle Guards
        // This test skeleton defines the DoD for cycle-safety in CSprite.
        // It exercises the helper functions with a cyclic class hierarchy.

        let data = "{?s a <http://example/ClassA>.}=>{?s a <http://example/ClassB>.}\n\
{?s a <http://example/ClassB>.}=>{?s a <http://example/ClassC>.}\n\
{?s a <http://example/ClassC>.}=>{?s a <http://example/ClassA>.}";

        // Placeholder that fails the test until TICKET-003 is implemented.
        // Once implemented, set is_implemented to true to run the actual test.
        let is_implemented = true;
        if !is_implemented {
            panic!("TICKET-003: Csprite Cycle Guards are not yet implemented. Set is_implemented to true once implemented.");
        }

        // Test eval_backward_csprite (recursive helper, lines 119 and 127)
        let (tx1, rx1) = std::sync::mpsc::channel();
        let data_str1 = data.to_string();
        let handle1 = std::thread::Builder::new()
            .name("csprite_recursive_cycle_test".to_string())
            .spawn(move || {
                let store = CSprite::from(&data_str1);
                let query = Triple {
                    s: VarOrTerm::new_var("?s".to_string()),
                    p: VarOrTerm::new_term("a".to_string()),
                    o: VarOrTerm::new_term("<http://example/ClassA>".to_string()),
                    g: None,
                };
                let (matched_rules, hierarchies) = store.eval_backward_csprite(&query);
                tx1.send((matched_rules.len(), hierarchies.len())).unwrap();
            })
            .expect("failed to spawn recursive helper thread");

        match rx1.recv_timeout(std::time::Duration::from_millis(500)) {
            Ok((matched_len, hierarchy_len)) => {
                let _ = matched_len;
                let _ = hierarchy_len;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                panic!("Test failed: CSprite recursive helper did not terminate within timeout.");
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                let join_res = handle1.join();
                panic!(
                    "Test failed: CSprite recursive helper thread crashed: {:?}",
                    join_res
                );
            }
        }

        // Test eval_backward_csprite_helper_with_stack (stack-based helper, lines 153 and 154)
        let (tx2, rx2) = std::sync::mpsc::channel();
        let data_str2 = data.to_string();
        let handle2 = std::thread::Builder::new()
            .name("csprite_stack_cycle_test".to_string())
            .spawn(move || {
                let store = CSprite::from(&data_str2);
                let query = Triple {
                    s: VarOrTerm::new_var("?s".to_string()),
                    p: VarOrTerm::new_term("a".to_string()),
                    o: VarOrTerm::new_term("<http://example/ClassA>".to_string()),
                    g: None,
                };
                let (matched_rules, hierarchies) =
                    store.eval_backward_csprite_helper_with_stack(&query);
                tx2.send((matched_rules.len(), hierarchies.len())).unwrap();
            })
            .expect("failed to spawn stack-based helper thread");

        match rx2.recv_timeout(std::time::Duration::from_millis(500)) {
            Ok((matched_len, hierarchy_len)) => {
                let _ = matched_len;
                let _ = hierarchy_len;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                panic!("Test failed: CSprite stack helper did not terminate within timeout.");
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                let join_res = handle2.join();
                panic!(
                    "Test failed: CSprite stack helper thread crashed: {:?}",
                    join_res
                );
            }
        }
    }

    // #[test]
    // fn test_transitive(){
    //     let rules ="{?a in ?b.?b in ?c}=>{?a in ?c}";
    //     let data =":1 in :0.\n\
    //         :2 in :1.\n\
    //         :3 in :2.\n\
    //         :4 in :3.\n\
    //         :5 in :4.\n\
    //         :6 in :5";
    //     let csprite = CSprite::from_with_window(rules, 4, 2);
    //     let (mut content, mut rules) = Parser::parse(data.to_string(), &mut csprite.borrow_mut().encoder);
    //
    //
    //
    //
    //
    //     content.into_iter().enumerate().for_each(|(i, t)| csprite.borrow_mut().window.add(t, i as i32));
    //
    //     //contains 4 triples and 1 inferred triple
    //     assert_eq!(19, csprite.borrow_mut().window.len());
    // }

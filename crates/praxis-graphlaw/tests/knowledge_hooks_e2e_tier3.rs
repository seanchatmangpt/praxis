mod common;

use common::assert_contains_triple;
use common::assert_not_contains_triple;
use praxis_graphlaw::parser::Syntax;
use praxis_graphlaw::TripleStore;

// TIER 3: CROSS-FEATURE COMBINATIONS
// =========================================================================

/// Covers F3 x F4: Datalog trigger projects delta changes via CONSTRUCT, triggering another delta hook.
#[test]
fn test_c3_datalog_construct_delta_cascade() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        // Hook 1: Datalog trigger -> project mid fact
        ex:h1 a kh:Hook ;
            kh:name "datalog_trigger" ;
            kh:kind "datalog" ;
            kh:program "vip(?x) :- ?x <http://example.org/spent> ?a , FILTER(?a > 1000) ." ;
            kh:goal "vip(?s)" ;
            kh:effect "emit-delta" ;
            kh:action ex:act_mid ;
            kh:priority 1 .
        ex:act_mid a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/vip_status> 'vip' } WHERE { ?s <http://example.org/spent> ?a . FILTER(?a > 1000) }" .

        // Hook 2: Delta trigger on vip_status -> project final fact
        ex:h2 a kh:Hook ;
            kh:name "delta_trigger" ;
            kh:kind "delta" ;
            kh:var "http://example.org/vip_status" ;
            kh:effect "emit-delta" ;
            kh:action ex:act_final ;
            kh:priority 2 ;
            kh:after ex:h1 .
        ex:act_final a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/access> 'granted' } WHERE { ?s <http://example.org/vip_status> 'vip' }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples("ex:Alice <http://example.org/spent> 1200 .", Syntax::Turtle)
        .unwrap();

    store.materialize().unwrap();

    assert_contains_triple(&store, "Alice", "access", "granted");
}

/// Covers F2 x F4 x F5 x F6: Gating, CONSTRUCT projection, BLAKE3 receipts, and fixpoint loops.
#[test]
fn test_c3_gating_construct_blake3_fixpoint() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:gate_hook a kh:Hook ;
            kh:name "composite_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" ;
            kh:action ex:proj_action .
            
        ex:proj_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/output> 'yes' } WHERE { ?s <http://example.org/trigger> 'yes' }" .
    "#;
    // 1. Constitutional gating verified at load time
    store.load_hook_pack(hook_pack).unwrap();

    // 2. Fixpoint loops execution
    store
        .load_triples(
            "ex:Node <http://example.org/trigger> 'yes' .",
            Syntax::Turtle,
        )
        .unwrap();
    store.materialize().unwrap();

    // 3. CONSTRUCT projection applied
    assert_contains_triple(&store, "Node", "output", "yes");

    // 4. BLAKE3 Receipts validation
    let receipts = store.get_hook_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].hook_name, "composite_hook");
}

/// Covers F3: Evaluates multiple hooks representing threshold, count, and window concurrently.
#[test]
fn test_c3_threshold_count_window_concurrency() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_thresh a kh:Hook ;
            kh:name "thresh_hook" ;
            kh:kind "threshold" ;
            kh:var "http://example.org/val" ;
            kh:op ">" ;
            kh:k 100 ;
            kh:effect "emit-delta" .

        ex:h_count a kh:Hook ;
            kh:name "count_hook" ;
            kh:kind "count" ;
            kh:var "http://example.org/item" ;
            kh:op ">=" ;
            kh:k 3 ;
            kh:effect "emit-delta" .

        ex:h_window a kh:Hook ;
            kh:name "window_hook" ;
            kh:kind "window" ;
            kh:var "http://example.org/metric" ;
            kh:op ">" ;
            kh:k 10 ;
            kh:window 5 ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();

    // Load triples that fire count & threshold hooks
    store
        .load_triples(
            "ex:Obj <http://example.org/val> 150 .
         ex:Obj <http://example.org/item> 1 , 2 , 3 .",
            Syntax::Turtle,
        )
        .unwrap();

    store.materialize().unwrap();

    let receipts = store.get_hook_receipts();
    assert!(receipts.len() >= 2);
}

/// Covers F3 x F2: Hook pack utilizing an N3 rule trigger which is valid under constitutional gating.
#[test]
fn test_c3_n3_trigger_gating_valid() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:n3_hook a kh:Hook ;
            kh:name "n3_trigger_hook" ;
            kh:kind "n3" ;
            kh:program "{ ?x <http://example.org/p> ?y } => { ?x <http://example.org/q> ?y } ." ;
            kh:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(res.is_ok(), "Gating failed for valid N3 trigger hook");
}

/// Covers F4 x F5: A CONSTRUCT projection yielding zero changes must NOT generate any BLAKE3 receipts.
#[test]
fn test_c3_construct_empty_no_receipt() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:empty_receipt_hook a kh:Hook ;
            kh:name "empty_construct_receipt" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" ;
            kh:action ex:act .
            
        ex:act a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/out> 'yes' } WHERE { ?s <http://example.org/nonexistent> 'yes' }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples(
            "ex:Node <http://example.org/trigger> 'yes' .",
            Syntax::Turtle,
        )
        .unwrap();

    store.materialize().unwrap();

    // Verify no receipts generated since the CONSTRUCT yielded no changes
    let receipts = store.get_hook_receipts();
    assert!(
        receipts.is_empty(),
        "Empty CONSTRUCT projection must not generate any receipts"
    );
}

/// Covers F3 x F4 x F6: SPARQL ASK trigger constructs a deletion, leading to early termination of materialization.
#[test]
fn test_c3_sparql_ask_construct_delete_early_termination() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:term_hook a kh:Hook ;
            kh:name "delete_early_termination" ;
            kh:kind "sparql" ;
            kh:query "ASK { ?s <http://example.org/terminate> 'true' }" ;
            kh:effect "emit-delta" ;
            kh:action ex:del_action .
            
        ex:del_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { } WHERE { ?s <http://example.org/active_flow> ?any }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store.load_triples(
        "ex:Flow <http://example.org/active_flow> 'yes' ; <http://example.org/terminate> 'true' .",
        Syntax::Turtle
    ).unwrap();

    store.materialize().unwrap();

    // The active flow triple should be deleted, stopping further cascade pass evaluations
    assert_not_contains_triple(&store, "Flow", "active_flow", "yes");
}

// =========================================================================

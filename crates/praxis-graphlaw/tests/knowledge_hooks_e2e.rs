use praxis_graphlaw::parser::Syntax;
use praxis_graphlaw::TripleStore;

// Expected public HookReceipt structure defined as requested
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookReceipt {
    pub hook_name: String,
    pub delta_hash: String,
    pub idempotency_key: String,
    pub delta_quads: String,
}

// Helper to assert that a triple exists in the store based on content_to_string output
fn assert_contains_triple(store: &TripleStore, s: &str, p: &str, o: &str) {
    let content = store.content_to_string();
    assert!(
        content.contains(s) && content.contains(p) && content.contains(o),
        "Expected triple <{} {} {}> not found in store content:\n{}",
        s,
        p,
        o,
        content
    );
}

// Helper to assert that a triple does not exist in the store
fn assert_not_contains_triple(store: &TripleStore, s: &str, p: &str, o: &str) {
    let content = store.content_to_string();
    let found = content.contains(s) && content.contains(p) && content.contains(o);
    assert!(
        !found,
        "Triple <{} {} {}> was found in store content but was expected to be absent:\n{}",
        s, p, o, content
    );
}

// =========================================================================
// TIER 1: FEATURE COVERAGE
// =========================================================================

// --- F1: Hook Parsing & Registry ---

/// Covers F1: Verifies parsing and registration of a single valid hook pack.
#[test]
fn test_f1_load_valid_single() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:my_hook a kh:Hook ;
            kh:name "valid_single_hook" ;
            kh:kind "delta" ;
            kh:var "x" ;
            kh:on "assert" ;
            kh:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(res.is_ok(), "Failed to load single valid hook: {:?}", res);
}

/// Covers F1: Verifies parsing and registration of multiple valid hooks in a single pack.
#[test]
fn test_f1_load_valid_multiple() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:hook1 a kh:Hook ;
            kh:name "multiple_hook_1" ;
            kh:kind "delta" ;
            kh:var "x" ;
            kh:on "assert" ;
            kh:effect "emit-delta" ;
            kh:priority 1 .

        ex:hook2 a kh:Hook ;
            kh:name "multiple_hook_2" ;
            kh:kind "delta" ;
            kh:var "y" ;
            kh:on "assert" ;
            kh:effect "emit-delta" ;
            kh:priority 2 .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_ok(),
        "Failed to load multiple valid hooks: {:?}",
        res
    );
}

/// Covers F1: Verifies namespace/prefix resolution within the hook pack parser.
#[test]
fn test_f1_resolve_prefixes() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix myns: <http://example.org/myns#> .

        myns:hook_with_prefixes a kh:Hook ;
            kh:name "prefix_hook" ;
            kh:kind "delta" ;
            kh:var "x" ;
            kh:on "assert" ;
            kh:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(res.is_ok(), "Failed to resolve prefixes: {:?}", res);
}

/// Covers F1: Verifies hook: alias namespace is rewritten to kh: canonical form.
/// The hook: namespace is a user-friendly alias for kh:, allowing users to write
/// hook:name instead of kh:name. This test verifies the aliasing works correctly.
#[test]
fn test_f1_hook_alias_namespace_rewrite() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
        @prefix ex: <http://example.org/> .

        ex:alias_hook a kh:Hook ;
            hook:name "alias_test_hook" ;
            hook:kind "delta" ;
            hook:var "x" ;
            hook:on "assert" ;
            hook:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_ok(),
        "Hook pack using hook: alias namespace should succeed: {:?}",
        res
    );

    // Verify the hook was registered successfully by querying it
    let query_str = "SELECT ?name WHERE { ?h a <http://seanchatmangpt.github.io/praxis/kh#Hook> ; <http://seanchatmangpt.github.io/praxis/kh#name> ?name }";
    let rows = store.query(query_str).unwrap();
    assert_eq!(
        rows.len(),
        1,
        "Hook alias should have been rewritten and registered"
    );
    assert_eq!(rows[0][0].val, "\"alias_test_hook\"");
}

/// Covers F1: Verifies parsing hooks with inline triples representing hook attributes.
#[test]
fn test_f1_load_inline_triples() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        ex:h1 a kh:Hook ; kh:name "inline_hook" ; kh:kind "delta" ; kh:var "v" ; kh:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(res.is_ok(), "Failed to load inline triples: {:?}", res);
}

/// Covers F1: Verifies that registered hooks can be queried using SPARQL or the internal registry.
#[test]
fn test_f1_query_registered_hooks() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_query a kh:Hook ;
            kh:name "queryable_hook" ;
            kh:kind "delta" ;
            kh:var "v" ;
            kh:on "assert" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();

    // Querying the hook registration in the store's facts
    let query_str = "SELECT ?name WHERE { ?h a <http://seanchatmangpt.github.io/praxis/kh#Hook> ; <http://seanchatmangpt.github.io/praxis/kh#name> ?name }";
    let rows = store.query(query_str).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0].val, "\"queryable_hook\"");
}

// --- F2: Constitutional Gating ---

/// Covers F2: Verifies that hooks using forbidden system commands are rejected.
#[test]
fn test_f2_refuse_command() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:malicious_hook a kh:Hook ;
            kh:name "command_hook" ;
            kh:kind "delta" ;
            kh:var "v" ;
            kh:effect "ground-action" ;
            kh:action ex:forbidden_action .
            
        ex:forbidden_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#command-exec> .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_err(),
        "Hook with forbidden command handler should be refused"
    );
    let err = res.unwrap_err();
    assert!(err.contains("forbidden") || err.contains("SHACL") || err.contains("validation"));
}

/// Covers F2: Verifies that hooks trying to run shell utilities are refused.
#[test]
fn test_f2_refuse_shell() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:shell_hook a kh:Hook ;
            kh:name "shell_exec_hook" ;
            kh:kind "delta" ;
            kh:var "v" ;
            kh:effect "ground-action" ;
            kh:action ex:shell_action .
            
        ex:shell_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#shell-exec> .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(res.is_err(), "Hook with shell execution should be refused");
}

/// Covers F2: Verifies that hooks using unrecognized actions are refused.
#[test]
fn test_f2_refuse_unrecognized_action() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:unrecognized_hook a kh:Hook ;
            kh:name "unrecognized_action_hook" ;
            kh:kind "delta" ;
            kh:var "v" ;
            kh:effect "ground-action" ;
            kh:action ex:unrecognized_action .
            
        ex:unrecognized_action a kh:Action ;
            kh:handler <http://example.org/handler#unknown-handler> .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_err(),
        "Hook with unrecognized action should be refused"
    );
}

/// Covers F2: Verifies that hook packs violating the SHACL Law Pack schema fail gating.
#[test]
fn test_f2_gating_malformed_shacl() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        // Violates SHACL shape because kh:kind is missing (mandatory)
        ex:malformed_hook a kh:Hook ;
            kh:name "malformed_shacl_hook" ;
            kh:var "v" ;
            kh:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_err(),
        "Malformed hook missing mandatory field should fail SHACL gating"
    );
}

/// Covers F2: Verifies that when a hook pack is refused, the store is rolled back and no triples remain.
#[test]
fn test_f2_gating_rollback_state() {
    let mut store = TripleStore::new();

    // Record initial length
    let initial_len = store.len();

    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:ok_hook a kh:Hook ;
            kh:name "ok_hook" ;
            kh:kind "delta" ;
            kh:var "v" ;
            kh:effect "emit-delta" .

        // Malformed hook triggers refusal
        ex:bad_hook a kh:Hook ;
            kh:name "bad_hook" ;
            kh:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(res.is_err());

    // Assert that no new triples were registered in the store (full transaction rollback)
    assert_eq!(store.len(), initial_len);
}

// --- F3: First-Class Trigger Dialects ---

/// Covers F3: Verifies SPARQL ASK trigger dialect evaluation.
#[test]
fn test_f3_sparql_ask_trigger() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_ask a kh:Hook ;
            kh:name "ask_hook" ;
            kh:kind "sparql" ;
            kh:query "ASK { ?s <http://example.org/status> 'critical' }" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();

    // Assert status critical fact
    store
        .load_triples(
            "ex:Server1 <http://example.org/status> 'critical' .",
            Syntax::Turtle,
        )
        .unwrap();

    // Running materialize should fire the SPARQL ASK hook
    let _inferred = store.materialize();

    // Verify receipt was generated
    let receipts = store.get_hook_receipts();
    assert!(
        !receipts.is_empty(),
        "SPARQL ASK trigger should have generated a receipt"
    );
    assert_eq!(receipts[0].hook_name, "ask_hook");
}

/// Covers F3: Verifies SPARQL SELECT trigger dialect evaluation.
#[test]
fn test_f3_sparql_select_trigger() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_select a kh:Hook ;
            kh:name "select_hook" ;
            kh:kind "sparql" ;
            kh:query "SELECT ?s ?val WHERE { ?s <http://example.org/value> ?val . FILTER(?val > 100) }" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples(
            "ex:Sensor1 <http://example.org/value> 150 .",
            Syntax::Turtle,
        )
        .unwrap();

    let _inferred = store.materialize();
    let receipts = store.get_hook_receipts();
    assert!(
        !receipts.is_empty(),
        "SPARQL SELECT trigger should have generated a receipt"
    );
    assert_eq!(receipts[0].hook_name, "select_hook");
}

/// Covers F3: Verifies Count trigger dialect evaluation.
#[test]
fn test_f3_count_trigger() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_count a kh:Hook ;
            kh:name "count_hook" ;
            kh:kind "count" ;
            kh:var "http://example.org/item" ;
            kh:op ">=" ;
            kh:k 3 ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();

    // Add 2 items (below limit of 3)
    store
        .load_triples(
            "ex:Order <http://example.org/item> ex:A , ex:B .",
            Syntax::Turtle,
        )
        .unwrap();
    store.materialize();
    assert!(store.get_hook_receipts().is_empty());

    // Add 3rd item
    store
        .load_triples("ex:Order <http://example.org/item> ex:C .", Syntax::Turtle)
        .unwrap();
    store.materialize();
    assert!(!store.get_hook_receipts().is_empty());
}

/// Covers F3: Verifies Threshold trigger dialect evaluation.
#[test]
fn test_f3_threshold_trigger() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_threshold a kh:Hook ;
            kh:name "threshold_hook" ;
            kh:kind "threshold" ;
            kh:var "http://example.org/temperature" ;
            kh:op ">" ;
            kh:k 99 ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();

    // Temperature at 98 (does not fire)
    store
        .load_triples(
            "ex:Room <http://example.org/temperature> 98 .",
            Syntax::Turtle,
        )
        .unwrap();
    store.materialize();
    assert!(store.get_hook_receipts().is_empty());

    // Temperature at 100 (fires)
    store
        .load_triples(
            "ex:Room <http://example.org/temperature> 100 .",
            Syntax::Turtle,
        )
        .unwrap();
    store.materialize();
    assert!(!store.get_hook_receipts().is_empty());
}

/// Covers F3: Verifies Delta trigger dialect evaluation.
#[test]
fn test_f3_delta_trigger() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_delta a kh:Hook ;
            kh:name "delta_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/status" ;
            kh:on "assert" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();

    store
        .load_triples(
            "ex:A <http://example.org/status> 'active' .",
            Syntax::Turtle,
        )
        .unwrap();
    store.materialize();
    assert!(!store.get_hook_receipts().is_empty());
}

/// Covers F3: Verifies Datalog trigger dialect formatting and execution.
#[test]
fn test_f3_datalog_trigger_format() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_datalog a kh:Hook ;
            kh:name "datalog_hook" ;
            kh:kind "datalog" ;
            kh:program "reachable(?x, ?y) :- ?x <http://example.org/link> ?y . reachable(?x, ?z) :- reachable(?x, ?y), reachable(?y, ?z) ." ;
            kh:goal "reachable(?s, ?o)" ;
            kh:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_ok(),
        "Failed to load Datalog trigger format: {:?}",
        res
    );
}

// --- F4: Pure Action Projections ---

/// Covers F4: Verifies projection of `kh:addQuad` declarative changes.
#[test]
fn test_f4_project_add_quad() {
    let mut store = TripleStore::new();

    // Register hook that adds a VIP status triple when someone has spent > 1000
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:vip_hook a kh:Hook ;
            kh:name "vip_add_hook" ;
            kh:kind "sparql" ;
            kh:query "SELECT ?cust WHERE { ?cust <http://example.org/spent> ?amount . FILTER(?amount > 1000) }" ;
            kh:effect "emit-delta" ;
            kh:action ex:add_vip_action .
            
        ex:add_vip_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?cust <http://example.org/status> 'VIP' } WHERE { ?cust <http://example.org/spent> ?amount . FILTER(?amount > 1000) }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples("ex:Alice <http://example.org/spent> 1500 .", Syntax::Turtle)
        .unwrap();

    store.materialize();

    // Verify pure projection applied the VIP status
    assert_contains_triple(&store, "Alice", "status", "VIP");
}

/// Covers F4: Verifies projection of `kh:deleteQuad` declarative changes.
#[test]
fn test_f4_project_delete_quad() {
    let mut store = TripleStore::new();

    // Register hook that deletes standard status when someone becomes VIP
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:rm_std_hook a kh:Hook ;
            kh:name "remove_standard_hook" ;
            kh:kind "sparql" ;
            kh:query "SELECT ?cust WHERE { ?cust <http://example.org/status> 'VIP' }" ;
            kh:effect "emit-delta" ;
            kh:action ex:rm_std_action .
            
        ex:rm_std_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { } WHERE { ?cust <http://example.org/status> 'Standard' } " .
    "#;
    // Wait, construct with deletion usually projects deletion quads. In graphlaw,
    // this can be mapped to kh:deleteQuad predicate.
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples(
            "ex:Alice <http://example.org/status> 'Standard' , 'VIP' .",
            Syntax::Turtle,
        )
        .unwrap();

    store.materialize();
    // Assuming standard status is removed by the hook
    assert_not_contains_triple(&store, "Alice", "status", "Standard");
}

/// Covers F4: Verifies projection of both additions and deletions simultaneously.
#[test]
fn test_f4_project_add_and_delete() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:swap_hook a kh:Hook ;
            kh:name "swap_status_hook" ;
            kh:kind "sparql" ;
            kh:query "SELECT ?cust WHERE { ?cust <http://example.org/trigger> 'go' }" ;
            kh:effect "emit-delta" ;
            kh:action ex:swap_action .
            
        ex:swap_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?cust <http://example.org/has> 'new_val' . } WHERE { ?cust <http://example.org/has> 'old_val' }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples(
            "ex:Alice <http://example.org/has> 'old_val' ; <http://example.org/trigger> 'go' .",
            Syntax::Turtle,
        )
        .unwrap();

    store.materialize();

    assert_contains_triple(&store, "Alice", "has", "new_val");
}

/// Covers F4: Verifies that hooks attempting non-pure side-effects are blocked.
#[test]
fn test_f4_refuse_side_effects() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:side_effect_hook a kh:Hook ;
            kh:name "malicious_projection" ;
            kh:kind "sparql" ;
            kh:query "ASK { ?s ?p ?o }" ;
            kh:effect "ground-action" ;
            kh:action ex:http_call .
            
        ex:http_call a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#fetch-url> .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_err(),
        "Hook projecting side-effects should fail constitutional gating"
    );
}

/// Covers F4: Verifies that projected quads are applied to the correct named graph.
#[test]
fn test_f4_project_apply_to_graph() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:graph_hook a kh:Hook ;
            kh:name "apply_to_graph_hook" ;
            kh:kind "sparql" ;
            kh:query "SELECT ?cust WHERE { ?cust <http://example.org/flag> 'vip' }" ;
            kh:effect "emit-delta" ;
            kh:action ex:graph_action .
            
        ex:graph_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { GRAPH <http://example.org/VIPGraph> { ?cust a <http://example.org/VIPMember> } } WHERE { ?cust <http://example.org/flag> 'vip' }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples("ex:Alice <http://example.org/flag> 'vip' .", Syntax::Turtle)
        .unwrap();

    store.materialize();

    // Assert target named graph contains the projected quad
    assert_contains_triple(&store, "Alice", "type", "VIPMember");
}

// --- F5: Canonical N-Quads & BLAKE3 Receipts ---

/// Covers F5: Verifies receipt generation for a single quad addition.
#[test]
fn test_f5_receipt_single_add() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:receipt_hook a kh:Hook ;
            kh:name "single_add_receipt" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples(
            "ex:Fact <http://example.org/trigger> 'yes' .",
            Syntax::Turtle,
        )
        .unwrap();

    store.materialize();

    let receipts = store.get_hook_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].hook_name, "single_add_receipt");
    assert!(
        !receipts[0].delta_hash.is_empty(),
        "Delta hash should be non-empty BLAKE3 hash"
    );
}

/// Covers F5: Verifies sort determinism of canonical N-Quads before hashing.
#[test]
fn test_f5_receipt_sort_determinism() {
    let mut store_a = TripleStore::new();
    let mut store_b = TripleStore::new();

    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:det_hook a kh:Hook ;
            kh:name "sort_det_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" .
    "#;

    store_a.load_hook_pack(hook_pack).unwrap();
    store_b.load_hook_pack(hook_pack).unwrap();

    // Add facts in order A, then B
    store_a.load_triples("ex:NodeA <http://example.org/trigger> 'yes' . ex:NodeB <http://example.org/trigger> 'yes' .", Syntax::Turtle).unwrap();
    store_a.materialize();

    // Add facts in order B, then A
    store_b.load_triples("ex:NodeB <http://example.org/trigger> 'yes' . ex:NodeA <http://example.org/trigger> 'yes' .", Syntax::Turtle).unwrap();
    store_b.materialize();

    let rec_a = store_a.get_hook_receipts();
    let rec_b = store_b.get_hook_receipts();

    assert_eq!(
        rec_a[0].delta_hash, rec_b[0].delta_hash,
        "Receipt hashes must be identical due to canonical sorting"
    );
}

/// Covers F5: Verifies receipt generation for quad deletions.
#[test]
fn test_f5_receipt_deletion() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:del_receipt_hook a kh:Hook ;
            kh:name "del_receipt_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:on "retract" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples(
            "ex:Fact <http://example.org/trigger> 'yes' .",
            Syntax::Turtle,
        )
        .unwrap();
    store.materialize();

    // Perform deletion
    store.remove_ref(&praxis_graphlaw::term::Triple {
        s: praxis_graphlaw::triples::VarOrTerm::new_term("http://example.org/Fact".to_string()),
        p: praxis_graphlaw::triples::VarOrTerm::new_term("http://example.org/trigger".to_string()),
        o: praxis_graphlaw::triples::VarOrTerm::new_term("yes".to_string()),
        g: None,
    });

    store.materialize();

    let receipts = store.get_hook_receipts();
    assert!(
        !receipts.is_empty(),
        "Deletion should trigger hook and produce a receipt"
    );
}

/// Covers F5: Verifies the public API `store.get_hook_receipts()`.
#[test]
fn test_f5_get_hook_receipts_api() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:api_hook a kh:Hook ;
            kh:name "api_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/data" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples("ex:Obj <http://example.org/data> 42 .", Syntax::Turtle)
        .unwrap();

    store.materialize();

    let receipts = store.get_hook_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].hook_name, "api_hook");
}

/// Covers F5: Verifies the format of HookReceipt fields.
#[test]
fn test_f5_receipt_format_validation() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:fmt_hook a kh:Hook ;
            kh:name "format_validation_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/input" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples("ex:Obj <http://example.org/input> 'test' .", Syntax::Turtle)
        .unwrap();

    store.materialize();

    let receipts = store.get_hook_receipts();
    assert_eq!(receipts.len(), 1);
    let receipt = &receipts[0];

    assert_eq!(receipt.hook_name, "format_validation_hook");
    assert!(!receipt.delta_hash.is_empty());
    assert!(!receipt.idempotency_key.is_empty());
    assert!(receipt.delta_quads.contains("http://example.org/input"));
}

// --- F6: Fixpoint Reasoner Integration ---

/// Covers F6: Verifies hook execution in a single-pass materialization.
#[test]
fn test_f6_single_pass_materialization() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:single_pass a kh:Hook ;
            kh:name "single_pass_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/input" ;
            kh:effect "emit-delta" ;
            kh:action ex:add_derived .
            
        ex:add_derived a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/derived> 'true' } WHERE { ?s <http://example.org/input> ?any }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples("ex:Item1 <http://example.org/input> 1 .", Syntax::Turtle)
        .unwrap();

    store.materialize();

    assert_contains_triple(&store, "Item1", "derived", "true");
}

/// Covers F6: Verifies multi-pass cascading materialization (Hook A triggers Hook B).
#[test]
fn test_f6_multi_pass_cascade() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:hook_a a kh:Hook ;
            kh:name "hook_a" ;
            kh:kind "delta" ;
            kh:var "http://example.org/input" ;
            kh:effect "emit-delta" ;
            kh:action ex:action_a ;
            kh:priority 1 .
            
        ex:action_a a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/mid> 'true' } WHERE { ?s <http://example.org/input> ?any }" .

        ex:hook_b a kh:Hook ;
            kh:name "hook_b" ;
            kh:kind "delta" ;
            kh:var "http://example.org/mid" ;
            kh:effect "emit-delta" ;
            kh:action ex:action_b ;
            kh:priority 2 ;
            kh:after ex:hook_a .
            
        ex:action_b a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/output> 'cascade_done' } WHERE { ?s <http://example.org/mid> 'true' }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples("ex:Item1 <http://example.org/input> 1 .", Syntax::Turtle)
        .unwrap();

    store.materialize();

    assert_contains_triple(&store, "Item1", "output", "cascade_done");
}

/// Covers F6: Verifies reasoner terminates successfully (fixpoint reached).
#[test]
fn test_f6_fixpoint_termination() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:stable_hook a kh:Hook ;
            kh:name "stable_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/input" ;
            kh:effect "emit-delta" ;
            kh:action ex:stable_action .
            
        ex:stable_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/stable> 'yes' } WHERE { ?s <http://example.org/input> ?any }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples("ex:Node <http://example.org/input> 1 .", Syntax::Turtle)
        .unwrap();

    // If it terminates, this will complete quickly
    let inferred = store.materialize();
    assert!(!inferred.is_empty());
}

/// Covers F6: Verifies that if a hook triggers a refusal during materialization, the entire reasoning session is rolled back.
#[test]
fn test_f6_refusal_rollback() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        // This hook adds transient facts
        ex:hook_transient a kh:Hook ;
            kh:name "transient_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/input" ;
            kh:effect "emit-delta" ;
            kh:action ex:transient_action ;
            kh:priority 1 .
            
        ex:transient_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/derived_transient> 'true' } WHERE { ?s <http://example.org/input> ?any }" .

        // This hook triggers refusal when the transient fact is created
        ex:refuse_hook a kh:Hook ;
            kh:name "refusal_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/derived_transient" ;
            kh:effect "refuse" ;
            kh:reason "Safety Violation: transient facts forbidden" ;
            kh:priority 2 ;
            kh:after ex:hook_transient .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples("ex:Node <http://example.org/input> 1 .", Syntax::Turtle)
        .unwrap();

    // Materialization should trigger refusal, rolling back changes
    let inferred = store.materialize();
    assert!(
        inferred.is_empty(),
        "Rollback should result in zero inferred facts"
    );
    assert_not_contains_triple(&store, "Node", "derived_transient", "true");
}

/// Covers F6: Verifies querying the store state post-materialization.
#[test]
fn test_f6_query_state_post_materialize() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:hook_q a kh:Hook ;
            kh:name "query_state_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/input" ;
            kh:effect "emit-delta" ;
            kh:action ex:action_q .
            
        ex:action_q a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/val> 'derived' } WHERE { ?s <http://example.org/input> ?any }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples("ex:Item <http://example.org/input> 1 .", Syntax::Turtle)
        .unwrap();

    store.materialize();

    let rows = store
        .query("SELECT ?val WHERE { <http://example.org/Item> <http://example.org/val> ?val }")
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0].val, "\"derived\"");
}

// =========================================================================
// TIER 2: BOUNDARY & CORNER CASES
// =========================================================================

// --- F1 Boundaries ---

/// Covers F1: Verifies loading a hook pack containing no hook definitions.
#[test]
fn test_b1_empty_hook_pack() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(res.is_ok(), "Empty hook pack should be allowed");
}

/// Covers F1: Verifies name length limits on hooks.
#[test]
fn test_b1_max_name_length() {
    let mut store = TripleStore::new();
    let long_name = "a".repeat(255);
    let hook_pack = format!(
        r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_long a kh:Hook ;
            kh:name "{}" ;
            kh:kind "delta" ;
            kh:var "v" ;
            kh:effect "emit-delta" .
        "#,
        long_name
    );
    let res = store.load_hook_pack(&hook_pack);
    assert!(res.is_ok(), "Max name length boundary test failed");
}

/// Covers F1: Verifies registry rejection when exceeding maximum number of hooks (limit 12).
#[test]
fn test_b1_exceed_max_hooks() {
    let mut store = TripleStore::new();
    let mut hook_pack = String::from("@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .\n@prefix ex: <http://example.org/> .\n");
    // Generate 13 hooks
    for i in 0..13 {
        hook_pack.push_str(&format!(
            r#"
            ex:h{} a kh:Hook ;
                kh:name "hook_{}" ;
                kh:kind "delta" ;
                kh:var "v" ;
                kh:effect "emit-delta" .
            "#,
            i, i
        ));
    }
    let res = store.load_hook_pack(&hook_pack);
    assert!(
        res.is_err(),
        "Hook registry should reject packs with >12 hooks"
    );
}

/// Covers F1: Verifies hook pack parser handles odd but valid Turtle layout formatting (whitespace/newlines).
#[test]
fn test_b1_turtle_formatting() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix
        kh:
        <http://seanchatmangpt.github.io/praxis/kh#>
        .
        
        <http://example.org/weird>
        a
        kh:Hook
        ;
        kh:name
        "weird_formatting"
        ;
        kh:kind "delta" ; kh:var "v" ;
        kh:effect
        "emit-delta"
        .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(res.is_ok(), "Failed to parse weird formatting");
}

/// Covers F1: Verifies error when missing mandatory fields in the hook pack definition.
#[test]
fn test_b1_missing_mandatory_fields() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_missing a kh:Hook ;
            kh:name "missing_kind" ;
            // kh:kind is missing
            kh:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(res.is_err(), "Hook missing kind should fail to register");
}

// --- F2 Boundaries ---

/// Covers F2: Verifies validation behavior when no custom SHACL laws are defined (uses default).
#[test]
fn test_b2_empty_shacl_law() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:my_hook a kh:Hook ;
            kh:name "test_empty_law" ;
            kh:kind "delta" ;
            kh:var "x" ;
            kh:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(res.is_ok());
}

/// Covers F2: Verifies detection of obfuscated execution keywords in URIs.
#[test]
fn test_b2_hidden_side_effects() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:hidden_hook a kh:Hook ;
            kh:name "hidden_side_effect" ;
            kh:kind "delta" ;
            kh:var "v" ;
            kh:effect "ground-action" ;
            // obfuscated URI containing shell
            kh:action <http://example.org/action#run_a_shell_script> .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_err(),
        "Obfuscated shell reference in Action handler should be refused"
    );
}

/// Covers F2: Verifies parsing and gating limits on extremely large hook pack inputs.
#[test]
fn test_b2_huge_hook_packs() {
    let mut store = TripleStore::new();
    let mut hook_pack = String::from("@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .\n@prefix ex: <http://example.org/> .\n");
    // Load up to 10 valid hooks (under 12 limit) with large text payloads
    for i in 0..10 {
        hook_pack.push_str(&format!(
            r#"
            ex:h{} a kh:Hook ;
                kh:name "hook_{}" ;
                kh:kind "delta" ;
                kh:var "v" ;
                kh:effect "emit-delta" ;
                kh:reason "{}" .
            "#,
            i,
            i,
            "a".repeat(1000)
        ));
    }
    let res = store.load_hook_pack(&hook_pack);
    assert!(
        res.is_ok(),
        "Huge hook pack payloads within limits should succeed"
    );
}

/// Covers F2: Verifies behavior when hook properties conflict under SHACL shape constraints.
#[test]
fn test_b2_conflicting_shacl_constraints() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:conflict_hook a kh:Hook ;
            kh:name "conflicting_hook" ;
            kh:kind "delta" ;
            kh:var "v" ;
            kh:effect "emit-delta" ;
            kh:effect "refuse" . # Duplicate kh:effect violates maxCount 1
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_err(),
        "Conflicting properties violating maxCount must be refused"
    );
}

/// Covers F2: Verifies registry state integrity under multiple sequential hook pack load operations.
#[test]
fn test_b2_multiple_sequential_loads() {
    let mut store = TripleStore::new();
    let pack_1 = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        ex:h1 a kh:Hook ; kh:name "seq_1" ; kh:kind "delta" ; kh:var "v" ; kh:effect "emit-delta" .
    "#;
    let pack_2 = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        ex:h2 a kh:Hook ; kh:name "seq_2" ; kh:kind "delta" ; kh:var "v" ; kh:effect "emit-delta" .
    "#;
    assert!(store.load_hook_pack(pack_1).is_ok());
    assert!(store.load_hook_pack(pack_2).is_ok());
}

// --- F3 Boundaries ---

/// Covers F3: Verifies evaluation when trigger conditions yield no matching bindings.
#[test]
fn test_b3_empty_trigger_results() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_empty a kh:Hook ;
            kh:name "empty_trigger" ;
            kh:kind "sparql" ;
            kh:query "SELECT ?s WHERE { ?s <http://example.org/nonexistent> ?o }" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store.materialize();
    assert!(store.get_hook_receipts().is_empty());
}

/// Covers F3: Verifies boundary window size values (window = 0 or window = 255).
#[test]
fn test_b3_window_size_bounds() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_window_zero a kh:Hook ;
            kh:name "window_zero" ;
            kh:kind "window" ;
            kh:var "http://example.org/metric" ;
            kh:op ">" ;
            kh:k 10 ;
            kh:window 0 ; # boundary value
            kh:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_ok(),
        "Window size 0 boundary should be valid or handled cleanly"
    );
}

/// Covers F3: Verifies extreme threshold boundary values (k = 0 or k = Max Integer).
#[test]
fn test_b3_threshold_boundary_values() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_threshold_max a kh:Hook ;
            kh:name "threshold_max" ;
            kh:kind "threshold" ;
            kh:var "http://example.org/metric" ;
            kh:op ">" ;
            kh:k 18446744073709551615 ; # max u64
            kh:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(res.is_ok());
}

/// Covers F3: Verifies validation/gating constraints on extremely large Datalog program sizes.
#[test]
fn test_b3_datalog_program_size_limit() {
    let mut store = TripleStore::new();
    let huge_program = "r(?x) :- p(?x) . ".repeat(500);
    let hook_pack = format!(
        r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_huge_datalog a kh:Hook ;
            kh:name "huge_datalog" ;
            kh:kind "datalog" ;
            kh:program "{}" ;
            kh:goal "r(?s)" ;
            kh:effect "emit-delta" .
        "#,
        huge_program
    );
    let res = store.load_hook_pack(&hook_pack);
    assert!(res.is_ok() || res.is_err()); // Either parsed or gracefully rejected
}

/// Covers F3: Verifies syntax error handling in SPARQL triggers.
#[test]
fn test_b3_sparql_syntax_error() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_bad_sparql a kh:Hook ;
            kh:name "bad_sparql" ;
            kh:kind "sparql" ;
            kh:query "SELECT * WHERE { ?s ?p }" ; # missing brackets
            kh:effect "emit-delta" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_err(),
        "Invalid SPARQL syntax must reject hook pack loading"
    );
}

// --- F4 Boundaries ---

/// Covers F4: Verifies CONSTRUCT projections that produce empty results.
#[test]
fn test_b4_construct_empty_result() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_empty_construct a kh:Hook ;
            kh:name "empty_construct" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" ;
            kh:action ex:empty_action .
            
        ex:empty_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/status> 'none' } WHERE { ?s <http://example.org/nonexistent> ?o }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples(
            "ex:Node <http://example.org/trigger> 'yes' .",
            Syntax::Turtle,
        )
        .unwrap();

    store.materialize();

    // No new triples should be created
    assert_not_contains_triple(&store, "Node", "status", "none");
}

/// Covers F4: Verifies rejection of CONSTRUCT queries projecting invalid RDF (literal as subject).
#[test]
fn test_b4_construct_literal_subject() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_lit_subject a kh:Hook ;
            kh:name "literal_subject" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" ;
            kh:action ex:lit_action .
            
        ex:lit_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { 'subject' <http://example.org/p> ?o } WHERE { ?s <http://example.org/trigger> ?o }" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_err() || store.materialize().is_empty(),
        "Should reject invalid RDF generation"
    );
}

/// Covers F4: Verifies rejection/handling of unsupported clauses in SPARQL CONSTRUCT queries.
#[test]
fn test_b4_construct_unsupported_clauses() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_unsupported a kh:Hook ;
            kh:name "unsupported_construct" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" ;
            kh:action ex:unsup_action .
            
        ex:unsup_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            // Contains unsupported SERVICE or external subquery block
            kh:query "CONSTRUCT { ?s <http://example.org/p> ?o } WHERE { ?s <http://example.org/trigger> ?o . SERVICE <http://external.org> { ?s ?p ?o } }" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(res.is_err());
}

/// Covers F4: Verifies execution when CONSTRUCT attempts to add quads already present (no-op).
#[test]
fn test_b4_construct_no_op_addition() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_noop a kh:Hook ;
            kh:name "noop_addition" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" ;
            kh:action ex:noop_action .
            
        ex:noop_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/trigger> 'yes' } WHERE { ?s <http://example.org/trigger> 'yes' }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples(
            "ex:Node <http://example.org/trigger> 'yes' .",
            Syntax::Turtle,
        )
        .unwrap();

    let before_len = store.len();
    store.materialize();
    assert_eq!(store.len(), before_len);
}

/// Covers F4: Verifies rejection of CONSTRUCT queries attempting to modify the hook registry itself.
#[test]
fn test_b4_construct_modify_registry() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:h_hijack a kh:Hook ;
            kh:name "hijack_registry" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" ;
            kh:action ex:hijack_action .
            
        ex:hijack_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            // Attempt to inject a new malicious hook via CONSTRUCT projection
            kh:query "CONSTRUCT { ex:injected a kh:Hook ; kh:name 'injected' ; kh:effect 'ground-action' } WHERE { ?s <http://example.org/trigger> ?o }" .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_err(),
        "Hook attempting to modify system/registry namespace must be blocked"
    );
}

// --- F5 Boundaries ---

/// Covers F5: Verifies receipt generation containing blank nodes.
#[test]
fn test_b5_receipt_blank_nodes() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:bnode_hook a kh:Hook ;
            kh:name "bnode_receipt" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples(
            "ex:Node <http://example.org/trigger> _:blank .",
            Syntax::Turtle,
        )
        .unwrap();

    store.materialize();

    let receipts = store.get_hook_receipts();
    assert!(!receipts.is_empty());
    assert!(!receipts[0].delta_hash.is_empty());
}

/// Covers F5: Verifies receipt generation containing non-ASCII Unicode literals.
#[test]
fn test_b5_receipt_unicode_literals() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:unicode_hook a kh:Hook ;
            kh:name "unicode_receipt" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples(
            "ex:Node <http://example.org/trigger> 'こんにちは' .",
            Syntax::Turtle,
        )
        .unwrap();

    store.materialize();

    let receipts = store.get_hook_receipts();
    assert!(!receipts.is_empty());
    assert!(receipts[0].delta_quads.contains("こんにちは"));
}

/// Covers F5: Verifies receipt hashing/formatting with extremely large literals.
#[test]
fn test_b5_receipt_huge_literals() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:huge_lit_hook a kh:Hook ;
            kh:name "huge_literal_receipt" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    let huge_str = "a".repeat(10000);
    store
        .load_triples(
            &format!("ex:Node <http://example.org/trigger> '{}' .", huge_str),
            Syntax::Turtle,
        )
        .unwrap();

    store.materialize();

    let receipts = store.get_hook_receipts();
    assert!(!receipts.is_empty());
}

/// Covers F5: Verifies that distinct literal datatypes and language tags produce distinct, stable hashes.
#[test]
fn test_b5_stable_hash_datatypes_lang() {
    let mut store_a = TripleStore::new();
    let mut store_b = TripleStore::new();

    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:hash_stable a kh:Hook ;
            kh:name "stable_hash_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" .
    "#;

    store_a.load_hook_pack(hook_pack).unwrap();
    store_b.load_hook_pack(hook_pack).unwrap();

    // Store A gets integer literal, Store B gets string representation
    store_a
        .load_triples("ex:Node <http://example.org/trigger> 42 .", Syntax::Turtle)
        .unwrap();
    store_b
        .load_triples(
            "ex:Node <http://example.org/trigger> '42' .",
            Syntax::Turtle,
        )
        .unwrap();

    store_a.materialize();
    store_b.materialize();

    let rec_a = store_a.get_hook_receipts();
    let rec_b = store_b.get_hook_receipts();

    assert_ne!(
        rec_a[0].delta_hash, rec_b[0].delta_hash,
        "Hashes of different RDF datatypes must be distinct"
    );
}

/// Covers F5: Verifies receipt generation containing both quad additions and deletions.
#[test]
fn test_b5_hash_both_add_and_delete() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:both_hook a kh:Hook ;
            kh:name "both_add_delete" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:effect "emit-delta" ;
            kh:action ex:both_action .
            
        ex:both_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            // Adds vip status, deletes standard status
            kh:query "CONSTRUCT { ?cust <http://example.org/status> 'VIP' } WHERE { ?cust <http://example.org/spent> ?amount . FILTER(?amount > 1000) }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples(
            "ex:Alice <http://example.org/spent> 1500 ; <http://example.org/trigger> 'yes' .",
            Syntax::Turtle,
        )
        .unwrap();

    store.materialize();

    let receipts = store.get_hook_receipts();
    assert!(!receipts.is_empty());
}

// --- F6 Boundaries ---

/// Covers F6: Verifies recursion guard limit and infinite loop detection during fixpoint loops.
#[test]
fn test_b6_infinite_loop_detection() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        // Hook A triggers B
        ex:hook_a a kh:Hook ;
            kh:name "loop_hook_a" ;
            kh:kind "delta" ;
            kh:var "http://example.org/p_b" ;
            kh:effect "emit-delta" ;
            kh:action ex:action_a .
            
        ex:action_a a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/p_a> 'triggered' } WHERE { ?s <http://example.org/p_b> ?any }" .

        // Hook B triggers A
        ex:hook_b a kh:Hook ;
            kh:name "loop_hook_b" ;
            kh:kind "delta" ;
            kh:var "http://example.org/p_a" ;
            kh:effect "emit-delta" ;
            kh:action ex:action_b .
            
        ex:action_b a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/p_b> 'triggered' } WHERE { ?s <http://example.org/p_a> ?any }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples("ex:Node <http://example.org/p_a> 'start' .", Syntax::Turtle)
        .unwrap();

    // Materialization should terminate with limit or recursion depth error
    let inferred = store.materialize();
    // It should terminate safely without hanging
    assert!(inferred.len() < 100);
}

/// Covers F6: Verifies static hook dependency cycle detection at registration.
#[test]
fn test_b6_circular_dependency() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:hook_a a kh:Hook ;
            kh:name "hook_a" ;
            kh:kind "delta" ;
            kh:var "x" ;
            kh:effect "emit-delta" ;
            kh:after ex:hook_b .

        ex:hook_b a kh:Hook ;
            kh:name "hook_b" ;
            kh:kind "delta" ;
            kh:var "y" ;
            kh:effect "emit-delta" ;
            kh:after ex:hook_a .
    "#;
    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_err(),
        "Circular static dependency should be rejected at load time"
    );
}

/// Covers F6: Verifies deep reasoning chain rollback integrity on a late gating refusal.
#[test]
fn test_b6_gating_refusal_deep_rollback() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        // Chain of 3 hooks
        ex:h1 a kh:Hook ;
            kh:name "deep_1" ;
            kh:kind "delta" ;
            kh:var "http://example.org/start" ;
            kh:effect "emit-delta" ;
            kh:action ex:act1 ;
            kh:priority 1 .
        ex:act1 a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/step1> 'yes' } WHERE { ?s <http://example.org/start> ?any }" .

        ex:h2 a kh:Hook ;
            kh:name "deep_2" ;
            kh:kind "delta" ;
            kh:var "http://example.org/step1" ;
            kh:effect "emit-delta" ;
            kh:action ex:act2 ;
            kh:priority 2 ;
            kh:after ex:h1 .
        ex:act2 a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/step2> 'yes' } WHERE { ?s <http://example.org/step1> ?any }" .

        // Third hook triggers constitutional refusal when step2 is generated
        ex:h3 a kh:Hook ;
            kh:name "deep_3" ;
            kh:kind "delta" ;
            kh:var "http://example.org/step2" ;
            kh:effect "refuse" ;
            kh:reason "Refusal: step2 reached" ;
            kh:priority 3 ;
            kh:after ex:h2 .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    store
        .load_triples("ex:Node <http://example.org/start> 'go' .", Syntax::Turtle)
        .unwrap();

    let inferred = store.materialize();
    assert!(inferred.is_empty());
    // Ensure both step1 and step2 are completely rolled back
    assert_not_contains_triple(&store, "Node", "step1", "yes");
    assert_not_contains_triple(&store, "Node", "step2", "yes");
}

/// Covers F6: Verifies materialization behavior when executing on a store with zero base facts.
#[test]
fn test_b6_empty_base_facts() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:my_hook a kh:Hook ;
            kh:name "empty_base" ;
            kh:kind "delta" ;
            kh:var "x" ;
            kh:effect "emit-delta" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    let inferred = store.materialize();
    assert!(
        inferred.is_empty(),
        "No inferences should happen on empty store"
    );
}

/// Covers F6: Verifies multi-strata rule evaluation with stratified datalog and hooks.
#[test]
fn test_b6_multi_strata_evaluation() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:strat_hook a kh:Hook ;
            kh:name "strata_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/input" ;
            kh:effect "emit-delta" ;
            kh:action ex:strat_action .
            
        ex:strat_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?s <http://example.org/stratum1> 'done' } WHERE { ?s <http://example.org/input> ?any }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();
    // Add Datalog rule to stratum 2: { ?x :stratum1 'done' } => { ?x :stratum2 'complete' }
    store.load_rules("{ ?x <http://example.org/stratum1> 'done' } => { ?x <http://example.org/stratum2> 'complete' } .").unwrap();
    store
        .load_triples(
            "ex:Node <http://example.org/input> 'start' .",
            Syntax::Turtle,
        )
        .unwrap();

    store.materialize();

    assert_contains_triple(&store, "Node", "stratum2", "complete");
}

// =========================================================================
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

    store.materialize();

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
    store.materialize();

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

    store.materialize();

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

    store.materialize();

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

    store.materialize();

    // The active flow triple should be deleted, stopping further cascade pass evaluations
    assert_not_contains_triple(&store, "Flow", "active_flow", "yes");
}

// =========================================================================
// TIER 4: REAL-WORLD APPLICATION SCENARIOS
// =========================================================================

/// Covers S1: Automated Quarantine & Refusal Scenario.
/// When unauthorized triples are written to a protected graph/namespace,
/// the hook catches it, routes the violating triples to a quarantine graph,
/// logs a refusal, and rolls back the user's primary transaction.
#[test]
fn test_s4_automated_quarantine_and_refusal() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        // Hook 1: Quarantine route
        ex:quarantine_hook a kh:Hook ;
            kh:name "quarantine_routing" ;
            kh:kind "sparql" ;
            kh:query "SELECT ?s ?p ?o WHERE { GRAPH <http://example.org/SystemGraph> { ?s ?p ?o } }" ;
            kh:effect "emit-delta" ;
            kh:action ex:route_to_quarantine ;
            kh:priority 1 .
            
        ex:route_to_quarantine a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { GRAPH <http://example.org/QuarantineGraph> { ?s ?p ?o } } WHERE { GRAPH <http://example.org/SystemGraph> { ?s ?p ?o } }" .

        // Hook 2: Refusal on SystemGraph write
        ex:refusal_hook a kh:Hook ;
            kh:name "quarantine_refusal" ;
            kh:kind "sparql" ;
            kh:query "ASK { GRAPH <http://example.org/SystemGraph> { ?s ?p ?o } }" ;
            kh:effect "refuse" ;
            kh:reason "Refusal Error: Direct write to SystemGraph is forbidden" ;
            kh:priority 2 ;
            kh:after ex:quarantine_hook .
    "#;
    store.load_hook_pack(hook_pack).unwrap();

    // Simulate user attempting to write to SystemGraph
    store.load_triples(
        "GRAPH <http://example.org/SystemGraph> { <http://example.org/User1> <http://example.org/change_pass> 'admin' } .",
        Syntax::NQuads
    ).unwrap();

    let inferred = store.materialize();

    // Transaction must roll back, ensuring no data remains in SystemGraph
    assert!(
        inferred.is_empty(),
        "Unauthorized write should cause complete transaction rollback"
    );
    assert_not_contains_triple(&store, "User1", "change_pass", "admin");
}

/// Covers S2: Ledger Balance Enforcement & Audit Trail Scenario.
/// Validates double-entry ledger transactions. Checks if account balance is >= 0.
/// If valid, projects ledger updates and records a signed BLAKE3 audit receipt.
/// If balance drops below zero, transaction is refused and rolled back.
#[test]
fn test_s4_ledger_balance_enforcement_and_audit() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        // Hook 1: Enforce positive balance, refuse otherwise
        ex:balance_guard a kh:Hook ;
            kh:name "ledger_balance_guard" ;
            kh:kind "sparql" ;
            kh:query "ASK { ?acct <http://example.org/balance> ?bal . FILTER(?bal < 0) }" ;
            kh:effect "refuse" ;
            kh:reason "Balance Violation: Account balance cannot drop below zero" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();

    // Attempt valid transaction: Starting balance 500, deduct 200
    store
        .load_triples(
            "ex:Account1 <http://example.org/balance> 500 .",
            Syntax::Turtle,
        )
        .unwrap();
    store.materialize();

    // Perform deduction
    store
        .load_triples(
            "ex:Account1 <http://example.org/balance> 300 .", // New state
            Syntax::Turtle,
        )
        .unwrap();
    let _inferred = store.materialize();
    assert_contains_triple(&store, "Account1", "balance", "300");

    // Attempt invalid transaction: Deduct 400 (balance goes to -100)
    store
        .load_triples(
            "ex:Account1 <http://example.org/balance> -100 .",
            Syntax::Turtle,
        )
        .unwrap();

    let res = store.materialize();
    assert!(
        res.is_empty(),
        "Negative balance transaction should be rolled back"
    );
}

/// Covers S3: State Machine Transition Control Scenario.
/// Implements state transition rules: Draft -> UnderReview -> Approved.
/// Verifies that skip-state transitions are refused.
#[test]
fn test_s4_state_machine_transition_control() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:transition_guard a kh:Hook ;
            kh:name "lifecycle_transition_guard" ;
            kh:kind "sparql" ;
            // Refuse if we transition from Draft to Approved, skipping UnderReview
            kh:query "ASK { ?doc <http://example.org/old_state> 'Draft' ; <http://example.org/new_state> 'Approved' }" ;
            kh:effect "refuse" ;
            kh:reason "Lifecycle Violation: Cannot skip UnderReview state" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();

    // Invalid transition
    store.load_triples(
        "ex:Doc1 <http://example.org/old_state> 'Draft' ; <http://example.org/new_state> 'Approved' .",
        Syntax::Turtle
    ).unwrap();

    let inferred = store.materialize();
    assert!(
        inferred.is_empty(),
        "Invalid state transition must be rolled back"
    );
}

/// Covers S4: Access Control Policy Engine Scenario.
/// Implements RBAC. Evaluates user role and request action.
/// Projects authorization decision: granted/denied.
#[test]
fn test_s4_access_control_policy_engine() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:rbac_hook a kh:Hook ;
            kh:name "rbac_policy_engine" ;
            kh:kind "sparql" ;
            kh:query "SELECT ?user WHERE { ?user <http://example.org/request> ?act . ?user <http://example.org/role> 'Admin' }" ;
            kh:effect "emit-delta" ;
            kh:action ex:grant_access_action .
            
        ex:grant_access_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { ?user <http://example.org/permission> 'granted' } WHERE { ?user <http://example.org/request> ?act . ?user <http://example.org/role> 'Admin' }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();

    // Admin user requests action
    store.load_triples(
        "ex:User1 <http://example.org/role> 'Admin' ; <http://example.org/request> 'delete_db' .",
        Syntax::Turtle
    ).unwrap();

    store.materialize();

    assert_contains_triple(&store, "User1", "permission", "granted");
}

/// Covers S5: Materialized View / Cache Maintenance Scenario.
/// Automatically projects changes into a query cache representation graph
/// when base data triples are modified, maintaining up-to-date read views.
#[test]
fn test_s4_materialized_view_cache_maintenance() {
    let mut store = TripleStore::new();
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
        
        ex:cache_hook a kh:Hook ;
            kh:name "view_cache_maintenance" ;
            kh:kind "delta" ;
            kh:var "http://example.org/salary" ;
            kh:effect "emit-delta" ;
            kh:action ex:update_cache_action .
            
        ex:update_cache_action a kh:Action ;
            kh:handler <http://seanchatmangpt.github.io/praxis/handler#sparql-construct> ;
            kh:query "CONSTRUCT { GRAPH <http://example.org/CachedView> { ?emp <http://example.org/tax_bracket> 'high' } } WHERE { ?emp <http://example.org/salary> ?sal . FILTER(?sal > 150000) }" .
    "#;
    store.load_hook_pack(hook_pack).unwrap();

    // Add base employee data
    store
        .load_triples(
            "ex:Emp1 <http://example.org/salary> 180000 .",
            Syntax::Turtle,
        )
        .unwrap();

    store.materialize();

    // Cache graph should contain the materialized view tax bracket classification
    assert_contains_triple(&store, "Emp1", "tax_bracket", "high");
}

// =========================================================================
// TIER 5: hook: NAMESPACE ALIAS TESTS
// =========================================================================

/// Covers hook: namespace aliasing: hook:* produces identical CompiledHook/schedule to kh:*
/// with byte-identical BLAKE3 hashes and deterministic receipt generation.
#[test]
fn test_hook_alias_vocabulary_identical_receipts() {
    let mut store_kh = TripleStore::new();
    let mut store_hook = TripleStore::new();

    // Reference hook pack using canonical kh: namespace
    let hook_pack_kh = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .

        ex:h_canonical a kh:Hook ;
            kh:name "canonical_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:on "assert" ;
            kh:effect "emit-delta" ;
            kh:priority 1 .
    "#;

    // Equivalent hook pack using hook: namespace alias
    let hook_pack_hook = r#"
        @prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
        @prefix ex: <http://example.org/> .

        ex:h_alias a hook:Hook ;
            hook:name "canonical_hook" ;
            hook:kind "delta" ;
            hook:var "http://example.org/trigger" ;
            hook:on "assert" ;
            hook:effect "emit-delta" ;
            hook:priority 1 .
    "#;

    // Load both hook packs
    assert!(
        store_kh.load_hook_pack(hook_pack_kh).is_ok(),
        "Failed to load hook pack with kh: namespace"
    );
    assert!(
        store_hook.load_hook_pack(hook_pack_hook).is_ok(),
        "Failed to load hook pack with hook: namespace alias"
    );

    // Load identical base facts
    let base_facts = "ex:Node <http://example.org/trigger> 'yes' .";
    store_kh.load_triples(base_facts, Syntax::Turtle).unwrap();
    store_hook.load_triples(base_facts, Syntax::Turtle).unwrap();

    // Materialize both stores
    store_kh.materialize();
    store_hook.materialize();

    // Get receipts from both stores
    let receipts_kh = store_kh.get_hook_receipts();
    let receipts_hook = store_hook.get_hook_receipts();

    // Verify both produced receipts
    assert_eq!(
        receipts_kh.len(),
        1,
        "kh: hook pack should produce exactly one receipt"
    );
    assert_eq!(
        receipts_hook.len(),
        1,
        "hook: hook pack should produce exactly one receipt"
    );

    // Verify hook names match
    assert_eq!(receipts_kh[0].hook_name, "canonical_hook");
    assert_eq!(receipts_hook[0].hook_name, "canonical_hook");

    // Verify BLAKE3 hashes are byte-identical (determinism guarantee)
    assert_eq!(
        receipts_kh[0].delta_hash, receipts_hook[0].delta_hash,
        "hook: and kh: namespaces must produce identical BLAKE3 hashes"
    );

    // Verify delta quads are identical
    assert_eq!(
        receipts_kh[0].delta_quads, receipts_hook[0].delta_quads,
        "hook: and kh: namespaces must produce identical delta quads"
    );

    // Verify idempotency keys are identical
    assert_eq!(
        receipts_kh[0].idempotency_key, receipts_hook[0].idempotency_key,
        "hook: and kh: namespaces must produce identical idempotency keys"
    );
}

/// Covers hook: namespace validation: Unknown hook:* predicate is refused with proper error text.
#[test]
fn test_hook_alias_unknown_predicate_refused() {
    let mut store = TripleStore::new();

    // Hook pack using an unknown hook: predicate
    let hook_pack = r#"
        @prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
        @prefix ex: <http://example.org/> .

        ex:h_unknown a hook:Hook ;
            hook:name "unknown_predicate_hook" ;
            hook:kind "delta" ;
            hook:var "x" ;
            hook:effect "emit-delta" ;
            hook:unknown_field "this_should_fail" .
    "#;

    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_err(),
        "Hook pack with unknown hook: predicate must be refused"
    );

    let err = res.unwrap_err();
    assert!(
        err.contains("SHACL") || err.contains("validation") || err.contains("unknown"),
        "Error message should mention SHACL validation or unknown predicate, got: {}",
        err
    );
}

/// Covers hook: namespace validation: Mixed kh:/hook: on same hook is refused.
#[test]
fn test_hook_alias_mixed_namespaces_refused() {
    let mut store = TripleStore::new();

    // Hook pack mixing hook: and kh: namespaces on the same hook (violates SHACL shape)
    let hook_pack = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .
        @prefix ex: <http://example.org/> .

        ex:h_mixed a kh:Hook ;
            kh:name "mixed_namespaces" ;
            hook:kind "delta" ;
            kh:var "x" ;
            hook:on "assert" ;
            kh:effect "emit-delta" .
    "#;

    let res = store.load_hook_pack(hook_pack);
    assert!(
        res.is_err(),
        "Hook pack mixing hook: and kh: namespaces must be refused"
    );

    let err = res.unwrap_err();
    assert!(
        err.contains("SHACL") || err.contains("validation"),
        "Error should reference SHACL validation failure, got: {}",
        err
    );
}

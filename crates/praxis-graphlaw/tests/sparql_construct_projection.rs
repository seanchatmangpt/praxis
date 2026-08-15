use praxis_graphlaw::TripleStore;

#[test]
fn construct_instantiates_a_future_graph_without_mutating_source_state() {
    let store = TripleStore::from(
        r#"
        @prefix ex: <https://praxis.chatman.io/test#> .
        ex:activity ex:performs ex:function .
        "#,
    );
    let before = store.content_to_string();

    let constructed = store
        .construct(
            r#"
            PREFIX ex: <https://praxis.chatman.io/test#>
            CONSTRUCT {
              ?activity ex:hasFutureDisposition ex:Eliminate .
            }
            WHERE {
              ?activity ex:performs ex:function .
            }
            "#,
        )
        .expect("CONSTRUCT must instantiate its template from WHERE bindings");

    assert_eq!(constructed.len(), 1);
    let rendered = TripleStore::decode_triple(&constructed[0]);
    assert!(rendered.contains("activity"));
    assert!(rendered.contains("hasFutureDisposition"));
    assert!(rendered.contains("Eliminate"));
    assert_eq!(
        store.content_to_string(),
        before,
        "CONSTRUCT is a reversible graph projection and must not mutate law state"
    );
}

#[test]
fn construct_rejects_non_construct_queries() {
    let store = TripleStore::from(
        r#"
        @prefix ex: <https://praxis.chatman.io/test#> .
        ex:a ex:p ex:b .
        "#,
    );

    let error = store
        .construct(
            r#"
            PREFIX ex: <https://praxis.chatman.io/test#>
            SELECT ?s WHERE { ?s ex:p ex:b }
            "#,
        )
        .expect_err("construct() must refuse SELECT instead of pretending it made a graph");

    assert!(error.contains("CONSTRUCT"));
}

#[test]
fn construct_refuses_blank_node_templates_until_identity_is_explicit() {
    let store = TripleStore::from(
        r#"
        @prefix ex: <https://praxis.chatman.io/test#> .
        ex:a ex:p ex:b .
        "#,
    );

    let error = store
        .construct(
            r#"
            PREFIX ex: <https://praxis.chatman.io/test#>
            CONSTRUCT { [] ex:derivedFrom ?s }
            WHERE { ?s ex:p ex:b }
            "#,
        )
        .expect_err("anonymous constructed subjects need an explicit deterministic identity law");

    assert!(error.contains("blank node"));
}

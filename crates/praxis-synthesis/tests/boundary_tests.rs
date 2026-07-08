use praxis_synthesis::boundary::{
    execute_emit_delta, get_delta_template, project_delta_template, BoundaryRequest,
};
use praxis_synthesis::Reference;

const BASE: &str = "@prefix ex: <http://e/> .\n\
@prefix hook: <http://seanchatmangpt.github.io/praxis/hook#> .\n\
ex:my_action hook:adds_ttl \"<http://e/x> <http://e/p> ?0 .\" ;\n\
            hook:removes_ttl \"<http://e/a> <http://e/p> <http://e/b> .\" .\n\
<http://e/a> <http://e/p> <http://e/b> .\n";

#[test]
fn test_boundary_request_construction() {
    let reference = Reference::genesis(BASE).unwrap();
    let hook_iri = "http://e/my_hook";
    let hook_name = "test-hook";
    let event_hash = "event-123";
    let delta_ttl_hash = "ttl-456";
    let history_hash = "history-789";

    let req = BoundaryRequest::new(
        &reference,
        hook_iri,
        hook_name,
        event_hash,
        delta_ttl_hash,
        history_hash,
    );

    assert_eq!(req.state_epoch, 0);
    assert_eq!(req.hook_iri, hook_iri);
    assert_eq!(req.hook_name, hook_name);
    assert_eq!(req.event_hash, event_hash);
    assert_eq!(req.delta_ttl_hash, delta_ttl_hash);
    assert_eq!(req.freshness_token, history_hash);
    assert!(!req.idempotency_key.is_empty());

    // Verify determinism: constructing again must yield same idempotency key
    let req2 = BoundaryRequest::new(
        &reference,
        hook_iri,
        hook_name,
        event_hash,
        delta_ttl_hash,
        history_hash,
    );
    assert_eq!(req.idempotency_key, req2.idempotency_key);
}

#[test]
fn test_get_and_project_delta_template() {
    let reference = Reference::genesis(BASE).unwrap();
    let (adds_temp, removes_temp) = get_delta_template(reference.triples(), "http://e/my_action");
    assert_eq!(adds_temp, "<http://e/x> <http://e/p> ?0 .");
    assert_eq!(removes_temp, "<http://e/a> <http://e/p> <http://e/b> .");

    let (adds, removes) =
        project_delta_template(&adds_temp, &removes_temp, &["http://e/y".to_string()]);
    assert_eq!(adds, "<http://e/x> <http://e/p> <http://e/y> .");
    assert_eq!(removes, "<http://e/a> <http://e/p> <http://e/b> .");
}

#[test]
fn test_execute_emit_delta_reenters_quarantine_and_admits() {
    let reference = Reference::genesis(BASE).unwrap();
    let adds = "<http://e/new_fact> <http://e/p> 1 .";
    let removes = "<http://e/a> <http://e/p> <http://e/b> .";

    // This should successfully re-enter the quarantine gate and admit
    let admitted = execute_emit_delta(&reference, adds, removes).unwrap();
    assert_eq!(admitted.record().epoch, 1);
    assert_eq!(admitted.record().base_graph_hash, reference.graph_hash());
    assert_eq!(admitted.post().len(), 3);
}

use praxis_graphlaw::parser::Syntax;
use praxis_graphlaw::TripleStore;

const INDUSTRY: &str = include_str!("../innovation/industry-fixture.n3");
const LAWS: &str = include_str!("../innovation/blue-ocean-triz.n3");
const SHACL: &str = include_str!("../innovation/candidate.shacl.ttl");
const SHEX: &str = include_str!("../innovation/candidate.shex.json");
const OPPORTUNITY_QUERY: &str = include_str!("../innovation/opportunities.rq");
const CONSTRUCT_FUTURES: &str = include_str!("../innovation/construct-futures.rq");

const NS: &str = "https://praxis.chatman.io/innovation#";
const CANDIDATE: &str = "https://praxis.chatman.io/innovation/demo#direct-outcome";
const CANDIDATE_SHAPE: &str = "https://praxis.chatman.io/innovation#CandidateFutureShape";

fn crown_document() -> String {
    format!("{INDUSTRY}\n{LAWS}")
}

fn crown_store() -> TripleStore {
    TripleStore::from(&crown_document())
}

fn decoded(store: &TripleStore) -> String {
    store.content_to_string()
}

#[test]
fn crown_derives_full_errc_and_preserves_triz_alternatives() {
    let mut store = crown_store();
    store
        .materialize()
        .expect("innovation law closure must reach a deterministic fixpoint");
    let graph = decoded(&store);

    assert!(graph.contains("EliminateOpportunity"));
    assert!(graph.contains("ReduceOpportunity"));
    assert!(graph.contains("RaiseOpportunity"));
    assert!(graph.contains("direct-outcome"));
    assert!(graph.contains("CandidateFuture"));
    assert!(graph.contains("Create"));
    assert!(graph.contains("SeparationInTime"));
    assert!(graph.contains("Intermediary"));
    assert!(
        !graph.contains("selectedWinner"),
        "DfCM preserves lawful candidates; GraphLaw must not smuggle in a winner-selection act"
    );

    let opportunity_rows = store
        .query(OPPORTUNITY_QUERY)
        .expect("SPARQL opportunity discovery must remain executable");
    assert_eq!(
        opportunity_rows.len(),
        4,
        "the reference crown must expose exactly Eliminate, Reduce, Raise, and Create"
    );

    let denials = store.check_denials();
    assert!(
        denials.is_empty(),
        "the crown fixture is intended to be admissible, got {denials:?}"
    );
}

#[test]
fn crown_candidate_passes_shacl_and_shex_before_planning_handoff() {
    let mut store = crown_store();
    store.materialize().unwrap();

    let shacl = store
        .validate_shacl(SHACL)
        .expect("candidate SHACL shapes must parse");
    assert!(shacl.conforms, "SHACL admission must accept the complete crown future");

    let shex = store
        .validate_shex(
            SHEX,
            &[(CANDIDATE.to_string(), CANDIDATE_SHAPE.to_string())],
        )
        .expect("candidate ShEx grammar must parse");
    assert!(shex.conforms, "ShEx grammar must accept the complete crown future");
}

#[test]
fn crown_constructs_a_pddl_frontier_without_actuation_or_selection() {
    let mut store = crown_store();
    store.materialize().unwrap();

    let future = store
        .construct(CONSTRUCT_FUTURES)
        .expect("SPARQL CONSTRUCT must manufacture the planning frontier graph");
    let rendered = TripleStore::decode_triples(&future);

    assert!(rendered.contains("handoff"));
    assert!(rendered.contains("PDDL"));
    assert!(rendered.contains("AdmittedCandidate"));
    assert!(
        !rendered.contains("actuate") && !rendered.contains("selectedWinner"),
        "innovation crown ends at reversible planning candidates; DO remains behind BRCE"
    );
}

#[test]
fn denial_refuses_agent_mediation_when_an_equivalent_morphism_exists() {
    let mut store = TripleStore::from(&crown_document());
    store
        .load_triples(
            &format!(
                r#"
                @prefix inv: <{NS}> .
                @prefix demo: <https://praxis.chatman.io/innovation/demo#> .
                demo:bad-agent-future a inv:CandidateFuture ;
                    inv:agentMediation inv:Required ;
                    inv:hasEquivalentDeterministicMorphism inv:True .
                "#
            ),
            Syntax::Turtle,
        )
        .unwrap();
    store.materialize().unwrap();

    let denials = store.check_denials();
    assert!(
        denials.iter().any(|denial| denial.contains("agentMediation")),
        "Post-AGI law must refuse required agent mediation when a deterministic morphism exists"
    );
}

#[test]
fn incomplete_future_is_not_promoted_by_shacl() {
    let mut store = TripleStore::new();
    store
        .load_triples(
            &format!(
                r#"
                @prefix inv: <{NS}> .
                @prefix demo: <https://praxis.chatman.io/innovation/demo#> .
                demo:incomplete a inv:CandidateFuture .
                "#
            ),
            Syntax::Turtle,
        )
        .unwrap();

    let report = store.validate_shacl(SHACL).unwrap();
    assert!(!report.conforms, "an ungrounded idea is not an admitted future");
}

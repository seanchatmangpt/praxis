//! Integration tests for the cng library pipeline: seeded fixture rendering
//! (Turtle enters only from `.ttl` files — the template at
//! `tests/fixtures/pddl-pair.template.ttl`), the joseph many-to-one example,
//! and typed refusals.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use cng::pipeline::{generate_plan, import_artifacts, plan_id};
use cng::powl::{powl_to_turtle, project_tape_to_powl, CngRefusal, Powl, POWL2_PREFIX};
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

const BASE_IRI: &str = "urn:chatman:powl:cng-test";
const DERIVED_FROM: &str = "urn:chatman:plan:cng-test";
const JOSEPH_PLAN_ID: &str =
    "blake3:c025532cc3e9c1a96625dfa551f7ec8ba9b0af68138403540149805c7ec63749";

/// Seed-derived placeholder values for the fixture template. All names are
/// deterministic functions of the seed (blake3 of its LE bytes).
struct SeededNames {
    domain: String,
    obj: String,
    action_0: String,
    action_1: String,
    pred_0: String,
    pred_1: String,
    pred_2: String,
}

fn seeded_names(seed: u64) -> SeededNames {
    let hex = blake3::hash(&seed.to_le_bytes()).to_hex().to_string();
    let s = &hex[..8];
    SeededNames {
        domain: format!("dom-{s}"),
        obj: format!("obj-{s}"),
        action_0: format!("act0-{s}"),
        action_1: format!("act1-{s}"),
        pred_0: format!("pred0-{s}"),
        pred_1: format!("pred1-{s}"),
        pred_2: format!("pred2-{s}"),
    }
}

fn template_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pddl-pair.template.ttl")
}

fn joseph_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("plans/joseph")
}

/// Renders the fixture template with seed-derived names into
/// `target/chatman/cng-tests/<test_name>/seed-<seed>/<variant>/` and returns
/// the artifact directory.
fn render_fixture(test_name: &str, seed: u64, variant: &str) -> PathBuf {
    let names = seeded_names(seed);
    let template = fs::read_to_string(template_path()).expect("read fixture template");
    let rendered = template
        .replace("{{DOMAIN_NAME}}", &names.domain)
        .replace("{{OBJ}}", &names.obj)
        .replace("{{ACTION_0}}", &names.action_0)
        .replace("{{ACTION_1}}", &names.action_1)
        .replace("{{PRED_0}}", &names.pred_0)
        .replace("{{PRED_1}}", &names.pred_1)
        .replace("{{PRED_2}}", &names.pred_2);
    assert!(
        !rendered.contains("{{"),
        "unsubstituted placeholder left in rendered fixture"
    );
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests")
        .join(test_name)
        .join(format!("seed-{seed}"))
        .join(variant);
    fs::create_dir_all(&dir).expect("create fixture dir");
    fs::write(dir.join("pddl-pair.ttl"), rendered).expect("write rendered fixture");
    dir
}

/// Full pipeline over an artifact dir: import -> plan -> project -> Turtle.
fn pipeline_turtle(dir: &Path) -> String {
    let artifacts = import_artifacts(dir).expect("import_artifacts");
    let (tape, _surface) = generate_plan(&artifacts).expect("generate_plan");
    let model = project_tape_to_powl(&tape).expect("project_tape_to_powl");
    powl_to_turtle(&model, BASE_IRI, Some(DERIVED_FROM))
}

fn load_store(turtle: &str) -> Store {
    let store = Store::new().expect("store");
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .expect("Turtle output must parse via oxigraph");
    store
}

/// Runs a two-column (?term, ?count) grouping query loaded from a fixture
/// `.rq` file and returns term-IRI → count. No SPARQL text lives in Rust
/// sources — queries are fixture files (query-authority boundary).
fn fixture_counts(store: &Store, query_file: &str) -> std::collections::BTreeMap<String, u64> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/queries")
        .join(query_file);
    let query = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture query {}: {e}", path.display()));
    let results = SparqlEvaluator::new()
        .parse_query(&query)
        .expect("parse fixture query")
        .on_store(store)
        .execute()
        .expect("execute fixture query");
    let QueryResults::Solutions(solutions) = results else {
        panic!("expected SELECT solutions from fixture query {query_file}");
    };
    let mut counts = std::collections::BTreeMap::new();
    for solution in solutions {
        let solution = solution.expect("solution");
        let term = solution
            .get("type")
            .or_else(|| solution.get("p"))
            .map(ToString::to_string)
            .expect("grouping term bound");
        let count = solution
            .get("count")
            .map(ToString::to_string)
            .and_then(|s| s.split('"').nth(1).map(str::to_string))
            .and_then(|s| s.parse::<u64>().ok())
            .expect("count literal bound");
        counts.insert(term, count);
    }
    counts
}

fn count_type(store: &Store, class: &str) -> usize {
    fixture_counts(store, "type-counts.rq")
        .get(&format!("<{POWL2_PREFIX}{class}>"))
        .copied()
        .unwrap_or(0) as usize
}

fn count_predicate(store: &Store, predicate: &str) -> usize {
    fixture_counts(store, "predicate-counts.rq")
        .get(&format!("<{POWL2_PREFIX}{predicate}>"))
        .copied()
        .unwrap_or(0) as usize
}

#[test]
fn seeded_fixture_same_seed_is_byte_identical() {
    let seed = 1u64;
    println!("PDDL_FIXTURE_SEED={seed}");
    let dir_a = render_fixture("seeded_fixture_same_seed_is_byte_identical", seed, "a");
    let dir_b = render_fixture("seeded_fixture_same_seed_is_byte_identical", seed, "b");
    let turtle_a = pipeline_turtle(&dir_a);
    let turtle_b = pipeline_turtle(&dir_b);
    assert!(!turtle_a.is_empty(), "pipeline produced empty Turtle");
    assert_eq!(
        turtle_a.as_bytes(),
        turtle_b.as_bytes(),
        "same seed must yield byte-identical Turtle"
    );
    println!(
        "POWL_DIGEST=blake3:{}",
        blake3::hash(turtle_a.as_bytes()).to_hex()
    );
}

#[test]
fn seeded_fixture_different_seed_changes_powl() {
    let seed_1 = 1u64;
    let seed_2 = 2u64;
    println!("PDDL_FIXTURE_SEED={seed_1}");
    println!("PDDL_FIXTURE_SEED={seed_2}");
    let dir_1 = render_fixture("seeded_fixture_different_seed_changes_powl", seed_1, "s1");
    let dir_2 = render_fixture("seeded_fixture_different_seed_changes_powl", seed_2, "s2");
    let turtle_1 = pipeline_turtle(&dir_1);
    let turtle_2 = pipeline_turtle(&dir_2);
    assert_ne!(
        turtle_1.as_bytes(),
        turtle_2.as_bytes(),
        "different seeds must change the POWL Turtle bytes"
    );
    // Both outputs parse via oxigraph.
    let _ = load_store(&turtle_1);
    let _ = load_store(&turtle_2);
    // Seed-derived action names appear in the matching output only.
    let names_1 = seeded_names(seed_1);
    let names_2 = seeded_names(seed_2);
    for action in [&names_1.action_0, &names_1.action_1] {
        assert!(
            turtle_1.contains(action.as_str()),
            "seed-1 action {action} missing in seed-1 Turtle"
        );
        assert!(
            !turtle_2.contains(action.as_str()),
            "seed-1 action {action} leaked into seed-2 Turtle"
        );
    }
    for action in [&names_2.action_0, &names_2.action_1] {
        assert!(
            turtle_2.contains(action.as_str()),
            "seed-2 action {action} missing in seed-2 Turtle"
        );
        assert!(
            !turtle_1.contains(action.as_str()),
            "seed-2 action {action} leaked into seed-1 Turtle"
        );
    }
}

#[test]
fn joseph_example_many_to_one() {
    let artifacts = import_artifacts(&joseph_dir()).expect("import joseph artifacts");
    assert_eq!(artifacts.len(), 26, "joseph example holds 26 artifacts");
    let (tape, _surface) = generate_plan(&artifacts).expect("generate joseph plan");
    assert_eq!(tape.ops.len(), 20, "joseph plan has 20 steps");
    assert_eq!(plan_id(&tape), JOSEPH_PLAN_ID);

    let model = project_tape_to_powl(&tape).expect("project joseph tape");
    match &model {
        Powl::PartialOrder { children, order } => {
            assert_eq!(children.len(), 20, "20 children");
            assert_eq!(order.len(), 190, "C(20,2) = 190 order pairs");
            let expected: BTreeSet<(usize, usize)> = (0..20)
                .flat_map(|i| ((i + 1)..20).map(move |j| (i, j)))
                .collect();
            assert_eq!(order, &expected, "order relation is the full closure");
        }
        other => panic!("expected PartialOrder root, got {other:?}"),
    }

    let turtle = powl_to_turtle(&model, BASE_IRI, Some(DERIVED_FROM));
    let store = load_store(&turtle);
    assert_eq!(count_type(&store, "Model"), 1);
    assert_eq!(count_type(&store, "PartialOrder"), 1);
    assert_eq!(count_type(&store, "ActivityLeaf"), 20);
    assert_eq!(count_type(&store, "ChildBinding"), 20);
    assert_eq!(count_predicate(&store, "precedes"), 190);
    assert_eq!(count_predicate(&store, "derivedFrom"), 1);
}

#[test]
fn refusals_are_typed() {
    // Empty dir: no .ttl artifacts -> MissingDomain (CNG_R02).
    let empty_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/refusals_are_typed/empty");
    fs::create_dir_all(&empty_dir).expect("create empty dir");
    match import_artifacts(&empty_dir) {
        Err(CngRefusal::MissingDomain(msg)) => {
            assert!(
                msg.contains("no .ttl"),
                "message names the missing artifacts: {msg}"
            )
        }
        Err(other) => panic!("expected MissingDomain for empty dir, got {other:?}"),
        Ok(artifacts) => panic!(
            "expected MissingDomain for empty dir, got Ok with {} artifacts",
            artifacts.len()
        ),
    }

    // Domain-only artifacts (copied from the joseph example) -> generate_plan
    // refuses MissingProblem (CNG_R03) naming the missing fragment kind.
    let domain_only_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/refusals_are_typed/domain-only");
    fs::create_dir_all(&domain_only_dir).expect("create domain-only dir");
    let mut copied = 0usize;
    for entry in fs::read_dir(joseph_dir()).expect("read joseph dir") {
        let path = entry.expect("dir entry").path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("utf-8 file name")
            .to_string();
        if name.ends_with(".domain.ttl") {
            fs::copy(&path, domain_only_dir.join(&name)).expect("copy domain artifact");
            copied += 1;
        }
    }
    assert_eq!(copied, 13, "joseph example holds 13 domain artifacts");
    let artifacts = import_artifacts(&domain_only_dir).expect("import domain-only artifacts");
    match generate_plan(&artifacts) {
        Err(CngRefusal::MissingProblem(msg)) => {
            assert!(
                msg.contains("problem"),
                "message mentions the missing problem: {msg}"
            )
        }
        Err(other) => panic!("expected MissingProblem for domain-only dir, got {other:?}"),
        Ok((tape, _surface)) => panic!(
            "expected MissingProblem for domain-only dir, got a {}-step plan",
            tape.ops.len()
        ),
    }

    // Empty tape -> project_tape_to_powl refuses PlanUnsolvable (CNG_R04).
    let empty_tape = bcinr_pddl::Pddl8Tape { ops: vec![] };
    match project_tape_to_powl(&empty_tape) {
        Err(CngRefusal::PlanUnsolvable(msg)) => assert!(!msg.is_empty()),
        other => panic!("expected PlanUnsolvable for empty tape, got {other:?}"),
    }
}

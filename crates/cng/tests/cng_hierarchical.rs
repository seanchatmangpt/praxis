//! Integration test for the increment-1 hierarchical POWL projection: the
//! full chain over the joseph file fixtures (`plans/joseph/*.ttl`) —
//! import → plan → hierarchical projection → phase-provenance Turtle →
//! parse → shape validation → lawful flattening → runner conformance.
//!
//! Artifact boundary: Turtle enters only from the `.ttl` files on disk and
//! leaves only as serializer output (written to a scratch dir under
//! `target/chatman/cng-tests/`). Assertions are typed Rust comparisons on
//! the `Powl` model and labels, `cng::shape::validate_powl_store` over the
//! parsed output, and `RunnerReport` fields — no inline Turtle, no inline
//! SPARQL strings.
//!
//! The load-bearing equivalence: `linearize_hierarchical`'s label sequence
//! equals the flat `project_tape_to_powl` label sequence (Vec equality),
//! proving lawful flattening ≡ flat projection for execution while the
//! hierarchy stays real in the serialized artifact.

use chicago_tdd_tools::prelude::*;

use std::fs;
use std::path::{Path, PathBuf};

use cng::pipeline::{generate_plan, hierarchical_projection, import_artifacts};
use cng::powl::{powl_to_turtle_with_phase_provenance, project_tape_to_powl, Powl};
use cng::runner::{linearize_hierarchical, validate_run, validate_run_hierarchical};
use cng::shape::validate_powl_store;
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

const BASE_IRI: &str = "urn:chatman:powl:cng-hierarchical-test";
const DERIVED_FROM: &str = "urn:chatman:plan:cng-hierarchical-test";

fn joseph_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("plans/joseph")
}

fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests")
        .join(test_name);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn load_store(turtle: &str) -> Store {
    let store = Store::new().expect("store");
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .expect("serializer output must parse via oxigraph");
    store
}

/// Activity labels of a flat linear model, extracted by typed matching.
fn flat_labels(model: &Powl) -> Vec<&str> {
    let Powl::PartialOrder { children, .. } = model else {
        panic!("flat projection must be a root PartialOrder");
    };
    children
        .iter()
        .map(|child| match child {
            Powl::Leaf(Some(label)) => label.as_str(),
            other => panic!("flat projection children must be labelled leaves, got {other:?}"),
        })
        .collect()
}

test!(
    joseph_hierarchical_chain_flattens_to_flat_projection_and_runs,
    {
        // Arrange: import the joseph artifacts and plan once.
        let artifacts = import_artifacts(&joseph_dir()).expect("import joseph artifacts");
        let (tape, surface) = generate_plan(&artifacts).expect("generate joseph plan");
        assert!(!tape.ops.is_empty(), "joseph plan must be non-empty");

        // Act: hierarchical projection + phase-provenance serialization.
        let (model, phase_sources) =
            hierarchical_projection(&tape, &surface).expect("hierarchical projection");
        let turtle = powl_to_turtle_with_phase_provenance(
            &model,
            BASE_IRI,
            Some(DERIVED_FROM),
            &phase_sources,
        )
        .expect("phase provenance serialization");
        let out_path = scratch_dir("cng_hierarchical").join("joseph-hierarchical.powl.ttl");
        fs::write(&out_path, &turtle).expect("write generated POWL artifact");
        println!("GENERATED_POWL_TTL_PATH={}", out_path.display());

        // Assert: the model is genuinely two-level with one source per phase.
        let Powl::PartialOrder { children, .. } = &model else {
            panic!("hierarchical projection must return a root PartialOrder");
        };
        assert_eq!(children.len(), phase_sources.len(), "one source per phase");
        assert!(
            children
                .iter()
                .all(|child| matches!(child, Powl::PartialOrder { .. })),
            "every top-level child must be a phase PartialOrder"
        );
        assert!(children.len() > 1, "joseph plan must group into >1 phase");

        // Assert: the serialized artifact parses and passes the crate's own
        // structural validator (provenance required on the root).
        let store = load_store(&turtle);
        let report = validate_powl_store(&store, true).expect("nested model must validate");
        assert_eq!(report.models, 1);
        assert_eq!(report.activity_leaves, tape.ops.len());
        assert_eq!(report.partial_orders, children.len() + 1, "root + phases");
        assert_eq!(report.derived_from, 1);

        // Assert: lawful flattening reproduces the flat projection's labels
        // exactly (Vec equality) — flattening is a projection, not a choice.
        let (labels, edges) = linearize_hierarchical(&model).expect("linearize hierarchical model");
        let flat = project_tape_to_powl(&tape).expect("flat projection");
        assert_eq!(
            labels,
            flat_labels(&flat),
            "hierarchical flattening must equal the flat projection label sequence"
        );
        let n = labels.len();
        assert_eq!(
            edges.len(),
            n * (n - 1) / 2,
            "flattened edges must be the full transitive closure"
        );

        // Assert: hierarchical execution is conformant on the published runner
        // and reports the same executed-op count as the flat path.
        let hierarchical_report =
            validate_run_hierarchical(&tape, &model).expect("hierarchical run must validate");
        assert!(hierarchical_report.validated);
        assert!(hierarchical_report.conformant);
        assert_eq!(hierarchical_report.executed_ops, tape.ops.len());
        let flat_report = validate_run(&tape, &flat).expect("flat run must validate");
        assert_eq!(hierarchical_report.executed_ops, flat_report.executed_ops);
    }
);

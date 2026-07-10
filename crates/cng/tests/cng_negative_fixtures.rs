//! Artifact-backed negative fixtures: invalid source graphs and unsupported
//! structures must refuse with their exact `CNG_Rxx` codes. Every fixture is
//! a real `.ttl` file under `tests/fixtures/negative/` — no inline payloads.

use std::fs;
use std::path::{Path, PathBuf};

use cng::pipeline::{generate_plan, import_artifacts};
use cng::powl::CngRefusal;

fn negative_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("negative")
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("chatman")
        .join("cng-tests")
        .join("negative")
        .join(name);
    fs::create_dir_all(&dir).expect("scratch dir must be creatable");
    // Clear stale artifacts from prior runs so each test sees exactly its set.
    for entry in fs::read_dir(&dir).expect("scratch dir must list").flatten() {
        let _removed = fs::remove_file(entry.path());
    }
    dir
}

fn copy_into(dir: &Path, from: &Path) {
    let name = from.file_name().expect("fixture has a file name");
    fs::copy(from, dir.join(name)).expect("fixture copy must succeed");
}

#[test]
fn malformed_ttl_refuses_cng_r01() {
    let dir = scratch_dir("malformed");
    copy_into(&dir, &negative_dir().join("malformed.ttl"));
    match import_artifacts(&dir) {
        Err(refusal @ CngRefusal::MalformedTtl(_)) => {
            assert_eq!(refusal.code(), "CNG_R01");
            assert!(refusal.message().contains("Turtle parse failed"));
        }
        Err(other) => panic!("expected MalformedTtl, got {other:?}"),
        Ok(artifacts) => panic!(
            "expected MalformedTtl, got {} imported artifacts",
            artifacts.len()
        ),
    }
}

#[test]
fn unsolvable_goal_refuses_cng_r04() {
    let dir = scratch_dir("unsolvable");
    copy_into(&dir, &negative_dir().join("unsolvable.ttl"));
    let artifacts = import_artifacts(&dir).expect("unsolvable fixture imports cleanly");
    match generate_plan(&artifacts) {
        Err(refusal @ CngRefusal::PlanUnsolvable(_)) => {
            assert_eq!(refusal.code(), "CNG_R04");
        }
        Err(other) => panic!("expected PlanUnsolvable, got {other:?}"),
        Ok((tape, _)) => panic!(
            "expected PlanUnsolvable, got a {}-step plan",
            tape.ops.len()
        ),
    }
}

#[test]
fn duplicate_actions_refuse_cng_r05() {
    // Copy the same joseph domain fragment twice under different names: the
    // structural merge must refuse the duplicate action names.
    let dir = scratch_dir("duplicate-actions");
    let joseph = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("plans")
        .join("joseph");
    copy_into(&dir, &joseph.join("forecast.domain.ttl"));
    fs::copy(
        joseph.join("forecast.domain.ttl"),
        dir.join("forecast-copy.domain.ttl"),
    )
    .expect("duplicate copy must succeed");
    copy_into(&dir, &joseph.join("forecast.problem.ttl"));
    let artifacts = import_artifacts(&dir).expect("duplicate fixture imports cleanly");
    match generate_plan(&artifacts) {
        Err(refusal @ CngRefusal::UnsupportedConstruct(_)) => {
            assert_eq!(refusal.code(), "CNG_R05");
            assert!(refusal.message().contains("duplicate PDDL action name"));
        }
        Err(other) => panic!("expected UnsupportedConstruct, got {other:?}"),
        Ok(_) => panic!("expected UnsupportedConstruct for duplicated domain fragment"),
    }
}

#[test]
fn invalid_powl_refuses_cng_r06_via_shape_validation() {
    use oxigraph::io::{RdfFormat, RdfParser};
    use oxigraph::store::Store;

    let turtle = fs::read_to_string(negative_dir().join("invalid-powl.ttl"))
        .expect("invalid-powl fixture must be readable");
    let store = Store::new().expect("store construction");
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .expect("invalid-powl fixture parses as Turtle (shape is what fails)");
    match cng::shape::validate_powl_store(&store, false) {
        Err(refusal @ CngRefusal::InvalidPowl(_)) => {
            assert_eq!(refusal.code(), "CNG_R06");
            assert!(refusal.message().contains("shape violation"));
        }
        Err(other) => panic!("expected InvalidPowl, got {other:?}"),
        Ok(report) => panic!("expected InvalidPowl, got shape-valid report {report:?}"),
    }
}

#[test]
fn cli_inspect_refuses_invalid_powl() {
    // The CLI surface must expose the same typed refusal.
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cng"))
        .args([
            "workflow",
            "inspect",
            "--file",
            negative_dir()
                .join("invalid-powl.ttl")
                .to_str()
                .expect("utf8 path"),
        ])
        .output()
        .expect("cng binary must run");
    assert!(
        !output.status.success(),
        "inspect of an invalid POWL graph must exit nonzero"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("CNG_R06"),
        "CLI refusal must carry the CNG_R06 code, got: {combined}"
    );
}

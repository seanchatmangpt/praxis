//! CLI smoke test: drive the compiled `cng` binary over the joseph example
//! (generate -> export -> inspect) and assert on the JSON report substrings
//! (serde_json is not a cng dependency, so assertions stay textual).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const JOSEPH_PLAN_ID: &str =
    "blake3:c025532cc3e9c1a96625dfa551f7ec8ba9b0af68138403540149805c7ec63749";

fn joseph_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("plans/joseph")
}

fn run_cng(args: &[&str]) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_cng"))
        .args(args)
        .output()
        .expect("spawn cng binary");
    (
        String::from_utf8(output.stdout).expect("utf-8 stdout"),
        String::from_utf8(output.stderr).expect("utf-8 stderr"),
        output.status.success(),
    )
}

#[test]
fn cli_generate_export_inspect_smoke() {
    let joseph = joseph_dir();
    let joseph_arg = joseph.to_str().expect("utf-8 joseph dir");

    // plan generate
    let (stdout, stderr, ok) = run_cng(&["plan", "generate", "--dir", joseph_arg]);
    assert!(ok, "plan generate failed: stderr={stderr}");
    assert!(
        stdout.contains("generated_plan_id"),
        "generate stdout must report generated_plan_id: {stdout}"
    );
    assert!(
        stdout.contains(JOSEPH_PLAN_ID),
        "generate stdout must carry the joseph plan id: {stdout}"
    );

    // workflow export
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/chatman/cng-tests/cli_generate_export_inspect_smoke");
    fs::create_dir_all(&out_dir).expect("create export dir");
    let out_file = out_dir.join("joseph.powl.ttl");
    let out_arg = out_file.to_str().expect("utf-8 out path");
    let (stdout, stderr, ok) =
        run_cng(&["workflow", "export", "--dir", joseph_arg, "--out", out_arg]);
    assert!(ok, "workflow export failed: stderr={stderr}");
    assert!(
        out_file.is_file(),
        "export must write {}",
        out_file.display()
    );
    assert!(
        stdout.contains(JOSEPH_PLAN_ID),
        "export stdout must carry the joseph plan id: {stdout}"
    );

    // workflow inspect
    //
    // The joseph fixture's admitted surface traces to more than one source
    // artifact, so `manufacture()` now takes the hierarchical projection
    // branch (see `crates/cng/tests/cng_hierarchical.rs` for the direct
    // library-level proof this is the same, already-validated structure:
    // `partial_orders == children.len() + 1`, shape-valid). A flat
    // single-PartialOrder projection over these 20 leaves would report a
    // full transitive closure of C(20,2) = 190 `precedes` pairs; the
    // hierarchical projection instead orders leaves within each of the 14
    // phase/root PartialOrders, giving 85 -- fewer edges is the correct
    // signature of the branch firing, not a regression.
    let (stdout, stderr, ok) = run_cng(&["workflow", "inspect", "--file", out_arg]);
    assert!(ok, "workflow inspect failed: stderr={stderr}");
    assert!(
        stdout.contains("\"partial_orders\": 14"),
        "inspect stdout must report the hierarchical partial-order count: {stdout}"
    );
    assert!(
        stdout.contains("\"precedes\": 85"),
        "inspect stdout must report 85 precedes pairs (hierarchical, not the \
         flat 190-pair transitive closure): {stdout}"
    );
}

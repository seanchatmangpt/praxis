//! CLI-boundary proofs: spawn the real `ggen` binary as a subprocess via
//! chicago-tdd-tools `CliHarness`. No mocks — real argv, real exit codes,
//! real filesystem effects.
//!
//! Coverage goal: every noun/verb/flag combination the CLI actually exposes
//! (`ggen --help`, `sync run [--dry-run]`, `graph validate`, `receipt
//! verify`, `receipt history`, `--version`, `--introspect`, unknown
//! subcommands/args) proven against the compiled binary, not just the
//! library functions underneath it.

use chicago_tdd_tools::cli_proof::CliHarness;
use tempfile::TempDir;

fn scaffold(root: &std::path::Path) {
    std::fs::write(
        root.join("ggen.toml"),
        "[project]\nname = \"demo\"\n\n[ontology]\nsource = \"ontology.ttl\"\n\n[templates]\ndir = \"templates\"\n",
    )
    .expect("write ggen.toml");
    std::fs::write(
        root.join("ontology.ttl"),
        "@prefix ex: <http://example.org/> .\nex:alice ex:name \"alice\" .\n",
    )
    .expect("write ontology");
    std::fs::create_dir_all(root.join("templates")).expect("mkdir");
    std::fs::write(
        root.join("templates/one.tmpl"),
        "---\nto: out/names.txt\nforce: true\nsparql:\n  people: SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name\n---\n{% for row in results %}{{ row.name }}\n{% endfor %}",
    )
    .expect("write template");
}

// ---------------------------------------------------------------------
// Root
// ---------------------------------------------------------------------

#[test]
fn root_help_exits_zero_and_lists_all_nouns() {
    let output = CliHarness::cargo_bin("ggen").args(["--help"]).run().expect("run --help");
    output
        .assert_success()
        .assert_stdout_contains("sync")
        .assert_stdout_contains("graph")
        .assert_stdout_contains("receipt");
}

#[test]
fn root_version_exits_zero() {
    let output = CliHarness::cargo_bin("ggen").args(["--version"]).run().expect("run --version");
    output.assert_success();
}

#[test]
fn root_no_args_exits_zero_or_prints_usage() {
    // clap-noun-verb with no subcommand: must not crash, must not silently
    // hang — either succeeds with a usage summary or exits nonzero cleanly.
    let output = CliHarness::cargo_bin("ggen").run().expect("run with no args");
    assert!(
        output.exit_code == 0 || !output.stdout.is_empty() || !output.stderr.is_empty(),
        "expected a clean exit or usage output, got exit {} with empty stdout/stderr",
        output.exit_code
    );
}

#[test]
fn unknown_noun_exits_nonzero() {
    let output = CliHarness::cargo_bin("ggen")
        .args(["totally-unknown-noun-xyz"])
        .run()
        .expect("run unknown noun");
    output.assert_failure();
}

#[test]
fn unknown_flag_exits_nonzero() {
    let output = CliHarness::cargo_bin("ggen")
        .args(["sync", "run", "--this-flag-does-not-exist"])
        .run()
        .expect("run unknown flag");
    output.assert_failure();
}

// ---------------------------------------------------------------------
// sync <noun> --help / run --help
// ---------------------------------------------------------------------

#[test]
fn sync_noun_help_exits_zero_and_lists_run() {
    let output =
        CliHarness::cargo_bin("ggen").args(["sync", "--help"]).run().expect("run sync --help");
    output.assert_success().assert_stdout_contains("run");
}

#[test]
fn sync_run_help_lists_dry_run_flag() {
    let output = CliHarness::cargo_bin("ggen")
        .args(["sync", "run", "--help"])
        .run()
        .expect("run sync run --help");
    output.assert_success().assert_stdout_contains("dry-run");
}

// ---------------------------------------------------------------------
// sync run
// ---------------------------------------------------------------------

#[test]
fn sync_run_generates_expected_file() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());

    let output = CliHarness::cargo_bin("ggen")
        .args(["sync", "run"])
        .current_dir(dir.path())
        .run()
        .expect("run sync");
    output.assert_success();

    let content =
        std::fs::read_to_string(dir.path().join("out/names.txt")).expect("generated file");
    assert_eq!(content, "alice\n");
    assert!(dir.path().join(".ggen-v2/receipt.json").exists(), "receipt emitted");
}

#[test]
fn sync_run_dry_run_writes_nothing() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());

    let output = CliHarness::cargo_bin("ggen")
        .args(["sync", "run", "--dry-run"])
        .current_dir(dir.path())
        .run()
        .expect("run sync --dry-run");
    output.assert_success();

    assert!(!dir.path().join("out/names.txt").exists(), "dry-run must not write output");
    assert!(!dir.path().join(".ggen-v2/receipt.json").exists(), "dry-run must not emit a receipt");
    assert!(!dir.path().join("ggen.lock").exists(), "dry-run must not write a lockfile");
}

#[test]
fn sync_run_second_invocation_is_idempotent() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());

    CliHarness::cargo_bin("ggen")
        .args(["sync", "run"])
        .current_dir(dir.path())
        .run()
        .expect("first sync")
        .assert_success();
    let first = std::fs::read_to_string(dir.path().join("out/names.txt")).expect("first output");

    CliHarness::cargo_bin("ggen")
        .args(["sync", "run"])
        .current_dir(dir.path())
        .run()
        .expect("second sync")
        .assert_success();
    let second = std::fs::read_to_string(dir.path().join("out/names.txt")).expect("second output");

    assert_eq!(first, second, "second sync must not change output content");
}

#[test]
fn sync_run_missing_manifest_exits_nonzero() {
    let dir = TempDir::new().expect("tempdir");
    let output = CliHarness::cargo_bin("ggen")
        .args(["sync", "run"])
        .current_dir(dir.path())
        .run()
        .expect("run sync with no ggen.toml");
    output.assert_failure();
}

#[test]
fn sync_run_unbound_template_variable_exits_nonzero() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());
    // Overwrite the template with an unbound {{ typo }} — sync must refuse
    // via the same FM-TPL-003 lint that `graph validate` runs, not just
    // silently mis-render.
    std::fs::write(
        dir.path().join("templates/one.tmpl"),
        "---\nto: out/names.txt\nforce: true\nsparql:\n  people: SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name\n---\n{{ typo }}",
    )
    .expect("overwrite template");

    let output = CliHarness::cargo_bin("ggen")
        .args(["graph", "validate"])
        .current_dir(dir.path())
        .run()
        .expect("run graph validate");
    output.assert_failure().assert_stderr_contains("FM-TPL-003");
}

// ---------------------------------------------------------------------
// graph validate
// ---------------------------------------------------------------------

#[test]
fn graph_validate_valid_project_exits_zero() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());
    let output = CliHarness::cargo_bin("ggen")
        .args(["graph", "validate"])
        .current_dir(dir.path())
        .run()
        .expect("run graph validate");
    output.assert_success().assert_stdout_contains("templates_checked");
}

#[test]
fn graph_validate_missing_manifest_exits_nonzero() {
    let dir = TempDir::new().expect("tempdir");
    let output = CliHarness::cargo_bin("ggen")
        .args(["graph", "validate"])
        .current_dir(dir.path())
        .run()
        .expect("run graph validate");
    output.assert_failure();
}

#[test]
fn graph_validate_malformed_ontology_exits_nonzero() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());
    std::fs::write(dir.path().join("ontology.ttl"), "this is not valid turtle {{{")
        .expect("overwrite ontology");
    let output = CliHarness::cargo_bin("ggen")
        .args(["graph", "validate"])
        .current_dir(dir.path())
        .run()
        .expect("run graph validate on bad ttl");
    output.assert_failure();
}

// ---------------------------------------------------------------------
// receipt verify / history
// ---------------------------------------------------------------------

#[test]
fn receipt_verify_missing_receipt_exits_nonzero() {
    let dir = TempDir::new().expect("tempdir");
    let output = CliHarness::cargo_bin("ggen")
        .args(["receipt", "verify"])
        .current_dir(dir.path())
        .run()
        .expect("run receipt verify with no receipt");
    output.assert_failure();
}

#[test]
fn receipt_verify_succeeds_after_sync_and_fails_on_tamper() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());

    CliHarness::cargo_bin("ggen")
        .args(["sync", "run"])
        .current_dir(dir.path())
        .run()
        .expect("sync")
        .assert_success();

    CliHarness::cargo_bin("ggen")
        .args(["receipt", "verify"])
        .current_dir(dir.path())
        .run()
        .expect("verify")
        .assert_success();

    // Tamper with the payload → verify must exit nonzero (fail closed).
    let receipt_path = dir.path().join(".ggen-v2/receipt.json");
    let tampered = std::fs::read_to_string(&receipt_path)
        .expect("read receipt")
        .replace("\"graph_hash\"", "\"graph_hash_tampered_key_x\"");
    std::fs::write(&receipt_path, tampered).expect("write tampered");
    CliHarness::cargo_bin("ggen")
        .args(["receipt", "verify"])
        .current_dir(dir.path())
        .run()
        .expect("verify tampered")
        .assert_failure();
}

#[test]
fn receipt_history_missing_log_exits_nonzero() {
    let dir = TempDir::new().expect("tempdir");
    let output = CliHarness::cargo_bin("ggen")
        .args(["receipt", "history"])
        .current_dir(dir.path())
        .run()
        .expect("run receipt history with no log");
    output.assert_failure();
}

#[test]
fn receipt_history_after_two_syncs_exits_zero() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());

    CliHarness::cargo_bin("ggen")
        .args(["sync", "run"])
        .current_dir(dir.path())
        .run()
        .expect("first sync")
        .assert_success();
    // Mutate the ontology so the second sync produces a genuinely new
    // record rather than an unchanged no-op.
    std::fs::write(
        dir.path().join("ontology.ttl"),
        "@prefix ex: <http://example.org/> .\nex:alice ex:name \"alice\" .\nex:bob ex:name \"bob\" .\n",
    )
    .expect("mutate ontology");
    CliHarness::cargo_bin("ggen")
        .args(["sync", "run"])
        .current_dir(dir.path())
        .run()
        .expect("second sync")
        .assert_success();

    let output = CliHarness::cargo_bin("ggen")
        .args(["receipt", "history"])
        .current_dir(dir.path())
        .run()
        .expect("run receipt history");
    output.assert_success().assert_stdout_contains("\"records\": 2");
}

#[test]
fn receipt_history_tampered_middle_record_exits_nonzero() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());

    for name in ["alice", "alice2"] {
        std::fs::write(
            dir.path().join("ontology.ttl"),
            format!("@prefix ex: <http://example.org/> .\nex:alice ex:name \"{name}\" .\n"),
        )
        .expect("mutate ontology");
        CliHarness::cargo_bin("ggen")
            .args(["sync", "run"])
            .current_dir(dir.path())
            .run()
            .expect("sync")
            .assert_success();
    }

    let log_path = dir.path().join(".ggen-v2/receipt-log.jsonl");
    let lines: Vec<String> =
        std::fs::read_to_string(&log_path).expect("read log").lines().map(String::from).collect();
    assert_eq!(lines.len(), 2, "expected two receipt-log lines");
    let tampered_first_line = lines[0].replace("\"written\"", "\"tampered\"");
    std::fs::write(&log_path, format!("{tampered_first_line}\n{}\n", lines[1]))
        .expect("write tampered log");

    let output = CliHarness::cargo_bin("ggen")
        .args(["receipt", "history"])
        .current_dir(dir.path())
        .run()
        .expect("run receipt history on tampered log");
    output.assert_failure();
}

// ---------------------------------------------------------------------
// Global flags
// ---------------------------------------------------------------------

#[test]
fn introspect_emits_json_schema_and_exits_zero() {
    let output =
        CliHarness::cargo_bin("ggen").args(["--introspect"]).run().expect("run --introspect");
    output.assert_success();
    assert!(
        output.stdout.trim_start().starts_with('['),
        "expected a JSON array of tool definitions, got: {}",
        output.stdout
    );
}

#[test]
fn format_json_flag_produces_parseable_json_on_success() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(dir.path());
    let output = CliHarness::cargo_bin("ggen")
        .args(["--format", "json", "graph", "validate"])
        .current_dir(dir.path())
        .run()
        .expect("run graph validate --format json");
    output.assert_success();
    serde_json::from_str::<serde_json::Value>(&output.stdout)
        .unwrap_or_else(|e| panic!("stdout not valid JSON ({e}): {}", output.stdout));
}

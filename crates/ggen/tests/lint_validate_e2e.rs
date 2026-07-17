//! `ggen graph validate` static-lint proofs: real filesystem (`TempDir`),
//! real template parsing, real CLI subprocess at the boundary. No mocks.

#![allow(clippy::expect_used)]

use std::path::Path;

use chicago_tdd_tools::cli_proof::CliHarness;
use ggen::lint::lint_template;
use ggen::template::Template;
use tempfile::TempDir;

/// Scaffold a minimal project with one template body.
fn scaffold(root: &Path, template: &str) {
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
    std::fs::create_dir_all(root.join("templates")).expect("mkdir templates");
    std::fs::write(root.join("templates/one.tmpl"), template).expect("write template");
}

fn parse_file(path: &Path) -> Template {
    let content = std::fs::read_to_string(path).expect("read template");
    Template::parse(&content).expect("parse template")
}

#[test]
fn unbound_body_var_on_disk_yields_fm_tpl_003() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(
        dir.path(),
        "---\nto: out/names.txt\nsparql:\n  people: SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name\n---\n{{ typo }}",
    );
    let path = dir.path().join("templates/one.tmpl");
    let errs = lint_template(&path, &parse_file(&path));
    assert_eq!(errs.len(), 1, "{errs:?}");
    let msg = errs[0].to_string();
    assert!(msg.contains("FM-TPL-003"), "{msg}");
    assert!(msg.contains("typo"), "{msg}");
    assert!(msg.contains("one.tmpl"), "{msg}");
}

#[test]
fn unbound_to_path_var_on_disk_yields_fm_tpl_004() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(
        dir.path(),
        "---\nto: \"out/{{ nope }}.txt\"\nsparql:\n  people: SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name\n---\n{{ name }}",
    );
    let path = dir.path().join("templates/one.tmpl");
    let errs = lint_template(&path, &parse_file(&path));
    assert_eq!(errs.len(), 1, "{errs:?}");
    let msg = errs[0].to_string();
    assert!(msg.contains("FM-TPL-004"), "{msg}");
    assert!(msg.contains("nope"), "{msg}");
}

#[test]
fn select_star_disables_projection_check() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(
        dir.path(),
        "---\nto: out/all.txt\nsparql:\n  all: SELECT * WHERE { ?s ?p ?o }\n---\n{{ anything_at_all }}",
    );
    let path = dir.path().join("templates/one.tmpl");
    let errs = lint_template(&path, &parse_file(&path));
    assert!(errs.is_empty(), "SELECT * must disable the check: {errs:?}");
}

#[test]
fn identity_construct_on_disk_yields_fm_tpl_005() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(
        dir.path(),
        "---\nto: out/static.txt\nconstruct: \"CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }\"\n---\nstatic body",
    );
    let path = dir.path().join("templates/one.tmpl");
    let errs = lint_template(&path, &parse_file(&path));
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].to_string().contains("FM-TPL-005"), "{}", errs[0]);
}

#[test]
fn clean_demo_project_validates_via_cli() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(
        dir.path(),
        "---\nto: out/names.txt\nsparql:\n  people: SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name\n---\n{% for row in results %}{{ row.name }}\n{% endfor %}",
    );
    let output = CliHarness::cargo_bin("ggen")
        .args(["graph", "validate"])
        .current_dir(dir.path())
        .run()
        .expect("run graph validate");
    output.assert_success().assert_stdout_contains("templates_checked");
}

#[test]
fn cli_graph_validate_fails_closed_on_unbound_var() {
    let dir = TempDir::new().expect("tempdir");
    scaffold(
        dir.path(),
        "---\nto: out/names.txt\nsparql:\n  people: SELECT ?name WHERE { ?s <http://example.org/name> ?name } ORDER BY ?name\n---\n{{ typo }}",
    );
    let output = CliHarness::cargo_bin("ggen")
        .args(["graph", "validate"])
        .current_dir(dir.path())
        .run()
        .expect("run graph validate");
    output.assert_failure();
    let combined = format!("{}{}", output.stdout, output.stderr);
    assert!(combined.contains("FM-TPL-003"), "must name FM-TPL-003: {combined}");
    assert!(combined.contains("typo"), "must name the variable: {combined}");
}

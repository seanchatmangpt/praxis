use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use wasm4pm_arazzo::parse::DocumentIndex;

// Correctness tests for `DocumentIndex::add_document_from_file` /
// `add_documents_from_files_par`. Formerly `bench_mmap.rs`: that file compared
// `Instant::now()` wall-clock timings between an `unsafe` memory-mapped read and a plain
// `fs::read_to_string`, asserting mmap was "generally faster or within margin of error" --
// a flaky, machine-load-sensitive perf claim (docs/jira/v26.7.11/ADVERSARIAL_DOD.md,
// "PROJ-753's wasm4pm-arazzo full suite green claim" finding), already `#[ignore]`d for
// exactly that reason. Swarm audit wnl2yhbgm finding #11 removed the `unsafe` mmap entirely
// (no valid justification existed under this repo's own core-team discipline: unsafe is
// permitted only for cryptographic verification or FFI memory-layout guarantees, and this
// was neither) -- the mmap-vs-read timing comparison this file used to run no longer has
// two distinct code paths to compare, so it is replaced here with real correctness
// coverage of the function that survived: does it actually read and parse a file, and
// does it refuse a genuinely unreadable one, instead of silently swallowing the error.

fn write_temp_arazzo(path: &PathBuf, workflow_count: usize) {
    let mut doc_str = String::new();
    doc_str.push_str(
        r#"{
        "arazzo": "1.1.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "sourceDescriptions": [],
        "workflows": ["#,
    );
    for i in 0..workflow_count {
        if i > 0 {
            doc_str.push(',');
        }
        doc_str.push_str(&format!(
            r#"
        {{
            "workflowId": "wf_{i}",
            "summary": "Workflow {i}",
            "steps": [
                {{
                    "stepId": "step_1",
                    "operationId": "op_1",
                    "successCriteria": []
                }}
            ]
        }}"#
        ));
    }
    doc_str.push_str(
        r#"]
    }"#,
    );
    let mut file = File::create(path).expect("create temp arazzo fixture");
    file.write_all(doc_str.as_bytes())
        .expect("write temp arazzo fixture");
    file.flush().expect("flush temp arazzo fixture");
}

#[test]
fn add_document_from_file_reads_and_parses_a_real_file() {
    let test_file = PathBuf::from("test_document_from_file.json");
    write_temp_arazzo(&test_file, 3);

    let mut index = DocumentIndex::new();
    index
        .add_document_from_file(&test_file, "http://example.com/from-file")
        .expect("a well-formed Arazzo document must load from a real file");

    std::fs::remove_file(&test_file).expect("cleanup temp arazzo fixture");

    assert_eq!(index.documents.len(), 1);
    let doc = index
        .documents
        .get("http://example.com/from-file")
        .expect("document indexed under its fallback base URI");
    assert_eq!(doc.arazzo, "1.1.0");
    assert_eq!(doc.workflows.len(), 3);
}

#[test]
fn add_document_from_file_refuses_a_missing_file() {
    let mut index = DocumentIndex::new();
    let err = index
        .add_document_from_file(
            &PathBuf::from("this-file-does-not-exist.json"),
            "http://example.com/missing",
        )
        .expect_err("a missing file must refuse, not silently succeed");
    assert!(
        format!("{err:?}").contains("Failed to read file"),
        "expected the real I/O-failure message, got: {err:?}"
    );
    assert!(index.documents.is_empty());
}

#[test]
fn add_documents_from_files_par_reads_and_parses_multiple_real_files() {
    let file_a = PathBuf::from("test_document_from_file_par_a.json");
    let file_b = PathBuf::from("test_document_from_file_par_b.json");
    write_temp_arazzo(&file_a, 1);
    write_temp_arazzo(&file_b, 2);

    let mut index = DocumentIndex::new();
    let result = index.add_documents_from_files_par(&[
        (file_a.as_path(), "http://example.com/par-a"),
        (file_b.as_path(), "http://example.com/par-b"),
    ]);

    std::fs::remove_file(&file_a).expect("cleanup temp arazzo fixture a");
    std::fs::remove_file(&file_b).expect("cleanup temp arazzo fixture b");

    result.expect("two well-formed Arazzo documents must load in parallel");
    assert_eq!(index.documents.len(), 2);
    assert_eq!(
        index
            .documents
            .get("http://example.com/par-a")
            .expect("doc a indexed")
            .workflows
            .len(),
        1
    );
    assert_eq!(
        index
            .documents
            .get("http://example.com/par-b")
            .expect("doc b indexed")
            .workflows
            .len(),
        2
    );
}

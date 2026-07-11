use wasm4pm_arazzo::parse::DocumentIndex;
use std::time::Instant;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[test]
fn bench_mmap_vs_read() {
    let mut doc_str = String::new();
    doc_str.push_str(r#"{
        "arazzo": "1.1.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "sourceDescriptions": [],
        "workflows": ["#);
    
    // Generate a massive JSON document to magnify the difference
    for i in 0..10_000 {
        if i > 0 { doc_str.push_str(","); }
        doc_str.push_str(&format!(r#"
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
        }}"#));
    }
    doc_str.push_str(r#"]
    }"#);

    let test_file = PathBuf::from("test_massive_arazzo.json");
    let mut file = File::create(&test_file).unwrap();
    file.write_all(doc_str.as_bytes()).unwrap();
    file.flush().unwrap();

    // Standard string loading (Slow)
    let mut index_str = DocumentIndex::new();
    let start_str = Instant::now();
    let content = std::fs::read_to_string(&test_file).unwrap();
    index_str.add_document(&content, "http://example.com/str").unwrap();
    let duration_str = start_str.elapsed();
    
    // Memory-mapped zero-copy loading (Fast)
    let mut index_mmap = DocumentIndex::new();
    let start_mmap = Instant::now();
    index_mmap.add_document_from_file(&test_file, "http://example.com/mmap").unwrap();
    let duration_mmap = start_mmap.elapsed();

    // Cleanup
    std::fs::remove_file(&test_file).unwrap();

    println!("Standard string loading took: {:?}", duration_str);
    println!("Memory-mapped zero-copy loading took: {:?}", duration_mmap);

    assert!(duration_mmap < duration_str, "mmap should be faster than standard read_to_string on large files");
}

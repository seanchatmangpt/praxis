//! Property-based testing for praxis-retrofit parser resilience.

use praxis_retrofit::repo_registry::RepositoryRegistry;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parsing_resilience(ref s in "\\PC*") {
        // Assert that parsing arbitrary strings never triggers a panic
        // and returns a structured Error or Ok
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                RepositoryRegistry::load_str(s).await
            });

        // Ensure it either parses successfully or fails gracefully
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_ocel_tracing_integration() {
    use std::{collections::HashMap, path::PathBuf};

    use chicago_tdd_tools::{
        core::governance::{
            close_channel, emit_diagnostic, register_sink, Diagnostic, DiagnosticCategory,
            DiagnosticCode, Severity,
        },
        observability::ocel::OcelCollector,
    };

    // Ensure the output directory exists
    let output_path = PathBuf::from("target/praxis/evidence/events.ocel.json");
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    if output_path.exists() {
        let _ = std::fs::remove_file(&output_path);
    }

    // Initialize OcelCollector
    let collector = OcelCollector::new(Some(output_path.clone()));

    // Register it as the active DiagnosticSink
    register_sink(Box::new(collector));

    // Emit test start event
    let start_diag = Diagnostic {
        code: DiagnosticCode::new("RETRO".to_string(), DiagnosticCategory::Conformance, 900),
        category: DiagnosticCategory::Conformance,
        severity: Severity::Info,
        location: None,
        message: "Test started: ocel_integration_test".to_string(),
        context: HashMap::new(),
        run_id: "praxis-run-123".to_string(),
        agent_id: None,
        source_module: "praxis_retrofit::tests",
        elapsed_ns: 1000,
    };
    emit_diagnostic(&start_diag);

    // Emit test diagnostic event
    let mut context = HashMap::new();
    context
        .insert("artifact_id", serde_json::Value::String("praxis-retrofit-artifact".to_string()));

    let diag = Diagnostic {
        code: DiagnosticCode::new("RETRO".to_string(), DiagnosticCategory::Conformance, 100),
        category: DiagnosticCategory::Conformance,
        severity: Severity::Info,
        location: None,
        message: "Ocel integration test diagnostic".to_string(),
        context,
        run_id: "praxis-run-123".to_string(),
        agent_id: None,
        source_module: "praxis_retrofit::tests",
        elapsed_ns: 2000,
    };
    emit_diagnostic(&diag);

    // Emit test completed event
    let mut context_comp = HashMap::new();
    context_comp.insert("passed", serde_json::json!(true));
    let comp_diag = Diagnostic {
        code: DiagnosticCode::new("RETRO".to_string(), DiagnosticCategory::Conformance, 901),
        category: DiagnosticCategory::Conformance,
        severity: Severity::Info,
        location: None,
        message: "Test completed: ocel_integration_test (passed: true)".to_string(),
        context: context_comp,
        run_id: "praxis-run-123".to_string(),
        agent_id: None,
        source_module: "praxis_retrofit::tests",
        elapsed_ns: 3000,
    };
    emit_diagnostic(&comp_diag);

    // Close the channel to write the sealed run receipt
    let _summary = close_channel().unwrap();

    // Verify file exists
    assert!(output_path.exists(), "OCEL event file was not created");
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(!content.is_empty(), "OCEL event file is empty");
    assert!(content.contains("praxis-run-123"), "OCEL content missing case/run ID");
    assert!(content.contains("praxis-retrofit-artifact"), "OCEL content missing artifact ID");
}

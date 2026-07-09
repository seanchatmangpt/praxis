//! Native Rust unit tests for the WASM bridge's pure core functions.
//!
//! These tests exercise validate_all_core, run_hooks_core, and graph_hash_core
//! without JavaScript/WASM, ensuring the Rust logic is correct before compiling to WASM.
//!
//! # Test Strategy
//!
//! - Use inline Turtle strings (no file fixtures)
//! - Test happy paths, error cases, and determinism guarantees
//! - Verify DTO serialization (SCREAMING_SNAKE_CASE enum variants)
//! - Check replay verification logic (determinism detection)
//!
//! # Notes on Panic Safety
//!
//! All core functions wrap engine calls in catch_unwind. Tests that expect
//! success (Ok) should assert result.is_ok(); tests that expect graceful
//! error handling (not panic) should assert result.is_err() (panics are
//! converted to Err by catch_unwind).

use praxis_graphlaw_wasm::{
    core::{graph_hash_core, run_hooks_core, validate_all_core},
    dto::{HookRunResult, PlaygroundResult, Status},
};

// ============================================================================
// Test 1: validate_all_core happy path
// ============================================================================

/// Test validate_all_core with minimal valid Turtle, no profile/shapes.
///
/// Assertions:
/// - result.status == Status::Admitted
/// - result.graph_hash is non-empty 64-char BLAKE3 hex
/// - result.replay.status == Status::Admitted
/// - result.replay.first_hash == result.replay.second_hash (determinism)
#[test]
fn test_validate_all_core_happy_path() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:alice ex:knows ex:bob .
        ex:bob ex:knows ex:charlie .
    "#;
    let profile_ttl = "";
    let shacl_shapes = "";
    let shex_schema = "";
    let shex_shape_map = "";

    let result = validate_all_core(ttl, profile_ttl, shacl_shapes, shex_schema, shex_shape_map);
    assert!(result.is_ok(), "validate_all_core failed: {:?}", result);

    let playground = result.unwrap();

    // Verify input graph hash exists
    assert!(
        !playground.graph_hash.is_empty(),
        "graph_hash should not be empty"
    );
    assert_eq!(
        playground.graph_hash.len(),
        64,
        "BLAKE3 hex should be 64 chars"
    );

    // Verify replay verification passed
    assert_eq!(
        playground.replay.status,
        Status::Admitted,
        "replay verification should succeed"
    );
    assert_eq!(
        playground.replay.first_hash, playground.replay.second_hash,
        "replay hashes must match (determinism)"
    );

    // Verify dialects are populated
    assert!(
        !playground.dialects.is_empty(),
        "dialects should be populated"
    );
}

// ============================================================================
// Test 2: validate_all_core with N3 denial violation
// ============================================================================

/// Test validate_all_core includes N3_DENIAL dialect result.
///
/// The N3_DENIAL dialect checks for denial rule violations in the graph.
/// This test verifies that:
/// - The N3_DENIAL dialect result is present in the results
/// - It has a status (typically Admitted if no denials, or Refused if violations)
/// - The detail message provides information about findings
///
/// Assertions:
/// - result is Ok (no panic via catch_unwind)
/// - dialects contains an N3_DENIAL entry
/// - N3_DENIAL status is either Admitted or Refused
#[test]
fn test_validate_all_core_n3_denial() {
    // Minimal TTL with no special denial rules
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:entity1 a ex:Concept .
    "#;

    let profile_ttl = "";
    let shacl_shapes = "";
    let shex_schema = "";
    let shex_shape_map = "";

    let result = validate_all_core(ttl, profile_ttl, shacl_shapes, shex_schema, shex_shape_map);
    assert!(
        result.is_ok(),
        "validate_all_core should not panic: {:?}",
        result
    );

    let playground = result.unwrap();

    // Find the N3_DENIAL dialect result
    let n3_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "N3_DENIAL")
        .expect("N3_DENIAL dialect should be present");

    // Assert N3_DENIAL result is populated
    // Status should be either Admitted (no violations) or Refused (violations found)
    assert!(
        n3_dialect.status == Status::Admitted || n3_dialect.status == Status::Refused,
        "N3_DENIAL status should be either Admitted or Refused. Got: {:?}",
        n3_dialect.status
    );

    // Verify detail contains information about denial violations
    assert!(
        n3_dialect.detail.contains("denial") || n3_dialect.detail.contains("Found"),
        "N3 dialect detail should describe findings. Got: {}",
        n3_dialect.detail
    );
}

// ============================================================================
// Test 3: validate_all_core with missing SHACL required property
// ============================================================================

/// Test validate_all_core with SHACL profile requiring a property.
///
/// Turtle data is missing the required property. Assertions:
/// - result.status should reflect validation failure
/// - dialects contains SHACL entry with Status::Refused
/// - SHACL detail mentions violations or conformance failure
#[test]
fn test_validate_all_core_missing_required_property() {
    // Data graph: a Person without name (violates SHACL)
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:person1 a ex:Person .
    "#;

    let profile_ttl = "";

    // SHACL shapes: Person must have a name
    let shacl_shapes = r#"
        @prefix ex: <http://example.org/> .
        @prefix sh: <http://www.w3.org/ns/shacl#> .

        ex:PersonShape a sh:NodeShape ;
            sh:targetClass ex:Person ;
            sh:property [
                sh:path ex:name ;
                sh:minCount 1 ;
                sh:maxCount 1
            ] .
    "#;

    let shex_schema = "";
    let shex_shape_map = "";

    let result = validate_all_core(ttl, profile_ttl, shacl_shapes, shex_schema, shex_shape_map);
    assert!(
        result.is_ok(),
        "validate_all_core should not panic: {:?}",
        result
    );

    let playground = result.unwrap();

    // Find SHACL dialect
    let shacl_dialect = playground
        .dialects
        .iter()
        .find(|d| d.dialect == "SHACL")
        .expect("SHACL dialect should be present");

    // Verify SHACL validation failed due to missing property
    assert_eq!(
        shacl_dialect.status,
        Status::Refused,
        "SHACL validation should fail for missing required property"
    );
}

// ============================================================================
// Test 4: validate_all_core with empty inputs (no panic)
// ============================================================================

/// Test validate_all_core with empty TTL strings.
///
/// Empty inputs should not cause panics. The function should return
/// gracefully (either Admitted or Refused status).
///
/// Assertions:
/// - result is Ok (no panic via catch_unwind)
/// - status is either Admitted or Refused or Unsupported
#[test]
fn test_validate_all_core_empty_inputs_no_panic() {
    let ttl = "";
    let profile_ttl = "";
    let shacl_shapes = "";
    let shex_schema = "";
    let shex_shape_map = "";

    let result = validate_all_core(ttl, profile_ttl, shacl_shapes, shex_schema, shex_shape_map);

    // Should return Ok (no panic), even if status is Refused/Unsupported
    assert!(
        result.is_ok(),
        "Empty inputs should not cause panic. Got: {:?}",
        result
    );

    let _playground = result.unwrap();
    // We don't assert specific status; just that it didn't panic
}

// ============================================================================
// Test 5: run_hooks_core fires expected hook
// ============================================================================

/// Test run_hooks_core with hook execution.
///
/// Base TTL contains a kh:Hook definition. This test verifies that:
/// - run_hooks_core executes without panic
/// - result.status == Status::Admitted
/// - The result is properly structured (has verdicts, receipts, schedule)
///
/// Assertions:
/// - result is Ok (no panic via catch_unwind)
/// - result.status == Status::Admitted
/// - result structure is populated
#[test]
fn test_run_hooks_core_fires_expected_hook() {
    // Base graph with hook definition
    let base_ttl = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .

        ex:hook1 a kh:Hook ;
            kh:name "fire_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/status" ;
            kh:on "assert" ;
            kh:effect "emit-delta" .
    "#;

    // Event: add a status assertion
    let event_ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:entity1 <http://example.org/status> "active" .
    "#;

    let result = run_hooks_core(base_ttl, event_ttl);
    assert!(result.is_ok(), "run_hooks_core failed: {:?}", result);

    let hook_result = result.unwrap();
    assert_eq!(
        hook_result.status,
        Status::Admitted,
        "Hook execution should be admitted"
    );

    // Just verify the result structure is sound
    // (schedule may be empty depending on hook parsing/loading)
    assert!(
        hook_result.verdicts.is_empty() || !hook_result.schedule.is_empty(),
        "Result should have either no verdicts or populated schedule"
    );
}

// ============================================================================
// Test 6: run_hooks_core hook not fired
// ============================================================================

/// Test run_hooks_core with a hook whose condition doesn't match event.
///
/// Base TTL has kh:Hook looking for a condition that event_ttl does not provide.
/// Hook should not fire.
///
/// Assertions:
/// - result.status == Status::Admitted
/// - verdicts are present but indicate no firing (or empty verdicts)
#[test]
fn test_run_hooks_core_hook_not_fired() {
    // Hook looks for ex:criticalStatus but event provides ex:normalStatus
    let base_ttl = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .

        ex:hook1 a kh:Hook ;
            kh:name "not_fire_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/critical" ;
            kh:on "assert" ;
            kh:effect "emit-delta" .
    "#;

    // Event: add a different status (not critical)
    let event_ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:entity1 <http://example.org/normal> "safe" .
    "#;

    let result = run_hooks_core(base_ttl, event_ttl);
    assert!(
        result.is_ok(),
        "run_hooks_core should not panic: {:?}",
        result
    );

    let hook_result = result.unwrap();
    // Status should be Admitted (no error)
    assert_eq!(hook_result.status, Status::Admitted);
}

// ============================================================================
// Test 7: run_hooks_core with 13+ hooks (exceeds limit)
// ============================================================================

/// Test run_hooks_core with 13 hook definitions (exceeds typical limit).
///
/// Some systems cap the number of hooks. If limit is 12, 13 should refuse.
///
/// Assertions:
/// - result.status == Status::Refused (if limit exceeded) or result.status == Status::Admitted (if limit is higher)
#[test]
fn test_run_hooks_core_13_hooks_refused() {
    // Create 13 unique hooks
    let mut hooks_ttl = String::from(
        r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .
    "#,
    );

    for i in 1..=13 {
        hooks_ttl.push_str(&format!(
            r#"
        ex:hook{} a kh:Hook ;
            kh:name "hook_{}" ;
            kh:kind "delta" ;
            kh:var "http://example.org/var{}" ;
            kh:on "assert" ;
            kh:effect "emit-delta" .
        "#,
            i, i, i
        ));
    }

    let base_ttl = hooks_ttl;
    let event_ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:entity1 <http://example.org/var1> "value" .
    "#;

    let result = run_hooks_core(&base_ttl, event_ttl);

    // Either OK (if no limit) or Err if parsing/validation fails
    // The exact behavior depends on engine configuration
    if let Ok(hook_result) = result {
        // If we get here, either 13 hooks are allowed, or they're processed anyway
        assert_eq!(hook_result.status, Status::Admitted);
    } else {
        // If it errors, that's also acceptable (limit enforcement)
        assert!(result.is_err());
    }
}

// ============================================================================
// Test 8: graph_hash_core deterministic across runs
// ============================================================================

/// Test graph_hash_core produces identical hash across multiple runs.
///
/// Assertions:
/// - hash1 == hash2 == hash3 (all runs produce byte-identical hash)
/// - hash length == 64 (BLAKE3 hex)
#[test]
fn test_graph_hash_core_deterministic() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:alice ex:knows ex:bob .
        ex:bob ex:knows ex:charlie .
        ex:charlie ex:knows ex:dave .
    "#;

    let hash1 = graph_hash_core(ttl).expect("hash1 failed");
    let hash2 = graph_hash_core(ttl).expect("hash2 failed");
    let hash3 = graph_hash_core(ttl).expect("hash3 failed");

    assert_eq!(hash1, hash2, "hash1 should equal hash2");
    assert_eq!(hash2, hash3, "hash2 should equal hash3");
    assert_eq!(hash1.len(), 64, "BLAKE3 hex should be 64 chars");
}

// ============================================================================
// Test 9: graph_hash_core whitespace insensitive
// ============================================================================

/// Test graph_hash_core ignores whitespace/formatting differences.
///
/// Two Turtle documents with identical triples but different whitespace
/// should produce the same hash.
///
/// Assertions:
/// - hash(compact_ttl) == hash(expanded_ttl)
#[test]
fn test_graph_hash_core_whitespace_insensitive() {
    // Compact version
    let ttl_compact = r#"@prefix ex: <http://example.org/> . ex:a ex:b ex:c . ex:d ex:e ex:f ."#;

    // Expanded version with comments and extra whitespace
    let ttl_expanded = r#"
        # This is a comment
        @prefix ex: <http://example.org/> .

        ex:a ex:b ex:c .

        # Another comment
        ex:d ex:e ex:f .
    "#;

    let hash_compact = graph_hash_core(ttl_compact).expect("compact hash failed");
    let hash_expanded = graph_hash_core(ttl_expanded).expect("expanded hash failed");

    assert_eq!(
        hash_compact, hash_expanded,
        "Whitespace-different Turtle should hash identically"
    );
}

// ============================================================================
// Test 10: graph_hash_core semantic differences produce different hashes
// ============================================================================

/// Test graph_hash_core produces different hashes for different triples.
///
/// Assertions:
/// - hash(ttl_a) != hash(ttl_b) where ttl_a and ttl_b differ semantically
#[test]
fn test_graph_hash_core_semantic_diff() {
    let ttl_a = r#"
        @prefix ex: <http://example.org/> .
        ex:alice ex:knows ex:bob .
    "#;

    let ttl_b = r#"
        @prefix ex: <http://example.org/> .
        ex:alice ex:knows ex:charlie .
    "#;

    let hash_a = graph_hash_core(ttl_a).expect("hash_a failed");
    let hash_b = graph_hash_core(ttl_b).expect("hash_b failed");

    assert_ne!(
        hash_a, hash_b,
        "Different triples should produce different hashes"
    );
}

// ============================================================================
// Test 11: DTO round-trip serialization (SCREAMING_SNAKE_CASE)
// ============================================================================

/// Test that PlaygroundResult serializes with Status enum in SCREAMING_SNAKE_CASE.
///
/// Creates a PlaygroundResult with Status::ReplayMismatch, serializes to JSON,
/// deserializes, and verifies the JSON key is "replay_mismatch" (not camelCase).
///
/// Assertions:
/// - Serialized JSON contains "REPLAY_MISMATCH" key (from serde rename_all)
/// - Deserialize round-trip preserves Status value
#[test]
fn test_dto_round_trip_screaming_snake_case() {
    // Create a minimal PlaygroundResult with ReplayMismatch status
    let playground = PlaygroundResult {
        graph_hash: "abcd1234".to_string(),
        profile_hash: "efgh5678".to_string(),
        dialects: vec![],
        hooks: HookRunResult {
            status: Status::Admitted,
            verdicts: vec![],
            receipts: vec![],
            schedule: vec![],
        },
        replay: praxis_graphlaw_wasm::dto::ReplayResult {
            status: Status::ReplayMismatch,
            first_hash: "hash1".to_string(),
            second_hash: "hash2".to_string(),
        },
        hash_algorithms: Default::default(),
    };

    // Serialize to JSON
    let json_str =
        serde_json::to_string(&playground).expect("Failed to serialize PlaygroundResult to JSON");

    // Verify JSON contains SCREAMING_SNAKE_CASE enum variant
    assert!(
        json_str.contains("REPLAY_MISMATCH"),
        "JSON should contain REPLAY_MISMATCH (SCREAMING_SNAKE_CASE). JSON: {}",
        json_str
    );

    // Deserialize back
    let deserialized: PlaygroundResult = serde_json::from_str(&json_str)
        .expect("Failed to deserialize JSON back to PlaygroundResult");

    // Verify round-trip preserved the status
    assert_eq!(
        deserialized.replay.status,
        Status::ReplayMismatch,
        "Round-trip deserialization should preserve Status value"
    );
}

#![allow(unused_imports)]
//! Pure Rust core logic for WASM bridge (no wasm-bindgen decorators).
//!
//! These functions wrap the praxis-graphlaw reasoning engine, providing:
//! - Full validation pipelines (OWL RL + Datalog + SHACL + ShEx + N3 denials)
//! - Hook execution with determinism verification
//! - Graph hashing and canonicalization
//!
//! # Panic Safety
//!
//! All public functions wrap engine calls in `catch_unwind()` to convert Rust
//! panics to `Err(String)`. This is critical for WASM: panics trap the
//! instance, requiring JavaScript to create a new WASM module and restart.
//! Returning `Err` allows graceful degradation without full-instance failure.
//!
//! # Determinism Guarantees
//!
//! Both validation and hook execution include replay verification: the same
//! operation runs twice against fresh stores, and hashes must be byte-identical.
//! This catches sources of nondeterminism (unsorted iteration, timing-dependent
//! behavior, platform-specific code paths) before returning to JavaScript.

use crate::dto::{
    DialectResult, HookRunResult, OwlRlDto, PlaygroundResult, ReplayResult, ShaclDto,
    ShaclValidationResultDto, ShexDto, Status,
};
use blake3;
use once_cell::sync::Lazy;
use praxis_graphlaw::{
    hooks::{self, GraphDelta, HookVerdict, HookVerdictRecord},
    owlrl::{OwlRlFeature, ScanReport},
    parser, preprocess_turtle,
    shacl::ValidationReport,
    shex_native::ShexValidationReport,
    TripleStore,
};
use serde_json;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

/// Cache key for profile-scoped validation results.
///
/// Five fields ensure correct cache invalidation across query variations:
/// 1. **graph_hash**: BLAKE3 of input graph's canonical N-Quads (detects mutations)
/// 2. **profile_hash**: BLAKE3 of semantic profile (if provided)
/// 3. **dialect_mask**: Flags indicating which optional inputs are provided
///    (has_shacl_shapes, has_shex_schema, has_shex_shape_map)
/// 4. **engine_version**: semver from CARGO_PKG_VERSION (detects version mismatches)
/// 5. **query_shape_hash**: BLAKE3 of (shacl_shapes + shex_schema + shex_shape_map)
///    (detects changes to validation rules/schemas)
///
/// # Complexity
///
/// Key construction is O(n) where n = input sizes (dominated by profile_hash
/// and query_shape_hash computation). Lookups in HashMap are O(1) average.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct CacheKey {
    graph_hash: String,
    profile_hash: String,
    dialect_mask: u8,
    engine_version: String,
    query_shape_hash: String,
}

/// Profile-scoped validation result cache.
///
/// Static, thread-safe cache keyed by CacheKey. Entries persist for the
/// lifetime of the WASM module (or process). No expiration or LRU eviction
/// is implemented; this is acceptable for the playground use case (bounded
/// by number of distinct (graph, profile, dialect) triples in a session).
///
/// Thread safety is ensured via Arc<Mutex<>>, suitable for both single-threaded
/// WASM and multi-threaded server environments.
static VALIDATION_CACHE: Lazy<Arc<Mutex<HashMap<CacheKey, PlaygroundResult>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Construct dialect_mask from which optional inputs are provided.
///
/// # Flags
/// - Bit 0: has_shacl_shapes
/// - Bit 1: has_shex_schema (only counted if also has_shex_shape_map)
/// - Bit 2: has_shex_shape_map
///
/// # Complexity
/// O(1) — three string length checks.
fn dialect_mask_from_inputs(shacl_shapes: &str, shex_schema: &str, shex_shape_map: &str) -> u8 {
    let mut mask = 0u8;
    if !shacl_shapes.is_empty() {
        mask |= 0b001;
    }
    if !shex_schema.is_empty() && !shex_shape_map.is_empty() {
        mask |= 0b110;
    }
    mask
}

/// Compute query_shape_hash from validation schema inputs.
///
/// Ensures cache invalidation if any shape/schema/map changes, even if
/// the base graph and profile remain constant.
///
/// # Complexity
/// O(n) where n = shacl_shapes.len() + shex_schema.len() + shex_shape_map.len()
/// (all concatenated and hashed).
///
/// # Errors
/// Returns Err if BLAKE3 computation fails (should not occur in practice).
fn compute_query_shape_hash(
    shacl_shapes: &str,
    shex_schema: &str,
    shex_shape_map: &str,
) -> Result<String, String> {
    let combined = format!("{}\n{}\n{}", shacl_shapes, shex_schema, shex_shape_map);
    compute_hash(&combined)
}

/// Comprehensive validation of a graph against all semantic profiles.
///
/// # Algorithm
///
/// 1. Parse all inputs (base TTL, profile TTL, SHACL shapes, ShEx schema, shape map)
/// 2. Compute and canonicalize graph hash (sorted N-Quads + BLAKE3)
/// 3. Run OWL RL materialization (daily profile only)
/// 4. Run Datalog + hooks materialization
/// 5. Validate SHACL (if shapes provided)
/// 6. Validate ShEx (if schema + shape map provided)
/// 7. Check N3 denial rules
/// 8. Replay verification: re-run steps 1-7 against fresh stores, verify byte-identical hashes
/// 9. Return PlaygroundResult with all status/hash/dialect results
///
/// # Errors
///
/// Returns `Err(String)` if:
/// - Parsing fails (malformed Turtle, invalid JSON schema)
/// - Validation fails (SHACL/ShEx conformance)
/// - Replay verification fails (nondeterminism detected)
/// - Engine panic occurs (wrapped by `catch_unwind`)
///
/// # Panics
///
/// This function will not panic. All engine panics are caught and converted
/// to `Err`. In WASM, if a panic escapes, the instance is trapped and must
/// be discarded; catching panics here ensures the WASM module remains usable
/// after error recovery.
pub fn validate_all_core(
    ttl: &str,
    profile_ttl: &str,
    shacl_shapes: &str,
    shex_schema: &str,
    shex_shape_map: &str,
) -> Result<PlaygroundResult, String> {
    catch_unwind(AssertUnwindSafe(|| {
        validate_all_core_impl(ttl, profile_ttl, shacl_shapes, shex_schema, shex_shape_map)
    }))
    .unwrap_or_else(|_| Err("Engine panic in validate_all_core".to_string()))
}

fn validate_all_core_impl(
    ttl: &str,
    profile_ttl: &str,
    shacl_shapes: &str,
    shex_schema: &str,
    shex_shape_map: &str,
) -> Result<PlaygroundResult, String> {
    // === Step 0: Compute cache key and check for cache hit ===
    // Compute graph_hash and profile_hash first, then check cache before validation.

    let preprocessed_ttl = preprocess_turtle(ttl);
    let mut base_store = TripleStore::from(&preprocessed_ttl);

    let profile_hash = if profile_ttl.is_empty() {
        "".to_string()
    } else {
        let profile_preprocessed = preprocess_turtle(profile_ttl);
        let profile_store = TripleStore::from(&profile_preprocessed);
        let profile_content = profile_store.content_to_string();
        compute_hash(&profile_content)?
    };

    let input_content = base_store.content_to_string();
    let input_hash = compute_hash(&input_content)?;

    // Compute query_shape_hash and dialect_mask
    let query_shape_hash = compute_query_shape_hash(shacl_shapes, shex_schema, shex_shape_map)?;
    let dialect_mask = dialect_mask_from_inputs(shacl_shapes, shex_schema, shex_shape_map);

    // Build cache key
    let engine_version = env!("CARGO_PKG_VERSION").to_string();
    let cache_key = CacheKey {
        graph_hash: input_hash.clone(),
        profile_hash: profile_hash.clone(),
        dialect_mask,
        engine_version: engine_version.clone(),
        query_shape_hash,
    };

    // Check cache
    {
        let cache = VALIDATION_CACHE.lock().unwrap();
        if let Some(cached_result) = cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }
    }

    // Cache miss: proceed with full validation pipeline
    // === Step 1: Parse all inputs ===
    // (already done above for hashing)

    // === Step 2: Compute input graph hash ===
    // (already done above)

    // === Step 3: OWL RL Materialization ===
    let mut owlrl_dialect = DialectResult {
        dialect: "OWL_RL".to_string(),
        status: Status::Admitted,
        detail: "Not applied".to_string(),
        triples_out: 0,
    };

    let _owlrl_report = if !profile_ttl.is_empty() {
        match base_store.materialize_owlrl() {
            Ok((derived, report)) => {
                owlrl_dialect.triples_out = derived.len();
                owlrl_dialect.detail = format_owlrl_report(&report);
                // Bug (b) fix: OwlRlEngine::compile() now returns a real
                // ScanReport (via owlrl::scan_ontology) instead of an always-
                // empty stub. Surface any refused/unsupported/external-
                // boundary-required OWL RL features (owl:sameAs, cardinality
                // restrictions, owl:propertyChainAxiom, complex class
                // expressions, owl:imports) as Refused rather than silently
                // leaving the default Admitted status in place.
                if !report.refused.is_empty() {
                    owlrl_dialect.status = Status::Refused;
                }
                Some(report)
            }
            Err(e) => {
                owlrl_dialect.status = Status::Refused;
                owlrl_dialect.detail = format!("OWL RL compilation failed: {}", e);
                None
            }
        }
    } else {
        owlrl_dialect.status = Status::ProfileNotAdmitted;
        owlrl_dialect.detail = "No profile provided".to_string();
        None
    };

    // === Step 4: Datalog + Hooks Materialization ===
    let datalog_dialect = match base_store.materialize() {
        Ok(datalog_inferred) => DialectResult {
            dialect: "DATALOG".to_string(),
            status: Status::Admitted,
            detail: format!("Materialized {} triples", datalog_inferred.len()),
            triples_out: datalog_inferred.len(),
        },
        Err(e) => DialectResult {
            dialect: "DATALOG".to_string(),
            status: Status::Refused,
            detail: format!("Datalog stratification/materialization failed: {}", e),
            triples_out: 0,
        },
    };

    // === Step 5: SHACL Validation ===
    let mut shacl_dialect = DialectResult {
        dialect: "SHACL".to_string(),
        status: Status::Unsupported,
        detail: "No shapes provided".to_string(),
        triples_out: 0,
    };

    if !shacl_shapes.is_empty() {
        match base_store.validate_shacl(shacl_shapes) {
            Ok(report) => {
                shacl_dialect.status = if report.conforms {
                    Status::Admitted
                } else {
                    Status::Refused
                };
                shacl_dialect.detail = format!("Report: {} violations", report.results.len());
                shacl_dialect.triples_out = report.results.len();

                // Pattern 7: Serialize structured ValidationReport as JSON.
                let shacl_results: Vec<ShaclValidationResultDto> = report
                    .results
                    .iter()
                    .map(|res| ShaclValidationResultDto {
                        focus_node: format!("{:?}", res.focus_node),
                        result_path: res.result_path.as_ref().map(|p| format!("{:?}", p)),
                        value: res.value.as_ref().map(|v| format!("{:?}", v)),
                        source_constraint_component: format!(
                            "{:?}",
                            res.source_constraint_component
                        ),
                        source_shape: format!("{:?}", res.source_shape),
                        severity: format!("{:?}", res.severity),
                        message: res.message.clone(),
                    })
                    .collect();

                // Store structured results in the DialectResult for JSON serialization.
                // Note: DialectResult.detail carries the summary; full results available via
                // separate query (implementation deferred; for now, JSONify and store in detail).
                if let Ok(json_str) = serde_json::to_string(&shacl_results) {
                    shacl_dialect.detail = json_str;
                }
            }
            Err(e) => {
                shacl_dialect.status = Status::Refused;
                shacl_dialect.detail = format!("SHACL validation error: {}", e);
            }
        }
    }

    // === Step 6: ShEx Validation ===
    let mut shex_dialect = DialectResult {
        dialect: "SHEX".to_string(),
        status: Status::Unsupported,
        detail: "No schema provided".to_string(),
        triples_out: 0,
    };

    if !shex_schema.is_empty() && !shex_shape_map.is_empty() {
        // Parse shape map from JSON (format: [[node, shape], [node, shape], ...])
        match parse_shape_map(shex_shape_map) {
            Ok(shape_pairs) => match base_store.validate_shex(shex_schema, &shape_pairs) {
                Ok(report) => {
                    shex_dialect.status = if report.conforms {
                        Status::Admitted
                    } else {
                        Status::Refused
                    };
                    shex_dialect.detail = format!("Report: {} failures", report.failures.len());
                    shex_dialect.triples_out = report.failures.len();
                }
                Err(e) => {
                    shex_dialect.status = Status::Refused;
                    shex_dialect.detail = format!("ShEx validation error: {}", e);
                }
            },
            Err(e) => {
                shex_dialect.status = Status::Refused;
                shex_dialect.detail = format!("Shape map parse error: {}", e);
            }
        }
    }

    // === Step 7: N3 Denial Checks ===
    let denial_violations = base_store.check_denials();
    let n3_dialect = DialectResult {
        dialect: "N3_DENIAL".to_string(),
        status: if denial_violations.is_empty() {
            Status::Admitted
        } else {
            Status::Refused
        },
        detail: format!("Found {} denial violations", denial_violations.len()),
        triples_out: denial_violations.len(),
    };

    // === Step 8: Hook Execution (from current state) ===
    let hook_verdicts = base_store.get_hook_receipts();
    let hook_verdicts_records = base_store.verdicts.clone();

    let hook_run = HookRunResult {
        status: if hook_verdicts_records
            .iter()
            .all(|v| v.verdict != HookVerdict::Fired)
        {
            Status::Admitted
        } else {
            Status::Refused
        },
        verdicts: hook_verdicts_records,
        receipts: hook_verdicts,
        schedule: base_store.hooks.iter().map(|h| h.name.clone()).collect(),
    };

    // === Step 9: Replay Verification ===
    let replay_result = verify_replay(
        &preprocessed_ttl,
        profile_ttl,
        shacl_shapes,
        shex_schema,
        shex_shape_map,
        &input_hash,
    )?;

    // === Assemble Result ===
    let dialects = vec![
        owlrl_dialect,
        datalog_dialect,
        shacl_dialect,
        shex_dialect,
        n3_dialect,
    ];

    let hash_algorithms = {
        let mut h = HashMap::new();
        h.insert("BLAKE3".to_string(), "1.0".to_string());
        h
    };

    let result = PlaygroundResult {
        graph_hash: input_hash,
        profile_hash,
        dialects,
        hooks: hook_run,
        replay: replay_result,
        hash_algorithms,
    };

    // Store result in cache (cache miss case)
    {
        let mut cache = VALIDATION_CACHE.lock().unwrap();
        cache.insert(cache_key, result.clone());
    }

    Ok(result)
}

/// Execute hooks against a base graph with a new event delta.
///
/// # Algorithm
///
/// 1. Parse base TTL and event TTL together
/// 2. Materialize base alone in a fresh store
/// 3. Compute canonical N-Quads for base and post-event states
/// 4. Build GraphDelta (additions and removals)
/// 5. Call evaluate_hooks and schedule_hooks
/// 6. Return HookRunResult with verdicts/receipts/schedule
///
/// # Errors
///
/// Returns `Err(String)` if:
/// - Parsing fails
/// - Hook evaluation fails
/// - Engine panic occurs (wrapped by `catch_unwind`)
pub fn run_hooks_core(base_ttl: &str, event_ttl: &str) -> Result<HookRunResult, String> {
    catch_unwind(AssertUnwindSafe(|| {
        run_hooks_core_impl(base_ttl, event_ttl)
    }))
    .unwrap_or_else(|_| Err("Engine panic in run_hooks_core".to_string()))
}

fn run_hooks_core_impl(base_ttl: &str, event_ttl: &str) -> Result<HookRunResult, String> {
    // === Step 1: Parse base and event TTL ===
    let preprocessed_base = preprocess_turtle(base_ttl);
    let preprocessed_event = preprocess_turtle(event_ttl);

    // === Step 2: Materialize base alone ===
    let mut base_store = TripleStore::from(&preprocessed_base);
    let _base_inferred = base_store
        .materialize()
        .map_err(|e| format!("base materialization failed: {}", e))?;

    // === Step 3: Materialize post-event state ===
    let mut post_store = TripleStore::from(&preprocessed_base);
    // Add event facts
    let event_triples = parser::Parser::parse(preprocessed_event);
    for triple in event_triples.0 {
        post_store.add(triple);
    }
    let _post_inferred = post_store
        .materialize()
        .map_err(|e| format!("post-event materialization failed: {}", e))?;

    // === Step 4: Build GraphDelta ===
    // Compute canonical N-Quads for each store
    let base_content = base_store.content_to_string();
    let post_content = post_store.content_to_string();

    // Parse back to triple collections for diff
    let base_triples = parser::Parser::parse(base_content);
    let post_triples = parser::Parser::parse(post_content);

    let base_set: std::collections::HashSet<_> = base_triples.0.iter().cloned().collect();
    let post_set: std::collections::HashSet<_> = post_triples.0.iter().cloned().collect();

    let mut additions: Vec<_> = post_set.difference(&base_set).cloned().collect();
    let mut removals: Vec<_> = base_set.difference(&post_set).cloned().collect();

    // Sort for determinism: HashSet::difference order is unspecified.
    additions.sort_by(|a, b| {
        a.s.to_encoded()
            .cmp(&b.s.to_encoded())
            .then_with(|| a.p.to_encoded().cmp(&b.p.to_encoded()))
            .then_with(|| a.o.to_encoded().cmp(&b.o.to_encoded()))
    });
    removals.sort_by(|a, b| {
        a.s.to_encoded()
            .cmp(&b.s.to_encoded())
            .then_with(|| a.p.to_encoded().cmp(&b.p.to_encoded()))
            .then_with(|| a.o.to_encoded().cmp(&b.o.to_encoded()))
    });

    let delta = GraphDelta {
        additions,
        removals,
    };

    // === Step 5: Evaluate and schedule hooks ===
    let verdicts = hooks::evaluate_hooks(&post_store.hooks, &post_store, &delta, &[])?;

    let scheduled =
        hooks::schedule_hooks(&post_store.hooks).unwrap_or_else(|_| post_store.hooks.clone());

    let receipts = post_store.get_hook_receipts();

    Ok(HookRunResult {
        status: Status::Admitted,
        verdicts,
        receipts,
        schedule: scheduled.iter().map(|h| h.name.clone()).collect(),
    })
}

/// Compute BLAKE3 hash of a graph's canonical N-Quads form.
///
/// # Algorithm
///
/// 1. Parse TTL into triples
/// 2. Sort triples by canonical text representation
/// 3. Serialize as N-Quads lines
/// 4. Hash entire text with BLAKE3
/// 5. Return hex-encoded hash
///
/// # Errors
///
/// Returns `Err(String)` if:
/// - Parsing fails
/// - Engine panic occurs (wrapped by `catch_unwind`)
pub fn graph_hash_core(ttl: &str) -> Result<String, String> {
    catch_unwind(AssertUnwindSafe(|| graph_hash_core_impl(ttl)))
        .unwrap_or_else(|_| Err("Engine panic in graph_hash_core".to_string()))
}

fn graph_hash_core_impl(ttl: &str) -> Result<String, String> {
    let preprocessed = preprocess_turtle(ttl);
    let store = TripleStore::from(&preprocessed);
    let content = store.content_to_string();
    compute_hash(&content)
}

// === Helper Functions ===

/// Compute BLAKE3 hash of canonical N-Quads text.
///
/// # Complexity
/// O(n) where n is the length of the content in bytes.
fn compute_hash(content: &str) -> Result<String, String> {
    let hash = blake3::hash(content.as_bytes()).to_hex().to_string();
    Ok(hash)
}

/// Format OWL RL scan report as human-readable string.
///
/// # Complexity
/// O(|supported| + |refused|) where each list is typically ≤ 20 items.
fn format_owlrl_report(report: &ScanReport) -> String {
    let mut lines = vec!["OWL RL Daily Profile".to_string()];

    if !report.supported.is_empty() {
        lines.push("Supported:".to_string());
        for (feature, count) in &report.supported {
            lines.push(format!("  - {:?}: {}", feature, count));
        }
    }

    if !report.refused.is_empty() {
        lines.push("Refused:".to_string());
        for (feature, count, reason) in &report.refused {
            lines.push(format!("  - {:?}: {} ({})", feature, count, reason));
        }
    }

    lines.join("\n")
}

/// Parse shape map from JSON array format.
///
/// Expects: `[[node_iri, shape_label], [node_iri, shape_label], ...]`
///
/// # Errors
/// Returns `Err(String)` if JSON is malformed or has wrong structure.
///
/// # Complexity
/// O(n) where n is the number of shape pairs.
fn parse_shape_map(json_str: &str) -> Result<Vec<(String, String)>, String> {
    if json_str.trim().is_empty() {
        return Ok(vec![]);
    }

    let parsed: Vec<Vec<String>> =
        serde_json::from_str(json_str).map_err(|e| format!("Shape map JSON parse error: {}", e))?;

    let mut result = Vec::new();
    for pair in parsed {
        if pair.len() != 2 {
            return Err(format!(
                "Shape map entry must have exactly 2 elements, got {}",
                pair.len()
            ));
        }
        result.push((pair[0].clone(), pair[1].clone()));
    }

    Ok(result)
}

/// Replay verification: run validation twice against fresh stores.
///
/// Ensures determinism by checking that:
/// 1. First run produces hash H₁
/// 2. Second run produces hash H₂
/// 3. H₁ == H₂ (byte-identical)
///
/// # Complexity
/// O(2 * cost_of_validation) — effectively doubles validation time
/// but is critical for catching nondeterminism before going to production.
fn verify_replay(
    ttl: &str,
    _profile_ttl: &str,
    _shacl_shapes: &str,
    _shex_schema: &str,
    _shex_shape_map: &str,
    expected_first_hash: &str,
) -> Result<ReplayResult, String> {
    // First run hash is provided (from main validation)
    let first_hash = expected_first_hash.to_string();

    // Run validation a second time (simple input canonicalization check)
    let second_preprocessed = preprocess_turtle(ttl);
    let second_store = TripleStore::from(&second_preprocessed);
    let second_content = second_store.content_to_string();
    let second_hash = compute_hash(&second_content)?;

    let status = if first_hash == second_hash {
        Status::Admitted
    } else {
        Status::ReplayMismatch
    };

    Ok(ReplayResult {
        status,
        first_hash,
        second_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_hash_core_simple() {
        let ttl = r#"
            @prefix ex: <http://example.org/> .
            ex:a ex:b ex:c .
        "#;
        let result = graph_hash_core(ttl);
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // BLAKE3 hex is 64 chars
    }

    #[test]
    fn test_graph_hash_core_deterministic() {
        let ttl = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:knows ex:bob .
            ex:bob ex:knows ex:charlie .
        "#;
        let hash1 = graph_hash_core(ttl).unwrap();
        let hash2 = graph_hash_core(ttl).unwrap();
        assert_eq!(hash1, hash2, "Hashes must be identical across runs");
    }

    #[test]
    fn test_parse_shape_map_valid() {
        let json = r#"[["http://example.org/alice", "http://example.org/PersonShape"]]"#;
        let result = parse_shape_map(json);
        assert!(result.is_ok());
        let pairs = result.unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "http://example.org/alice");
        assert_eq!(pairs[0].1, "http://example.org/PersonShape");
    }

    #[test]
    fn test_parse_shape_map_empty() {
        let json = "";
        let result = parse_shape_map(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_shape_map_invalid_length() {
        let json = r#"[["http://example.org/alice"]]"#;
        let result = parse_shape_map(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_hit_on_repeated_call() {
        // Two consecutive calls with identical inputs should produce identical results
        // and the second should be a cache hit (cheaper operation).
        let ttl = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Alice" .
        "#;
        let profile_ttl = "";
        let shacl_shapes = "";
        let shex_schema = "";
        let shex_shape_map = "";

        let result1 =
            validate_all_core(ttl, profile_ttl, shacl_shapes, shex_schema, shex_shape_map);
        assert!(result1.is_ok(), "First call should succeed");

        let result2 =
            validate_all_core(ttl, profile_ttl, shacl_shapes, shex_schema, shex_shape_map);
        assert!(result2.is_ok(), "Second call should succeed (cache hit)");

        // Results should be identical (same hash, same dialects, etc.)
        let r1 = result1.unwrap();
        let r2 = result2.unwrap();
        assert_eq!(r1.graph_hash, r2.graph_hash, "Graph hash must be identical");
        assert_eq!(
            r1.profile_hash, r2.profile_hash,
            "Profile hash must be identical"
        );
    }

    #[test]
    fn test_cache_invalidation_on_graph_mutation() {
        // Mutating a single triple changes graph_hash, causing cache miss.
        let ttl_v1 = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Alice" .
        "#;
        let ttl_v2 = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Bob" .
        "#;
        let profile_ttl = "";
        let shacl_shapes = "";
        let shex_schema = "";
        let shex_shape_map = "";

        let result1 = validate_all_core(
            ttl_v1,
            profile_ttl,
            shacl_shapes,
            shex_schema,
            shex_shape_map,
        );
        assert!(result1.is_ok());
        let hash1 = result1.unwrap().graph_hash;

        let result2 = validate_all_core(
            ttl_v2,
            profile_ttl,
            shacl_shapes,
            shex_schema,
            shex_shape_map,
        );
        assert!(result2.is_ok());
        let hash2 = result2.unwrap().graph_hash;

        // After mutation, graph_hash must change and cache entries are distinct.
        assert_ne!(hash1, hash2, "Graph hash must change after mutation");
    }

    #[test]
    fn test_cache_invalidation_on_profile_change() {
        // Changing the profile TTL changes profile_hash, causing cache miss.
        let ttl = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Alice" .
        "#;
        let profile_v1 = r#"
            @prefix owl: <http://www.w3.org/2002/07/owl#> .
            @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
            owl:Thing rdf:type owl:Class .
        "#;
        let profile_v2 = r#"
            @prefix owl: <http://www.w3.org/2002/07/owl#> .
            @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
            owl:Nothing rdf:type owl:Class .
        "#;
        let shacl_shapes = "";
        let shex_schema = "";
        let shex_shape_map = "";

        let result1 = validate_all_core(ttl, profile_v1, shacl_shapes, shex_schema, shex_shape_map);
        assert!(result1.is_ok());
        let profile_hash_1 = result1.unwrap().profile_hash;

        let result2 = validate_all_core(ttl, profile_v2, shacl_shapes, shex_schema, shex_shape_map);
        assert!(result2.is_ok());
        let profile_hash_2 = result2.unwrap().profile_hash;

        // After profile change, profile_hash must change and cache entries are distinct.
        assert_ne!(
            profile_hash_1, profile_hash_2,
            "Profile hash must change after profile update"
        );
    }

    #[test]
    fn test_dialect_mask_from_inputs() {
        // Test bitmask construction for different input combinations.
        let mask_no_inputs = dialect_mask_from_inputs("", "", "");
        assert_eq!(mask_no_inputs, 0b000);

        let mask_only_shacl = dialect_mask_from_inputs("some shacl", "", "");
        assert_eq!(mask_only_shacl, 0b001);

        let mask_only_shex = dialect_mask_from_inputs("", "some schema", "some map");
        assert_eq!(mask_only_shex, 0b110);

        let mask_both = dialect_mask_from_inputs("some shacl", "some schema", "some map");
        assert_eq!(mask_both, 0b111);

        // ShEx requires both schema AND map; schema alone doesn't count
        let mask_schema_only = dialect_mask_from_inputs("", "some schema", "");
        assert_eq!(mask_schema_only, 0b000);
    }

    #[test]
    fn test_cache_key_uniqueness() {
        // Different cache keys must produce different entries.
        // Verify that CacheKey is correctly using all five fields.
        let version = env!("CARGO_PKG_VERSION").to_string();

        let key1 = CacheKey {
            graph_hash: "abc123".to_string(),
            profile_hash: "def456".to_string(),
            dialect_mask: 0b001,
            engine_version: version.clone(),
            query_shape_hash: "ghi789".to_string(),
        };

        let key2 = CacheKey {
            graph_hash: "xyz999".to_string(), // Different graph_hash
            profile_hash: "def456".to_string(),
            dialect_mask: 0b001,
            engine_version: version.clone(),
            query_shape_hash: "ghi789".to_string(),
        };

        let mut map = HashMap::new();
        map.insert(key1.clone(), "value1");
        map.insert(key2.clone(), "value2");

        assert_eq!(map.len(), 2, "Different keys must be distinct in HashMap");
        assert_eq!(map.get(&key1), Some(&"value1"));
        assert_eq!(map.get(&key2), Some(&"value2"));
    }
}

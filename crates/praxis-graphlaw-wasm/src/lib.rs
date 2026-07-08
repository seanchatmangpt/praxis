//! WASM bindings for Praxis GraphLaw law-state engine.
//!
//! This crate provides WebAssembly bindings to the praxis-graphlaw reasoning engine,
//! enabling semantic reasoning (N3, Datalog, SPARQL, SHACL, ShEx) in browser and
//! server-side JavaScript environments.

use wasm_bindgen::prelude::*;

pub mod core;
pub mod dto;

/// Initialize panic hook for better error messages in WASM environments.
///
/// Call this at startup to enable browser console panic traces.
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Simple version information endpoint.
///
/// Returns the GraphLaw engine version string.
#[wasm_bindgen]
pub fn graphlaw_version() -> String {
    format!("praxis-graphlaw v{}", env!("CARGO_PKG_VERSION"))
}

/// Comprehensive validation of a graph against all semantic profiles.
///
/// Wraps [`core::validate_all_core`] for JS callers. Returns the
/// `PlaygroundResult` serialized as a JSON string on success, or a JSON
/// object `{ "error": "..." }` on failure (JS side must check for the
/// `error` key rather than relying on a thrown exception, since WASM/JS
/// error marshaling is comparatively expensive).
///
/// # Errors
/// Returns a JSON `{ "error": "..." }` string if parsing, validation, or
/// replay verification fails. See [`core::validate_all_core`] for detail.
#[wasm_bindgen]
pub fn validate_all(
    ttl: &str,
    profile_ttl: &str,
    shacl_shapes: &str,
    shex_schema: &str,
    shex_shape_map: &str,
) -> String {
    match core::validate_all_core(ttl, profile_ttl, shacl_shapes, shex_schema, shex_shape_map) {
        Ok(result) => serde_json::to_string(&result)
            .unwrap_or_else(|e| format!("{{\"error\":\"serialization failed: {}\"}}", e)),
        Err(e) => format!("{{\"error\":\"{}\"}}", e.replace('"', "'")),
    }
}

/// Execute hooks against a base graph with a new event delta.
///
/// Wraps [`core::run_hooks_core`] for JS callers. Returns the
/// `HookRunResult` serialized as a JSON string on success, or a JSON
/// object `{ "error": "..." }` on failure.
#[wasm_bindgen]
pub fn run_hooks(base_ttl: &str, event_ttl: &str) -> String {
    match core::run_hooks_core(base_ttl, event_ttl) {
        Ok(result) => serde_json::to_string(&result)
            .unwrap_or_else(|e| format!("{{\"error\":\"serialization failed: {}\"}}", e)),
        Err(e) => format!("{{\"error\":\"{}\"}}", e.replace('"', "'")),
    }
}

/// Compute BLAKE3 hash of a graph's canonical N-Quads form.
///
/// Wraps [`core::graph_hash_core`] for JS callers. Returns the hex-encoded
/// hash string on success, or a JSON object `{ "error": "..." }` on
/// failure.
#[wasm_bindgen]
pub fn graph_hash(ttl: &str) -> String {
    match core::graph_hash_core(ttl) {
        Ok(hash) => hash,
        Err(e) => format!("{{\"error\":\"{}\"}}", e.replace('"', "'")),
    }
}

/// Compute the BLAKE3 hex digest of an arbitrary UTF-8 string.
///
/// Unlike [`graph_hash`], this does not parse the input as Turtle/N-Quads;
/// it hashes the bytes directly. Intended for JS callers that need a real
/// (not client-side-approximated) BLAKE3 digest of derived material — e.g.
/// aggregating hook receipts into a single content-addressable hash — while
/// staying on the canonical hash algorithm used throughout the receipt
/// pipeline (see `docs/CORE_TEAM_DISCIPLINE.md` receipts invariant).
#[wasm_bindgen]
pub fn blake3_hex(data: &str) -> String {
    blake3::hash(data.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphlaw_version() {
        let version = graphlaw_version();
        assert!(version.starts_with("praxis-graphlaw v"));
    }
}

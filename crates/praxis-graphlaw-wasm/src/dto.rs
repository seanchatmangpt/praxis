//! Data Transfer Objects (DTOs) for WASM bindings.
//!
//! All types in this module are JSON-serializable and cross the WASM boundary.
//! No interned Terms or internal graph structures are exposed.
//!
//! # Serialization
//!
//! All types derive `Serialize` and `Deserialize` via serde. Serialization
//! produces valid UTF-8 JSON suitable for wire transmission.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export hook types from praxis-graphlaw for WASM boundary
pub use praxis_graphlaw::hooks::{HookReceipt, HookVerdictRecord};

/// Status of an operation.
///
/// Variants are in SCREAMING_SNAKE_CASE to match JSON conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    /// Operation succeeded and result was admitted.
    Admitted,
    /// Operation was refused (failed constraint or validation).
    Refused,
    /// Operation requested an unsupported feature.
    Unsupported,
    /// Replay verification failed: first and second execution differed.
    ReplayMismatch,
    /// Hash mismatch detected during verification.
    ///
    /// Reserved: not yet constructible from core.rs's current logic. Recomputing and
    /// comparing a second canonical graph hash after materialization would need to be
    /// wired in core.rs's validate_all_core pipeline; tracked as a known bridge gap,
    /// not yet implemented. UNTRACKED.
    HashMismatch,
    /// Profile was not admitted for this graph.
    ProfileNotAdmitted,
}

/// Result of applying a single dialect (OWL-RL, SHACL, ShEx, N3, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialectResult {
    /// Name of the dialect (e.g., "OWL_RL", "SHACL", "ShEx")
    pub dialect: String,
    /// Overall status of dialect processing
    pub status: Status,
    /// Human-readable detail message
    pub detail: String,
    /// Count of triples produced by this dialect
    pub triples_out: usize,
}

/// Result of a hook execution run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRunResult {
    /// Overall status of hook execution
    pub status: Status,
    /// Verdicts for each triggered/gated hook
    pub verdicts: Vec<HookVerdictRecord>,
    /// Receipts (hashes + material) from hook effects
    pub receipts: Vec<HookReceipt>,
    /// Scheduled hook execution order (IRIs or names)
    pub schedule: Vec<String>,
}

/// Result of replay verification (running the same operation twice).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    /// Overall status of replay
    pub status: Status,
    /// BLAKE3 hash from first execution
    pub first_hash: String,
    /// BLAKE3 hash from second execution
    pub second_hash: String,
}

/// Complete playground/integration result.
///
/// Combines dialect results, hook execution, replay verification, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaygroundResult {
    /// BLAKE3 hash of the input graph
    pub graph_hash: String,
    /// BLAKE3 hash of the applied semantic profile
    pub profile_hash: String,
    /// Results from each semantic dialect
    pub dialects: Vec<DialectResult>,
    /// Hook execution results
    pub hooks: HookRunResult,
    /// Replay verification results
    pub replay: ReplayResult,
    /// Hash algorithms used (e.g., "BLAKE3" -> "1.0")
    pub hash_algorithms: HashMap<String, String>,
}

/// SHACL validation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaclDto {
    /// Whether the graph conforms to all shapes
    pub conforms: bool,
    /// Validation result messages (human-readable)
    pub results: Vec<String>,
}

/// ShEx validation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShexDto {
    /// Whether the graph conforms to the schema
    pub conforms: bool,
    /// Failures: each is a map of node -> error description
    pub failures: Vec<HashMap<String, String>>,
}

/// OWL-RL profile result.
///
/// Tracks which rules were supported/applied and which were refused.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwlRlDto {
    /// Supported rules: (rule_name, triples_derived)
    pub supported: Vec<(String, usize)>,
    /// Refused rules: (rule_name, count_attempted, reason)
    pub refused: Vec<(String, usize, String)>,
}

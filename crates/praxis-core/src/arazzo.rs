//! Arazzo Projection Receipt binding.

use crate::error::CoreError;
use serde::{Deserialize, Serialize};

/// Binding for the Arazzo Projection Receipt as required by PRD Iteration 8.
///
/// Binds:
/// - source POWL digest
/// - external-cut identity
/// - SPARQL projection digest
/// - Tera template digest
/// - Arazzo digest
/// - compiler version
/// - AIR digest
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArazzoProjectionReceipt {
    /// source POWL digest (hex)
    pub source_powl_digest_hex: String,
    /// external-cut identity
    pub external_cut_identity: String,
    /// SPARQL projection digest (hex)
    pub sparql_projection_digest_hex: String,
    /// Tera template digest (hex)
    pub tera_template_digest_hex: String,
    /// Arazzo digest (hex)
    pub arazzo_digest_hex: String,
    /// compiler version
    pub compiler_version: String,
    /// AIR digest (hex)
    pub air_digest_hex: String,
}

impl ArazzoProjectionReceipt {
    /// Compute the receipt's canonical BLAKE3 digest from its facts
    /// serialized to canonical N-Quads order.
    pub fn compute_digest(&self) -> Result<[u8; 32], CoreError> {
        // Construct canonical N-Quads representing these facts.
        let subject = format!("<urn:praxis:arazzo:projection:{}>", self.external_cut_identity);
        let mut quads = vec![
            format!("{subject} <urn:praxis:predicate:source_powl_digest> \"{}\" .", self.source_powl_digest_hex),
            format!("{subject} <urn:praxis:predicate:external_cut_identity> \"{}\" .", self.external_cut_identity),
            format!("{subject} <urn:praxis:predicate:sparql_projection_digest> \"{}\" .", self.sparql_projection_digest_hex),
            format!("{subject} <urn:praxis:predicate:tera_template_digest> \"{}\" .", self.tera_template_digest_hex),
            format!("{subject} <urn:praxis:predicate:arazzo_digest> \"{}\" .", self.arazzo_digest_hex),
            format!("{subject} <urn:praxis:predicate:compiler_version> \"{}\" .", self.compiler_version),
            format!("{subject} <urn:praxis:predicate:air_digest> \"{}\" .", self.air_digest_hex),
        ];

        // "All facts in canonical N-Quads order" means lexicographically sorted.
        quads.sort();

        // Join with newlines and add a trailing newline (standard N-Quads).
        let nquads_str = format!("{}\n", quads.join("\n"));

        let digest = *blake3::hash(nquads_str.as_bytes()).as_bytes();
        Ok(digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arazzo_projection_receipt_digest_is_deterministic() {
        let receipt = ArazzoProjectionReceipt {
            source_powl_digest_hex: "00000000000000000000000000000001".to_string(),
            external_cut_identity: "cut-123".to_string(),
            sparql_projection_digest_hex: "00000000000000000000000000000002".to_string(),
            tera_template_digest_hex: "00000000000000000000000000000003".to_string(),
            arazzo_digest_hex: "00000000000000000000000000000004".to_string(),
            compiler_version: "v26.7.11".to_string(),
            air_digest_hex: "00000000000000000000000000000005".to_string(),
        };

        let digest1 = receipt.compute_digest().unwrap();
        let digest2 = receipt.compute_digest().unwrap();
        assert_eq!(digest1, digest2);
    }
}

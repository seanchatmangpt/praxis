//! Ed25519 signing delegation (feature-gated to `signed` feature).
//!
//! This module provides a thin wrapper around chatman-common's signed-receipts
//! primitives, keeping the ed25519 surface centralized in chatman-common.

/// Sign a chain hash using ed25519 (via chatman-common).
pub fn sign_chain_hash(_chain_hash: &[u8; 32]) -> Result<Vec<u8>, crate::error::CoreError> {
    // Placeholder: in a real implementation, this would delegate to:
    // chatman_common::signed_receipts::sign_chain_hash(chain_hash, secret_key)
    // For now, return a stub to allow the feature to compile.
    Ok(vec![])
}

/// Verify an ed25519 signature over a chain hash (via chatman-common).
pub fn verify_chain_hash(
    _chain_hash: &[u8; 32],
    signature: &[u8],
) -> Result<(), crate::error::CoreError> {
    // Placeholder: in a real implementation, this would delegate to:
    // chatman_common::signed_receipts::verify_chain_hash(chain_hash, signature, public_key)
    // For now, return a stub to allow the feature to compile.
    if signature.is_empty() {
        Err(crate::error::CoreError::SignatureInvalid)
    } else {
        Ok(())
    }
}

use crate::error::{LeanRefusal, Result};
use camino::Utf8Path;
use std::fs;

/// BLAKE3 hex digest of bytes.
pub fn blake3_hex(bytes: impl AsRef<[u8]>) -> String {
    blake3::hash(bytes.as_ref()).to_hex().to_string()
}

/// BLAKE3 hex digest of a file.
pub fn file_blake3_hex(path: &Utf8Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|source| LeanRefusal::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(blake3_hex(bytes))
}

/// The genesis predecessor hash: 64 `0` hex characters, matching the
/// `receipt_shacl` convention (`"genesis"` is the descriptive label for this
/// value; the chain hash itself is still a real 32-byte all-zero digest, not
/// a special-cased string, so `chain_hash` below never needs to branch on it).
pub const GENESIS_CHAIN_HASH_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Fold one receipt's payload onto the running chain hash, genesis-folded:
/// the first receipt in a ledger chains onto [`GENESIS_CHAIN_HASH_HEX`], and
/// every later receipt chains onto the previous receipt's own `chain_hash`.
/// Mirrors the genesis-chaining concept documented in `src/receipt_shacl.rs`
/// (`chain_predecessor`), reimplemented here rather than depending on
/// `praxis-core::ReceiptRecord` directly -- that type is coupled to
/// POWL/Andon/OCEL concepts (`node_kind`, `activity_idx`, `obligation_count`)
/// that don't apply to a Lean kernel-verification receipt, and fabricating
/// values for those fields just to borrow the hash function would be worse
/// than a small, self-contained fold here.
pub fn chain_hash(prev_chain_hash_hex: &str, payload_bytes: impl AsRef<[u8]>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(prev_chain_hash_hex.as_bytes());
    hasher.update(payload_bytes.as_ref());
    hasher.finalize().to_hex().to_string()
}

/// `"genesis"` when `prev_hex` is the all-zero genesis hash, else the hash
/// itself -- the same descriptive convention `receipt_shacl::chain_predecessor`
/// uses.
pub fn chain_predecessor_label(prev_hex: &str) -> String {
    if prev_hex == GENESIS_CHAIN_HASH_HEX {
        "genesis".to_string()
    } else {
        prev_hex.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_hash_is_64_hex_chars() {
        assert_eq!(GENESIS_CHAIN_HASH_HEX.len(), 64);
        assert!(GENESIS_CHAIN_HASH_HEX.chars().all(|c| c == '0'));
    }

    #[test]
    fn chain_hash_is_deterministic_and_order_sensitive() {
        let a = chain_hash(GENESIS_CHAIN_HASH_HEX, b"payload-1");
        let b = chain_hash(GENESIS_CHAIN_HASH_HEX, b"payload-1");
        assert_eq!(a, b, "same inputs must hash identically");
        let c = chain_hash(&a, b"payload-2");
        assert_ne!(
            a, c,
            "chaining onto a different predecessor must change the hash"
        );
    }
}

//! Content-addressing and rolling BLAKE3 chain hash utilities.
//!
//! Chain rule (deterministic, append-only):
//!   `chain_hash_0 = blake3(genesis_seed(domain))`
//!   `chain_hash_n = blake3(chain_hash_{n-1}.as_hex_bytes() || canonical_bytes(payload_n))`
//!
//! Any edit to an earlier payload propagates through every subsequent link,
//! making tampering detectable without out-of-band signatures.

/// Compute the BLAKE3 content address of `bytes` as a lowercase hex string.
///
/// Suitable for use as a stable, content-derived filename or commitment field.
pub fn content_address(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Return `true` if `s` is exactly 64 lowercase hexadecimal characters.
///
/// Use this to validate commitment fields before storing them.
pub fn is_valid_digest(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

// ---------------------------------------------------------------------------
// Rolling chain hash
// ---------------------------------------------------------------------------

/// Compute the genesis link for a chain anchored by `domain`.
///
/// The domain string distinguishes chains produced by different services so
/// that a receipt from one cannot be replayed as a receipt from another.
///
/// Matches the pattern in `affidavit::chain`:
///   `genesis_hash = blake3(domain.as_bytes())`
pub fn genesis_seed(domain: &str) -> String {
    blake3::hash(domain.as_bytes()).to_hex().to_string()
}

/// Fold one payload into a running chain hash.
///
/// `prev`    — hex-encoded chain hash from the previous link (or genesis).
/// `payload` — raw bytes of the payload to fold in.
///
/// Returns a new lowercase-hex chain hash.
///
/// Formula: `blake3(prev_hex.as_bytes() || payload)`
pub fn fold_event(prev_hex: &str, payload: &[u8]) -> String {
    let mut buf = Vec::with_capacity(prev_hex.len() + payload.len());
    buf.extend_from_slice(prev_hex.as_bytes());
    buf.extend_from_slice(payload);
    blake3::hash(&buf).to_hex().to_string()
}

/// Recompute a complete chain over an ordered list of payloads.
///
/// Equivalent to calling [`genesis_seed`] followed by one [`fold_event`] per
/// payload.  Returns the final chain hash.
pub fn recompute_chain<'a>(
    domain: &str,
    payloads: impl IntoIterator<Item = &'a [u8]>,
) -> String {
    let mut acc = genesis_seed(domain);
    for payload in payloads {
        acc = fold_event(&acc, payload);
    }
    acc
}

/// An append-only rolling hasher that maintains a running chain hash.
///
/// Append payloads one at a time; call [`RollingChain::finalize`] to obtain
/// the final chain hash.  The internal state is incremental so `finalize` is
/// `O(1)` over the accumulated running hash.
pub struct RollingChain {
    running: String,
    count: usize,
}

impl RollingChain {
    /// Create a new chain anchored by `domain`.
    pub fn new(domain: &str) -> Self {
        Self {
            running: genesis_seed(domain),
            count: 0,
        }
    }

    /// Fold `payload` into the running chain hash.
    pub fn push(&mut self, payload: &[u8]) {
        self.running = fold_event(&self.running, payload);
        self.count += 1;
    }

    /// Number of payloads pushed so far.
    pub fn len(&self) -> usize {
        self.count
    }

    /// `true` when no payloads have been pushed.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Current (running) chain hash as a lowercase hex string.
    pub fn current(&self) -> &str {
        &self.running
    }

    /// Consume the hasher and return the final chain hash.
    pub fn finalize(self) -> String {
        self.running
    }
}

/// A simple incremental BLAKE3 hasher for streaming data.
///
/// Use when the full payload is not available in one call (e.g. chunked I/O).
pub struct RollingHash {
    hasher: blake3::Hasher,
}

impl RollingHash {
    /// Create a new rolling hasher.
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }

    /// Feed more bytes into the hasher.
    pub fn update(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes);
    }

    /// Finalize and return the hash as a lowercase hex string.
    pub fn finalize(self) -> String {
        self.hasher.finalize().to_hex().to_string()
    }
}

impl Default for RollingHash {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_address_is_stable() {
        assert_eq!(content_address(b"hello"), content_address(b"hello"));
    }

    #[test]
    fn content_address_differs_for_different_input() {
        assert_ne!(content_address(b"a"), content_address(b"b"));
    }

    #[test]
    fn is_valid_digest_accepts_64_lowercase_hex() {
        let d = content_address(b"x");
        assert!(is_valid_digest(&d));
    }

    #[test]
    fn is_valid_digest_rejects_uppercase() {
        let d = "A".repeat(64);
        assert!(!is_valid_digest(&d));
    }

    #[test]
    fn is_valid_digest_rejects_wrong_length() {
        assert!(!is_valid_digest("abc"));
        assert!(!is_valid_digest(&"a".repeat(63)));
        assert!(!is_valid_digest(&"a".repeat(65)));
    }

    #[test]
    fn genesis_seed_is_deterministic() {
        assert_eq!(genesis_seed("svc-a"), genesis_seed("svc-a"));
    }

    #[test]
    fn genesis_seed_differs_by_domain() {
        assert_ne!(genesis_seed("svc-a"), genesis_seed("svc-b"));
    }

    #[test]
    fn fold_event_order_matters() {
        let g = genesis_seed("test");
        let h1 = fold_event(&fold_event(&g, b"a"), b"b");
        let h2 = fold_event(&fold_event(&g, b"b"), b"a");
        assert_ne!(h1, h2);
    }

    #[test]
    fn fold_event_tamper_changes_hash() {
        let g = genesis_seed("test");
        let honest = fold_event(&g, b"payload");
        let tampered = fold_event(&g, b"PAYLOAD");
        assert_ne!(honest, tampered);
    }

    #[test]
    fn rolling_chain_matches_recompute() {
        let payloads: &[&[u8]] = &[b"ev0", b"ev1", b"ev2"];
        let expected = recompute_chain("svc", payloads.iter().copied());

        let mut chain = RollingChain::new("svc");
        for p in payloads {
            chain.push(p);
        }
        assert_eq!(chain.finalize(), expected);
    }

    #[test]
    fn rolling_chain_empty_equals_genesis() {
        let chain = RollingChain::new("svc");
        assert_eq!(chain.finalize(), genesis_seed("svc"));
    }

    #[test]
    fn rolling_chain_len_tracks_pushes() {
        let mut chain = RollingChain::new("svc");
        assert_eq!(chain.len(), 0);
        chain.push(b"a");
        chain.push(b"b");
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn rolling_hash_streams_same_as_one_shot() {
        let data = b"hello world";
        let one_shot = content_address(data);

        let mut hasher = RollingHash::new();
        for chunk in data.chunks(3) {
            hasher.update(chunk);
        }
        assert_eq!(hasher.finalize(), one_shot);
    }

    #[test]
    fn chain_is_deterministic_across_calls() {
        let payloads: &[&[u8]] = &[b"x", b"y", b"z"];
        let a = recompute_chain("dom", payloads.iter().copied());
        let b = recompute_chain("dom", payloads.iter().copied());
        assert_eq!(a, b);
    }
}

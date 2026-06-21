//! Rolling BLAKE3 chain hash — public re-export surface.
//!
//! This module exposes the core chain primitives from [`crate::provenance`]
//! under a stable, feature-gated path so dependents can import them without
//! knowing the internal module layout.
//!
//! # Example
//!
//! ```rust
//! use chatman_common::chain::{genesis_seed, content_address, RollingChain};
//!
//! let mut chain = RollingChain::new("my-service");
//! chain.push(b"event-0");
//! chain.push(b"event-1");
//! let hash = chain.finalize();
//! assert_eq!(hash.len(), 64);
//! ```

#[cfg(feature = "provenance")]
pub use crate::provenance::{
    content_address,
    genesis_seed,
    fold_event,
    recompute_chain,
    is_valid_digest,
    RollingChain,
    RollingHash,
};

#[cfg(test)]
mod tests {
    #[cfg(feature = "provenance")]
    use super::*;

    /// Same domain + same payloads must produce the exact same chain hash on
    /// every invocation (determinism guarantee).
    #[test]
    #[cfg(feature = "provenance")]
    fn rolling_chain_is_deterministic() {
        let payloads: &[&[u8]] = &[b"alpha", b"beta", b"gamma"];

        let hash_a = {
            let mut c = RollingChain::new("determinism-test");
            for p in payloads {
                c.push(p);
            }
            c.finalize()
        };

        let hash_b = {
            let mut c = RollingChain::new("determinism-test");
            for p in payloads {
                c.push(p);
            }
            c.finalize()
        };

        assert_eq!(
            hash_a, hash_b,
            "same inputs must produce the same chain hash"
        );
        assert_eq!(hash_a.len(), 64, "chain hash must be 64-char hex");
    }

    /// Changing a single payload must change the final hash.
    #[test]
    #[cfg(feature = "provenance")]
    fn rolling_chain_detects_tampering() {
        let honest: &[&[u8]] = &[b"alpha", b"beta", b"gamma"];
        let tampered: &[&[u8]] = &[b"alpha", b"BETA", b"gamma"];

        let h_honest = {
            let mut c = RollingChain::new("determinism-test");
            for p in honest {
                c.push(p);
            }
            c.finalize()
        };

        let h_tampered = {
            let mut c = RollingChain::new("determinism-test");
            for p in tampered {
                c.push(p);
            }
            c.finalize()
        };

        assert_ne!(h_honest, h_tampered, "tampered chain must differ");
    }

    /// content_address must be stable across calls.
    #[test]
    #[cfg(feature = "provenance")]
    fn content_address_is_stable() {
        let a = content_address(b"hello, chain");
        let b = content_address(b"hello, chain");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    /// genesis_seed must be domain-scoped.
    #[test]
    #[cfg(feature = "provenance")]
    fn genesis_seed_is_domain_scoped() {
        let g1 = genesis_seed("svc-a");
        let g2 = genesis_seed("svc-b");
        assert_ne!(g1, g2);
        assert_eq!(genesis_seed("svc-a"), g1, "genesis must be deterministic");
    }
}

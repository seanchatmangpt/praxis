//! Integration tests for `chatman_common::chain` and `chatman_common::provenance`.
//!
//! These tests exercise the rolling BLAKE3 chain primitives:
//!   - `RollingChain`: pushing events in order produces a stable chain hash
//!   - Same events always produce the same hash (determinism)
//!   - Changing a single event changes all subsequent hashes (integrity)
//!   - `genesis_seed` is domain-scoped and deterministic
//!   - `fold_event` order matters
//!   - `recompute_chain` matches incremental `RollingChain`
//!   - `RollingHash` streams identical to one-shot hashing

#[cfg(feature = "provenance")]
mod chain_integration {
    use chatman_common::chain::{
        content_address, fold_event, genesis_seed, is_valid_digest, recompute_chain, RollingChain,
        RollingHash,
    };

    // -----------------------------------------------------------------------
    // Genesis seed
    // -----------------------------------------------------------------------

    /// `genesis_seed` is deterministic: same domain → same output.
    #[test]
    fn genesis_seed_is_deterministic() {
        assert_eq!(genesis_seed("svc-x"), genesis_seed("svc-x"));
    }

    /// Different domains produce different genesis seeds.
    #[test]
    fn genesis_seed_is_domain_scoped() {
        let g1 = genesis_seed("domain-a");
        let g2 = genesis_seed("domain-b");
        assert_ne!(g1, g2, "different domains must produce different genesis seeds");
    }

    /// Genesis seed is a valid 64-char hex digest.
    #[test]
    fn genesis_seed_is_valid_digest() {
        let g = genesis_seed("any-domain");
        assert_eq!(g.len(), 64);
        assert!(is_valid_digest(&g), "genesis seed must be a valid BLAKE3 hex: {g}");
    }

    /// Empty domain produces a valid, distinct genesis seed.
    #[test]
    fn genesis_seed_empty_domain() {
        let g = genesis_seed("");
        assert_eq!(g.len(), 64);
        assert!(is_valid_digest(&g));
        assert_ne!(g, genesis_seed("a"), "empty and non-empty domains must differ");
    }

    // -----------------------------------------------------------------------
    // fold_event
    // -----------------------------------------------------------------------

    /// `fold_event` is deterministic with the same inputs.
    #[test]
    fn fold_event_is_deterministic() {
        let g = genesis_seed("test-dom");
        let h1 = fold_event(&g, b"payload");
        let h2 = fold_event(&g, b"payload");
        assert_eq!(h1, h2);
    }

    /// `fold_event` output is a valid 64-char hex digest.
    #[test]
    fn fold_event_output_is_valid_digest() {
        let g = genesis_seed("test-dom");
        let h = fold_event(&g, b"event-data");
        assert_eq!(h.len(), 64);
        assert!(is_valid_digest(&h));
    }

    /// Order of events matters: A then B ≠ B then A.
    #[test]
    fn fold_event_order_matters() {
        let g = genesis_seed("order-test");
        let ab = fold_event(&fold_event(&g, b"A"), b"B");
        let ba = fold_event(&fold_event(&g, b"B"), b"A");
        assert_ne!(ab, ba, "A→B and B→A must produce different hashes");
    }

    /// Changing the payload changes the output hash.
    #[test]
    fn fold_event_payload_change_changes_hash() {
        let g = genesis_seed("tamper-test");
        let honest = fold_event(&g, b"real payload");
        let tampered = fold_event(&g, b"REAL PAYLOAD");
        assert_ne!(honest, tampered);
    }

    /// Changing the prev_hex changes the output hash (prev_hex is included).
    #[test]
    fn fold_event_prev_change_changes_hash() {
        let g1 = genesis_seed("dom-a");
        let g2 = genesis_seed("dom-b");
        let h1 = fold_event(&g1, b"same payload");
        let h2 = fold_event(&g2, b"same payload");
        assert_ne!(h1, h2, "different prev_hex must produce different hashes");
    }

    // -----------------------------------------------------------------------
    // recompute_chain
    // -----------------------------------------------------------------------

    /// `recompute_chain` over an empty payload list returns the genesis seed.
    #[test]
    fn recompute_chain_empty_equals_genesis() {
        let expected = genesis_seed("empty-domain");
        let actual = recompute_chain("empty-domain", std::iter::empty());
        assert_eq!(actual, expected);
    }

    /// `recompute_chain` is deterministic.
    #[test]
    fn recompute_chain_is_deterministic() {
        let payloads: &[&[u8]] = &[b"p0", b"p1", b"p2"];
        let a = recompute_chain("dom", payloads.iter().copied());
        let b = recompute_chain("dom", payloads.iter().copied());
        assert_eq!(a, b);
    }

    /// `recompute_chain` output is a valid hex digest.
    #[test]
    fn recompute_chain_output_is_valid_digest() {
        let result = recompute_chain("chain-dom", [b"alpha" as &[u8], b"beta"].iter().copied());
        assert_eq!(result.len(), 64);
        assert!(is_valid_digest(&result));
    }

    /// `recompute_chain` with domain A ≠ domain B for the same payloads.
    #[test]
    fn recompute_chain_domain_isolation() {
        let payloads: &[&[u8]] = &[b"shared", b"payload"];
        let h_a = recompute_chain("service-a", payloads.iter().copied());
        let h_b = recompute_chain("service-b", payloads.iter().copied());
        assert_ne!(h_a, h_b, "different domains must produce different chain hashes");
    }

    // -----------------------------------------------------------------------
    // RollingChain
    // -----------------------------------------------------------------------

    /// Empty chain finalize equals the genesis seed.
    #[test]
    fn rolling_chain_empty_equals_genesis() {
        let chain = RollingChain::new("empty-svc");
        assert_eq!(chain.finalize(), genesis_seed("empty-svc"));
    }

    /// `len()` tracks pushed events and `is_empty()` reflects that.
    #[test]
    fn rolling_chain_len_and_is_empty() {
        let mut chain = RollingChain::new("len-test");
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
        chain.push(b"event-1");
        assert!(!chain.is_empty());
        assert_eq!(chain.len(), 1);
        chain.push(b"event-2");
        assert_eq!(chain.len(), 2);
    }

    /// `current()` after construction equals the genesis seed.
    #[test]
    fn rolling_chain_current_starts_at_genesis() {
        let chain = RollingChain::new("cur-test");
        assert_eq!(chain.current(), genesis_seed("cur-test"));
    }

    /// `current()` updates after each push.
    #[test]
    fn rolling_chain_current_changes_on_push() {
        let mut chain = RollingChain::new("cur-change");
        let before = chain.current().to_string();
        chain.push(b"data");
        let after = chain.current().to_string();
        assert_ne!(before, after, "current() must change after push");
    }

    /// `finalize()` matches incremental `recompute_chain` for the same domain+payloads.
    #[test]
    fn rolling_chain_matches_recompute_chain() {
        let domain = "match-test";
        let payloads: &[&[u8]] = &[b"ev-0", b"ev-1", b"ev-2"];

        let expected = recompute_chain(domain, payloads.iter().copied());

        let mut chain = RollingChain::new(domain);
        for p in payloads {
            chain.push(p);
        }
        assert_eq!(chain.finalize(), expected);
    }

    /// Same domain + same payloads always produce the same final hash.
    #[test]
    fn rolling_chain_is_deterministic() {
        let domain = "determ-chain";
        let payloads: &[&[u8]] = &[b"alpha", b"beta", b"gamma"];

        let hash_a = {
            let mut c = RollingChain::new(domain);
            for p in payloads {
                c.push(p);
            }
            c.finalize()
        };

        let hash_b = {
            let mut c = RollingChain::new(domain);
            for p in payloads {
                c.push(p);
            }
            c.finalize()
        };

        assert_eq!(hash_a, hash_b, "same inputs must produce identical chain hash");
        assert_eq!(hash_a.len(), 64, "chain hash must be 64 hex chars");
    }

    /// Mutating a single payload changes the final hash (tamper detection).
    #[test]
    fn rolling_chain_detects_single_event_tamper() {
        let domain = "tamper-chain";
        let honest: &[&[u8]] = &[b"event-a", b"event-b", b"event-c"];
        let tampered: &[&[u8]] = &[b"event-a", b"EVENT-B", b"event-c"];

        let h_honest = {
            let mut c = RollingChain::new(domain);
            for p in honest {
                c.push(p);
            }
            c.finalize()
        };

        let h_tampered = {
            let mut c = RollingChain::new(domain);
            for p in tampered {
                c.push(p);
            }
            c.finalize()
        };

        assert_ne!(h_honest, h_tampered, "tampered chain must differ from honest chain");
    }

    /// Inserting an extra event changes the final hash.
    #[test]
    fn rolling_chain_detects_extra_event() {
        let domain = "extra-event";
        let short: &[&[u8]] = &[b"x", b"y"];
        let long: &[&[u8]] = &[b"x", b"y", b"z"];

        let build = |payloads: &[&[u8]]| {
            let mut c = RollingChain::new(domain);
            for p in payloads {
                c.push(p);
            }
            c.finalize()
        };

        assert_ne!(build(short), build(long), "extra event must change chain hash");
    }

    /// Domain isolation: same payloads in different domains produce different hashes.
    #[test]
    fn rolling_chain_domain_isolation() {
        let payloads: &[&[u8]] = &[b"shared"];
        let h_a = {
            let mut c = RollingChain::new("domain-a");
            for p in payloads {
                c.push(p);
            }
            c.finalize()
        };
        let h_b = {
            let mut c = RollingChain::new("domain-b");
            for p in payloads {
                c.push(p);
            }
            c.finalize()
        };
        assert_ne!(h_a, h_b);
    }

    /// A large number of events still produces a valid 64-char digest.
    #[test]
    fn rolling_chain_many_events_produces_valid_digest() {
        let mut chain = RollingChain::new("large-chain");
        for i in 0u32..1000 {
            chain.push(i.to_le_bytes().as_ref());
        }
        let hash = chain.finalize();
        assert_eq!(hash.len(), 64);
        assert!(is_valid_digest(&hash));
    }

    // -----------------------------------------------------------------------
    // RollingHash (streaming hasher)
    // -----------------------------------------------------------------------

    /// Streaming in chunks produces the same result as a single one-shot hash.
    #[test]
    fn rolling_hash_streams_match_one_shot() {
        let data = b"The quick brown fox jumps over the lazy dog";
        let one_shot = content_address(data);

        let mut hasher = RollingHash::new();
        for chunk in data.chunks(5) {
            hasher.update(chunk);
        }
        assert_eq!(hasher.finalize(), one_shot);
    }

    /// `Default::default()` and `new()` produce the same empty hasher.
    #[test]
    fn rolling_hash_default_equals_new() {
        let data = b"test data";
        let mut a = RollingHash::new();
        let mut b = RollingHash::default();
        a.update(data);
        b.update(data);
        assert_eq!(a.finalize(), b.finalize());
    }

    /// Empty RollingHash finalize matches content_address(b"").
    #[test]
    fn rolling_hash_empty_matches_empty_content_address() {
        let empty_hash = content_address(b"");
        let hasher = RollingHash::new();
        assert_eq!(hasher.finalize(), empty_hash);
    }

    /// Single-byte update.
    #[test]
    fn rolling_hash_single_byte() {
        let data = b"x";
        let expected = content_address(data);
        let mut hasher = RollingHash::new();
        hasher.update(data);
        assert_eq!(hasher.finalize(), expected);
    }
}

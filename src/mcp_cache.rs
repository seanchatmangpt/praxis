//! MCP tool-result cache (feature `mcp` only).
//!
//! Ported from `template-mcp/src/cache.rs` (itself modeled on
//! `bcinr-mcp`'s `CapabilityCache`/`CapabilityCacheKey`), specialized to
//! this crate's naming (`ToolResultCache`/`ToolCacheKey`) so future
//! generated projects and this server share one proven idiom: get-before,
//! insert-after, cache only deterministic output.
//!
//! Keys a cached tool response on `(tool_name, BLAKE3(canonical_input))` —
//! the two fields every tool call always has a real referent for — plus an
//! optional richer "law" shape ([`ToolCacheKey`]) for dimensions that vary
//! per tool (a policy/capability-version digest, or — for tools that read a
//! mutable store — an environment digest standing in for "the store hasn't
//! changed since this was cached").
//!
//! Only wrap tools that are pure functions of their input with no
//! observable external mutable state (or that have been given an
//! `environment_digest` capturing that state). A side-effecting or
//! non-deterministic tool must never be cached this way — see
//! `mcp_lawobject_server.rs`'s cache-policy doc comment for which tools
//! qualify and why.

use moka::future::Cache;
use std::time::Duration;

/// Shared cache: key is the caller-built composite key string (see
/// [`ToolCacheKey::to_key_string`]), value is the tool's serialized JSON
/// response text.
#[derive(Clone)]
pub struct ToolResultCache {
    inner: Cache<String, String>,
}

impl ToolResultCache {
    /// `max_capacity` bounds the number of cached entries; `ttl` bounds how
    /// long an entry is considered valid. Pick a TTL appropriate to how the
    /// underlying data can change — there is no free lunch: caching pure
    /// tool output is only safe as long as "same input" really does mean
    /// "same output" for as long as the entry lives.
    #[must_use]
    pub fn new(max_capacity: u64, ttl: Duration) -> Self {
        Self { inner: Cache::builder().max_capacity(max_capacity).time_to_live(ttl).build() }
    }

    /// Build the plain two-field cache key for `tool` given its
    /// already-serialized canonical input bytes (e.g.
    /// `serde_json::to_vec(&parsed_value)`). Callers must serialize inputs
    /// canonically (parse-then-reserialize, not raw request bytes) or
    /// whitespace-different but logically identical JSON will miss.
    #[must_use]
    pub fn key(tool: &str, canonical_input: &[u8]) -> String {
        format!("{tool}:{}", blake3::hash(canonical_input).to_hex())
    }

    /// Look up a cached response by key.
    pub async fn get(&self, key: &str) -> Option<String> {
        self.inner.get(key).await
    }

    /// Insert (or overwrite) a cached response.
    pub async fn insert(&self, key: String, value: String) {
        self.inner.insert(key, value).await;
    }

    /// Drop every cached entry.
    pub fn invalidate_all(&self) {
        self.inner.invalidate_all();
    }
}

impl Default for ToolResultCache {
    /// Defaults to 10k entries with a 5 minute TTL. Adjust per server.
    fn default() -> Self {
        Self::new(10_000, Duration::from_secs(5 * 60))
    }
}

/// Extended ("law object") cache key shape: `tool`/`input_hash` are always
/// present; every other dimension is `Option` so a tool only pays for the
/// fields it actually has a real source of truth for.
///
/// **Load-bearing property**: a field only affects the resulting key string
/// when it is `Some`. A `None` field is *absent* from the key, not
/// present-as-empty — so two calls that both leave a field unset produce
/// the same key, while two calls that set a field to different values
/// produce different keys. See `law_field_matrix_tests` below for a proof
/// of this property; keep it passing if you add fields.
#[derive(Debug, Clone, Copy)]
pub struct ToolCacheKey<'a> {
    /// The MCP tool name (e.g. `"judge"`, `"admit"`).
    pub tool: &'a str,
    /// Hex BLAKE3 digest of the canonicalized (parse-then-reserialize) input JSON.
    pub input_hash: &'a str,
    /// This crate's `CARGO_PKG_VERSION`, distinguishing cache entries across
    /// binary rebuilds that might change tool semantics.
    pub capability_version: Option<&'a str>,
    /// Hex BLAKE3 digest of the policy/law name governing this call (e.g.
    /// `judge`'s `law` param, `admit`'s `policy` param).
    pub policy_digest: Option<&'a str>,
    /// Hex BLAKE3 digest of an authority identity, when one applies.
    pub authority_digest: Option<&'a str>,
    /// Hex BLAKE3 digest of external mutable state the tool's output
    /// depends on (e.g. a receipt ledger's `last_chain_hash`), standing in
    /// for "the environment hasn't changed since this was cached".
    pub environment_digest: Option<&'a str>,
    /// A replay/validation mode discriminator, when a tool has more than
    /// one deterministic mode over the same input.
    pub replay_mode: Option<&'a str>,
}

impl<'a> ToolCacheKey<'a> {
    /// Render this key to its composite cache-key string. Field order is
    /// fixed and stable; `None` fields contribute nothing (not even a
    /// placeholder marker) to the output.
    #[must_use]
    pub fn to_key_string(&self) -> String {
        let mut s = format!("{}:{}", self.tool, self.input_hash);
        if let Some(v) = self.capability_version {
            s.push_str(&format!(":cv={v}"));
        }
        if let Some(v) = self.policy_digest {
            s.push_str(&format!(":pd={v}"));
        }
        if let Some(v) = self.authority_digest {
            s.push_str(&format!(":ad={v}"));
        }
        if let Some(v) = self.environment_digest {
            s.push_str(&format!(":ed={v}"));
        }
        if let Some(v) = self.replay_mode {
            s.push_str(&format!(":rm={v}"));
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_insert_roundtrip() {
        let cache = ToolResultCache::default();
        let key = ToolResultCache::key("my_tool", b"{\"a\":1}");
        assert!(cache.get(&key).await.is_none());
        cache.insert(key.clone(), "cached-value".into()).await;
        assert_eq!(cache.get(&key).await.as_deref(), Some("cached-value"));
    }

    #[test]
    fn key_is_deterministic_and_input_sensitive() {
        let k1 = ToolResultCache::key("my_tool", b"input-a");
        let k2 = ToolResultCache::key("my_tool", b"input-a");
        let k3 = ToolResultCache::key("my_tool", b"input-b");
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }

    mod law_field_matrix_tests {
        //! Proves the exact claim `ToolCacheKey`'s doc comment makes:
        //! `None` fields do not enforce law (identical keys regardless of
        //! what an unset field "could have been"), and `Some` fields do.
        use super::ToolCacheKey;

        fn base<'a>() -> ToolCacheKey<'a> {
            ToolCacheKey {
                tool: "example_tool",
                input_hash: "deadbeef",
                capability_version: None,
                policy_digest: None,
                authority_digest: None,
                environment_digest: None,
                replay_mode: None,
            }
        }

        #[test]
        fn none_fields_do_not_enforce_law_identical_keys_when_all_unset() {
            let a = base();
            let b = base();
            assert_eq!(a.to_key_string(), b.to_key_string());
        }

        #[test]
        fn tool_or_input_hash_change_always_changes_key() {
            let mut other_tool = base();
            other_tool.tool = "other_tool";
            assert_ne!(base().to_key_string(), other_tool.to_key_string());

            let mut other_hash = base();
            other_hash.input_hash = "cafebabe";
            assert_ne!(base().to_key_string(), other_hash.to_key_string());
        }

        macro_rules! optional_field_enforces_law {
            ($test_name:ident, $field:ident) => {
                #[test]
                fn $test_name() {
                    let unset = base();
                    let mut set_a = base();
                    set_a.$field = Some("v1");
                    let mut set_b = base();
                    set_b.$field = Some("v2");

                    assert_ne!(
                        unset.to_key_string(),
                        set_a.to_key_string(),
                        "setting {} must change the key relative to it being unset",
                        stringify!($field)
                    );
                    assert_ne!(
                        set_a.to_key_string(),
                        set_b.to_key_string(),
                        "two different Some values for {} must produce different keys",
                        stringify!($field)
                    );
                }
            };
        }

        optional_field_enforces_law!(capability_version_enforces_law, capability_version);
        optional_field_enforces_law!(policy_digest_enforces_law, policy_digest);
        optional_field_enforces_law!(authority_digest_enforces_law, authority_digest);
        optional_field_enforces_law!(environment_digest_enforces_law, environment_digest);
        optional_field_enforces_law!(replay_mode_enforces_law, replay_mode);

        #[test]
        fn all_fields_unset_matches_plain_two_field_key_semantics() {
            let full_key_all_none = base().to_key_string();
            let plain_key = format!("{}:{}", "example_tool", "deadbeef");
            assert_eq!(full_key_all_none, plain_key);
        }
    }
}

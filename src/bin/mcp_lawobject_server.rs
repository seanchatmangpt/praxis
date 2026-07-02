//! MCP server — exposes the praxis `LawObject` lifecycle as capability admission tools.
//!
//! This server wires the Law Object lifecycle (Raw → Validated → Admitted → Receipted)
//! into Claude Code as MCP+ capability tools, calling the exact same pure
//! `my_conforming_project::ops::*_payload` functions the CLI's `law`/`receipt`
//! verbs call — one shared wire schema, one source of truth, no drift
//! between the two entry points.
//!
//! The `CPhy` MCP+ layer asks:
//! - What capability is being requested?
//! - What authority applies?
//! - What resources are touched?
//! - What policy boundary exists?
//! - What receipt is required?
//! - Has this consequence already been admitted?
//!
//! Compile and run with the `mcp` feature:
//!
//! ```bash
//! cargo run --bin mcp_lawobject_server --features mcp
//! ```
//!
//! The server speaks the Model Context Protocol over stdio and exposes these tools:
//!
//! 1. **`inspect_obligation`** — describe what a JSON array of tagged obligations requires.
//! 2. **judge** — `ops::judge_payload`: Raw → Validated or `Andon::Halted`.
//! 3. **admit** — `ops::admit_payload`: Validated → Admitted or a denial.
//! 4. **receipt** — `ops::receipt_payload`: Admitted → Receipted, BLAKE3-chained.
//! 5. **`show_andon`** — query the Andon state embedded in any `LawObject`-shaped JSON.
//! 6. **promote** — `ops::promote_payload`: auditor-gated `BreedStanding` promotion.
//! 7. **`receipt_validate`** — `ops::receipt_validate_payload`: validate the persisted receipt ledger.
//! 8. **`receipt_replay`** — `ops::receipt_replay_payload`: POWL-replay the persisted receipt ledger.
//!
//! # Domain denial vs. tool error
//!
//! A *domain* denial — halted obligations, a prolog8 rejection, a missing
//! auditor — is a **successful** tool call whose JSON body carries
//! `"status": "denied"` (or `"verdict": "halted"`), matching the CLI's
//! "denial is data, not an error" convention. Only malformed input (bad
//! JSON, bad hex, an unreadable ledger directory) is an MCP tool error.
//!
//! # Follow-up (not yet wired)
//!
//! `plan_route`/`plan_solve` (over `bcinr-pddl` via the `plan` noun) are not
//! exposed here yet: `src/verbs/plan.rs`'s `route_payload`/`solve_payload`
//! live in the `verbs` module tree, which is compiled only into the `main`
//! binary crate, not into the library this server links against. Exposing
//! them requires promoting those pure functions into `my_conforming_project::ops`
//! first (mirroring what lane 8a already did for `law`/`receipt`) — tracked
//! as a follow-up, not blocking this lane.

#[cfg(feature = "mcp")]
mod server {
    use my_conforming_project::{
        mcp_cache::{ToolCacheKey, ToolResultCache},
        ops,
    };
    use praxis_core::{
        receipt_store::{ReceiptStore, DEFAULT_RECEIPTS_DIR},
        Andon, Obligation,
    };
    use rmcp::{
        handler::server::{router::tool::ToolRouter, wrapper::Parameters},
        model::CallToolResult,
        tool, tool_handler, tool_router, ServerHandler,
    };
    use schemars::JsonSchema;
    use serde::Deserialize;
    use serde_json::{json, Value};

    /// This crate's `Cargo.toml` version — the `capability_version` cache
    /// dimension, distinguishing entries across binary rebuilds that might
    /// change tool semantics.
    const CAPABILITY_VERSION: &str = env!("CARGO_PKG_VERSION");

    /// Shared server state: tool routing plus the tool-result cache.
    #[derive(Clone)]
    pub struct ServerState {
        tool_router: ToolRouter<Self>,
        cache: ToolResultCache,
    }

    impl Default for ServerState {
        fn default() -> Self {
            Self { tool_router: Self::tool_router(), cache: ToolResultCache::default() }
        }
    }

    #[tool_handler(router = self.tool_router)]
    impl ServerHandler for ServerState {}

    // =========================================================================
    // CACHE / SERIALIZATION HELPERS
    // =========================================================================

    /// Parse `input` as JSON and re-serialize it canonically (parse then
    /// re-emit) so whitespace-only-different requests hit the same cache
    /// key. Malformed input falls back to the raw bytes — a hard-error
    /// result is never cached anyway, so this only needs to be a stable key
    /// for the success/denial path.
    fn canonical_bytes(input: &str) -> Vec<u8> {
        match serde_json::from_str::<Value>(input) {
            Ok(value) => serde_json::to_vec(&value).unwrap_or_else(|_| input.as_bytes().to_vec()),
            Err(_) => input.as_bytes().to_vec(),
        }
    }

    /// Hex BLAKE3 digest of already-canonicalized bytes.
    fn input_hash_hex(bytes: &[u8]) -> String {
        blake3::hash(bytes).to_hex().to_string()
    }

    /// Hex BLAKE3 digest of a short string (a policy/law/auditor name).
    fn digest_hex(s: &str) -> String {
        blake3::hash(s.as_bytes()).to_hex().to_string()
    }

    /// Pretty-print a JSON value, never panicking.
    fn pretty(value: &Value) -> String {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    }

    fn success_from_text(text: String) -> CallToolResult {
        CallToolResult::success(vec![rmcp::model::Content::text(text)])
    }

    fn error_text(msg: impl Into<String>) -> CallToolResult {
        CallToolResult::error(vec![rmcp::model::Content::text(msg.into())])
    }

    // =========================================================================
    // PARAMETER TYPES
    // =========================================================================

    /// Parameters for inspecting obligations.
    ///
    /// `obligations_json` is a JSON array of tagged obligations — the same
    /// shape embedded in `judge`/`admit`/`receipt`'s `payload_json` under
    /// its `"obligations"` key: `[{"type": "precondition", "predicate_id":
    /// "...", "params_hash_hex": "..."}, {"type": "blocking_constraint",
    /// "reason": "..."}, {"type": "evidence_required", "evidence_type":
    /// "..."}]`.
    #[derive(Deserialize, JsonSchema)]
    pub struct InspectObligationParams {
        /// JSON array of tagged obligations to inspect (see struct doc for the wire schema).
        pub obligations_json: String,
    }

    /// Parameters for `judge`/`admit`/`receipt`: one JSON payload carrying
    /// the full `LawInput` wire schema (`value`, `obligations`, `evidence`,
    /// `satisfied_predicates`, and — for a prolog8 admission/kernel-query
    /// check — `atom`/`rule`/`catalog`/`query`/`facts`). This is
    /// deliberately the exact shape `my_conforming_project::ops` deserializes:
    /// the CLI (`law judge|admit|receipt`) and this MCP tool share one wire
    /// schema, so there is exactly one source of truth for what a payload
    /// means.
    #[derive(Deserialize, JsonSchema)]
    pub struct JudgeParams {
        /// The `LawInput` JSON payload (see struct doc for the full shape).
        pub payload_json: String,
        /// Policy name to judge against; defaults to `"default"`.
        #[serde(default)]
        pub law: Option<String>,
    }

    /// Parameters for `admit`. Same `payload_json` shape as [`JudgeParams`].
    #[derive(Deserialize, JsonSchema)]
    pub struct AdmitParams {
        /// The `LawInput` JSON payload — same shape as `judge`'s `payload_json`.
        pub payload_json: String,
        /// Admission policy name; defaults to `"default"`.
        #[serde(default)]
        pub policy: Option<String>,
    }

    /// Parameters for `receipt`. Same `payload_json` shape as
    /// [`JudgeParams`], additionally read for `prev_chain_hash` (64-hex
    /// chars), `ts_ns`, `instruction_id`, `activity_idx`, `node_kind`.
    #[derive(Deserialize, JsonSchema)]
    pub struct ReceiptParams {
        /// The `LawInput`+receipt-fields JSON payload (see struct doc).
        pub payload_json: String,
    }

    /// Parameters for querying Andon state.
    #[derive(Deserialize, JsonSchema)]
    pub struct ShowAndonParams {
        /// JSON representation of any LawObject-shaped value carrying an `"andon"` field.
        pub lawobject_json: String,
    }

    /// Parameters for `promote`.
    #[derive(Deserialize, JsonSchema)]
    pub struct PromoteParams {
        /// JSON payload carrying `{"standing": "<BreedStanding registry name>"}`.
        pub payload_json: String,
        /// Auditor name endorsing the promotion; required for promotions to
        /// `Replayable` or `Certified`.
        #[serde(default)]
        pub auditor: Option<String>,
    }

    /// Parameters for `receipt_validate`/`receipt_replay`.
    #[derive(Deserialize, JsonSchema)]
    pub struct ReceiptDirParams {
        /// Receipts ledger directory; defaults to `"receipts"`. Unlike the
        /// CLI's `receipt` noun, this tool does not consult `PraxisConfig`
        /// (no config wiring in this server yet) — a follow-up.
        #[serde(default)]
        pub dir: Option<String>,
    }

    // =========================================================================
    // INTERNAL HELPERS (show_andon only — everything else goes through `ops`)
    // =========================================================================

    /// Serialize an obligation list to JSON for display.
    fn format_obligations(obligations: &[Obligation]) -> Value {
        Value::Array(
            obligations
                .iter()
                .map(|o| match o {
                    Obligation::Precondition { predicate_id, params_hash } => {
                        json!({
                            "type": "Precondition",
                            "predicate_id": predicate_id,
                            "params_hash": hex::encode(params_hash),
                        })
                    }
                    Obligation::BlockingConstraint { reason } => {
                        json!({
                            "type": "BlockingConstraint",
                            "reason": reason,
                        })
                    }
                    Obligation::EvidenceRequired { evidence_type } => {
                        json!({
                            "type": "EvidenceRequired",
                            "evidence_type": evidence_type,
                        })
                    }
                })
                .collect::<Vec<_>>(),
        )
    }

    /// Serialize Andon state to JSON for display.
    fn format_andon(andon: &Andon) -> Value {
        match andon {
            Andon::Green => {
                json!({
                    "status": "Green",
                    "message": "All obligations satisfied; proceed",
                })
            }
            Andon::Halted { unmet, refusals, at } => {
                let categories: Vec<Value> =
                    refusals.iter().map(|r| json!(format!("{}", r.category()))).collect();
                json!({
                    "status": "Halted",
                    "unmet_obligations": format_obligations(unmet),
                    "refusals": serde_json::to_value(refusals).unwrap_or(Value::Null),
                    "refusal_categories": categories,
                    "halted_at_ms": at,
                })
            }
            Andon::Overridden { by, reason, at } => {
                json!({
                    "status": "Overridden",
                    "by": by,
                    "reason": reason,
                    "overridden_at_ms": at,
                })
            }
        }
    }

    // =========================================================================
    // TOOL HANDLERS (MCP tools)
    // =========================================================================

    #[tool_router]
    impl ServerState {
        /// Inspect obligations: describe what each obligation requires before judgment.
        ///
        /// Pure and deterministic — always cached.
        #[tool(
            description = "Inspect obligations: list what each requires before judgment. \
                          Input: obligations_json (JSON array of tagged obligations, same shape \
                          embedded in judge/admit/receipt's payload_json.obligations). \
                          Output: JSON describing each obligation's requirements."
        )]
        async fn inspect_obligation(
            &self,
            params: Parameters<InspectObligationParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let payload = &params.0.obligations_json;
            let input_hash = input_hash_hex(&canonical_bytes(payload));
            let key = ToolCacheKey {
                tool: "inspect_obligation",
                input_hash: &input_hash,
                capability_version: Some(CAPABILITY_VERSION),
                policy_digest: None,
                authority_digest: None,
                environment_digest: None,
                replay_mode: None,
            }
            .to_key_string();

            if let Some(cached) = self.cache.get(&key).await {
                return Ok(success_from_text(cached));
            }
            match ops::inspect_obligations_payload(payload) {
                Ok(value) => {
                    let text = pretty(&value);
                    self.cache.insert(key, text.clone()).await;
                    Ok(success_from_text(text))
                }
                Err(e) => Ok(error_text(e)),
            }
        }

        /// Judge a raw `LawObject` payload: Raw → Validated or `Andon::Halted`.
        ///
        /// Calls `ops::judge_payload` directly — the same pipeline the CLI's
        /// `law judge` verb runs. A domain denial (halted obligations, a
        /// prolog8/kernel refusal) is a successful call with
        /// `"verdict": "halted"`; only malformed `payload_json` is a tool error.
        #[tool(
            description = "Judge a raw LawObject payload: evaluate obligations (and, if present, \
                          prolog8 atom/rule/query checks) and transition to Validated or Halted. \
                          Input: payload_json (LawInput JSON: value, obligations, evidence, \
                          satisfied_predicates, atom/rule/catalog/query/facts), law (policy name, \
                          default \"default\"). Output: {status, law, verdict, andon|unmet/refusals, \
                          prolog8, prolog8_query}. A domain halt is a successful call, not an error."
        )]
        async fn judge(
            &self,
            params: Parameters<JudgeParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let payload = &params.0.payload_json;
            let law = params.0.law.as_deref().unwrap_or("default");
            let input_hash = input_hash_hex(&canonical_bytes(payload));
            let policy_digest = digest_hex(law);
            let key = ToolCacheKey {
                tool: "judge",
                input_hash: &input_hash,
                capability_version: Some(CAPABILITY_VERSION),
                policy_digest: Some(&policy_digest),
                authority_digest: None,
                environment_digest: None,
                replay_mode: None,
            }
            .to_key_string();

            if let Some(cached) = self.cache.get(&key).await {
                return Ok(success_from_text(cached));
            }
            match ops::judge_payload(payload, law) {
                Ok(value) => {
                    let text = pretty(&value);
                    self.cache.insert(key, text.clone()).await;
                    Ok(success_from_text(text))
                }
                Err(e) => Ok(error_text(e)),
            }
        }

        /// Admit a judged `LawObject` payload: Validated → Admitted or a denial.
        ///
        /// Calls `ops::admit_payload` (runs judge, then admit) — the same
        /// pipeline as the CLI's `law admit` verb. A domain denial is a
        /// successful call with `"status": "denied"`.
        #[tool(
            description = "Admit a LawObject payload: run judge then, if validated, admit. \
                          Input: payload_json (LawInput JSON, same shape as judge), policy \
                          (admission policy name, default \"default\"). \
                          Output: {status: \"admitted\"|\"denied\", ...}. A domain denial is a \
                          successful call, not an error."
        )]
        async fn admit(
            &self,
            params: Parameters<AdmitParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let payload = &params.0.payload_json;
            let policy = params.0.policy.as_deref().unwrap_or("default");
            let input_hash = input_hash_hex(&canonical_bytes(payload));
            let policy_digest = digest_hex(policy);
            let key = ToolCacheKey {
                tool: "admit",
                input_hash: &input_hash,
                capability_version: Some(CAPABILITY_VERSION),
                policy_digest: Some(&policy_digest),
                authority_digest: None,
                environment_digest: None,
                replay_mode: None,
            }
            .to_key_string();

            if let Some(cached) = self.cache.get(&key).await {
                return Ok(success_from_text(cached));
            }
            match ops::admit_payload(payload, policy) {
                Ok(value) => {
                    let text = pretty(&value);
                    self.cache.insert(key, text.clone()).await;
                    Ok(success_from_text(text))
                }
                Err(e) => Ok(error_text(e)),
            }
        }

        /// Receipt a judged+admitted `LawObject` payload: compute a BLAKE3
        /// chain hash and (with `law-signed`) a signature.
        ///
        /// Calls `ops::receipt_payload`. Cached **only** when `payload_json`
        /// carries an explicit `ts_ns` — without it, `receipt_payload`
        /// defaults `ts_ns` to `now()`, making the result nondeterministic;
        /// caching that would silently return a stale timestamp on repeat
        /// calls.
        #[tool(
            description = "Receipt a LawObject payload: run judge -> admit -> receipt, computing a \
                          BLAKE3 chain hash bound to prev_chain_hash + the payload. \
                          Input: payload_json (LawInput JSON, optionally with prev_chain_hash, \
                          ts_ns, instruction_id, activity_idx, node_kind). \
                          Output: {status: \"receipted\"|\"denied\", chain_hash, ...}. Supplying an \
                          explicit ts_ns makes the result deterministic (and cacheable); omitting it \
                          defaults to now() and is never cached."
        )]
        async fn receipt(
            &self,
            params: Parameters<ReceiptParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let payload = &params.0.payload_json;
            let has_ts_ns = serde_json::from_str::<Value>(payload)
                .ok()
                .is_some_and(|v| v.get("ts_ns").is_some());
            let input_hash = input_hash_hex(&canonical_bytes(payload));
            let key = ToolCacheKey {
                tool: "receipt",
                input_hash: &input_hash,
                capability_version: Some(CAPABILITY_VERSION),
                policy_digest: None,
                authority_digest: None,
                environment_digest: None,
                replay_mode: None,
            }
            .to_key_string();

            if has_ts_ns {
                if let Some(cached) = self.cache.get(&key).await {
                    return Ok(success_from_text(cached));
                }
            }
            match ops::receipt_payload(payload) {
                Ok(value) => {
                    let text = pretty(&value);
                    if has_ts_ns {
                        self.cache.insert(key, text.clone()).await;
                    }
                    Ok(success_from_text(text))
                }
                Err(e) => Ok(error_text(e)),
            }
        }

        /// Show Andon state: query the halt/override status embedded in a LawObject-shaped JSON.
        #[tool(
            description = "Show Andon state: extract and describe the halt/override status from any \
                          LawObject-shaped JSON. Input: lawobject_json (JSON object with an \"andon\" \
                          field). Output: JSON describing the Andon state (Green/Halted/Overridden)."
        )]
        async fn show_andon(
            &self,
            params: Parameters<ShowAndonParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let payload = &params.0.lawobject_json;
            let input_hash = input_hash_hex(&canonical_bytes(payload));
            let key = ToolCacheKey {
                tool: "show_andon",
                input_hash: &input_hash,
                capability_version: Some(CAPABILITY_VERSION),
                policy_digest: None,
                authority_digest: None,
                environment_digest: None,
                replay_mode: None,
            }
            .to_key_string();

            if let Some(cached) = self.cache.get(&key).await {
                return Ok(success_from_text(cached));
            }

            match serde_json::from_str::<Value>(payload) {
                Ok(obj) => {
                    let Some(andon_value) = obj.get("andon") else {
                        return Ok(error_text("No 'andon' field found in LawObject JSON"));
                    };
                    let result = match serde_json::from_value::<Andon>(andon_value.clone()) {
                        Ok(andon) => json!({ "andon": format_andon(&andon) }),
                        Err(_) => json!({ "andon": andon_value }),
                    };
                    let text = pretty(&result);
                    self.cache.insert(key, text.clone()).await;
                    Ok(success_from_text(text))
                }
                Err(e) => Ok(error_text(format!("Failed to parse LawObject: {e}"))),
            }
        }

        /// Promote a law object's `BreedStanding` via `ops::promote_payload`.
        ///
        /// Promotions to `Replayable`/`Certified` require a non-empty
        /// `auditor`; a missing auditor is a domain denial (successful call
        /// with `"status": "denied"`), not a tool error.
        #[tool(
            description = "Promote a law object's BreedStanding rung. \
                          Input: payload_json ({\"standing\": \"<registry name>\"}), auditor \
                          (required for promotions to REPLAYABLE/CERTIFIED). \
                          Output: {status: \"promoted\"|\"denied\", from, to, ...}."
        )]
        async fn promote(
            &self,
            params: Parameters<PromoteParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let payload = &params.0.payload_json;
            let auditor = params.0.auditor.unwrap_or_default();
            let input_hash = input_hash_hex(&canonical_bytes(payload));
            let authority_digest = if auditor.is_empty() { None } else { Some(digest_hex(&auditor)) };
            let key = ToolCacheKey {
                tool: "promote",
                input_hash: &input_hash,
                capability_version: Some(CAPABILITY_VERSION),
                policy_digest: None,
                authority_digest: authority_digest.as_deref(),
                environment_digest: None,
                replay_mode: None,
            }
            .to_key_string();

            if let Some(cached) = self.cache.get(&key).await {
                return Ok(success_from_text(cached));
            }
            match ops::promote_payload(payload, &auditor) {
                Ok(value) => {
                    let text = pretty(&value);
                    self.cache.insert(key, text.clone()).await;
                    Ok(success_from_text(text))
                }
                Err(e) => Ok(error_text(e)),
            }
        }

        /// Validate the persisted receipt ledger (schema, chain-tamper
        /// recompute, linkage, monotonicity, POWL replay conformance).
        ///
        /// Reads a mutable, append-only JSONL ledger, so it is cached with
        /// an `environment_digest` of the ledger's current head chain hash
        /// (`ReceiptStore::last_chain_hash`): a cached result is only ever
        /// reused while the ledger hasn't grown since it was produced.
        #[tool(
            description = "Validate the persisted receipt ledger: schema, chain-tamper recompute, \
                          chain linkage, monotonicity, and POWL replay conformance. On success, also \
                          archives the ledger to data/validated_receipts/. \
                          Input: dir (receipts ledger directory, default \"receipts\"). \
                          Output: {verdict: {ok, records_checked, ...}}."
        )]
        async fn receipt_validate(
            &self,
            params: Parameters<ReceiptDirParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let dir = params.0.dir.unwrap_or_else(|| DEFAULT_RECEIPTS_DIR.to_string());
            let store = match ReceiptStore::open(&dir) {
                Ok(s) => s,
                Err(e) => return Ok(error_text(format!("failed to open receipts dir {dir}: {e}"))),
            };
            let head_hash = match store.last_chain_hash() {
                Ok(h) => hex::encode(h),
                Err(e) => {
                    return Ok(error_text(format!("failed to read receipts ledger head: {e}")))
                }
            };
            let input_hash = input_hash_hex(dir.as_bytes());
            let key = ToolCacheKey {
                tool: "receipt_validate",
                input_hash: &input_hash,
                capability_version: Some(CAPABILITY_VERSION),
                policy_digest: None,
                authority_digest: None,
                environment_digest: Some(&head_hash),
                replay_mode: None,
            }
            .to_key_string();

            if let Some(cached) = self.cache.get(&key).await {
                return Ok(success_from_text(cached));
            }
            match ops::receipt_validate_payload(&dir) {
                Ok(value) => {
                    let text = pretty(&value);
                    self.cache.insert(key, text.clone()).await;
                    Ok(success_from_text(text))
                }
                Err(e) => Ok(error_text(e)),
            }
        }

        /// Replay the persisted receipt ledger through the fixed
        /// judge->admit->receipt POWL lifecycle model and report per-receipt
        /// conformance metrics.
        ///
        /// Same `environment_digest`-keyed caching as `receipt_validate`,
        /// for the same reason: the ledger it reads is mutable.
        #[tool(
            description = "Replay every receipt in the ledger through the fixed POWL \
                          judge->admit->receipt lifecycle model and report per-receipt \
                          fitness/precision conformance metrics. \
                          Input: dir (receipts ledger directory, default \"receipts\"). \
                          Output: {records_replayed, results: [{instruction_id, chain_hash, fitness, \
                          precision}]}."
        )]
        async fn receipt_replay(
            &self,
            params: Parameters<ReceiptDirParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let dir = params.0.dir.unwrap_or_else(|| DEFAULT_RECEIPTS_DIR.to_string());
            let store = match ReceiptStore::open(&dir) {
                Ok(s) => s,
                Err(e) => return Ok(error_text(format!("failed to open receipts dir {dir}: {e}"))),
            };
            let head_hash = match store.last_chain_hash() {
                Ok(h) => hex::encode(h),
                Err(e) => {
                    return Ok(error_text(format!("failed to read receipts ledger head: {e}")))
                }
            };
            let input_hash = input_hash_hex(dir.as_bytes());
            let key = ToolCacheKey {
                tool: "receipt_replay",
                input_hash: &input_hash,
                capability_version: Some(CAPABILITY_VERSION),
                policy_digest: None,
                authority_digest: None,
                environment_digest: Some(&head_hash),
                replay_mode: Some("judge_admit_receipt_seq"),
            }
            .to_key_string();

            if let Some(cached) = self.cache.get(&key).await {
                return Ok(success_from_text(cached));
            }
            match ops::receipt_replay_payload(&dir) {
                Ok(value) => {
                    let text = pretty(&value);
                    self.cache.insert(key, text.clone()).await;
                    Ok(success_from_text(text))
                }
                Err(e) => Ok(error_text(e)),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn text_of(result: &CallToolResult) -> String {
            result
                .content
                .iter()
                .find_map(|c| c.as_text().map(|t| t.text.clone()))
                .unwrap_or_default()
        }

        /// Fixed 64-hex-char (32-byte) ed25519 seed used only by these tests.
        /// Not security-sensitive: under `--features law-signed` (part of
        /// `all-features`) the receipt paths sign fail-closed, so every test
        /// that produces a receipt or seeds a ledger needs a deterministic
        /// `PRAXIS_SIGNING_KEY` — same house pattern as `ops::tests`.
        #[cfg(feature = "law-signed")]
        const MCP_TEST_SIGNING_KEY_HEX: &str =
            "3c9d1e2f4a5b6c7d8e9fa0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9cadb";

        /// Set `PRAXIS_SIGNING_KEY` for the duration of the returned guard.
        /// `std::env` is process-global, so signing-path tests serialize on
        /// this lock rather than racing each other's env mutation.
        #[cfg(feature = "law-signed")]
        fn with_mcp_test_signing_key() -> std::sync::MutexGuard<'static, ()> {
            use std::sync::{Mutex, MutexGuard, OnceLock};
            fn env_lock() -> MutexGuard<'static, ()> {
                static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
                LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner())
            }
            let guard = env_lock();
            std::env::set_var("PRAXIS_SIGNING_KEY", MCP_TEST_SIGNING_KEY_HEX);
            guard
        }

        /// Remove the wall-clock `andon.Halted.at` field before comparing a
        /// tool result against a direct `ops::*` call: the two invocations
        /// legitimately run milliseconds apart, and `at` is documented as
        /// diagnostic, not part of the admission decision.
        fn strip_halted_at(v: &mut Value) {
            if let Some(halted) = v.get_mut("andon").and_then(|a| a.get_mut("Halted")) {
                if let Some(map) = halted.as_object_mut() {
                    map.remove("at");
                }
            }
        }

        // ── judge/admit/receipt match ops::*_payload directly ──────────────

        #[tokio::test]
        async fn judge_tool_matches_ops_judge_payload() {
            let state = ServerState::default();
            let payload = r#"{"value":{"id":1}}"#.to_string();
            let params = Parameters(JudgeParams { payload_json: payload.clone(), law: None });
            let tool_result = state.judge(params).await.expect("judge should not error");
            assert!(!tool_result.is_error.unwrap_or(false));

            let tool_json: Value =
                serde_json::from_str(&text_of(&tool_result)).expect("tool output is JSON");
            let direct_json = ops::judge_payload(&payload, "default").expect("direct judge_payload");
            assert_eq!(tool_json, direct_json);
        }

        #[tokio::test]
        async fn admit_tool_matches_ops_admit_payload_denial_shape() {
            let state = ServerState::default();
            let payload = r#"{"value":{"id":1},"obligations":[{"type":"blocking_constraint","reason":"stop"}]}"#.to_string();
            let params = Parameters(AdmitParams { payload_json: payload.clone(), policy: None });
            let tool_result = state.admit(params).await.expect("admit should not error");
            assert!(!tool_result.is_error.unwrap_or(false));

            let mut tool_json: Value =
                serde_json::from_str(&text_of(&tool_result)).expect("tool output is JSON");
            assert_eq!(tool_json["status"], json!("denied"));
            let mut direct_json =
                ops::admit_payload(&payload, "default").expect("direct admit_payload");
            // The two calls run milliseconds apart; `at` is a wall-clock
            // diagnostic, not part of the denial shape under test.
            strip_halted_at(&mut tool_json);
            strip_halted_at(&mut direct_json);
            assert_eq!(tool_json, direct_json);
        }

        #[tokio::test]
        async fn receipt_tool_matches_ops_receipt_payload_with_explicit_ts_ns() {
            #[cfg(feature = "law-signed")]
            let _guard = with_mcp_test_signing_key();
            let state = ServerState::default();
            let payload = r#"{"value":{"id":1},"ts_ns":42}"#.to_string();
            let params = Parameters(ReceiptParams { payload_json: payload.clone() });
            let tool_result = state.receipt(params).await.expect("receipt should not error");
            assert!(!tool_result.is_error.unwrap_or(false));

            let tool_json: Value =
                serde_json::from_str(&text_of(&tool_result)).expect("tool output is JSON");
            let direct_json = ops::receipt_payload(&payload).expect("direct receipt_payload");
            assert_eq!(tool_json["chain_hash"], direct_json["chain_hash"]);
            assert_eq!(tool_json["ts_ns"], json!(42));
        }

        #[tokio::test]
        async fn inspect_obligation_tool_matches_ops_directly() {
            let state = ServerState::default();
            let obligations =
                json!([{"type": "blocking_constraint", "reason": "stop"}]).to_string();
            let params =
                Parameters(InspectObligationParams { obligations_json: obligations.clone() });
            let tool_result =
                state.inspect_obligation(params).await.expect("inspect_obligation should not error");
            let tool_json: Value =
                serde_json::from_str(&text_of(&tool_result)).expect("tool output is JSON");
            let direct_json =
                ops::inspect_obligations_payload(&obligations).expect("direct inspect");
            assert_eq!(tool_json, direct_json);
        }

        #[tokio::test]
        async fn promote_tool_matches_ops_promote_payload() {
            let state = ServerState::default();
            let payload = r#"{"standing":"NAMED"}"#.to_string();
            let params =
                Parameters(PromoteParams { payload_json: payload.clone(), auditor: None });
            let tool_result = state.promote(params).await.expect("promote should not error");
            let tool_json: Value =
                serde_json::from_str(&text_of(&tool_result)).expect("tool output is JSON");
            let direct_json = ops::promote_payload(&payload, "").expect("direct promote_payload");
            assert_eq!(tool_json, direct_json);
        }

        #[tokio::test]
        async fn malformed_judge_payload_is_tool_error() {
            let state = ServerState::default();
            let params =
                Parameters(JudgeParams { payload_json: "not json".to_string(), law: None });
            let tool_result = state.judge(params).await.expect("call itself should not error");
            assert!(tool_result.is_error.unwrap_or(false));
        }

        // ── cache hit/skip behavior ─────────────────────────────────────────

        #[tokio::test]
        async fn identical_judge_calls_hit_cache_and_return_same_json() {
            let state = ServerState::default();
            let payload = r#"{"value":{"id":7}}"#.to_string();
            let make_params =
                || Parameters(JudgeParams { payload_json: payload.clone(), law: None });

            let first = state.judge(make_params()).await.expect("first call");
            let second = state.judge(make_params()).await.expect("second call");
            assert_eq!(text_of(&first), text_of(&second));

            // Same input, canonicalized differently (whitespace) — must still hit.
            let spaced = Parameters(JudgeParams {
                payload_json: r#"{ "value" : { "id" : 7 } }"#.to_string(),
                law: None,
            });
            let third = state.judge(spaced).await.expect("third call");
            assert_eq!(text_of(&first), text_of(&third));
        }

        #[tokio::test]
        async fn receipt_without_ts_ns_is_never_cached() {
            #[cfg(feature = "law-signed")]
            let _guard = with_mcp_test_signing_key();
            let state = ServerState::default();
            let payload = r#"{"value":{"id":9}}"#.to_string();
            let make_params = || Parameters(ReceiptParams { payload_json: payload.clone() });

            let first = state.receipt(make_params()).await.expect("first call");
            let second = state.receipt(make_params()).await.expect("second call");
            let first_json: Value = serde_json::from_str(&text_of(&first)).expect("json");
            let second_json: Value = serde_json::from_str(&text_of(&second)).expect("json");
            // Without an explicit ts_ns, each call defaults to a fresh now() —
            // the chain hash (which folds in ts_ns via ReceiptMeta) must not
            // be pinned to a stale cached value across calls.
            assert_ne!(first_json["ts_ns"], second_json["ts_ns"]);
        }

        #[tokio::test]
        async fn receipt_with_ts_ns_is_cached_and_deterministic() {
            #[cfg(feature = "law-signed")]
            let _guard = with_mcp_test_signing_key();
            let state = ServerState::default();
            let payload = r#"{"value":{"id":11},"ts_ns":123}"#.to_string();
            let make_params = || Parameters(ReceiptParams { payload_json: payload.clone() });

            let first = state.receipt(make_params()).await.expect("first call");
            let second = state.receipt(make_params()).await.expect("second call");
            assert_eq!(text_of(&first), text_of(&second));
        }

        #[tokio::test]
        async fn show_andon_round_trips_green_and_halted() {
            let state = ServerState::default();
            let green_lawobject = json!({"andon": {"status": "Green"}}).to_string();
            let params = Parameters(ShowAndonParams { lawobject_json: green_lawobject });
            let result = state.show_andon(params).await.expect("show_andon should not error");
            let json: Value = serde_json::from_str(&text_of(&result)).expect("json");
            assert_eq!(json["andon"]["status"], json!("Green"));
        }

        #[tokio::test]
        async fn receipt_validate_and_replay_on_fresh_ledger() {
            #[cfg(feature = "law-signed")]
            let _guard = with_mcp_test_signing_key();
            let state = ServerState::default();
            let dir = std::env::temp_dir().join(format!(
                "praxis-mcp-server-tests-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ));
            let dir_str = dir.to_string_lossy().to_string();

            // Seed the ledger via the same `ops::receipt_issue_payload` the
            // `receipt issue` CLI verb calls, so this test exercises the
            // tool against a real (if empty-then-one-entry) ledger rather
            // than a hand-built fixture.
            ops::receipt_issue_payload(r#"{"value":{"id":1}}"#, &dir_str).expect("seed ledger");

            let validate_params = Parameters(ReceiptDirParams { dir: Some(dir_str.clone()) });
            let validate_result = state
                .receipt_validate(validate_params)
                .await
                .expect("receipt_validate should not error");
            let validate_json: Value =
                serde_json::from_str(&text_of(&validate_result)).expect("json");
            assert_eq!(validate_json["verdict"]["ok"], json!(true));

            let replay_params = Parameters(ReceiptDirParams { dir: Some(dir_str.clone()) });
            let replay_result =
                state.receipt_replay(replay_params).await.expect("receipt_replay should not error");
            let replay_json: Value = serde_json::from_str(&text_of(&replay_result)).expect("json");
            assert_eq!(replay_json["records_replayed"], json!(1));

            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}

#[cfg(feature = "mcp")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rmcp::{transport::stdio, ServiceExt as _};
    server::ServerState::default().serve(stdio()).await?;
    Ok(())
}

#[cfg(not(feature = "mcp"))]
fn main() -> anyhow::Result<()> {
    eprintln!("mcp_lawobject_server requires the `mcp` Cargo feature. Rebuild with --features mcp");
    std::process::exit(1);
}

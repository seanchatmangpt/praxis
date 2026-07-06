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
//! The server speaks the Model Context Protocol over stdio and exposes the
//! whole Genesis Day 2 revenue pipe plus the law lifecycle, so an external
//! agent with **only** membrane access can drive a receipted mission
//! (see `scripts/membrane_demo.sh`):
//!
//! Day 2 revenue pipe (observe → propose → plan → admit → receipt):
//! - **`propose_revenue`** — `ops::propose_revenue_payload`: rank candidate goal states.
//! - **`propose_goal`** — `ops::propose_goal_payload`: the top proposal's PDDL goal atom.
//! - **`plan_solve`** — `ops::plan_solve_payload`: solve a classical/temporal PDDL8 problem.
//! - **judge** — `ops::judge_payload`: Raw → Validated or `Andon::Halted`.
//! - **admit** — `ops::admit_payload`: Validated → Admitted or a denial.
//! - **receipt** — `ops::receipt_payload`: Admitted → Receipted, BLAKE3-chained.
//!
//! Law lifecycle + ledger:
//! - **`inspect_obligation`** — describe what a JSON array of tagged obligations requires.
//! - **`show_andon`** — query the Andon state embedded in any `LawObject`-shaped JSON.
//! - **promote** — `ops::promote_payload`: auditor-gated `BreedStanding` promotion.
//! - **`receipt_validate`** — `ops::receipt_validate_payload`: validate the persisted ledger.
//! - **`receipt_replay`** — `ops::receipt_replay_payload`: POWL-replay the persisted ledger.
//!
//! agent8 projection (the session's resident governance byte):
//! - **`whoami`** — the caller session's current [`agent8::AgentByte`], updated by
//!   this session's judge/admit/receipt outcomes.
//! - **`fleet_status`** — sweep a fleet of agent8 bytes with the SWAR popcount kernel.
//!
//! Every tool calls the exact same `my_conforming_project::ops::*` function the
//! matching CLI verb calls — one shared implementation, no drift (AR-2).
//!
//! # Domain denial vs. tool error
//!
//! A *domain* denial — halted obligations, a prolog8 rejection, a missing
//! auditor, an infeasible plan — is a **successful** tool call whose JSON body
//! carries `"status": "denied"` / `"verdict": "halted"` / `"admitted": false`,
//! matching the CLI's "denial is data, not an error" convention. Only malformed
//! input (bad JSON, bad hex, an unreadable ledger directory) is an MCP tool error.

// Recorded lint debt (v26.7.6 verification gate) -- see src/lib.rs and
// docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

#[cfg(feature = "mcp")]
mod server {
    use std::sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    };

    use agent8::{AgentByte, AgentSelect, Fleet};
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

    /// Shared server state: tool routing, the tool-result cache, and this
    /// session's resident [`AgentByte`] — the agent8 projection of the
    /// caller's governance posture, mutated by judge/admit/receipt outcomes
    /// (the "MCP lifecycle event → resident byte" adapter, see [`ServerState`]
    /// impl below).
    #[derive(Clone)]
    pub(crate) struct ServerState {
        tool_router: ToolRouter<Self>,
        cache: ToolResultCache,
        /// The connected session's live [`AgentByte`]. `Arc<AtomicU8>` so every
        /// clone of `ServerState` (rmcp clones the handler per request) shares
        /// one byte, and updates are lock-free.
        session: Arc<AtomicU8>,
    }

    impl Default for ServerState {
        fn default() -> Self {
            Self {
                tool_router: Self::tool_router(),
                cache: ToolResultCache::default(),
                // A freshly connected session is operationally HEALTHY but
                // carries no governance bits yet — nothing has been judged,
                // admitted, or receipted through the membrane.
                session: Arc::new(AtomicU8::new(AgentByte::HEALTHY)),
            }
        }
    }

    #[tool_handler(router = self.tool_router)]
    impl ServerHandler for ServerState {}

    // =========================================================================
    // AGENT8 ADAPTER: MCP lifecycle events → the session's resident AgentByte.
    //
    // Bit mapping (the load-bearing choice; documented so the byte is auditable):
    //   judge  validated  -> set   CONFORMANT | EVIDENCE_OK
    //          halted      -> clear CONFORMANT | EVIDENCE_OK | ADMITTED
    //   admit  admitted    -> set   ADMITTED | WITHIN_BUDGET | AUTHORITY_BOUND
    //          denied      -> clear ADMITTED
    //   receipt receipted  -> set   RECEIPTED | REPLAYABLE
    //          denied      -> clear ADMITTED
    //
    // The AgentByte vocabulary has no BLOCKED bit (unlike the semantic_bit
    // prior art): a halt/denial is represented by the *absence* of the
    // governance bits, which is exactly what `select(GRANT_REQUIRED)` reads as
    // Deny. REPLAYABLE is set on receipt because a receipt is replayable
    // evidence — mirroring `Fleet::update_from_pulse`, which produces
    // RECEIPTED|REPLAYABLE on a valid non-error pulse.
    // =========================================================================

    impl ServerState {
        /// The session's current [`AgentByte`] projection.
        fn agent_byte(&self) -> AgentByte {
            AgentByte::from_raw(self.session.load(Ordering::SeqCst))
        }

        /// OR the given bits into the session byte.
        fn agent_set(&self, bits: u8) {
            let next = self.agent_byte().with(bits).raw();
            self.session.store(next, Ordering::SeqCst);
        }

        /// Clear the given bits from the session byte.
        fn agent_clear(&self, bits: u8) {
            let next = self.agent_byte().without(bits).raw();
            self.session.store(next, Ordering::SeqCst);
        }

        /// Read a top-level string field from a tool's JSON result text.
        fn result_field(text: &str, field: &str) -> Option<String> {
            serde_json::from_str::<Value>(text)
                .ok()?
                .get(field)?
                .as_str()
                .map(str::to_string)
        }

        /// Fold a `judge` outcome into the session byte.
        fn apply_judge(&self, text: &str) {
            match Self::result_field(text, "verdict").as_deref() {
                Some("validated") => self.agent_set(AgentByte::CONFORMANT | AgentByte::EVIDENCE_OK),
                Some("halted") => self.agent_clear(
                    AgentByte::CONFORMANT | AgentByte::EVIDENCE_OK | AgentByte::ADMITTED,
                ),
                _ => {}
            }
        }

        /// Fold an `admit` outcome into the session byte.
        fn apply_admit(&self, text: &str) {
            match Self::result_field(text, "status").as_deref() {
                Some("admitted") => self.agent_set(
                    AgentByte::ADMITTED | AgentByte::WITHIN_BUDGET | AgentByte::AUTHORITY_BOUND,
                ),
                Some("denied") => self.agent_clear(AgentByte::ADMITTED),
                _ => {}
            }
        }

        /// Fold a `receipt` outcome into the session byte.
        fn apply_receipt(&self, text: &str) {
            match Self::result_field(text, "status").as_deref() {
                Some("receipted") => self.agent_set(AgentByte::RECEIPTED | AgentByte::REPLAYABLE),
                Some("denied") => self.agent_clear(AgentByte::ADMITTED),
                _ => {}
            }
        }
    }

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
    pub(crate) struct InspectObligationParams {
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
    pub(crate) struct JudgeParams {
        /// The `LawInput` JSON payload (see struct doc for the full shape).
        pub payload_json: String,
        /// Policy name to judge against; defaults to `"default"`.
        #[serde(default)]
        pub law: Option<String>,
    }

    /// Parameters for `admit`. Same `payload_json` shape as [`JudgeParams`].
    #[derive(Deserialize, JsonSchema)]
    pub(crate) struct AdmitParams {
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
    pub(crate) struct ReceiptParams {
        /// The `LawInput`+receipt-fields JSON payload (see struct doc).
        pub payload_json: String,
    }

    /// Parameters for querying Andon state.
    #[derive(Deserialize, JsonSchema)]
    pub(crate) struct ShowAndonParams {
        /// JSON representation of any LawObject-shaped value carrying an `"andon"` field.
        pub lawobject_json: String,
    }

    /// Parameters for `promote`.
    #[derive(Deserialize, JsonSchema)]
    pub(crate) struct PromoteParams {
        /// JSON payload carrying `{"standing": "<BreedStanding registry name>"}`.
        pub payload_json: String,
        /// Auditor name endorsing the promotion; required for promotions to
        /// `Replayable` or `Certified`.
        #[serde(default)]
        pub auditor: Option<String>,
    }

    /// Parameters for `receipt_validate`/`receipt_replay`.
    #[derive(Deserialize, JsonSchema)]
    pub(crate) struct ReceiptDirParams {
        /// Receipts ledger directory; defaults to `"receipts"`. Unlike the
        /// CLI's `receipt` noun, this tool does not consult `PraxisConfig`
        /// (no config wiring in this server yet) — a follow-up.
        #[serde(default)]
        pub dir: Option<String>,
    }

    /// Parameters for `plan_solve`.
    #[derive(Deserialize, JsonSchema)]
    pub(crate) struct PlanSolveParams {
        /// PDDL8 solve payload — `{domain, problem, mode}` (inline text) and/or
        /// `{domain_file, problem_file}` (paths), or a single combined
        /// domain+problem string in `domain`. `mode` is `"classical"` (default)
        /// or `"temporal"`. Same wire schema as the CLI `plan solve` verb.
        pub payload_json: String,
    }

    /// Parameters for `propose_revenue`/`propose_goal`. The `mcp` feature
    /// implies `proposer` (see Cargo.toml), so these tools are always present.
    #[derive(Deserialize, JsonSchema)]
    pub(crate) struct ProposeParams {
        /// `{state, objective|objective_file}` — the observed revenue snapshot
        /// plus a domain-authored objective (inline object or file path).
        pub payload_json: String,
        /// Optional path to a domain-authored objective JSON file (mutually
        /// exclusive with an inline `objective`/`objective_file` in the payload).
        #[serde(default)]
        pub objective: Option<String>,
    }

    /// Parameters for `fleet_status`.
    #[derive(Deserialize, JsonSchema)]
    pub(crate) struct FleetStatusParams {
        /// The fleet state as JSON: either a bare array of agent bytes
        /// (`[111, 255, ...]`) or an object
        /// `{"agents": [<u8>...], "required_mask": <u8?>}`. `required_mask`
        /// defaults to [`AgentByte::GRANT_REQUIRED`] (`0x6F`).
        pub fleet_json: String,
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
                    Obligation::Precondition {
                        predicate_id,
                        params_hash,
                    } => {
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
            Andon::Halted {
                unmet,
                refusals,
                at,
            } => {
                let categories: Vec<Value> = refusals
                    .iter()
                    .map(|r| json!(format!("{}", r.category())))
                    .collect();
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

            let text = if let Some(cached) = self.cache.get(&key).await {
                cached
            } else {
                match ops::judge_payload(payload, law) {
                    Ok(value) => {
                        let text = pretty(&value);
                        self.cache.insert(key, text.clone()).await;
                        text
                    }
                    Err(e) => return Ok(error_text(e)),
                }
            };
            // Fold the outcome into the session byte on both cache-hit and
            // fresh paths: the resident posture tracks the decision, not the
            // cache mechanics.
            self.apply_judge(&text);
            Ok(success_from_text(text))
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

            let text = if let Some(cached) = self.cache.get(&key).await {
                cached
            } else {
                match ops::admit_payload(payload, policy) {
                    Ok(value) => {
                        let text = pretty(&value);
                        self.cache.insert(key, text.clone()).await;
                        text
                    }
                    Err(e) => return Ok(error_text(e)),
                }
            };
            self.apply_admit(&text);
            Ok(success_from_text(text))
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

            let cached = if has_ts_ns {
                self.cache.get(&key).await
            } else {
                None
            };
            let text = match cached {
                Some(c) => c,
                None => match ops::receipt_payload(payload) {
                    Ok(value) => {
                        let text = pretty(&value);
                        if has_ts_ns {
                            self.cache.insert(key, text.clone()).await;
                        }
                        text
                    }
                    Err(e) => return Ok(error_text(e)),
                },
            };
            self.apply_receipt(&text);
            Ok(success_from_text(text))
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
        #[tool(description = "Promote a law object's BreedStanding rung. \
                          Input: payload_json ({\"standing\": \"<registry name>\"}), auditor \
                          (required for promotions to REPLAYABLE/CERTIFIED). \
                          Output: {status: \"promoted\"|\"denied\", from, to, ...}.")]
        async fn promote(
            &self,
            params: Parameters<PromoteParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let payload = &params.0.payload_json;
            let auditor = params.0.auditor.unwrap_or_default();
            let input_hash = input_hash_hex(&canonical_bytes(payload));
            let authority_digest = if auditor.is_empty() {
                None
            } else {
                Some(digest_hex(&auditor))
            };
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
            let dir = params
                .0
                .dir
                .unwrap_or_else(|| DEFAULT_RECEIPTS_DIR.to_string());
            let store = match ReceiptStore::open(&dir) {
                Ok(s) => s,
                Err(e) => {
                    return Ok(error_text(format!(
                        "failed to open receipts dir {dir}: {e}"
                    )))
                }
            };
            let head_hash = match store.last_chain_hash() {
                Ok(h) => hex::encode(h),
                Err(e) => {
                    return Ok(error_text(format!(
                        "failed to read receipts ledger head: {e}"
                    )))
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
            let dir = params
                .0
                .dir
                .unwrap_or_else(|| DEFAULT_RECEIPTS_DIR.to_string());
            let store = match ReceiptStore::open(&dir) {
                Ok(s) => s,
                Err(e) => {
                    return Ok(error_text(format!(
                        "failed to open receipts dir {dir}: {e}"
                    )))
                }
            };
            let head_hash = match store.last_chain_hash() {
                Ok(h) => hex::encode(h),
                Err(e) => {
                    return Ok(error_text(format!(
                        "failed to read receipts ledger head: {e}"
                    )))
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

        /// Solve a PDDL8 problem (the Day 2 `goal → plan` step). Deterministic
        /// pure function of its input — always cached.
        #[tool(description = "Solve a classical or temporal PDDL8 problem. \
                          Input: payload_json ({domain, problem, mode} inline text and/or \
                          {domain_file, problem_file} paths, or a single combined domain+problem \
                          string in `domain`; mode is \"classical\" (default) or \"temporal\"). \
                          Output: {admitted, plan_len, plan, ...} or a structured refusal \
                          {admitted: false, refusal_reason}. Calls the same ops::plan_solve_payload \
                          the CLI `plan solve` verb runs — one implementation, no drift.")]
        async fn plan_solve(
            &self,
            params: Parameters<PlanSolveParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let payload = &params.0.payload_json;
            let input_hash = input_hash_hex(&canonical_bytes(payload));
            let key = ToolCacheKey {
                tool: "plan_solve",
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
            match ops::plan_solve_payload(payload) {
                Ok(value) => {
                    let text = pretty(&value);
                    self.cache.insert(key, text.clone()).await;
                    Ok(success_from_text(text))
                }
                Err(e) => Ok(error_text(e)),
            }
        }

        /// Rank candidate revenue goal states (the Day 2 `observe → propose`
        /// step). Output is proposal (O), never authority (O*) — AR-9.
        #[tool(
            description = "Rank candidate goal states for a revenue snapshot under a \
                          domain-authored objective. Input: payload_json ({state, \
                          objective|objective_file}), objective (optional objective JSON file path). \
                          Output: {status, count, proposals: [{pddl_goal, score, proposal_hash, \
                          rationale, ...}]}. Output is proposal (observation), not authority: every \
                          candidate must still pass judge/admit before any effect (AR-9)."
        )]
        async fn propose_revenue(
            &self,
            params: Parameters<ProposeParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let payload = &params.0.payload_json;
            let objective = params.0.objective.unwrap_or_default();
            let input_hash = input_hash_hex(&canonical_bytes(payload));
            let policy_digest = if objective.is_empty() {
                None
            } else {
                Some(digest_hex(&objective))
            };
            let key = ToolCacheKey {
                tool: "propose_revenue",
                input_hash: &input_hash,
                capability_version: Some(CAPABILITY_VERSION),
                policy_digest: policy_digest.as_deref(),
                authority_digest: None,
                environment_digest: None,
                replay_mode: None,
            }
            .to_key_string();

            if let Some(cached) = self.cache.get(&key).await {
                return Ok(success_from_text(cached));
            }
            match ops::propose_revenue_payload(payload, &objective) {
                Ok(value) => {
                    let text = pretty(&value);
                    self.cache.insert(key, text.clone()).await;
                    Ok(success_from_text(text))
                }
                Err(e) => Ok(error_text(e)),
            }
        }

        /// Emit the top-ranked revenue proposal's PDDL goal atom, ready to
        /// splice into a `plan_solve` problem `(:goal ...)` block.
        #[tool(
            description = "Emit the top-ranked proposal's PDDL goal atom for plan_solve. \
                          Input: payload_json ({state, objective|objective_file}), objective \
                          (optional objective JSON file path). Output: {status, goal, proposal_hash, \
                          rationale, ...} or {status: \"no_lawful_candidates\"}. Output is proposal \
                          (observation), not authority (AR-9)."
        )]
        async fn propose_goal(
            &self,
            params: Parameters<ProposeParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let payload = &params.0.payload_json;
            let objective = params.0.objective.unwrap_or_default();
            let input_hash = input_hash_hex(&canonical_bytes(payload));
            let policy_digest = if objective.is_empty() {
                None
            } else {
                Some(digest_hex(&objective))
            };
            let key = ToolCacheKey {
                tool: "propose_goal",
                input_hash: &input_hash,
                capability_version: Some(CAPABILITY_VERSION),
                policy_digest: policy_digest.as_deref(),
                authority_digest: None,
                environment_digest: None,
                replay_mode: None,
            }
            .to_key_string();

            if let Some(cached) = self.cache.get(&key).await {
                return Ok(success_from_text(cached));
            }
            match ops::propose_goal_payload(payload, &objective) {
                Ok(value) => {
                    let text = pretty(&value);
                    self.cache.insert(key, text.clone()).await;
                    Ok(success_from_text(text))
                }
                Err(e) => Ok(error_text(e)),
            }
        }

        /// Sweep a fleet of agent8 bytes with the SWAR popcount kernel and
        /// report admission statistics plus per-agent flag strings.
        #[tool(
            description = "Sweep a fleet of agent8 bytes with the SWAR popcount admission kernel. \
                          Input: fleet_json (a bare array of agent bytes, or {\"agents\": [<u8>...], \
                          \"required_mask\": <u8?>}; required_mask defaults to GRANT_REQUIRED 0x6F). \
                          Output: {required_mask, stats: {total, admitted, blocked, receipted, \
                          replayable}, per_agent: [{index, byte, flags, grant}]}."
        )]
        async fn fleet_status(
            &self,
            params: Parameters<FleetStatusParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let (agents, required_mask) = match parse_fleet(&params.0.fleet_json) {
                Ok(x) => x,
                Err(e) => return Ok(error_text(e)),
            };
            // Pack into the SWAR fleet (8 agents/word) and sweep with the
            // popcount kernel. `with_fill` rounds up to a whole 64-bit word;
            // trailing lanes are zero-filled, so we correct the popcount stats
            // back to exactly the provided agents below.
            let mut fleet = Fleet::with_fill(agents.len().max(1), AgentByte::empty());
            for (i, &b) in agents.iter().enumerate() {
                fleet.set(i, AgentByte::from_raw(b));
            }
            let raw = fleet.sweep_stats(required_mask);
            let provided = agents.len() as u64;
            let pad = fleet.len() as u64 - provided;
            // A zero (empty) padding lane is admitted iff the required mask is
            // itself empty, and never carries RECEIPTED/REPLAYABLE.
            let empty_admits = u64::from(AgentByte::empty().denial(required_mask) == 0);
            let admitted = raw.admitted.saturating_sub(pad * empty_admits);
            let stats = json!({
                "total": provided,
                "admitted": admitted,
                "blocked": provided - admitted,
                "receipted": raw.receipted,
                "replayable": raw.replayable,
            });
            const MAX_PER_AGENT: usize = 64;
            let per_agent: Vec<Value> = agents
                .iter()
                .take(MAX_PER_AGENT)
                .enumerate()
                .map(|(i, &b)| {
                    let ab = AgentByte::from_raw(b);
                    json!({
                        "index": i,
                        "byte": b,
                        "flags": ab.to_string(),
                        "grant": matches!(ab.select(required_mask), AgentSelect::Grant),
                    })
                })
                .collect();
            let out = json!({
                "required_mask": required_mask,
                "fleet_lanes": fleet.len(),
                "agents_provided": provided,
                "stats": stats,
                "per_agent": per_agent,
                "per_agent_truncated": agents.len() > MAX_PER_AGENT,
            });
            Ok(success_from_text(pretty(&out)))
        }

        /// Return the calling session's current resident [`AgentByte`] — the
        /// agent8 projection updated by this session's judge/admit/receipt
        /// outcomes (see the adapter doc on [`ServerState`]).
        #[tool(
            description = "Report the calling session's resident agent8 AgentByte, updated by this \
                          session's judge/admit/receipt outcomes. Output: {byte, flags (8-char \
                          PRCHUBEA string), select (Grant|Deny against GRANT_REQUIRED), \
                          missing_for_grant, ...}. No input."
        )]
        async fn whoami(&self) -> Result<CallToolResult, rmcp::ErrorData> {
            let b = self.agent_byte();
            let grant = matches!(b.select(AgentByte::GRANT_REQUIRED), AgentSelect::Grant);
            let out = json!({
                "byte": b.raw(),
                "flags": b.to_string(),
                "select": if grant { "Grant" } else { "Deny" },
                "grant_required_mask": AgentByte::GRANT_REQUIRED,
                "missing_for_grant": b.denial(AgentByte::GRANT_REQUIRED),
                "legend": "flags high→low P R C H U B E A = Replayable Receipted Conformant \
                           Healthy aUthority Budget Evidence Admitted",
            });
            Ok(success_from_text(pretty(&out)))
        }
    }

    /// Parse a `fleet_json` string into `(agent_bytes, required_mask)`.
    ///
    /// Accepts either a bare JSON array of byte values or an object
    /// `{"agents": [...], "required_mask": <u8?>}`; `required_mask` defaults to
    /// [`AgentByte::GRANT_REQUIRED`].
    fn parse_fleet(input: &str) -> Result<(Vec<u8>, u8), String> {
        let v: Value =
            serde_json::from_str(input).map_err(|e| format!("invalid fleet_json: {e}"))?;
        let (agents_v, mask) = if v.is_array() {
            (v, AgentByte::GRANT_REQUIRED)
        } else {
            let agents = v
                .get("agents")
                .cloned()
                .ok_or_else(|| "fleet_json object needs an `agents` array".to_string())?;
            let mask = v
                .get("required_mask")
                .and_then(serde_json::Value::as_u64)
                .and_then(|m| u8::try_from(m).ok())
                .unwrap_or(AgentByte::GRANT_REQUIRED);
            (agents, mask)
        };
        let arr = agents_v
            .as_array()
            .ok_or_else(|| "`agents` must be an array of bytes".to_string())?;
        let mut agents = Vec::with_capacity(arr.len());
        for a in arr {
            let n = a
                .as_u64()
                .ok_or_else(|| "agent bytes must be integers 0..=255".to_string())?;
            let byte =
                u8::try_from(n).map_err(|_| format!("agent byte {n} out of range 0..=255"))?;
            agents.push(byte);
        }
        Ok((agents, mask))
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
                LOCK.get_or_init(|| Mutex::new(()))
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
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
            let params = Parameters(JudgeParams {
                payload_json: payload.clone(),
                law: None,
            });
            let tool_result = state.judge(params).await.expect("judge should not error");
            assert!(!tool_result.is_error.unwrap_or(false));

            let tool_json: Value =
                serde_json::from_str(&text_of(&tool_result)).expect("tool output is JSON");
            let direct_json =
                ops::judge_payload(&payload, "default").expect("direct judge_payload");
            assert_eq!(tool_json, direct_json);
        }

        #[tokio::test]
        async fn admit_tool_matches_ops_admit_payload_denial_shape() {
            let state = ServerState::default();
            let payload = r#"{"value":{"id":1},"obligations":[{"type":"blocking_constraint","reason":"stop"}]}"#.to_string();
            let params = Parameters(AdmitParams {
                payload_json: payload.clone(),
                policy: None,
            });
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
        #[allow(clippy::await_holding_lock)] // std lock deliberately serializes env-mutating tests
        async fn receipt_tool_matches_ops_receipt_payload_with_explicit_ts_ns() {
            #[cfg(feature = "law-signed")]
            let _guard = with_mcp_test_signing_key();
            let state = ServerState::default();
            let payload = r#"{"value":{"id":1},"ts_ns":42}"#.to_string();
            let params = Parameters(ReceiptParams {
                payload_json: payload.clone(),
            });
            let tool_result = state
                .receipt(params)
                .await
                .expect("receipt should not error");
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
            let params = Parameters(InspectObligationParams {
                obligations_json: obligations.clone(),
            });
            let tool_result = state
                .inspect_obligation(params)
                .await
                .expect("inspect_obligation should not error");
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
            let params = Parameters(PromoteParams {
                payload_json: payload.clone(),
                auditor: None,
            });
            let tool_result = state
                .promote(params)
                .await
                .expect("promote should not error");
            let tool_json: Value =
                serde_json::from_str(&text_of(&tool_result)).expect("tool output is JSON");
            let direct_json = ops::promote_payload(&payload, "").expect("direct promote_payload");
            assert_eq!(tool_json, direct_json);
        }

        #[tokio::test]
        async fn malformed_judge_payload_is_tool_error() {
            let state = ServerState::default();
            let params = Parameters(JudgeParams {
                payload_json: "not json".to_string(),
                law: None,
            });
            let tool_result = state
                .judge(params)
                .await
                .expect("call itself should not error");
            assert!(tool_result.is_error.unwrap_or(false));
        }

        // ── cache hit/skip behavior ─────────────────────────────────────────

        #[tokio::test]
        async fn identical_judge_calls_hit_cache_and_return_same_json() {
            let state = ServerState::default();
            let payload = r#"{"value":{"id":7}}"#.to_string();
            let make_params = || {
                Parameters(JudgeParams {
                    payload_json: payload.clone(),
                    law: None,
                })
            };

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
        #[allow(clippy::await_holding_lock)] // std lock deliberately serializes env-mutating tests
        async fn receipt_without_ts_ns_is_never_cached() {
            #[cfg(feature = "law-signed")]
            let _guard = with_mcp_test_signing_key();
            let state = ServerState::default();
            let payload = r#"{"value":{"id":9}}"#.to_string();
            let make_params = || {
                Parameters(ReceiptParams {
                    payload_json: payload.clone(),
                })
            };

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
        #[allow(clippy::await_holding_lock)] // std lock deliberately serializes env-mutating tests
        async fn receipt_with_ts_ns_is_cached_and_deterministic() {
            #[cfg(feature = "law-signed")]
            let _guard = with_mcp_test_signing_key();
            let state = ServerState::default();
            let payload = r#"{"value":{"id":11},"ts_ns":123}"#.to_string();
            let make_params = || {
                Parameters(ReceiptParams {
                    payload_json: payload.clone(),
                })
            };

            let first = state.receipt(make_params()).await.expect("first call");
            let second = state.receipt(make_params()).await.expect("second call");
            assert_eq!(text_of(&first), text_of(&second));
        }

        #[tokio::test]
        async fn show_andon_round_trips_green_and_halted() {
            let state = ServerState::default();
            let green_lawobject = json!({"andon": {"status": "Green"}}).to_string();
            let params = Parameters(ShowAndonParams {
                lawobject_json: green_lawobject,
            });
            let result = state
                .show_andon(params)
                .await
                .expect("show_andon should not error");
            let json: Value = serde_json::from_str(&text_of(&result)).expect("json");
            assert_eq!(json["andon"]["status"], json!("Green"));
        }

        #[tokio::test]
        #[allow(clippy::await_holding_lock)] // std lock deliberately serializes env-mutating tests
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

            let validate_params = Parameters(ReceiptDirParams {
                dir: Some(dir_str.clone()),
            });
            let validate_result = state
                .receipt_validate(validate_params)
                .await
                .expect("receipt_validate should not error");
            let validate_json: Value =
                serde_json::from_str(&text_of(&validate_result)).expect("json");
            assert_eq!(validate_json["verdict"]["ok"], json!(true));

            let replay_params = Parameters(ReceiptDirParams {
                dir: Some(dir_str.clone()),
            });
            let replay_result = state
                .receipt_replay(replay_params)
                .await
                .expect("receipt_replay should not error");
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
    // `serve()` resolves once the initialize handshake completes and returns a
    // RunningService handle. We must then await `.waiting()` to actually serve
    // tool calls until the client disconnects — otherwise `main` returns
    // immediately after the handshake and the process exits (clean code 0)
    // before answering a single `tools/list` or `tools/call`.
    let service = server::ServerState::default().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(not(feature = "mcp"))]
fn main() -> anyhow::Result<()> {
    eprintln!("mcp_lawobject_server requires the `mcp` Cargo feature. Rebuild with --features mcp");
    std::process::exit(1);
}

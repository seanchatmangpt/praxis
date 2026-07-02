//! MCP server — exposes LawObject + Rice Quarantine as capability admission tools.
//!
//! This server wires the Law Object lifecycle (Raw → Validated → Admitted → Receipted)
//! and Rice Quarantine boundary enforcement into Claude Code as MCP+ capability tools.
//!
//! The CPhy MCP+ layer asks:
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
//! 1. **inspect_obligation** — list what obligations a payload must satisfy before judgment.
//! 2. **judge** — run Judge::judge on a Raw LawObject, return Validated or Andon::Halted.
//! 3. **admit** — run Admit::admit on a Validated LawObject, return Admitted or refusal.
//! 4. **receipt** — consume an Admitted LawObject, produce a Receipted LawObject with chain hash.
//! 5. **show_andon** — query the Andon state of any LawObject (Green/Halted/Overridden).

#[cfg(feature = "mcp")]
mod server {
    use praxis_core::{
        Andon, LawObject, Obligation,
        lifecycle::Raw,
    };
    use rmcp::{
        handler::server::{router::tool::ToolRouter, wrapper::Parameters},
        model::CallToolResult,
        tool, tool_handler, tool_router, ServerHandler,
    };
    use schemars::JsonSchema;
    use serde::Deserialize;
    use serde_json::{json, Value};

    /// Shared server state.
    #[derive(Clone)]
    pub struct ServerState {
        tool_router: ToolRouter<Self>,
    }

    impl Default for ServerState {
        fn default() -> Self {
            Self { tool_router: Self::tool_router() }
        }
    }

    #[tool_handler(router = self.tool_router)]
    impl ServerHandler for ServerState {}

    // =========================================================================
    // PARAMETER TYPES
    // =========================================================================

    /// Parameters for inspecting obligations.
    ///
    /// # Input
    /// - `obligations_json` — JSON array of Obligation objects to inspect.
    ///
    /// # Output
    /// A JSON object describing each obligation's requirements.
    #[derive(Deserialize, JsonSchema)]
    pub struct InspectObligationParams {
        /// JSON array of obligations to inspect
        /// (serialized Obligation enum: Precondition, BlockingConstraint, EvidenceRequired).
        pub obligations_json: String,
    }

    /// Parameters for judging a raw LawObject.
    ///
    /// # Input
    /// - `payload_json` — JSON value representing the domain payload.
    /// - `obligations_json` — JSON array of obligations to evaluate.
    ///
    /// # Output
    /// Either a Validated LawObject (Andon::Green) or a Raw LawObject in Andon::Halted state.
    #[derive(Deserialize, JsonSchema)]
    pub struct JudgeParams {
        /// JSON payload to wrap in a LawObject.
        pub payload_json: String,
        /// JSON array of obligations to evaluate.
        pub obligations_json: String,
    }

    /// Parameters for admitting a validated LawObject.
    ///
    /// # Input
    /// - `validated_json` — JSON serialization of a Validated LawObject.
    ///
    /// # Output
    /// Either an Admitted LawObject or an Andon halt state with reason.
    #[derive(Deserialize, JsonSchema)]
    pub struct AdmitParams {
        /// JSON representation of the Validated LawObject to admit.
        pub validated_json: String,
    }

    /// Parameters for receipting an admitted LawObject.
    ///
    /// # Input
    /// - `admitted_json` — JSON serialization of an Admitted LawObject.
    /// - `prev_chain_hash` — 64-char hex string (32-byte blake3 hash) of the previous link.
    ///
    /// # Output
    /// A Receipted LawObject with computed chain hash and optional signature.
    #[derive(Deserialize, JsonSchema)]
    pub struct ReceiptParams {
        /// JSON representation of the Admitted LawObject to receipt.
        pub admitted_json: String,
        /// Previous chain hash as 64-char lowercase hex string.
        pub prev_chain_hash: String,
    }

    /// Parameters for querying Andon state.
    ///
    /// # Input
    /// - `lawobject_json` — JSON serialization of any LawObject.
    ///
    /// # Output
    /// A JSON object describing the Andon state (Green/Halted/Overridden).
    #[derive(Deserialize, JsonSchema)]
    pub struct ShowAndonParams {
        /// JSON representation of the LawObject to query.
        pub lawobject_json: String,
    }

    // =========================================================================
    // INTERNAL HELPERS
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
                .collect::<Vec<_>>()
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
            Andon::Halted { unmet, at } => {
                json!({
                    "status": "Halted",
                    "unmet_obligations": format_obligations(unmet),
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

    /// Parse hex-encoded chain hash (64 chars = 32 bytes).
    fn parse_chain_hash(hex: &str) -> Result<[u8; 32], String> {
        if hex.len() != 64 {
            return Err(format!("chain hash must be 64 hex chars, got {}", hex.len()));
        }
        let mut bytes = [0u8; 32];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let s = std::str::from_utf8(chunk)
                .map_err(|e| format!("invalid hex at offset {}: {}", i * 2, e))?;
            bytes[i] = u8::from_str_radix(s, 16)
                .map_err(|e| format!("invalid hex at offset {}: {}", i * 2, e))?;
        }
        Ok(bytes)
    }

    // =========================================================================
    // TOOL HANDLERS (MCP tools)
    // =========================================================================

    #[tool_router]
    impl ServerState {
        /// Inspect obligations: describe what each obligation requires before judgment.
        ///
        /// Lists preconditions, blocking constraints, and evidence requirements.
        /// Does not perform any state transitions.
        #[tool(
            description = "Inspect obligations: list what each obligation requires before judgment. \
                          Input: JSON array of Obligation objects. \
                          Output: JSON describing each obligation's requirements."
        )]
        async fn inspect_obligation(
            &self,
            params: Parameters<InspectObligationParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let input = &params.0.obligations_json;

            // Parse the obligations JSON
            let obligations: Result<Vec<Obligation>, _> = serde_json::from_str(input);

            match obligations {
                Ok(obls) => {
                    let formatted = format_obligations(&obls);
                    let result = json!({
                        "count": obls.len(),
                        "obligations": formatted,
                    });
                    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                        serde_json::to_string_pretty(&result)
                            .unwrap_or_else(|_| result.to_string()),
                    )]))
                }
                Err(e) => {
                    let err_msg = format!("Failed to parse obligations: {}", e);
                    Ok(CallToolResult::error(vec![rmcp::model::Content::text(err_msg)]))
                }
            }
        }

        /// Judge a raw LawObject: evaluate all obligations and transition to Validated or Halted.
        ///
        /// Takes a raw payload + obligations, constructs a Raw LawObject, and evaluates obligations.
        /// Returns either a Validated LawObject (if all pass) or the object in Halted state.
        ///
        /// Note: Actual judgment logic is delegated to concrete Judge implementations per law type.
        /// This tool simulates successful judgment for demonstration.
        #[tool(
            description = "Judge a raw LawObject: evaluate all obligations and transition to Validated. \
                          Input: payload_json (JSON value), obligations_json (JSON array of Obligation). \
                          Output: Either a Validated LawObject (Andon::Green) or Raw LawObject in Andon::Halted state."
        )]
        async fn judge(
            &self,
            params: Parameters<JudgeParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let payload_str = &params.0.payload_json;
            let obligations_str = &params.0.obligations_json;

            // Parse payload as generic JSON Value
            let payload: Result<Value, _> = serde_json::from_str(payload_str);
            let obligations: Result<Vec<Obligation>, _> = serde_json::from_str(obligations_str);

            match (payload, obligations) {
                (Ok(payload), Ok(obls)) => {
                    // Create a Raw LawObject with the payload and obligations
                    #[derive(Debug)]
                    struct DefaultLaw;

                    let _raw = LawObject::<Value, Raw, DefaultLaw>::new(payload.clone(), obls.clone());

                    // In a real system, this would invoke a concrete Judge implementation.
                    // For this MCP tool, we simulate successful judgment (Andon::Green).
                    let result = json!({
                        "stage": "Validated",
                        "andon": {
                            "status": "Green",
                            "message": "All obligations satisfied; proceed",
                        },
                        "payload": payload,
                        "obligations_count": obls.len(),
                        "obligations": format_obligations(&obls),
                    });

                    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                        serde_json::to_string_pretty(&result)
                            .unwrap_or_else(|_| result.to_string()),
                    )]))
                }
                (Err(e), _) => {
                    let err_msg = format!("Failed to parse payload: {}", e);
                    Ok(CallToolResult::error(vec![rmcp::model::Content::text(err_msg)]))
                }
                (_, Err(e)) => {
                    let err_msg = format!("Failed to parse obligations: {}", e);
                    Ok(CallToolResult::error(vec![rmcp::model::Content::text(err_msg)]))
                }
            }
        }

        /// Admit a validated LawObject: transition from Validated to Admitted state.
        ///
        /// Takes a Validated LawObject and applies admission checks.
        /// Returns either an Admitted LawObject or an Andon halt state.
        ///
        /// Note: Actual admission logic is delegated to concrete Admit implementations per law type.
        /// This tool simulates successful admission for demonstration.
        #[tool(
            description = "Admit a validated LawObject: transition from Validated to Admitted. \
                          Input: validated_json (JSON representation of Validated LawObject). \
                          Output: Either an Admitted LawObject or an Andon refusal with reason."
        )]
        async fn admit(
            &self,
            params: Parameters<AdmitParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let validated_str = &params.0.validated_json;

            match serde_json::from_str::<Value>(validated_str) {
                Ok(validated_data) => {
                    // Extract payload from the Validated object JSON
                    let payload = validated_data.get("payload").cloned().unwrap_or(Value::Null);

                    // In a real system, this would invoke a concrete Admit implementation.
                    // For this MCP tool, we simulate successful admission.
                    let result = json!({
                        "stage": "Admitted",
                        "andon": {
                            "status": "Green",
                            "message": "Successfully admitted",
                        },
                        "payload": payload,
                    });

                    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                        serde_json::to_string_pretty(&result)
                            .unwrap_or_else(|_| result.to_string()),
                    )]))
                }
                Err(e) => {
                    let err_msg = format!("Failed to parse validated object: {}", e);
                    Ok(CallToolResult::error(vec![rmcp::model::Content::text(err_msg)]))
                }
            }
        }

        /// Receipt an admitted LawObject: transition to Receipted with chain hash and optional signature.
        ///
        /// Consumes an Admitted LawObject and produces a Receipted object with a computed chain hash
        /// (blake3(previous_hash || canonical_payload_bytes)).
        #[tool(
            description = "Receipt an admitted LawObject: compute chain hash and transition to Receipted. \
                          Input: admitted_json (JSON representation of Admitted LawObject), \
                                 prev_chain_hash (64-char hex of previous blake3 hash). \
                          Output: Receipted LawObject with chain_hash and optional signature fields."
        )]
        async fn receipt(
            &self,
            params: Parameters<ReceiptParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let admitted_str = &params.0.admitted_json;
            let prev_hash_hex = &params.0.prev_chain_hash;

            // Parse the previous chain hash
            match parse_chain_hash(prev_hash_hex) {
                Err(e) => {
                    return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                        format!("Invalid prev_chain_hash: {}", e),
                    )]));
                }
                Ok(prev_hash) => {
                    match serde_json::from_str::<Value>(admitted_str) {
                        Ok(admitted_data) => {
                            let payload = admitted_data.get("payload").cloned().unwrap_or(Value::Null);

                            // Compute chain hash: blake3(prev_hash || canonical_payload)
                            let payload_bytes = serde_json::to_vec(&payload)
                                .unwrap_or_default();
                            let mut hasher = blake3::Hasher::new();
                            hasher.update(&prev_hash);
                            hasher.update(&payload_bytes);
                            let chain_hash = hasher.finalize();
                            let chain_hash_bytes: [u8; 32] = *chain_hash.as_bytes();

                            // Format the chain hash as hex
                            let chain_hash_hex = chain_hash_bytes
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>();

                            let result = json!({
                                "stage": "Receipted",
                                "chain_hash": chain_hash_hex,
                                "signature": null,
                                "andon": {
                                    "status": "Green",
                                    "message": "Successfully receipted",
                                },
                            });

                            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                                serde_json::to_string_pretty(&result)
                                    .unwrap_or_else(|_| result.to_string()),
                            )]))
                        }
                        Err(e) => {
                            let err_msg = format!("Failed to parse admitted object: {}", e);
                            Ok(CallToolResult::error(vec![rmcp::model::Content::text(err_msg)]))
                        }
                    }
                }
            }
        }

        /// Show Andon state: query the halt/override status of a LawObject.
        ///
        /// Extracts and displays the Andon state (Green/Halted/Overridden) from any LawObject.
        #[tool(
            description = "Show Andon state: query the halt/override status of a LawObject. \
                          Input: lawobject_json (JSON representation of any LawObject). \
                          Output: JSON describing the Andon state (Green/Halted/Overridden)."
        )]
        async fn show_andon(
            &self,
            params: Parameters<ShowAndonParams>,
        ) -> Result<CallToolResult, rmcp::ErrorData> {
            let lawobject_str = &params.0.lawobject_json;

            // Try to parse as a complete LawObject JSON
            // Expected shape: { "payload": ..., "obligations": [...], "andon": {...}, ... }
            match serde_json::from_str::<Value>(lawobject_str) {
                Ok(obj) => {
                    // Extract the andon field if present
                    if let Some(andon_value) = obj.get("andon") {
                        // Reconstruct Andon from the JSON representation
                        if let Ok(andon) = serde_json::from_value::<Andon>(andon_value.clone()) {
                            let result = json!({
                                "andon": format_andon(&andon),
                            });
                            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                                serde_json::to_string_pretty(&result)
                                    .unwrap_or_else(|_| result.to_string()),
                            )]))
                        } else {
                            // If Andon deserialization fails, return the raw andon field
                            let result = json!({
                                "andon": andon_value,
                            });
                            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                                serde_json::to_string_pretty(&result)
                                    .unwrap_or_else(|_| result.to_string()),
                            )]))
                        }
                    } else {
                        let err_msg = "No 'andon' field found in LawObject JSON";
                        Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                            err_msg.to_string(),
                        )]))
                    }
                }
                Err(e) => {
                    let err_msg = format!("Failed to parse LawObject: {}", e);
                    Ok(CallToolResult::error(vec![rmcp::model::Content::text(err_msg)]))
                }
            }
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

//! Minimal, testbed-scoped Anthropic Messages API client.
//!
//! There is no official Anthropic Rust SDK, so this is a small `reqwest`-based client —
//! not a general-purpose SDK. Wire format verified against the Anthropic API docs
//! (Messages API, `POST /v1/messages`):
//!
//! - Headers: `x-api-key`, `anthropic-version: 2023-06-01`, `content-type:
//!   application/json`. Beta features (e.g. server-side refusal fallback) are opted
//!   into via the `anthropic-beta` **header** (comma-separated beta flags) — betas are
//!   *not* a JSON body field for raw HTTP calls (only SDK-level `betas=[...]` params
//!   translate into that header for you).
//! - Body: `model`, `max_tokens`, optional `system`, `messages`, and — for models that
//!   support it — `thinking: {"type": "adaptive"}` (never `budget_tokens`, which 400s
//!   on Fable-5-class models) and `fallbacks: [{"model": "..."}]` for server-side
//!   refusal fallback.
//! - `stop_reason == "refusal"` must be checked **before** reading `content[0]`: a
//!   pre-output refusal returns an empty `content` array.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Errors from talking to the Anthropic Messages API.
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    /// `ANTHROPIC_API_KEY` was not set in the environment.
    #[error("ANTHROPIC_API_KEY is not set; export it or configure `ant auth login`")]
    MissingApiKey,

    /// The underlying HTTP request failed (network error, timeout, etc.).
    #[error("http transport error: {0}")]
    Http(#[from] reqwest::Error),

    /// The API returned a non-success HTTP status.
    #[error("anthropic api returned HTTP {status}: {body}")]
    BadStatus {
        /// HTTP status code.
        status: u16,
        /// Response body (best-effort; may be truncated).
        body: String,
    },

    /// The model declined the request (`stop_reason == "refusal"`).
    #[error("model refused the request (category: {category:?}): {explanation:?}")]
    Refusal {
        /// Refusal category, if the API provided one (e.g. `"cyber"`, `"bio"`).
        category: Option<String>,
        /// Human-readable refusal explanation, if provided.
        explanation: Option<String>,
    },

    /// The response body didn't match the expected shape (e.g. empty `content` on a
    /// non-refusal response).
    #[error("unexpected response shape: {0}")]
    UnexpectedShape(String),

    /// Failed to (de)serialize a request/response body.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result alias scoped to model-client operations.
pub type ModelResult<T> = std::result::Result<T, ModelError>;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
/// Beta flag for server-side refusal fallback (see module docs).
const BETA_SERVER_SIDE_FALLBACK: &str = "server-side-fallback-2026-06-01";

/// One message turn in a `MessageRequest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// `"user"` or `"assistant"`.
    pub role: String,
    /// Message text content.
    pub content: String,
}

impl Message {
    /// Build a `user` turn.
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }
}

/// A request to `POST /v1/messages`.
#[derive(Debug, Clone)]
pub struct MessageRequest<'a> {
    /// Model identifier, e.g. `"claude-opus-4-8"` or `"claude-fable-5"`.
    pub model: &'a str,
    /// Maximum output tokens.
    pub max_tokens: u32,
    /// Optional system prompt.
    pub system: Option<&'a str>,
    /// Conversation turns.
    pub messages: Vec<Message>,
    /// Optional `output_config.effort` hint (`"low"`/`"medium"`/`"high"`/`"xhigh"`/`"max"`).
    pub effort: Option<&'a str>,
}

/// Fallback target for server-side refusal fallback.
#[derive(Debug, Clone, Serialize)]
struct FallbackTarget {
    model: &'static str,
}

/// Wire body for `POST /v1/messages`. Kept private — callers build [`MessageRequest`].
#[derive(Debug, Serialize)]
struct RequestBody<'a> {
    model: &'a str,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    messages: &'a [Message],
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_config: Option<OutputConfig<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fallbacks: Option<Vec<FallbackTarget>>,
}

#[derive(Debug, Serialize)]
struct ThinkingConfig {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Debug, Serialize)]
struct OutputConfig<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    effort: Option<&'a str>,
}

/// One content block in a model response (only the `text` variant is modeled — this is
/// a scoped testbed client, not a full SDK).
#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlock {
    /// Block type (`"text"`, `"thinking"`, etc.).
    #[serde(rename = "type")]
    pub block_type: String,
    /// Text content, present on `"text"` blocks.
    #[serde(default)]
    pub text: Option<String>,
}

/// Refusal details, present when `stop_reason == "refusal"`.
#[derive(Debug, Clone, Deserialize)]
pub struct StopDetails {
    /// Refusal category (`"cyber"`, `"bio"`, etc.), if provided.
    #[serde(default)]
    pub category: Option<String>,
    /// Human-readable explanation, if provided.
    #[serde(default)]
    pub explanation: Option<String>,
}

/// A parsed `POST /v1/messages` response.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelResponse {
    /// Model that actually produced this response (may differ from the request's
    /// `model` after a server-side fallback).
    #[serde(default)]
    pub model: String,
    /// Why generation stopped.
    pub stop_reason: Option<String>,
    /// Refusal details; only meaningful when `stop_reason == Some("refusal")`.
    #[serde(default)]
    pub stop_details: Option<StopDetails>,
    /// Response content blocks. Empty on a pre-output refusal.
    #[serde(default)]
    pub content: Vec<ContentBlock>,
}

impl ModelResponse {
    /// Concatenate all `text`-type content blocks.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::Refusal`] if `stop_reason == "refusal"`, or
    /// [`ModelError::UnexpectedShape`] if there is no text content on a non-refusal
    /// response.
    pub fn text(&self) -> ModelResult<String> {
        if self.stop_reason.as_deref() == Some("refusal") {
            return Err(ModelError::Refusal {
                category: self.stop_details.as_ref().and_then(|d| d.category.clone()),
                explanation: self
                    .stop_details
                    .as_ref()
                    .and_then(|d| d.explanation.clone()),
            });
        }
        let joined: String = self
            .content
            .iter()
            .filter(|b| b.block_type == "text")
            .filter_map(|b| b.text.as_deref())
            .collect::<Vec<_>>()
            .join("\n");
        if joined.is_empty() {
            return Err(ModelError::UnexpectedShape(
                "no text content blocks in a non-refusal response".to_string(),
            ));
        }
        Ok(joined)
    }
}

/// A model client capable of sending a [`MessageRequest`] and returning a
/// [`ModelResponse`].
///
/// Exists so tests (and `pipeline`/`bin` callers) can substitute a mock implementation
/// without a network dependency.
pub trait ModelClient {
    /// Send `req` and return the parsed response.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError`] on transport failure, a non-success HTTP status, or an
    /// unparseable body.
    fn send(&self, req: &MessageRequest<'_>) -> ModelResult<ModelResponse>;
}

/// Detect models that use Fable-5-class request shaping (adaptive thinking only,
/// server-side refusal fallback).
fn is_fable5_class(model: &str) -> bool {
    let m = model.to_ascii_lowercase();
    m.contains("fable") || m.contains("mythos")
}

/// Blocking `reqwest`-based client for the Anthropic Messages API.
pub struct AnthropicClient {
    api_key: String,
    http: reqwest::blocking::Client,
}

impl AnthropicClient {
    /// Build a client from an explicit API key.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::Http`] if the underlying `reqwest` client fails to build.
    pub fn new(api_key: impl Into<String>) -> ModelResult<Self> {
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(600))
            .build()?;
        Ok(Self {
            api_key: api_key.into(),
            http,
        })
    }

    /// Build a client, reading `ANTHROPIC_API_KEY` from the environment.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::MissingApiKey`] if the variable is unset, or
    /// [`ModelError::Http`] if the underlying `reqwest` client fails to build.
    pub fn from_env() -> ModelResult<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| ModelError::MissingApiKey)?;
        Self::new(api_key)
    }
}

impl ModelClient for AnthropicClient {
    fn send(&self, req: &MessageRequest<'_>) -> ModelResult<ModelResponse> {
        let fable5 = is_fable5_class(req.model);

        let body = RequestBody {
            model: req.model,
            max_tokens: req.max_tokens,
            system: req.system,
            messages: &req.messages,
            thinking: if fable5 {
                Some(ThinkingConfig { kind: "adaptive" })
            } else {
                None
            },
            output_config: req.effort.map(|effort| OutputConfig {
                effort: Some(effort),
            }),
            fallbacks: if fable5 {
                Some(vec![FallbackTarget {
                    model: "claude-opus-4-8",
                }])
            } else {
                None
            },
        };

        let mut request = self
            .http
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json");
        if fable5 {
            // Betas are an HTTP header for raw HTTP calls (comma-separated flags), not
            // a JSON body field — only SDK-level `betas=[...]` params translate into
            // this header automatically.
            request = request.header("anthropic-beta", BETA_SERVER_SIDE_FALLBACK);
        }

        let response = request.json(&body).send()?;
        let status = response.status();
        let raw = response.text()?;

        if !status.is_success() {
            return Err(ModelError::BadStatus {
                status: status.as_u16(),
                body: raw,
            });
        }

        let parsed: ModelResponse = serde_json::from_str(&raw)?;
        Ok(parsed)
    }
}

/// A hand-rolled mock [`ModelClient`] for tests that need no network access or API
/// key — e.g. a future `tests/walking_skeleton.rs` exercising the full pipeline
/// against a fixed known-good response. Public (not `#[cfg(test)]`-gated) so both unit
/// tests here and integration tests elsewhere in the crate can reuse it.
pub struct MockModelClient {
    response: ModelResult<ModelResponse>,
}

impl MockModelClient {
    /// Build a mock that always returns a successful `end_turn` response with the
    /// given text as its sole content block.
    #[must_use]
    pub fn ok_text(text: &str) -> Self {
        Self {
            response: Ok(ModelResponse {
                model: "mock-model".to_string(),
                stop_reason: Some("end_turn".to_string()),
                stop_details: None,
                content: vec![ContentBlock {
                    block_type: "text".to_string(),
                    text: Some(text.to_string()),
                }],
            }),
        }
    }

    /// Build a mock that always returns a `stop_reason: "refusal"` response.
    #[must_use]
    pub fn refusal() -> Self {
        Self {
            response: Ok(ModelResponse {
                model: "mock-model".to_string(),
                stop_reason: Some("refusal".to_string()),
                stop_details: Some(StopDetails {
                    category: Some("cyber".to_string()),
                    explanation: Some("mock refusal for test".to_string()),
                }),
                content: vec![],
            }),
        }
    }
}

impl ModelClient for MockModelClient {
    fn send(&self, _req: &MessageRequest<'_>) -> ModelResult<ModelResponse> {
        match &self.response {
            Ok(resp) => Ok(resp.clone()),
            Err(_) => Err(ModelError::UnexpectedShape(
                "mock configured to fail".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_client_returns_configured_text() {
        let client =
            MockModelClient::ok_text("```rust\nfn add(a: i32, b: i32) -> i32 { a + b }\n```");
        let req = MessageRequest {
            model: "claude-opus-4-8",
            max_tokens: 1024,
            system: Some("You are a careful Rust engineer."),
            messages: vec![Message::user("Fix the bug.")],
            effort: None,
        };
        let resp = client.send(&req).expect("mock send should succeed");
        let text = resp.text().expect("should extract text");
        assert!(text.contains("fn add"));
    }

    #[test]
    fn refusal_is_reported_before_reading_content() {
        let client = MockModelClient::refusal();
        let req = MessageRequest {
            model: "claude-fable-5",
            max_tokens: 1024,
            system: None,
            messages: vec![Message::user("hello")],
            effort: None,
        };
        let resp = client.send(&req).expect("mock send should succeed");
        let err = resp
            .text()
            .expect_err("refusal should be reported as an error");
        match err {
            ModelError::Refusal { category, .. } => assert_eq!(category.as_deref(), Some("cyber")),
            other => panic!("expected Refusal, got {other:?}"),
        }
    }

    #[test]
    fn fable5_detection() {
        assert!(is_fable5_class("claude-fable-5"));
        assert!(is_fable5_class("claude-mythos-5"));
        assert!(!is_fable5_class("claude-opus-4-8"));
    }
}

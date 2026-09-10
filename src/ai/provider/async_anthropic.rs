//! Async Anthropic Claude API client
//!
//! Implements async SSE streaming for the Anthropic Messages API with cancellation support.
//! Uses reqwest for HTTP and tokio for async runtime.

use std::sync::mpsc::Sender;

use futures::StreamExt;
use reqwest::Client;
use tokio_util::sync::CancellationToken;

use super::AiError;
use super::sse::{AnthropicEventParser, SseParser};
use crate::ai::ai_state::AiResponse;
use crate::ai::chat::{AiPrompt, ChatRole};
use crate::config::ai_types::AiEffort;

/// Anthropic API endpoint
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Anthropic API version header
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic beta flag that opts a request into the 1M-token context window.
const CONTEXT_1M_BETA: &str = "context-1m-2025-08-07";

/// Wire name of a conversation role in the Anthropic Messages API.
fn role_name(role: ChatRole) -> &'static str {
    match role {
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
    }
}

/// Async Anthropic Claude API client
///
/// Uses reqwest for async HTTP requests with streaming support.
/// Supports cancellation via CancellationToken.
#[derive(Debug, Clone)]
pub struct AsyncAnthropicClient {
    client: Client,
    api_key: String,
    model: String,
    max_tokens: u32,
    effort: Option<AiEffort>,
    context_1m: bool,
}

impl AsyncAnthropicClient {
    /// Create a new async Anthropic client
    pub fn new(api_key: String, model: String, max_tokens: u32) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
            max_tokens,
            effort: None,
            context_1m: false,
        }
    }

    /// Set the reasoning effort (Claude 4.6+). None leaves the model default.
    pub fn with_effort(mut self, effort: Option<AiEffort>) -> Self {
        self.effort = effort;
        self
    }

    /// Enable the 1M-token context window beta (Claude Sonnet 4 / 4.5).
    pub fn with_context_1m(mut self, context_1m: bool) -> Self {
        self.context_1m = context_1m;
        self
    }

    /// Build the request body JSON for the Anthropic Messages API.
    ///
    /// The system prompt goes in the top-level `system` field (omitted when
    /// empty) and each conversation turn becomes a `messages` entry with its
    /// `user` / `assistant` role.
    ///
    /// Effort rides Claude's adaptive-thinking shape:
    /// `{"thinking": {"type": "adaptive"}, "output_config": {"effort": "<level>"}}`.
    fn build_request_body(&self, prompt: &AiPrompt) -> Result<String, AiError> {
        let messages: Vec<serde_json::Value> = prompt
            .messages
            .iter()
            .map(|message| {
                serde_json::json!({
                    "role": role_name(message.role),
                    "content": message.content,
                })
            })
            .collect();

        let mut request_body = serde_json::json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "stream": true,
            "messages": messages,
        });

        if !prompt.system.is_empty() {
            request_body["system"] = serde_json::json!(prompt.system);
        }

        if let Some(effort) = self.effort {
            request_body["thinking"] = serde_json::json!({"type": "adaptive"});
            request_body["output_config"] = serde_json::json!({"effort": effort.as_str()});
        }

        serde_json::to_string(&request_body).map_err(|e| AiError::Parse {
            provider: "Anthropic".to_string(),
            message: e.to_string(),
        })
    }

    /// Apply a whole-request timeout (from `[ai] request_timeout_secs`).
    /// None leaves requests unbounded.
    pub fn with_timeout(mut self, timeout: Option<std::time::Duration>) -> Self {
        if let Some(duration) = timeout {
            self.client = Client::builder()
                .timeout(duration)
                .build()
                .unwrap_or_default();
        }
        self
    }

    /// Stream a response from the Anthropic API with cancellation support
    ///
    /// Uses `tokio::select!` to race the stream against the cancellation token.
    /// Sends chunks via the response channel as they arrive.
    ///
    /// # Arguments
    /// * `prompt` - System prompt plus conversation turns to send to the API
    /// * `request_id` - Unique ID for this request
    /// * `cancel_token` - Token to cancel the request
    /// * `response_tx` - Channel to send response chunks
    ///
    /// # Returns
    /// * `Ok(())` - Stream completed successfully
    /// * `Err(AiError::Cancelled)` - Request was cancelled
    /// * `Err(AiError::*)` - Other errors
    pub async fn stream_with_cancel(
        &self,
        prompt: &AiPrompt,
        request_id: u64,
        cancel_token: CancellationToken,
        response_tx: Sender<AiResponse>,
    ) -> Result<(), AiError> {
        // Check if already cancelled before starting
        if cancel_token.is_cancelled() {
            return Err(AiError::Cancelled);
        }

        let body = self.build_request_body(prompt)?;

        // Make the request
        let mut request = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json");

        // Opt into the 1M-token context window beta when configured
        if self.context_1m {
            request = request.header("anthropic-beta", CONTEXT_1M_BETA);
        }

        let response = request
            .body(body)
            .send()
            .await
            .map_err(|e| AiError::Network {
                provider: "Anthropic".to_string(),
                message: e.to_string(),
            })?;

        // Check for HTTP errors
        if !response.status().is_success() {
            let code = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiError::Api {
                provider: "Anthropic".to_string(),
                code,
                message,
            });
        }

        // Get the byte stream
        let mut stream = response.bytes_stream();
        let mut sse_parser = SseParser::new(AnthropicEventParser);

        // Process stream with cancellation support
        loop {
            tokio::select! {
                biased;

                // Check cancellation first (biased mode)
                _ = cancel_token.cancelled() => {
                    return Err(AiError::Cancelled);
                }

                // Process next chunk from stream
                chunk = stream.next() => {
                    match chunk {
                        Some(Ok(bytes)) => {
                            // Parse SSE events from bytes
                            for text in sse_parser.parse_chunk(&bytes) {
                                if response_tx
                                    .send(AiResponse::Chunk {
                                        text,
                                        request_id,
                                    })
                                    .is_err()
                                {
                                    // Main thread disconnected
                                    return Ok(());
                                }
                            }
                        }
                        Some(Err(e)) => {
                            return Err(AiError::Network {
                                provider: "Anthropic".to_string(),
                                message: e.to_string(),
                            });
                        }
                        None => {
                            // Stream ended
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "async_anthropic_tests.rs"]
mod async_anthropic_tests;

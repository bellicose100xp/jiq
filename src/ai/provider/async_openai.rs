//! Async OpenAI API client
//!
//! Implements async SSE streaming for the OpenAI Chat Completions API with cancellation support.
//! Uses reqwest for HTTP and tokio for async runtime.

use std::sync::mpsc::Sender;

use futures::StreamExt;
use reqwest::Client;
use serde::Serialize;
use tokio_util::sync::CancellationToken;

use super::AiError;
use super::sse::{OpenAiEventParser, SseParser};
use crate::ai::ai_state::AiResponse;

/// OpenAI API endpoint
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// Async OpenAI API client
///
/// Uses reqwest for async HTTP requests with streaming support.
/// Supports cancellation via CancellationToken.
#[derive(Debug, Clone)]
pub struct AsyncOpenAiClient {
    client: Client,
    api_key: String,
    model: String,
}

impl AsyncOpenAiClient {
    /// Create a new async OpenAI client
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }

    /// Build the request body JSON for OpenAI Chat Completions API
    ///
    /// Creates a JSON request body with the model, messages array, and streaming enabled.
    /// Does not set max_tokens, allowing OpenAI to use its default.
    ///
    /// # Arguments
    /// * `prompt` - The user prompt to send to the API
    ///
    /// # Returns
    /// * `Ok(String)` - Serialized JSON request body
    /// * `Err(AiError::Parse)` - If serialization fails
    fn build_request_body(&self, prompt: &str) -> Result<String, AiError> {
        #[derive(Serialize)]
        struct Message {
            role: String,
            content: String,
        }

        #[derive(Serialize)]
        struct RequestBody {
            model: String,
            messages: Vec<Message>,
            stream: bool,
        }

        let body = RequestBody {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            stream: true,
        };

        serde_json::to_string(&body).map_err(|e| AiError::Parse {
            provider: "OpenAI".to_string(),
            message: format!("Failed to serialize request body: {}", e),
        })
    }

    /// Stream a response from the OpenAI API with cancellation support
    ///
    /// Uses `tokio::select!` to race the stream against the cancellation token.
    /// Sends chunks via the response channel as they arrive.
    ///
    /// # Arguments
    /// * `prompt` - The prompt to send to the API
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
        prompt: &str,
        request_id: u64,
        cancel_token: CancellationToken,
        response_tx: Sender<AiResponse>,
    ) -> Result<(), AiError> {
        // Check if already cancelled before starting
        if cancel_token.is_cancelled() {
            return Err(AiError::Cancelled);
        }

        // Build request body
        let body = self.build_request_body(prompt)?;

        // Make the POST request to OpenAI API
        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| AiError::Network {
                provider: "OpenAI".to_string(),
                message: e.to_string(),
            })?;

        // Check HTTP status and return AiError::Api for errors
        if !response.status().is_success() {
            let code = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiError::Api {
                provider: "OpenAI".to_string(),
                code,
                message,
            });
        }

        // Get byte stream from response
        let mut stream = response.bytes_stream();
        let mut sse_parser = SseParser::new(OpenAiEventParser);

        // Use tokio::select! with biased mode to race stream against cancellation
        loop {
            tokio::select! {
                biased;

                // Check cancellation first (biased mode prioritizes this)
                _ = cancel_token.cancelled() => {
                    log::debug!("Request {} cancelled during streaming", request_id);
                    return Err(AiError::Cancelled);
                }

                // Process next chunk from stream
                chunk = stream.next() => {
                    match chunk {
                        Some(Ok(bytes)) => {
                            // Parse chunks and send via response_tx as AiResponse::Chunk
                            for text in sse_parser.parse_chunk(&bytes) {
                                if response_tx
                                    .send(AiResponse::Chunk {
                                        text,
                                        request_id,
                                    })
                                    .is_err()
                                {
                                    // Main thread disconnected - stop streaming gracefully
                                    return Ok(());
                                }
                            }
                        }
                        Some(Err(e)) => {
                            // Return AiError::Network for stream errors
                            return Err(AiError::Network {
                                provider: "OpenAI".to_string(),
                                message: e.to_string(),
                            });
                        }
                        None => {
                            // Stream ended - return Ok(()) on successful completion
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
#[path = "async_openai_tests.rs"]
mod async_openai_tests;

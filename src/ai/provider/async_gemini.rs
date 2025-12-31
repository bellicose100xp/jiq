//! Async Gemini API client
//!
//! Implements async SSE streaming for the Google Generative Language API with cancellation support.
//! Uses reqwest for HTTP and tokio for async runtime.

use std::sync::mpsc::Sender;

use futures::StreamExt;
use reqwest::Client;
use serde::Serialize;
use tokio_util::sync::CancellationToken;

use super::AiError;
use super::sse::{GeminiEventParser, SseParser};
use crate::ai::ai_state::AiResponse;

/// Gemini API endpoint
const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// Async Gemini API client
///
/// Uses reqwest for async HTTP requests with streaming support.
/// Supports cancellation via CancellationToken.
#[derive(Debug, Clone)]
pub struct AsyncGeminiClient {
    client: Client,
    api_key: String,
    model: String,
}

impl AsyncGeminiClient {
    /// Create a new async Gemini client
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }

    /// Returns the stored API key (used in tests)
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Returns the stored model (used in tests)
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Build the request body JSON for Gemini API
    ///
    /// Creates a JSON request body with the contents array containing user role and parts.
    /// Gemini uses query parameters for streaming, not a body field.
    ///
    /// # Arguments
    /// * `prompt` - The user prompt to send to the API
    ///
    /// # Returns
    /// * `Ok(String)` - Serialized JSON request body
    /// * `Err(AiError::Parse)` - If serialization fails
    fn build_request_body(&self, prompt: &str) -> Result<String, AiError> {
        #[derive(Serialize)]
        struct Part {
            text: String,
        }

        #[derive(Serialize)]
        struct Content {
            role: String,
            parts: Vec<Part>,
        }

        #[derive(Serialize)]
        struct RequestBody {
            contents: Vec<Content>,
        }

        let body = RequestBody {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
        };

        serde_json::to_string(&body).map_err(|e| AiError::Parse {
            provider: "Gemini".to_string(),
            message: format!("Failed to serialize request body: {}", e),
        })
    }

    /// Build the URL for Gemini streaming API
    ///
    /// Constructs URL: `{GEMINI_API_URL}/{model}:streamGenerateContent?alt=sse&key={api_key}`
    fn build_url(&self) -> String {
        format!(
            "{}/{}:streamGenerateContent?alt=sse&key={}",
            GEMINI_API_URL, self.model, self.api_key
        )
    }

    /// Stream a response from the Gemini API with cancellation support
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

        // Build request body and URL
        let body = self.build_request_body(prompt)?;
        let url = self.build_url();

        // Make the POST request to Gemini API
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| AiError::Network {
                provider: "Gemini".to_string(),
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
                provider: "Gemini".to_string(),
                code,
                message,
            });
        }

        // Get byte stream from response
        let mut stream = response.bytes_stream();
        let mut sse_parser = SseParser::new(GeminiEventParser);

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
                                provider: "Gemini".to_string(),
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
#[path = "async_gemini_tests.rs"]
mod async_gemini_tests;

//! Async Anthropic Claude API client
//!
//! Implements async SSE streaming for the Anthropic Messages API with cancellation support.
//! Uses reqwest for HTTP and tokio for async runtime.

use std::sync::mpsc::Sender;

use bytes::Bytes;
use futures::StreamExt;
use reqwest::Client;
use tokio_util::sync::CancellationToken;

use super::AiError;
use crate::ai::ai_state::AiResponse;

/// Anthropic API endpoint
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Anthropic API version header
const ANTHROPIC_VERSION: &str = "2023-06-01";

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
}

impl AsyncAnthropicClient {
    /// Create a new async Anthropic client
    pub fn new(api_key: String, model: String, max_tokens: u32) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
            max_tokens,
        }
    }

    /// Stream a response from the Anthropic API with cancellation support
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

        let request_body = serde_json::json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "stream": true,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let body =
            serde_json::to_string(&request_body).map_err(|e| AiError::Parse(e.to_string()))?;

        // Make the request
        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| AiError::Network(e.to_string()))?;

        // Check for HTTP errors
        if !response.status().is_success() {
            let code = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiError::Api { code, message });
        }

        // Get the byte stream
        let mut stream = response.bytes_stream();
        let mut sse_parser = SseParser::new();

        // Process stream with cancellation support
        loop {
            tokio::select! {
                biased;

                // Check cancellation first (biased mode)
                _ = cancel_token.cancelled() => {
                    log::debug!("Request {} cancelled during streaming", request_id);
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
                            return Err(AiError::Network(e.to_string()));
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

/// SSE (Server-Sent Events) parser for async streaming
///
/// Buffers incoming bytes and extracts complete SSE events.
/// Parses `content_block_delta` events to extract text chunks.
struct SseParser {
    buffer: String,
}

impl SseParser {
    fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Parse a chunk of bytes and return any complete text chunks
    ///
    /// Buffers incomplete lines and processes complete SSE events.
    fn parse_chunk(&mut self, bytes: &Bytes) -> Vec<String> {
        let mut results = Vec::new();

        // Append new bytes to buffer
        if let Ok(text) = std::str::from_utf8(bytes) {
            self.buffer.push_str(text);
        } else {
            // Invalid UTF-8, skip this chunk
            return results;
        }

        // Process complete lines
        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = self.buffer[..newline_pos].trim().to_string();
            self.buffer = self.buffer[newline_pos + 1..].to_string();

            // Skip empty lines and event type lines
            if line.is_empty() || line.starts_with("event:") {
                continue;
            }

            // Handle data lines
            if let Some(data) = line.strip_prefix("data: ") {
                // Check for stream end
                if data == "[DONE]" {
                    continue;
                }

                // Try to parse as content_block_delta
                if let Some(text) = Self::parse_delta_text(data) {
                    if !text.is_empty() {
                        results.push(text);
                    }
                }
            }
        }

        results
    }

    /// Parse a content_block_delta event and extract the text
    fn parse_delta_text(data: &str) -> Option<String> {
        let json: serde_json::Value = serde_json::from_str(data).ok()?;

        // Check if this is a content_block_delta event
        if json.get("type")?.as_str()? != "content_block_delta" {
            return None;
        }

        // Extract text from delta.text
        json.get("delta")?
            .get("text")?
            .as_str()
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
#[path = "async_anthropic_tests.rs"]
mod async_anthropic_tests;

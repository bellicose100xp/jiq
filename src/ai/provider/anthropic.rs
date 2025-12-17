//! Anthropic Claude API client
//!
//! Implements SSE streaming for the Anthropic Messages API.

use std::io::{BufRead, BufReader};

use super::AiError;

/// Anthropic API endpoint
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Anthropic API version header
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic Claude API client
#[derive(Debug)]
pub struct AnthropicClient {
    api_key: String,
    model: String,
    max_tokens: u32,
}

impl AnthropicClient {
    /// Create a new Anthropic client
    pub fn new(api_key: String, model: String, max_tokens: u32) -> Self {
        Self {
            api_key,
            model,
            max_tokens,
        }
    }

    /// Stream a response from the Anthropic API
    ///
    /// Returns an iterator that yields text chunks as they arrive via SSE
    pub fn stream(
        &self,
        prompt: &str,
    ) -> Result<Box<dyn Iterator<Item = Result<String, AiError>> + '_>, AiError> {
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

        let response = ureq::post(ANTHROPIC_API_URL)
            .set("x-api-key", &self.api_key)
            .set("anthropic-version", ANTHROPIC_VERSION)
            .set("content-type", "application/json")
            .send_string(&body)
            .map_err(|e| match e {
                ureq::Error::Status(code, response) => {
                    let message = response
                        .into_string()
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    AiError::Api { code, message }
                }
                ureq::Error::Transport(t) => AiError::Network(t.to_string()),
            })?;

        let reader = BufReader::new(response.into_reader());
        Ok(Box::new(SseIterator::new(reader)))
    }
}

/// Iterator over SSE events from the Anthropic API
struct SseIterator<R: BufRead> {
    reader: R,
    buffer: String,
    done: bool,
}

impl<R: BufRead> SseIterator<R> {
    fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: String::new(),
            done: false,
        }
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

impl<R: BufRead> Iterator for SseIterator<R> {
    type Item = Result<String, AiError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        loop {
            self.buffer.clear();
            match self.reader.read_line(&mut self.buffer) {
                Ok(0) => {
                    // EOF
                    self.done = true;
                    return None;
                }
                Ok(_) => {
                    let line = self.buffer.trim();

                    // Skip empty lines and event type lines
                    if line.is_empty() || line.starts_with("event:") {
                        continue;
                    }

                    // Handle data lines
                    if let Some(data) = line.strip_prefix("data: ") {
                        // Check for stream end
                        if data == "[DONE]" {
                            self.done = true;
                            return None;
                        }

                        // Try to parse as content_block_delta
                        if let Some(text) = Self::parse_delta_text(data)
                            && !text.is_empty()
                        {
                            return Some(Ok(text));
                        }
                        // Continue to next line if not a text delta
                        continue;
                    }

                    // Skip other lines
                    continue;
                }
                Err(e) => {
                    self.done = true;
                    return Some(Err(AiError::Network(e.to_string())));
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "anthropic_tests.rs"]
mod anthropic_tests;

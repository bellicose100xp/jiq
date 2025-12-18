//! Shared SSE (Server-Sent Events) parsing module
//!
//! Provides a generic SSE parser that handles buffering and line splitting,
//! with provider-specific JSON extraction via the SseEventParser trait.

use bytes::Bytes;

/// Provider-specific JSON extractor trait
///
/// Implementations define how to extract text content from SSE data lines
/// for different AI provider APIs (Anthropic, OpenAI, etc.).
pub trait SseEventParser: Send + Sync {
    /// Extract text content from an SSE data line
    ///
    /// Returns None if the event doesn't contain text or parsing fails.
    fn parse_data(&self, data: &str) -> Option<String>;

    /// Check if this event signals the end of the stream
    ///
    /// Most providers use `[DONE]` as the end-of-stream marker.
    fn is_done(&self, data: &str) -> bool;
}

/// Generic SSE line buffer and event splitter
///
/// Handles buffering of incomplete lines across chunks and splits
/// complete lines for processing by provider-specific parsers.
pub struct SseParser<P: SseEventParser> {
    buffer: String,
    parser: P,
}

impl<P: SseEventParser> SseParser<P> {
    /// Create a new SSE parser with a provider-specific event parser
    pub fn new(parser: P) -> Self {
        Self {
            buffer: String::new(),
            parser,
        }
    }

    /// Parse a chunk of bytes and return extracted text
    ///
    /// Buffers incomplete lines across calls. Filters out empty lines,
    /// whitespace-only lines, and `event:` type lines. Extracts `data:`
    /// prefix and passes content to the parser trait.
    ///
    /// # Arguments
    /// * `bytes` - Raw bytes from the HTTP stream
    ///
    /// # Returns
    /// Vector of extracted text chunks (may be empty)
    pub fn parse_chunk(&mut self, bytes: &Bytes) -> Vec<String> {
        let mut results = Vec::new();

        // Handle invalid UTF-8 gracefully by skipping malformed chunks
        if let Ok(text) = std::str::from_utf8(bytes) {
            self.buffer.push_str(text);
        } else {
            // Invalid UTF-8, skip this chunk
            return results;
        }

        // Process complete lines (split on newlines)
        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = self.buffer[..newline_pos].trim().to_string();
            self.buffer = self.buffer[newline_pos + 1..].to_string();

            // Filter out empty lines and whitespace-only lines
            if line.is_empty() {
                continue;
            }

            // Filter out event type lines
            if line.starts_with("event:") {
                continue;
            }

            // Handle data lines
            if let Some(data) = line.strip_prefix("data: ") {
                // Check for stream end
                if self.parser.is_done(data) {
                    continue;
                }

                // Extract text using provider-specific parser
                if let Some(text) = self.parser.parse_data(data)
                    && !text.is_empty()
                {
                    results.push(text);
                }
            }
        }

        results
    }
}

/// Anthropic-specific SSE parser
///
/// Parses Anthropic's content_block_delta events:
/// `{"type":"content_block_delta","delta":{"text":"..."}}`
pub struct AnthropicEventParser;

impl SseEventParser for AnthropicEventParser {
    fn parse_data(&self, data: &str) -> Option<String> {
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

    fn is_done(&self, data: &str) -> bool {
        data == "[DONE]"
    }
}

/// OpenAI-specific SSE parser
///
/// Parses OpenAI's chat completion delta events:
/// `{"choices":[{"delta":{"content":"..."}}]}`
pub struct OpenAiEventParser;

impl SseEventParser for OpenAiEventParser {
    fn parse_data(&self, data: &str) -> Option<String> {
        let json: serde_json::Value = serde_json::from_str(data).ok()?;

        // Extract text from choices[0].delta.content
        json.get("choices")?
            .get(0)?
            .get("delta")?
            .get("content")?
            .as_str()
            .map(|s| s.to_string())
    }

    fn is_done(&self, data: &str) -> bool {
        data == "[DONE]"
    }
}

#[cfg(test)]
#[path = "sse_tests.rs"]
mod sse_tests;

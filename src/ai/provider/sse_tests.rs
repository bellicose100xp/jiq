//! Tests for shared SSE parsing module

use super::*;
use bytes::Bytes;
use proptest::prelude::*;

// ============================================================================
// Unit Tests
// ============================================================================

#[test]
fn test_anthropic_parser_valid_delta() {
    let parser = AnthropicEventParser;
    let data =
        r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
    let result = parser.parse_data(data);
    assert_eq!(result, Some("Hello".to_string()));
}

#[test]
fn test_anthropic_parser_not_delta() {
    let parser = AnthropicEventParser;
    let data = r#"{"type":"message_start","message":{"id":"msg_123"}}"#;
    let result = parser.parse_data(data);
    assert_eq!(result, None);
}

#[test]
fn test_anthropic_parser_invalid_json() {
    let parser = AnthropicEventParser;
    let data = "not valid json";
    let result = parser.parse_data(data);
    assert_eq!(result, None);
}

#[test]
fn test_anthropic_parser_is_done() {
    let parser = AnthropicEventParser;
    assert!(parser.is_done("[DONE]"));
    assert!(!parser.is_done("other"));
}

#[test]
fn test_openai_parser_valid_delta() {
    let parser = OpenAiEventParser;
    let data = r#"{"choices":[{"delta":{"content":"Hello"}}]}"#;
    let result = parser.parse_data(data);
    assert_eq!(result, Some("Hello".to_string()));
}

#[test]
fn test_openai_parser_no_content() {
    let parser = OpenAiEventParser;
    let data = r#"{"choices":[{"delta":{}}]}"#;
    let result = parser.parse_data(data);
    assert_eq!(result, None);
}

#[test]
fn test_openai_parser_invalid_json() {
    let parser = OpenAiEventParser;
    let data = "not valid json";
    let result = parser.parse_data(data);
    assert_eq!(result, None);
}

#[test]
fn test_openai_parser_is_done() {
    let parser = OpenAiEventParser;
    assert!(parser.is_done("[DONE]"));
    assert!(!parser.is_done("other"));
}

#[test]
fn test_sse_parser_single_event() {
    let mut parser = SseParser::new(AnthropicEventParser);
    let data = b"data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hello\"}}\n";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert_eq!(results, vec!["Hello".to_string()]);
}

#[test]
fn test_sse_parser_multiple_events() {
    let mut parser = SseParser::new(AnthropicEventParser);
    let data = b"data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hello\"}}\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\" World\"}}\n";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert_eq!(results, vec!["Hello".to_string(), " World".to_string()]);
}

#[test]
fn test_sse_parser_filters_empty_lines() {
    let mut parser = SseParser::new(AnthropicEventParser);
    let data = b"\n\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hello\"}}\n\n";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert_eq!(results, vec!["Hello".to_string()]);
}

#[test]
fn test_sse_parser_filters_whitespace_lines() {
    let mut parser = SseParser::new(AnthropicEventParser);
    let data =
        b"   \n\t\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hello\"}}\n  \n";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert_eq!(results, vec!["Hello".to_string()]);
}

#[test]
fn test_sse_parser_filters_event_lines() {
    let mut parser = SseParser::new(AnthropicEventParser);
    let data = b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hello\"}}\n";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert_eq!(results, vec!["Hello".to_string()]);
}

#[test]
fn test_sse_parser_handles_done() {
    let mut parser = SseParser::new(AnthropicEventParser);
    let data =
        b"data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Test\"}}\ndata: [DONE]\n";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert_eq!(results, vec!["Test".to_string()]);
}

#[test]
fn test_sse_parser_buffers_incomplete_lines() {
    let mut parser = SseParser::new(AnthropicEventParser);

    // First chunk: incomplete line
    let data1 = b"data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hel";
    let results1 = parser.parse_chunk(&Bytes::from_static(data1));
    assert!(results1.is_empty()); // No complete event yet

    // Second chunk: completes the line
    let data2 = b"lo\"}}\n";
    let results2 = parser.parse_chunk(&Bytes::from_static(data2));
    assert_eq!(results2, vec!["Hello".to_string()]);
}

#[test]
fn test_sse_parser_skips_empty_text() {
    let mut parser = SseParser::new(AnthropicEventParser);
    let data = b"data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"\"}}\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Real\"}}\n";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert_eq!(results, vec!["Real".to_string()]);
}

#[test]
fn test_sse_parser_invalid_utf8() {
    let mut parser = SseParser::new(AnthropicEventParser);
    // Invalid UTF-8 sequence
    let data = &[0xFF, 0xFE, 0xFD];
    let results = parser.parse_chunk(&Bytes::from(data.to_vec()));
    assert!(results.is_empty());
}

// ============================================================================
// Property-Based Tests
// ============================================================================

// **Feature: openai-provider, Property 13: SSE line splitting**
// *For any* sequence of bytes containing newline characters, the SseParser
// should split on newlines and process each line independently, maintaining
// buffer state across chunks.
// **Validates: Requirements 2.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_sse_line_splitting(
        text1 in "[a-zA-Z0-9 ]{1,20}",
        text2 in "[a-zA-Z0-9 ]{1,20}",
        text3 in "[a-zA-Z0-9 ]{1,20}",
    ) {
        let mut parser = SseParser::new(AnthropicEventParser);

        // Create SSE data with multiple lines
        let data = format!(
            "data: {{\"type\":\"content_block_delta\",\"delta\":{{\"text\":\"{}\"}}}}\ndata: {{\"type\":\"content_block_delta\",\"delta\":{{\"text\":\"{}\"}}}}\ndata: {{\"type\":\"content_block_delta\",\"delta\":{{\"text\":\"{}\"}}}}\n",
            text1, text2, text3
        );

        let results = parser.parse_chunk(&Bytes::from(data));

        // Should extract all three text chunks
        prop_assert_eq!(results.len(), 3);
        prop_assert_eq!(&results[0], &text1);
        prop_assert_eq!(&results[1], &text2);
        prop_assert_eq!(&results[2], &text3);
    }
}

// **Feature: openai-provider, Property 14: Empty and whitespace line handling**
// *For any* SSE stream containing empty lines or lines with only whitespace,
// the parser should skip these lines and continue processing valid data lines.
// **Validates: Requirements 2.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_empty_and_whitespace_line_handling(
        text in "[a-zA-Z0-9 ]{1,20}",
        num_empty_before in 0..5usize,
        num_empty_after in 0..5usize,
        num_whitespace in 0..5usize,
    ) {
        let mut parser = SseParser::new(AnthropicEventParser);

        // Build data with empty lines before, whitespace lines, and empty lines after
        let mut data = String::new();

        // Add empty lines before
        for _ in 0..num_empty_before {
            data.push('\n');
        }

        // Add whitespace lines
        for _ in 0..num_whitespace {
            data.push_str("   \n");
        }

        // Add valid data line
        data.push_str(&format!(
            "data: {{\"type\":\"content_block_delta\",\"delta\":{{\"text\":\"{}\"}}}}\n",
            text
        ));

        // Add empty lines after
        for _ in 0..num_empty_after {
            data.push('\n');
        }

        let results = parser.parse_chunk(&Bytes::from(data));

        // Should extract only the valid text, skipping empty and whitespace lines
        prop_assert_eq!(results.len(), 1);
        prop_assert_eq!(&results[0], &text);
    }
}

// **Feature: openai-provider, Property 15: Event type line filtering**
// *For any* SSE stream containing `event:` type lines, the parser should
// skip these lines and only process `data:` lines.
// **Validates: Requirements 2.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_event_type_line_filtering(
        text in "[a-zA-Z0-9 ]{1,20}",
        event_type in "[a-z_]{5,15}",
        num_event_lines in 0..5usize,
    ) {
        let mut parser = SseParser::new(AnthropicEventParser);

        // Build data with event type lines
        let mut data = String::new();

        // Add event type lines
        for _ in 0..num_event_lines {
            data.push_str(&format!("event: {}\n", event_type));
        }

        // Add valid data line
        data.push_str(&format!(
            "data: {{\"type\":\"content_block_delta\",\"delta\":{{\"text\":\"{}\"}}}}\n",
            text
        ));

        let results = parser.parse_chunk(&Bytes::from(data));

        // Should extract only the data line, skipping event type lines
        prop_assert_eq!(results.len(), 1);
        prop_assert_eq!(&results[0], &text);
    }
}

// **Feature: openai-provider, Property 16: Invalid UTF-8 handling**
// *For any* byte chunk containing invalid UTF-8 sequences, the parser should
// skip that chunk and continue processing subsequent valid chunks without crashing.
// **Validates: Requirements 2.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_invalid_utf8_handling(
        text in "[a-zA-Z0-9 ]{1,20}",
        invalid_byte in 0x80u8..0xFFu8,
    ) {
        let mut parser = SseParser::new(AnthropicEventParser);

        // First chunk: invalid UTF-8
        let invalid_data = vec![invalid_byte, 0xFF, 0xFE];
        let results1 = parser.parse_chunk(&Bytes::from(invalid_data));

        // Should skip invalid UTF-8 without crashing
        prop_assert!(results1.is_empty());

        // Second chunk: valid UTF-8
        let valid_data = format!(
            "data: {{\"type\":\"content_block_delta\",\"delta\":{{\"text\":\"{}\"}}}}\n",
            text
        );
        let results2 = parser.parse_chunk(&Bytes::from(valid_data));

        // Should process valid chunk successfully
        prop_assert_eq!(results2.len(), 1);
        prop_assert_eq!(&results2[0], &text);
    }
}

// **Feature: openai-provider, Property 4: SSE buffering consistency**
// *For any* sequence of byte chunks that together form valid SSE events,
// the SseParser should extract the same text chunks regardless of how
// the bytes are split across chunks.
// **Validates: Requirements 2.1, 2.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_sse_buffering_consistency(
        text1 in "[a-zA-Z0-9 ]{1,20}",
        text2 in "[a-zA-Z0-9 ]{1,20}",
        split_point in 1..50usize,
    ) {
        // Create complete SSE data
        let complete_data = format!(
            "data: {{\"type\":\"content_block_delta\",\"delta\":{{\"text\":\"{}\"}}}}\ndata: {{\"type\":\"content_block_delta\",\"delta\":{{\"text\":\"{}\"}}}}\n",
            text1, text2
        );

        // Parse as single chunk
        let mut parser1 = SseParser::new(AnthropicEventParser);
        let results1 = parser1.parse_chunk(&Bytes::from(complete_data.clone()));

        // Parse split at arbitrary point
        let mut parser2 = SseParser::new(AnthropicEventParser);
        let split_idx = split_point.min(complete_data.len());
        let chunk1 = &complete_data[..split_idx];
        let chunk2 = &complete_data[split_idx..];

        let mut results2 = parser2.parse_chunk(&Bytes::from(chunk1.to_string()));
        results2.extend(parser2.parse_chunk(&Bytes::from(chunk2.to_string())));

        // Should extract same text regardless of chunking
        prop_assert_eq!(results1, results2);
    }
}

// **Feature: openai-provider, Property 5: Provider-specific parsing**
// *For any* valid OpenAI SSE data line containing text, the OpenAiEventParser
// should extract the text from the choices[0].delta.content field.
// **Validates: Requirements 2.2, 2.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_provider_specific_parsing(
        text in "[a-zA-Z0-9 ]{1,50}",
    ) {
        // Test OpenAI parser
        let openai_parser = OpenAiEventParser;
        let openai_data = format!(
            r#"{{"choices":[{{"delta":{{"content":"{}"}}}}]}}"#,
            text
        );
        let openai_result = openai_parser.parse_data(&openai_data);
        prop_assert_eq!(openai_result, Some(text.clone()));

        // Test Anthropic parser
        let anthropic_parser = AnthropicEventParser;
        let anthropic_data = format!(
            r#"{{"type":"content_block_delta","delta":{{"text":"{}"}}}}"#,
            text
        );
        let anthropic_result = anthropic_parser.parse_data(&anthropic_data);
        prop_assert_eq!(anthropic_result, Some(text));
    }
}

// ============================================================================
// Snapshot Tests
// ============================================================================

#[test]
fn snapshot_parsed_openai_sse_stream() {
    let mut parser = SseParser::new(OpenAiEventParser);

    // Simulate a complete OpenAI SSE stream
    let stream_data = r#"data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4o-mini","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4o-mini","choices":[{"index":0,"delta":{"content":" world"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4o-mini","choices":[{"index":0,"delta":{"content":"!"},"finish_reason":null}]}

data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4o-mini","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
"#;

    let results = parser.parse_chunk(&Bytes::from(stream_data));
    insta::assert_debug_snapshot!(results);
}

#[test]
fn snapshot_parsed_anthropic_sse_stream() {
    let mut parser = SseParser::new(AnthropicEventParser);

    // Simulate a complete Anthropic SSE stream
    let stream_data = r#"event: message_start
data: {"type":"message_start","message":{"id":"msg_123","type":"message","role":"assistant"}}

event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" world"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"!"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"}}

event: message_stop
data: {"type":"message_stop"}

data: [DONE]
"#;

    let results = parser.parse_chunk(&Bytes::from(stream_data));
    insta::assert_debug_snapshot!(results);
}

#[test]
fn snapshot_malformed_json_handling() {
    let mut openai_parser = SseParser::new(OpenAiEventParser);
    let mut anthropic_parser = SseParser::new(AnthropicEventParser);

    // Stream with malformed JSON
    let malformed_data = r#"data: {"choices":[{"delta":{"content":"Valid"}}]}
data: {this is not valid json}
data: {"choices":[{"delta":{"content":"Also valid"}}]}
data: {"incomplete":
data: {"choices":[{"delta":{"content":"Still works"}}]}
"#;

    let openai_results = openai_parser.parse_chunk(&Bytes::from(malformed_data));
    let anthropic_results = anthropic_parser.parse_chunk(&Bytes::from(malformed_data));

    insta::assert_debug_snapshot!("openai_malformed", openai_results);
    insta::assert_debug_snapshot!("anthropic_malformed", anthropic_results);
}

#[test]
fn snapshot_mixed_valid_invalid_events() {
    let mut parser = SseParser::new(OpenAiEventParser);

    // Mix of valid events, invalid JSON, empty content, and other event types
    let mixed_data = r#"event: ping
data: {"type":"ping"}

data: {"choices":[{"delta":{"content":"First"}}]}

data: not json at all

data: {"choices":[{"delta":{}}]}

data: {"choices":[{"delta":{"content":""}}]}

event: error
data: {"error":"something went wrong"}

data: {"choices":[{"delta":{"content":"Second"}}]}

data: {"wrong":"structure"}

data: {"choices":[{"delta":{"content":"Third"}}]}

data: [DONE]
"#;

    let results = parser.parse_chunk(&Bytes::from(mixed_data));
    insta::assert_debug_snapshot!(results);
}

//! Tests for Anthropic Claude API client

use super::*;
use std::io::Cursor;

#[test]
fn test_parse_delta_text_valid() {
    let data =
        r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
    let result = SseIterator::<Cursor<&[u8]>>::parse_delta_text(data);
    assert_eq!(result, Some("Hello".to_string()));
}

#[test]
fn test_parse_delta_text_not_delta() {
    let data = r#"{"type":"message_start","message":{"id":"msg_123"}}"#;
    let result = SseIterator::<Cursor<&[u8]>>::parse_delta_text(data);
    assert_eq!(result, None);
}

#[test]
fn test_parse_delta_text_invalid_json() {
    let data = "not valid json";
    let result = SseIterator::<Cursor<&[u8]>>::parse_delta_text(data);
    assert_eq!(result, None);
}

#[test]
fn test_sse_iterator_parses_chunks() {
    let sse_data = r#"event: message_start
data: {"type":"message_start","message":{"id":"msg_123"}}

event: content_block_start
data: {"type":"content_block_start","index":0}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" World"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_stop
data: {"type":"message_stop"}

"#;
    let reader = Cursor::new(sse_data.as_bytes());
    let mut iter = SseIterator::new(reader);

    let chunk1 = iter.next();
    assert!(chunk1.is_some());
    assert_eq!(chunk1.unwrap().unwrap(), "Hello");

    let chunk2 = iter.next();
    assert!(chunk2.is_some());
    assert_eq!(chunk2.unwrap().unwrap(), " World");

    // No more text chunks
    assert!(iter.next().is_none());
}

#[test]
fn test_sse_iterator_handles_done() {
    let sse_data = r#"event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Test"}}

data: [DONE]
"#;
    let reader = Cursor::new(sse_data.as_bytes());
    let mut iter = SseIterator::new(reader);

    let chunk = iter.next();
    assert!(chunk.is_some());
    assert_eq!(chunk.unwrap().unwrap(), "Test");

    // [DONE] should end the stream
    assert!(iter.next().is_none());
}

#[test]
fn test_sse_iterator_empty_stream() {
    let sse_data = "";
    let reader = Cursor::new(sse_data.as_bytes());
    let mut iter = SseIterator::new(reader);

    assert!(iter.next().is_none());
}

#[test]
fn test_sse_iterator_skips_empty_text() {
    let sse_data = r#"event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Real content"}}

"#;
    let reader = Cursor::new(sse_data.as_bytes());
    let mut iter = SseIterator::new(reader);

    // Should skip empty text and return "Real content"
    let chunk = iter.next();
    assert!(chunk.is_some());
    assert_eq!(chunk.unwrap().unwrap(), "Real content");
}

#[test]
fn test_anthropic_client_new() {
    let client = AnthropicClient::new(
        "sk-ant-test".to_string(),
        "claude-3-haiku".to_string(),
        1024,
    );
    // Just verify it creates without panic
    assert!(format!("{:?}", client).contains("AnthropicClient"));
}

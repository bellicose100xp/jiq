//! Tests for Async Anthropic Claude API client

use super::*;
use proptest::prelude::*;
use std::sync::mpsc;

#[test]
fn test_async_anthropic_client_new() {
    let client = AsyncAnthropicClient::new(
        "sk-ant-test".to_string(),
        "claude-3-haiku".to_string(),
        1024,
    );
    // Verify it creates without panic
    assert!(format!("{:?}", client).contains("AsyncAnthropicClient"));
}

#[test]
fn test_sse_parser_parse_delta_text_valid() {
    let data =
        r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
    let result = SseParser::parse_delta_text(data);
    assert_eq!(result, Some("Hello".to_string()));
}

#[test]
fn test_sse_parser_parse_delta_text_not_delta() {
    let data = r#"{"type":"message_start","message":{"id":"msg_123"}}"#;
    let result = SseParser::parse_delta_text(data);
    assert_eq!(result, None);
}

#[test]
fn test_sse_parser_parse_delta_text_invalid_json() {
    let data = "not valid json";
    let result = SseParser::parse_delta_text(data);
    assert_eq!(result, None);
}

#[test]
fn test_sse_parser_parse_chunk_single_event() {
    let mut parser = SseParser::new();
    let data = b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert_eq!(results, vec!["Hello".to_string()]);
}

#[test]
fn test_sse_parser_parse_chunk_multiple_events() {
    let mut parser = SseParser::new();
    let data = b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\nevent: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\" World\"}}\n\n";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert_eq!(results, vec!["Hello".to_string(), " World".to_string()]);
}

#[test]
fn test_sse_parser_parse_chunk_skips_non_delta_events() {
    let mut parser = SseParser::new();
    let data = b"event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_123\"}}\n\nevent: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert_eq!(results, vec!["Hello".to_string()]);
}

#[test]
fn test_sse_parser_parse_chunk_handles_done() {
    let mut parser = SseParser::new();
    let data = b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Test\"}}\n\ndata: [DONE]\n";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert_eq!(results, vec!["Test".to_string()]);
}

#[test]
fn test_sse_parser_parse_chunk_empty() {
    let mut parser = SseParser::new();
    let data = b"";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert!(results.is_empty());
}

#[test]
fn test_sse_parser_parse_chunk_skips_empty_text() {
    let mut parser = SseParser::new();
    let data = b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"\"}}\n\nevent: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Real content\"}}\n\n";
    let results = parser.parse_chunk(&Bytes::from_static(data));
    assert_eq!(results, vec!["Real content".to_string()]);
}

#[test]
fn test_sse_parser_buffers_incomplete_lines() {
    let mut parser = SseParser::new();

    // First chunk: incomplete line
    let data1 = b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hel";
    let results1 = parser.parse_chunk(&Bytes::from_static(data1));
    assert!(results1.is_empty()); // No complete event yet

    // Second chunk: completes the line
    let data2 = b"lo\"}}\n\n";
    let results2 = parser.parse_chunk(&Bytes::from_static(data2));
    assert_eq!(results2, vec!["Hello".to_string()]);
}

// **Feature: ai-request-cancellation, Property 2: Cancellation aborts the request**
// *For any* in-flight async request with a cancellation token, when the token is cancelled,
// the stream_with_cancel method SHALL return AiError::Cancelled and stop processing.
// **Validates: Requirements 1.2, 3.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_cancellation_aborts_request(
        api_key in "[a-zA-Z0-9]{10,20}",
        model in "[a-zA-Z0-9-]{5,20}",
        max_tokens in 100u32..4096u32,
        prompt in "[a-zA-Z0-9 ]{1,50}",
    ) {
        // Create a client
        let client = AsyncAnthropicClient::new(
            api_key,
            model,
            max_tokens,
        );

        // Create a cancellation token that's already cancelled
        let cancel_token = CancellationToken::new();
        cancel_token.cancel();

        // Create a response channel
        let (response_tx, _response_rx) = mpsc::channel();

        // Create a tokio runtime for the async test
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // Run the async function
        let result = rt.block_on(async {
            client.stream_with_cancel(
                &prompt,
                1,
                cancel_token,
                response_tx,
            ).await
        });

        // Should return Cancelled error
        prop_assert!(
            matches!(result, Err(AiError::Cancelled)),
            "Pre-cancelled token should result in AiError::Cancelled, got {:?}",
            result
        );
    }

    #[test]
    fn prop_cancellation_checked_before_request(
        api_key in "[a-zA-Z0-9]{10,20}",
        model in "[a-zA-Z0-9-]{5,20}",
        max_tokens in 100u32..4096u32,
        prompt in "[a-zA-Z0-9 ]{1,50}",
        request_id in 1u64..1000u64,
    ) {
        // Create a client
        let client = AsyncAnthropicClient::new(
            api_key,
            model,
            max_tokens,
        );

        // Create a cancellation token and cancel it immediately
        let cancel_token = CancellationToken::new();
        cancel_token.cancel();

        // Create a response channel
        let (response_tx, response_rx) = mpsc::channel();

        // Create a tokio runtime for the async test
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // Run the async function
        let result = rt.block_on(async {
            client.stream_with_cancel(
                &prompt,
                request_id,
                cancel_token,
                response_tx,
            ).await
        });

        // Should return Cancelled error without making any HTTP request
        prop_assert!(
            matches!(result, Err(AiError::Cancelled)),
            "Pre-cancelled token should return AiError::Cancelled immediately"
        );

        // No chunks should have been sent
        prop_assert!(
            response_rx.try_recv().is_err(),
            "No response chunks should be sent when cancelled before start"
        );
    }
}

// **Feature: ai-request-cancellation, Property 6: Idempotent cancellation**
// *For any* cancellation token, calling cancel() multiple times SHALL have the same
// effect as calling it once (idempotent operation).
// **Validates: Requirements 3.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_idempotent_cancellation(
        num_cancels in 1..10usize,
    ) {
        let token = CancellationToken::new();

        // Token should not be cancelled initially
        prop_assert!(!token.is_cancelled(), "Token should not be cancelled initially");

        // Cancel multiple times
        for i in 0..num_cancels {
            token.cancel();
            prop_assert!(
                token.is_cancelled(),
                "Token should be cancelled after cancel() call {}",
                i + 1
            );
        }

        // Token should still be cancelled
        prop_assert!(token.is_cancelled(), "Token should remain cancelled");
    }
}

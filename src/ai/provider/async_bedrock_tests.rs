use super::*;

use aws_sdk_bedrockruntime::types::{
    ContentBlockDelta, ContentBlockDeltaEvent, ConverseStreamOutput, MessageStopEvent, StopReason,
    ToolUseBlockDelta,
};

/// Wrap a `ContentBlockDelta` in the `ConverseStreamOutput::ContentBlockDelta`
/// event shape that `extract_text_from_event` matches against.
/// `content_block_index` is a required field on the SDK builder, so it is
/// always set here; tests only care about the `delta` payload.
fn content_delta_event(delta: ContentBlockDelta) -> ConverseStreamOutput {
    let event = ContentBlockDeltaEvent::builder()
        .delta(delta)
        .content_block_index(0)
        .build()
        .unwrap();
    ConverseStreamOutput::ContentBlockDelta(event)
}

#[test]
fn test_new_creates_client_with_fields() {
    let client = AsyncBedrockClient::new(
        "us-east-1".to_string(),
        "anthropic.claude-3-haiku-20240307-v1:0".to_string(),
        Some("my-profile".to_string()),
    );

    assert_eq!(client.region, "us-east-1");
    assert_eq!(client.model, "anthropic.claude-3-haiku-20240307-v1:0");
    assert_eq!(client.profile, Some("my-profile".to_string()));
}

#[test]
fn test_new_without_profile() {
    let client = AsyncBedrockClient::new(
        "us-west-2".to_string(),
        "amazon.titan-text-express-v1".to_string(),
        None,
    );

    assert_eq!(client.region, "us-west-2");
    assert_eq!(client.model, "amazon.titan-text-express-v1");
    assert_eq!(client.profile, None);
}

// extract_text_from_event: the happy path that streams tokens to the user.
// A ContentBlockDelta carrying a Text delta must yield Some(text).
#[test]
fn test_extract_text_from_content_block_delta_text() {
    let event = content_delta_event(ContentBlockDelta::Text("hello".to_string()));

    assert_eq!(
        AsyncBedrockClient::extract_text_from_event(&event),
        Some("hello".to_string())
    );
}

// extract_text_from_event: a ContentBlockDelta whose delta is a non-Text
// variant (here ToolUse) must be ignored, returning None. This guards the
// inner catch-all arm so tool-use deltas never leak into the streamed answer.
#[test]
fn test_extract_text_from_non_text_delta_returns_none() {
    let tool_use = ToolUseBlockDelta::builder()
        .input("{\"key\":\"value\"}")
        .build()
        .unwrap();
    let event = content_delta_event(ContentBlockDelta::ToolUse(tool_use));

    assert_eq!(AsyncBedrockClient::extract_text_from_event(&event), None);
}

// extract_text_from_event: control events that are not ContentBlockDelta
// (e.g. MessageStop), and ContentBlockDelta events with no delta set, must
// both return None so interleaved stream control events are not surfaced as
// garbage chunks.
#[test]
fn test_extract_text_from_non_content_block_delta_event_returns_none() {
    let stop_event = MessageStopEvent::builder()
        .stop_reason(StopReason::EndTurn)
        .build()
        .unwrap();
    let message_stop = ConverseStreamOutput::MessageStop(stop_event);
    assert_eq!(
        AsyncBedrockClient::extract_text_from_event(&message_stop),
        None
    );

    let empty_delta = ContentBlockDeltaEvent::builder()
        .content_block_index(0)
        .build()
        .unwrap();
    let no_delta = ConverseStreamOutput::ContentBlockDelta(empty_delta);
    assert_eq!(AsyncBedrockClient::extract_text_from_event(&no_delta), None);
}

// stream_with_cancel must short-circuit with Err(AiError::Cancelled) when the
// token is already cancelled on entry, returning before build_client() and
// thus before any AWS credential/network access.
#[tokio::test]
async fn test_stream_with_cancel_returns_cancelled_when_token_pre_cancelled() {
    use std::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    let client = AsyncBedrockClient::new("us-east-1".to_string(), "model".to_string(), None);

    let (tx, _rx) = mpsc::channel();
    let cancel_token = CancellationToken::new();
    cancel_token.cancel();

    let result = client.stream_with_cancel("hi", 1, cancel_token, tx).await;

    assert!(
        matches!(result, Err(AiError::Cancelled)),
        "Pre-cancelled token should return AiError::Cancelled before any network call, got {:?}",
        result
    );
}

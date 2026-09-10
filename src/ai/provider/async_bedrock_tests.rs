use super::*;

use aws_sdk_bedrockruntime::types::{
    ContentBlockDelta, ContentBlockDeltaEvent, ConverseStreamOutput, MessageStopEvent, StopReason,
    ToolUseBlockDelta,
};
use aws_smithy_types::Document;
use std::collections::HashMap;

use crate::ai::chat::{AiPrompt, ChatMessage};
use crate::config::ai_types::AiEffort;

/// Collapse Converse messages into (role, text) pairs for assertions.
fn message_turns(messages: &[Message]) -> Vec<(ConversationRole, &str)> {
    messages
        .iter()
        .map(|m| {
            let text = m.content()[0].as_text().expect("text block").as_str();
            (m.role().clone(), text)
        })
        .collect()
}

// build_conversation: a non-empty system prompt becomes a SystemContentBlock::Text
// and the single user turn a ConversationRole::User message.
#[test]
fn test_build_conversation_includes_system_when_set() {
    let (system, messages) =
        build_conversation(&AiPrompt::single("You are a jq expert.", "prompt")).unwrap();

    assert_eq!(
        system.as_ref().and_then(|s| s.as_text().ok()),
        Some(&"You are a jq expert.".to_string())
    );
    assert_eq!(
        message_turns(&messages),
        vec![(ConversationRole::User, "prompt")]
    );
}

// build_conversation: an empty system prompt yields None so no system block is
// attached to the request.
#[test]
fn test_build_conversation_omits_system_when_empty() {
    let (system, messages) = build_conversation(&AiPrompt::single("", "prompt")).unwrap();

    assert!(system.is_none());
    assert_eq!(messages.len(), 1);
}

// build_conversation: turns keep their order and map onto User/Assistant roles.
#[test]
fn test_build_conversation_preserves_turn_order_and_roles() {
    let prompt = AiPrompt {
        system: String::new(),
        messages: vec![
            ChatMessage::user("first question"),
            ChatMessage::assistant("first answer"),
            ChatMessage::user("follow-up"),
        ],
    };

    let (_, messages) = build_conversation(&prompt).unwrap();

    assert_eq!(
        message_turns(&messages),
        vec![
            (ConversationRole::User, "first question"),
            (ConversationRole::Assistant, "first answer"),
            (ConversationRole::User, "follow-up"),
        ]
    );
}

/// Unwrap a `Document::Object` map or panic, for asserting on request fields.
fn as_object(doc: &Document) -> &HashMap<String, Document> {
    match doc {
        Document::Object(map) => map,
        other => panic!("expected Document::Object, got {:?}", other),
    }
}

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

#[test]
fn test_with_effort_and_context_1m_store_fields() {
    let client = AsyncBedrockClient::new("us-east-1".to_string(), "model".to_string(), None)
        .with_effort(Some(AiEffort::High))
        .with_context_1m(true);

    assert_eq!(client.effort, Some(AiEffort::High));
    assert!(client.context_1m);
}

#[test]
fn test_with_timeout_stores_duration() {
    let client = AsyncBedrockClient::new("us-east-1".to_string(), "model".to_string(), None)
        .with_timeout(Some(std::time::Duration::from_secs(30)));
    assert_eq!(client.timeout, Some(std::time::Duration::from_secs(30)));

    let unbounded = AsyncBedrockClient::new("us-east-1".to_string(), "model".to_string(), None)
        .with_timeout(None);
    assert_eq!(unbounded.timeout, None);
}

// build_additional_fields: with neither effort nor 1M context configured, no
// additionalModelRequestFields document is attached (preserves prior behavior).
#[test]
fn test_build_additional_fields_none_when_unset() {
    let client = AsyncBedrockClient::new("us-east-1".to_string(), "model".to_string(), None);
    assert!(client.build_additional_fields().is_none());
}

// build_additional_fields: effort alone yields adaptive thinking plus the
// output_config.effort level, and does not add the 1M-context beta.
#[test]
fn test_build_additional_fields_effort_only() {
    let client = AsyncBedrockClient::new("us-east-1".to_string(), "model".to_string(), None)
        .with_effort(Some(AiEffort::Xhigh));

    let doc = client
        .build_additional_fields()
        .expect("effort should produce fields");
    let obj = as_object(&doc);

    assert!(!obj.contains_key("anthropic_beta"));

    let thinking = as_object(obj.get("thinking").expect("thinking present"));
    assert!(matches!(thinking.get("type"), Some(Document::String(s)) if s == "adaptive"));

    let output_config = as_object(obj.get("output_config").expect("output_config present"));
    assert!(matches!(output_config.get("effort"), Some(Document::String(s)) if s == "xhigh"));
}

// build_additional_fields: the 1M-context flag alone yields only the
// anthropic_beta array and no reasoning fields.
#[test]
fn test_build_additional_fields_context_1m_only() {
    let client = AsyncBedrockClient::new("us-east-1".to_string(), "model".to_string(), None)
        .with_context_1m(true);

    let doc = client
        .build_additional_fields()
        .expect("context_1m should produce fields");
    let obj = as_object(&doc);

    assert!(!obj.contains_key("thinking"));
    assert!(!obj.contains_key("output_config"));

    match obj.get("anthropic_beta") {
        Some(Document::Array(items)) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(&items[0], Document::String(s) if s == "context-1m-2025-08-07"));
        }
        other => panic!("expected anthropic_beta array, got {:?}", other),
    }
}

// build_additional_fields: OpenAI models on Converse take the chat-completions
// `reasoning_effort` field, not Claude's thinking/output_config shape (which
// those models reject).
#[test]
fn test_build_additional_fields_openai_model_uses_reasoning_effort() {
    for model in ["openai.gpt-oss-120b-1:0", "us.openai.gpt-oss-20b-1:0"] {
        let client = AsyncBedrockClient::new("us-east-1".to_string(), model.to_string(), None)
            .with_effort(Some(AiEffort::High));

        let doc = client
            .build_additional_fields()
            .expect("effort should produce fields");
        let obj = as_object(&doc);

        assert!(
            matches!(obj.get("reasoning_effort"), Some(Document::String(s)) if s == "high"),
            "OpenAI model should get reasoning_effort, got {:?}",
            obj
        );
        assert!(!obj.contains_key("thinking"));
        assert!(!obj.contains_key("output_config"));
    }
}

// build_additional_fields: effort and 1M context together produce all three keys.
#[test]
fn test_build_additional_fields_effort_and_context_1m() {
    let client = AsyncBedrockClient::new("us-east-1".to_string(), "model".to_string(), None)
        .with_effort(Some(AiEffort::Max))
        .with_context_1m(true);

    let doc = client
        .build_additional_fields()
        .expect("effort + context_1m should produce fields");
    let obj = as_object(&doc);

    assert!(obj.contains_key("thinking"));
    assert!(obj.contains_key("output_config"));
    assert!(obj.contains_key("anthropic_beta"));
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

    let result = client
        .stream_with_cancel(&AiPrompt::single("", "hi"), 1, cancel_token, tx)
        .await;

    assert!(
        matches!(result, Err(AiError::Cancelled)),
        "Pre-cancelled token should return AiError::Cancelled before any network call, got {:?}",
        result
    );
}

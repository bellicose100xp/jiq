//! Tests for Async Gemini API client

use super::*;
use insta::assert_snapshot;
use proptest::prelude::*;

#[test]
fn test_async_gemini_client_new() {
    let client =
        AsyncGeminiClient::new("AIza-test-key".to_string(), "gemini-2.0-flash".to_string());
    // Verify it creates without panic
    assert!(format!("{:?}", client).contains("AsyncGeminiClient"));
}

// Subtask 4.5: Write property test for API key storage
// **Feature: gemini-provider, Property 2: API key storage**
// *For any* non-empty API key string, the constructed AsyncGeminiClient should
// store this exact API key for use in authentication.
// **Validates: Requirements 1.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_api_key_storage(
        api_key in "[a-zA-Z0-9-_]{10,100}",
        model in "[a-zA-Z0-9-]{5,20}",
    ) {
        // Create a client with the generated API key
        let client = AsyncGeminiClient::new(
            api_key.clone(),
            model,
        );

        // Verify the API key is stored correctly using the accessor method
        prop_assert_eq!(
            client.api_key(),
            &api_key,
            "Client should store the exact API key provided"
        );
    }
}

// Subtask 4.6: Write property test for model selection storage
// **Feature: gemini-provider, Property 3: Model selection storage**
// *For any* non-empty model string, the constructed AsyncGeminiClient should
// store this exact model name for use in API requests.
// **Validates: Requirements 1.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_model_selection_storage(
        api_key in "[a-zA-Z0-9-_]{10,50}",
        model in "[a-zA-Z0-9-]{5,50}",
    ) {
        // Create a client with the generated model
        let client = AsyncGeminiClient::new(
            api_key,
            model.clone(),
        );

        // Verify the model is stored correctly using the accessor method
        prop_assert_eq!(
            client.model(),
            &model,
            "Client should store the exact model name provided"
        );
    }
}

// Subtask 4.7: Write property test for request format correctness
// **Feature: gemini-provider, Property: Request format correctness**
// *For any* prompt string, the Gemini client should construct a request body
// that includes a contents array with user role and parts, without a stream field.
// **Validates: Requirements 2.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_request_format_correctness(
        api_key in "[a-zA-Z0-9-_]{10,50}",
        model in "[a-zA-Z0-9-]{5,50}",
        prompt in ".*",
    ) {
        // Create a client
        let client = AsyncGeminiClient::new(
            api_key,
            model,
        );

        // Build the request body
        let result = client.build_request_body(&prompt);

        // Verify the request body was created successfully
        prop_assert!(result.is_ok(), "Request body should serialize successfully");

        let body = result.unwrap();

        // Parse the JSON to verify structure
        let json: serde_json::Value = serde_json::from_str(&body)
            .expect("Request body should be valid JSON");

        // Verify stream field is NOT set (Gemini uses query param)
        prop_assert!(
            json.get("stream").is_none(),
            "Request should not include stream field (Gemini uses query param)"
        );

        // Verify contents array exists and has one element
        let contents = json.get("contents").and_then(|v| v.as_array());
        prop_assert!(contents.is_some(), "Request should have a contents array");
        let contents = contents.unwrap();
        prop_assert_eq!(contents.len(), 1, "Contents array should have exactly one element");

        // Verify content structure
        let content = &contents[0];
        prop_assert_eq!(
            content.get("role").and_then(|v| v.as_str()),
            Some("user"),
            "Content should have role 'user'"
        );

        // Verify parts array
        let parts = content.get("parts").and_then(|v| v.as_array());
        prop_assert!(parts.is_some(), "Content should have a parts array");
        let parts = parts.unwrap();
        prop_assert_eq!(parts.len(), 1, "Parts array should have exactly one element");

        // Verify part text matches prompt
        prop_assert_eq!(
            parts[0].get("text").and_then(|v| v.as_str()),
            Some(prompt.as_str()),
            "Part text should match the prompt"
        );
    }
}

// Subtask 4.8: Write snapshot test for request body format
// Snapshot of serialized Gemini request body
// Verify JSON structure matches API specification
#[test]
fn snapshot_request_body_format() {
    let client = AsyncGeminiClient::new("AIza-test123".to_string(), "gemini-2.0-flash".to_string());

    let body = client
        .build_request_body("suggest jq filters for: extract user names")
        .expect("Request body should serialize successfully");

    // Parse and pretty-print for snapshot readability
    let json: serde_json::Value =
        serde_json::from_str(&body).expect("Request body should be valid JSON");
    let pretty_json =
        serde_json::to_string_pretty(&json).expect("Should be able to pretty-print JSON");

    assert_snapshot!(pretty_json);
}

#[test]
fn test_build_url_format() {
    let client =
        AsyncGeminiClient::new("AIza-test-key".to_string(), "gemini-2.0-flash".to_string());
    let url = client.build_url();

    assert!(url.starts_with("https://generativelanguage.googleapis.com/v1beta/models/"));
    assert!(url.contains("gemini-2.0-flash:streamGenerateContent"));
    assert!(url.contains("alt=sse"));
    assert!(url.contains("key=AIza-test-key"));
}

// Test cancellation before response
#[tokio::test]
async fn test_cancellation_before_response() {
    use std::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    let client =
        AsyncGeminiClient::new("AIza-test-key".to_string(), "gemini-2.0-flash".to_string());

    let (tx, _rx) = mpsc::channel();
    let cancel_token = CancellationToken::new();

    // Cancel immediately before making request
    cancel_token.cancel();

    let result = client
        .stream_with_cancel("test prompt", 1, cancel_token, tx)
        .await;

    // Should return Cancelled error
    assert!(
        matches!(result, Err(AiError::Cancelled)),
        "Should return Cancelled error when token is already cancelled"
    );
}

// Test channel disconnection handling
#[test]
fn test_channel_disconnection() {
    use crate::ai::ai_state::AiResponse;
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    // Drop the receiver to simulate main thread disconnection
    drop(rx);

    // Try to send a chunk
    let result = tx.send(AiResponse::Chunk {
        text: "test".to_string(),
        request_id: 1,
    });

    // Should fail because receiver is dropped
    assert!(result.is_err(), "Send should fail when receiver is dropped");
}

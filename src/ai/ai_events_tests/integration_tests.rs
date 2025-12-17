//! Integration tests for full AI event flow

use super::*;

/// Test: query change → jq executes → error result → AI request with error
/// Validates the full flow for error results
#[test]
fn test_full_flow_error_result() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = true; // Popup must be visible for requests to be sent
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Simulate initial query
    ai_state.set_last_query_hash(".initial");

    // Start an in-flight request (simulating previous query)
    ai_state.start_request();
    let _old_request_id = ai_state.current_request_id();

    // Clear channel
    while rx.try_recv().is_ok() {}

    // Simulate: query change → jq executes → error result
    let error_result: Result<String, String> =
        Err("jq: error: .invalid is not defined".to_string());
    handle_execution_result(
        &mut ai_state,
        &error_result,
        ".invalid",
        8,
        r#"{"name": "test"}"#,
    );

    // Verify the flow:
    // 1. In-flight request was cleared (cancellation handled via token)
    // 2. New Query request was sent with error context
    let mut found_query = false;
    let mut query_prompt = String::new();

    while let Ok(msg) = rx.try_recv() {
        match msg {
            AiRequest::Query { prompt, .. } => {
                found_query = true;
                query_prompt = prompt;
            }
        }
    }

    assert!(found_query, "Should have sent new Query request");
    assert!(
        query_prompt.contains("troubleshoot"),
        "Error prompt should mention troubleshooting"
    );
    assert!(
        query_prompt.contains(".invalid is not defined"),
        "Error prompt should contain error message"
    );
}

/// Test: query change → jq executes → success result → AI request with output
/// Validates the full flow for success results
#[test]
fn test_full_flow_success_result() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = true; // Popup must be visible for requests to be sent
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Simulate initial query
    ai_state.set_last_query_hash(".initial");

    // Start an in-flight request (simulating previous query)
    ai_state.start_request();
    let _old_request_id = ai_state.current_request_id();

    // Clear channel
    while rx.try_recv().is_ok() {}

    // Simulate: query change → jq executes → success result
    let success_result: Result<String, String> = Ok(r#""test_value""#.to_string());
    handle_execution_result(
        &mut ai_state,
        &success_result,
        ".name",
        5,
        r#"{"name": "test_value"}"#,
    );

    // Verify the flow:
    // 1. In-flight request was cleared (cancellation handled via token)
    // 2. New Query request was sent with success context
    let mut found_query = false;
    let mut query_prompt = String::new();

    while let Ok(msg) = rx.try_recv() {
        match msg {
            AiRequest::Query { prompt, .. } => {
                found_query = true;
                query_prompt = prompt;
            }
        }
    }

    assert!(found_query, "Should have sent new Query request");
    assert!(
        query_prompt.contains("optimize"),
        "Success prompt should mention optimization"
    );
    assert!(
        query_prompt.contains(".name"),
        "Success prompt should contain query"
    );
}

/// Test: rapid typing → multiple jq executions → multiple AI requests
/// Validates that rapid query changes result in multiple requests with proper tracking
#[test]
fn test_rapid_typing_sends_multiple_requests() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = true; // Popup must be visible for requests to be sent
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Simulate rapid typing: .n → .na → .nam → .name
    let queries = [".n", ".na", ".nam", ".name"];
    let mut last_request_id = 0;

    for (i, query) in queries.iter().enumerate() {
        // Each query change triggers execution result handler
        let result: Result<String, String> = if i < queries.len() - 1 {
            // Intermediate queries might error (partial field names)
            Err(format!("{} is not defined", query))
        } else {
            // Final query succeeds
            Ok(r#""test""#.to_string())
        };

        handle_execution_result(
            &mut ai_state,
            &result,
            query,
            query.len(),
            r#"{"name": "test"}"#,
        );

        last_request_id = ai_state.current_request_id();
    }

    // Drain the channel and count messages
    let mut query_count = 0;
    let mut last_query_request_id = 0;

    while let Ok(msg) = rx.try_recv() {
        match msg {
            AiRequest::Query { request_id, .. } => {
                query_count += 1;
                last_query_request_id = request_id;
            }
        }
    }

    // Should have 4 Query requests (one per query change)
    assert_eq!(
        query_count, 4,
        "Should have sent 4 Query requests (one per query)"
    );

    // The last Query request should have the latest request_id
    assert_eq!(
        last_query_request_id, last_request_id,
        "Last Query should have latest request_id"
    );
}

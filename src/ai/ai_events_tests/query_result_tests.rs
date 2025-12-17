//! Tests for query result handling

use super::*;

#[test]
fn test_poll_without_channel_does_nothing() {
    let mut ai_state = AiState::new(true);
    // No channel set

    poll_response_channel(&mut ai_state);

    // Should not crash, state unchanged
    assert!(!ai_state.loading);
    assert!(ai_state.response.is_empty());
}

#[test]
fn test_poll_processes_chunk() {
    let mut ai_state = AiState::new(true);
    let (_tx, rx) = mpsc::channel();
    ai_state.response_rx = Some(rx);
    ai_state.loading = true;

    // Send a chunk through a new channel
    let (tx, rx) = mpsc::channel();
    ai_state.response_rx = Some(rx);
    // Simulate starting a request to set request_id
    ai_state.start_request();
    let request_id = ai_state.current_request_id();
    tx.send(AiResponse::Chunk {
        text: "Hello ".to_string(),
        request_id,
    })
    .unwrap();

    poll_response_channel(&mut ai_state);

    assert_eq!(ai_state.response, "Hello ");
    assert!(ai_state.loading); // Still loading until Complete
}

#[test]
fn test_poll_processes_multiple_chunks() {
    let mut ai_state = AiState::new(true);
    let (tx, rx) = mpsc::channel();
    ai_state.response_rx = Some(rx);
    ai_state.start_request();
    let request_id = ai_state.current_request_id();

    tx.send(AiResponse::Chunk {
        text: "Hello ".to_string(),
        request_id,
    })
    .unwrap();
    tx.send(AiResponse::Chunk {
        text: "World".to_string(),
        request_id,
    })
    .unwrap();

    poll_response_channel(&mut ai_state);

    assert_eq!(ai_state.response, "Hello World");
}

#[test]
fn test_poll_processes_complete() {
    let mut ai_state = AiState::new(true);
    let (tx, rx) = mpsc::channel();
    ai_state.response_rx = Some(rx);
    ai_state.start_request();
    let request_id = ai_state.current_request_id();
    ai_state.response = "Full response".to_string();

    tx.send(AiResponse::Complete { request_id }).unwrap();

    poll_response_channel(&mut ai_state);

    assert!(!ai_state.loading);
    assert_eq!(ai_state.response, "Full response");
}

#[test]
fn test_poll_processes_error() {
    let mut ai_state = AiState::new(true);
    let (tx, rx) = mpsc::channel();
    ai_state.response_rx = Some(rx);
    ai_state.loading = true;

    tx.send(AiResponse::Error("Network error".to_string()))
        .unwrap();

    poll_response_channel(&mut ai_state);

    assert!(!ai_state.loading);
    assert_eq!(ai_state.error, Some("Network error".to_string()));
}

#[test]
fn test_poll_processes_cancelled() {
    let mut ai_state = AiState::new(true);
    let (tx, rx) = mpsc::channel();
    ai_state.response_rx = Some(rx);
    ai_state.start_request();
    let request_id = ai_state.current_request_id();

    tx.send(AiResponse::Cancelled { request_id }).unwrap();

    poll_response_channel(&mut ai_state);

    assert!(!ai_state.loading);
    assert!(ai_state.in_flight_request_id.is_none());
}

#[test]
fn test_poll_handles_disconnected_channel() {
    let mut ai_state = AiState::new(true);
    let (tx, rx) = mpsc::channel::<AiResponse>();
    ai_state.response_rx = Some(rx);
    ai_state.loading = true;

    // Drop sender to disconnect channel
    drop(tx);

    poll_response_channel(&mut ai_state);

    // Should set error when loading and channel disconnects
    assert!(!ai_state.loading);
    assert!(ai_state.error.is_some());
}

#[test]
fn test_poll_empty_channel_does_nothing() {
    let mut ai_state = AiState::new(true);
    let (_tx, rx) = mpsc::channel::<AiResponse>();
    ai_state.response_rx = Some(rx);
    ai_state.loading = true;

    // Don't send anything

    poll_response_channel(&mut ai_state);

    // State should be unchanged
    assert!(ai_state.loading);
    assert!(ai_state.response.is_empty());
    assert!(ai_state.error.is_none());
}

#[test]
fn test_stale_responses_filtered() {
    let mut ai_state = AiState::new(true);
    let (tx, rx) = mpsc::channel();
    ai_state.response_rx = Some(rx);

    // Start first request
    ai_state.start_request();
    let old_request_id = ai_state.current_request_id();

    // Start second request (increments request_id)
    ai_state.start_request();
    let new_request_id = ai_state.current_request_id();

    assert!(new_request_id > old_request_id);

    // Send chunk from old request - should be ignored
    tx.send(AiResponse::Chunk {
        text: "old chunk".to_string(),
        request_id: old_request_id,
    })
    .unwrap();

    // Send chunk from new request - should be processed
    tx.send(AiResponse::Chunk {
        text: "new chunk".to_string(),
        request_id: new_request_id,
    })
    .unwrap();

    poll_response_channel(&mut ai_state);

    // Only the new chunk should be in the response
    assert_eq!(ai_state.response, "new chunk");
}

#[test]
fn test_stale_complete_filtered() {
    let mut ai_state = AiState::new(true);
    let (tx, rx) = mpsc::channel();
    ai_state.response_rx = Some(rx);

    // Start first request
    ai_state.start_request();
    let old_request_id = ai_state.current_request_id();

    // Start second request
    ai_state.start_request();

    // Send complete from old request - should be ignored
    tx.send(AiResponse::Complete {
        request_id: old_request_id,
    })
    .unwrap();

    poll_response_channel(&mut ai_state);

    // Loading should still be true (stale complete was ignored)
    assert!(ai_state.loading);
}

// Test: query changes from error to success → stale response cleared, new request sent
#[test]
fn test_query_error_to_success_clears_response() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = true;
    ai_state.response = "Error explanation".to_string();
    ai_state.error = Some("Query error".to_string());
    ai_state.set_last_query_hash(".invalid");

    // Simulate successful query result with different query
    let result: Result<String, String> = Ok("success output".to_string());
    handle_execution_result(&mut ai_state, &result, ".valid", 6, "{}");

    // Stale response should be cleared (query changed)
    // Note: response is cleared by clear_stale_response, then new request starts
    // Since we don't have a channel, send_request returns false but state is still cleared
    assert!(ai_state.error.is_none());
    // Visibility preserved
    assert!(ai_state.visible);
}

// Test: query changes from one error to different error → old response cleared
#[test]
fn test_query_error_to_different_error_clears_response() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = true;
    ai_state.response = "Old error explanation".to_string();
    ai_state.set_last_query_hash(".old");

    // Simulate new error result with different query
    let result: Result<String, String> = Err("new error".to_string());
    handle_execution_result(&mut ai_state, &result, ".new", 4, "{}");

    // Old response should be cleared (query changed)
    assert!(ai_state.response.is_empty());
}

// Test: different query with same error → new request (query changed)
#[test]
fn test_different_query_same_error_triggers_new_request() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.response = "Old explanation".to_string();
    ai_state.set_last_query_hash(".query1");

    // Different query should clear stale response (even with same error)
    let result: Result<String, String> = Err("same error".to_string());
    handle_execution_result(&mut ai_state, &result, ".query2", 7, "{}");

    // Response should be cleared because query changed
    assert!(ai_state.response.is_empty());
}

// Test: same query with same error → no new request
#[test]
fn test_same_query_same_error_no_change() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.response = "Existing explanation".to_string();
    ai_state.set_last_query_hash(".same");

    // Same query should NOT clear response (regardless of error)
    let result: Result<String, String> = Err("same error".to_string());
    handle_execution_result(&mut ai_state, &result, ".same", 5, "{}");

    // Response should be preserved (query didn't change)
    assert_eq!(ai_state.response, "Existing explanation");
}

// Test: same query with different error → no new request (query is the only trigger)
#[test]
fn test_same_query_different_error_no_change() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.response = "Existing explanation".to_string();
    ai_state.set_last_query_hash(".same");

    // Same query should NOT clear response even with different error
    let result: Result<String, String> = Err("different error".to_string());
    handle_execution_result(&mut ai_state, &result, ".same", 5, "{}");

    // Response should be preserved (query didn't change)
    assert_eq!(ai_state.response, "Existing explanation");
}

// Test: different query with different error → new request (query changed)
#[test]
fn test_different_query_different_error_triggers_new_request() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.response = "Old explanation".to_string();
    ai_state.set_last_query_hash(".query1");

    // Different query should trigger new request
    let result: Result<String, String> = Err("different error".to_string());
    handle_execution_result(&mut ai_state, &result, ".query2", 7, "{}");

    // Response should be cleared because query changed
    assert!(ai_state.response.is_empty());
}

// Test: successful query triggers AI request with output context
#[test]
fn test_success_triggers_ai_request() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = true; // Popup must be visible for requests to be sent
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Simulate successful query result
    let result: Result<String, String> = Ok("output data".to_string());
    handle_execution_result(&mut ai_state, &result, ".name", 5, r#"{"name": "test"}"#);

    // Should have sent a request
    let request = rx.try_recv();
    assert!(request.is_ok(), "Should have sent AI request for success");

    // Verify it's a Query request with success context
    let AiRequest::Query { prompt, .. } = request.unwrap();
    // Success prompt should contain "optimize" (from build_success_prompt)
    assert!(
        prompt.contains("optimize"),
        "Success prompt should mention optimization"
    );
}

// Test: error query triggers AI request with error context
#[test]
fn test_error_triggers_ai_request() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = true; // Popup must be visible for requests to be sent
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Simulate error query result
    let result: Result<String, String> = Err("syntax error".to_string());
    handle_execution_result(&mut ai_state, &result, ".invalid", 8, r#"{"name": "test"}"#);

    // Should have sent a request
    let request = rx.try_recv();
    assert!(request.is_ok(), "Should have sent AI request for error");

    // Verify it's a Query request with error context
    let AiRequest::Query { prompt, .. } = request.unwrap();
    // Error prompt should contain "troubleshoot" (from build_error_prompt)
    assert!(
        prompt.contains("troubleshoot"),
        "Error prompt should mention troubleshooting"
    );
    assert!(
        prompt.contains("syntax error"),
        "Error prompt should contain error message"
    );
}

// Test: query change clears in-flight request tracking
#[test]
fn test_query_change_clears_in_flight_request() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = true; // Popup must be visible for requests to be sent
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Start an in-flight request
    ai_state.start_request();
    let _old_request_id = ai_state.current_request_id();
    assert!(ai_state.has_in_flight_request());

    // Clear the channel
    while rx.try_recv().is_ok() {}

    // Set up for new query
    ai_state.set_last_query_hash(".old");

    // Simulate new query result (different query)
    let result: Result<String, String> = Ok("output".to_string());
    handle_execution_result(&mut ai_state, &result, ".new", 4, "{}");

    // Should have sent Query for new request
    // (cancellation is now handled via CancellationToken, not Cancel message)
    let mut found_query = false;

    while let Ok(msg) = rx.try_recv() {
        match msg {
            AiRequest::Query { .. } => {
                found_query = true;
            }
        }
    }

    assert!(found_query, "Should have sent new Query request");
}

// Test: handle_query_result wrapper works correctly
#[test]
fn test_handle_query_result_wrapper() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.set_last_query_hash(".old");

    // Test with generic type that implements ToString
    let result: Result<&str, String> = Ok("output");
    handle_query_result(&mut ai_state, &result, ".new", 4, "{}");

    // Should have updated query hash
    assert!(!ai_state.is_query_changed(".new"));
}

// Test: same query repeated → no duplicate AI requests
#[test]
fn test_same_query_no_duplicate_requests() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // First execution
    let result: Result<String, String> = Ok(r#""test""#.to_string());
    handle_execution_result(&mut ai_state, &result, ".name", 5, r#"{"name": "test"}"#);

    // Drain channel
    while rx.try_recv().is_ok() {}

    // Same query executed again (e.g., user pressed Enter)
    handle_execution_result(
        &mut ai_state,
        &result,
        ".name", // Same query
        5,
        r#"{"name": "test"}"#,
    );

    // Should NOT have sent any new requests
    let request = rx.try_recv();
    assert!(
        request.is_err(),
        "Should not send duplicate request for same query"
    );
}

// Test: AI disabled → no requests sent
#[test]
fn test_ai_disabled_no_requests() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = false; // AI disabled
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Execute query
    let result: Result<String, String> = Ok(r#""test""#.to_string());
    handle_execution_result(&mut ai_state, &result, ".name", 5, r#"{"name": "test"}"#);

    // Should NOT have sent any requests
    let request = rx.try_recv();
    assert!(
        request.is_err(),
        "Should not send request when AI is disabled"
    );
}

/// Test: visible=true → AI requests sent on error
#[test]
fn test_visible_sends_requests_on_error() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = true; // Popup visible
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Execute query with error
    let result: Result<String, String> = Err("syntax error".to_string());
    handle_execution_result(&mut ai_state, &result, ".invalid", 8, r#"{"name": "test"}"#);

    // Should have sent AI request
    let request = rx.try_recv();
    assert!(
        request.is_ok(),
        "Should send AI request when popup is visible"
    );

    // Verify it's a Query request with error context
    let AiRequest::Query { prompt, .. } = request.unwrap();
    assert!(
        prompt.contains("troubleshoot"),
        "Error prompt should mention troubleshooting"
    );
    assert!(
        prompt.contains("syntax error"),
        "Error prompt should contain error message"
    );
}

/// Test: visible=false → no AI requests on error
#[test]
fn test_hidden_no_requests_on_error() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = false; // Popup hidden
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Execute query with error
    let result: Result<String, String> = Err("syntax error".to_string());
    handle_execution_result(&mut ai_state, &result, ".invalid", 8, r#"{"name": "test"}"#);

    // Should NOT have sent AI request
    let request = rx.try_recv();
    assert!(
        request.is_err(),
        "Should not send AI request when popup is hidden"
    );
}

/// Test: visible=true with success → AI request sent
#[test]
fn test_visible_sends_requests_on_success() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = true; // Popup visible
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Execute query with success
    let result: Result<String, String> = Ok(r#""test_value""#.to_string());
    handle_execution_result(
        &mut ai_state,
        &result,
        ".name",
        5,
        r#"{"name": "test_value"}"#,
    );

    // Should have sent AI request
    let request = rx.try_recv();
    assert!(
        request.is_ok(),
        "Should send AI request for success when popup is visible"
    );

    // Verify it's a Query request with success context
    let AiRequest::Query { prompt, .. } = request.unwrap();
    assert!(
        prompt.contains("optimize"),
        "Success prompt should mention optimization"
    );
}

/// Test: visible=false with success → no AI request sent
#[test]
fn test_hidden_no_requests_on_success() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = false; // Popup hidden
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Execute query with success
    let result: Result<String, String> = Ok(r#""test_value""#.to_string());
    handle_execution_result(
        &mut ai_state,
        &result,
        ".name",
        5,
        r#"{"name": "test_value"}"#,
    );

    // Should NOT have sent AI request
    let request = rx.try_recv();
    assert!(
        request.is_err(),
        "Should not send AI request when popup is hidden"
    );
}

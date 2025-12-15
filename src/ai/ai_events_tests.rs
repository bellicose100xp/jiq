//! Tests for AI event handling
//!
//! This module contains unit tests and property-based tests for the AI event handlers.

use super::ai_events::*;
use super::ai_state::{AiResponse, AiState};
use proptest::prelude::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::mpsc;

// Helper to create key events
fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn key_with_mods(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

// =========================================================================
// Unit Tests for handle_toggle_key
// =========================================================================

#[test]
fn test_ctrl_a_toggles_visibility_on() {
    let mut ai_state = AiState::new(true, 1000);
    assert!(!ai_state.visible);

    let handled = handle_toggle_key(
        key_with_mods(KeyCode::Char('a'), KeyModifiers::CONTROL),
        &mut ai_state,
    );

    assert!(handled);
    assert!(ai_state.visible);
}

#[test]
fn test_ctrl_a_toggles_visibility_off() {
    let mut ai_state = AiState::new(true, 1000);
    ai_state.visible = true;

    let handled = handle_toggle_key(
        key_with_mods(KeyCode::Char('a'), KeyModifiers::CONTROL),
        &mut ai_state,
    );

    assert!(handled);
    assert!(!ai_state.visible);
}

#[test]
fn test_plain_a_not_handled() {
    let mut ai_state = AiState::new(true, 1000);

    let handled = handle_toggle_key(key(KeyCode::Char('a')), &mut ai_state);

    assert!(!handled);
    assert!(!ai_state.visible);
}

#[test]
fn test_ctrl_other_key_not_handled() {
    let mut ai_state = AiState::new(true, 1000);

    let handled = handle_toggle_key(
        key_with_mods(KeyCode::Char('b'), KeyModifiers::CONTROL),
        &mut ai_state,
    );

    assert!(!handled);
    assert!(!ai_state.visible);
}

// =========================================================================
// Unit Tests for handle_close_key
// =========================================================================

#[test]
fn test_esc_closes_visible_popup() {
    let mut ai_state = AiState::new(true, 1000);
    ai_state.visible = true;

    let handled = handle_close_key(key(KeyCode::Esc), &mut ai_state);

    assert!(handled);
    assert!(!ai_state.visible);
}

#[test]
fn test_esc_not_handled_when_popup_hidden() {
    let mut ai_state = AiState::new(true, 1000);
    assert!(!ai_state.visible);

    let handled = handle_close_key(key(KeyCode::Esc), &mut ai_state);

    assert!(!handled);
}

#[test]
fn test_other_key_not_handled_for_close() {
    let mut ai_state = AiState::new(true, 1000);
    ai_state.visible = true;

    let handled = handle_close_key(key(KeyCode::Enter), &mut ai_state);

    assert!(!handled);
    assert!(ai_state.visible);
}

// =========================================================================
// Unit Tests for poll_response_channel
// =========================================================================

#[test]
fn test_poll_without_channel_does_nothing() {
    let mut ai_state = AiState::new(true, 1000);
    // No channel set

    poll_response_channel(&mut ai_state);

    // Should not crash, state unchanged
    assert!(!ai_state.loading);
    assert!(ai_state.response.is_empty());
}

#[test]
fn test_poll_processes_chunk() {
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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

// =========================================================================
// Property-Based Tests
// =========================================================================

// **Feature: ai-assistant, Property 10: Streaming concatenation**
// *For any* sequence of response chunks [c1, c2, ..., cn], the final displayed
// response should equal c1 + c2 + ... + cn.
// **Validates: Requirements 4.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_streaming_concatenation(
        chunks in prop::collection::vec("[a-zA-Z0-9 .,!?]{0,50}", 0..10)
    ) {
        let mut ai_state = AiState::new(true, 1000);
        let (tx, rx) = mpsc::channel();
        ai_state.response_rx = Some(rx);
        ai_state.start_request();
        let request_id = ai_state.current_request_id();

        // Calculate expected concatenation
        let expected: String = chunks.iter().cloned().collect();

        // Send all chunks with matching request_id
        for chunk in &chunks {
            tx.send(AiResponse::Chunk {
                text: chunk.clone(),
                request_id,
            })
            .unwrap();
        }

        // Poll to process all chunks
        poll_response_channel(&mut ai_state);

        prop_assert_eq!(
            ai_state.response, expected,
            "Response should be concatenation of all chunks"
        );
    }
}

// **Feature: ai-assistant, Property 11: Loading state during request**
// *For any* AiState that has sent a request and not received Complete or Error,
// `loading` should be true.
// **Validates: Requirements 4.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_loading_state_during_request(
        num_chunks in 1usize..10,
        chunk_content in "[a-zA-Z0-9 ]{1,20}"
    ) {
        let mut ai_state = AiState::new(true, 1000);
        let (tx, rx) = mpsc::channel();
        ai_state.response_rx = Some(rx);

        // Start a request (sets loading = true)
        ai_state.start_request();
        let request_id = ai_state.current_request_id();
        prop_assert!(ai_state.loading, "Loading should be true after start_request");

        // Send chunks but NOT Complete or Error
        for _ in 0..num_chunks {
            tx.send(AiResponse::Chunk {
                text: chunk_content.clone(),
                request_id,
            })
            .unwrap();
        }

        // Poll to process chunks
        poll_response_channel(&mut ai_state);

        // Loading should still be true (no Complete/Error received)
        prop_assert!(
            ai_state.loading,
            "Loading should remain true until Complete or Error is received"
        );

        // Now send Complete
        tx.send(AiResponse::Complete { request_id }).unwrap();
        poll_response_channel(&mut ai_state);

        // Loading should now be false
        prop_assert!(
            !ai_state.loading,
            "Loading should be false after Complete is received"
        );
    }

    #[test]
    fn prop_loading_state_cleared_on_error(
        error_msg in "[a-zA-Z0-9 ]{1,50}"
    ) {
        let mut ai_state = AiState::new(true, 1000);
        let (tx, rx) = mpsc::channel();
        ai_state.response_rx = Some(rx);

        // Start a request
        ai_state.start_request();
        prop_assert!(ai_state.loading, "Loading should be true after start_request");

        // Send Error
        tx.send(AiResponse::Error(error_msg.clone())).unwrap();
        poll_response_channel(&mut ai_state);

        // Loading should be false after error
        prop_assert!(
            !ai_state.loading,
            "Loading should be false after Error is received"
        );
        prop_assert_eq!(
            ai_state.error,
            Some(error_msg),
            "Error message should be set"
        );
    }
}

// Test that stale responses are filtered out
#[test]
fn test_stale_responses_filtered() {
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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

// =========================================================================
// Tests for Execution Result Handler (Task 20)
// Execution result is the trigger for AI requests (both success and error)
// =========================================================================

// Test: query changes from error to success → stale response cleared, new request sent
#[test]
fn test_query_error_to_success_clears_response() {
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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
    let mut ai_state = AiState::new(true, 1000);
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

    // Verify it's a Query request
    match request.unwrap() {
        super::ai_state::AiRequest::Query { prompt, .. } => {
            // Success prompt should contain "optimize" (from build_success_prompt)
            assert!(
                prompt.contains("optimize"),
                "Success prompt should mention optimization"
            );
        }
        _ => panic!("Expected Query request"),
    }
}

// Test: error query triggers AI request with error context
#[test]
fn test_error_triggers_ai_request() {
    let mut ai_state = AiState::new(true, 1000);
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

    // Verify it's a Query request
    match request.unwrap() {
        super::ai_state::AiRequest::Query { prompt, .. } => {
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
        _ => panic!("Expected Query request"),
    }
}

// Test: query change cancels in-flight request
#[test]
fn test_query_change_cancels_in_flight_request() {
    let mut ai_state = AiState::new(true, 1000);
    ai_state.enabled = true;
    ai_state.visible = true; // Popup must be visible for requests to be sent
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Start an in-flight request
    ai_state.start_request();
    let old_request_id = ai_state.current_request_id();
    assert!(ai_state.has_in_flight_request());

    // Clear the channel
    while rx.try_recv().is_ok() {}

    // Set up for new query
    ai_state.set_last_query_hash(".old");

    // Simulate new query result (different query)
    let result: Result<String, String> = Ok("output".to_string());
    handle_execution_result(&mut ai_state, &result, ".new", 4, "{}");

    // Should have sent Cancel for old request, then Query for new
    let mut found_cancel = false;
    let mut found_query = false;

    while let Ok(msg) = rx.try_recv() {
        match msg {
            super::ai_state::AiRequest::Cancel { request_id } => {
                assert_eq!(request_id, old_request_id);
                found_cancel = true;
            }
            super::ai_state::AiRequest::Query { .. } => {
                found_query = true;
            }
        }
    }

    assert!(
        found_cancel,
        "Should have sent Cancel for in-flight request"
    );
    assert!(found_query, "Should have sent new Query request");
}

// Test: handle_query_result wrapper works correctly
#[test]
fn test_handle_query_result_wrapper() {
    let mut ai_state = AiState::new(true, 1000);
    ai_state.enabled = true;
    ai_state.set_last_query_hash(".old");

    // Test with generic type that implements ToString
    let result: Result<&str, String> = Ok("output");
    handle_query_result(&mut ai_state, &result, ".new", 4, "{}");

    // Should have updated query hash
    assert!(!ai_state.is_query_changed(".new"));
}

// =========================================================================
// Integration Tests for Full Flow
// Tests the complete execution result → AI request flow
// =========================================================================

/// Test: query change → jq executes → error result → cancel → AI request with error
/// Validates the full flow for error results
#[test]
fn test_full_flow_error_result() {
    let mut ai_state = AiState::new(true, 1000);
    ai_state.enabled = true;
    ai_state.visible = true; // Popup must be visible for requests to be sent
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Simulate initial query
    ai_state.set_last_query_hash(".initial");

    // Start an in-flight request (simulating previous query)
    ai_state.start_request();
    let old_request_id = ai_state.current_request_id();

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
    // 1. Cancel was sent for old request
    // 2. New Query request was sent with error context
    let mut found_cancel = false;
    let mut found_query = false;
    let mut query_prompt = String::new();

    while let Ok(msg) = rx.try_recv() {
        match msg {
            super::ai_state::AiRequest::Cancel { request_id } => {
                assert_eq!(
                    request_id, old_request_id,
                    "Cancel should be for old request"
                );
                found_cancel = true;
            }
            super::ai_state::AiRequest::Query { prompt, .. } => {
                found_query = true;
                query_prompt = prompt;
            }
        }
    }

    assert!(found_cancel, "Should have cancelled in-flight request");
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

/// Test: query change → jq executes → success result → cancel → AI request with output
/// Validates the full flow for success results
#[test]
fn test_full_flow_success_result() {
    let mut ai_state = AiState::new(true, 1000);
    ai_state.enabled = true;
    ai_state.visible = true; // Popup must be visible for requests to be sent
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);

    // Simulate initial query
    ai_state.set_last_query_hash(".initial");

    // Start an in-flight request (simulating previous query)
    ai_state.start_request();
    let old_request_id = ai_state.current_request_id();

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
    // 1. Cancel was sent for old request
    // 2. New Query request was sent with success context
    let mut found_cancel = false;
    let mut found_query = false;
    let mut query_prompt = String::new();

    while let Ok(msg) = rx.try_recv() {
        match msg {
            super::ai_state::AiRequest::Cancel { request_id } => {
                assert_eq!(
                    request_id, old_request_id,
                    "Cancel should be for old request"
                );
                found_cancel = true;
            }
            super::ai_state::AiRequest::Query { prompt, .. } => {
                found_query = true;
                query_prompt = prompt;
            }
        }
    }

    assert!(found_cancel, "Should have cancelled in-flight request");
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

/// Test: rapid typing → multiple jq executions → only last result triggers AI request
/// Validates that rapid query changes result in proper cancellation
#[test]
fn test_rapid_typing_only_last_result_triggers() {
    let mut ai_state = AiState::new(true, 1000);
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
    let mut cancel_count = 0;
    let mut query_count = 0;
    let mut last_query_request_id = 0;

    while let Ok(msg) = rx.try_recv() {
        match msg {
            super::ai_state::AiRequest::Cancel { .. } => {
                cancel_count += 1;
            }
            super::ai_state::AiRequest::Query { request_id, .. } => {
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

    // Should have 3 Cancel requests (for the first 3 queries)
    assert_eq!(cancel_count, 3, "Should have sent 3 Cancel requests");

    // The last Query request should have the latest request_id
    assert_eq!(
        last_query_request_id, last_request_id,
        "Last Query should have latest request_id"
    );
}

/// Test: same query repeated → no duplicate AI requests
/// Validates that identical queries don't trigger new requests
#[test]
fn test_same_query_no_duplicate_requests() {
    let mut ai_state = AiState::new(true, 1000);
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

/// Test: AI disabled → no requests sent
/// Validates that AI requests are not sent when AI is disabled
#[test]
fn test_ai_disabled_no_requests() {
    let mut ai_state = AiState::new(true, 1000);
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

// =========================================================================
// Tests for visibility-based AI request behavior (Phase 2.5)
// Validates that AI requests are sent when popup is visible
// =========================================================================

/// Test: visible=true → AI requests sent on error
/// Validates that error analysis sends AI requests when popup is visible
#[test]
fn test_visible_sends_requests_on_error() {
    let mut ai_state = AiState::new(true, 1000);
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
    match request.unwrap() {
        super::ai_state::AiRequest::Query { prompt, .. } => {
            assert!(
                prompt.contains("troubleshoot"),
                "Error prompt should mention troubleshooting"
            );
            assert!(
                prompt.contains("syntax error"),
                "Error prompt should contain error message"
            );
        }
        _ => panic!("Expected Query request"),
    }
}

/// Test: visible=false → no AI requests on error
/// Validates that no requests are sent when popup is hidden
#[test]
fn test_hidden_no_requests_on_error() {
    let mut ai_state = AiState::new(true, 1000);
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
/// Validates that success results also send AI requests when popup is visible
#[test]
fn test_visible_sends_requests_on_success() {
    let mut ai_state = AiState::new(true, 1000);
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
    match request.unwrap() {
        super::ai_state::AiRequest::Query { prompt, .. } => {
            assert!(
                prompt.contains("optimize"),
                "Success prompt should mention optimization"
            );
        }
        _ => panic!("Expected Query request"),
    }
}

/// Test: visible=false with success → no AI request sent
/// Validates that no requests are sent when popup is hidden, even for success
#[test]
fn test_hidden_no_requests_on_success() {
    let mut ai_state = AiState::new(true, 1000);
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

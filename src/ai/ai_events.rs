//! AI event handling
//!
//! Handles keyboard events (Ctrl+A toggle, Esc close) and response channel polling.
//!
//! The AI request flow is triggered by jq execution results:
//! - Query changes → jq executes → result available → cancel in-flight → debounce → AI request
//! - Both success and error results trigger AI requests with appropriate context

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::mpsc::TryRecvError;

use super::ai_state::{AiResponse, AiState};
use super::context::QueryContext;
use super::prompt::build_prompt;

/// Handle Ctrl+A to toggle AI popup visibility
///
/// Returns true if the key was handled, false otherwise.
///
/// # Requirements
/// - 2.1: WHEN a user presses Ctrl+A THEN the AI_Popup SHALL toggle its visibility state
// TODO: Remove #[allow(dead_code)] when this function is used
#[allow(dead_code)] // Phase 1: Not used yet, handled in global keys
pub fn handle_toggle_key(key: KeyEvent, ai_state: &mut AiState) -> bool {
    if key.code == KeyCode::Char('a') && key.modifiers.contains(KeyModifiers::CONTROL) {
        ai_state.toggle();
        return true;
    }
    false
}

/// Handle Esc to close AI popup when visible
///
/// Returns true if the key was handled (popup was closed), false otherwise.
///
/// Note: In Phase 1, ESC does NOT close the popup - only Ctrl+A toggles it
// TODO: Remove #[allow(dead_code)] if this function is needed in future
#[allow(dead_code)] // Phase 1: Not used, ESC doesn't close popup
pub fn handle_close_key(key: KeyEvent, ai_state: &mut AiState) -> bool {
    if key.code == KeyCode::Esc && ai_state.visible {
        ai_state.close();
        return true;
    }
    false
}

/// Poll the response channel for incoming AI responses
///
/// This should be called in the main event loop to process streaming responses.
/// Uses try_recv() for non-blocking polling.
///
/// # Requirements
/// - 4.1: WHEN the AI provider sends a streaming response THEN the AI_Popup SHALL display text incrementally
/// - 4.4: WHEN the streaming response completes THEN the AI_Popup SHALL remove the loading indicator
pub fn poll_response_channel(ai_state: &mut AiState) {
    // Only poll if we have a receiver channel
    if ai_state.response_rx.is_none() {
        return;
    }

    // Collect responses first to avoid borrow issues
    let mut responses = Vec::new();
    let mut disconnected = false;

    if let Some(ref rx) = ai_state.response_rx {
        loop {
            match rx.try_recv() {
                Ok(response) => {
                    responses.push(response);
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }
    }

    // Process collected responses
    for response in responses {
        process_response(ai_state, response);
    }

    // Handle disconnection after processing
    if disconnected && ai_state.loading {
        ai_state.set_error("AI worker disconnected unexpectedly".to_string());
    }
}

/// Handle AI state after jq execution completes
///
/// This is the single entry point for updating AI state based on execution results.
/// Called AFTER jq execution completes with the result (success OR error).
///
/// The execution result flow:
/// - jq executes → result available → cancel in-flight → clear stale → send AI request
/// - Both success and error results trigger AI requests with appropriate context
///
/// # Arguments
/// * `ai_state` - The AI state to update
/// * `query_result` - The result of the query execution (Ok with output or Err with message)
/// * `auto_show_on_error` - Whether to auto-show the popup on error (controls visibility only)
/// * `query` - The current query text
/// * `cursor_pos` - The cursor position in the query
/// * `json_input` - The JSON input being queried
///
/// # Behavior
/// - Query changed + error: Cancel in-flight, clear stale, send AI request with error context
/// - Query changed + success: Cancel in-flight, clear stale, send AI request with output context
/// - Query unchanged: Do nothing (no duplicate requests)
///
/// # Requirements
/// - 3.1: WHEN a jq query produces an error AND `auto_show_on_error` is true
///        THEN the AI_Popup SHALL automatically open
/// - 3.2: WHEN `auto_show_on_error` is false THEN the AI_Popup SHALL remain
///        closed on query errors until manually opened with Ctrl+A
/// - 3.3: WHEN a query executes with an error THEN the AI_Assistant SHALL send
///        the error context to the AI provider
/// - 3.4: WHEN a query executes successfully THEN the AI_Assistant SHALL send
///        the success context to the AI provider
/// - 3.5: WHEN a new query change occurs THEN the AI_Assistant SHALL cancel
///        any in-flight API request
pub fn handle_execution_result(
    ai_state: &mut AiState,
    query_result: &Result<String, String>,
    auto_show_on_error: bool,
    query: &str,
    cursor_pos: usize,
    json_input: &str,
) {
    // Check if query has changed - this is the ONLY trigger for AI actions
    let query_changed = ai_state.is_query_changed(query);

    // If query hasn't changed, do nothing (no duplicate requests)
    if !query_changed {
        return;
    }

    // Cancel any in-flight request before processing new result
    // Requirements 3.5, 5.4
    ai_state.cancel_in_flight_request();

    // Clear stale response from previous query
    ai_state.clear_stale_response();

    // Update query hash to track this query
    ai_state.set_last_query_hash(query);

    match query_result {
        Err(error) => {
            // Optionally auto-show popup based on config (error only)
            if auto_show_on_error {
                ai_state.auto_show_on_error(true);
            }

            // Send AI request with error context
            if ai_state.enabled {
                let context = QueryContext::new(
                    query.to_string(),
                    cursor_pos,
                    json_input,
                    None,
                    Some(error.to_string()),
                );
                let prompt = build_prompt(&context);
                ai_state.send_request(prompt);
            }
        }
        Ok(output) => {
            // Send AI request with success context (for optimization suggestions)
            if ai_state.enabled {
                let context = QueryContext::new(
                    query.to_string(),
                    cursor_pos,
                    json_input,
                    Some(output.clone()),
                    None,
                );
                let prompt = build_prompt(&context);
                ai_state.send_request(prompt);
            }
        }
    }
}

/// Handle AI state after query execution (legacy wrapper)
///
/// This function wraps handle_execution_result for backward compatibility.
/// It converts the generic Result<T, String> to Result<String, String>.
///
/// # Deprecated
/// Use handle_execution_result directly when possible.
pub fn handle_query_result<T: ToString>(
    ai_state: &mut AiState,
    query_result: &Result<T, String>,
    auto_show_on_error: bool,
    query: &str,
    cursor_pos: usize,
    json_input: &str,
) {
    // Convert to Result<String, String> for the unified handler
    let result: Result<String, String> = match query_result {
        Ok(output) => Ok(output.to_string()),
        Err(e) => Err(e.clone()),
    };

    handle_execution_result(
        ai_state,
        &result,
        auto_show_on_error,
        query,
        cursor_pos,
        json_input,
    );
}

/// Process a single AI response message
///
/// Filters stale responses by checking request_id against the current
/// AiState request_id. Responses from old requests are ignored.
///
/// # Requirements
/// - 4.1: Streaming concatenation - chunks are appended to response
/// - 4.2: Loading state during request
/// - 4.4: Complete removes loading indicator
/// - 5.3: Stale responses are filtered out
fn process_response(ai_state: &mut AiState, response: AiResponse) {
    let current_request_id = ai_state.current_request_id();

    match response {
        AiResponse::Chunk { text, request_id } => {
            // Filter stale responses from old requests
            if request_id < current_request_id {
                log::debug!(
                    "Ignoring stale chunk from request {} (current: {})",
                    request_id,
                    current_request_id
                );
                return;
            }
            // Append chunk to current response (streaming concatenation)
            ai_state.append_chunk(&text);
        }
        AiResponse::Complete { request_id } => {
            // Filter stale responses from old requests
            if request_id < current_request_id {
                log::debug!(
                    "Ignoring stale complete from request {} (current: {})",
                    request_id,
                    current_request_id
                );
                return;
            }
            // Mark request as complete, clear loading state
            ai_state.complete_request();
        }
        AiResponse::Error(error_msg) => {
            // Errors are not filtered - always show the latest error
            // Set error state, preserves previous response
            ai_state.set_error(error_msg);
        }
        AiResponse::Cancelled { request_id } => {
            // Filter stale cancelled responses
            if request_id < current_request_id {
                log::debug!(
                    "Ignoring stale cancelled from request {} (current: {})",
                    request_id,
                    current_request_id
                );
                return;
            }
            // Request was cancelled, clear loading state and in_flight_request_id
            ai_state.loading = false;
            ai_state.in_flight_request_id = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use ratatui::crossterm::event::KeyEvent;
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
        handle_execution_result(&mut ai_state, &result, true, ".valid", 6, "{}");

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
        handle_execution_result(&mut ai_state, &result, false, ".new", 4, "{}");

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
        handle_execution_result(&mut ai_state, &result, false, ".query2", 7, "{}");

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
        handle_execution_result(&mut ai_state, &result, false, ".same", 5, "{}");

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
        handle_execution_result(&mut ai_state, &result, false, ".same", 5, "{}");

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
        handle_execution_result(&mut ai_state, &result, false, ".query2", 7, "{}");

        // Response should be cleared because query changed
        assert!(ai_state.response.is_empty());
    }

    // Test: successful query triggers AI request with output context
    #[test]
    fn test_success_triggers_ai_request() {
        use std::sync::mpsc;

        let mut ai_state = AiState::new(true, 1000);
        ai_state.enabled = true;
        let (tx, rx) = mpsc::channel();
        ai_state.request_tx = Some(tx);

        // Simulate successful query result
        let result: Result<String, String> = Ok("output data".to_string());
        handle_execution_result(
            &mut ai_state,
            &result,
            false,
            ".name",
            5,
            r#"{"name": "test"}"#,
        );

        // Should have sent a request
        let request = rx.try_recv();
        assert!(request.is_ok(), "Should have sent AI request for success");

        // Verify it's a Query request
        match request.unwrap() {
            super::super::ai_state::AiRequest::Query { prompt, .. } => {
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
        use std::sync::mpsc;

        let mut ai_state = AiState::new(true, 1000);
        ai_state.enabled = true;
        let (tx, rx) = mpsc::channel();
        ai_state.request_tx = Some(tx);

        // Simulate error query result
        let result: Result<String, String> = Err("syntax error".to_string());
        handle_execution_result(
            &mut ai_state,
            &result,
            false,
            ".invalid",
            8,
            r#"{"name": "test"}"#,
        );

        // Should have sent a request
        let request = rx.try_recv();
        assert!(request.is_ok(), "Should have sent AI request for error");

        // Verify it's a Query request
        match request.unwrap() {
            super::super::ai_state::AiRequest::Query { prompt, .. } => {
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
        use std::sync::mpsc;

        let mut ai_state = AiState::new(true, 1000);
        ai_state.enabled = true;
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
        handle_execution_result(&mut ai_state, &result, false, ".new", 4, "{}");

        // Should have sent Cancel for old request, then Query for new
        let mut found_cancel = false;
        let mut found_query = false;

        while let Ok(msg) = rx.try_recv() {
            match msg {
                super::super::ai_state::AiRequest::Cancel { request_id } => {
                    assert_eq!(request_id, old_request_id);
                    found_cancel = true;
                }
                super::super::ai_state::AiRequest::Query { .. } => {
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
        handle_query_result(&mut ai_state, &result, false, ".new", 4, "{}");

        // Should have updated query hash
        assert!(!ai_state.is_query_changed(".new"));
    }

    // =========================================================================
    // Integration Tests for Full Flow (Task 20.5)
    // Tests the complete execution result → AI request flow
    // **Validates: Requirements 3.3, 3.4, 3.5, 5.4**
    // =========================================================================

    /// Test: query change → jq executes → error result → cancel → AI request with error
    /// Validates the full flow for error results
    #[test]
    fn test_full_flow_error_result() {
        use std::sync::mpsc;

        let mut ai_state = AiState::new(true, 1000);
        ai_state.enabled = true;
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
            true, // auto_show_on_error
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
                super::super::ai_state::AiRequest::Cancel { request_id } => {
                    assert_eq!(
                        request_id, old_request_id,
                        "Cancel should be for old request"
                    );
                    found_cancel = true;
                }
                super::super::ai_state::AiRequest::Query { prompt, .. } => {
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
        assert!(
            ai_state.visible,
            "Popup should be visible (auto_show_on_error=true)"
        );
    }

    /// Test: query change → jq executes → success result → cancel → AI request with output
    /// Validates the full flow for success results
    #[test]
    fn test_full_flow_success_result() {
        use std::sync::mpsc;

        let mut ai_state = AiState::new(true, 1000);
        ai_state.enabled = true;
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
            false, // auto_show_on_error (doesn't apply to success)
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
                super::super::ai_state::AiRequest::Cancel { request_id } => {
                    assert_eq!(
                        request_id, old_request_id,
                        "Cancel should be for old request"
                    );
                    found_cancel = true;
                }
                super::super::ai_state::AiRequest::Query { prompt, .. } => {
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
        use std::sync::mpsc;

        let mut ai_state = AiState::new(true, 1000);
        ai_state.enabled = true;
        let (tx, rx) = mpsc::channel();
        ai_state.request_tx = Some(tx);

        // Simulate rapid typing: .n → .na → .nam → .name
        let queries = vec![".n", ".na", ".nam", ".name"];
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
                false,
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
                super::super::ai_state::AiRequest::Cancel { .. } => {
                    cancel_count += 1;
                }
                super::super::ai_state::AiRequest::Query { request_id, .. } => {
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
        use std::sync::mpsc;

        let mut ai_state = AiState::new(true, 1000);
        ai_state.enabled = true;
        let (tx, rx) = mpsc::channel();
        ai_state.request_tx = Some(tx);

        // First execution
        let result: Result<String, String> = Ok(r#""test""#.to_string());
        handle_execution_result(
            &mut ai_state,
            &result,
            false,
            ".name",
            5,
            r#"{"name": "test"}"#,
        );

        // Drain channel
        while rx.try_recv().is_ok() {}

        // Same query executed again (e.g., user pressed Enter)
        handle_execution_result(
            &mut ai_state,
            &result,
            false,
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
        use std::sync::mpsc;

        let mut ai_state = AiState::new(true, 1000);
        ai_state.enabled = false; // AI disabled
        let (tx, rx) = mpsc::channel();
        ai_state.request_tx = Some(tx);

        // Execute query
        let result: Result<String, String> = Ok(r#""test""#.to_string());
        handle_execution_result(
            &mut ai_state,
            &result,
            false,
            ".name",
            5,
            r#"{"name": "test"}"#,
        );

        // Should NOT have sent any requests
        let request = rx.try_recv();
        assert!(
            request.is_err(),
            "Should not send request when AI is disabled"
        );
    }
}

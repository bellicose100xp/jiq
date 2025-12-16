//! Tests for AI state management

use super::{AiRequest, AiState};
use crate::ai::suggestion::{Suggestion, SuggestionType};
use proptest::prelude::*;

#[test]
fn test_new_ai_state_disabled() {
    let state = AiState::new(false);
    assert!(!state.visible);
    assert!(!state.enabled);
    assert!(!state.configured);
    assert!(!state.loading);
    assert!(state.error.is_none());
    assert!(state.response.is_empty());
    assert!(state.previous_response.is_none());
}

#[test]
fn test_new_ai_state_enabled() {
    let state = AiState::new(true);
    assert!(!state.visible);
    assert!(state.enabled);
    assert!(!state.configured);
    assert!(!state.loading);
}

#[test]
fn test_new_with_config_configured() {
    let state = AiState::new_with_config(true, true);
    assert!(state.visible);
    assert!(state.enabled);
    assert!(state.configured);
    assert!(!state.loading);
}

#[test]
fn test_new_with_config_not_configured() {
    let state = AiState::new_with_config(true, false);
    assert!(state.visible);
    assert!(state.enabled);
    assert!(!state.configured);
    assert!(!state.loading);
}

#[test]
fn test_toggle_visibility() {
    let mut state = AiState::new(true);
    assert!(!state.visible);
    state.toggle();
    assert!(state.visible);
    state.toggle();
    assert!(!state.visible);
}

#[test]
fn test_close() {
    let mut state = AiState::new(true);
    state.visible = true;
    state.close();
    assert!(!state.visible);
}

#[test]
fn test_start_request_preserves_response() {
    let mut state = AiState::new(true);
    state.response = "previous answer".to_string();
    state.start_request();
    assert!(state.loading);
    assert!(state.response.is_empty());
    assert_eq!(state.previous_response, Some("previous answer".to_string()));
}

#[test]
fn test_start_request_empty_response() {
    let mut state = AiState::new(true);
    state.start_request();
    assert!(state.loading);
    assert!(state.response.is_empty());
    assert!(state.previous_response.is_none());
}

#[test]
fn test_append_chunk() {
    let mut state = AiState::new(true);
    state.append_chunk("Hello ");
    state.append_chunk("World");
    assert_eq!(state.response, "Hello World");
}

#[test]
fn test_complete_request() {
    let mut state = AiState::new(true);
    state.loading = true;
    state.previous_response = Some("old".to_string());
    state.complete_request();
    assert!(!state.loading);
    assert!(state.previous_response.is_none());
}

#[test]
fn test_set_error() {
    let mut state = AiState::new(true);
    state.loading = true;
    state.set_error("Network error".to_string());
    assert!(!state.loading);
    assert_eq!(state.error, Some("Network error".to_string()));
}

#[test]
fn test_clear_on_success() {
    let mut state = AiState::new(true);
    state.visible = true;
    state.response = "Error explanation".to_string();
    state.error = Some("Query error".to_string());
    state.previous_response = Some("Old response".to_string());
    state.loading = true;

    state.clear_on_success();

    // Response and error should be cleared
    assert!(state.response.is_empty());
    assert!(state.error.is_none());
    assert!(state.previous_response.is_none());
    assert!(!state.loading);
    // Visibility should be preserved (don't auto-close)
    assert!(state.visible);
}

#[test]
fn test_clear_stale_response() {
    let mut state = AiState::new(true);
    state.visible = true;
    state.response = "Old error explanation".to_string();
    state.error = Some("Old query error".to_string());
    state.previous_response = Some("Previous response".to_string());
    state.loading = true;

    state.clear_stale_response();

    // Response and error should be cleared
    assert!(state.response.is_empty());
    assert!(state.error.is_none());
    assert!(state.previous_response.is_none());
    assert!(!state.loading);
    // Visibility should be preserved
    assert!(state.visible);
}

#[test]
fn test_default() {
    let state = AiState::default();
    assert!(!state.enabled);
    assert!(!state.configured);
    assert!(!state.visible);
}

#[test]
fn test_request_id_increments() {
    let mut state = AiState::new(true);
    assert_eq!(state.request_id, 0);

    state.start_request();
    assert_eq!(state.request_id, 1);

    state.start_request();
    assert_eq!(state.request_id, 2);
}

#[test]
fn test_is_query_changed_no_previous() {
    let state = AiState::new(true);
    assert!(state.is_query_changed(".name"));
}

#[test]
fn test_is_query_changed_same_query() {
    let mut state = AiState::new(true);
    state.set_last_query_hash(".name");
    assert!(!state.is_query_changed(".name"));
}

#[test]
fn test_is_query_changed_different_query() {
    let mut state = AiState::new(true);
    state.set_last_query_hash(".name");
    assert!(state.is_query_changed(".age"));
}

#[test]
fn test_same_query_no_new_request() {
    let mut state = AiState::new(true);
    state.set_last_query_hash(".name");

    assert!(!state.is_query_changed(".name"));
}

#[test]
fn test_different_query_triggers_new_request() {
    let mut state = AiState::new(true);
    state.set_last_query_hash(".name");

    assert!(state.is_query_changed(".age"));
}

// **Feature: ai-assistant, Property 5: Toggle visibility**
// *For any* AiState with visibility V, calling toggle() should result in visibility !V.
// **Validates: Requirements 2.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_toggle_visibility(initial_visible: bool, enabled: bool) {
        let mut state = AiState::new(enabled);
        state.visible = initial_visible;

        let expected = !initial_visible;
        state.toggle();

        prop_assert_eq!(
            state.visible, expected,
            "Toggle should flip visibility from {} to {}",
            initial_visible, expected
        );

        // Toggle again should return to original
        state.toggle();
        prop_assert_eq!(
            state.visible, initial_visible,
            "Double toggle should return to original visibility"
        );
    }
}

// **Feature: ai-assistant, Property 6: Toggle is the only way to close popup**
// *For any* AiState with `visible = true`, only calling toggle() should change visibility.
// **Validates: Requirements 2.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_toggle_is_only_way_to_close(enabled: bool, has_response: bool, response in "[a-zA-Z ]{0,100}") {
        let mut state = AiState::new(enabled);
        state.visible = true;

        if has_response {
            state.response = response;
        }

        // Toggle should close the popup
        state.toggle();

        prop_assert!(
            !state.visible,
            "Toggle should close visible popup"
        );

        // Toggle again should open it
        state.toggle();

        prop_assert!(
            state.visible,
            "Toggle should open closed popup"
        );
    }
}

// **Feature: ai-assistant, Property 12: Previous response preservation**
// *For any* AiState with non-empty `response`, starting a new request should set
// `previous_response` to the current response.
// **Validates: Requirements 4.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_previous_response_preservation(
        enabled: bool,
        response in "[a-zA-Z0-9 ]{1,100}"
    ) {
        let mut state = AiState::new(enabled);
        state.response = response.clone();

        state.start_request();

        prop_assert_eq!(
            state.previous_response,
            Some(response.clone()),
            "Previous response should be preserved when starting new request"
        );
        prop_assert!(
            state.response.is_empty(),
            "Current response should be cleared when starting new request"
        );
        prop_assert!(
            state.loading,
            "Loading should be true after starting request"
        );
    }

    #[test]
    fn prop_empty_response_not_preserved(enabled: bool) {
        let mut state = AiState::new(enabled);
        // Response is empty by default

        state.start_request();

        prop_assert!(
            state.previous_response.is_none(),
            "Empty response should not be preserved"
        );
    }
}

// **Feature: ai-assistant, Property 19: Response cleared on successful query**
// *For any* AiState with response and/or error set, calling clear_on_success()
// should clear response, error, and previous_response while preserving visibility.
// **Validates: Requirements 3.1, 4.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_response_cleared_on_success(
        enabled: bool,
        initial_visible in prop::bool::ANY,
        response in "[a-zA-Z0-9 ]{0,100}",
        error in prop::option::of("[a-zA-Z0-9 ]{1,50}"),
        previous_response in prop::option::of("[a-zA-Z0-9 ]{1,50}"),
        loading in prop::bool::ANY,
    ) {
        let mut state = AiState::new(enabled);
        state.visible = initial_visible;
        state.response = response;
        state.error = error;
        state.previous_response = previous_response;
        state.loading = loading;

        state.clear_on_success();

        // Response and error should be cleared
        prop_assert!(
            state.response.is_empty(),
            "Response should be cleared on success"
        );
        prop_assert!(
            state.error.is_none(),
            "Error should be cleared on success"
        );
        prop_assert!(
            state.previous_response.is_none(),
            "Previous response should be cleared on success"
        );
        prop_assert!(
            !state.loading,
            "Loading should be false after clear_on_success"
        );
        // Visibility should be preserved (don't auto-close)
        prop_assert_eq!(
            state.visible, initial_visible,
            "Visibility should be preserved after clear_on_success"
        );
    }
}

#[test]
fn test_start_request_sets_in_flight_request_id() {
    let mut state = AiState::new(true);
    assert!(state.in_flight_request_id.is_none());

    state.start_request();
    assert_eq!(state.in_flight_request_id, Some(1));

    state.start_request();
    assert_eq!(state.in_flight_request_id, Some(2));
}

#[test]
fn test_complete_request_clears_in_flight_request_id() {
    let mut state = AiState::new(true);
    state.start_request();
    assert!(state.in_flight_request_id.is_some());

    state.complete_request();
    assert!(state.in_flight_request_id.is_none());
}

#[test]
fn test_set_error_clears_in_flight_request_id() {
    let mut state = AiState::new(true);
    state.start_request();
    assert!(state.in_flight_request_id.is_some());

    state.set_error("test error".to_string());
    assert!(state.in_flight_request_id.is_none());
}

#[test]
fn test_has_in_flight_request() {
    let mut state = AiState::new(true);
    assert!(!state.has_in_flight_request());

    state.start_request();
    assert!(state.has_in_flight_request());

    state.complete_request();
    assert!(!state.has_in_flight_request());
}

#[test]
fn test_cancel_in_flight_request_without_channel() {
    let mut state = AiState::new(true);
    state.start_request();

    // Without a channel, cancel should return false
    let result = state.cancel_in_flight_request();
    assert!(!result);
}

#[test]
fn test_cancel_in_flight_request_with_channel() {
    use std::sync::mpsc;

    let mut state = AiState::new(true);
    let (tx, rx) = mpsc::channel();
    state.request_tx = Some(tx);
    state.start_request();
    let request_id = state.request_id;

    // With a channel and in-flight request, cancel should return true
    let result = state.cancel_in_flight_request();
    assert!(result);
    assert!(state.in_flight_request_id.is_none());

    // Verify the cancel message was sent
    let msg = rx.recv().unwrap();
    assert!(matches!(msg, AiRequest::Cancel { request_id: id } if id == request_id));
}

#[test]
fn test_cancel_in_flight_request_no_active_request() {
    use std::sync::mpsc;

    let mut state = AiState::new(true);
    let (tx, _rx) = mpsc::channel();
    state.request_tx = Some(tx);
    // Don't start a request

    // Without an in-flight request, cancel should return false
    let result = state.cancel_in_flight_request();
    assert!(!result);
}

// **Feature: ai-assistant, Property 21: Query change cancels in-flight request**
// *For any* in-flight AI request, a new query change should send a Cancel message
// before starting the debounce timer.
// **Validates: Requirements 3.5, 5.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_cancel_in_flight_request_sends_cancel(
        enabled: bool,
    ) {
        use std::sync::mpsc;

        let mut state = AiState::new(enabled);
        let (tx, rx) = mpsc::channel();
        state.request_tx = Some(tx);

        // Start a request to create an in-flight request
        state.start_request();
        let in_flight_id = state.request_id;
        prop_assert!(state.has_in_flight_request(), "Should have in-flight request");

        // Cancel the in-flight request
        let cancelled = state.cancel_in_flight_request();
        prop_assert!(cancelled, "Should successfully cancel in-flight request");
        prop_assert!(!state.has_in_flight_request(), "Should clear in-flight request");

        // Verify cancel message was sent with correct request_id
        let msg = rx.try_recv();
        prop_assert!(msg.is_ok(), "Should have sent cancel message");
        match msg.unwrap() {
            AiRequest::Cancel { request_id } => {
                prop_assert_eq!(request_id, in_flight_id, "Cancel should have correct request_id");
            }
            _ => prop_assert!(false, "Should have sent Cancel message"),
        }
    }

    #[test]
    fn prop_no_cancel_without_in_flight_request(
        enabled: bool,
    ) {
        use std::sync::mpsc;

        let mut state = AiState::new(enabled);
        let (tx, rx) = mpsc::channel();
        state.request_tx = Some(tx);

        // Don't start a request - no in-flight request
        prop_assert!(!state.has_in_flight_request(), "Should not have in-flight request");

        // Try to cancel
        let cancelled = state.cancel_in_flight_request();
        prop_assert!(!cancelled, "Should not cancel when no in-flight request");

        // Verify no message was sent
        let msg = rx.try_recv();
        prop_assert!(msg.is_err(), "Should not have sent any message");
    }
}

#[test]
fn test_complete_request_parses_suggestions() {
    let mut state = AiState::new(true);
    state.response = "1. [Fix] .users[]\n   Fix the query".to_string();
    state.loading = true;

    state.complete_request();

    assert!(!state.loading);
    assert_eq!(state.suggestions.len(), 1);
    assert_eq!(state.suggestions[0].query, ".users[]");
}

#[test]
fn test_start_request_clears_suggestions() {
    let mut state = AiState::new(true);
    state.suggestions = vec![Suggestion {
        query: ".test".to_string(),
        description: "Test".to_string(),
        suggestion_type: SuggestionType::Fix,
    }];

    state.start_request();

    assert!(state.suggestions.is_empty());
}

// **Feature: ai-assistant-phase2, Property 12: Initial visibility matches config**
// *For any* startup with AI enabled in config, the AI popup SHALL be visible by default.
// **Validates: Requirements 8.1, 8.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_initial_visibility_matches_config(
        ai_enabled in prop::bool::ANY,
        configured in prop::bool::ANY,
    ) {
        let state = AiState::new_with_config(ai_enabled, configured);

        // Visibility should match enabled state
        prop_assert_eq!(
            state.visible,
            ai_enabled,
            "Initial visibility should be {} when AI is {}",
            ai_enabled,
            if ai_enabled { "enabled" } else { "disabled" }
        );

        // Enabled and configured should match inputs
        prop_assert_eq!(state.enabled, ai_enabled, "Enabled should match input");
        prop_assert_eq!(state.configured, configured, "Configured should match input");
    }
}

#[test]
fn test_selection_initialized_in_new() {
    let state = AiState::new(true);
    assert!(state.selection.get_selected().is_none());
    assert!(!state.selection.is_navigation_active());
}

#[test]
fn test_selection_initialized_in_new_with_config() {
    let state = AiState::new_with_config(true, true);
    assert!(state.selection.get_selected().is_none());
    assert!(!state.selection.is_navigation_active());
}

#[test]
fn test_selection_initialized_in_default() {
    let state = AiState::default();
    assert!(state.selection.get_selected().is_none());
    assert!(!state.selection.is_navigation_active());
}

#[test]
fn test_selection_cleared_on_new_request() {
    let mut state = AiState::new(true);

    // Set up a selection
    state.selection.select_index(2);
    assert_eq!(state.selection.get_selected(), Some(2));

    // Start a new request
    state.start_request();

    // Selection should be cleared
    assert!(state.selection.get_selected().is_none());
    assert!(!state.selection.is_navigation_active());
}

#[test]
fn test_selection_cleared_on_new_request_with_navigation() {
    let mut state = AiState::new(true);

    // Set up navigation mode
    state.selection.navigate_next(5);
    assert!(state.selection.is_navigation_active());
    assert_eq!(state.selection.get_selected(), Some(0));

    // Start a new request
    state.start_request();

    // Selection and navigation mode should be cleared
    assert!(state.selection.get_selected().is_none());
    assert!(!state.selection.is_navigation_active());
}

#[test]
fn test_selection_persists_during_response_streaming() {
    let mut state = AiState::new(true);

    // Set up a selection
    state.selection.select_index(1);
    assert_eq!(state.selection.get_selected(), Some(1));

    // Simulate response streaming (append_chunk doesn't clear selection)
    state.append_chunk("1. [Fix] .users[]\n");
    state.append_chunk("   Fix the query\n");

    // Selection should persist
    assert_eq!(state.selection.get_selected(), Some(1));
}

#[test]
fn test_selection_persists_after_complete_request() {
    let mut state = AiState::new(true);
    state.loading = true;
    state.response = "1. [Fix] .users[]\n   Fix the query".to_string();

    // Set up a selection
    state.selection.select_index(0);
    assert_eq!(state.selection.get_selected(), Some(0));

    // Complete the request
    state.complete_request();

    // Selection should persist (user may want to apply it)
    assert_eq!(state.selection.get_selected(), Some(0));
}

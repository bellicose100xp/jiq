//! Tests for AI state management

use super::AiState;
use super::lifecycle::TEST_MAX_CONTEXT_LENGTH;
use crate::ai::suggestion::{Suggestion, SuggestionType};

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
    let state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
    assert!(state.visible);
    assert!(state.enabled);
    assert!(state.configured);
    assert!(!state.loading);
}

#[test]
fn test_new_with_config_not_configured() {
    let state = AiState::new_with_config(
        true,
        false,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
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
fn test_cancel_in_flight_request_without_active_request() {
    let mut state = AiState::new(true);
    // No in-flight request

    // Without an in-flight request, cancel should return false
    let result = state.cancel_in_flight_request();
    assert!(!result);
}

#[test]
fn test_cancel_in_flight_request_with_active_request() {
    use tokio_util::sync::CancellationToken;

    let mut state = AiState::new(true);
    state.start_request();
    // Set up a cancellation token (simulating what send_request does)
    let token = CancellationToken::new();
    state.current_cancel_token = Some(token.clone());

    // With an in-flight request and token, cancel should return true and clear the request
    let result = state.cancel_in_flight_request();
    assert!(result);
    assert!(state.in_flight_request_id.is_none());
    assert!(state.current_cancel_token.is_none());
    assert!(token.is_cancelled());
}

#[test]
fn test_cancel_in_flight_request_clears_request_id() {
    use tokio_util::sync::CancellationToken;

    let mut state = AiState::new(true);
    state.start_request();
    assert!(state.in_flight_request_id.is_some());

    // Set up a cancellation token (simulating what send_request does)
    let token = CancellationToken::new();
    state.current_cancel_token = Some(token.clone());

    state.cancel_in_flight_request();
    assert!(state.in_flight_request_id.is_none());
    assert!(state.current_cancel_token.is_none());
    assert!(token.is_cancelled());
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

#[test]
fn test_selection_initialized_in_new() {
    let state = AiState::new(true);
    assert!(state.selection.get_selected().is_none());
    assert!(!state.selection.is_navigation_active());
}

#[test]
fn test_selection_initialized_in_new_with_config() {
    let state = AiState::new_with_config(
        true,
        true,
        "Anthropic".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        TEST_MAX_CONTEXT_LENGTH,
    );
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

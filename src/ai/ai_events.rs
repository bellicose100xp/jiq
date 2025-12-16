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
use super::selection::{apply::apply_suggestion, keybindings};
use crate::autocomplete::AutocompleteState;
use crate::input::InputState;
use crate::query::QueryState;

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

/// Handle suggestion selection events (Alt+1-5, Alt+Up/Down, Enter)
///
/// This function handles all suggestion selection keybindings:
/// 1. Direct selection (Alt+1-5): Immediately applies the selected suggestion
/// 2. Navigation (Alt+Up/Down): Moves selection highlight through suggestions
/// 3. Enter: Applies the currently navigated selection (if navigation is active)
///
/// Returns true if the key was handled, false otherwise.
///
/// # Arguments
/// * `key` - The key event to handle
/// * `ai_state` - The AI state containing suggestions and selection state
/// * `input_state` - The input state to modify when applying suggestions
/// * `query_state` - The query state for execution
/// * `autocomplete_state` - The autocomplete state to hide when applying
///
/// # Requirements
/// - 1.1-1.5: Alt+1-5 selects corresponding suggestion
/// - 8.1, 8.2: Alt+Up/Down navigates through suggestions
/// - 9.1: Enter applies the highlighted suggestion when navigation is active
pub fn handle_suggestion_selection(
    key: KeyEvent,
    ai_state: &mut AiState,
    input_state: &mut InputState,
    query_state: &mut QueryState,
    autocomplete_state: &mut AutocompleteState,
) -> bool {
    // Log entry for troubleshooting
    #[cfg(debug_assertions)]
    log::debug!(
        "handle_suggestion_selection: visible={}, suggestions={}, key={:?}",
        ai_state.visible,
        ai_state.suggestions.len(),
        key.code
    );

    // Only handle when AI popup is visible and has suggestions
    if !ai_state.visible || ai_state.suggestions.is_empty() {
        return false;
    }

    let suggestion_count = ai_state.suggestions.len();

    // 1. Try direct selection (Alt+1-5)
    if let Some(index) = keybindings::handle_direct_selection(key, suggestion_count) {
        #[cfg(debug_assertions)]
        log::debug!("Direct selection matched: index={}", index);

        // Apply the selected suggestion
        if let Some(suggestion) = ai_state.suggestions.get(index) {
            #[cfg(debug_assertions)]
            log::debug!("Applying suggestion: query={}", suggestion.query);

            apply_suggestion(suggestion, input_state, query_state, autocomplete_state);
            ai_state.selection.clear_selection();
            return true;
        }
    }

    // 2. Try navigation (Alt+Up/Down)
    if keybindings::handle_navigation(key, &mut ai_state.selection, suggestion_count) {
        return true;
    }

    // 3. Try Enter to apply navigated selection
    if let Some(index) = keybindings::handle_apply_selection(key, &ai_state.selection)
        && let Some(suggestion) = ai_state.suggestions.get(index)
    {
        apply_suggestion(suggestion, input_state, query_state, autocomplete_state);
        ai_state.selection.clear_selection();
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
/// * `query` - The current query text
/// * `cursor_pos` - The cursor position in the query
/// * `json_input` - The JSON input being queried
///
/// # Behavior
/// - Query changed + error: Cancel in-flight, clear stale, send AI request with error context
/// - Query changed + success: Cancel in-flight, clear stale, send AI request with output context
/// - Query unchanged: Do nothing (no duplicate requests)
/// - AI requests are sent when AI popup is visible (for both errors and success)
pub fn handle_execution_result(
    ai_state: &mut AiState,
    query_result: &Result<String, String>,
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
    ai_state.cancel_in_flight_request();

    // Clear stale response from previous query
    ai_state.clear_stale_response();

    // Update query hash to track this query
    ai_state.set_last_query_hash(query);

    match query_result {
        Err(error) => {
            // Send AI request with error context when popup is visible
            if ai_state.visible {
                let context = QueryContext::new(
                    query.to_string(),
                    cursor_pos,
                    json_input,
                    None,
                    Some(error.to_string()),
                );
                // Phase 2: Use word limit calculated from popup dimensions
                // Requirements: 2.1, 7.4
                let word_limit = ai_state.word_limit;
                let prompt = build_prompt(&context, word_limit);
                ai_state.send_request(prompt);
            }
        }
        Ok(output) => {
            // Send AI request with success context when popup is visible
            if ai_state.visible {
                let context = QueryContext::new(
                    query.to_string(),
                    cursor_pos,
                    json_input,
                    Some(output.clone()),
                    None,
                );
                // Phase 2: Use word limit calculated from popup dimensions
                // Requirements: 2.1, 7.4
                let word_limit = ai_state.word_limit;
                let prompt = build_prompt(&context, word_limit);
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
    query: &str,
    cursor_pos: usize,
    json_input: &str,
) {
    // Convert to Result<String, String> for the unified handler
    let result: Result<String, String> = match query_result {
        Ok(output) => Ok(output.to_string()),
        Err(e) => Err(e.clone()),
    };

    handle_execution_result(ai_state, &result, query, cursor_pos, json_input);
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

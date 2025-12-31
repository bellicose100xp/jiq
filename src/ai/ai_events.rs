//! AI event handling
//!
//! Handles keyboard events (Ctrl+A toggle, Esc close) and response channel polling.
//!
//! The AI request flow is triggered by jq execution results:
//! - Query changes → jq executes → result available → cancel in-flight → debounce → AI request
//! - Both success and error results trigger AI requests with appropriate context

use ratatui::crossterm::event::KeyEvent;
use std::sync::mpsc::TryRecvError;

use super::ai_state::{AiResponse, AiState};
use super::context::{ContextParams, QueryContext};
use super::prompt::build_prompt;
use super::selection::{apply::apply_suggestion, keybindings};
use crate::autocomplete::AutocompleteState;
use crate::input::InputState;
use crate::query::QueryState;

/// Handle suggestion selection events (Alt+1-5, Alt+Up/Down/j/k, Enter)
///
/// This function handles all suggestion selection keybindings:
/// 1. Direct selection (Alt+1-5): Immediately applies the selected suggestion
/// 2. Navigation (Alt+Up/Down or Alt+j/k): Moves selection highlight through suggestions
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
pub fn handle_suggestion_selection(
    key: KeyEvent,
    ai_state: &mut AiState,
    input_state: &mut InputState,
    query_state: &mut QueryState,
    autocomplete_state: &mut AutocompleteState,
) -> bool {
    #[cfg(debug_assertions)]
    log::debug!(
        "handle_suggestion_selection: visible={}, suggestions={}, key={:?}",
        ai_state.visible,
        ai_state.suggestions.len(),
        key.code
    );

    if !ai_state.visible || ai_state.suggestions.is_empty() {
        return false;
    }

    let suggestion_count = ai_state.suggestions.len();

    if let Some(index) = keybindings::handle_direct_selection(key, suggestion_count) {
        #[cfg(debug_assertions)]
        log::debug!("Direct selection matched: index={}", index);

        if let Some(suggestion) = ai_state.suggestions.get(index) {
            #[cfg(debug_assertions)]
            log::debug!("Applying suggestion: query={}", suggestion.query);

            apply_suggestion(suggestion, input_state, query_state, autocomplete_state);
            ai_state.selection.clear_selection();
            return true;
        }
    }

    if keybindings::handle_navigation(key, &mut ai_state.selection, suggestion_count) {
        return true;
    }

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
/// Returns true if any state changed (responses received or disconnected).
pub fn poll_response_channel(ai_state: &mut AiState) -> bool {
    if ai_state.response_rx.is_none() {
        return false;
    }

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

    let had_responses = !responses.is_empty();

    for response in responses {
        process_response(ai_state, response);
    }

    if disconnected && ai_state.loading {
        ai_state.set_error("AI worker disconnected unexpectedly".to_string());
    }

    had_responses || disconnected
}

/// Handle AI state after jq execution completes
///
/// This is the single entry point for updating AI state based on execution results.
/// Called AFTER jq execution completes with the result (success OR error).
///
/// # Arguments
/// * `ai_state` - The AI state to update
/// * `query_result` - The result of the query execution (Ok with output or Err with message)
/// * `query` - The current query text
/// * `cursor_pos` - The cursor position in the query
/// * `params` - Additional context parameters (schema, base query result, etc.)
pub fn handle_execution_result(
    ai_state: &mut AiState,
    query_result: &Result<String, String>,
    query: &str,
    cursor_pos: usize,
    params: ContextParams,
) {
    let query_changed = ai_state.is_query_changed(query);

    if !query_changed {
        return;
    }

    ai_state.cancel_in_flight_request();
    ai_state.clear_stale_response();
    ai_state.set_last_query_hash(query);

    match query_result {
        Err(error) => {
            if ai_state.visible {
                let context = QueryContext::new(
                    query.to_string(),
                    cursor_pos,
                    None,
                    Some(error.to_string()),
                    params,
                );
                let prompt = build_prompt(&context);
                ai_state.send_request(prompt);
            }
        }
        Ok(output) => {
            if ai_state.visible {
                let context = QueryContext::new(
                    query.to_string(),
                    cursor_pos,
                    Some(output.clone()),
                    None,
                    params,
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
    query: &str,
    cursor_pos: usize,
    params: ContextParams,
) {
    // Convert to Result<String, String> for the unified handler
    let result: Result<String, String> = match query_result {
        Ok(output) => Ok(output.to_string()),
        Err(e) => Err(e.clone()),
    };

    handle_execution_result(ai_state, &result, query, cursor_pos, params);
}

/// Process a single AI response message
///
/// Filters stale responses by checking request_id against the current
/// AiState request_id. Responses from old requests are ignored.
///
/// Note: `AiResponse::Cancelled` does not require request_id comparison because
/// token-based cancellation ensures the response is always for the request that
/// was actually cancelled. The CancellationToken is tied to a specific request.
fn process_response(ai_state: &mut AiState, response: AiResponse) {
    let current_request_id = ai_state.current_request_id();

    match response {
        AiResponse::Chunk { text, request_id } => {
            if request_id < current_request_id {
                log::debug!(
                    "Ignoring stale chunk from request {} (current: {})",
                    request_id,
                    current_request_id
                );
                return;
            }
            ai_state.append_chunk(&text);
        }
        AiResponse::Complete { request_id } => {
            if request_id < current_request_id {
                log::debug!(
                    "Ignoring stale complete from request {} (current: {})",
                    request_id,
                    current_request_id
                );
                return;
            }
            ai_state.complete_request();
        }
        AiResponse::Error(error_msg) => {
            ai_state.set_error(error_msg);
        }
        // Token-based cancellation: no request_id comparison needed.
        // The CancellationToken is tied to a specific request, so when
        // we receive Cancelled, it's always for the request we cancelled.
        AiResponse::Cancelled { request_id: _ } => {
            log::debug!("Request cancelled via token");
            ai_state.loading = false;
            ai_state.in_flight_request_id = None;
        }
    }
}

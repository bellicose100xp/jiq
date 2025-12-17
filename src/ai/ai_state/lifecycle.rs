//! AI state lifecycle management
//!
//! Handles initialization, state transitions, and clearing operations.

use super::super::selection::SelectionState;
use super::super::suggestion::parse_suggestions;
use crate::ai::ai_state::AiState;

impl AiState {
    /// Create a new AiState
    ///
    /// # Arguments
    /// * `enabled` - Whether AI features are enabled (from config)
    #[allow(dead_code)]
    pub fn new(enabled: bool) -> Self {
        Self {
            visible: false,
            enabled,
            configured: false,
            loading: false,
            error: None,
            response: String::new(),
            previous_response: None,
            request_tx: None,
            response_rx: None,
            request_id: 0,
            last_query_hash: None,
            in_flight_request_id: None,
            current_cancel_token: None,
            suggestions: Vec::new(),
            word_limit: 200,
            selection: SelectionState::new(),
        }
    }

    /// Create a new AiState with configuration status
    ///
    /// # Arguments
    /// * `enabled` - Whether AI features are enabled (from config)
    /// * `configured` - Whether AI is properly configured (has API key)
    pub fn new_with_config(enabled: bool, configured: bool) -> Self {
        Self {
            visible: enabled,
            enabled,
            configured,
            loading: false,
            error: None,
            response: String::new(),
            previous_response: None,
            request_tx: None,
            response_rx: None,
            request_id: 0,
            last_query_hash: None,
            in_flight_request_id: None,
            current_cancel_token: None,
            suggestions: Vec::new(),
            word_limit: 200,
            selection: SelectionState::new(),
        }
    }

    /// Toggle the visibility of the AI popup
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Close the AI popup (Esc key handler)
    #[allow(dead_code)]
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Start a new request, preserving the current response
    ///
    /// Increments the request_id to ensure stale responses from previous
    /// requests are filtered out. Also sets in_flight_request_id to track
    /// the active request for cancellation.
    pub fn start_request(&mut self) {
        if !self.response.is_empty() {
            self.previous_response = Some(self.response.clone());
        }
        self.response.clear();
        self.error = None;
        self.loading = true;
        self.request_id = self.request_id.wrapping_add(1);
        self.in_flight_request_id = Some(self.request_id);
        self.suggestions.clear();
        self.selection.clear_selection();
    }

    /// Mark the request as complete
    ///
    /// Clears loading state, previous response, and in_flight_request_id.
    pub fn complete_request(&mut self) {
        self.loading = false;
        self.previous_response = None;
        self.in_flight_request_id = None;
        self.suggestions = parse_suggestions(&self.response);
    }

    /// Set an error state
    ///
    /// Clears loading state and in_flight_request_id.
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.loading = false;
        self.in_flight_request_id = None;
    }

    /// Clear AI response and error when query becomes successful
    ///
    /// This should be called when the query transitions from error to success
    /// to remove stale error explanations.
    #[allow(dead_code)]
    pub fn clear_on_success(&mut self) {
        self.response.clear();
        self.error = None;
        self.previous_response = None;
        self.loading = false;
    }

    /// Clear stale AI response when query changes
    ///
    /// This should be called when the query changes to remove
    /// advice that was for a different query context.
    pub fn clear_stale_response(&mut self) {
        self.response.clear();
        self.error = None;
        self.previous_response = None;
        self.loading = false;
    }
}

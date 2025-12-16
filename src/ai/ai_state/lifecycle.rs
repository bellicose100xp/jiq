//! AI state lifecycle management
//!
//! Handles initialization, state transitions, and clearing operations.

use super::super::ai_debouncer::AiDebouncer;
use super::super::selection::SelectionState;
use super::super::suggestion::parse_suggestions;
use crate::ai::ai_state::AiState;

impl AiState {
    /// Create a new AiState
    ///
    /// # Arguments
    /// * `enabled` - Whether AI features are enabled (from config)
    /// * `debounce_ms` - Debounce delay in milliseconds
    // TODO: Remove #[allow(dead_code)] when this constructor is used
    #[allow(dead_code)] // Phase 1: Use new_with_config instead
    pub fn new(enabled: bool, debounce_ms: u64) -> Self {
        Self {
            visible: false,
            enabled,
            configured: false, // Will be set to true when API key is provided
            loading: false,
            error: None,
            response: String::new(),
            previous_response: None,
            debouncer: AiDebouncer::new(debounce_ms),
            request_tx: None,
            response_rx: None,
            request_id: 0,
            last_query_hash: None,
            in_flight_request_id: None,
            suggestions: Vec::new(),
            word_limit: 200, // Default word limit, updated during rendering
            selection: SelectionState::new(),
        }
    }

    /// Create a new AiState with configuration status
    ///
    /// # Arguments
    /// * `enabled` - Whether AI features are enabled (from config)
    /// * `configured` - Whether AI is properly configured (has API key)
    /// * `debounce_ms` - Debounce delay in milliseconds
    ///
    /// # Requirements
    /// - 8.1: WHEN AI is enabled in config THEN the AI_Popup SHALL be visible by default
    /// - 8.2: WHEN AI is disabled in config THEN the AI_Popup SHALL be hidden by default
    pub fn new_with_config(enabled: bool, configured: bool, debounce_ms: u64) -> Self {
        Self {
            visible: enabled, // Phase 2: visible by default when AI enabled
            enabled,
            configured,
            loading: false,
            error: None,
            response: String::new(),
            previous_response: None,
            debouncer: AiDebouncer::new(debounce_ms),
            request_tx: None,
            response_rx: None,
            request_id: 0,
            last_query_hash: None,
            in_flight_request_id: None,
            suggestions: Vec::new(),
            word_limit: 200, // Default word limit, updated during rendering
            selection: SelectionState::new(),
        }
    }

    /// Toggle the visibility of the AI popup
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Close the AI popup (Esc key handler)
    // TODO: Remove #[allow(dead_code)] if close() is needed in future
    #[allow(dead_code)] // Phase 1: ESC doesn't close popup, only toggle does
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Start a new request, preserving the current response
    ///
    /// Increments the request_id to ensure stale responses from previous
    /// requests are filtered out. Also sets in_flight_request_id to track
    /// the active request for cancellation.
    ///
    /// # Requirements (Phase 3)
    /// - 1.1-1.5: Selection state is cleared when starting a new request
    pub fn start_request(&mut self) {
        if !self.response.is_empty() {
            self.previous_response = Some(self.response.clone());
        }
        self.response.clear();
        self.error = None;
        self.loading = true;
        self.request_id = self.request_id.wrapping_add(1);
        self.in_flight_request_id = Some(self.request_id);
        self.suggestions.clear(); // Phase 2: Clear suggestions on new request
        self.selection.clear_selection(); // Phase 3: Clear selection on new request
    }

    /// Mark the request as complete
    ///
    /// Clears loading state, previous response, and in_flight_request_id.
    /// Also parses suggestions from the response (Phase 2).
    pub fn complete_request(&mut self) {
        self.loading = false;
        self.previous_response = None;
        self.in_flight_request_id = None;
        // Phase 2: Parse suggestions from response
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
    /// Note: Does not clear last_query_hash - that's managed by handle_query_result
    #[allow(dead_code)] // Used in tests
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

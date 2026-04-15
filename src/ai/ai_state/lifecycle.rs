//! AI state lifecycle management
//!
//! Handles initialization, state transitions, and clearing operations.

use super::super::selection::SelectionState;
use super::super::suggestion::parse_suggestions;
use crate::ai::ai_state::AiState;

/// Default max context length for tests
#[cfg(test)]
pub const TEST_MAX_CONTEXT_LENGTH: usize = 50_000;

impl AiState {
    /// Create a new AiState (test helper)
    ///
    /// # Arguments
    /// * `enabled` - Whether AI features are enabled (from config)
    #[cfg(test)]
    pub fn new(enabled: bool) -> Self {
        Self {
            visible: false,
            enabled,
            configured: false,
            provider_name: "AI".to_string(),
            model_name: String::new(),
            max_context_length: TEST_MAX_CONTEXT_LENGTH,
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
            parse_failed: false,
            selection: SelectionState::new(),
            previous_popup_height: None,
        }
    }

    /// Create a new AiState with configuration status
    ///
    /// # Arguments
    /// * `enabled` - Whether AI features are enabled (from config)
    /// * `configured` - Whether AI is properly configured (has API key)
    /// * `provider_name` - Name of the AI provider (e.g., "Anthropic", "Bedrock", "OpenAI")
    /// * `model_name` - Model name (e.g., "claude-3-5-sonnet-20241022", "gpt-4o-mini")
    /// * `max_context_length` - Maximum character length for JSON context samples
    pub fn new_with_config(
        enabled: bool,
        configured: bool,
        provider_name: String,
        model_name: String,
        max_context_length: usize,
    ) -> Self {
        Self {
            visible: enabled,
            enabled,
            configured,
            provider_name,
            model_name,
            max_context_length,
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
            parse_failed: false,
            selection: SelectionState::new(),
            previous_popup_height: None,
        }
    }

    /// Toggle the visibility of the AI popup
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Close the AI popup (test helper)
    #[cfg(test)]
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
        self.parse_failed = false;
        self.selection.clear_selection();
        self.selection.clear_layout();
    }

    /// Mark the request as complete
    ///
    /// Clears loading state, previous response, and in_flight_request_id.
    pub fn complete_request(&mut self) {
        self.loading = false;
        self.previous_response = None;
        self.in_flight_request_id = None;
        self.suggestions = parse_suggestions(&self.response);
        self.parse_failed = !self.response.is_empty() && self.suggestions.is_empty();
        // Diagnostic: when parsing fails, write the raw response to a debug
        // file so the exact model output can be inspected. Only active when
        // JIQ_DEBUG_AI=1 is set to avoid filesystem writes in normal use.
        if self.parse_failed && std::env::var("JIQ_DEBUG_AI").as_deref() == Ok("1") {
            let path = std::env::temp_dir().join("jiq-ai-failed-response.log");
            let _ = std::fs::write(&path, &self.response);
        }
        self.selection.clear_layout();
    }

    /// Set an error state
    ///
    /// Clears loading state and in_flight_request_id.
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.loading = false;
        self.in_flight_request_id = None;
    }

    /// Clear AI response and error when query becomes successful (test helper)
    ///
    /// Note: Production code uses clear_stale_response() instead.
    #[cfg(test)]
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
        self.parse_failed = false;
    }
}

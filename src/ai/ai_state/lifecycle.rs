//! AI state lifecycle management
//!
//! Handles initialization, state transitions, and clearing operations.

use super::super::selection::SelectionState;
use super::super::suggestion::{ParseOutcome, parse_response};
use super::conversation::new_chat_input;
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
            extra_instructions: None,
            loading: false,
            error: None,
            response: String::new(),
            request_tx: None,
            response_rx: None,
            request_id: 0,
            last_query_hash: None,
            in_flight_request_id: None,
            current_cancel_token: None,
            suggestions: Vec::new(),
            parse_failed: false,
            no_suggestions: false,
            selection: SelectionState::new(),
            previous_popup_height: None,
            history: Vec::new(),
            current_question: None,
            current_query: String::new(),
            answer: None,
            chat_input: new_chat_input(),
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
    ///
    /// `extra_instructions` starts as None; the app sets it from config after
    /// construction (keeps this constructor stable across its many test callers).
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
            extra_instructions: None,
            loading: false,
            error: None,
            response: String::new(),
            request_tx: None,
            response_rx: None,
            request_id: 0,
            last_query_hash: None,
            in_flight_request_id: None,
            current_cancel_token: None,
            suggestions: Vec::new(),
            parse_failed: false,
            no_suggestions: false,
            selection: SelectionState::new(),
            previous_popup_height: None,
            history: Vec::new(),
            current_question: None,
            current_query: String::new(),
            answer: None,
            chat_input: new_chat_input(),
        }
    }

    /// Toggle the visibility of the AI popup (test helper; the app uses
    /// `App::toggle_ai_popup` so focus follows the popup)
    #[cfg(test)]
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Close the AI popup (test helper)
    #[cfg(test)]
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Start a new request
    ///
    /// A completed chat exchange on screen is archived into `history` first
    /// so it stays visible (dimmed) and is replayed on later requests.
    /// Increments the request_id to ensure stale responses from previous
    /// requests are filtered out. Also sets in_flight_request_id to track
    /// the active request for cancellation.
    pub fn start_request(&mut self) {
        self.archive_current_exchange();
        self.response.clear();
        self.error = None;
        self.answer = None;
        self.loading = true;
        self.request_id = self.request_id.wrapping_add(1);
        self.in_flight_request_id = Some(self.request_id);
        self.suggestions.clear();
        self.parse_failed = false;
        self.no_suggestions = false;
        self.selection.clear_selection();
        self.selection.clear_layout();
        self.selection.request_scroll_to_bottom();
    }

    /// Mark the request as complete
    ///
    /// Clears loading state and in_flight_request_id, then
    /// classifies the accumulated response into one of three outcomes:
    /// - parsed content -> populate `answer` and `suggestions`;
    /// - valid empty list -> set `no_suggestions` (model had nothing to say);
    /// - unparseable -> set `parse_failed` and log the raw response.
    ///
    /// A genuinely-empty response (no bytes received at all) is treated as
    /// neither: it leaves all three cleared so the UI stays blank.
    pub fn complete_request(&mut self) {
        self.loading = false;
        self.in_flight_request_id = None;

        self.suggestions = Vec::new();
        self.answer = None;
        self.parse_failed = false;
        self.no_suggestions = false;

        if !self.response.is_empty() {
            match parse_response(&self.response) {
                ParseOutcome::Parsed(parsed) => {
                    self.answer = parsed.answer;
                    self.suggestions = parsed.suggestions;
                }
                ParseOutcome::Empty => self.no_suggestions = true,
                ParseOutcome::Unparseable => {
                    self.parse_failed = true;
                    log::warn!(
                        "AI response failed to parse (len={}):\n{}",
                        self.response.len(),
                        self.response
                    );
                }
            }
        } else {
            // No bytes received before completion: all classification flags stay
            // clear, so the popup renders empty until the next query.
            log::debug!("complete_request: empty response, popup will be blank until next query");
        }

        self.selection.clear_layout();
        self.selection.request_scroll_to_bottom();
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
        self.loading = false;
    }

    /// Clear stale AI response when query changes
    ///
    /// This should be called when the query changes to remove
    /// advice that was for a different query context. A completed chat
    /// exchange is archived rather than dropped so the conversation survives
    /// query edits.
    pub fn clear_stale_response(&mut self) {
        self.archive_current_exchange();
        self.response.clear();
        self.error = None;
        self.answer = None;
        self.loading = false;
        self.parse_failed = false;
        self.no_suggestions = false;
    }
}

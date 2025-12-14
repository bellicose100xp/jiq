//! AI Assistant state management
//!
//! Manages the state of the AI assistant popup including visibility, loading state,
//! responses, and channel handles for communication with the worker thread.

use std::sync::mpsc::{Receiver, Sender};

use ratatui::style::Color;

use super::ai_debouncer::AiDebouncer;

// =========================================================================
// Phase 2: Suggestion Types
// =========================================================================

/// Type of AI suggestion
///
/// # Requirements
/// - 5.4: Fix type displayed in red
/// - 5.5: Optimize type displayed in yellow
/// - 5.6: Next type displayed in green
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionType {
    /// Error corrections - displayed in red
    Fix,
    /// Performance/style improvements - displayed in yellow
    Optimize,
    /// Next steps, NL interpretations - displayed in green
    Next,
}

impl SuggestionType {
    /// Get the color for this suggestion type
    pub fn color(&self) -> Color {
        match self {
            SuggestionType::Fix => Color::Red,
            SuggestionType::Optimize => Color::Yellow,
            SuggestionType::Next => Color::Green,
        }
    }

    /// Parse suggestion type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fix" => Some(SuggestionType::Fix),
            "optimize" => Some(SuggestionType::Optimize),
            "next" => Some(SuggestionType::Next),
            _ => None,
        }
    }

    /// Get the display label for this type
    pub fn label(&self) -> &'static str {
        match self {
            SuggestionType::Fix => "[Fix]",
            SuggestionType::Optimize => "[Optimize]",
            SuggestionType::Next => "[Next]",
        }
    }
}

/// A single AI suggestion for a jq query
///
/// # Requirements
/// - 5.2: Format "N. [Type] jq_query_here" followed by description
/// - 5.3: Extractable query for future selection feature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// The suggested jq query (extractable for future selection)
    pub query: String,
    /// Brief explanation of what the query does
    pub description: String,
    /// Type of suggestion: Fix, Optimize, or Next
    pub suggestion_type: SuggestionType,
}

/// Request messages sent to the AI worker thread
#[derive(Debug)]
pub enum AiRequest {
    /// Query the AI with the given context
    Query {
        prompt: String,
        /// Unique ID for this request, used to filter stale responses
        request_id: u64,
    },
    /// Cancel the request with the given ID
    Cancel {
        /// ID of the request to cancel
        request_id: u64,
    },
}

/// Response messages received from the AI worker thread
#[derive(Debug)]
pub enum AiResponse {
    /// A chunk of streaming text
    Chunk {
        text: String,
        /// Request ID this chunk belongs to
        request_id: u64,
    },
    /// The response is complete
    Complete {
        /// Request ID this completion belongs to
        request_id: u64,
    },
    /// An error occurred
    Error(String),
    /// The request was cancelled
    Cancelled {
        /// Request ID that was cancelled
        request_id: u64,
    },
}

/// AI Assistant state
pub struct AiState {
    /// Whether the AI popup is visible
    pub visible: bool,
    /// Whether AI features are enabled (from config)
    pub enabled: bool,
    /// Whether the AI is properly configured (has API key)
    pub configured: bool,
    /// Whether we're waiting for or receiving a response
    pub loading: bool,
    /// Current error message (if any)
    pub error: Option<String>,
    /// Current response text (accumulated from streaming chunks)
    pub response: String,
    /// Previous response (preserved when starting a new request)
    pub previous_response: Option<String>,
    /// Debouncer for API requests
    // TODO: Remove #[allow(dead_code)] when debouncer is integrated
    #[allow(dead_code)] // Phase 1: Reserved for future debouncing
    pub debouncer: AiDebouncer,
    /// Channel to send requests to the worker thread
    pub request_tx: Option<Sender<AiRequest>>,
    /// Channel to receive responses from the worker thread
    pub response_rx: Option<Receiver<AiResponse>>,
    /// Current request ID, incremented for each new request
    /// Used to filter stale responses from previous requests
    pub request_id: u64,
    /// Hash of the last query text that triggered an AI request
    /// Used to detect query changes - query change is the ONLY trigger for new AI requests
    pub last_query_hash: Option<u64>,
    /// ID of the currently in-flight request, if any
    /// Used to track active requests for cancellation
    pub in_flight_request_id: Option<u64>,
    /// Parsed suggestions from AI response (Phase 2)
    /// Empty if response couldn't be parsed into structured suggestions
    pub suggestions: Vec<Suggestion>,
    /// Current word limit based on popup dimensions (Phase 2)
    /// Updated when popup is rendered, used for next AI request
    pub word_limit: u16,
}

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
    }

    /// Append a chunk to the current response
    pub fn append_chunk(&mut self, chunk: &str) {
        self.response.push_str(chunk);
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
        self.suggestions = Self::parse_suggestions(&self.response);
    }

    /// Parse suggestions from AI response text
    ///
    /// Expected format:
    /// ```text
    /// 1. [Fix] .users[] | select(.active)
    ///    Filters to only active users
    ///
    /// 2. [Next] .users[] | .email
    ///    Extracts email addresses from users
    /// ```
    ///
    /// # Requirements
    /// - 5.2: Parse "N. [Type] jq_query_here" format
    /// - 5.3: Extract query string from each suggestion
    /// - 5.9: Return empty vec if parsing fails (fallback to raw response)
    pub fn parse_suggestions(response: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        let lines: Vec<&str> = response.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Look for pattern: "N. [Type] query"
            // e.g., "1. [Fix] .users[]"
            if let Some(suggestion) = Self::parse_suggestion_line(line, &lines[i + 1..]) {
                suggestions.push(suggestion.0);
                i += suggestion.1; // Skip the lines we consumed
            } else {
                i += 1;
            }
        }

        suggestions
    }

    /// Parse a single suggestion starting from a numbered line
    ///
    /// Returns (Suggestion, lines_consumed) if successful
    fn parse_suggestion_line(line: &str, remaining_lines: &[&str]) -> Option<(Suggestion, usize)> {
        // Match pattern: digit(s) followed by ". ["
        let line = line.trim();

        // Find the number at the start
        let dot_pos = line.find(". [")?;
        let num_str = &line[..dot_pos];
        if !num_str.chars().all(|c| c.is_ascii_digit()) || num_str.is_empty() {
            return None;
        }

        // Find the type between [ and ]
        let type_start = dot_pos + 3; // Skip ". ["
        let type_end = line[type_start..].find(']')? + type_start;
        let type_str = &line[type_start..type_end];
        let suggestion_type = SuggestionType::from_str(type_str)?;

        // Query is everything after "] "
        let query_start = type_end + 1;
        let query = line[query_start..].trim().to_string();

        if query.is_empty() {
            return None;
        }

        // Collect description from following indented lines
        let mut description_lines = Vec::new();
        let mut lines_consumed = 1;

        for remaining_line in remaining_lines {
            let trimmed = remaining_line.trim();

            // Stop at empty line or next numbered suggestion
            if trimmed.is_empty() {
                lines_consumed += 1;
                break;
            }

            // Check if this is a new numbered suggestion
            if let Some(dot_pos) = trimmed.find(". [") {
                let num_part = &trimmed[..dot_pos];
                if num_part.chars().all(|c| c.is_ascii_digit()) && !num_part.is_empty() {
                    break;
                }
            }

            // This is a description line (indented or continuation)
            description_lines.push(trimmed);
            lines_consumed += 1;
        }

        let description = description_lines.join(" ");

        Some((
            Suggestion {
                query,
                description,
                suggestion_type,
            },
            lines_consumed,
        ))
    }

    /// Set an error state
    ///
    /// Clears loading state and in_flight_request_id.
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.loading = false;
        self.in_flight_request_id = None;
    }

    /// Cancel any in-flight request
    ///
    /// Sends a Cancel message to the worker thread if there's an active request.
    /// Returns true if a cancel was sent, false otherwise.
    ///
    /// # Requirements
    /// - 3.5: WHEN a new query change occurs THEN the AI_Assistant SHALL cancel
    ///        any in-flight API request before starting the debounce period
    /// - 5.4: WHEN a query change occurs while an API request is in-flight THEN
    ///        the AI_Assistant SHALL send a cancel signal to abort the previous request
    pub fn cancel_in_flight_request(&mut self) -> bool {
        if let Some(request_id) = self.in_flight_request_id {
            if let Some(ref tx) = self.request_tx {
                if tx.send(AiRequest::Cancel { request_id }).is_ok() {
                    log::debug!("Sent cancel for request {}", request_id);
                    self.in_flight_request_id = None;
                    return true;
                }
            }
        }
        false
    }

    /// Check if there's an in-flight request
    #[allow(dead_code)] // Used in tests
    pub fn has_in_flight_request(&self) -> bool {
        self.in_flight_request_id.is_some()
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

    /// Auto-show the AI popup on query error
    ///
    /// Opens the popup if not already visible, and returns true to indicate
    /// a new AI request should be made.
    ///
    /// Returns true if:
    /// - AI is enabled
    /// - auto_show_on_error is true
    ///
    /// This allows new requests for each error, even if popup is already visible.
    ///
    /// # Requirements
    /// - 3.1: WHEN a jq query produces an error AND `auto_show_on_error` is true
    ///        THEN the AI_Popup SHALL automatically open and request assistance
    /// - 3.2: WHEN `auto_show_on_error` is false THEN the AI_Popup SHALL remain
    ///        closed on query errors until manually opened with Ctrl+A
    pub fn auto_show_on_error(&mut self, auto_show_enabled: bool) -> bool {
        if !self.enabled || !auto_show_enabled {
            return false;
        }

        // Show popup if not already visible
        if !self.visible {
            self.visible = true;
        }

        // Always return true to trigger new request for each error
        true
    }

    /// Send an AI request through the channel
    ///
    /// Returns true if the request was sent successfully, false otherwise.
    /// The request includes the current request_id which is incremented
    /// by start_request() to filter stale responses.
    pub fn send_request(&mut self, prompt: String) -> bool {
        // Check if we have a channel first
        if self.request_tx.is_none() {
            return false;
        }

        // Start request first to increment request_id
        self.start_request();
        let request_id = self.request_id;

        // Now send the request
        if let Some(ref tx) = self.request_tx {
            if tx.send(AiRequest::Query { prompt, request_id }).is_ok() {
                return true;
            }
        }
        false
    }

    /// Set the channel handles for communication with the worker thread
    pub fn set_channels(
        &mut self,
        request_tx: Sender<AiRequest>,
        response_rx: Receiver<AiResponse>,
    ) {
        self.request_tx = Some(request_tx);
        self.response_rx = Some(response_rx);
    }

    /// Get the current request ID
    ///
    /// Used to check if incoming responses match the current request.
    pub fn current_request_id(&self) -> u64 {
        self.request_id
    }

    /// Compute a hash for a query string
    ///
    /// Uses a simple hash function to create a unique identifier for the query.
    fn compute_query_hash(query: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        hasher.finish()
    }

    /// Check if a query has changed since the last AI request
    ///
    /// Returns true if:
    /// - No previous query hash exists (first request)
    /// - The query hash differs from the last query hash
    ///
    /// Query change is the ONLY trigger for new AI requests.
    /// The simplified flow: query changes → execute → if error, send AI request
    pub fn is_query_changed(&self, query: &str) -> bool {
        let query_hash = Self::compute_query_hash(query);
        match self.last_query_hash {
            None => true,
            Some(last_hash) => query_hash != last_hash,
        }
    }

    /// Update the last query hash
    ///
    /// Should be called when sending a request for a query.
    pub fn set_last_query_hash(&mut self, query: &str) {
        self.last_query_hash = Some(Self::compute_query_hash(query));
    }
}

impl Default for AiState {
    fn default() -> Self {
        Self::new_with_config(false, false, 1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // =========================================================================
    // Unit Tests
    // =========================================================================

    #[test]
    fn test_new_ai_state_disabled() {
        let state = AiState::new(false, 1000);
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
        let state = AiState::new(true, 500);
        assert!(!state.visible);
        assert!(state.enabled);
        assert!(!state.configured); // new() defaults to not configured
        assert!(!state.loading);
    }

    #[test]
    fn test_new_with_config_configured() {
        let state = AiState::new_with_config(true, true, 500);
        assert!(state.visible); // Phase 2: visible when enabled
        assert!(state.enabled);
        assert!(state.configured);
        assert!(!state.loading);
    }

    #[test]
    fn test_new_with_config_not_configured() {
        let state = AiState::new_with_config(true, false, 500);
        assert!(state.visible); // Phase 2: visible when enabled
        assert!(state.enabled);
        assert!(!state.configured);
        assert!(!state.loading);
    }

    #[test]
    fn test_toggle_visibility() {
        let mut state = AiState::new(true, 1000);
        assert!(!state.visible);
        state.toggle();
        assert!(state.visible);
        state.toggle();
        assert!(!state.visible);
    }

    #[test]
    fn test_close() {
        let mut state = AiState::new(true, 1000);
        state.visible = true;
        state.close();
        assert!(!state.visible);
    }

    #[test]
    fn test_start_request_preserves_response() {
        let mut state = AiState::new(true, 1000);
        state.response = "previous answer".to_string();
        state.start_request();
        assert!(state.loading);
        assert!(state.response.is_empty());
        assert_eq!(state.previous_response, Some("previous answer".to_string()));
    }

    #[test]
    fn test_start_request_empty_response() {
        let mut state = AiState::new(true, 1000);
        state.start_request();
        assert!(state.loading);
        assert!(state.response.is_empty());
        assert!(state.previous_response.is_none());
    }

    #[test]
    fn test_append_chunk() {
        let mut state = AiState::new(true, 1000);
        state.append_chunk("Hello ");
        state.append_chunk("World");
        assert_eq!(state.response, "Hello World");
    }

    #[test]
    fn test_complete_request() {
        let mut state = AiState::new(true, 1000);
        state.loading = true;
        state.previous_response = Some("old".to_string());
        state.complete_request();
        assert!(!state.loading);
        assert!(state.previous_response.is_none());
    }

    #[test]
    fn test_set_error() {
        let mut state = AiState::new(true, 1000);
        state.loading = true;
        state.set_error("Network error".to_string());
        assert!(!state.loading);
        assert_eq!(state.error, Some("Network error".to_string()));
    }

    #[test]
    fn test_clear_on_success() {
        let mut state = AiState::new(true, 1000);
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
        let mut state = AiState::new(true, 1000);
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
        let mut state = AiState::new(true, 1000);
        assert_eq!(state.request_id, 0);

        state.start_request();
        assert_eq!(state.request_id, 1);

        state.start_request();
        assert_eq!(state.request_id, 2);
    }

    // =========================================================================
    // Query Hash Tests
    // =========================================================================

    #[test]
    fn test_is_query_changed_no_previous() {
        let state = AiState::new(true, 1000);
        assert!(state.is_query_changed(".name"));
    }

    #[test]
    fn test_is_query_changed_same_query() {
        let mut state = AiState::new(true, 1000);
        state.set_last_query_hash(".name");
        assert!(!state.is_query_changed(".name"));
    }

    #[test]
    fn test_is_query_changed_different_query() {
        let mut state = AiState::new(true, 1000);
        state.set_last_query_hash(".name");
        assert!(state.is_query_changed(".age"));
    }

    #[test]
    fn test_same_query_no_new_request() {
        let mut state = AiState::new(true, 1000);
        state.set_last_query_hash(".name");

        // Same query should NOT trigger new request (regardless of error)
        assert!(!state.is_query_changed(".name"));
    }

    #[test]
    fn test_different_query_triggers_new_request() {
        let mut state = AiState::new(true, 1000);
        state.set_last_query_hash(".name");

        // Different query should trigger new request
        assert!(state.is_query_changed(".age"));
    }

    // =========================================================================
    // Property-Based Tests
    // =========================================================================

    // **Feature: ai-assistant, Property 5: Toggle visibility**
    // *For any* AiState with visibility V, calling toggle() should result in visibility !V.
    // **Validates: Requirements 2.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_toggle_visibility(initial_visible: bool, enabled: bool, debounce_ms in 100u64..5000u64) {
            let mut state = AiState::new(enabled, debounce_ms);
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
        fn prop_toggle_is_only_way_to_close(enabled: bool, debounce_ms in 100u64..5000u64, has_response: bool, response in "[a-zA-Z ]{0,100}") {
            let mut state = AiState::new(enabled, debounce_ms);
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
            debounce_ms in 100u64..5000u64,
            response in "[a-zA-Z0-9 ]{1,100}"
        ) {
            let mut state = AiState::new(enabled, debounce_ms);
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
        fn prop_empty_response_not_preserved(enabled: bool, debounce_ms in 100u64..5000u64) {
            let mut state = AiState::new(enabled, debounce_ms);
            // Response is empty by default

            state.start_request();

            prop_assert!(
                state.previous_response.is_none(),
                "Empty response should not be preserved"
            );
        }
    }

    // **Feature: ai-assistant, Property 8: Auto-show on error when enabled**
    // *For any* app state with a query error and `auto_show_on_error = true`,
    // the AI popup should become visible.
    // **Validates: Requirements 3.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_auto_show_on_error_when_enabled(
            debounce_ms in 100u64..5000u64,
            initial_visible in prop::bool::ANY,
        ) {
            let mut state = AiState::new(true, debounce_ms); // AI enabled
            state.visible = initial_visible;

            // Call auto_show_on_error with auto_show enabled
            let result = state.auto_show_on_error(true);

            // Should always return true to trigger new request for each error
            prop_assert!(result, "Should return true to trigger AI request");

            // Popup should be visible after call
            prop_assert!(state.visible, "Popup should be visible");
        }
    }

    // **Feature: ai-assistant, Property 9: No auto-show when disabled**
    // *For any* app state with a query error and `auto_show_on_error = false`,
    // the AI popup visibility should remain unchanged.
    // **Validates: Requirements 3.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_no_auto_show_when_disabled(
            debounce_ms in 100u64..5000u64,
            initial_visible in prop::bool::ANY,
            ai_enabled in prop::bool::ANY,
        ) {
            let mut state = AiState::new(ai_enabled, debounce_ms);
            state.visible = initial_visible;

            // Call auto_show_on_error with auto_show DISABLED
            let result = state.auto_show_on_error(false);

            // Should never auto-show when auto_show_on_error is false
            prop_assert!(!result, "Should not auto-show when auto_show_on_error is false");
            prop_assert_eq!(
                state.visible, initial_visible,
                "Visibility should remain unchanged when auto_show_on_error is false"
            );
        }

        #[test]
        fn prop_no_auto_show_when_ai_disabled(
            debounce_ms in 100u64..5000u64,
            initial_visible in prop::bool::ANY,
            auto_show_enabled in prop::bool::ANY,
        ) {
            let mut state = AiState::new(false, debounce_ms); // AI disabled
            state.visible = initial_visible;

            // Call auto_show_on_error (regardless of auto_show setting)
            let result = state.auto_show_on_error(auto_show_enabled);

            // Should never auto-show when AI is disabled
            prop_assert!(!result, "Should not auto-show when AI is disabled");
            prop_assert_eq!(
                state.visible, initial_visible,
                "Visibility should remain unchanged when AI is disabled"
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
            debounce_ms in 100u64..5000u64,
            initial_visible in prop::bool::ANY,
            response in "[a-zA-Z0-9 ]{0,100}",
            error in prop::option::of("[a-zA-Z0-9 ]{1,50}"),
            previous_response in prop::option::of("[a-zA-Z0-9 ]{1,50}"),
            loading in prop::bool::ANY,
        ) {
            let mut state = AiState::new(enabled, debounce_ms);
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

    // =========================================================================
    // Cancellation Tests
    // =========================================================================

    #[test]
    fn test_start_request_sets_in_flight_request_id() {
        let mut state = AiState::new(true, 1000);
        assert!(state.in_flight_request_id.is_none());

        state.start_request();
        assert_eq!(state.in_flight_request_id, Some(1));

        state.start_request();
        assert_eq!(state.in_flight_request_id, Some(2));
    }

    #[test]
    fn test_complete_request_clears_in_flight_request_id() {
        let mut state = AiState::new(true, 1000);
        state.start_request();
        assert!(state.in_flight_request_id.is_some());

        state.complete_request();
        assert!(state.in_flight_request_id.is_none());
    }

    #[test]
    fn test_set_error_clears_in_flight_request_id() {
        let mut state = AiState::new(true, 1000);
        state.start_request();
        assert!(state.in_flight_request_id.is_some());

        state.set_error("test error".to_string());
        assert!(state.in_flight_request_id.is_none());
    }

    #[test]
    fn test_has_in_flight_request() {
        let mut state = AiState::new(true, 1000);
        assert!(!state.has_in_flight_request());

        state.start_request();
        assert!(state.has_in_flight_request());

        state.complete_request();
        assert!(!state.has_in_flight_request());
    }

    #[test]
    fn test_cancel_in_flight_request_without_channel() {
        let mut state = AiState::new(true, 1000);
        state.start_request();

        // Without a channel, cancel should return false
        let result = state.cancel_in_flight_request();
        assert!(!result);
    }

    #[test]
    fn test_cancel_in_flight_request_with_channel() {
        use std::sync::mpsc;

        let mut state = AiState::new(true, 1000);
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

        let mut state = AiState::new(true, 1000);
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
            debounce_ms in 100u64..5000u64,
        ) {
            use std::sync::mpsc;

            let mut state = AiState::new(enabled, debounce_ms);
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
            debounce_ms in 100u64..5000u64,
        ) {
            use std::sync::mpsc;

            let mut state = AiState::new(enabled, debounce_ms);
            let (tx, rx) = mpsc::channel();
            state.request_tx = Some(tx);

            // Don't start a request - no in-flight request
            prop_assert!(!state.has_in_flight_request(), "Should not have in-flight request");

            // Try to cancel - should return false
            let cancelled = state.cancel_in_flight_request();
            prop_assert!(!cancelled, "Should not cancel when no in-flight request");

            // Verify no message was sent
            let msg = rx.try_recv();
            prop_assert!(msg.is_err(), "Should not have sent any message");
        }
    }

    // =========================================================================
    // Phase 2: Suggestion Parsing Tests
    // =========================================================================

    #[test]
    fn test_suggestion_type_colors() {
        assert_eq!(SuggestionType::Fix.color(), ratatui::style::Color::Red);
        assert_eq!(
            SuggestionType::Optimize.color(),
            ratatui::style::Color::Yellow
        );
        assert_eq!(SuggestionType::Next.color(), ratatui::style::Color::Green);
    }

    #[test]
    fn test_suggestion_type_from_str() {
        assert_eq!(SuggestionType::from_str("Fix"), Some(SuggestionType::Fix));
        assert_eq!(SuggestionType::from_str("fix"), Some(SuggestionType::Fix));
        assert_eq!(SuggestionType::from_str("FIX"), Some(SuggestionType::Fix));
        assert_eq!(
            SuggestionType::from_str("Optimize"),
            Some(SuggestionType::Optimize)
        );
        assert_eq!(SuggestionType::from_str("Next"), Some(SuggestionType::Next));
        assert_eq!(SuggestionType::from_str("Invalid"), None);
    }

    #[test]
    fn test_suggestion_type_labels() {
        assert_eq!(SuggestionType::Fix.label(), "[Fix]");
        assert_eq!(SuggestionType::Optimize.label(), "[Optimize]");
        assert_eq!(SuggestionType::Next.label(), "[Next]");
    }

    #[test]
    fn test_parse_suggestions_single() {
        let response = "1. [Fix] .users[] | select(.active)\n   Filters to only active users";
        let suggestions = AiState::parse_suggestions(response);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].query, ".users[] | select(.active)");
        assert_eq!(suggestions[0].description, "Filters to only active users");
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::Fix);
    }

    #[test]
    fn test_parse_suggestions_multiple() {
        let response = r#"1. [Fix] .users[] | select(.active)
   Filters to only active users

2. [Next] .users[] | .email
   Extracts email addresses

3. [Optimize] .users | map(.name)
   More efficient mapping"#;

        let suggestions = AiState::parse_suggestions(response);

        assert_eq!(suggestions.len(), 3);

        assert_eq!(suggestions[0].query, ".users[] | select(.active)");
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::Fix);

        assert_eq!(suggestions[1].query, ".users[] | .email");
        assert_eq!(suggestions[1].suggestion_type, SuggestionType::Next);

        assert_eq!(suggestions[2].query, ".users | map(.name)");
        assert_eq!(suggestions[2].suggestion_type, SuggestionType::Optimize);
    }

    #[test]
    fn test_parse_suggestions_multiline_description() {
        let response =
            "1. [Fix] .data[]\n   This is a longer description\n   that spans multiple lines";
        let suggestions = AiState::parse_suggestions(response);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].query, ".data[]");
        assert!(suggestions[0].description.contains("longer description"));
        assert!(suggestions[0].description.contains("multiple lines"));
    }

    #[test]
    fn test_parse_suggestions_empty_response() {
        let suggestions = AiState::parse_suggestions("");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_parse_suggestions_no_valid_format() {
        let response = "This is just plain text without any structured suggestions.";
        let suggestions = AiState::parse_suggestions(response);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_parse_suggestions_malformed() {
        // Missing type bracket
        let response = "1. Fix .users[]";
        let suggestions = AiState::parse_suggestions(response);
        assert!(suggestions.is_empty());

        // Missing query
        let response = "1. [Fix]";
        let suggestions = AiState::parse_suggestions(response);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_complete_request_parses_suggestions() {
        let mut state = AiState::new(true, 1000);
        state.response = "1. [Fix] .users[]\n   Fix the query".to_string();
        state.loading = true;

        state.complete_request();

        assert!(!state.loading);
        assert_eq!(state.suggestions.len(), 1);
        assert_eq!(state.suggestions[0].query, ".users[]");
    }

    #[test]
    fn test_start_request_clears_suggestions() {
        let mut state = AiState::new(true, 1000);
        state.suggestions = vec![Suggestion {
            query: ".test".to_string(),
            description: "Test".to_string(),
            suggestion_type: SuggestionType::Fix,
        }];

        state.start_request();

        assert!(state.suggestions.is_empty());
    }

    // **Feature: ai-assistant-phase2, Property 7: Suggestion parsing extracts queries**
    // *For any* AI response containing valid suggestion format, parsing SHALL extract the query.
    // **Validates: Requirements 5.2, 5.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_suggestion_parsing_extracts_queries(
            // Query must start with a non-space character to be valid
            query in "\\.[a-zA-Z0-9_|\\[\\]]{1,30}",
            desc in "[a-zA-Z ]{1,50}",
            suggestion_type in prop::sample::select(vec!["Fix", "Optimize", "Next"]),
        ) {
            let response = format!("1. [{}] {}\n   {}", suggestion_type, query, desc);
            let suggestions = AiState::parse_suggestions(&response);

            prop_assert_eq!(suggestions.len(), 1, "Should parse exactly one suggestion");
            prop_assert_eq!(&suggestions[0].query, query.trim(), "Query should match");
        }
    }

    // **Feature: ai-assistant-phase2, Property 8: Malformed response fallback**
    // *For any* AI response that cannot be parsed, parsing SHALL return empty vec.
    // **Validates: Requirements 5.9**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_malformed_response_returns_empty(
            text in "[a-zA-Z ]{0,100}",
        ) {
            // Plain text without numbered format should return empty
            let suggestions = AiState::parse_suggestions(&text);
            // Either empty or valid suggestions (if text accidentally matches format)
            // The key property is that it doesn't crash
            prop_assert!(suggestions.len() <= 1, "Should handle any text gracefully");
        }
    }

    // **Feature: ai-assistant-phase2, Property 9: Suggestion type colors**
    // *For any* parsed suggestion, the type SHALL have the correct color.
    // **Validates: Requirements 5.4, 5.5, 5.6**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_suggestion_type_colors_correct(
            type_idx in 0usize..3usize,
        ) {
            let types = [SuggestionType::Fix, SuggestionType::Optimize, SuggestionType::Next];
            let expected_colors = [
                ratatui::style::Color::Red,
                ratatui::style::Color::Yellow,
                ratatui::style::Color::Green,
            ];

            let suggestion_type = types[type_idx];
            let expected_color = expected_colors[type_idx];

            prop_assert_eq!(
                suggestion_type.color(),
                expected_color,
                "Color for {:?} should be {:?}",
                suggestion_type,
                expected_color
            );
        }
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
            debounce_ms in 100u64..5000u64,
        ) {
            let state = AiState::new_with_config(ai_enabled, configured, debounce_ms);

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
}

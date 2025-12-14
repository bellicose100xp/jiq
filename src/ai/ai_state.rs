//! AI Assistant state management
//!
//! Manages the state of the AI assistant popup including visibility, loading state,
//! responses, and channel handles for communication with the worker thread.

use std::sync::mpsc::{Receiver, Sender};

use super::ai_debouncer::AiDebouncer;

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
        }
    }

    /// Create a new AiState with configuration status
    ///
    /// # Arguments
    /// * `enabled` - Whether AI features are enabled (from config)
    /// * `configured` - Whether AI is properly configured (has API key)
    /// * `debounce_ms` - Debounce delay in milliseconds
    pub fn new_with_config(enabled: bool, configured: bool, debounce_ms: u64) -> Self {
        Self {
            visible: false,
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
    }

    /// Append a chunk to the current response
    pub fn append_chunk(&mut self, chunk: &str) {
        self.response.push_str(chunk);
    }

    /// Mark the request as complete
    ///
    /// Clears loading state, previous response, and in_flight_request_id.
    pub fn complete_request(&mut self) {
        self.loading = false;
        self.previous_response = None;
        self.in_flight_request_id = None;
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
        assert!(!state.visible);
        assert!(state.enabled);
        assert!(state.configured);
        assert!(!state.loading);
    }

    #[test]
    fn test_new_with_config_not_configured() {
        let state = AiState::new_with_config(true, false, 500);
        assert!(!state.visible);
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
}

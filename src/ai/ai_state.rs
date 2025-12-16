//! AI Assistant state management
//!
//! Manages the state of the AI assistant popup including visibility, loading state,
//! responses, and channel handles for communication with the worker thread.

use std::sync::mpsc::{Receiver, Sender};

use super::selection::SelectionState;

// Re-export for backward compatibility
#[allow(unused_imports)]
pub use super::suggestion::{Suggestion, SuggestionType};

// Module declarations
#[path = "ai_state/lifecycle.rs"]
mod lifecycle;
#[path = "ai_state/response.rs"]
mod response;
#[path = "ai_state/suggestions.rs"]
mod suggestions;

// Test module
#[cfg(test)]
#[path = "ai_state_tests.rs"]
mod ai_state_tests;

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
    /// Parsed suggestions from AI response
    /// Empty if response couldn't be parsed into structured suggestions
    pub suggestions: Vec<Suggestion>,
    /// Current word limit based on popup dimensions
    /// Updated when popup is rendered, used for next AI request
    pub word_limit: u16,
    /// Selection state for suggestion navigation
    /// Tracks which suggestion is selected and navigation mode
    pub selection: SelectionState,
}

impl Default for AiState {
    fn default() -> Self {
        Self::new_with_config(false, false)
    }
}

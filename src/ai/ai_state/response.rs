//! AI response handling
//!
//! Handles response processing, query hash management, and request cancellation.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{Receiver, Sender};

use tokio_util::sync::CancellationToken;

use crate::ai::ai_state::{AiRequest, AiResponse, AiState};

impl AiState {
    /// Append a chunk to the current response
    pub fn append_chunk(&mut self, chunk: &str) {
        self.response.push_str(chunk);
    }

    /// Send an AI request through the channel
    ///
    /// Returns true if the request was sent successfully, false otherwise.
    /// The request includes the current request_id which is incremented
    /// by start_request() to filter stale responses, and a CancellationToken
    /// for aborting the request.
    ///
    /// This method:
    /// 1. Cancels any existing in-flight request first
    /// 2. Creates a new CancellationToken
    /// 3. Stores the token in current_cancel_token
    /// 4. Sends the request with the token
    pub fn send_request(&mut self, prompt: String) -> bool {
        // Check if we have a channel first
        if self.request_tx.is_none() {
            return false;
        }

        // Cancel any existing in-flight request first
        self.cancel_in_flight_request();

        // Start request to increment request_id and set up state
        self.start_request();
        let request_id = self.request_id;

        // Create a new cancellation token for this request
        let cancel_token = CancellationToken::new();
        // Store the token so we can cancel it later
        self.current_cancel_token = Some(cancel_token.clone());

        // Now send the request
        if let Some(ref tx) = self.request_tx
            && tx
                .send(AiRequest::Query {
                    prompt,
                    request_id,
                    cancel_token,
                })
                .is_ok()
        {
            return true;
        }
        // If send failed, clear the token
        self.current_cancel_token = None;
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
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        hasher.finish()
    }

    /// Check if a query has changed since the last AI request
    ///
    /// Returns true if:
    /// - No previous query hash exists (first request)
    /// - The query hash differs from the last query hash
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

    /// Cancel any in-flight request
    ///
    /// Calls cancel() on the CancellationToken to immediately abort the HTTP request,
    /// then clears both the token and in-flight request tracking.
    /// Returns true if there was an in-flight request to cancel, false otherwise.
    pub fn cancel_in_flight_request(&mut self) -> bool {
        if let Some(token) = self.current_cancel_token.take() {
            log::debug!(
                "Cancelling in-flight request {:?}",
                self.in_flight_request_id
            );
            token.cancel();
            self.in_flight_request_id = None;
            return true;
        }
        false
    }

    /// Check if there's an in-flight request
    #[allow(dead_code)]
    pub fn has_in_flight_request(&self) -> bool {
        self.in_flight_request_id.is_some()
    }
}

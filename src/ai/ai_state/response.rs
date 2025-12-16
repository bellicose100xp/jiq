//! AI response handling
//!
//! Handles response processing, query hash management, and request cancellation.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{Receiver, Sender};

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
        if let Some(ref tx) = self.request_tx
            && tx.send(AiRequest::Query { prompt, request_id }).is_ok()
        {
            return true;
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
    /// Sends a Cancel message to the worker thread if there's an active request.
    /// Returns true if a cancel was sent, false otherwise.
    pub fn cancel_in_flight_request(&mut self) -> bool {
        if let Some(request_id) = self.in_flight_request_id
            && let Some(ref tx) = self.request_tx
            && tx.send(AiRequest::Cancel { request_id }).is_ok()
        {
            log::debug!("Sent cancel for request {}", request_id);
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

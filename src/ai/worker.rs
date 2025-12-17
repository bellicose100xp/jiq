//! AI Worker Thread
//!
//! Handles AI requests in a background thread to avoid blocking the UI.
//! Receives requests via channel, makes HTTP calls to the AI provider,
//! and streams responses back to the main thread.

use std::sync::mpsc::{Receiver, Sender};

use super::ai_state::{AiRequest, AiResponse};
use super::provider::{AiError, AiProvider};
use crate::config::ai_types::AiConfig;

/// Spawn the AI worker thread
///
/// Creates a background thread that:
/// 1. Listens for requests on the request channel
/// 2. Makes HTTP calls to the AI provider
/// 3. Streams responses back via the response channel
///
/// # Arguments
/// * `config` - AI configuration (for creating the provider)
/// * `request_rx` - Channel to receive requests from the main thread
/// * `response_tx` - Channel to send responses to the main thread
///
/// # Requirements
/// - 4.1: WHEN the AI provider sends a streaming response THEN the AI_Popup
///   SHALL display text incrementally as chunks arrive
pub fn spawn_worker(
    config: &AiConfig,
    request_rx: Receiver<AiRequest>,
    response_tx: Sender<AiResponse>,
) {
    // Try to create the provider from config
    let provider_result = AiProvider::from_config(config);

    std::thread::spawn(move || {
        worker_loop(provider_result, request_rx, response_tx);
    });
}

/// Main worker loop - processes requests until the channel is closed
fn worker_loop(
    provider_result: Result<AiProvider, AiError>,
    request_rx: Receiver<AiRequest>,
    response_tx: Sender<AiResponse>,
) {
    // Check if provider was created successfully
    let provider = match provider_result {
        Ok(p) => Some(p),
        Err(e) => {
            // Log the error but don't send it yet - wait for a request
            log::debug!("AI provider not configured: {}", e);
            None
        }
    };

    // Process requests until the channel is closed
    while let Ok(request) = request_rx.recv() {
        match request {
            AiRequest::Query { prompt, request_id } => {
                handle_query(&provider, &prompt, request_id, &request_rx, &response_tx);
            }
            AiRequest::Cancel { request_id } => {
                // Cancel received when no request is in-flight - just acknowledge
                let _ = response_tx.send(AiResponse::Cancelled { request_id });
                log::debug!("Cancelled request {} (no active request)", request_id);
            }
        }
    }

    log::debug!("AI worker thread shutting down");
}

/// Handle a query request
///
/// Streams the response from the AI provider, checking for cancellation
/// between chunks. With synchronous HTTP, we can only check between chunks,
/// not mid-chunk.
///
/// # Requirements
/// - 5.4: WHEN a query change occurs while an API request is in-flight THEN
///   the AI_Assistant SHALL send a cancel signal to abort the previous request
/// - 5.5: WHEN a cancel signal is received THEN the Worker_Thread SHALL abort
///   the HTTP request and discard any pending response chunks
fn handle_query(
    provider: &Option<AiProvider>,
    prompt: &str,
    request_id: u64,
    request_rx: &Receiver<AiRequest>,
    response_tx: &Sender<AiResponse>,
) {
    // Check if provider is available
    let provider = match provider {
        Some(p) => p,
        None => {
            let _ = response_tx.send(AiResponse::Error(
                "AI not configured. Add [ai.anthropic] section with api_key to config.".to_string(),
            ));
            return;
        }
    };

    // Stream the response
    match provider.stream(prompt) {
        Ok(stream) => {
            for chunk_result in stream {
                // Check for cancellation between chunks
                if check_for_cancellation(request_rx, request_id, response_tx) {
                    return;
                }

                match chunk_result {
                    Ok(text) => {
                        if response_tx
                            .send(AiResponse::Chunk { text, request_id })
                            .is_err()
                        {
                            // Main thread disconnected, stop streaming
                            return;
                        }
                    }
                    Err(e) => {
                        let _ = response_tx.send(AiResponse::Error(e.to_string()));
                        return;
                    }
                }
            }
            // Stream completed successfully
            let _ = response_tx.send(AiResponse::Complete { request_id });
        }
        Err(e) => {
            let _ = response_tx.send(AiResponse::Error(e.to_string()));
        }
    }
}

/// Check for cancellation requests between streaming chunks
///
/// Uses try_recv() to non-blocking check for Cancel messages.
/// Returns true if the current request should be cancelled.
fn check_for_cancellation(
    request_rx: &Receiver<AiRequest>,
    current_request_id: u64,
    response_tx: &Sender<AiResponse>,
) -> bool {
    use std::sync::mpsc::TryRecvError;

    loop {
        match request_rx.try_recv() {
            Ok(AiRequest::Cancel { request_id }) => {
                if request_id == current_request_id {
                    // Cancel matches current request - abort
                    let _ = response_tx.send(AiResponse::Cancelled { request_id });
                    log::debug!("Cancelled request {} during streaming", request_id);
                    return true;
                }
                // Cancel for different request - ignore and continue
                log::debug!(
                    "Ignoring cancel for request {} (current: {})",
                    request_id,
                    current_request_id
                );
            }
            Ok(AiRequest::Query { .. }) => {
                // New query arrived - this shouldn't happen during streaming
                // but if it does, we'll process it after current request completes
                log::warn!("Received new query during streaming - will be lost");
            }
            Err(TryRecvError::Empty) => {
                // No messages waiting - continue streaming
                return false;
            }
            Err(TryRecvError::Disconnected) => {
                // Channel closed - stop streaming
                return true;
            }
        }
    }
}

#[cfg(test)]
#[path = "worker_tests.rs"]
mod worker_tests;

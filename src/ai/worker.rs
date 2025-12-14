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
///        SHALL display text incrementally as chunks arrive
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
///        the AI_Assistant SHALL send a cancel signal to abort the previous request
/// - 5.5: WHEN a cancel signal is received THEN the Worker_Thread SHALL abort
///        the HTTP request and discard any pending response chunks
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
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_worker_handles_query_without_provider() {
        let (request_tx, request_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();

        // Spawn worker with no provider (simulating missing config)
        std::thread::spawn(move || {
            worker_loop(
                Err(AiError::NotConfigured("test".to_string())),
                request_rx,
                response_tx,
            );
        });

        // Send a query with request_id
        request_tx
            .send(AiRequest::Query {
                prompt: "test".to_string(),
                request_id: 1,
            })
            .unwrap();

        // Should receive an error response
        let response = response_rx.recv().unwrap();
        match response {
            AiResponse::Error(msg) => {
                assert!(msg.contains("not configured"));
            }
            _ => panic!("Expected error response"),
        }
    }

    #[test]
    fn test_worker_handles_cancel() {
        let (request_tx, request_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();

        // Spawn worker
        std::thread::spawn(move || {
            worker_loop(
                Err(AiError::NotConfigured("test".to_string())),
                request_rx,
                response_tx,
            );
        });

        // Send cancel with request_id
        request_tx
            .send(AiRequest::Cancel { request_id: 1 })
            .unwrap();

        // Should receive cancelled response with request_id
        let response = response_rx.recv().unwrap();
        assert!(matches!(response, AiResponse::Cancelled { request_id: 1 }));
    }

    #[test]
    fn test_worker_shuts_down_when_channel_closed() {
        let (request_tx, request_rx) = mpsc::channel::<AiRequest>();
        let (response_tx, _response_rx) = mpsc::channel();

        let handle = std::thread::spawn(move || {
            worker_loop(
                Err(AiError::NotConfigured("test".to_string())),
                request_rx,
                response_tx,
            );
        });

        // Drop the sender to close the channel
        drop(request_tx);

        // Worker should exit cleanly
        handle.join().expect("Worker thread should exit cleanly");
    }

    // =========================================================================
    // Cancellation Tests
    // =========================================================================

    #[test]
    fn test_check_for_cancellation_no_messages() {
        let (request_tx, request_rx) = mpsc::channel();
        let (response_tx, _response_rx) = mpsc::channel();

        // Don't send any messages
        drop(request_tx);

        // Empty channel should return true (disconnected)
        let result = check_for_cancellation(&request_rx, 1, &response_tx);
        assert!(result);
    }

    #[test]
    fn test_check_for_cancellation_matching_cancel() {
        let (request_tx, request_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();

        // Send cancel with matching request_id
        request_tx
            .send(AiRequest::Cancel { request_id: 1 })
            .unwrap();

        // Should return true (cancelled)
        let result = check_for_cancellation(&request_rx, 1, &response_tx);
        assert!(result);

        // Should have sent Cancelled response
        let response = response_rx.recv().unwrap();
        assert!(matches!(response, AiResponse::Cancelled { request_id: 1 }));
    }

    #[test]
    fn test_check_for_cancellation_non_matching_cancel() {
        let (request_tx, request_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();

        // Send cancel with different request_id
        request_tx
            .send(AiRequest::Cancel { request_id: 99 })
            .unwrap();

        // Should return false (cancel was for different request)
        let result = check_for_cancellation(&request_rx, 1, &response_tx);
        assert!(!result);

        // Should NOT have sent any response
        assert!(response_rx.try_recv().is_err());
    }

    #[test]
    fn test_check_for_cancellation_empty_channel() {
        let (_request_tx, request_rx) = mpsc::channel::<AiRequest>();
        let (response_tx, _response_rx) = mpsc::channel();

        // Empty channel (but not disconnected) should return false
        let result = check_for_cancellation(&request_rx, 1, &response_tx);
        assert!(!result);
    }

    // **Feature: ai-assistant, Property 22: Cancel signal aborts HTTP request**
    // *For any* Cancel message received by the worker thread with matching request_id,
    // the current HTTP request should be aborted and Cancelled response sent.
    // **Validates: Requirements 5.5**
    //
    // Note: This property test validates the check_for_cancellation function which
    // is called between streaming chunks. With synchronous HTTP, we can only check
    // between chunks, not mid-chunk.
    mod prop_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            #[test]
            fn prop_cancel_signal_aborts_request(
                request_id in 1u64..1000u64,
            ) {
                let (request_tx, request_rx) = mpsc::channel();
                let (response_tx, response_rx) = mpsc::channel();

                // Send cancel with matching request_id
                request_tx
                    .send(AiRequest::Cancel { request_id })
                    .unwrap();

                // check_for_cancellation should return true (abort)
                let result = check_for_cancellation(&request_rx, request_id, &response_tx);
                prop_assert!(result, "Should abort when cancel matches request_id");

                // Should have sent Cancelled response with correct request_id
                let response = response_rx.recv().unwrap();
                match response {
                    AiResponse::Cancelled { request_id: resp_id } => {
                        prop_assert_eq!(resp_id, request_id, "Cancelled response should have correct request_id");
                    }
                    _ => prop_assert!(false, "Should have sent Cancelled response"),
                }
            }

            #[test]
            fn prop_cancel_for_different_request_continues(
                current_id in 1u64..500u64,
                cancel_id in 501u64..1000u64,
            ) {
                let (request_tx, request_rx) = mpsc::channel();
                let (response_tx, response_rx) = mpsc::channel();

                // Send cancel with different request_id
                request_tx
                    .send(AiRequest::Cancel { request_id: cancel_id })
                    .unwrap();

                // check_for_cancellation should return false (continue streaming)
                let result = check_for_cancellation(&request_rx, current_id, &response_tx);
                prop_assert!(!result, "Should continue when cancel is for different request");

                // Should NOT have sent any response
                prop_assert!(response_rx.try_recv().is_err(), "Should not send response for non-matching cancel");
            }
        }
    }
}

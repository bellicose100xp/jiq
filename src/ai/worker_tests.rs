//! Tests for AI worker thread

use super::*;
use proptest::prelude::*;
use std::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// Helper to run async tests with a tokio runtime
fn run_async<F: std::future::Future>(f: F) -> F::Output {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");
    rt.block_on(f)
}

#[test]
fn test_worker_handles_query_without_provider() {
    let (request_tx, request_rx) = mpsc::channel();
    let (response_tx, response_rx) = mpsc::channel();

    // Spawn worker with no provider (simulating missing config)
    // The worker now creates its own tokio runtime internally
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");
        rt.block_on(worker_loop(
            Err(AiError::NotConfigured {
                provider: "Test".to_string(),
                message: "test".to_string(),
            }),
            request_rx,
            response_tx,
        ));
    });

    // Send a query with request_id and cancel_token
    let cancel_token = CancellationToken::new();
    request_tx
        .send(AiRequest::Query {
            prompt: "test".to_string(),
            request_id: 1,
            cancel_token,
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
fn test_worker_handles_pre_cancelled_request() {
    let (request_tx, request_rx) = mpsc::channel();
    let (response_tx, response_rx) = mpsc::channel();

    // Spawn worker with no provider
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");
        rt.block_on(worker_loop(
            Err(AiError::NotConfigured {
                provider: "Test".to_string(),
                message: "test".to_string(),
            }),
            request_rx,
            response_tx,
        ));
    });

    // Create a token and cancel it before sending
    let cancel_token = CancellationToken::new();
    cancel_token.cancel();

    request_tx
        .send(AiRequest::Query {
            prompt: "test".to_string(),
            request_id: 1,
            cancel_token,
        })
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
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");
        rt.block_on(worker_loop(
            Err(AiError::NotConfigured {
                provider: "Test".to_string(),
                message: "test".to_string(),
            }),
            request_rx,
            response_tx,
        ));
    });

    // Drop the sender to close the channel
    drop(request_tx);

    // Worker should exit cleanly
    handle.join().expect("Worker thread should exit cleanly");
}

// =========================================================================
// CancellationToken Tests
// =========================================================================

#[test]
fn test_cancellation_token_not_cancelled_initially() {
    let token = CancellationToken::new();
    assert!(!token.is_cancelled());
}

#[test]
fn test_cancellation_token_cancelled_after_cancel() {
    let token = CancellationToken::new();
    token.cancel();
    assert!(token.is_cancelled());
}

// **Feature: ai-request-cancellation, Property 6: Idempotent cancellation**
// *For any* CancellationToken, calling cancel() multiple times should have the same
// effect as calling it once - the token remains cancelled.
// **Validates: Requirements 3.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_cancellation_is_idempotent(
        cancel_count in 1usize..10usize,
    ) {
        let token = CancellationToken::new();

        // Initially not cancelled
        prop_assert!(!token.is_cancelled(), "Token should not be cancelled initially");

        // Cancel multiple times
        for _ in 0..cancel_count {
            token.cancel();
        }

        // Should still be cancelled
        prop_assert!(token.is_cancelled(), "Token should be cancelled after cancel()");

        // Cancel again to verify idempotence
        token.cancel();
        prop_assert!(token.is_cancelled(), "Token should remain cancelled after additional cancel()");
    }

    #[test]
    fn prop_cancel_signal_aborts_request(
        request_id in 1u64..1000u64,
    ) {
        let (response_tx, response_rx) = mpsc::channel();
        let cancel_token = CancellationToken::new();

        // Cancel the token
        cancel_token.cancel();

        // handle_query_async should detect cancellation and send Cancelled response
        run_async(handle_query_async(&None, "test prompt", request_id, cancel_token, &response_tx));

        // Should have sent Cancelled response with correct request_id
        let response = response_rx.recv().unwrap();
        match response {
            AiResponse::Cancelled { request_id: resp_id } => {
                prop_assert_eq!(resp_id, request_id, "Cancelled response should have correct request_id");
            }
            _ => prop_assert!(false, "Should have sent Cancelled response, got {:?}", response),
        }
    }
}

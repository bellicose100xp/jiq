//! Tests for AI worker thread

use super::*;
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

#[test]
fn test_cancellation_is_idempotent() {
    let token = CancellationToken::new();
    assert!(!token.is_cancelled());

    token.cancel();
    assert!(token.is_cancelled());

    token.cancel();
    token.cancel();
    assert!(
        token.is_cancelled(),
        "Token should remain cancelled after multiple cancel()"
    );
}

#[test]
fn test_cancel_signal_aborts_request() {
    let (response_tx, response_rx) = mpsc::channel();
    let cancel_token = CancellationToken::new();
    let request_id = 42;

    cancel_token.cancel();

    run_async(handle_query_async(
        &None,
        "test prompt",
        request_id,
        cancel_token,
        &response_tx,
    ));

    let response = response_rx.recv().unwrap();
    match response {
        AiResponse::Cancelled {
            request_id: resp_id,
        } => {
            assert_eq!(resp_id, request_id);
        }
        _ => panic!("Should have sent Cancelled response, got {:?}", response),
    }
}

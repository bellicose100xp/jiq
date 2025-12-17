//! Tests for AI worker thread

use super::*;
use proptest::prelude::*;
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

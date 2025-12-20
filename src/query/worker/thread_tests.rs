// Basic worker thread tests
// More comprehensive tests will be added in Phase 3.4

use super::*;
use std::sync::mpsc::channel;
use tokio_util::sync::CancellationToken;

#[test]
fn test_worker_spawns_successfully() {
    let json_input = r#"{"test": "data"}"#.to_string();
    let (request_tx, request_rx) = channel();
    let (response_tx, response_rx) = channel();

    // Spawn worker - should not panic
    spawn_worker(json_input, request_rx, response_tx);

    // Send a simple query
    let cancel_token = CancellationToken::new();
    request_tx
        .send(QueryRequest {
            query: ".".to_string(),
            request_id: 1,
            cancel_token,
        })
        .unwrap();

    // Should get a success response
    match response_rx.recv_timeout(std::time::Duration::from_secs(2)) {
        Ok(QueryResponse::Success { query, .. }) => {
            assert_eq!(query, ".");
        }
        Ok(other) => panic!("Expected Success, got {:?}", other),
        Err(e) => panic!("Timeout waiting for response: {}", e),
    }
}

#[test]
fn test_worker_handles_invalid_query() {
    let json_input = r#"{"test": "data"}"#.to_string();
    let (request_tx, request_rx) = channel();
    let (response_tx, response_rx) = channel();

    spawn_worker(json_input, request_rx, response_tx);

    // Send an invalid query
    let cancel_token = CancellationToken::new();
    request_tx
        .send(QueryRequest {
            query: ".invalid syntax [".to_string(),
            request_id: 1,
            cancel_token,
        })
        .unwrap();

    // Should get an error response
    match response_rx.recv_timeout(std::time::Duration::from_secs(2)) {
        Ok(QueryResponse::Error { .. }) => {
            // Success - got expected error
        }
        Ok(other) => panic!("Expected Error, got {:?}", other),
        Err(e) => panic!("Timeout waiting for response: {}", e),
    }
}

#[test]
fn test_worker_handles_pre_cancelled_request() {
    let json_input = r#"{"test": "data"}"#.to_string();
    let (request_tx, request_rx) = channel();
    let (response_tx, response_rx) = channel();

    spawn_worker(json_input, request_rx, response_tx);

    // Cancel before sending
    let cancel_token = CancellationToken::new();
    cancel_token.cancel();

    request_tx
        .send(QueryRequest {
            query: ".".to_string(),
            request_id: 1,
            cancel_token,
        })
        .unwrap();

    // Should get a cancelled response
    match response_rx.recv_timeout(std::time::Duration::from_secs(2)) {
        Ok(QueryResponse::Cancelled { .. }) => {
            // Success
        }
        Ok(other) => panic!("Expected Cancelled, got {:?}", other),
        Err(e) => panic!("Timeout waiting for response: {}", e),
    }
}

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
        Ok(QueryResponse::ProcessedSuccess { processed, .. }) => {
            assert_eq!(processed.query, ".");
        }
        Ok(other) => panic!("Expected Success or ProcessedSuccess, got {:?}", other),
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

#[test]
fn test_worker_sends_error_response_for_jq_failure() {
    let json_input = r#"{"test": "data"}"#.to_string();
    let (request_tx, request_rx) = channel();
    let (response_tx, response_rx) = channel();

    spawn_worker(json_input, request_rx, response_tx);

    // Send query with invalid syntax
    let cancel_token = CancellationToken::new();
    request_tx
        .send(QueryRequest {
            query: ".invalid syntax [".to_string(),
            request_id: 1,
            cancel_token,
        })
        .unwrap();

    // Should get error response with correct request_id and query
    match response_rx.recv_timeout(std::time::Duration::from_secs(2)) {
        Ok(QueryResponse::Error {
            message,
            query,
            request_id,
        }) => {
            assert_eq!(request_id, 1);
            assert_eq!(query, ".invalid syntax [");
            assert!(message.contains("parse error") || message.contains("syntax"));
        }
        Ok(other) => panic!("Expected Error, got {:?}", other),
        Err(e) => panic!("Timeout waiting for error response: {}", e),
    }
}

#[test]
fn test_worker_handles_multiple_rapid_queries() {
    let json_input = r#"{"a": 1, "b": 2, "c": 3}"#.to_string();
    let (request_tx, request_rx) = channel();
    let (response_tx, response_rx) = channel();

    spawn_worker(json_input, request_rx, response_tx);

    // Send multiple queries rapidly
    for i in 1..=5 {
        let cancel_token = CancellationToken::new();
        request_tx
            .send(QueryRequest {
                query: format!(".{}", if i % 2 == 0 { "a" } else { "b" }),
                request_id: i,
                cancel_token,
            })
            .unwrap();
    }

    // Should receive 5 responses
    let mut received_count = 0;
    for _ in 0..5 {
        match response_rx.recv_timeout(std::time::Duration::from_secs(3)) {
            Ok(QueryResponse::Success { .. })
            | Ok(QueryResponse::ProcessedSuccess { .. })
            | Ok(QueryResponse::Error { .. }) => {
                received_count += 1;
            }
            Ok(QueryResponse::Cancelled { .. }) => {
                // Acceptable - query was cancelled
                received_count += 1;
            }
            Err(e) => panic!("Timeout after {} responses: {}", received_count, e),
        }
    }

    assert_eq!(received_count, 5, "Should receive all 5 responses");
}

#[test]
fn test_worker_response_includes_original_query() {
    let json_input = r#"{"test": "value"}"#.to_string();
    let (request_tx, request_rx) = channel();
    let (response_tx, response_rx) = channel();

    spawn_worker(json_input, request_rx, response_tx);

    let original_query = ".test";
    let cancel_token = CancellationToken::new();
    request_tx
        .send(QueryRequest {
            query: original_query.to_string(),
            request_id: 42,
            cancel_token,
        })
        .unwrap();

    // Response should include the original query
    match response_rx.recv_timeout(std::time::Duration::from_secs(2)) {
        Ok(QueryResponse::Success {
            query, request_id, ..
        }) => {
            assert_eq!(
                query, original_query,
                "Response should include original query"
            );
            assert_eq!(request_id, 42);
        }
        Ok(QueryResponse::ProcessedSuccess {
            processed,
            request_id,
        }) => {
            assert_eq!(
                processed.query, original_query,
                "Response should include original query"
            );
            assert_eq!(request_id, 42);
        }
        Ok(other) => panic!("Expected Success or ProcessedSuccess, got {:?}", other),
        Err(e) => panic!("Timeout: {}", e),
    }
}

#[test]
fn test_worker_error_response_includes_original_query() {
    let json_input = r#"{"test": "value"}"#.to_string();
    let (request_tx, request_rx) = channel();
    let (response_tx, response_rx) = channel();

    spawn_worker(json_input, request_rx, response_tx);

    let original_query = ".invalid syntax [";
    let cancel_token = CancellationToken::new();
    request_tx
        .send(QueryRequest {
            query: original_query.to_string(),
            request_id: 99,
            cancel_token,
        })
        .unwrap();

    // Error response should include the original query
    match response_rx.recv_timeout(std::time::Duration::from_secs(2)) {
        Ok(QueryResponse::Error {
            query, request_id, ..
        }) => {
            assert_eq!(
                query, original_query,
                "Error response should include original query"
            );
            assert_eq!(request_id, 99);
        }
        Ok(other) => panic!("Expected Error, got {:?}", other),
        Err(e) => panic!("Timeout: {}", e),
    }
}

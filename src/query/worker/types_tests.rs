use super::*;

#[test]
fn test_query_error_display() {
    let err = QueryError::SpawnFailed("test error".to_string());
    assert_eq!(err.to_string(), "Failed to spawn jq: test error");

    let err = QueryError::StdinWriteFailed("write error".to_string());
    assert_eq!(err.to_string(), "Failed to write to jq stdin: write error");

    let err = QueryError::OutputReadFailed("read error".to_string());
    assert_eq!(err.to_string(), "Failed to read jq output: read error");

    let err = QueryError::Cancelled;
    assert_eq!(err.to_string(), "Query execution cancelled");

    let err = QueryError::ExecutionFailed("jq error".to_string());
    assert_eq!(err.to_string(), "jq error");
}

#[test]
fn test_query_request_creation() {
    let cancel_token = CancellationToken::new();
    let request = QueryRequest {
        query: ".foo".to_string(),
        request_id: 42,
        cancel_token: cancel_token.clone(),
    };

    assert_eq!(request.query, ".foo");
    assert_eq!(request.request_id, 42);
    assert!(!request.cancel_token.is_cancelled());
}

#[test]
fn test_query_response_variants() {
    // Test Success variant
    let response = QueryResponse::Success {
        output: "result".to_string(),
        query: ".foo".to_string(),
        request_id: 1,
    };
    match response {
        QueryResponse::Success {
            output,
            query,
            request_id,
        } => {
            assert_eq!(output, "result");
            assert_eq!(query, ".foo");
            assert_eq!(request_id, 1);
        }
        _ => panic!("Expected Success variant"),
    }

    // Test Error variant
    let response = QueryResponse::Error {
        message: "error".to_string(),
        request_id: 2,
    };
    match response {
        QueryResponse::Error {
            message,
            request_id,
        } => {
            assert_eq!(message, "error");
            assert_eq!(request_id, 2);
        }
        _ => panic!("Expected Error variant"),
    }

    // Test Cancelled variant
    let response = QueryResponse::Cancelled { request_id: 3 };
    match response {
        QueryResponse::Cancelled { request_id } => {
            assert_eq!(request_id, 3);
        }
        _ => panic!("Expected Cancelled variant"),
    }
}

#[test]
fn test_worker_error_request_id() {
    // Test that request_id = 0 is reserved for worker-level errors
    let response = QueryResponse::Error {
        message: "Worker crashed".to_string(),
        request_id: 0,
    };
    match response {
        QueryResponse::Error { request_id, .. } => {
            assert_eq!(
                request_id, 0,
                "Worker-level errors should use request_id = 0"
            );
        }
        _ => panic!("Expected Error variant"),
    }
}

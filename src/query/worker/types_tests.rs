use super::*;

#[test]
fn test_query_error_display() {
    let err = QueryError::SpawnFailed("test error".to_string());
    assert_eq!(err.to_string(), "Failed to spawn jq: test error");

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
    // Test ProcessedSuccess variant
    let processed = ProcessedResult {
        output: std::sync::Arc::new("result".to_string()),
        unformatted: std::sync::Arc::new("result".to_string()),
        rendered_lines: vec![],
        parsed: None,
        line_count: 1,
        max_width: 6,
        result_type: ResultType::String,
        query: ".foo".to_string(),
        execution_time_ms: None,
    };
    let response = QueryResponse::ProcessedSuccess {
        processed,
        request_id: 1,
    };
    match response {
        QueryResponse::ProcessedSuccess {
            processed,
            request_id,
        } => {
            assert_eq!(*processed.output, "result");
            assert_eq!(processed.query, ".foo");
            assert_eq!(request_id, 1);
        }
        _ => panic!("Expected ProcessedSuccess variant"),
    }

    // Test Error variant
    let response = QueryResponse::Error {
        message: "error".to_string(),
        query: ".invalid".to_string(),
        request_id: 2,
    };
    match response {
        QueryResponse::Error {
            message,
            query,
            request_id,
        } => {
            assert_eq!(message, "error");
            assert_eq!(query, ".invalid");
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
        query: String::new(), // Worker-level errors have no associated query
        request_id: 0,
    };
    match response {
        QueryResponse::Error {
            request_id, query, ..
        } => {
            assert_eq!(
                request_id, 0,
                "Worker-level errors should use request_id = 0"
            );
            assert!(
                query.is_empty(),
                "Worker-level errors should have empty query"
            );
        }
        _ => panic!("Expected Error variant"),
    }
}

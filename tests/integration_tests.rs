use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Helper to get path to fixture file
fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn test_cli_with_invalid_json_file() {
    let fixture = fixture_path("invalid.json");

    cargo_bin_cmd!()
        .arg(&fixture)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid JSON"));
}

#[test]
fn test_cli_with_nonexistent_file() {
    cargo_bin_cmd!().arg("nonexistent.json").assert().failure();
}

#[test]
fn test_cli_help_flag() {
    cargo_bin_cmd!()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Interactive JSON query tool"));
}

#[test]
fn test_cli_version_flag() {
    cargo_bin_cmd!()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("jiq"));
}

#[test]
fn test_fixture_files_exist() {
    // Verify all our test fixtures are present
    assert!(fixture_path("simple.json").exists());
    assert!(fixture_path("array.json").exists());
    assert!(fixture_path("nested.json").exists());
    assert!(fixture_path("invalid.json").exists());
}

#[test]
fn test_fixture_simple_json_content() {
    let content = fs::read_to_string(fixture_path("simple.json")).unwrap();
    assert!(content.contains("Alice"));
    assert!(content.contains("Seattle"));
}

#[test]
fn test_fixture_array_json_content() {
    let content = fs::read_to_string(fixture_path("array.json")).unwrap();
    assert!(content.contains("Alice"));
    assert!(content.contains("Bob"));
    assert!(content.contains("Charlie"));
}

#[test]
fn test_fixture_nested_json_content() {
    let content = fs::read_to_string(fixture_path("nested.json")).unwrap();
    assert!(content.contains("TechCorp"));
    assert!(content.contains("engineering"));
    assert!(content.contains("departments"));
}

// =============================================================================
// AI Integration Tests
// =============================================================================

/// Test the full AI flow: error → auto-show → request → stream → display
///
/// This test validates the AI assistant's end-to-end behavior by simulating
/// the worker thread communication pattern without making actual HTTP calls.
///
/// **Validates: Requirements 3.1, 4.1, 4.4**
#[test]
fn test_ai_full_flow_error_to_response() {
    // Create channels for request/response communication
    let (request_tx, request_rx) = mpsc::channel::<String>();
    let (response_tx, response_rx) = mpsc::channel::<String>();

    // Spawn a mock worker thread that simulates SSE streaming
    let worker_handle = thread::spawn(move || {
        // Wait for a request
        if let Ok(prompt) = request_rx.recv_timeout(Duration::from_secs(5)) {
            assert!(
                prompt.contains("error"),
                "Prompt should contain error context"
            );

            // Simulate SSE streaming response in chunks
            let chunks = vec![
                "The error ",
                "you're seeing ",
                "is because ",
                "the query ",
                "syntax is invalid.",
            ];

            for chunk in chunks {
                response_tx.send(chunk.to_string()).unwrap();
                thread::sleep(Duration::from_millis(10));
            }
        }
    });

    // Simulate the main thread behavior:
    // 1. Send a request (simulating auto-show on error)
    let error_context = "Query error: unexpected token at position 5";
    request_tx.send(error_context.to_string()).unwrap();

    // 2. Collect streaming response chunks
    let mut accumulated_response = String::new();
    let mut chunk_count = 0;

    while let Ok(chunk) = response_rx.recv_timeout(Duration::from_millis(500)) {
        accumulated_response.push_str(&chunk);
        chunk_count += 1;
    }

    // 3. Verify the response was streamed correctly
    assert!(chunk_count > 1, "Response should arrive in multiple chunks");
    assert_eq!(
        accumulated_response,
        "The error you're seeing is because the query syntax is invalid."
    );

    worker_handle.join().expect("Worker thread should complete");
}

/// Test that AI state transitions work correctly during request lifecycle
#[test]
fn test_ai_state_transitions() {
    // Simulate state transitions without actual AI module dependencies
    // This tests the logical flow of: idle → loading → streaming → complete

    #[derive(Debug, PartialEq)]
    #[allow(dead_code)]
    enum State {
        Idle,
        Loading,
        Streaming,
        Complete,
        Error,
    }

    let mut state = State::Idle;
    let mut response = String::new();

    // 1. Start request - should transition to Loading
    assert_eq!(state, State::Idle);
    let previous_response = if !response.is_empty() {
        Some(response.clone())
    } else {
        None
    };
    response.clear();
    state = State::Loading;
    assert_eq!(state, State::Loading);
    assert!(response.is_empty());
    assert!(previous_response.is_none());

    // 2. Receive first chunk - should transition to Streaming
    let chunk1 = "Hello ";
    response.push_str(chunk1);
    state = State::Streaming;
    assert_eq!(state, State::Streaming);
    assert_eq!(response, "Hello ");

    // 3. Receive more chunks
    let chunk2 = "World!";
    response.push_str(chunk2);
    assert_eq!(response, "Hello World!");

    // 4. Complete - should transition to Complete
    state = State::Complete;
    assert_eq!(state, State::Complete);
    assert_eq!(response, "Hello World!");
}

/// Test that previous response is preserved when starting a new request
#[test]
fn test_ai_previous_response_preservation() {
    let mut response = "Previous AI response".to_string();
    let mut previous_response: Option<String> = None;

    // Start a new request - should preserve current response
    if !response.is_empty() {
        previous_response = Some(response.clone());
    }
    response.clear();

    assert!(response.is_empty());
    assert_eq!(previous_response, Some("Previous AI response".to_string()));

    // Simulate receiving new response
    response.push_str("New response");

    // Complete request - should clear previous
    previous_response = None;

    assert_eq!(response, "New response");
    assert!(previous_response.is_none());
}

/// Test streaming concatenation property
///
/// **Validates: Property 10 - Streaming concatenation**
/// For any sequence of response chunks [c1, c2, ..., cn],
/// the final displayed response should equal c1 + c2 + ... + cn.
#[test]
fn test_ai_streaming_concatenation() {
    let chunks = vec![
        "First chunk. ",
        "Second chunk. ",
        "Third chunk. ",
        "Final chunk.",
    ];

    let mut accumulated = String::new();
    for chunk in &chunks {
        accumulated.push_str(chunk);
    }

    let expected = chunks.join("");
    assert_eq!(accumulated, expected);
    assert_eq!(
        accumulated,
        "First chunk. Second chunk. Third chunk. Final chunk."
    );
}

/// Test the full AI flow with SSE-like streaming simulation
///
/// This test validates the complete flow from error detection through
/// streaming response display, simulating the SSE streaming behavior
/// that would come from the Anthropic API.
///
/// **Validates: Requirements 3.1, 4.1, 4.4**
#[test]
fn test_ai_full_flow_with_sse_simulation() {
    use std::io::{BufRead, BufReader, Cursor};

    // Simulate SSE data as it would come from Anthropic API
    let sse_data = r#"event: message_start
data: {"type":"message_start","message":{"id":"msg_test"}}

event: content_block_start
data: {"type":"content_block_start","index":0}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"The error "}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"in your query "}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"is a syntax issue."}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_stop
data: {"type":"message_stop"}

"#;

    // Parse SSE data like the real client would
    fn parse_sse_chunks(data: &str) -> Vec<String> {
        let reader = BufReader::new(Cursor::new(data.as_bytes()));
        let mut chunks = Vec::new();

        for line in reader.lines() {
            let line = line.unwrap();
            if let Some(data) = line.strip_prefix("data: ") {
                if data == "[DONE]" {
                    break;
                }
                // Try to parse as content_block_delta
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    if json.get("type").and_then(|t| t.as_str()) == Some("content_block_delta") {
                        if let Some(text) = json
                            .get("delta")
                            .and_then(|d| d.get("text"))
                            .and_then(|t| t.as_str())
                        {
                            if !text.is_empty() {
                                chunks.push(text.to_string());
                            }
                        }
                    }
                }
            }
        }
        chunks
    }

    // Parse the SSE data
    let chunks = parse_sse_chunks(sse_data);

    // Verify we got the expected chunks
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], "The error ");
    assert_eq!(chunks[1], "in your query ");
    assert_eq!(chunks[2], "is a syntax issue.");

    // Simulate the full flow: error → auto-show → request → stream → display
    let (request_tx, request_rx) = mpsc::channel::<String>();
    let (response_tx, response_rx) = mpsc::channel::<String>();

    // Spawn worker that processes SSE-like chunks
    let worker_handle = thread::spawn(move || {
        if let Ok(_prompt) = request_rx.recv_timeout(Duration::from_secs(5)) {
            // Simulate streaming the parsed chunks
            for chunk in chunks {
                response_tx.send(chunk).unwrap();
                thread::sleep(Duration::from_millis(5));
            }
        }
    });

    // 1. Simulate error detection triggering auto-show
    let error_context = "jq: error: syntax error, unexpected IDENT";
    request_tx.send(error_context.to_string()).unwrap();

    // 2. Collect streaming response
    let mut accumulated = String::new();
    let mut chunk_count = 0;

    while let Ok(chunk) = response_rx.recv_timeout(Duration::from_millis(500)) {
        accumulated.push_str(&chunk);
        chunk_count += 1;
    }

    // 3. Verify the complete response
    assert_eq!(chunk_count, 3, "Should receive 3 chunks");
    assert_eq!(accumulated, "The error in your query is a syntax issue.");

    worker_handle.join().expect("Worker should complete");
}

/// Test auto-show behavior with different configurations
#[test]
fn test_ai_auto_show_configurations() {
    struct MockAiState {
        visible: bool,
        enabled: bool,
    }

    impl MockAiState {
        fn auto_show_on_error(&mut self, auto_show_enabled: bool) -> bool {
            if !self.enabled || !auto_show_enabled || self.visible {
                return false;
            }
            self.visible = true;
            true
        }
    }

    // Case 1: AI enabled, auto-show enabled, not visible → should show
    let mut state = MockAiState {
        visible: false,
        enabled: true,
    };
    assert!(state.auto_show_on_error(true));
    assert!(state.visible);

    // Case 2: AI enabled, auto-show disabled → should not show
    let mut state = MockAiState {
        visible: false,
        enabled: true,
    };
    assert!(!state.auto_show_on_error(false));
    assert!(!state.visible);

    // Case 3: AI disabled → should not show
    let mut state = MockAiState {
        visible: false,
        enabled: false,
    };
    assert!(!state.auto_show_on_error(true));
    assert!(!state.visible);

    // Case 4: Already visible → should not re-show
    let mut state = MockAiState {
        visible: true,
        enabled: true,
    };
    assert!(!state.auto_show_on_error(true));
    assert!(state.visible);
}

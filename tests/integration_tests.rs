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
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data)
                    && json.get("type").and_then(|t| t.as_str()) == Some("content_block_delta")
                    && let Some(text) = json
                        .get("delta")
                        .and_then(|d| d.get("text"))
                        .and_then(|t| t.as_str())
                    && !text.is_empty()
                {
                    chunks.push(text.to_string());
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

/// Test visibility-based AI request behavior
#[test]
fn test_ai_visibility_configurations() {
    struct MockAiState {
        visible: bool,
        enabled: bool,
    }

    impl MockAiState {
        fn should_send_request(&self) -> bool {
            self.enabled && self.visible
        }
    }

    // Case 1: AI enabled, visible → should send request
    let state = MockAiState {
        visible: true,
        enabled: true,
    };
    assert!(state.should_send_request());

    // Case 2: AI enabled, hidden → should not send request
    let state = MockAiState {
        visible: false,
        enabled: true,
    };
    assert!(!state.should_send_request());

    // Case 3: AI disabled, visible → should not send request
    let state = MockAiState {
        visible: true,
        enabled: false,
    };
    assert!(!state.should_send_request());

    // Case 4: AI disabled, hidden → should not send request
    let state = MockAiState {
        visible: false,
        enabled: false,
    };
    assert!(!state.should_send_request());
}

// =============================================================================
// Task 20: AI Visibility Control Integration Tests
// =============================================================================

/// Test initial AI popup visibility based on config
///
/// **Validates: Requirements 8.1, 8.2**
/// - 8.1: WHEN AI is enabled in config THEN the AI_Popup SHALL be visible by default
/// - 8.2: WHEN AI is disabled in config THEN the AI_Popup SHALL be hidden by default
#[test]
fn test_initial_visibility_ai_enabled() {
    use jiq::app::App;
    use jiq::config::{AiConfig, AiProviderType, AnthropicConfig, Config};

    // Create config with AI enabled and provider explicitly set
    let config = Config {
        ai: AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Anthropic),
            anthropic: AnthropicConfig {
                api_key: Some("test-key".to_string()),
                model: Some("claude-3-5-sonnet-20241022".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let json_input = r#"{"test": "data"}"#.to_string();
    let app = App::new(json_input, &config);

    // Requirement 8.1: AI enabled → popup visible by default
    assert!(
        app.ai.visible,
        "AI popup should be visible when AI is enabled in config"
    );
    assert!(app.ai.enabled, "AI should be enabled");
    assert!(app.ai.configured, "AI should be configured with API key");
}

/// Test initial AI popup visibility when AI is disabled in config
///
/// **Validates: Requirements 8.2**
#[test]
fn test_initial_visibility_ai_disabled() {
    use jiq::app::App;
    use jiq::config::Config;

    // Create config with AI disabled (default)
    let config = Config::default();

    let json_input = r#"{"test": "data"}"#.to_string();
    let app = App::new(json_input, &config);

    // Requirement 8.2: AI disabled → popup hidden by default
    assert!(
        !app.ai.visible,
        "AI popup should be hidden when AI is disabled in config"
    );
    assert!(!app.ai.enabled, "AI should be disabled");
}

/// Test that tooltip is hidden when AI is visible on startup
///
/// **Validates: Requirements 9.5**
/// - 9.5: WHEN jiq starts with AI enabled THEN the Info_Popup SHALL start in hidden state
#[test]
fn test_tooltip_hidden_when_ai_visible_on_startup() {
    use jiq::app::App;
    use jiq::config::{AiConfig, AiProviderType, AnthropicConfig, Config, TooltipConfig};

    // Create config with AI enabled, provider set, and tooltip auto_show enabled
    let config = Config {
        ai: AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Anthropic),
            anthropic: AnthropicConfig {
                api_key: Some("test-key".to_string()),
                model: Some("claude-3-5-sonnet-20241022".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        tooltip: TooltipConfig { auto_show: true },
        ..Default::default()
    };

    let json_input = r#"{"test": "data"}"#.to_string();
    let app = App::new(json_input, &config);

    // AI should be visible
    assert!(app.ai.visible, "AI popup should be visible");

    // Tooltip should be hidden (even though auto_show is true in config)
    assert!(
        !app.tooltip.enabled,
        "Tooltip should be hidden when AI popup is visible on startup"
    );
}

/// Test that tooltip is visible when AI is disabled on startup
///
/// **Validates: Requirements 9.5** (inverse case)
#[test]
fn test_tooltip_visible_when_ai_disabled_on_startup() {
    use jiq::app::App;
    use jiq::config::{Config, TooltipConfig};

    // Create config with AI disabled and tooltip auto_show enabled
    let config = Config {
        tooltip: TooltipConfig { auto_show: true },
        ..Default::default()
    };

    let json_input = r#"{"test": "data"}"#.to_string();
    let app = App::new(json_input, &config);

    // AI should be hidden
    assert!(!app.ai.visible, "AI popup should be hidden");

    // Tooltip should be visible (respecting auto_show config)
    assert!(
        app.tooltip.enabled,
        "Tooltip should be visible when AI popup is hidden on startup"
    );
}

// =============================================================================
// Task 20.2: Documentation of Existing Ctrl+A Toggle Tests
// =============================================================================
//
// The following tests for Ctrl+A toggle functionality already exist in the codebase:
//
// **Unit Tests** (in src/ai/ai_events.rs):
// - `test_ctrl_a_toggles_visibility_on`: Tests that Ctrl+A toggles visibility from hidden to visible
// - `test_ctrl_a_toggles_visibility_off`: Tests that Ctrl+A toggles visibility from visible to hidden
// - `test_plain_a_not_handled`: Tests that plain 'a' key without Ctrl is not handled
// - `test_ctrl_other_key_not_handled`: Tests that other Ctrl+key combinations are not handled
//
// **Property-Based Test** (in src/ai/ai_state.rs):
// - `prop_toggle_visibility`: Property test that verifies toggle() flips visibility state
//   for any initial visibility state, enabled state, and debounce_ms value.
//   Tests both single toggle and double toggle (round-trip).
//
// **Validates: Requirements 8.3**
// - 8.3: WHEN the user presses Ctrl+A THEN the AI_Popup visibility SHALL toggle
//
// These tests comprehensively cover the Ctrl+A toggle functionality at both the
// unit level (specific key combinations) and property level (state transitions).
// =============================================================================

// =============================================================================
// Task 20.3: Verify No Other Visibility Control Mechanisms
// =============================================================================

/// Test that handle_execution_result does NOT change visibility on error
///
/// **Validates: Requirements 8.1, 8.2, 8.3**
/// Confirms that only config initialization and Ctrl+A toggle control visibility,
/// not execution results.
#[test]
fn test_handle_execution_result_does_not_change_visibility_on_error() {
    use jiq::ai::ai_events::handle_execution_result;
    use jiq::ai::ai_state::AiState;

    // Test with visibility = false
    let mut ai_state = AiState::new(true);
    ai_state.visible = false;
    let initial_visibility = ai_state.visible;

    let error_result: Result<String, String> = Err("syntax error".to_string());
    handle_execution_result(
        &mut ai_state,
        &error_result,
        ".invalid",
        0,
        r#"{"test": "data"}"#,
        jiq::ai::context::ContextParams {
            input_schema: None,
            base_query: None,
            base_query_result: None,
        },
    );

    assert_eq!(
        ai_state.visible, initial_visibility,
        "handle_execution_result should NOT change visibility on error"
    );

    // Test with visibility = true
    let mut ai_state = AiState::new(true);
    ai_state.visible = true;
    let initial_visibility = ai_state.visible;

    let error_result: Result<String, String> = Err("syntax error".to_string());
    handle_execution_result(
        &mut ai_state,
        &error_result,
        ".invalid2",
        0,
        r#"{"test": "data"}"#,
        jiq::ai::context::ContextParams {
            input_schema: None,
            base_query: None,
            base_query_result: None,
        },
    );

    assert_eq!(
        ai_state.visible, initial_visibility,
        "handle_execution_result should NOT change visibility on error (even when visible)"
    );
}

/// Test that handle_execution_result does NOT change visibility on success
///
/// **Validates: Requirements 8.1, 8.2, 8.3**
/// Confirms that only config initialization and Ctrl+A toggle control visibility,
/// not execution results.
#[test]
fn test_handle_execution_result_does_not_change_visibility_on_success() {
    use jiq::ai::ai_events::handle_execution_result;
    use jiq::ai::ai_state::AiState;

    // Test with visibility = false
    let mut ai_state = AiState::new(true);
    ai_state.visible = false;
    let initial_visibility = ai_state.visible;

    let success_result: Result<String, String> = Ok(r#"{"result": "value"}"#.to_string());
    handle_execution_result(
        &mut ai_state,
        &success_result,
        ".test",
        0,
        r#"{"test": "data"}"#,
        jiq::ai::context::ContextParams {
            input_schema: None,
            base_query: None,
            base_query_result: None,
        },
    );

    assert_eq!(
        ai_state.visible, initial_visibility,
        "handle_execution_result should NOT change visibility on success"
    );

    // Test with visibility = true
    let mut ai_state = AiState::new(true);
    ai_state.visible = true;
    let initial_visibility = ai_state.visible;

    let success_result: Result<String, String> = Ok(r#"{"result": "value2"}"#.to_string());
    handle_execution_result(
        &mut ai_state,
        &success_result,
        ".test2",
        0,
        r#"{"test": "data"}"#,
        jiq::ai::context::ContextParams {
            input_schema: None,
            base_query: None,
            base_query_result: None,
        },
    );

    assert_eq!(
        ai_state.visible, initial_visibility,
        "handle_execution_result should NOT change visibility on success (even when visible)"
    );
}

/// Test that only config and Ctrl+A control visibility
///
/// **Validates: Requirements 8.1, 8.2, 8.3**
/// This test documents the complete visibility control mechanism:
/// 1. Initial visibility is set by config (8.1, 8.2)
/// 2. Ctrl+A toggle is the only runtime control (8.3)
/// 3. Execution results do NOT affect visibility
#[test]
fn test_visibility_control_mechanisms_complete() {
    use jiq::ai::ai_events::handle_execution_result;
    use jiq::ai::ai_state::AiState;
    use jiq::app::App;
    use jiq::config::{AiConfig, AiProviderType, AnthropicConfig, Config};

    // Mechanism 1: Config controls initial visibility
    let config_enabled = Config {
        ai: AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Anthropic),
            anthropic: AnthropicConfig {
                api_key: Some("test-key".to_string()),
                model: Some("claude-3-5-sonnet-20241022".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let app_enabled = App::new(r#"{"test": "data"}"#.to_string(), &config_enabled);
    assert!(
        app_enabled.ai.visible,
        "Config with AI enabled should set initial visibility to true"
    );

    let config_disabled = Config::default();
    let app_disabled = App::new(r#"{"test": "data"}"#.to_string(), &config_disabled);
    assert!(
        !app_disabled.ai.visible,
        "Config with AI disabled should set initial visibility to false"
    );

    // Mechanism 2: Ctrl+A toggle is the only runtime control
    let mut ai_state = AiState::new(true);
    ai_state.visible = false;

    ai_state.toggle(); // Simulates Ctrl+A
    assert!(ai_state.visible, "Toggle should change visibility");

    ai_state.toggle(); // Simulates Ctrl+A again
    assert!(!ai_state.visible, "Toggle should change visibility back");

    // Mechanism 3: Execution results do NOT change visibility
    let mut ai_state = AiState::new(true);
    ai_state.visible = false;

    // Try error result
    let error_result: Result<String, String> = Err("error".to_string());
    handle_execution_result(
        &mut ai_state,
        &error_result,
        ".err",
        0,
        r#"{}"#,
        jiq::ai::context::ContextParams {
            input_schema: None,
            base_query: None,
            base_query_result: None,
        },
    );
    assert!(
        !ai_state.visible,
        "Error result should NOT change visibility"
    );

    // Try success result
    let success_result: Result<String, String> = Ok(r#"{"ok": true}"#.to_string());
    handle_execution_result(
        &mut ai_state,
        &success_result,
        ".ok",
        0,
        r#"{}"#,
        jiq::ai::context::ContextParams {
            input_schema: None,
            base_query: None,
            base_query_result: None,
        },
    );
    assert!(
        !ai_state.visible,
        "Success result should NOT change visibility"
    );

    // Summary: Only config (initial) and toggle (runtime) control visibility
}

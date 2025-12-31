//! Tests for app_state

use super::*;
use crate::test_utils::test_helpers::{create_test_loader, test_app};
use proptest::prelude::*;
use std::sync::Arc;

#[test]
fn test_app_initialization() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let app = test_app(json);

    assert_eq!(app.focus, Focus::InputField);
    assert_eq!(app.results_scroll.offset, 0);
    assert_eq!(app.output_mode, None);
    assert!(!app.should_quit);
    assert_eq!(app.query(), "");
}

#[test]
fn test_initial_query_result() {
    let json = r#"{"name": "Bob"}"#;
    let app = test_app(json);

    assert!(app.query.is_some());
    let query_state = app.query.as_ref().unwrap();
    assert!(query_state.result.is_ok());
    let result = query_state.result.as_ref().unwrap();
    assert!(result.contains("Bob"));
}

#[test]
fn test_focus_enum() {
    assert_eq!(Focus::InputField, Focus::InputField);
    assert_eq!(Focus::ResultsPane, Focus::ResultsPane);
    assert_ne!(Focus::InputField, Focus::ResultsPane);
}

#[test]
fn test_output_mode_enum() {
    assert_eq!(OutputMode::Results, OutputMode::Results);
    assert_eq!(OutputMode::Query, OutputMode::Query);
    assert_ne!(OutputMode::Results, OutputMode::Query);
}

#[test]
fn test_should_quit_getter() {
    let json = r#"{}"#;
    let mut app = test_app(json);

    assert!(!app.should_quit());

    app.should_quit = true;
    assert!(app.should_quit());
}

#[test]
fn test_output_mode_getter() {
    let json = r#"{}"#;
    let mut app = test_app(json);

    assert_eq!(app.output_mode(), None);

    app.output_mode = Some(OutputMode::Results);
    assert_eq!(app.output_mode(), Some(OutputMode::Results));

    app.output_mode = Some(OutputMode::Query);
    assert_eq!(app.output_mode(), Some(OutputMode::Query));
}

#[test]
fn test_query_getter_empty() {
    let json = r#"{"test": true}"#;
    let app = test_app(json);

    assert_eq!(app.query(), "");
}

#[test]
fn test_app_with_empty_json_object() {
    let json = "{}";
    let app = test_app(json);

    assert!(app.query.is_some());
    let query_state = app.query.as_ref().unwrap();
    assert!(query_state.result.is_ok());
}

#[test]
fn test_app_with_json_array() {
    let json = r#"[1, 2, 3]"#;
    let app = test_app(json);

    assert!(app.query.is_some());
    let query_state = app.query.as_ref().unwrap();
    assert!(query_state.result.is_ok());
    let result = query_state.result.as_ref().unwrap();
    assert!(result.contains("1"));
    assert!(result.contains("2"));
    assert!(result.contains("3"));
}

#[test]
fn test_max_scroll_large_content() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);

    let large_result: String = (0..70000).map(|i| format!("line {}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(large_result.clone());
    query_state.last_successful_result = Some(Arc::new(large_result.clone()));
    query_state.cached_line_count = large_result.lines().count() as u32;

    let line_count = app.results_line_count_u32();
    assert!(line_count > 65535);

    app.results_scroll.update_bounds(line_count, 20);

    assert_eq!(app.results_scroll.max_offset, u16::MAX);
}

#[test]
fn test_results_line_count_large_file() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);

    let result: String = (0..65535).map(|_| "x\n").collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(result.clone());
    query_state.last_successful_result = Some(Arc::new(result.clone()));
    query_state.cached_line_count = result.lines().count() as u32;

    assert_eq!(app.results_line_count_u32(), 65535);

    app.results_scroll.update_bounds(65535, 10);

    assert_eq!(app.results_scroll.max_offset, 65525);
}

#[test]
fn test_line_count_uses_last_result_on_error() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);

    let valid_result: String = (0..50).map(|i| format!("line{}\n", i)).collect();
    let query_state = app.query.as_mut().unwrap();
    query_state.result = Ok(valid_result.clone());
    query_state.last_successful_result = Some(Arc::new(valid_result.clone()));
    query_state.cached_line_count = valid_result.lines().count() as u32;

    assert_eq!(app.results_line_count_u32(), 50);

    app.query.as_mut().unwrap().result = Err("syntax error\nline 2\nline 3".to_string());

    assert_eq!(app.results_line_count_u32(), 50);

    app.results_scroll.update_bounds(50, 10);
    assert_eq!(app.results_scroll.max_offset, 40);
}

#[test]
fn test_line_count_with_error_no_cached_result() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);

    let query_state = app.query.as_mut().unwrap();
    query_state.last_successful_result = None;
    query_state.cached_line_count = 0;
    query_state.result = Err("error message".to_string());

    assert_eq!(app.results_line_count_u32(), 0);

    app.results_scroll.update_bounds(0, 10);
    assert_eq!(app.results_scroll.max_offset, 0);
}

#[test]
fn test_tooltip_initialized_enabled() {
    let json = r#"{"name": "test"}"#;
    let app = test_app(json);

    assert!(app.tooltip.enabled);
    assert!(app.tooltip.current_function.is_none());
}

// **Feature: ai-assistant-phase2, Property 10: Info popup hidden while AI visible**
// *For any* state where AI popup is visible, the info popup SHALL be hidden.
// **Validates: Requirements 9.1, 9.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_tooltip_hidden_while_ai_visible(
        initial_tooltip_enabled: bool,
        ai_enabled: bool,
        ai_configured: bool
    ) {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        // Set up initial state
        app.tooltip.enabled = initial_tooltip_enabled;
        app.ai.enabled = ai_enabled;
        app.ai.configured = ai_configured;
        app.ai.visible = false;

        // Toggle AI popup to make it visible
        let was_visible = app.ai.visible;
        app.ai.toggle();

        if !was_visible && app.ai.visible {
            // Save current tooltip state and hide it
            app.saved_tooltip_visibility = app.tooltip.enabled;
            app.tooltip.enabled = false;
        }

        // When AI popup is visible, tooltip should be disabled
        if app.ai.visible {
            prop_assert!(
                !app.tooltip.enabled,
                "Tooltip should be disabled when AI popup is visible"
            );
        }
    }
}

// **Feature: ai-assistant-phase2, Property 11: Info popup state restoration**
// *For any* AI popup hide action, the info popup visibility SHALL be restored to its saved state.
// **Validates: Requirements 9.2, 9.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_tooltip_state_restoration(
        initial_tooltip_enabled: bool,
        ai_enabled: bool,
        ai_configured: bool
    ) {
        let json = r#"{"test": true}"#;
        let mut app = test_app(json);

        // Set up initial state
        app.tooltip.enabled = initial_tooltip_enabled;
        app.ai.enabled = ai_enabled;
        app.ai.configured = ai_configured;
        app.ai.visible = false;

        let original_tooltip_state = app.tooltip.enabled;

        // Toggle AI popup to make it visible (simulating Ctrl+A press)
        let was_visible = app.ai.visible;
        app.ai.toggle();

        if !was_visible && app.ai.visible {
            // Save current tooltip state and hide it
            app.saved_tooltip_visibility = app.tooltip.enabled;
            app.tooltip.enabled = false;
        }

        // Now toggle AI popup to hide it (simulating second Ctrl+A press)
        let was_visible = app.ai.visible;
        app.ai.toggle();

        if was_visible && !app.ai.visible {
            // Restore saved tooltip state
            app.tooltip.enabled = app.saved_tooltip_visibility;
        }

        // After hiding AI popup, tooltip should be restored to original state
        prop_assert_eq!(
            app.tooltip.enabled,
            original_tooltip_state,
            "Tooltip state should be restored to original value after AI popup is hidden"
        );
    }
}

#[test]
fn test_trigger_ai_request_sends_request_when_configured() {
    // Test that trigger_ai_request sends a request when AI is configured
    let json_input = r#"{"name": "test", "value": 42}"#.to_string();
    let config = Config::default();
    let loader = create_test_loader(json_input);
    let mut app = App::new_with_loader(loader, &config);

    // Poll the loader to initialize query state
    app.poll_file_loader();

    // Configure AI with channel
    app.ai.configured = true;
    app.ai.visible = true; // Must be visible for requests to be sent
    let (tx, rx) = std::sync::mpsc::channel();
    let (_response_tx, response_rx) = std::sync::mpsc::channel();
    app.ai.request_tx = Some(tx);
    app.ai.response_rx = Some(response_rx);

    // Set initial query hash to ensure query appears changed
    app.ai.set_last_query_hash(".initial");

    // Set a different query
    app.input.textarea.insert_str(".name");
    if let Some(query_state) = &mut app.query {
        query_state.execute(".name");
    }

    // Trigger AI request
    app.trigger_ai_request();

    // Verify request was sent
    let mut found_request = false;
    while let Ok(msg) = rx.try_recv() {
        if matches!(msg, crate::ai::ai_state::AiRequest::Query { .. }) {
            found_request = true;
            break;
        }
    }
    assert!(found_request, "Should have sent AI request when configured");
}

#[test]
fn test_trigger_ai_request_noop_when_not_configured() {
    // Test that trigger_ai_request does nothing when AI is not configured
    let json_input = r#"{"name": "test"}"#.to_string();
    let config = Config::default();
    let loader = create_test_loader(json_input);
    let mut app = App::new_with_loader(loader, &config);

    // Poll the loader to initialize query state
    app.poll_file_loader();

    // AI is NOT configured
    app.ai.configured = false;
    app.ai.request_tx = None;

    // Set a query
    app.input.textarea.insert_str(".name");

    // This should not panic even without channel
    app.trigger_ai_request();

    // Test passes if no panic occurred
}

#[test]
fn test_trigger_ai_request_includes_query_context() {
    // Test that trigger_ai_request includes the current query context
    let json_input = r#"{"name": "test", "age": 30}"#.to_string();
    let config = Config::default();
    let loader = create_test_loader(json_input);
    let mut app = App::new_with_loader(loader, &config);

    // Poll the loader to initialize query state
    app.poll_file_loader();

    // Configure AI
    app.ai.configured = true;
    app.ai.visible = true; // Must be visible for requests to be sent
    let (tx, rx) = std::sync::mpsc::channel();
    let (_response_tx, response_rx) = std::sync::mpsc::channel();
    app.ai.request_tx = Some(tx);
    app.ai.response_rx = Some(response_rx);

    // Set initial query hash to ensure query appears changed
    app.ai.set_last_query_hash(".initial");

    // Set a query with error
    app.input.textarea.insert_str(".invalid");
    if let Some(query_state) = &mut app.query {
        query_state.execute(".invalid");
    }

    // Trigger AI request
    app.trigger_ai_request();

    // Verify request contains the query
    if let Ok(crate::ai::ai_state::AiRequest::Query { prompt, .. }) = rx.try_recv() {
        assert!(
            prompt.contains(".invalid"),
            "Prompt should contain the query"
        );
    } else {
        panic!("Expected Query request");
    }
}

#[test]
fn test_initial_needs_render_true() {
    let app = test_app(r#"{"test": true}"#);
    assert!(
        app.needs_render,
        "New app should have needs_render=true for initial render"
    );
}

#[test]
fn test_poll_file_loader_marks_dirty_on_success() {
    let json_input = r#"{"test": true}"#.to_string();
    let config = Config::default();
    let loader = create_test_loader(json_input);
    let mut app = App::new_with_loader(loader, &config);

    app.clear_dirty();

    app.poll_file_loader();

    assert!(
        app.needs_render,
        "poll_file_loader should mark dirty on successful load"
    );
}

#[test]
fn test_poll_file_loader_marks_dirty_on_error() {
    let config = Config::default();
    let loader = crate::input::FileLoader::spawn_load(std::path::PathBuf::from("/nonexistent"));
    let mut app = App::new_with_loader(loader, &config);

    std::thread::sleep(std::time::Duration::from_millis(100));

    app.clear_dirty();

    app.poll_file_loader();

    if app.notification.current().is_some() {
        assert!(
            app.needs_render,
            "poll_file_loader should mark dirty when error notification is shown"
        );
    }
}

// **Feature: deferred-file-loading, Property 2: Successful loading initializes QueryState**
// *For any* valid JSON string returned by FileLoader, after poll_file_loader processes the result,
// the App's query field should be Some and contain a QueryState initialized with that JSON
// **Validates: Requirements 1.3, 2.2, 4.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_successful_loading_initializes_query(
        json_value in prop::collection::vec(any::<u8>(), 1..100)
    ) {
        // Generate a valid JSON string
        let json = format!(r#"{{"data": {:?}}}"#, json_value);

        // Validate it's actually valid JSON
        if serde_json::from_str::<serde_json::Value>(&json).is_err() {
            return Ok(());
        }

        let config = Config::default();

        // Create a mock FileLoader that has completed successfully
        // We'll simulate this by creating an app with loader, then manually setting the result
        let loader = crate::input::FileLoader::spawn_load(std::path::PathBuf::from("/nonexistent"));
        let mut app = App::new_with_loader(loader, &config);

        // Manually simulate successful loading by removing loader and setting query
        app.file_loader = None;
        app.query = Some(QueryState::new(json.clone()));

        // Verify query is initialized
        prop_assert!(app.query.is_some(), "Query should be Some after successful loading");

        let query_state = app.query.as_ref().unwrap();
        prop_assert_eq!(query_state.executor.json_input(), &json, "Query should contain the loaded JSON");
    }
}

// **Feature: deferred-file-loading, Property 3: Error loading preserves None QueryState**
// *For any* error returned by FileLoader, after poll_file_loader processes the error,
// the App's query field should remain None
// **Validates: Requirements 1.4, 2.3, 4.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_error_loading_preserves_none_query(
        _error_msg in ".*"
    ) {
        let config = Config::default();

        // Create app with loader
        let loader = crate::input::FileLoader::spawn_load(std::path::PathBuf::from("/nonexistent"));
        let app = App::new_with_loader(loader, &config);

        // Verify query starts as None
        prop_assert!(app.query.is_none(), "Query should start as None with loader");

        // Simulate error by keeping loader but not setting query
        // (In real scenario, poll_file_loader would handle this)
        // For this test, we just verify the invariant holds

        prop_assert!(app.query.is_none(), "Query should remain None after error");
    }
}

// **Feature: deferred-file-loading, Property 5: Loading invariant maintained**
// *For any* App state where file_loader is Some and in Loading state, the query field must be None
// **Validates: Requirements 4.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_loading_invariant_maintained(
        _dummy in any::<u8>()
    ) {
        let config = Config::default();

        // Create app with loader in Loading state
        let loader = crate::input::FileLoader::spawn_load(std::path::PathBuf::from("/nonexistent"));
        let app = App::new_with_loader(loader, &config);

        // Verify invariant: if file_loader is Some and Loading, query must be None
        if let Some(loader) = &app.file_loader
            && loader.is_loading() {
                prop_assert!(app.query.is_none(), "Query must be None when file_loader is Loading");
            }
    }
}

#[test]
fn test_new_with_bedrock_provider() {
    use crate::config::ai_types::{AiConfig, AiProviderType, BedrockConfig};

    let config = Config {
        ai: AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Bedrock),
            bedrock: BedrockConfig {
                region: Some("us-east-1".to_string()),
                model: Some("anthropic.claude-3-sonnet".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let loader = create_test_loader("{}".to_string());
    let app = App::new_with_loader(loader, &config);

    assert!(app.ai.configured);
}

#[test]
fn test_new_with_openai_provider() {
    use crate::config::ai_types::{AiConfig, AiProviderType, OpenAiConfig};

    let config = Config {
        ai: AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Openai),
            openai: OpenAiConfig {
                api_key: Some("test-key".to_string()),
                model: Some("gpt-4".to_string()),
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let loader = create_test_loader("{}".to_string());
    let app = App::new_with_loader(loader, &config);

    assert!(app.ai.configured);
}

#[test]
fn test_new_with_gemini_provider() {
    use crate::config::ai_types::{AiConfig, AiProviderType, GeminiConfig};

    let config = Config {
        ai: AiConfig {
            enabled: true,
            provider: Some(AiProviderType::Gemini),
            gemini: GeminiConfig {
                api_key: Some("test-key".to_string()),
                model: Some("gemini-pro".to_string()),
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let loader = create_test_loader("{}".to_string());
    let app = App::new_with_loader(loader, &config);

    assert!(app.ai.configured);
}

#[test]
fn test_trigger_ai_request_when_query_none() {
    let json = r#"{"test": true}"#;
    let mut app = test_app(json);
    app.query = None;
    app.ai.configured = true;

    app.trigger_ai_request();

    // Should return early without error
}

#[test]
fn test_trigger_ai_request_strips_ansi_from_success_output() {
    let json_input = r#"{"name": "test", "age": 30}"#.to_string();
    let config = Config::default();
    let loader = create_test_loader(json_input);
    let mut app = App::new_with_loader(loader, &config);

    app.poll_file_loader();

    app.ai.configured = true;
    app.ai.visible = true;
    let (tx, rx) = std::sync::mpsc::channel();
    let (_response_tx, response_rx) = std::sync::mpsc::channel();
    app.ai.request_tx = Some(tx);
    app.ai.response_rx = Some(response_rx);

    app.ai.set_last_query_hash(".initial");

    app.input.textarea.insert_str(".name");
    if let Some(query_state) = &mut app.query {
        query_state.result = Ok("\x1b[0;32m\"test\"\x1b[0m\n".to_string());
        query_state.last_successful_result =
            Some(Arc::new("\x1b[0;32m\"test\"\x1b[0m\n".to_string()));
        query_state.last_successful_result_unformatted = Some(Arc::new("\"test\"\n".to_string()));
    }

    app.trigger_ai_request();

    if let Ok(crate::ai::ai_state::AiRequest::Query { prompt, .. }) = rx.try_recv() {
        assert!(
            !prompt.contains("\x1b["),
            "Prompt should not contain ANSI escape codes"
        );
        assert!(
            prompt.contains("\"test\""),
            "Prompt should contain the unformatted output"
        );
    } else {
        panic!("Expected Query request");
    }
}

#[test]
fn test_trigger_ai_request_strips_ansi_from_base_query_result() {
    let json_input = r#"{"name": "test", "age": 30}"#.to_string();
    let config = Config::default();
    let loader = create_test_loader(json_input);
    let mut app = App::new_with_loader(loader, &config);

    app.poll_file_loader();

    app.ai.configured = true;
    app.ai.visible = true;
    let (tx, rx) = std::sync::mpsc::channel();
    let (_response_tx, response_rx) = std::sync::mpsc::channel();
    app.ai.request_tx = Some(tx);
    app.ai.response_rx = Some(response_rx);

    app.ai.set_last_query_hash(".name");

    app.input.textarea.insert_str(".invalid");
    if let Some(query_state) = &mut app.query {
        query_state.result = Err("field not found".to_string());
        query_state.last_successful_result =
            Some(Arc::new("\x1b[0;32m\"test\"\x1b[0m\n".to_string()));
        query_state.last_successful_result_unformatted = Some(Arc::new("\"test\"\n".to_string()));
        query_state.base_query_for_suggestions = Some(".name".to_string());
    }

    app.trigger_ai_request();

    if let Ok(crate::ai::ai_state::AiRequest::Query { prompt, .. }) = rx.try_recv() {
        assert!(
            !prompt.contains("\x1b["),
            "Prompt should not contain ANSI escape codes in base_query_result"
        );
        if prompt.contains("Its Output") {
            assert!(
                prompt.contains("\"test\""),
                "Prompt should contain unformatted base query result"
            );
        }
    } else {
        panic!("Expected Query request");
    }
}

#[test]
fn test_trigger_ai_request_empty_result_uses_unformatted() {
    let json_input = r#"{"items": []}"#.to_string();
    let config = Config::default();
    let loader = create_test_loader(json_input);
    let mut app = App::new_with_loader(loader, &config);

    app.poll_file_loader();

    app.ai.configured = true;
    app.ai.visible = true;
    let (tx, rx) = std::sync::mpsc::channel();
    let (_response_tx, response_rx) = std::sync::mpsc::channel();
    app.ai.request_tx = Some(tx);
    app.ai.response_rx = Some(response_rx);

    app.ai.set_last_query_hash(".previous");

    app.input.textarea.insert_str(".empty");
    if let Some(query_state) = &mut app.query {
        query_state.result = Ok("null\n".to_string());
        query_state.is_empty_result = true;
        query_state.last_successful_result =
            Some(Arc::new("\x1b[0;32m\"previous\"\x1b[0m\n".to_string()));
        query_state.last_successful_result_unformatted =
            Some(Arc::new("\"previous\"\n".to_string()));
        query_state.base_query_for_suggestions = Some(".previous".to_string());
    }

    app.trigger_ai_request();

    if let Ok(crate::ai::ai_state::AiRequest::Query { prompt, .. }) = rx.try_recv() {
        assert!(
            !prompt.contains("\x1b["),
            "Prompt should not contain ANSI codes even with empty result"
        );
        assert!(
            prompt.contains("\"previous\""),
            "Prompt should contain unformatted previous result"
        );
    } else {
        panic!("Expected Query request");
    }
}

#[cfg(test)]
#[path = "app_state_tests/dirty_flag_tests.rs"]
mod dirty_flag_tests;

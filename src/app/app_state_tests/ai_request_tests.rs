//! Tests for `App::trigger_ai_request`: the auto request sent when the
//! query changes, and the context it carries.

use super::*;
use crate::ai::chat::AiPrompt;

/// The final user turn: current query, output or error, and task.
fn last_user_turn(prompt: &AiPrompt) -> &str {
    &prompt
        .messages
        .last()
        .expect("prompt has a user turn")
        .content
}

/// Whether any part of the prompt still carries terminal colour codes.
fn contains_ansi(prompt: &AiPrompt) -> bool {
    prompt.system.contains("\x1b[") || prompt.messages.iter().any(|m| m.content.contains("\x1b["))
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
            last_user_turn(&prompt).contains("## Current Query\n```\n.invalid\n```"),
            "Prompt should contain the query"
        );
    } else {
        panic!("Expected Query request");
    }
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
    app.ai.max_context_length = crate::ai::context::MAX_JSON_SAMPLE_LENGTH;
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
        query_state.last_successful_result_for_context = Some(Arc::new(
            crate::ai::context::prepare_json_for_context("\"test\"\n", app.ai.max_context_length),
        ));
        query_state.base_query_for_suggestions = Some(".name".to_string());
    }

    app.trigger_ai_request();

    if let Ok(crate::ai::ai_state::AiRequest::Query { prompt, .. }) = rx.try_recv() {
        assert!(
            !contains_ansi(&prompt),
            "Prompt should not contain ANSI escape codes"
        );
        assert!(
            last_user_turn(&prompt).contains("\"test\""),
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
            !contains_ansi(&prompt),
            "Prompt should not contain ANSI escape codes in base_query_result"
        );
        let turn = last_user_turn(&prompt);
        if turn.contains("## Last Working Query Output") {
            assert!(
                turn.contains("\"test\""),
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
    app.ai.max_context_length = crate::ai::context::MAX_JSON_SAMPLE_LENGTH;
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
        query_state.last_successful_result_for_context =
            Some(Arc::new(crate::ai::context::prepare_json_for_context(
                "\"previous\"\n",
                app.ai.max_context_length,
            )));
        query_state.base_query_for_suggestions = Some(".previous".to_string());
    }

    app.trigger_ai_request();

    if let Ok(crate::ai::ai_state::AiRequest::Query { prompt, .. }) = rx.try_recv() {
        assert!(
            !contains_ansi(&prompt),
            "Prompt should not contain ANSI codes even with empty result"
        );
        assert!(
            last_user_turn(&prompt).contains("\"previous\""),
            "Prompt should contain unformatted previous result"
        );
    } else {
        panic!("Expected Query request");
    }
}

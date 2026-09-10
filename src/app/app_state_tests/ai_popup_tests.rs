//! Tests for the AI popup's open/close and chat focus behaviour on `App`:
//! `toggle_ai_popup`, `focus_ai_chat`, and `send_ai_chat_question`.

use super::*;
use crate::ai::ai_state::{AiRequest, AiResponse};
use std::sync::mpsc;

/// Install a live request channel and return the worker-side receiver.
fn install_channel(app: &mut App) -> mpsc::Receiver<AiRequest> {
    let (req_tx, req_rx) = mpsc::channel();
    let (_resp_tx, resp_rx) = mpsc::channel::<AiResponse>();
    app.ai.set_channels(req_tx, resp_rx);
    req_rx
}

fn type_question(app: &mut App, text: &str) {
    app.ai.chat_input.insert_str(text);
}

// ---------------------------------------------------------------------------
// toggle_ai_popup
// ---------------------------------------------------------------------------

#[test]
fn test_toggle_opens_popup_and_leaves_focus_in_query_box() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = false;
    app.focus = Focus::InputField;
    app.tooltip.enabled = true;

    app.toggle_ai_popup();

    assert!(app.ai.visible);
    assert_eq!(app.focus, Focus::InputField);
    assert!(
        !app.tooltip.enabled,
        "tooltip hides while the popup is open"
    );
    assert!(
        app.saved_tooltip_visibility,
        "the user's tooltip preference is remembered for restore"
    );
}

#[test]
fn test_toggle_closes_popup_and_returns_focus_to_input() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = false;
    app.tooltip.enabled = true;
    app.toggle_ai_popup();
    assert!(app.focus_ai_chat());
    assert_eq!(app.focus, Focus::AiChat);

    app.toggle_ai_popup();

    assert!(!app.ai.visible);
    assert_eq!(app.focus, Focus::InputField);
    assert!(app.tooltip.enabled, "tooltip preference restored on close");
}

#[test]
fn test_toggle_chat_focus_opens_hidden_popup_and_focuses_chat() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = false;
    app.focus = Focus::InputField;
    app.tooltip.enabled = true;

    app.toggle_ai_chat_focus();

    assert!(app.ai.visible);
    assert_eq!(app.focus, Focus::AiChat);
    assert!(!app.tooltip.enabled);
    assert_eq!(
        app.ai.chat_input.cursor_style(),
        crate::theme::palette::cursor()
    );
}

#[test]
fn test_toggle_chat_focus_on_visible_popup_only_moves_focus() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = true;
    app.focus = Focus::InputField;
    let rx = install_channel(&mut app);

    app.toggle_ai_chat_focus();

    assert!(app.ai.visible);
    assert_eq!(app.focus, Focus::AiChat);
    assert!(
        rx.try_recv().is_err(),
        "no request is triggered by focusing"
    );
}

#[test]
fn test_toggle_chat_focus_from_chat_returns_to_query_box() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = true;
    assert!(app.focus_ai_chat());

    app.toggle_ai_chat_focus();

    assert_eq!(app.focus, Focus::InputField);
    assert!(app.ai.visible, "popup stays open");
}

#[test]
fn test_toggle_close_keeps_input_focus_when_chat_not_focused() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = true;
    app.focus = Focus::InputField;

    app.toggle_ai_popup();

    assert!(!app.ai.visible);
    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn test_toggle_open_from_results_pane_keeps_popup_and_tooltip_state_for_input_return() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = true;
    app.tooltip.enabled = true;
    app.focus_results_pane();
    assert!(!app.ai.visible, "results pane hides the popup");

    app.toggle_ai_popup();

    assert!(app.ai.visible);
    assert_eq!(app.focus, Focus::ResultsPane, "Ctrl+A does not move focus");
    assert!(
        app.saved_ai_visibility_for_results,
        "returning to the input field later keeps the popup open"
    );
    assert!(
        app.saved_tooltip_visibility,
        "tooltip preference comes from before the results pane hid it"
    );

    // Esc-style return to the query box keeps the popup open.
    app.focus_input_field();
    assert_eq!(app.focus, Focus::InputField);
    assert!(app.ai.visible);
    assert!(
        !app.tooltip.enabled,
        "tooltip stays hidden while popup is open"
    );
}

#[test]
fn test_toggle_open_triggers_request_when_configured() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = false;
    app.ai.configured = true;
    let rx = install_channel(&mut app);
    app.ai.set_last_query_hash(".stale");

    app.toggle_ai_popup();

    assert!(
        matches!(rx.try_recv(), Ok(AiRequest::Query { .. })),
        "opening the popup sends the current query for suggestions"
    );
}

#[test]
fn test_toggle_open_sends_nothing_when_not_configured() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = false;
    app.ai.configured = false;
    let rx = install_channel(&mut app);

    app.toggle_ai_popup();

    assert!(app.ai.visible);
    assert_eq!(app.focus, Focus::InputField);
    assert!(rx.try_recv().is_err());
}

#[test]
fn test_toggle_close_sends_nothing() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = true;
    app.ai.configured = true;
    let rx = install_channel(&mut app);

    app.toggle_ai_popup();

    assert!(!app.ai.visible);
    assert!(rx.try_recv().is_err());
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
        let mut app = test_app(r#"{"test": true}"#);
        app.tooltip.enabled = initial_tooltip_enabled;
        app.ai.enabled = ai_enabled;
        app.ai.configured = ai_configured;
        app.ai.visible = false;

        app.toggle_ai_popup();

        prop_assert!(app.ai.visible);
        prop_assert_eq!(app.focus, Focus::InputField);
        prop_assert!(
            !app.tooltip.enabled,
            "Tooltip should be disabled when AI popup is visible"
        );
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
        let mut app = test_app(r#"{"test": true}"#);
        app.tooltip.enabled = initial_tooltip_enabled;
        app.ai.enabled = ai_enabled;
        app.ai.configured = ai_configured;
        app.ai.visible = false;

        app.toggle_ai_popup();
        app.toggle_ai_popup();

        prop_assert!(!app.ai.visible);
        prop_assert_eq!(app.focus, Focus::InputField);
        prop_assert_eq!(
            app.tooltip.enabled,
            initial_tooltip_enabled,
            "Tooltip state should be restored to original value after AI popup is hidden"
        );
    }
}

// ---------------------------------------------------------------------------
// focus_ai_chat / focus_input_field
// ---------------------------------------------------------------------------

#[test]
fn test_focus_ai_chat_returns_false_when_popup_hidden() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = false;
    app.focus = Focus::InputField;

    assert!(!app.focus_ai_chat());
    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn test_focus_ai_chat_moves_focus_and_hides_autocomplete() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = true;
    app.focus = Focus::InputField;
    app.autocomplete
        .update_suggestions(vec![crate::autocomplete::Suggestion::new(
            "a",
            crate::autocomplete::SuggestionType::Field,
        )]);
    assert!(app.autocomplete.is_visible());

    assert!(app.focus_ai_chat());

    assert_eq!(app.focus, Focus::AiChat);
    assert!(!app.autocomplete.is_visible());
}

#[test]
fn test_focus_ai_chat_is_idempotent() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = true;

    assert!(app.focus_ai_chat());
    assert!(app.focus_ai_chat());
    assert_eq!(app.focus, Focus::AiChat);
}

#[test]
fn test_focus_input_field_from_chat_leaves_popup_open() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = true;
    app.focus_ai_chat();

    app.focus_input_field();

    assert_eq!(app.focus, Focus::InputField);
    assert!(app.ai.visible);
}

#[test]
fn test_focus_results_pane_from_chat_hides_popup() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = true;
    app.focus_ai_chat();

    app.focus_results_pane();

    assert_eq!(app.focus, Focus::ResultsPane);
    assert!(!app.ai.visible);
    assert!(app.saved_ai_visibility_for_results);
}

// ---------------------------------------------------------------------------
// send_ai_chat_question
// ---------------------------------------------------------------------------

#[test]
fn test_send_chat_question_returns_false_with_blank_input() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = true;
    app.ai.configured = true;
    let rx = install_channel(&mut app);
    type_question(&mut app, "   ");

    assert!(!app.send_ai_chat_question());
    assert!(rx.try_recv().is_err());
    assert!(app.ai.current_question.is_none());
}

#[test]
fn test_send_chat_question_returns_false_when_not_configured() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = true;
    app.ai.configured = false;
    let rx = install_channel(&mut app);
    type_question(&mut app, "why?");

    assert!(!app.send_ai_chat_question());
    assert!(rx.try_recv().is_err());
    assert!(app.ai.current_question.is_none());
}

#[test]
fn test_send_chat_question_returns_false_when_popup_hidden() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = false;
    app.ai.configured = true;
    let rx = install_channel(&mut app);
    type_question(&mut app, "why?");

    assert!(!app.send_ai_chat_question());
    assert!(rx.try_recv().is_err());
}

#[test]
fn test_send_chat_question_returns_false_without_query_state() {
    let mut app = test_app(r#"{"a": 1}"#);
    app.ai.visible = true;
    app.ai.configured = true;
    let rx = install_channel(&mut app);
    app.query = None;
    type_question(&mut app, "why?");

    assert!(!app.send_ai_chat_question());
    assert!(rx.try_recv().is_err());
}

#[test]
fn test_send_chat_question_sends_request_and_clears_input() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.ai.visible = true;
    app.ai.configured = true;
    let rx = install_channel(&mut app);
    app.input.textarea.insert_str(".name");
    if let Some(query_state) = &mut app.query {
        query_state.execute(".name");
    }
    type_question(&mut app, "why is this a string?");

    assert!(app.send_ai_chat_question());

    let AiRequest::Query { prompt, .. } = rx.try_recv().expect("request on channel");
    let turn = &prompt.messages.last().unwrap().content;
    assert!(turn.contains("## Question\nwhy is this a string?"));
    assert!(turn.contains("## Current Query\n```\n.name\n```"));
    assert_eq!(
        app.ai.current_question.as_deref(),
        Some("why is this a string?")
    );
    assert_eq!(app.ai.current_query, ".name");
    assert_eq!(app.ai.chat_input_text(), "", "input is emptied once sent");
    assert!(app.ai.loading);
}

#[test]
fn test_send_chat_question_includes_error_context() {
    let mut app = test_app(r#"{"name": "test"}"#);
    app.ai.visible = true;
    app.ai.configured = true;
    let rx = install_channel(&mut app);
    app.input.textarea.insert_str(".[");
    if let Some(query_state) = &mut app.query {
        query_state.result = Err("jq: error: syntax error".to_string());
    }
    type_question(&mut app, "what is wrong?");

    assert!(app.send_ai_chat_question());

    let AiRequest::Query { prompt, .. } = rx.try_recv().expect("request on channel");
    let turn = &prompt.messages.last().unwrap().content;
    assert!(turn.contains("## Error\n```\njq: error: syntax error"));
}

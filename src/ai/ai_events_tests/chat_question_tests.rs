//! Tests for `send_chat_question`: the request built when the user types a
//! question in the AI popup's chat input.

use super::*;
use crate::ai::chat::{ChatExchange, ChatRole};
use crate::ai::context::ContextParams;
use crate::ai::suggestion::{Suggestion, SuggestionType};

fn empty_params() -> ContextParams<'static> {
    ContextParams {
        input_schema: None,
        base_query: None,
        base_query_result: None,
        is_empty_result: false,
    }
}

/// A visible, enabled state wired to a request channel.
fn visible_state_with_channel() -> (AiState, mpsc::Receiver<AiRequest>) {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = true;
    let (tx, rx) = mpsc::channel();
    ai_state.request_tx = Some(tx);
    (ai_state, rx)
}

fn suggestion(query: &str, kind: SuggestionType, description: &str) -> Suggestion {
    Suggestion {
        query: query.to_string(),
        description: description.to_string(),
        suggestion_type: kind,
    }
}

fn exchange(question: &str, query: &str, raw: &str) -> ChatExchange {
    ChatExchange {
        question: question.to_string(),
        query: query.to_string(),
        raw_response: raw.to_string(),
        answer: Some("answer".to_string()),
        suggestions: Vec::new(),
    }
}

fn recv_prompt(rx: &mpsc::Receiver<AiRequest>) -> AiPrompt {
    let AiRequest::Query { prompt, .. } = rx.try_recv().expect("a request was sent");
    prompt
}

#[test]
fn sends_question_with_current_query_context() {
    let (mut ai_state, rx) = visible_state_with_channel();
    let result: Result<String, String> = Ok(r#"{"name":"x"}"#.to_string());

    let sent = send_chat_question(
        &mut ai_state,
        "why is name a string?",
        &result,
        ".user",
        5,
        empty_params(),
    );

    assert!(sent);
    let prompt = recv_prompt(&rx);
    let turn = last_user_turn(&prompt);
    assert!(
        turn.contains("## Question\nwhy is name a string?"),
        "question section missing: {turn}"
    );
    assert!(turn.contains("## Current Query\n```\n.user\n```"));
    assert!(turn.contains("Cursor position: 5"));
    assert!(turn.contains("## Current Query Output"));
    assert!(turn.contains(r#"{"name":"x"}"#));
    assert!(turn.contains("## Task"));
    assert!(
        turn.contains("Answer the question in `answer`"),
        "chat task asks for a prose answer"
    );
}

#[test]
fn records_question_and_query_on_state() {
    let (mut ai_state, _rx) = visible_state_with_channel();
    let result: Result<String, String> = Ok("1".to_string());

    send_chat_question(&mut ai_state, "  what?  ", &result, ".a", 2, empty_params());

    assert_eq!(
        ai_state.current_question.as_deref(),
        Some("what?"),
        "question is stored trimmed"
    );
    assert_eq!(ai_state.current_query, ".a");
    assert!(ai_state.loading);
    assert!(ai_state.has_in_flight_request());
}

#[test]
fn includes_error_context_when_query_failed() {
    let (mut ai_state, rx) = visible_state_with_channel();
    let result: Result<String, String> = Err("jq: error: syntax error".to_string());

    send_chat_question(
        &mut ai_state,
        "what went wrong?",
        &result,
        ".[",
        2,
        empty_params(),
    );

    let turn = last_user_turn(&recv_prompt(&rx)).to_string();
    assert!(turn.contains("## Error\n```\njq: error: syntax error"));
    assert!(turn.contains("## Question\nwhat went wrong?"));
}

#[test]
fn lists_suggestions_currently_shown() {
    let (mut ai_state, rx) = visible_state_with_channel();
    ai_state.suggestions = vec![
        suggestion(".a | length", SuggestionType::Fix, "Count items"),
        suggestion(".a[]", SuggestionType::Optimize, "Iterate"),
    ];
    let result: Result<String, String> = Ok("[]".to_string());

    send_chat_question(&mut ai_state, "explain 2", &result, ".a", 2, empty_params());

    let turn = last_user_turn(&recv_prompt(&rx)).to_string();
    assert!(turn.contains("## Suggestions Currently Shown to the User"));
    assert!(turn.contains("1. [Fix] `.a | length` - Count items"));
    assert!(turn.contains("2. [Optimize] `.a[]` - Iterate"));
    let list_pos = turn.find("## Suggestions Currently Shown").unwrap();
    let question_pos = turn.find("## Question").unwrap();
    assert!(
        list_pos < question_pos,
        "the shown suggestions precede the question they refer to"
    );
}

#[test]
fn omits_suggestions_section_when_none_shown() {
    let (mut ai_state, rx) = visible_state_with_channel();
    let result: Result<String, String> = Ok("[]".to_string());

    send_chat_question(&mut ai_state, "hi", &result, ".", 1, empty_params());

    let turn = last_user_turn(&recv_prompt(&rx)).to_string();
    assert!(!turn.contains("## Suggestions Currently Shown"));
}

#[test]
fn returns_false_when_popup_hidden() {
    let (mut ai_state, rx) = visible_state_with_channel();
    ai_state.visible = false;
    let result: Result<String, String> = Ok("1".to_string());

    let sent = send_chat_question(&mut ai_state, "why?", &result, ".a", 2, empty_params());

    assert!(!sent);
    assert!(rx.try_recv().is_err(), "nothing sent while hidden");
    assert!(ai_state.current_question.is_none());
    assert!(!ai_state.loading);
}

#[test]
fn returns_false_for_blank_question() {
    let (mut ai_state, rx) = visible_state_with_channel();
    let result: Result<String, String> = Ok("1".to_string());

    for blank in ["", "   ", "\t\n"] {
        let sent = send_chat_question(&mut ai_state, blank, &result, ".a", 2, empty_params());
        assert!(!sent, "blank question {blank:?} must not send");
    }
    assert!(rx.try_recv().is_err());
    assert!(ai_state.current_question.is_none());
}

#[test]
fn returns_false_without_request_channel() {
    let mut ai_state = AiState::new(true);
    ai_state.enabled = true;
    ai_state.visible = true;
    let result: Result<String, String> = Ok("1".to_string());

    let sent = send_chat_question(&mut ai_state, "why?", &result, ".a", 2, empty_params());

    assert!(!sent);
    assert!(ai_state.current_question.is_none());
}

#[test]
fn does_not_require_query_change() {
    // Unlike auto requests, a typed question always goes out, even when the
    // query hash matches the last request.
    let (mut ai_state, rx) = visible_state_with_channel();
    ai_state.set_last_query_hash(".a");
    let result: Result<String, String> = Ok("1".to_string());

    assert!(send_chat_question(
        &mut ai_state,
        "first",
        &result,
        ".a",
        2,
        empty_params()
    ));
    assert!(rx.try_recv().is_ok());
    assert!(send_chat_question(
        &mut ai_state,
        "second",
        &result,
        ".a",
        2,
        empty_params()
    ));
    assert!(rx.try_recv().is_ok());
}

#[test]
fn cancels_in_flight_request_before_sending() {
    let (mut ai_state, rx) = visible_state_with_channel();
    let result: Result<String, String> = Ok("1".to_string());
    let error: Result<String, String> = Err("boom".to_string());

    handle_execution_result(&mut ai_state, &error, ".old", 4, empty_params());
    let first_token = ai_state
        .current_cancel_token
        .clone()
        .expect("auto request holds a token");
    rx.try_recv().expect("auto request sent");

    assert!(send_chat_question(
        &mut ai_state,
        "why?",
        &result,
        ".old",
        4,
        empty_params()
    ));

    assert!(first_token.is_cancelled());
    let AiRequest::Query { request_id, .. } = rx.try_recv().expect("chat request sent");
    assert_eq!(request_id, ai_state.current_request_id());
}

#[test]
fn replays_history_as_alternating_turns() {
    let (mut ai_state, rx) = visible_state_with_channel();
    ai_state.history = vec![
        exchange("q1", ".a", r#"{"answer":"a1","suggestions":[]}"#),
        exchange("q2", "", r#"{"answer":"a2","suggestions":[]}"#),
    ];
    let result: Result<String, String> = Ok("1".to_string());

    send_chat_question(&mut ai_state, "q3", &result, ".b", 2, empty_params());

    let prompt = recv_prompt(&rx);
    let roles: Vec<ChatRole> = prompt.messages.iter().map(|m| m.role).collect();
    assert_eq!(
        roles,
        vec![
            ChatRole::User,
            ChatRole::Assistant,
            ChatRole::User,
            ChatRole::Assistant,
            ChatRole::User,
        ]
    );
    assert_eq!(
        prompt.messages[0].content, "Query at the time: `.a`\n\nq1",
        "replayed user turn carries the query it referred to"
    );
    assert_eq!(
        prompt.messages[1].content,
        r#"{"answer":"a1","suggestions":[]}"#
    );
    assert_eq!(
        prompt.messages[2].content, "q2",
        "an exchange without a query replays the bare question"
    );
    assert_eq!(
        prompt.messages[3].content,
        r#"{"answer":"a2","suggestions":[]}"#
    );
    assert!(prompt.messages[4].content.contains("## Question\nq3"));
}

#[test]
fn history_is_also_replayed_on_auto_requests() {
    let (mut ai_state, rx) = visible_state_with_channel();
    ai_state.history = vec![exchange("q1", ".a", r#"{"answer":"a1","suggestions":[]}"#)];
    let result: Result<String, String> = Ok("1".to_string());

    handle_execution_result(&mut ai_state, &result, ".b", 2, empty_params());

    let prompt = recv_prompt(&rx);
    assert_eq!(prompt.messages.len(), 3);
    assert_eq!(prompt.messages[0].role, ChatRole::User);
    assert_eq!(prompt.messages[1].role, ChatRole::Assistant);
    assert_eq!(prompt.messages[2].role, ChatRole::User);
    assert!(prompt.messages[2].content.contains("## Task"));
    assert!(!prompt.messages[2].content.contains("## Question"));
}

#[test]
fn completed_answer_is_archived_before_next_question() {
    let (mut ai_state, rx) = visible_state_with_channel();
    let result: Result<String, String> = Ok("1".to_string());

    send_chat_question(&mut ai_state, "q1", &result, ".a", 2, empty_params());
    rx.try_recv().unwrap();
    ai_state.append_chunk(r#"{"answer":"a1","suggestions":[]}"#);
    ai_state.complete_request();
    assert_eq!(ai_state.answer.as_deref(), Some("a1"));

    send_chat_question(&mut ai_state, "q2", &result, ".a", 2, empty_params());

    assert_eq!(ai_state.history.len(), 1);
    assert_eq!(ai_state.history[0].question, "q1");
    assert_eq!(ai_state.current_question.as_deref(), Some("q2"));
    assert!(
        ai_state.answer.is_none(),
        "answer slot cleared for the new request"
    );
    let prompt = recv_prompt(&rx);
    assert_eq!(prompt.messages.len(), 3, "archived exchange is replayed");
    let roles: Vec<ChatRole> = prompt.messages.iter().map(|m| m.role).collect();
    assert_eq!(
        roles,
        vec![ChatRole::User, ChatRole::Assistant, ChatRole::User]
    );
    assert_eq!(
        prompt.messages[1].content, r#"{"answer":"a1","suggestions":[]}"#,
        "the assistant turn replays the raw first response"
    );
    assert!(prompt.messages[2].content.contains("## Question\nq2"));
}

#[test]
fn system_prompt_carries_schema_and_extra_instructions() {
    let (mut ai_state, rx) = visible_state_with_channel();
    ai_state.extra_instructions = Some("Prefer short queries.".to_string());
    let result: Result<String, String> = Ok("1".to_string());

    send_chat_question(
        &mut ai_state,
        "how?",
        &result,
        ".a",
        2,
        ContextParams {
            input_schema: Some(r#"{"a":"number"}"#),
            base_query: None,
            base_query_result: None,
            is_empty_result: false,
        },
    );

    let prompt = recv_prompt(&rx);
    assert!(prompt.system.contains("## Input JSON Schema"));
    assert!(prompt.system.contains(r#"{"a":"number"}"#));
    assert!(prompt.system.contains("## Additional User Preferences"));
    assert!(prompt.system.contains("Prefer short queries."));
    let turn = last_user_turn(&prompt);
    assert!(!turn.contains("## Input JSON Schema"));
}

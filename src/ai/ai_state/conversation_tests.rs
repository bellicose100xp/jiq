use std::sync::mpsc;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::ai::ai_state::{AiRequest, AiState};
use crate::ai::chat::{AiPrompt, ChatRole};
use crate::ai::prompt::MAX_HISTORY_EXCHANGES;
use crate::ai::suggestion::{Suggestion, SuggestionType};

fn state_with_channel() -> (AiState, mpsc::Receiver<AiRequest>) {
    let mut state = AiState::new(true);
    state.configured = true;
    state.visible = true;
    let (req_tx, req_rx) = mpsc::channel();
    let (_resp_tx, resp_rx) = mpsc::channel();
    state.set_channels(req_tx, resp_rx);
    (state, req_rx)
}

fn suggestion(query: &str) -> Suggestion {
    Suggestion {
        query: query.to_string(),
        description: "desc".to_string(),
        suggestion_type: SuggestionType::Query,
    }
}

fn complete_with(state: &mut AiState, raw: &str) {
    state.append_chunk(raw);
    state.complete_request();
}

const ANSWER_RESPONSE: &str = r#"{"answer": "Because select() drops them.", "suggestions": [{"type": "query", "query": ".users[] | .name", "details": "Names only"}]}"#;

#[test]
fn send_chat_request_records_question_and_query() {
    let (mut state, rx) = state_with_channel();

    let sent = state.send_chat_request(
        AiPrompt::single("sys", "why?"),
        "why?".to_string(),
        ".users".to_string(),
    );

    assert!(sent);
    assert_eq!(state.current_question.as_deref(), Some("why?"));
    assert_eq!(state.current_query, ".users");
    assert!(state.loading);
    assert!(
        rx.try_recv().is_ok(),
        "the request reaches the worker channel"
    );
}

#[test]
fn send_chat_request_without_channel_records_nothing() {
    let mut state = AiState::new(true);
    let sent = state.send_chat_request(
        AiPrompt::single("sys", "why?"),
        "why?".to_string(),
        ".users".to_string(),
    );
    assert!(!sent);
    assert!(state.current_question.is_none());
}

#[test]
fn completed_chat_answer_is_parsed_into_answer_and_suggestions() {
    let (mut state, _rx) = state_with_channel();
    state.send_chat_request(AiPrompt::single("s", "q"), "q".into(), ".".into());
    complete_with(&mut state, ANSWER_RESPONSE);

    assert_eq!(
        state.answer.as_deref(),
        Some("Because select() drops them.")
    );
    assert_eq!(state.suggestions.len(), 1);
    assert_eq!(state.suggestions[0].query, ".users[] | .name");
    assert!(!state.no_suggestions);
    assert!(!state.parse_failed);
}

#[test]
fn next_request_archives_completed_exchange() {
    let (mut state, _rx) = state_with_channel();
    state.send_chat_request(AiPrompt::single("s", "q1"), "q1".into(), ".a".into());
    complete_with(&mut state, ANSWER_RESPONSE);

    state.send_chat_request(AiPrompt::single("s", "q2"), "q2".into(), ".b".into());

    assert_eq!(state.history.len(), 1);
    let archived = &state.history[0];
    assert_eq!(archived.question, "q1");
    assert_eq!(archived.query, ".a");
    assert_eq!(archived.raw_response, ANSWER_RESPONSE);
    assert_eq!(
        archived.answer.as_deref(),
        Some("Because select() drops them.")
    );
    assert_eq!(archived.suggestions.len(), 1);
    assert_eq!(state.current_question.as_deref(), Some("q2"));
    assert!(
        state.answer.is_none(),
        "the new request starts with no answer"
    );
}

#[test]
fn query_change_archives_chat_exchange_instead_of_dropping_it() {
    let (mut state, _rx) = state_with_channel();
    state.send_chat_request(AiPrompt::single("s", "q1"), "q1".into(), ".a".into());
    complete_with(&mut state, ANSWER_RESPONSE);

    state.clear_stale_response();

    assert_eq!(state.history.len(), 1);
    assert!(state.current_question.is_none());
    assert!(state.answer.is_none());
    assert!(state.response.is_empty());
}

#[test]
fn in_flight_or_failed_exchange_is_not_archived() {
    let (mut state, _rx) = state_with_channel();
    state.send_chat_request(AiPrompt::single("s", "q1"), "q1".into(), ".a".into());
    // Still loading: superseded before any response arrived
    state.send_chat_request(AiPrompt::single("s", "q2"), "q2".into(), ".a".into());
    assert!(state.history.is_empty());

    // Errored response
    state.set_error("boom".to_string());
    state.send_chat_request(AiPrompt::single("s", "q3"), "q3".into(), ".a".into());
    assert!(state.history.is_empty());

    // Unparseable response has neither answer nor suggestions
    complete_with(&mut state, "not json at all");
    assert!(state.parse_failed);
    state.send_chat_request(AiPrompt::single("s", "q4"), "q4".into(), ".a".into());
    assert!(state.history.is_empty());
}

#[test]
fn history_is_capped_at_max_exchanges() {
    let (mut state, _rx) = state_with_channel();
    for i in 0..(MAX_HISTORY_EXCHANGES + 3) {
        state.send_chat_request(
            AiPrompt::single("s", "q"),
            format!("q{}", i),
            ".".to_string(),
        );
        complete_with(&mut state, ANSWER_RESPONSE);
    }
    state.archive_current_exchange();

    assert_eq!(state.history.len(), MAX_HISTORY_EXCHANGES);
    assert_eq!(
        state.history[0].question, "q3",
        "oldest exchanges are dropped first"
    );
    assert_eq!(
        state.history.last().unwrap().question,
        format!("q{}", MAX_HISTORY_EXCHANGES + 2)
    );
}

#[test]
fn auto_suggestions_are_not_archived_as_exchanges() {
    let (mut state, _rx) = state_with_channel();
    state.send_request(AiPrompt::single("s", "auto"));
    complete_with(
        &mut state,
        r#"{"suggestions": [{"type": "fix", "query": ".a", "details": "d"}]}"#,
    );
    assert!(state.current_question.is_none());

    state.send_request(AiPrompt::single("s", "auto again"));
    assert!(state.history.is_empty());
}

#[test]
fn clear_conversation_drops_history_and_chat_answer_on_screen() {
    let (mut state, _rx) = state_with_channel();
    state.send_chat_request(AiPrompt::single("s", "q1"), "q1".into(), ".a".into());
    complete_with(&mut state, ANSWER_RESPONSE);
    state.send_chat_request(AiPrompt::single("s", "q2"), "q2".into(), ".a".into());
    complete_with(&mut state, ANSWER_RESPONSE);
    assert_eq!(state.history.len(), 1);

    state.clear_conversation();

    assert!(state.history.is_empty());
    assert!(state.current_question.is_none());
    assert!(state.answer.is_none());
    assert!(state.suggestions.is_empty());
    assert!(state.response.is_empty());
}

#[test]
fn clear_conversation_keeps_auto_suggestions() {
    let (mut state, _rx) = state_with_channel();
    state.history.push(crate::ai::chat::ChatExchange {
        question: "old".into(),
        query: ".".into(),
        raw_response: "{}".into(),
        answer: Some("a".into()),
        suggestions: vec![],
    });
    state.suggestions = vec![suggestion(".auto")];

    state.clear_conversation();

    assert!(state.history.is_empty());
    assert_eq!(state.suggestions.len(), 1, "query-change suggestions stay");
}

#[test]
fn clear_conversation_while_loading_keeps_in_flight_request_alive() {
    let (mut state, _rx) = state_with_channel();
    state.send_chat_request(AiPrompt::single("s", "q1"), "q1".into(), ".a".into());
    state.append_chunk("partial");

    state.clear_conversation();

    assert!(state.loading, "the in-flight request is not cancelled");
    assert_eq!(state.response, "partial");
    assert!(state.current_question.is_none());
}

#[test]
fn chat_input_take_trims_and_clears() {
    let mut state = AiState::new(true);
    for ch in "  hello there  ".chars() {
        state
            .chat_input
            .input(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
    }
    assert_eq!(state.chat_input_text(), "  hello there  ");

    assert_eq!(state.take_chat_input().as_deref(), Some("hello there"));
    assert_eq!(state.chat_input_text(), "");
}

#[test]
fn chat_input_take_returns_none_for_blank_input() {
    let mut state = AiState::new(true);
    assert!(state.take_chat_input().is_none());
    state
        .chat_input
        .input(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
    assert!(state.take_chat_input().is_none());
    assert_eq!(state.chat_input_text(), "");
}

#[test]
fn set_chat_focused_toggles_cursor_style() {
    let mut state = AiState::new(true);
    state.set_chat_focused(true);
    assert_eq!(
        state.chat_input.cursor_style(),
        crate::theme::palette::cursor()
    );
    state.set_chat_focused(false);
    assert_eq!(
        state.chat_input.cursor_style(),
        ratatui::style::Style::default()
    );
}

#[test]
fn archived_exchange_replays_as_user_then_assistant_turns() {
    let (mut state, _rx) = state_with_channel();
    state.send_chat_request(AiPrompt::single("s", "q1"), "q1".into(), ".a".into());
    complete_with(&mut state, ANSWER_RESPONSE);
    state.archive_current_exchange();

    let turn = state.history[0].user_turn();
    assert!(turn.contains(".a"));
    assert!(turn.contains("q1"));

    let prompt = crate::ai::prompt::build_prompt(&crate::ai::prompt::PromptInputs {
        context: &crate::ai::context::QueryContext::new(
            ".b".into(),
            2,
            Some("[]".into()),
            None,
            crate::ai::context::ContextParams {
                input_schema: None,
                base_query: None,
                base_query_result: None,
                is_empty_result: false,
            },
            1000,
        ),
        extra_instructions: None,
        history: &state.history,
        question: Some("and now?"),
        displayed_suggestions: &[],
    });
    let roles: Vec<ChatRole> = prompt.messages.iter().map(|m| m.role).collect();
    assert_eq!(
        roles,
        vec![ChatRole::User, ChatRole::Assistant, ChatRole::User]
    );
    assert_eq!(prompt.messages[1].content, ANSWER_RESPONSE);
}

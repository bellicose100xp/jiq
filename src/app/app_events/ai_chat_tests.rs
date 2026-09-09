use std::sync::mpsc;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use super::*;
use crate::ai::ai_state::AiRequest;
use crate::ai::chat::ChatRole;
use crate::ai::suggestion::{Suggestion, SuggestionType};
use crate::app::app_state::Focus;
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};

/// An app with the AI popup open, configured, and the chat input focused.
/// Returns the request receiver so tests can inspect what was sent.
fn chat_app() -> (App, mpsc::Receiver<AiRequest>) {
    let mut app = app_with_query(".name");
    app.ai.configured = true;
    app.ai.enabled = true;
    let (req_tx, req_rx) = mpsc::channel();
    let (_resp_tx, resp_rx) = mpsc::channel();
    app.ai.set_channels(req_tx, resp_rx);
    app.ai.visible = true;
    assert!(app.focus_ai_chat());
    (app, req_rx)
}

fn type_text(app: &mut App, text: &str) {
    for ch in text.chars() {
        app.handle_key_event(key(KeyCode::Char(ch)));
    }
}

#[test]
fn typed_characters_go_to_chat_input_not_query() {
    let (mut app, _rx) = chat_app();
    type_text(&mut app, "why empty?");
    assert_eq!(app.ai.chat_input_text(), "why empty?");
    assert_eq!(app.query(), ".name", "query box is untouched");
    assert!(!app.should_quit);
}

#[test]
fn q_and_question_mark_are_typed_not_quit_or_help() {
    let (mut app, _rx) = chat_app();
    type_text(&mut app, "q?");
    assert_eq!(app.ai.chat_input_text(), "q?");
    assert!(!app.should_quit);
    assert!(!app.help.visible);
}

#[test]
fn enter_sends_question_with_history_and_clears_input() {
    let (mut app, rx) = chat_app();
    type_text(&mut app, "why empty?");
    app.handle_key_event(key(KeyCode::Enter));

    assert!(!app.should_quit, "Enter in chat must not exit the app");
    assert_eq!(app.ai.chat_input_text(), "");
    assert_eq!(app.ai.current_question.as_deref(), Some("why empty?"));
    assert!(app.ai.loading);

    let AiRequest::Query { prompt, .. } = rx.try_recv().expect("request sent");
    let last = prompt.messages.last().unwrap();
    assert_eq!(last.role, ChatRole::User);
    assert!(last.content.contains("## Question\nwhy empty?"));
    assert!(
        last.content.contains(".name"),
        "current query is in the context"
    );
    assert!(prompt.system.contains("jq query assistant"));
}

#[test]
fn enter_on_blank_input_sends_nothing() {
    let (mut app, rx) = chat_app();
    app.handle_key_event(key(KeyCode::Enter));
    assert!(rx.try_recv().is_err());
    assert!(!app.ai.loading);
    assert!(!app.should_quit);
}

#[test]
fn esc_returns_focus_to_query_and_keeps_popup_open() {
    let (mut app, _rx) = chat_app();
    app.handle_key_event(key(KeyCode::Esc));
    assert_eq!(app.focus, Focus::InputField);
    assert!(app.ai.visible);
    assert_eq!(
        app.ai.chat_input.cursor_style(),
        ratatui::style::Style::default(),
        "chat cursor hidden once unfocused"
    );
}

#[test]
fn ctrl_a_closes_popup_and_returns_focus() {
    let (mut app, _rx) = chat_app();
    app.handle_key_event(key_with_mods(KeyCode::Char('a'), KeyModifiers::CONTROL));
    assert!(!app.ai.visible);
    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn ctrl_a_from_query_box_opens_popup_with_chat_focused() {
    let mut app = app_with_query(".name");
    app.ai.configured = true;
    app.ai.visible = false;
    app.focus = Focus::InputField;

    app.handle_key_event(key_with_mods(KeyCode::Char('a'), KeyModifiers::CONTROL));

    assert!(app.ai.visible);
    assert_eq!(app.focus, Focus::AiChat);
    assert_eq!(
        app.ai.chat_input.cursor_style(),
        crate::theme::palette::cursor()
    );
}

#[test]
fn ctrl_a_from_results_pane_keeps_popup_open_when_returning_to_input() {
    let mut app = app_with_query(".name");
    app.ai.configured = true;
    app.ai.visible = true;
    app.focus_results_pane();
    assert!(!app.ai.visible);

    app.handle_key_event(key_with_mods(KeyCode::Char('a'), KeyModifiers::CONTROL));
    assert!(app.ai.visible);
    assert_eq!(app.focus, Focus::AiChat);

    app.handle_key_event(key(KeyCode::Esc));
    assert_eq!(app.focus, Focus::InputField);
    assert!(app.ai.visible, "popup stays open after Esc");
}

#[test]
fn ctrl_l_clears_conversation() {
    let (mut app, _rx) = chat_app();
    app.ai.history.push(crate::ai::chat::ChatExchange {
        question: "old".into(),
        query: ".".into(),
        raw_response: "{}".into(),
        answer: Some("a".into()),
        suggestions: vec![],
    });
    app.handle_key_event(key_with_mods(KeyCode::Char('l'), KeyModifiers::CONTROL));
    assert!(app.ai.history.is_empty());
    assert_eq!(app.focus, Focus::AiChat);
}

#[test]
fn alt_digit_applies_suggestion_while_chat_focused() {
    let (mut app, _rx) = chat_app();
    app.ai.suggestions = vec![Suggestion {
        query: ".value".to_string(),
        description: "d".to_string(),
        suggestion_type: SuggestionType::Query,
    }];

    app.handle_key_event(key_with_mods(KeyCode::Char('1'), KeyModifiers::ALT));

    assert_eq!(app.query(), ".value");
    assert_eq!(app.focus, Focus::AiChat, "focus stays in the chat");
}

#[test]
fn alt_navigation_then_enter_applies_instead_of_sending() {
    let (mut app, rx) = chat_app();
    app.ai.suggestions = vec![Suggestion {
        query: ".value".to_string(),
        description: "d".to_string(),
        suggestion_type: SuggestionType::Fix,
    }];
    type_text(&mut app, "pending question");

    app.handle_key_event(key_with_mods(KeyCode::Down, KeyModifiers::ALT));
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(app.query(), ".value");
    assert!(rx.try_recv().is_err(), "no chat request was sent");
    assert_eq!(
        app.ai.chat_input_text(),
        "pending question",
        "typed text survives"
    );
}

#[test]
fn ctrl_t_and_backtab_move_focus_to_results_pane() {
    let (mut app, _rx) = chat_app();
    app.handle_key_event(key_with_mods(KeyCode::Char('t'), KeyModifiers::CONTROL));
    assert_eq!(app.focus, Focus::ResultsPane);
    assert!(!app.ai.visible, "results pane hides the popup as before");

    app.handle_key_event(key_with_mods(KeyCode::Char('t'), KeyModifiers::CONTROL));
    assert_eq!(app.focus, Focus::InputField);
    assert!(app.ai.visible, "popup comes back with the input field");

    assert!(app.focus_ai_chat());
    app.handle_key_event(key(KeyCode::BackTab));
    assert_eq!(app.focus, Focus::ResultsPane);
}

#[test]
fn ctrl_q_still_exits_with_query_output() {
    let (mut app, _rx) = chat_app();
    app.handle_key_event(key_with_mods(KeyCode::Char('q'), KeyModifiers::CONTROL));
    assert!(app.should_quit);
}

#[test]
fn up_down_scroll_the_conversation() {
    let (mut app, _rx) = chat_app();
    app.ai.selection.update_layout_with_header(20, vec![], 5);
    let start = app.ai.selection.scroll_offset_u16();
    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.ai.selection.scroll_offset_u16(), start + 1);
    app.handle_key_event(key(KeyCode::Up));
    assert_eq!(app.ai.selection.scroll_offset_u16(), start);
    app.handle_key_event(key(KeyCode::PageDown));
    assert_eq!(app.ai.selection.scroll_offset_u16(), start + 5);
    app.handle_key_event(key(KeyCode::PageUp));
    assert_eq!(app.ai.selection.scroll_offset_u16(), start);
}

#[test]
fn tab_is_swallowed_without_inserting() {
    let (mut app, _rx) = chat_app();
    app.handle_key_event(key(KeyCode::Tab));
    assert_eq!(app.ai.chat_input_text(), "");
    assert_eq!(app.focus, Focus::AiChat);
}

#[test]
fn hidden_popup_with_stale_chat_focus_falls_back_to_query_box() {
    let (mut app, _rx) = chat_app();
    app.ai.visible = false;
    assert!(!handle_ai_chat_key(&mut app, key(KeyCode::Char('x'))));
    assert_eq!(app.focus, Focus::InputField);
}

#[test]
fn paste_goes_into_chat_input_flattened() {
    let (mut app, _rx) = chat_app();
    paste_into_chat(&mut app, "line one\nline two\r\n");
    assert_eq!(app.ai.chat_input_text(), "line one line two  ");
    assert_eq!(app.query(), ".name");
}

#[test]
fn pass_through_list_matches_expected_chords() {
    assert!(passes_through_to_global(key_with_mods(
        KeyCode::Char('a'),
        KeyModifiers::CONTROL
    )));
    assert!(passes_through_to_global(key_with_mods(
        KeyCode::Char('t'),
        KeyModifiers::CONTROL
    )));
    assert!(passes_through_to_global(key_with_mods(
        KeyCode::Char('q'),
        KeyModifiers::CONTROL
    )));
    assert!(passes_through_to_global(key(KeyCode::BackTab)));
    assert!(passes_through_to_global(key_with_mods(
        KeyCode::Enter,
        KeyModifiers::SHIFT
    )));
    assert!(passes_through_to_global(key_with_mods(
        KeyCode::Enter,
        KeyModifiers::ALT
    )));
    assert!(!passes_through_to_global(key(KeyCode::Enter)));
    assert!(!passes_through_to_global(key(KeyCode::Char('a'))));
    assert!(!passes_through_to_global(key_with_mods(
        KeyCode::Char('l'),
        KeyModifiers::CONTROL
    )));
}

#[test]
fn clicking_input_field_from_chat_moves_focus_back() {
    let (mut app, _rx) = chat_app();
    app.layout_regions.input_field = Some(ratatui::layout::Rect::new(0, 20, 80, 3));
    let mouse = ratatui::crossterm::event::MouseEvent {
        kind: ratatui::crossterm::event::MouseEventKind::Down(
            ratatui::crossterm::event::MouseButton::Left,
        ),
        column: 5,
        row: 21,
        modifiers: KeyModifiers::NONE,
    };
    crate::app::mouse_click::handle_click(&mut app, Some(crate::layout::Region::InputField), mouse);
    assert_eq!(app.focus, Focus::InputField);
    assert!(app.ai.visible);
}

use crate::autocomplete::{Suggestion, SuggestionType};
use crate::editor::EditorMode;
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_ctrl_s_opens_snippet_popup() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    assert!(!app.snippets.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    assert!(app.snippets.is_visible());
}

#[test]
fn test_esc_closes_snippet_popup() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.handle_key_event(key(KeyCode::Esc));
    assert!(!app.snippets.is_visible());
}

#[test]
fn test_snippet_popup_hides_autocomplete_on_open() {
    let mut app = app_with_query(".a");
    app.input.editor_mode = EditorMode::Insert;
    app.autocomplete
        .update_suggestions(vec![Suggestion::new("test", SuggestionType::Field)]);
    assert!(app.autocomplete.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    assert!(app.snippets.is_visible());
    assert!(!app.autocomplete.is_visible());
}

#[test]
fn test_snippet_popup_closes_history_on_open() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.history.add_entry_in_memory(".test");
    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));
    assert!(app.history.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    assert!(app.snippets.is_visible());
    assert!(!app.history.is_visible());
}

#[test]
fn test_snippet_popup_allows_f1_help() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());
    assert!(!app.help.visible);

    app.handle_key_event(key(KeyCode::F(1)));
    assert!(app.help.visible);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_snippet_popup_allows_question_mark_help() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());
    assert!(!app.help.visible);

    app.handle_key_event(key(KeyCode::Char('?')));
    assert!(app.help.visible);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_snippet_popup_allows_ctrl_c_quit() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());
    assert!(!app.should_quit);

    app.handle_key_event(key_with_mods(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert!(app.should_quit);
}

#[test]
fn test_snippet_popup_blocks_other_global_keys() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Char('a'), KeyModifiers::CONTROL));
    assert!(!app.ai.visible);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_ctrl_s_when_popup_already_open() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());
}

#[test]
fn test_snippet_popup_captures_backtab() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.handle_key_event(key(KeyCode::BackTab));
    assert!(app.snippets.is_visible());
    assert_eq!(app.focus, crate::app::Focus::InputField);
}

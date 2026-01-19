use crate::editor::EditorMode;
use crate::snippets::Snippet;
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_enter_applies_selected_snippet_and_closes_popup() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.snippets.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".foo".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".bar".to_string(),
            description: None,
        },
    ]);

    app.handle_key_event(key(KeyCode::Enter));

    assert!(!app.snippets.is_visible());
    assert_eq!(app.input.query(), ".foo");
}

#[test]
fn test_enter_applies_snippet_after_navigation() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![
        Snippet {
            name: "test1".to_string(),
            query: ".foo".to_string(),
            description: None,
        },
        Snippet {
            name: "test2".to_string(),
            query: ".bar".to_string(),
            description: None,
        },
    ]);

    app.handle_key_event(key(KeyCode::Down));
    assert_eq!(app.snippets.selected_index(), 1);

    app.handle_key_event(key(KeyCode::Enter));

    assert!(!app.snippets.is_visible());
    assert_eq!(app.input.query(), ".bar");
}

#[test]
fn test_enter_replaces_existing_query() {
    let mut app = app_with_query(".existing | query");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".new_query".to_string(),
        description: None,
    }]);

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(app.input.query(), ".new_query");
}

#[test]
fn test_enter_clears_error_overlay() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;
    app.error_overlay_visible = true;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".foo".to_string(),
        description: None,
    }]);

    app.handle_key_event(key(KeyCode::Enter));

    assert!(!app.error_overlay_visible);
}

#[test]
fn test_enter_resets_scroll_position() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;
    app.results_scroll.offset = 100;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![Snippet {
        name: "test".to_string(),
        query: ".foo".to_string(),
        description: None,
    }]);

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(app.results_scroll.offset, 0);
}

#[test]
fn test_enter_with_empty_list_just_closes() {
    let mut app = app_with_query(".existing");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(app.snippets.is_visible());

    app.snippets.set_snippets(vec![]);

    app.handle_key_event(key(KeyCode::Enter));

    assert!(!app.snippets.is_visible());
    assert_eq!(app.input.query(), ".existing");
}

#[test]
fn test_enter_executes_query() {
    let mut app = app_with_query("");
    app.input.editor_mode = EditorMode::Insert;

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));

    app.snippets.set_snippets(vec![Snippet {
        name: "keys query".to_string(),
        query: "keys".to_string(),
        description: Some("Get all keys".to_string()),
    }]);

    app.handle_key_event(key(KeyCode::Enter));

    assert!(app.query.is_some());
    if let Some(ref query_state) = app.query {
        assert_eq!(
            query_state.base_query_for_suggestions,
            Some("keys".to_string())
        );
    }
}

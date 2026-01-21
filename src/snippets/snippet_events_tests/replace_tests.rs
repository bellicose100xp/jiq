use crate::editor::EditorMode;
use crate::snippets::{Snippet, SnippetMode};
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_ctrl_r_enters_replace_mode() {
    let mut app = app_with_query(".new_query");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old_query".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();

    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

    assert!(matches!(
        app.snippets.mode(),
        SnippetMode::ConfirmUpdate { snippet_name, old_query, new_query }
        if snippet_name == "My Snippet" && old_query == ".old_query" && new_query == ".new_query"
    ));
}

#[test]
fn test_ctrl_r_with_no_snippets_does_nothing() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.snippets.on_search_input_changed();

    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
}

#[test]
fn test_ctrl_r_with_identical_query_shows_warning() {
    let mut app = app_with_query(".same");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".same".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();

    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert!(app.notification.current_message().is_some());
}

#[test]
fn test_enter_confirms_replace() {
    let mut app = app_with_query(".new_query");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old_query".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert_eq!(app.snippets.snippets()[0].query, ".new_query");
}

#[test]
fn test_esc_cancels_replace() {
    let mut app = app_with_query(".new_query");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old_query".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Esc));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert_eq!(app.snippets.snippets()[0].query, ".old_query");
    assert!(app.snippets.is_visible());
}

#[test]
fn test_other_keys_ignored_in_confirm_replace_mode() {
    let mut app = app_with_query(".new_query");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old_query".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('a')));
    app.handle_key_event(key(KeyCode::Char('y')));
    app.handle_key_event(key(KeyCode::Char('n')));
    app.handle_key_event(key(KeyCode::Up));
    app.handle_key_event(key(KeyCode::Down));

    assert!(matches!(
        app.snippets.mode(),
        SnippetMode::ConfirmUpdate { .. }
    ));
    assert_eq!(app.snippets.snippets()[0].query, ".old_query");
}

#[test]
fn test_replace_preserves_other_snippet_fields() {
    let mut app = app_with_query(".new_query");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old_query".to_string(),
        description: Some("A description".to_string()),
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(app.snippets.snippets()[0].name, "My Snippet");
    assert_eq!(app.snippets.snippets()[0].query, ".new_query");
    assert_eq!(
        app.snippets.snippets()[0].description,
        Some("A description".to_string())
    );
}

#[test]
fn test_is_editing_false_in_confirm_replace_mode() {
    let mut app = app_with_query(".new_query");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".old_query".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();

    assert!(!app.snippets.is_editing());
    app.handle_key_event(key_with_mods(KeyCode::Char('r'), KeyModifiers::CONTROL));
    assert!(!app.snippets.is_editing());
}

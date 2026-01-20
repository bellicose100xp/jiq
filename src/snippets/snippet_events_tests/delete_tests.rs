use crate::editor::EditorMode;
use crate::snippets::{Snippet, SnippetMode};
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_d_key_enters_delete_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();

    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));

    assert!(matches!(
        app.snippets.mode(),
        SnippetMode::ConfirmDelete { snippet_name } if snippet_name == "My Snippet"
    ));
}

#[test]
fn test_d_key_with_no_snippets_does_nothing() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.snippets.on_search_input_changed();

    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
}

#[test]
fn test_enter_confirms_delete() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert!(app.snippets.snippets().is_empty());
}

#[test]
fn test_esc_cancels_delete() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Esc));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert_eq!(app.snippets.snippets().len(), 1);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_other_keys_ignored_in_confirm_delete_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));

    app.handle_key_event(key(KeyCode::Char('a')));
    app.handle_key_event(key(KeyCode::Char('y')));
    app.handle_key_event(key(KeyCode::Char('n')));
    app.handle_key_event(key(KeyCode::Up));
    app.handle_key_event(key(KeyCode::Down));

    assert!(matches!(
        app.snippets.mode(),
        SnippetMode::ConfirmDelete { .. }
    ));
    assert_eq!(app.snippets.snippets().len(), 1);
}

#[test]
fn test_delete_adjusts_selection_when_deleting_last() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![
        Snippet {
            name: "First".to_string(),
            query: ".first".to_string(),
            description: None,
        },
        Snippet {
            name: "Second".to_string(),
            query: ".second".to_string(),
            description: None,
        },
    ]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key(KeyCode::Down));
    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(app.snippets.selected_index(), 0);
    assert_eq!(app.snippets.snippets().len(), 1);
    assert_eq!(app.snippets.snippets()[0].name, "First");
}

#[test]
fn test_is_editing_false_in_confirm_delete_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "My Snippet".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();

    assert!(!app.snippets.is_editing());
    app.handle_key_event(key_with_mods(KeyCode::Char('d'), KeyModifiers::CONTROL));
    assert!(!app.snippets.is_editing());
}

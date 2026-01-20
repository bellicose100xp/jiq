use crate::editor::EditorMode;
use crate::snippets::{Snippet, SnippetMode};
use crate::test_utils::test_helpers::{app_with_query, key, key_with_mods};
use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn test_e_key_enters_edit_name_mode() {
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

    app.handle_key_event(key(KeyCode::Char('e')));

    assert!(matches!(
        app.snippets.mode(),
        SnippetMode::EditName { original_name } if original_name == "My Snippet"
    ));
}

#[test]
fn test_e_key_with_no_snippets_does_nothing() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![]);
    app.snippets.on_search_input_changed();

    app.handle_key_event(key(KeyCode::Char('e')));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
}

#[test]
fn test_esc_in_edit_name_mode_cancels() {
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
    app.handle_key_event(key(KeyCode::Char('e')));

    app.handle_key_event(key(KeyCode::Esc));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert!(app.snippets.is_visible());
}

#[test]
fn test_typing_in_edit_name_mode_updates_name() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "Old".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key(KeyCode::Char('e')));

    for _ in 0..3 {
        app.handle_key_event(key(KeyCode::Backspace));
    }
    app.handle_key_event(key(KeyCode::Char('N')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('w')));

    assert_eq!(app.snippets.name_input(), "New");
}

#[test]
fn test_enter_in_edit_name_mode_saves_and_exits() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "Old".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key(KeyCode::Char('e')));

    for _ in 0..3 {
        app.handle_key_event(key(KeyCode::Backspace));
    }
    app.handle_key_event(key(KeyCode::Char('N')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('w')));
    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert_eq!(app.snippets.snippets()[0].name, "New");
}

#[test]
fn test_edit_name_empty_shows_error() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "Old".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key(KeyCode::Char('e')));

    for _ in 0..3 {
        app.handle_key_event(key(KeyCode::Backspace));
    }
    app.handle_key_event(key(KeyCode::Enter));

    assert!(matches!(app.snippets.mode(), SnippetMode::EditName { .. }));
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("empty"));
}

#[test]
fn test_edit_name_duplicate_shows_error() {
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
    app.handle_key_event(key(KeyCode::Char('e')));

    for _ in 0..5 {
        app.handle_key_event(key(KeyCode::Backspace));
    }
    app.handle_key_event(key(KeyCode::Char('S')));
    app.handle_key_event(key(KeyCode::Char('e')));
    app.handle_key_event(key(KeyCode::Char('c')));
    app.handle_key_event(key(KeyCode::Char('o')));
    app.handle_key_event(key(KeyCode::Char('n')));
    app.handle_key_event(key(KeyCode::Char('d')));
    app.handle_key_event(key(KeyCode::Enter));

    assert!(matches!(app.snippets.mode(), SnippetMode::EditName { .. }));
    assert!(app.notification.current().is_some());
    let notification = app.notification.current().unwrap();
    assert!(notification.message.contains("already exists"));
}

#[test]
fn test_edit_same_name_succeeds() {
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
    app.handle_key_event(key(KeyCode::Char('e')));

    app.handle_key_event(key(KeyCode::Enter));

    assert_eq!(*app.snippets.mode(), SnippetMode::Browse);
    assert_eq!(app.snippets.snippets()[0].name, "My Snippet");
}

#[test]
fn test_is_editing_true_in_edit_name_mode() {
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

    app.handle_key_event(key(KeyCode::Char('e')));
    assert!(app.snippets.is_editing());
}

#[test]
fn test_question_mark_blocked_in_edit_name_mode() {
    let mut app = app_with_query(".test");
    app.input.editor_mode = EditorMode::Insert;
    app.snippets.disable_persistence();

    app.handle_key_event(key_with_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    app.snippets.set_snippets(vec![Snippet {
        name: "Old".to_string(),
        query: ".test".to_string(),
        description: None,
    }]);
    app.snippets.on_search_input_changed();
    app.handle_key_event(key(KeyCode::Char('e')));

    app.handle_key_event(key(KeyCode::Char('?')));

    assert!(!app.help.visible);
    assert!(app.snippets.name_input().contains('?'));
}
